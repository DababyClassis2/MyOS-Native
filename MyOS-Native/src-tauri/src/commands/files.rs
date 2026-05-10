use tauri::{command, State};
use serde::Serialize;
use std::fs;
use crate::state::{AppState, Permission};

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
    state.permission_guard.assert_capability(&module_id, Permission::ListDir)?;

    // 2. Sandbox enforcement
    state.permission_guard.assert_path_in_sandbox(&path)?;

    // 3. Execution
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
    state.permission_guard.assert_capability(&module_id, Permission::ReadFile)?;
    state.permission_guard.assert_path_in_sandbox(&path)?;

    fs::read_to_string(path).map_err(|e| e.to_string())
}
