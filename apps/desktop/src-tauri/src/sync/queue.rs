use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};
use crate::errors::{AppError, Result};
use crate::persistence::Database;
use crate::telemetry::activity_aggregator::ActivityUsageItem;
use crate::session::{WorkSession, BreakSession, SessionSummary};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SyncPayload {
    WorkSession(WorkSession),
    BreakSession(BreakSession),
    SessionSummary(SessionSummary),
    ActivityUsage(Vec<ActivityUsageItem>),
}

#[derive(Debug, Clone)]
pub struct QueuedItem {
    pub id: String,
    pub payload: SyncPayload,
    pub retry_count: i64,
    pub next_attempt_at: String,
}

pub struct SyncQueue {
    db: Database,
}

impl SyncQueue {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn enqueue(&self, payload: &SyncPayload) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let payload_json = serde_json::to_string(payload)
            .map_err(|e| AppError::InternalError(e.to_string()))?;
        let now = Utc::now().to_rfc3339();

        let conn = self.db.get_connection()?;
        conn.execute(
            "INSERT INTO sync_queue (id, payload_json, retry_count, next_attempt_at) VALUES (?1, ?2, 0, ?3)",
            [&id, &payload_json, &now],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    pub fn dequeue_ready(&self) -> Result<Vec<QueuedItem>> {
        let conn = self.db.get_connection()?;
        let now = Utc::now().to_rfc3339();

        let mut stmt = conn
            .prepare("SELECT id, payload_json, retry_count, next_attempt_at FROM sync_queue WHERE next_attempt_at <= ?1 ORDER BY retry_count ASC, next_attempt_at ASC LIMIT 50")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let rows = stmt.query_map([&now], |row| {
            let id: String = row.get(0)?;
            let payload_json: String = row.get(1)?;
            let retry_count: i64 = row.get(2)?;
            let next_attempt_at: String = row.get(3)?;
            Ok((id, payload_json, retry_count, next_attempt_at))
        }).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut items = Vec::new();
        for r in rows {
            if let Ok((id, json_str, retry_count, next_attempt_at)) = r {
                if let Ok(payload) = serde_json::from_str::<SyncPayload>(&json_str) {
                    items.push(QueuedItem {
                        id,
                        payload,
                        retry_count,
                        next_attempt_at,
                    });
                }
            }
        }

        Ok(items)
    }

    pub fn mark_completed(&self, id: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        conn.execute("DELETE FROM sync_queue WHERE id = ?1", [id])
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    pub fn mark_failed(&self, id: &str) -> Result<()> {
        let conn = self.db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT retry_count FROM sync_queue WHERE id = ?1")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let current_retries: i64 = stmt.query_row([id], |row| row.get(0)).unwrap_or(0);
        let next_retries = current_retries + 1;

        // Exponential backoff: min(2^retry * 15, 3600) seconds
        let delay_seconds = std::cmp::min((2_i64.pow(std::cmp::min(next_retries as u32, 10))) * 15, 3600);
        let next_attempt = (Utc::now() + Duration::seconds(delay_seconds)).to_rfc3339();

        conn.execute(
            "UPDATE sync_queue SET retry_count = ?1, next_attempt_at = ?2 WHERE id = ?3",
            [&next_retries.to_string(), &next_attempt, id],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub fn queue_length(&self) -> Result<usize> {
        let conn = self.db.get_connection()?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue", [], |row| row.get(0))
            .unwrap_or(0);
        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_queue_lifecycle() {
        let db = Database::new_in_memory().unwrap();
        let queue = SyncQueue::new(db);

        assert_eq!(queue.queue_length().unwrap(), 0);

        let ws = WorkSession {
            id: "ws_1".to_string(),
            employee_id: "emp_1".to_string(),
            device_id: "dev_1".to_string(),
            started_at: "2026-09-08T12:00:00Z".to_string(),
            ended_at: None,
            status: "active".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        };

        let q_id = queue.enqueue(&SyncPayload::WorkSession(ws)).unwrap();
        assert_eq!(queue.queue_length().unwrap(), 1);

        let ready = queue.dequeue_ready().unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, q_id);

        queue.mark_completed(&q_id).unwrap();
        assert_eq!(queue.queue_length().unwrap(), 0);
    }

    #[test]
    fn test_sync_queue_retry_backoff() {
        let db = Database::new_in_memory().unwrap();
        let queue = SyncQueue::new(db);

        let bs = BreakSession {
            id: "bs_1".to_string(),
            work_session_id: "ws_1".to_string(),
            employee_id: "emp_1".to_string(),
            break_type: "manual".to_string(),
            started_at: "2026-09-08T12:00:00Z".to_string(),
            ended_at: None,
            duration_seconds: 0,
            created_at: "2026-09-08T12:00:00Z".to_string(),
        };

        let q_id = queue.enqueue(&SyncPayload::BreakSession(bs)).unwrap();
        queue.mark_failed(&q_id).unwrap();

        let ready = queue.dequeue_ready().unwrap();
        assert_eq!(ready.len(), 0);
        assert_eq!(queue.queue_length().unwrap(), 1);
    }
}