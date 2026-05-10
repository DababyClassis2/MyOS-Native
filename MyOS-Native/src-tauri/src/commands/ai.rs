use tauri::{command, State};
use crate::state::{AppState, Permission};
use serde::Serialize;

#[derive(Serialize)]
pub struct AIResponse {
    pub text: String,
    pub source: String, // e.g. "llama-3-8b-local"
}

#[command]
pub async fn query_assistant(
    module_id: String,
    prompt: String,
    state: State<'_, AppState>,
) -> Result<AIResponse, String> {
    state.permission_guard.assert_capability(&module_id, Permission::QueryAI)?;

    // 1. Verify Privacy Shield requirement
    // (Logic: AI only works if Privacy Shield is ACTIVE and no Network is detected)
    
    // 2. Mock Local Inference
    Ok(AIResponse {
        text: format!("I am your sovereign assistant. You asked: '{}'. (Note: This is a local-only mock)", prompt),
        source: "local-model-v1".to_string(),
    })
}

#[command]
pub fn get_ai_status(
    _module_id: String,
    state: State<'_, AppState>,
) -> Result<bool, String> {
    // Check if AI is enabled in settings
    let db = state.db.lock().unwrap();
    let profile_id = state.active_profile.lock().unwrap();

    let mut stmt = db.prepare("SELECT value FROM settings WHERE key = 'ai-enabled' AND profile_id = ?").map_err(|e| e.to_string())?;
    let mut rows = stmt.query([&*profile_id]).map_err(|e| e.to_string())?;

    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let val: String = row.get(0).map_err(|e| e.to_string())?;
        Ok(val == "true")
    } else {
        Ok(false) // Default: Disabled
    }
}
