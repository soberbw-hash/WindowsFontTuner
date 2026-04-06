mod models;
mod preset_data;
mod system;

use models::{ActionResult, BootstrapPayload};
use preset_data::preset_definitions;
use tauri::Manager;
#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

#[tauri::command]
fn load_bootstrap() -> Result<BootstrapPayload, String> {
    system::load_bootstrap(&preset_definitions()).map_err(|error| error.to_string())
}

#[tauri::command]
fn apply_preset(preset_id: String) -> Result<ActionResult, String> {
    system::apply_preset(&preset_id, &preset_definitions()).map_err(|error| error.to_string())
}

#[tauri::command]
fn restore_windows_default() -> Result<ActionResult, String> {
    system::restore_windows_default(&preset_definitions()).map_err(|error| error.to_string())
}

#[tauri::command]
fn repair_system_fonts() -> Result<(), String> {
    system::launch_system_font_repair().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if system::should_run_headless_recovery() {
        let _ = system::run_headless_recovery(&preset_definitions());
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                let _ = apply_mica(&window, None);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_bootstrap,
            apply_preset,
            restore_windows_default,
            repair_system_fonts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
