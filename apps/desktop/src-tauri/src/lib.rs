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
use auth::{AuthRepository, dev_adapter::DevelopmentAuthAdapter};
use device::{DeviceInfo, DeviceManager};
use persistence::Database;
use session::SessionManager;

pub struct AppState {
    pub auth: Arc<Mutex<Box<dyn AuthRepository>>>,
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
    let dev_auth = DevelopmentAuthAdapter::new();

    // If session was recovered, set user as logged in automatically
    if session_manager.is_recovered() {
        if let Some(active_sess) = session_manager.active_session() {
            let _ = dev_auth.login(&active_sess.employee_id, "");
        }
    }

    let app_state = AppState {
        auth: Arc::new(Mutex::new(Box::new(dev_auth))),
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
