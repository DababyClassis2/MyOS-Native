use tauri::{command, State};
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;
use serde::{Serialize, Deserialize};

#[command]
pub fn get_setting(
    module_id: String,
    key: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::GetSetting) {
        let _ = record_audit(&state, &module_id, "GET_SETTING_DENIED", Some(format!("Key: {}. Error: {}", key, e)), "WARN");
        return Err(e);
    }

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare("SELECT value FROM settings WHERE key = ? AND profile_id = ?")
        .map_err(|e| e.to_string())?;
    
    let mut rows = stmt.query([&key, &*profile_id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let val: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

#[command]
pub fn set_setting(
    module_id: String,
    key: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SetSetting) {
        let _ = record_audit(&state, &module_id, "SET_SETTING_DENIED", Some(format!("Key: {}. Error: {}", key, e)), "WARN");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "SET_SETTING", Some(format!("Key: {}", key)), "INFO");
    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    db.execute(
        "INSERT INTO settings (key, value, profile_id, updated_at) 
         VALUES (?, ?, ?, (unixepoch()))
         ON CONFLICT(key, profile_id) DO UPDATE SET 
         value = excluded.value, 
         updated_at = excluded.updated_at",
        [&key, &value, &*profile_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

#[command]
pub fn list_settings(
    module_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SettingEntry>, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::ListSettings) {
        let _ = record_audit(&state, &module_id, "LIST_SETTINGS_DENIED", Some(e.clone()), "WARN");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "LIST_SETTINGS", None, "INFO");

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare("SELECT key, value FROM settings WHERE profile_id = ?")
        .map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([&*profile_id], |row| {
        Ok(SettingEntry {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(results)
}
