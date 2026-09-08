pub mod auth;
pub mod commands;
pub mod device;
pub mod errors;
pub mod persistence;
pub mod session;
pub mod state;
pub mod tray;

use std::sync::{Arc, Mutex};
use tauri::WindowEvent;
use auth::{AuthMode, create_auth_repository};
use device::{DeviceInfo, DeviceManager};
use persistence::Database;
use session::SessionManager;

pub struct AppState {
    pub auth: Arc<Mutex<Box<dyn auth::AuthRepository>>>,
    pub session: Arc<Mutex<SessionManager>>,
    pub device_info: Arc<Mutex<DeviceInfo>>,
    pub db: Database,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = match dirs::data_local_dir() {
        Some(mut path) => {
            path.push("FocusDriveAgent");
            path.push("focusdrive_agent.db");
            Database::new_file(path).unwrap_or_else(|_| Database::new_in_memory().unwrap())
        }
        None => Database::new_in_memory().unwrap(),
    };

    let device_info = DeviceManager::get_or_create_device_info(&db)
        .unwrap_or_else(|_| DeviceInfo {
            device_id: uuid::Uuid::new_v4().to_string(),
            device_name: "Windows Workstation".to_string(),
            operating_system: "Windows".to_string(),
            agent_version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_seen_at: chrono::Utc::now().to_rfc3339(),
        });

    let session_manager = SessionManager::new(db.clone());

    // Determine auth mode explicitly (defaults to Development mode in dev environment)
    let auth_mode = match std::env::var("FOCUSDRIVE_AUTH_MODE").as_deref() {
        Ok("supabase") | Ok("production") => AuthMode::SupabaseProduction,
        _ => AuthMode::Development,
    };

    let supabase_url = std::env::var("VITE_SUPABASE_URL").ok();
    let anon_key = std::env::var("VITE_SUPABASE_ANON_KEY").ok();

    let auth_repo = create_auth_repository(auth_mode, supabase_url, anon_key)
        .unwrap_or_else(|_| create_auth_repository(AuthMode::Development, None, None).unwrap());

    // Note: Session recovery restores local shift state without fabricating authentication.

    let app_state = AppState {
        auth: Arc::new(Mutex::new(auth_repo)),
        session: Arc::new(Mutex::new(session_manager)),
        device_info: Arc::new(Mutex::new(device_info)),
        db,
    };

    tauri::Builder::default()
        .manage(app_state)
        .setup(|app| {
            let _ = tray::create_tray(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Prevent app from quitting when window close is requested
                // Hide window to system tray instead so background session continues
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_auth_state,
            commands::login,
            commands::logout,
            commands::get_session_state,
            commands::start_shift,
            commands::start_manual_break,
            commands::resume_work,
            commands::end_shift,
            commands::dismiss_summary,
            commands::get_current_shift,
            commands::get_current_break,
            commands::get_device_info,
            commands::get_agent_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running FocusDrive Desktop Agent");
}
