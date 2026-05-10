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
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::ExecWrite) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: ExecWrite. Error: {}", e)), "WARN");
        return Err(e);
    }

    // 2. Command whitelist check
    let cmd_parts: Vec<&str> = command.split_whitespace().collect();
    if cmd_parts.is_empty() {
        return Ok("".to_string());
    }

    let base_cmd = cmd_parts[0];
    if !state.permission_guard.exec_whitelist().contains(&base_cmd.to_string()) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Whitelisted command violation: {}", base_cmd)), "ERROR");
        return Err(format!("Command '{}' is not whitelisted", base_cmd));
    }

    // 3. Execution
    let _ = record_audit(&state, &module_id, "exec:command", Some(command.clone()), "INFO");
    
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
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let _ = record_audit(&state, &module_id, "exec:error", Some(stderr.clone()), "ERROR");
                Err(stderr)
            }
        },
        Err(e) => {
            let err_msg = e.to_string();
            let _ = record_audit(&state, &module_id, "exec:error", Some(err_msg.clone()), "ERROR");
            Err(err_msg)
        },
    }
}
