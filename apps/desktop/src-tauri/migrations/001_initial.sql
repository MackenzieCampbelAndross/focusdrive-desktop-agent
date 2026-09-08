-- Migration 001: initial schema for FocusDrive Desktop Agent
-- Version: 1

PRAGMA foreign_keys = ON;

-- Table to track migration version
CREATE TABLE IF NOT EXISTS migration_version (
    version INTEGER PRIMARY KEY
);

-- Devices table
CREATE TABLE IF NOT EXISTS devices (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    os TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Work sessions table
CREATE TABLE IF NOT EXISTS work_sessions (
    id TEXT PRIMARY KEY,
    employee_id TEXT NOT NULL,
    device_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(device_id) REFERENCES devices(id)
);

-- Break sessions table
CREATE TABLE IF NOT EXISTS break_sessions (
    id TEXT PRIMARY KEY,
    work_session_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    "type" TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY(work_session_id) REFERENCES work_sessions(id)
);

-- Activity categories table
CREATE TABLE IF NOT EXISTS activity_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    source TEXT NOT NULL, -- desktop_application or browser
    identifier TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_active INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

-- Activity usage table
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
    FOREIGN KEY(work_session_id) REFERENCES work_sessions(id),
    FOREIGN KEY(activity_category_id) REFERENCES activity_categories(id)
);

-- Sync metadata table for offline sync tracking
CREATE TABLE IF NOT EXISTS sync_metadata (
    id TEXT PRIMARY KEY,
    table_name TEXT NOT NULL,
    row_id TEXT NOT NULL,
    synced INTEGER NOT NULL,
    last_attempt_at TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Insert initial migration version
INSERT OR IGNORE INTO migration_version (version) VALUES (1);
