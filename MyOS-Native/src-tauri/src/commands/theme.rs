use tauri::{command, State, AppHandle, Manager};
use crate::state::{AppState, ThemePalette, Permission};
use crate::commands::logs::record_audit;

#[command]
pub fn set_wallpaper(
    module_id: String,
    _path: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<ThemePalette, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SetSetting) {
        let _ = record_audit(&state, &module_id, "SET_WALLPAPER_DENIED", Some(e.clone()), "WARN");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "SET_WALLPAPER", Some(format!("Path: {}", _path)), "INFO");
    // 1. Mock palette extraction (Phase 4 requirement)
    // In a full implementation, we'd use the `image` crate here.
    let palette = ThemePalette {
        accent: "#2ecc71".to_string(), // Green for YOS
        bg: "rgba(26, 26, 26, 0.8)".to_string(),
    };

    // 2. Update state
    let mut workspace = state.workspace.lock().unwrap();
    workspace.active_theme = palette.clone();

    // 3. Emit event to all windows
    app_handle.emit_all("theme:updated", &palette).map_err(|e| e.to_string())?;

    Ok(palette)
}
