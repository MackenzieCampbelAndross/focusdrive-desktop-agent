#[tauri::command]
fn health_check() -> String {
    "FocusDrive Desktop Agent Engine Active".to_string()
}

#[tauri::command]
fn get_agent_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            health_check,
            get_agent_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check() {
        assert_eq!(health_check(), "FocusDrive Desktop Agent Engine Active");
    }

    #[test]
    fn test_get_agent_version() {
        assert_eq!(get_agent_version(), "0.1.0");
    }
}
