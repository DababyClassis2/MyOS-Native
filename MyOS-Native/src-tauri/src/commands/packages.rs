use tauri::{command, State, AppHandle, Manager};
use crate::state::{AppState, Permission};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct YopsManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub permissions: Vec<String>,
}

#[command]
pub fn install_package(
    module_id: String,
    manifest_json: String,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    state.permission_guard.assert_capability(&module_id, Permission::SetSetting)?;

    // 1. Parse and validate manifest
    let manifest: YopsManifest = serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;
    
    // 2. Security Check: Reject permission escalation
    // (In a real app, we'd compare against a known dangerous list)
    if manifest.permissions.contains(&"RootAccess".to_string()) {
        return Err("Permission Escalation Denied".to_string());
    }

    // 3. Save to database
    let db = state.db.lock().unwrap();
    db.execute(
        "INSERT INTO packages (id, manifest_json, enabled, installed_at) 
         VALUES (?, ?, 1, (unixepoch()))
         ON CONFLICT(id) DO UPDATE SET 
         manifest_json = excluded.manifest_json",
        [&manifest.id, &manifest_json],
    ).map_err(|e| e.to_string())?;

    // 4. Emit event
    app_handle.emit_all("packages:installed", &manifest.id).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn list_packages(
    module_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<YopsManifest>, String> {
    state.permission_guard.assert_capability(&module_id, Permission::GetSetting)?;

    let db = state.db.lock().unwrap();
    let mut stmt = db.prepare("SELECT manifest_json FROM packages").map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([], |row| {
        let json: String = row.get(0)?;
        let manifest: YopsManifest = serde_json::from_str(&json).unwrap(); // Should handle error in production
        Ok(manifest)
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(results)
}
