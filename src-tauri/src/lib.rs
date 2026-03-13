mod capture;
mod commands;
mod content_type;
mod ics_sanitizer;
mod shortcuts;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::check_screen_permission,
            commands::trigger_scan,
            commands::parse_qr_content,
        ])
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            shortcuts::setup_shortcuts(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
