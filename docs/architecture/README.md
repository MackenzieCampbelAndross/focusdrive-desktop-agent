# FocusDrive Desktop Agent Architecture

## Overview
The FocusDrive Desktop Agent is a local-first Windows telemetry agent built with Tauri 2, React, TypeScript, and Rust.

## Phase 1 — Employee Session Architecture
- **Auth Abstraction**: `AuthRepository` trait with `DevelopmentAuthAdapter` (for test employee sessions) and `SupabaseAuthAdapter` (for production authentication).
- **Session State Machine**: Centralized state machine in Rust (`LOGGED_OUT`, `SHIFT_READY`, `WORKING`, `MANUAL_BREAK`, `SHIFT_COMPLETING`, `SHIFT_SUMMARY`).
- **Device Identity**: Persistent installation UUID auto-generated on first launch and stored in SQLite `devices` table.
- **Local Persistence (SQLite)**: SQLite database (`focusdrive_agent.db`) via `rusqlite` storing `devices`, `work_sessions`, `break_sessions`, and `app_state`.
- **Crash Recovery**: `SessionManager::recover_active_session()` detects active uncompleted shifts on application launch, restoring session state (`WORKING` or `MANUAL_BREAK`) and displaying recovery status to the employee.
- **Tauri System Tray & Window Behavior**: System tray reflects session status. Window close request hides to system tray rather than quitting, preserving active background shifts.
