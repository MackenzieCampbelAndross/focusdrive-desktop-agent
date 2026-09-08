use std::fs;
use std::path::PathBuf;
use serde_json::json;
use crate::errors::{AppError, Result};

pub const NATIVE_HOST_NAME: &str = "com.focusdrive.desktopagent.native";

/// Generates the manifest JSON content for Chrome and Edge
pub fn create_manifest_json(exe_path: &str, allowed_origins: &[&str]) -> String {
    let origins_val: Vec<String> = allowed_origins.iter().map(|s| s.to_string()).collect();
    let manifest = json!({
        "name": NATIVE_HOST_NAME,
        "description": "FocusDrive Workstation Telemetry Native Messaging Host",
        "path": exe_path,
        "type": "stdio",
        "allowed_origins": origins_val
    });
    serde_json::to_string_pretty(&manifest).unwrap_or_default()
}

/// Installs native messaging manifest for Chrome and Edge in LocalAppData
pub fn install_native_messaging_manifests(exe_path: &str) -> Result<()> {
    let local_app_data = std::env::var("LOCALAPPDATA")
        .map_err(|_| AppError::InternalError("LOCALAPPDATA environment variable not found".to_string()))?;

    let default_origins = [
        "chrome-extension://*",
    ];

    let manifest_content = create_manifest_json(exe_path, &default_origins);

    // Chrome destination
    let mut chrome_dir = PathBuf::from(&local_app_data);
    chrome_dir.push("Google");
    chrome_dir.push("Chrome");
    chrome_dir.push("User Data");
    chrome_dir.push("NativeMessagingHosts");

    let _ = fs::create_dir_all(&chrome_dir);
    let chrome_manifest_file = chrome_dir.join(format!("{}.json", NATIVE_HOST_NAME));
    fs::write(&chrome_manifest_file, &manifest_content)
        .map_err(|e| AppError::InternalError(format!("Failed to write Chrome native manifest: {}", e)))?;

    // Edge destination
    let mut edge_dir = PathBuf::from(&local_app_data);
    edge_dir.push("Microsoft");
    edge_dir.push("Edge");
    edge_dir.push("User Data");
    edge_dir.push("NativeMessagingHosts");

    let _ = fs::create_dir_all(&edge_dir);
    let edge_manifest_file = edge_dir.join(format!("{}.json", NATIVE_HOST_NAME));
    fs::write(&edge_manifest_file, &manifest_content)
        .map_err(|e| AppError::InternalError(format!("Failed to write Edge native manifest: {}", e)))?;

    tracing::info!("Native messaging manifests installed for Chrome and Edge");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_manifest_json() {
        let json_str = create_manifest_json(r"C:\Program Files\FocusDrive\agent.exe", &["chrome-extension://abcdef/"]);
        assert!(json_str.contains("com.focusdrive.desktopagent.native"));
        assert!(json_str.contains("stdio"));
        assert!(json_str.contains("chrome-extension://abcdef/"));
    }
}
