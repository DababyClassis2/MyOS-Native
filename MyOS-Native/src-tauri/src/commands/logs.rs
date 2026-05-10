use tauri::{command, State};
use crate::state::{AppState, Permission};
use serde::Serialize;

#[derive(Serialize)]
pub struct AuditEntry {
    pub id: i64,
    pub ts: i64,
    pub module_id: String,
    pub action: String,
    pub detail: Option<String>,
    pub severity: String,
}

#[command]
pub fn write_audit(
    module_id: String,
    action: String,
    detail: Option<String>,
    severity: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.permission_guard.assert_capability(&module_id, Permission::WriteAudit)?;

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    db.execute(
        "INSERT INTO audit_log (module_id, action, detail, severity, profile_id, ts) 
         VALUES (?, ?, ?, ?, ?, (unixepoch()))",
        [&module_id, &action, &detail.unwrap_or_default(), &severity, &*profile_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn query_audit(
    module_id: String,
    limit: u32,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEntry>, String> {
    state.permission_guard.assert_capability(&module_id, Permission::QueryAudit)?;

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare("SELECT id, ts, module_id, action, detail, severity FROM audit_log WHERE profile_id = ? ORDER BY ts DESC LIMIT ?")
        .map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([&*profile_id, &limit.to_string()], |row| {
        Ok(AuditEntry {
            id: row.get(0)?,
            ts: row.get(1)?,
            module_id: row.get(2)?,
            action: row.get(3)?,
            detail: row.get(4)?,
            severity: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(results)
}
