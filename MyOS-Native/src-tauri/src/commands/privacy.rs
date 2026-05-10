use tauri::{command, State};
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;
use serde::Serialize;

#[derive(Serialize)]
pub struct PrivacyStatus {
    pub mac_randomized: bool,
    pub network_guardian_active: bool,
    pub timezone_masked: bool,
    pub active_user_agent: String,
}

#[derive(Serialize)]
pub struct DataExposure {
    pub category: String,
    pub risk_level: f32, // 0.0 to 1.0
}

#[command]
pub fn get_privacy_status(
    module_id: String,
    state: State<'_, AppState>,
) -> Result<PrivacyStatus, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SystemMonitor) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: SystemMonitor. Error: {}", e)), "WARN");
        return Err(e);
    }

    Ok(PrivacyStatus {
        mac_randomized: true,
        network_guardian_active: true,
        timezone_masked: true,
        active_user_agent: "YfitopsOS/8.0 (Privacy Shield)".to_string(),
    })
}

#[command]
pub fn get_privacy_heatmap(
    module_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DataExposure>, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SystemMonitor) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: SystemMonitor. Error: {}", e)), "WARN");
        return Err(e);
    }

    Ok(vec![
        DataExposure { category: "Location".to_string(), risk_level: 0.1 },
        DataExposure { category: "Hardware ID".to_string(), risk_level: 0.0 },
        DataExposure { category: "Network Latency".to_string(), risk_level: 0.4 },
        DataExposure { category: "Browsing Habits".to_string(), risk_level: 0.05 },
    ])
}

#[command]
pub fn toggle_network_guardian(
    module_id: String,
    active: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::PrivacyControl) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: PrivacyControl. Error: {}", e)), "WARN");
        return Err(e);
    }

    let _ = record_audit(&state, &module_id, "settings:changed", Some(format!("Network Guardian: {}", active)), "INFO");
    // Implementation: Update iptables or proxy settings
    Ok(())
}
