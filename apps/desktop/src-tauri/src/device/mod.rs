use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use crate::errors::{AppError, Result};
use crate::persistence::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub operating_system: String,
    pub agent_version: String,
    pub created_at: String,
    pub last_seen_at: String,
}

pub struct DeviceManager;

impl DeviceManager {
    pub fn get_or_create_device_info(db: &Database) -> Result<DeviceInfo> {
        let conn = db.get_connection()?;
        let mut stmt = conn
            .prepare("SELECT id, device_name, os, agent_version, created_at, last_seen_at FROM devices LIMIT 1")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let existing = stmt
            .query_row([], |row| {
                Ok(DeviceInfo {
                    device_id: row.get(0)?,
                    device_name: row.get(1)?,
                    operating_system: row.get(2)?,
                    agent_version: row.get(3)?,
                    created_at: row.get(4)?,
                    last_seen_at: row.get(5)?,
                })
            })
            .ok();

        let now = Utc::now().to_rfc3339();

        if let Some(mut dev) = existing {
            // Update last_seen_at
            conn.execute(
                "UPDATE devices SET last_seen_at = ?1 WHERE id = ?2",
                [&now, &dev.device_id],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            dev.last_seen_at = now;
            Ok(dev)
        } else {
            // Create new stable installation UUID
            let new_id = Uuid::new_v4().to_string();
            let computer_name = std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "Windows Workstation".to_string());
            let os_name = "Windows".to_string();
            let agent_ver = env!("CARGO_PKG_VERSION").to_string();

            conn.execute(
                "INSERT INTO devices (id, device_name, os, agent_version, created_at, last_seen_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [&new_id, &computer_name, &os_name, &agent_ver, &now, &now],
            )
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            Ok(DeviceInfo {
                device_id: new_id,
                device_name: computer_name,
                operating_system: os_name,
                agent_version: agent_ver,
                created_at: now.clone(),
                last_seen_at: now,
            })
        }
    }
}
