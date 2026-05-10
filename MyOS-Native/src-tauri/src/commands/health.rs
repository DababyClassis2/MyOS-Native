use tauri::{command, State};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, PidExt};
use serde::Serialize;
use crate::state::{AppState, Permission};
use crate::commands::logs::record_audit;

#[derive(Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub total: u64,
    pub available: u64,
}

#[derive(Serialize)]
pub struct SystemHealth {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub disks: Vec<DiskInfo>,
    pub process_list: Vec<ProcessInfo>,
    pub os_name: String,
}

#[command]
pub fn get_system_health(module_id: String, state: State<'_, AppState>) -> Result<SystemHealth, String> {
    if let Err(e) = state.permission_guard.assert_capability(&module_id, Permission::SystemMonitor) {
        let _ = record_audit(&state, &module_id, "security:permission_denied", Some(format!("Permission: SystemMonitor. Error: {}", e)), "WARN");
        return Err(e);
    }

    let mut sys = System::new_all();
    sys.refresh_all();
    
    let disks = sys.disks().iter().map(|disk| {
        DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            total: disk.total_space(),
            available: disk.available_space(),
        }
    }).collect();

    let process_list = sys.processes().iter().take(20).map(|(pid, process)| {
        ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
        }
    }).collect();

    Ok(SystemHealth {
        cpu_usage: sys.global_cpu_info().cpu_usage(),
        memory_used: sys.used_memory(),
        memory_total: sys.total_memory(),
        disks,
        process_list,
        os_name: sys.name().unwrap_or_else(|| "Unknown".to_string()),
    })
}
