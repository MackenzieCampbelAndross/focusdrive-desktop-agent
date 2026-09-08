// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--native-messaging-host" || arg.contains("com.focusdrive.desktopagent.native")) {
        // Native messaging host mode invoked by Chrome/Edge browser
        focusdrive_desktop_agent_lib::run_native_messaging_host().await;
    } else {
        // Standard Desktop GUI Agent mode
        focusdrive_desktop_agent_lib::run();
    }
}
