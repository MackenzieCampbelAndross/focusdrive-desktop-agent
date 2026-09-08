use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};
use std::path::PathBuf;
use crate::errors::{AppError, Result};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    pub fn new_file(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }
        let conn = Connection::open(path)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", &"WAL")
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        // Run any needed migrations
        db.run_migrations()?;
        Ok(db)
    }

    pub fn get_connection(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.get_connection()?;
        
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            
            CREATE TABLE IF NOT EXISTS devices (
                id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                os TEXT NOT NULL,
                agent_version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS app_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS work_sessions (
                id TEXT PRIMARY KEY,
                employee_id TEXT NOT NULL,
                device_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS break_sessions (
                id TEXT PRIMARY KEY,
                work_session_id TEXT NOT NULL,
                employee_id TEXT NOT NULL,
                type TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                duration_seconds INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                FOREIGN KEY (work_session_id) REFERENCES work_sessions(id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS activity_categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                identifier TEXT NOT NULL,
                display_name TEXT NOT NULL,
                is_active INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS activity_usage (
                id TEXT PRIMARY KEY,
                work_session_id TEXT NOT NULL,
                employee_id TEXT NOT NULL,
                activity_category_id TEXT NOT NULL,
                source TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                first_seen_at TEXT NOT NULL,
                last_seen_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (activity_category_id) REFERENCES activity_categories(id) ON DELETE CASCADE,
                FOREIGN KEY (work_session_id) REFERENCES work_sessions(id) ON DELETE CASCADE
            );
            
            CREATE TABLE IF NOT EXISTS sync_queue (
                id TEXT PRIMARY KEY,
                payload_json TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                next_attempt_at TEXT NOT NULL
            );
            "
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    fn run_migrations(&self) -> Result<()> {
        // Placeholder for future migrations; currently schema is up to date.
        // In a real implementation, we would check a schema_version table and apply incremental SQL.
        Ok(())
    }
}
