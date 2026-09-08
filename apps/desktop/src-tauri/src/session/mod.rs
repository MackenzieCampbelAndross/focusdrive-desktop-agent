use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::errors::{AppError, Result};
use crate::persistence::Database;
use crate::state::{StateMachine, SessionState};
use crate::sync::queue::{SyncQueue, SyncPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    pub id: String,
    pub employee_id: String,
    pub device_id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakSession {
    pub id: String,
    pub work_session_id: String,
    pub employee_id: String,
    pub break_type: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub work_session_id: String,
    pub employee_id: String,
    pub started_at: String,
    pub ended_at: String,
    pub total_shift_duration_seconds: i64,
    pub manual_break_duration_seconds: i64,
    pub effective_working_duration_seconds: i64,
}

pub struct SessionManager {
    db: Database,
    state_machine: StateMachine,
    active_session: Option<WorkSession>,
    active_break: Option<BreakSession>,
    last_summary: Option<SessionSummary>,
    is_recovered_session: bool,
}

impl SessionManager {
    pub fn new(db: Database) -> Self {
        let mut manager = Self {
            db,
            state_machine: StateMachine::new(SessionState::LoggedOut),
            active_session: None,
            active_break: None,
            last_summary: None,
            is_recovered_session: false,
        };
        // Attempt session recovery from SQLite (without fabricating auth)
        let _ = manager.recover_active_session();
        manager
    }

    pub fn current_state(&self) -> SessionState {
        self.state_machine.current_state()
    }

    pub fn is_recovered(&self) -> bool {
        self.is_recovered_session
    }

    pub fn active_session(&self) -> Option<&WorkSession> {
        self.active_session.as_ref()
    }

    pub fn active_break(&self) -> Option<&BreakSession> {
        self.active_break.as_ref()
    }

    pub fn last_summary(&self) -> Option<&SessionSummary> {
        self.last_summary.as_ref()
    }

    pub fn set_logged_in(&mut self) -> Result<()> {
        if self.state_machine.current_state() == SessionState::LoggedOut {
            self.state_machine.transition_to(SessionState::ShiftReady)?;
        }
        Ok(())
    }

    pub fn set_logged_out(&mut self) -> Result<()> {
        if self.active_session.is_some() {
            return Err(AppError::SessionError(
                "Cannot log out while a shift is active. End your shift first.".to_string(),
            ));
        }
        if self.state_machine.can_transition_to(SessionState::LoggedOut) {
            self.state_machine.transition_to(SessionState::LoggedOut)?;
        }
        Ok(())
    }

    pub fn start_shift(&mut self, employee_id: &str, device_id: &str) -> Result<WorkSession> {
        if self.active_session.is_some() {
            return Err(AppError::SessionError(
                "A work shift is already active.".to_string(),
            ));
        }

        self.state_machine.transition_to(SessionState::Working)?;

        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let session = WorkSession {
            id: session_id.clone(),
            employee_id: employee_id.to_string(),
            device_id: device_id.to_string(),
            started_at: now.clone(),
            ended_at: None, // SQLite NULL
            status: "active".to_string(),
            created_at: now.clone(),
        };

        // Persist to SQLite with NULL ended_at
        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "INSERT INTO work_sessions (id, employee_id, device_id, started_at, ended_at, status, created_at)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
                [
                    &session.id,
                    &session.employee_id,
                    &session.device_id,
                    &session.started_at,
                    &session.status,
                    &session.created_at,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        self.active_session = Some(session.clone());
        self.is_recovered_session = false;

        // Lifecycle hook
        self.on_shift_start(&session);

        Ok(session)
    }

    pub fn start_manual_break(&mut self) -> Result<BreakSession> {
        let session = self.active_session.as_ref().ok_or_else(|| {
            AppError::SessionError("Cannot start break: No active work shift.".to_string())
        })?;

        if self.active_break.is_some() {
            return Err(AppError::SessionError("A break is already active.".to_string()));
        }

        self.state_machine.transition_to(SessionState::ManualBreak)?;

        let break_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let break_session = BreakSession {
            id: break_id,
            work_session_id: session.id.clone(),
            employee_id: session.employee_id.clone(),
            break_type: "manual".to_string(),
            started_at: now.clone(),
            ended_at: None, // SQLite NULL
            duration_seconds: 0,
            created_at: now.clone(),
        };

        // Persist to SQLite with NULL ended_at
        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "INSERT INTO break_sessions (id, work_session_id, employee_id, type, started_at, ended_at, duration_seconds, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6)",
                [
                    &break_session.id,
                    &break_session.work_session_id,
                    &break_session.employee_id,
                    &break_session.break_type,
                    &break_session.started_at,
                    &break_session.created_at,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        self.active_break = Some(break_session.clone());

        // Lifecycle hook
        self.on_break_start(&break_session);

        Ok(break_session)
    }

    /// Start an automatic idle break. Called by the telemetry loop when
    /// idle time >= 60 seconds while in Working state.
    /// The break_type is "automatic_idle". Duration is computed when ended.
    pub fn start_automatic_break(&mut self) -> Result<BreakSession> {
        let session = self.active_session.as_ref().ok_or_else(|| {
            AppError::SessionError("Cannot start automatic break: No active work shift.".to_string())
        })?;

        if self.active_break.is_some() {
            return Err(AppError::SessionError("A break is already active.".to_string()));
        }

        self.state_machine.transition_to(SessionState::AutomaticBreak)?;

        let break_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let break_session = BreakSession {
            id: break_id,
            work_session_id: session.id.clone(),
            employee_id: session.employee_id.clone(),
            break_type: "automatic_idle".to_string(),
            started_at: now.clone(),
            ended_at: None,
            duration_seconds: 0,
            created_at: now.clone(),
        };

        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "INSERT INTO break_sessions (id, work_session_id, employee_id, type, started_at, ended_at, duration_seconds, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6)",
                [
                    &break_session.id,
                    &break_session.work_session_id,
                    &break_session.employee_id,
                    &break_session.break_type,
                    &break_session.started_at,
                    &break_session.created_at,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        self.active_break = Some(break_session.clone());

        // Lifecycle hook
        self.on_break_start(&break_session);

        tracing::info!("Automatic idle break started");

        Ok(break_session)
    }

    /// End an automatic idle break. Called by the telemetry loop when
    /// idle time drops below 60 seconds while in AutomaticBreak state.
    /// Duration = total_break_seconds - 60 (first 60s not counted).
    pub fn end_automatic_break(&mut self) -> Result<()> {
        let active_break = self.active_break.as_ref().ok_or_else(|| {
            AppError::SessionError("Cannot end automatic break: No active break session.".to_string())
        })?;

        self.state_machine.transition_to(SessionState::Working)?;

        let now_dt = Utc::now();
        let now_str = now_dt.to_rfc3339();

        let start_dt = DateTime::parse_from_rfc3339(&active_break.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now_dt);

        let total_break_seconds = (now_dt - start_dt).num_seconds().max(0);
        // First 60 seconds are NOT counted as break time
        let effective_duration = (total_break_seconds - 60).max(0);

        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "UPDATE break_sessions SET ended_at = ?1, duration_seconds = ?2 WHERE id = ?3",
                [&now_str, &effective_duration.to_string(), &active_break.id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        let mut completed_break = active_break.clone();
        completed_break.ended_at = Some(now_str);
        completed_break.duration_seconds = effective_duration;

        self.active_break = None;

        // Lifecycle hook
        self.on_break_end(&completed_break);

        tracing::info!("Automatic idle break ended. Effective duration: {}s", effective_duration);

        Ok(())
    }

    pub fn resume_work(&mut self) -> Result<()> {
        let active_break = self.active_break.as_ref().ok_or_else(|| {
            AppError::SessionError("Cannot resume work: No active break session.".to_string())
        })?;

        self.state_machine.transition_to(SessionState::Working)?;

        let now_dt = Utc::now();
        let now_str = now_dt.to_rfc3339();

        let start_dt = DateTime::parse_from_rfc3339(&active_break.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now_dt);

        let duration = (now_dt - start_dt).num_seconds().max(0);

        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "UPDATE break_sessions SET ended_at = ?1, duration_seconds = ?2 WHERE id = ?3",
                [&now_str, &duration.to_string(), &active_break.id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        let mut completed_break = active_break.clone();
        completed_break.ended_at = Some(now_str);
        completed_break.duration_seconds = duration;

        self.active_break = None;

        // Lifecycle hook for resumed work
        self.on_break_end(&completed_break);

        Ok(())
    }

    /// Internal helper for ending active break during shift finalization without calling resume_work()
    fn end_break_for_shift_completion(&mut self) -> Result<BreakSession> {
        let active_break = self.active_break.as_ref().ok_or_else(|| {
            AppError::SessionError("No active break to close for shift completion.".to_string())
        })?;

        let now_dt = Utc::now();
        let now_str = now_dt.to_rfc3339();

        let start_dt = DateTime::parse_from_rfc3339(&active_break.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now_dt);

        let duration = (now_dt - start_dt).num_seconds().max(0);

        {
            let conn = self.db.get_connection()?;
            conn.execute(
                "UPDATE break_sessions SET ended_at = ?1, duration_seconds = ?2 WHERE id = ?3",
                [&now_str, &duration.to_string(), &active_break.id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        let mut completed_break = active_break.clone();
        completed_break.ended_at = Some(now_str);
        completed_break.duration_seconds = duration;

        self.active_break = None;

        // Fire distinct lifecycle hook for break termination due to shift completion
        self.on_break_end_for_shift_completion(&completed_break);

        Ok(completed_break)
    }

    pub fn end_shift(&mut self) -> Result<SessionSummary> {
        if self.active_session.is_none() {
            return Err(AppError::SessionError(
                "Cannot end shift: No active work shift.".to_string(),
            ));
        }

        // If currently on break, execute explicit break finalization for shift completion
        if self.active_break.is_some() {
            self.end_break_for_shift_completion()?;
        }

        let session = self.active_session.as_ref().unwrap().clone();

        self.state_machine.transition_to(SessionState::ShiftCompleting)?;

        let now_dt = Utc::now();
        let now_str = now_dt.to_rfc3339();

        let start_dt = DateTime::parse_from_rfc3339(&session.started_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or(now_dt);

        let total_shift_duration = (now_dt - start_dt).num_seconds().max(0);

        // Sum duration of all completed break sessions and update work session row
        let (total_break_duration, effective_working_duration) = {
            let conn = self.db.get_connection()?;
            let mut stmt = conn
                .prepare("SELECT SUM(duration_seconds) FROM break_sessions WHERE work_session_id = ?1 AND ended_at IS NOT NULL")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            let total_break: i64 = stmt
                .query_row([&session.id], |row| Ok(row.get(0).unwrap_or(0)))
                .unwrap_or(0);

            let effective = (total_shift_duration - total_break).max(0);

            // Update work session row with real timestamp (not NULL)
            conn.execute(
                "UPDATE work_sessions SET ended_at = ?1, status = 'completed' WHERE id = ?2",
                [&now_str, &session.id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            (total_break, effective)
        };

        let summary = SessionSummary {
            work_session_id: session.id.clone(),
            employee_id: session.employee_id.clone(),
            started_at: session.started_at.clone(),
            ended_at: now_str,
            total_shift_duration_seconds: total_shift_duration,
            manual_break_duration_seconds: total_break_duration,
            effective_working_duration_seconds: effective_working_duration,
        };

        self.last_summary = Some(summary.clone());
        self.active_session = None;
        self.is_recovered_session = false;

        self.state_machine.transition_to(SessionState::ShiftSummary)?;

        // Lifecycle hook
        self.on_shift_end(&summary);

        Ok(summary)
    }

    pub fn dismiss_summary(&mut self) -> Result<()> {
        if self.state_machine.current_state() == SessionState::ShiftSummary {
            self.state_machine.transition_to(SessionState::ShiftReady)?;
        }
        Ok(())
    }

    pub fn recover_active_session(&mut self) -> Result<bool> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT id, employee_id, device_id, started_at, ended_at, status, created_at FROM work_sessions WHERE status = 'active' AND ended_at IS NULL ORDER BY created_at DESC LIMIT 1")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let active_sess = stmt
            .query_row([], |row| {
                Ok(WorkSession {
                    id: row.get(0)?,
                    employee_id: row.get(1)?,
                    device_id: row.get(2)?,
                    started_at: row.get(3)?,
                    ended_at: row.get(4)?, // Should be None / NULL
                    status: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })
            .ok();

        if let Some(session) = active_sess {
            // Query active break session where ended_at IS NULL
            let mut break_stmt = conn
                .prepare("SELECT id, work_session_id, employee_id, type, started_at, ended_at, duration_seconds, created_at FROM break_sessions WHERE work_session_id = ?1 AND ended_at IS NULL LIMIT 1")
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            let active_brk = break_stmt
                .query_row([&session.id], |row| {
                    Ok(BreakSession {
                        id: row.get(0)?,
                        work_session_id: row.get(1)?,
                        employee_id: row.get(2)?,
                        break_type: row.get(3)?,
                        started_at: row.get(4)?,
                        ended_at: row.get(5)?,
                        duration_seconds: row.get(6)?,
                        created_at: row.get(7)?,
                    })
                })
                .ok();

            self.active_session = Some(session);
            self.active_break = active_brk;
            self.is_recovered_session = true;

            if self.active_break.is_some() {
                self.state_machine = StateMachine::new(SessionState::ManualBreak);
            } else {
                self.state_machine = StateMachine::new(SessionState::Working);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    // Telemetry lifecycle hooks
    fn on_shift_start(&self, session: &WorkSession) {
        tracing::info!("Telemetry Lifecycle Hook: Shift Started");
        let queue = SyncQueue::new(self.db.clone());
        let _ = queue.enqueue(&SyncPayload::WorkSession(session.clone()));
    }

    fn on_break_start(&self, break_session: &BreakSession) {
        tracing::info!("Telemetry Lifecycle Hook: Break Started");
        let queue = SyncQueue::new(self.db.clone());
        let _ = queue.enqueue(&SyncPayload::BreakSession(break_session.clone()));
    }

    fn on_break_end(&self, break_session: &BreakSession) {
        tracing::info!("Telemetry Lifecycle Hook: Break Resumed (Employee returned to work)");
        let queue = SyncQueue::new(self.db.clone());
        let _ = queue.enqueue(&SyncPayload::BreakSession(break_session.clone()));
    }

    fn on_break_end_for_shift_completion(&self, break_session: &BreakSession) {
        tracing::info!("Telemetry Lifecycle Hook: Break Closed directly for Shift Finalization");
        let queue = SyncQueue::new(self.db.clone());
        let _ = queue.enqueue(&SyncPayload::BreakSession(break_session.clone()));
    }

    fn on_shift_end(&self, summary: &SessionSummary) {
        tracing::info!("Telemetry Lifecycle Hook: Shift Finalized");
        let queue = SyncQueue::new(self.db.clone());
        let _ = queue.enqueue(&SyncPayload::SessionSummary(summary.clone()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqlite_null_semantics() {
        let db = Database::new_in_memory().unwrap();
        let mut sm = SessionManager::new(db.clone());

        sm.set_logged_in().unwrap();
        let session = sm.start_shift("emp_1", "dev_1").unwrap();
        assert!(session.ended_at.is_none());

        // Verify SQLite table directly has NULL for ended_at without holding mutex guard
        {
            let conn = db.get_connection().unwrap();
            let ended: Option<String> = conn
                .query_row("SELECT ended_at FROM work_sessions WHERE id = ?1", [&session.id], |row| row.get(0))
                .unwrap();
            assert!(ended.is_none());
        }

        let summary = sm.end_shift().unwrap();
        assert!(!summary.ended_at.is_empty());

        {
            let conn = db.get_connection().unwrap();
            let ended_completed: Option<String> = conn
                .query_row("SELECT ended_at FROM work_sessions WHERE id = ?1", [&session.id], |row| row.get(0))
                .unwrap();
            assert!(ended_completed.is_some());
        }
    }

    #[test]
    fn test_end_shift_during_break_semantics() {
        let db = Database::new_in_memory().unwrap();
        let mut sm = SessionManager::new(db);

        sm.set_logged_in().unwrap();
        sm.start_shift("emp_1", "dev_1").unwrap();
        sm.start_manual_break().unwrap();

        // End shift while on break - should not call resume_work()
        let summary = sm.end_shift().unwrap();
        assert_eq!(sm.current_state(), SessionState::ShiftSummary);
        assert_eq!(summary.employee_id, "emp_1");
        assert!(summary.manual_break_duration_seconds >= 0);
    }

    #[test]
    fn test_multi_break_time_accounting() {
        let db = Database::new_in_memory().unwrap();
        let mut sm = SessionManager::new(db);

        sm.set_logged_in().unwrap();
        sm.start_shift("emp_1", "dev_1").unwrap();

        // First break
        sm.start_manual_break().unwrap();
        sm.resume_work().unwrap();

        // Second break
        sm.start_manual_break().unwrap();
        sm.resume_work().unwrap();

        let summary = sm.end_shift().unwrap();
        assert_eq!(summary.total_shift_duration_seconds, summary.manual_break_duration_seconds + summary.effective_working_duration_seconds);
    }

    #[test]
    fn test_restart_recovery_separate_from_auth() {
        let db = Database::new_in_memory().unwrap();
        {
            let mut sm1 = SessionManager::new(db.clone());
            sm1.set_logged_in().unwrap();
            sm1.start_shift("emp_test_99", "dev_1").unwrap();
        }

        // Restart application - recovery recovers session, but does NOT create auth user
        let sm2 = SessionManager::new(db);
        assert_eq!(sm2.current_state(), SessionState::Working);
        assert!(sm2.is_recovered());
        assert_eq!(sm2.active_session().unwrap().employee_id, "emp_test_99");
    }
}
