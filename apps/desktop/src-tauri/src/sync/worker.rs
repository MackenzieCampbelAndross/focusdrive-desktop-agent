use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use crate::errors::{AppError, Result};
use crate::persistence::Database;
use crate::auth::AuthRepository;
use crate::sync::queue::{SyncQueue, SyncPayload, QueuedItem};

pub struct SyncWorker {
    queue: SyncQueue,
    client: Client,
    supabase_url: Option<String>,
    publishable_key: Option<String>,
    _auth_repo: Arc<Mutex<Box<dyn AuthRepository>>>,
}

impl SyncWorker {
    pub fn new(
        db: Database,
        auth_repo: Arc<Mutex<Box<dyn AuthRepository>>>,
        supabase_url: Option<String>,
        publishable_key: Option<String>,
    ) -> Self {
        Self {
            queue: SyncQueue::new(db),
            client: Client::new(),
            supabase_url: supabase_url.map(|u| u.trim_end_matches('/').to_string()),
            publishable_key,
            _auth_repo: auth_repo,
        }
    }

    pub async fn process_queue_batch(&self) -> Result<usize> {
        let items = self.queue.dequeue_ready()?;
        let mut processed = 0;

        for item in items {
            match self.sync_item(&item).await {
                Ok(_) => {
                    self.queue.mark_completed(&item.id)?;
                    processed += 1;
                }
                Err(e) => {
                    tracing::warn!("Sync failed for queue item {}: {}", item.id, e);
                    self.queue.mark_failed(&item.id)?;
                }
            }
        }

        Ok(processed)
    }

    async fn sync_item(&self, item: &QueuedItem) -> Result<()> {
        let (url, key) = match (&self.supabase_url, &self.publishable_key) {
            (Some(u), Some(k)) if !u.is_empty() && !k.is_empty() => (u.as_str(), k.as_str()),
            _ => {
                // Without remote credentials, items remain locally queued
                return Err(AppError::NetworkError("Supabase credentials not configured for sync".to_string()));
            }
        };

        match &item.payload {
            SyncPayload::WorkSession(ws) => {
                let endpoint = format!("{}/rest/v1/work_sessions", url);
                let body = json!({
                    "id": ws.id,
                    "employee_id": ws.employee_id,
                    "device_id": ws.device_id,
                    "started_at": ws.started_at,
                    "ended_at": ws.ended_at,
                    "status": ws.status,
                    "created_at": ws.created_at,
                });

                let res = self.client
                    .post(&endpoint)
                    .header("apikey", key)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .header("Prefer", "resolution=merge-duplicates")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(AppError::NetworkError(format!("WorkSession sync failed: {}", err_text)));
                }
            }

            SyncPayload::BreakSession(bs) => {
                let endpoint = format!("{}/rest/v1/break_sessions", url);
                let body = json!({
                    "id": bs.id,
                    "work_session_id": bs.work_session_id,
                    "employee_id": bs.employee_id,
                    "type": bs.break_type,
                    "started_at": bs.started_at,
                    "ended_at": bs.ended_at,
                    "duration_seconds": bs.duration_seconds,
                    "created_at": bs.created_at,
                });

                let res = self.client
                    .post(&endpoint)
                    .header("apikey", key)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .header("Prefer", "resolution=merge-duplicates")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(AppError::NetworkError(format!("BreakSession sync failed: {}", err_text)));
                }
            }

            SyncPayload::SessionSummary(sum) => {
                // Updates the work session with ended_at and status completed
                let endpoint = format!("{}/rest/v1/work_sessions?id=eq.{}", url, sum.work_session_id);
                let body = json!({
                    "ended_at": sum.ended_at,
                    "status": "completed",
                });

                let res = self.client
                    .patch(&endpoint)
                    .header("apikey", key)
                    .header("Authorization", format!("Bearer {}", key))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| AppError::NetworkError(e.to_string()))?;

                if !res.status().is_success() {
                    let err_text = res.text().await.unwrap_or_default();
                    return Err(AppError::NetworkError(format!("SessionSummary sync failed: {}", err_text)));
                }
            }

            SyncPayload::ActivityUsage(items) => {
                for item in items {
                    // 1. Upsert activity category
                    let cat_endpoint = format!("{}/rest/v1/activity_categories", url);
                    let cat_body = json!({
                        "name": item.activity_name,
                        "type": item.source,
                        "identifier": item.activity_name,
                        "display_name": item.display_name,
                        "is_active": true,
                    });

                    let _ = self.client
                        .post(&cat_endpoint)
                        .header("apikey", key)
                        .header("Authorization", format!("Bearer {}", key))
                        .header("Content-Type", "application/json")
                        .header("Prefer", "resolution=merge-duplicates")
                        .json(&cat_body)
                        .send()
                        .await;
                }
            }
        }

        Ok(())
    }
}

pub fn spawn_sync_worker(
    db: Database,
    auth_repo: Arc<Mutex<Box<dyn AuthRepository>>>,
    supabase_url: Option<String>,
    publishable_key: Option<String>,
    interval_seconds: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let worker = SyncWorker::new(db, auth_repo, supabase_url, publishable_key);
        let interval = Duration::from_secs(interval_seconds.max(5));

        loop {
            tokio::time::sleep(interval).await;
            let _ = worker.process_queue_batch().await;
        }
    })
}