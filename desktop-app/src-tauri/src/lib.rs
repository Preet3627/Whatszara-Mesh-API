use tauri::Manager;

#[tauri::command]
fn get_status() -> String {
    serde_json::json!({
        "app": "whatszara",
        "version": "0.1.0",
        "whatsapp_connected": false,
        "orchestrator_running": false,
        "active_provider": null
    })
    .to_string()
}

#[tauri::command]
fn get_settings() -> String {
    "{}".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_status, get_settings])
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};

                let _tray = TrayIconBuilder::new()
                    .tooltip("Whatszara")
                    .on_tray_icon_event(|tray, event| {
                        let TrayIconEvent::Click { button, button_state, .. } = event else {
                            return;
                        };
                        if button == tauri::tray::MouseButton::Left
                            && button_state == tauri::tray::MouseButtonState::Up
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
