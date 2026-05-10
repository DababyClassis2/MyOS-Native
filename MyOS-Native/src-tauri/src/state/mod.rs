use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Permission {
    ExecCommand,
    ListDir,
    ReadFile,
    GetSystemHealth,
    GetSetting,
    SetSetting,
    ListSettings,
    ListNotes,
    SaveNote,
    DeleteNote,
    SearchNotes,
    OpenModule,
    WriteAudit,
    QueryAudit,
    QueryAI,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub permissions: Vec<Permission>,
}

pub struct PermissionGuard {
    pub manifests: HashMap<String, ModuleManifest>,
    pub sandbox_root: String,
}

impl PermissionGuard {
    pub fn new() -> Self {
        let mut manifests = HashMap::new();

        // 1. Desktop Shell
        manifests.insert("desktop".to_string(), ModuleManifest {
            id: "desktop".to_string(),
            name: "Desktop Shell".to_string(),
            permissions: vec![
                Permission::GetSystemHealth,
                Permission::OpenModule,
                Permission::GetSetting,
                Permission::SetSetting,
                Permission::WriteAudit,
                Permission::QueryAudit,
            ],
        });

        // 2. Terminal
        manifests.insert("terminal".to_string(), ModuleManifest {
            id: "terminal".to_string(),
            name: "Terminal".to_string(),
            permissions: vec![
                Permission::ExecCommand,
                Permission::GetSystemHealth,
                Permission::WriteAudit,
            ],
        });

        // 3. Activity Monitor
        manifests.insert("activity".to_string(), ModuleManifest {
            id: "activity".to_string(),
            name: "Activity Monitor".to_string(),
            permissions: vec![Permission::GetSystemHealth, Permission::WriteAudit],
        });

        // 4. File Browser
        manifests.insert("files".to_string(), ModuleManifest {
            id: "files".to_string(),
            name: "File Browser".to_string(),
            permissions: vec![
                Permission::ListDir,
                Permission::ReadFile,
                Permission::WriteAudit,
            ],
        });

        // 5. Settings
        manifests.insert("settings".to_string(), ModuleManifest {
            id: "settings".to_string(),
            name: "Settings".to_string(),
            permissions: vec![
                Permission::ListSettings,
                Permission::GetSetting,
                Permission::SetSetting,
                Permission::WriteAudit,
            ],
        });

        // 6. Notes
        manifests.insert("notes".to_string(), ModuleManifest {
            id: "notes".to_string(),
            name: "Notes".to_string(),
            permissions: vec![
                Permission::ListNotes,
                Permission::SaveNote,
                Permission::DeleteNote,
                Permission::SearchNotes,
                Permission::WriteAudit,
            ],
        });

        // 7. Audit Logs
        manifests.insert("logs".to_string(), ModuleManifest {
            id: "logs".to_string(),
            name: "Audit Logs".to_string(),
            permissions: vec![
                Permission::QueryAudit,
                Permission::WriteAudit,
            ],
        });

        PermissionGuard { 
            manifests,
            sandbox_root: "D:/My-Os-Project".to_string(),
        }
    }

    pub fn exec_whitelist(&self) -> Vec<String> {
        vec!["ls".to_string(), "dir".to_string(), "pwd".to_string(), "whoami".to_string(), "date".to_string(), "echo".to_string(), "ping".to_string()]
    }

    pub fn assert_capability(&self, module_id: &str, permission: Permission) -> Result<(), String> {
        if let Some(manifest) = self.manifests.get(module_id) {
            if manifest.permissions.contains(&permission) {
                return Ok(());
            }
        }
        Err(format!("Permission Denied: Module '{}' lacks capability '{:?}'", module_id, permission))
    }

    pub fn assert_path_in_sandbox(&self, path: &str) -> Result<(), String> {
        let target = std::path::Path::new(path);
        let root = std::path::Path::new(&self.sandbox_root);
        if target.starts_with(root) {
            Ok(())
        } else {
            Err(format!("Sandbox Violation: Access to '{}' is restricted.", path))
        }
    }
}

pub struct AppState {
    pub db:               Arc<Mutex<Connection>>,
    pub permission_guard: Arc<PermissionGuard>,
    pub session_mode:     Arc<Mutex<SessionMode>>,
    pub workspace:        Arc<Mutex<WorkspaceState>>,
    pub active_profile:   Arc<Mutex<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionMode { Sovereign, Amnesic }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemePalette { pub accent: String, pub bg: String }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkspaceState {
    pub open_modules: HashMap<String, String>,
    pub focus_mode:   bool,
    pub active_theme: ThemePalette,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

impl AppState {
    pub fn new(db_conn: Connection) -> Self {
        AppState {
            db: Arc::new(Mutex::new(db_conn)),
            permission_guard: Arc::new(PermissionGuard::new()),
            session_mode: Arc::new(Mutex::new(SessionMode::Sovereign)),
            workspace: Arc::new(Mutex::new(WorkspaceState {
                open_modules: HashMap::new(),
                focus_mode: false,
                active_theme: ThemePalette {
                    accent: "#2ecc71".to_string(),
                    bg: "rgba(26, 26, 26, 0.7)".to_string(),
                },
            })),
            active_profile: Arc::new(Mutex::new("default".to_string())),
        }
    }
}
