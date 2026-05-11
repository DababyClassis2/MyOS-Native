use tauri::{command, State, AppHandle, Manager};
use crate::state::{AppState, SessionMode, Permission};
use crate::commands::logs::record_audit;
use rusqlite::Connection;
use crate::db;

#[command]
pub fn set_session_mode(
    module_id: String,
    mode: SessionMode,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Only desktop can change session mode
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SetSetting) {
        let _ = record_audit(&state, &module_id, "SET_SESSION_MODE_DENIED", Some(format!("Mode: {:?}. Error: {}", mode, e)), "WARN");
        return Err(e);
    }

    let mut current_mode = state.session_mode.lock().unwrap();
    if *current_mode == mode {
        return Ok(());
    }

    let _ = record_audit(&state, &module_id, "SET_SESSION_MODE", Some(format!("New Mode: {:?}", mode)), "INFO");
    let mut db = state.db.lock().unwrap();

    match mode {
        SessionMode::Amnesic => {
            // Switch to in-memory DB
            let mem_conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
            // Re-initialize schema in memory
            // In a real app, we might copy some data, but the roadmap says "zero surviving persistence"
            db::init_db_in_conn(&mem_conn).map_err(|e| e.to_string())?;
            *db = mem_conn;
        }
        SessionMode::Sovereign => {
            // Switch back to on-disk DB
            let db_path = "yfitops.db";
            let disk_conn = db::init_db(std::path::Path::new(db_path)).map_err(|e| e.to_string())?;
            *db = disk_conn;
        }
    }

    *current_mode = mode.clone();

    // Emit event to all windows
    app_handle.emit_all("session:mode_changed", mode).map_err(|e| e.to_string())?;

    Ok(())
}
