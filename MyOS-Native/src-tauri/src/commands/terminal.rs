use tauri::{command, State};
use std::process::Command;
use crate::state::{AppState, Permission};

#[command]
pub async fn exec_command(
    module_id: String,
    command: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Strict contract check
    state.permission_guard.assert_capability(&module_id, Permission::ExecCommand)?;

    // 2. Command whitelist check
    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Ok("".to_string());
    }

    let base_cmd = cmd_parts[0];
    if !state.permission_guard.exec_whitelist().contains(&base_cmd.to_string()) {
        return Err(format!("Command '{}' is not whitelisted", base_cmd));
    }

    // 3. Execution
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", &command])
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(String::from_utf8_lossy(&out.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&out.stderr).to_string())
            }
        },
        Err(e) => Err(e.to_string()),
    }
}
