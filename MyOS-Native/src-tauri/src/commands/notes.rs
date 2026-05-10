use tauri::{command, State};
use crate::state::{AppState, Permission};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub body: String,
    pub is_pinned: bool,
}

#[command]
pub fn list_notes(
    module_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Note>, String> {
    state.permission_guard.assert_capability(&module_id, Permission::ListNotes)?;

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare("SELECT id, title, body, is_pinned FROM notes WHERE profile_id = ? ORDER BY is_pinned DESC, updated_at DESC")
        .map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([&*profile_id], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            is_pinned: row.get::<_, i32>(3)? != 0,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(results)
}

#[command]
pub fn save_note(
    module_id: String,
    note: Note,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.permission_guard.assert_capability(&module_id, Permission::SaveNote)?;

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let id = if note.id.is_empty() {
        Uuid::new_v4().to_string()
    } else {
        note.id
    };

    db.execute(
        "INSERT INTO notes (id, title, body, is_pinned, profile_id, updated_at, created_at) 
         VALUES (?, ?, ?, ?, ?, (unixepoch()), (unixepoch()))
         ON CONFLICT(id) DO UPDATE SET 
         title = excluded.title, 
         body = excluded.body, 
         is_pinned = excluded.is_pinned,
         updated_at = excluded.updated_at",
        rusqlite::params![id, note.title, note.body, note.is_pinned as i32, &*profile_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[command]
pub fn delete_note(
    module_id: String,
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::DeleteNote) {
        let _ = record_audit(&state, &module_id, "DELETE_NOTE_DENIED", Some(format!("ID: {}. Error: {}", id, e)), "WARN");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "DELETE_NOTE", Some(format!("ID: {}", id)), "INFO");
    let db = state.db.lock().unwrap();
    db.execute("DELETE FROM notes WHERE id = ?", [id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn search_notes(
    module_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Note>, String> {
    state.permission_guard.assert_capability(&module_id, Permission::SearchNotes)?;

    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare(
        "SELECT id, title, body, is_pinned FROM notes 
         WHERE profile_id = ? AND rowid IN (SELECT rowid FROM notes_fts WHERE notes_fts MATCH ?)
         ORDER BY is_pinned DESC, updated_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([&*profile_id, &query], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            body: row.get(2)?,
            is_pinned: row.get::<_, i32>(3)? != 0,
        })
    }).map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for entry in entries {
        results.push(entry.map_err(|e| e.to_string())?);
    }

    Ok(results)
}
