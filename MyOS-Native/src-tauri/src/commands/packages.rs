use tauri::{command, State, AppHandle, Manager};
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;
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
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SetSetting) {
        let _ = record_audit(&state, &module_id, "INSTALL_PACKAGE_DENIED", Some(e.clone()), "WARN");
        return Err(e);
    }

    // 1. Parse and validate manifest
    let manifest: YopsManifest = serde_json::from_str(&manifest_json).map_err(|e| e.to_string())?;
    let _ = record_audit(&state, &module_id, "INSTALL_PACKAGE", Some(format!("ID: {}", manifest.id)), "INFO");
    
    // 2. Security Check: Reject permission escalation
    // (In a real app, we'd compare against a known dangerous list)
    if manifest.permissions.contains(&"RootAccess".to_string()) {
        let _ = record_audit(&state, &module_id, "INSTALL_DENIED_ESCALATION", Some(format!("ID: {}", manifest.id)), "ERROR");
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
pub async fn search_system_packages(
    module_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::ExecCommand) {
        return Err(e);
    }

    // Safety: only allow alphanumeric queries
    if !query.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return Err("Invalid search query".to_string());
    }

    let output = std::process::Command::new("apt-cache")
        .args(["search", &query])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[command]
pub async fn install_system_package(
    module_id: String,
    package_name: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::ExecCommand) {
        return Err(e);
    }

    // Safety: package_name check
    if !package_name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Invalid package name".to_string());
    }

    let _ = record_audit(&state, &module_id, "SYS_INSTALL_START", Some(package_name.clone()), "INFO");

    let output = std::process::Command::new("sudo")
        .args(["apt-get", "install", "-y", &package_name])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        let _ = record_audit(&state, &module_id, "SYS_INSTALL_SUCCESS", Some(package_name), "INFO");
        Ok("Installation successful".to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        let _ = record_audit(&state, &module_id, "SYS_INSTALL_FAILURE", Some(err.clone()), "ERROR");
        Err(err)
    }
}

