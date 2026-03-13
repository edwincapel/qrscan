use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

pub fn setup_shortcuts(app: &AppHandle) -> tauri::Result<()> {
    let handler = |app: &AppHandle, shortcut: &tauri_plugin_global_shortcut::Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyS) {
            let _ = app.emit("scan-region", ());
        } else if shortcut.matches(Modifiers::SUPER | Modifiers::ALT, Code::KeyW) {
            let _ = app.emit("scan-window", ());
        } else if shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyH) {
            let _ = app.emit("show-history", ());
        }
    };

    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(handler);

    let plugin = match plugin.with_shortcuts(["super+shift+s", "super+alt+w", "super+shift+h"]) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Shortcut registration failed: {e}");
            let _ = app.emit("shortcut-conflict", e.to_string());
            return Ok(());
        }
    };

    match app.plugin(plugin.build()) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Global shortcut plugin failed: {e}");
            let _ = app.emit("shortcut-conflict", e.to_string());
        }
    }
    Ok(())
}
