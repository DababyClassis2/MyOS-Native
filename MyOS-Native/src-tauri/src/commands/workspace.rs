use tauri::{command, State, AppHandle, Manager};
use crate::state::{AppState, Permission};

#[command]
pub fn save_workspace_state(
    module_id: String,
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // ... logic ...
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO workspace_state (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [&key, &value],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn load_workspace_state(
    module_id: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    state.permission_guard.assert_capability(&module_id, Permission::GetSetting)?;

    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT value FROM workspace_state WHERE key = ?")
        .map_err(|e| e.to_string())?;
    
    let mut rows = stmt.query([&key]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let val: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

#[command]
pub fn toggle_focus_mode(
    module_id: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    state.permission_guard.assert_capability(&module_id, Permission::SetSetting)?;

    let mut workspace = state.workspace.lock().unwrap();
    workspace.focus_mode = !workspace.focus_mode;
    
    let is_active = workspace.focus_mode;
    
    // Emit event to all windows
    app_handle.emit_all("workspace:focus_mode_changed", is_active).map_err(|e| e.to_string())?;

    Ok(is_active)
}
