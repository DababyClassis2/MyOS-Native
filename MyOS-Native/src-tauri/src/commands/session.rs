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
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::PrivacyControl) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: PrivacyControl. Error: {}", e)), "WARN");
        return Err(e);
    }

    let mut current_mode = state.session_mode.lock().unwrap();
    if *current_mode == mode {
        return Ok(());
    }

    let _ = record_audit(&state, &module_id, "session:mode_changed", Some(format!("{:?} -> {:?}", *current_mode, mode)), "WARN");
    
    let mut db = state.db.lock().unwrap();

    match mode {
        SessionMode::Amnesic => {
            let mem_conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
            db::init_db_in_conn(&mem_conn).map_err(|e| e.to_string())?;
            *db = mem_conn;
        }
        SessionMode::Sovereign => {
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
