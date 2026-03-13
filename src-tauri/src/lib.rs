mod capture;
mod commands;
mod content_type;
mod history;
mod ics_sanitizer;
mod keychain;
mod security;
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
            commands::get_history,
            commands::save_scan,
            commands::delete_history,
            commands::clear_history,
            commands::keychain_get_password,
            security::open_url,
            security::open_mailto,
            security::open_tel,
            security::open_sms,
            security::open_geo,
            security::open_calendar_event,
        ])
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            shortcuts::setup_shortcuts(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
