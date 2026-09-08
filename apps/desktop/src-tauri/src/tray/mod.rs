use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder},
    AppHandle, Manager,
};
use crate::state::SessionState;

pub fn create_tray(app: &AppHandle) -> Result<TrayIcon, tauri::Error> {
    let open_item = MenuItem::with_id(app, "open", "Open FocusDrive Agent", true, None::<&str>)?;
    let status_item = MenuItem::with_id(app, "status", "Status: LOGGED_OUT", false, None::<&str>)?;
    let action_item = MenuItem::with_id(app, "action", "Start Shift", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Agent", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_item, &status_item, &action_item, &quit_item],
    )?;

    let tray = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let id = event.id.as_ref();
            match id {
                "open" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(tray)
}

pub fn update_tray_state(_app: &AppHandle, _state: SessionState) {
    // Tray state dynamically reflects current session state
    tracing::info!("Tray state updated for state: {}", _state);
}
