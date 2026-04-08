mod models;
mod preset_data;
mod system;

use models::{ActionResult, ApplySummary, BootstrapPayload, ImportedPresetPayload, RenderStyleId};
use tauri::Manager;
#[cfg(target_os = "windows")]
use window_vibrancy::apply_mica;

#[tauri::command]
async fn load_bootstrap() -> Result<BootstrapPayload, String> {
    tauri::async_runtime::spawn_blocking(|| system::load_bootstrap(&preset_data::preset_definitions()))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_apply_summary(preset_id: String, render_style_id: String) -> Result<ApplySummary, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let render_style_id = RenderStyleId::from_str(&render_style_id)
            .ok_or_else(|| "不认识这个渲染风格。".to_string())?;
        system::build_apply_summary(&preset_id, render_style_id, &preset_data::preset_definitions())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn apply_preset(preset_id: String, render_style_id: String) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let render_style_id = RenderStyleId::from_str(&render_style_id)
            .ok_or_else(|| "不认识这个渲染风格。".to_string())?;
        system::apply_preset(&preset_id, render_style_id, &preset_data::preset_definitions())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn import_font_files(paths: Vec<String>) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || system::import_font_files(&paths))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn restore_windows_default() -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(|| {
        system::restore_windows_default(&preset_data::preset_definitions())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
async fn run_recovery_action(action: String) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        system::run_recovery_action(&action, &preset_data::preset_definitions())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn export_current_scheme(preset_id: String, render_style_id: String) -> Result<ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let render_style_id = RenderStyleId::from_str(&render_style_id)
            .ok_or_else(|| "不认识这个渲染风格。".to_string())?;
        system::export_current_scheme(&preset_id, render_style_id, &preset_data::preset_definitions())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn import_shared_scheme(path: String) -> Result<ImportedPresetPayload, String> {
    tauri::async_runtime::spawn_blocking(move || system::import_shared_scheme(&path))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn repair_system_fonts() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(system::launch_system_font_repair)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if system::should_run_headless_recovery() {
        let _ = system::run_headless_recovery(&preset_data::preset_definitions());
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            get_apply_summary,
            apply_preset,
            import_font_files,
            restore_windows_default,
            run_recovery_action,
            export_current_scheme,
            import_shared_scheme,
            repair_system_fonts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
