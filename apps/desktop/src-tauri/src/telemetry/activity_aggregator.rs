use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::errors::{AppError, Result};
use crate::persistence::Database;
use crate::telemetry::activity_resolver::{ActivityIdentity, ActivityResolver};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityUsageItem {
    pub id: String,
    pub work_session_id: String,
    pub employee_id: String,
    pub activity_name: String,
    pub display_name: String,
    pub category: String,
    pub source: String, // "desktop_application" | "browser"
    pub duration_seconds: i64,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

pub struct ActiveInterval {
    pub identity: ActivityIdentity,
    pub interval_start: DateTime<Utc>,
}

pub struct ActivityAggregator {
    db: Database,
    resolver: ActivityResolver,
    active_app_interval: Option<ActiveInterval>,
    active_web_interval: Option<ActiveInterval>,
    simulated_app: Option<String>,
    simulated_domain: Option<String>,
}

impl ActivityAggregator {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            resolver: ActivityResolver::new(),
            active_app_interval: None,
            active_web_interval: None,
            simulated_app: None,
            simulated_domain: None,
        }
    }

    pub fn set_simulated_app(&mut self, exec_name: Option<String>) {
        self.simulated_app = exec_name;
    }

    pub fn set_simulated_domain(&mut self, domain: Option<String>) {
        self.simulated_domain = domain;
    }

    pub fn update_foreground_application(
        &mut self,
        work_session_id: &str,
        employee_id: &str,
        exec_name: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let target_exec = self.simulated_app.as_deref().unwrap_or(exec_name);
        let new_identity = self.resolver.resolve_application(target_exec);

        // If current app is the same, keep interval going
        if let Some(ref current) = self.active_app_interval {
            if current.identity.identifier == new_identity.identifier {
                return Ok(());
            }
        }

        // Close previous app interval and flush duration
        self.flush_active_app_interval(work_session_id, employee_id, now)?;

        // Open new app interval
        self.active_app_interval = Some(ActiveInterval {
            identity: new_identity,
            interval_start: now,
        });

        Ok(())
    }

    pub fn update_active_website(
        &mut self,
        work_session_id: &str,
        employee_id: &str,
        domain_input: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let target_domain = self.simulated_domain.as_deref().unwrap_or(domain_input);
        let new_identity = self.resolver.resolve_website(target_domain);

        if let Some(ref current) = self.active_web_interval {
            if current.identity.identifier == new_identity.identifier {
                return Ok(());
            }
        }

        self.flush_active_web_interval(work_session_id, employee_id, now)?;

        self.active_web_interval = Some(ActiveInterval {
            identity: new_identity,
            interval_start: now,
        });

        Ok(())
    }

    pub fn flush_all(&mut self, work_session_id: &str, employee_id: &str) -> Result<()> {
        let now = Utc::now();
        self.flush_active_app_interval(work_session_id, employee_id, now)?;
        self.flush_active_web_interval(work_session_id, employee_id, now)?;
        Ok(())
    }

    pub fn pause_telemetry(&mut self, work_session_id: &str, employee_id: &str) -> Result<()> {
        self.flush_all(work_session_id, employee_id)?;
        self.active_app_interval = None;
        self.active_web_interval = None;
        Ok(())
    }

    /// Resume telemetry after an automatic break ends.
    /// Simply resets intervals so the next foreground/website update starts fresh.
    pub fn resume_telemetry(&mut self, _work_session_id: &str, _employee_id: &str) -> Result<()> {
        // Intervals were already cleared by pause_telemetry.
        // The next call to update_foreground_application / update_active_website
        // will naturally open a new interval. Nothing else needed.
        Ok(())
    }

    fn flush_active_app_interval(
        &mut self,
        work_session_id: &str,
        employee_id: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        if let Some(interval) = self.active_app_interval.take() {
            let duration_seconds = (now - interval.interval_start).num_seconds().max(0);
            if duration_seconds >= 0 {
                self.record_activity_duration(
                    work_session_id,
                    employee_id,
                    &interval.identity,
                    duration_seconds,
                    &interval.interval_start.to_rfc3339(),
                    &now.to_rfc3339(),
                )?;
            }
        }
        Ok(())
    }

    fn flush_active_web_interval(
        &mut self,
        work_session_id: &str,
        employee_id: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        if let Some(interval) = self.active_web_interval.take() {
            let duration_seconds = (now - interval.interval_start).num_seconds().max(0);
            if duration_seconds >= 0 {
                self.record_activity_duration(
                    work_session_id,
                    employee_id,
                    &interval.identity,
                    duration_seconds,
                    &interval.interval_start.to_rfc3339(),
                    &now.to_rfc3339(),
                )?;
            }
        }
        Ok(())
    }

    fn record_activity_duration(
        &self,
        work_session_id: &str,
        employee_id: &str,
        identity: &ActivityIdentity,
        added_duration_seconds: i64,
        first_seen_at: &str,
        last_seen_at: &str,
    ) -> Result<()> {
        let conn = self.db.get_connection()?;

        // 1. Ensure category exists in activity_categories table
        let mut cat_stmt = conn
            .prepare("SELECT id FROM activity_categories WHERE type = ?1 AND identifier = ?2")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let category_id: String = match cat_stmt.query_row([&identity.activity_type, &identity.identifier], |row| row.get(0)) {
            Ok(id) => id,
            Err(_) => {
                let new_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT INTO activity_categories (id, name, type, identifier, display_name, is_active, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, 1, ?6)",
                    [
                        &new_id,
                        &identity.display_name,
                        &identity.activity_type,
                        &identity.identifier,
                        &identity.display_name,
                        first_seen_at,
                    ],
                )
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
                new_id
            }
        };

        // 2. Upsert aggregate in activity_usage table
        let mut usage_stmt = conn
            .prepare("SELECT id, duration_seconds FROM activity_usage WHERE work_session_id = ?1 AND activity_category_id = ?2 AND source = ?3")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let existing_usage = usage_stmt
            .query_row([work_session_id, &category_id, &identity.source], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .ok();

        if let Some((usage_id, existing_dur)) = existing_usage {
            let updated_dur = existing_dur + added_duration_seconds;
            conn.execute(
                "UPDATE activity_usage SET duration_seconds = ?1, last_seen_at = ?2, updated_at = ?3 WHERE id = ?4",
                [&updated_dur.to_string(), last_seen_at, last_seen_at, &usage_id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        } else {
            let usage_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO activity_usage (id, work_session_id, employee_id, activity_category_id, source, duration_seconds, first_seen_at, last_seen_at, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                [
                    &usage_id,
                    work_session_id,
                    employee_id,
                    &category_id,
                    &identity.source,
                    &added_duration_seconds.to_string(),
                    first_seen_at,
                    last_seen_at,
                    first_seen_at,
                    last_seen_at,
                ],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    pub fn get_shift_activity_summary(&self, work_session_id: &str) -> Result<Vec<ActivityUsageItem>> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare(
                "SELECT u.id, u.work_session_id, u.employee_id, c.identifier, c.display_name, c.name, u.source, u.duration_seconds, u.first_seen_at, u.last_seen_at
                 FROM activity_usage u
                 JOIN activity_categories c ON u.activity_category_id = c.id
                 WHERE u.work_session_id = ?1
                 ORDER BY u.duration_seconds DESC",
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt
            .query_map([work_session_id], |row| {
                Ok(ActivityUsageItem {
                    id: row.get(0)?,
                    work_session_id: row.get(1)?,
                    employee_id: row.get(2)?,
                    activity_name: row.get(3)?,
                    display_name: row.get(4)?,
                    category: row.get(5)?,
                    source: row.get(6)?,
                    duration_seconds: row.get(7)?,
                    first_seen_at: row.get(8)?,
                    last_seen_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                items.push(item);
            }
        }
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_aggregation_in_sqlite() {
        let db = Database::new_in_memory().unwrap();

        let work_id = "work_100";
        let emp_id = "emp_100";

        // Insert a work_sessions row so the FK constraint is satisfied
        {
            let conn = db.get_connection().unwrap();
            conn.execute(
                "INSERT INTO work_sessions (id, employee_id, device_id, started_at, ended_at, status, created_at)
                 VALUES (?1, ?2, 'dev_1', '2025-01-01T00:00:00Z', NULL, 'active', '2025-01-01T00:00:00Z')",
                [work_id, emp_id],
            ).unwrap();
        }

        let mut aggregator = ActivityAggregator::new(db);

        // Record VS Code usage, then switch to Word
        aggregator.update_foreground_application(work_id, emp_id, "Code.exe").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        aggregator.update_foreground_application(work_id, emp_id, "WINWORD.EXE").unwrap();

        let summary = aggregator.get_shift_activity_summary(work_id).unwrap();
        assert!(!summary.is_empty());
        assert_eq!(summary[0].activity_name, "Code.exe");
    }
}
