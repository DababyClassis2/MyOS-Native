use tauri::{command, State};
use std::process::Command;
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;

#[command]
pub async fn exec_command(
    module_id: String,
    command: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // 1. Strict contract check
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::ExecCommand) {
        let _ = record_audit(&state, &module_id, "EXEC_DENIED", Some(format!("Cmd: {}. Error: {}", command, e)), "WARN");
        return Err(e);
    }

    // 2. Command whitelist check
    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Ok("".to_string());
    }

    let base_cmd = cmd_parts[0];
    if !state.permission_guard.exec_whitelist().contains(&base_cmd.to_string()) {
        let _ = record_audit(&state, &module_id, "EXEC_WHITELIST_VIOLATION", Some(format!("Cmd: {}", command)), "ERROR");
        return Err(format!("Command '{}' is not whitelisted", base_cmd));
    }

    // 3. Execution
    let _ = record_audit(&state, &module_id, "EXEC_START", Some(format!("Cmd: {}", command)), "INFO");
    
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .args(["/C", &command])
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let _ = record_audit(&state, &module_id, "EXEC_SUCCESS", Some(format!("Cmd: {}", command)), "INFO");
                Ok(stdout)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let _ = record_audit(&state, &module_id, "EXEC_FAILURE", Some(format!("Cmd: {}. Error: {}", command, stderr)), "ERROR");
                Err(stderr)
            }
        },
        Err(e) => {
            let err_msg = e.to_string();
            let _ = record_audit(&state, &module_id, "EXEC_ERROR", Some(format!("Cmd: {}. Error: {}", command, err_msg)), "ERROR");
            Err(err_msg)
        },
    }
}
