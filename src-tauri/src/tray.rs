use tauri::{
    image::Image,
    menu::MenuBuilder,
    tray::TrayIconBuilder,
    AppHandle, Emitter,
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
        .tooltip("QRSnap")
        .show_menu_on_left_click(true)
        .on_menu_event(handle_menu_event)
        .build(app)?;

    Ok(())
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
            let _ = app.emit("show-history", ());
        }
        "settings" => {
            let _ = app.emit("show-settings", ());
        }
        "about" => {
            let _ = app.emit("show-about", ());
        }
        _ => {}
    }
}
