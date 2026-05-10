use tauri::{command, State, Window, WindowBuilder, WindowUrl, Manager};
use crate::state::{AppState, Permission};

#[command]
pub async fn open_module(
    module_id: String,
    window: Window,
    target_module_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Strict contract check
    state.permission_guard.assert_capability(&module_id, Permission::OpenModule)?;

    let mut workspace = state.workspace.lock().unwrap();
    
    // Check if already open
    if let Some(label) = workspace.open_modules.get(&target_module_id) {
        if let Some(win) = window.get_window(label) {
            win.set_focus().map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    let module_url = format!("modules/{}.html", target_module_id);
    let label = target_module_id.clone();

    WindowBuilder::new(
        &window,
        &label,
        WindowUrl::App(module_url.into())
    )
    .title(format!("YOS - {}", target_module_id))
    .inner_size(800.0, 600.0)
    .decorations(true)
    .build()
    .map_err(|e| e.to_string())?;

    workspace.open_modules.insert(target_module_id, label);
    
    Ok(())
}
