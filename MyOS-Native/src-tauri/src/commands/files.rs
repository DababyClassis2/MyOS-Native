use tauri::{command, State};
use serde::Serialize;
use std::fs;
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;

#[derive(Serialize)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

#[command]
pub fn list_dir(
    module_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileEntry>, String> {
    // 1. Strict contract check
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::FileRead) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: FileRead. Error: {}", e)), "WARN");
        return Err(e);
    }

    // 2. Sandbox enforcement
    if let Err(e) = state.permission_guard.assert_path_in_sandbox(&path) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Sandbox violation: {}", e)), "ERROR");
        return Err(e);
    }

    // 3. Execution
    let _ = record_audit(&state, &module_id, "files:list", Some(path.clone()), "INFO");
    let entries = fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut files = Vec::new();

    for entry in entries {
        if let Ok(entry) = entry {
            let meta = entry.metadata().map_err(|e| e.to_string())?;
            files.push(FileEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                is_dir: meta.is_dir(),
                size: meta.len(),
            });
        }
    }

    Ok(files)
}

#[command]
pub fn read_text_file(
    module_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::FileRead) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: FileRead. Error: {}", e)), "WARN");
        return Err(e);
    }
    
    if let Err(e) = state.permission_guard.assert_path_in_sandbox(&path) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Sandbox violation: {}", e)), "ERROR");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "files:read", Some(path.clone()), "INFO");
    fs::read_to_string(path).map_err(|e| e.to_string())
}
