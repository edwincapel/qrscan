use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager,
};

const TRAY_ICON: &[u8] = include_bytes!("../icons/icon.png");

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("scan_region", "Scan Region\t⌘⇧S")
        .text("scan_window", "Scan Window\t⌘⌥W")
        .separator()
        .text("history", "History\t⌘⇧H")
        .text("settings", "Settings")
        .separator()
        .text("about", "About QRSnap")
        .quit()
        .build()?;

    let icon = Image::from_bytes(TRAY_ICON)?;

    TrayIconBuilder::with_id("qrsnap-tray")
        .menu(&menu)
        .icon(icon)
        .icon_as_template(true) // Adapts to macOS dark/light menu bar
        .tooltip("QRSnap")
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn handle_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "scan_region" => {
            let _ = app.emit("scan-region", ());
        }
        "scan_window" => {
            let _ = app.emit("scan-window", ());
        }
        "history" => {
            show_window(app);
            let _ = app.emit("show-history", ());
        }
        "settings" => {
            show_window(app);
            let _ = app.emit("show-settings", ());
        }
        "about" => {
            let _ = app.emit("show-about", ());
        }
        _ => {}
    }
}
