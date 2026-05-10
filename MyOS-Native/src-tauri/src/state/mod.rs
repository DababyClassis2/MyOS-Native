use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use rusqlite::Connection;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Permission {
    ExecRead,
    ExecWrite,
    FileRead,
    FileWrite,
    FileWatch,
    ClipboardRead,
    ClipboardWrite,
    SystemMonitor,
    NetworkRead,
    NetworkControl,
    AuditRead,
    PackageInstall,
    PrivacyControl,
    // Internal orchestration permissions
    OpenModule,
    DataRead,    // Covers GetSetting, ListNotes, etc.
    DataWrite,   // Covers SetSetting, SaveNote, etc.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleManifest {
    pub id: String,
    pub name: String,
    pub permissions: Vec<Permission>,
    pub trusted: bool,
}

pub struct PermissionGuard {
    pub manifests: HashMap<String, ModuleManifest>,
    pub sandbox_root: String,
}

impl PermissionGuard {
    pub fn new() -> Self {
        let mut manifests = HashMap::new();

        // 1. Desktop Shell
        manifests.insert("com.yfitops.desktop".to_string(), ModuleManifest {
            id: "com.yfitops.desktop".to_string(),
            name: "Desktop Shell".to_string(),
            permissions: vec![
                Permission::SystemMonitor,
                Permission::OpenModule,
                Permission::DataRead,
                Permission::DataWrite,
                Permission::AuditRead,
            ],
            trusted: true,
        });

        // 2. Terminal
        manifests.insert("com.yfitops.terminal".to_string(), ModuleManifest {
            id: "com.yfitops.terminal".to_string(),
            name: "Terminal".to_string(),
            permissions: vec![
                Permission::ExecWrite,
                Permission::ExecRead,
                Permission::AuditRead,
            ],
            trusted: true,
        });

        // 3. Activity Monitor
        manifests.insert("com.yfitops.monitor".to_string(), ModuleManifest {
            id: "com.yfitops.monitor".to_string(),
            name: "Activity Monitor".to_string(),
            permissions: vec![
                Permission::SystemMonitor,
                Permission::NetworkRead,
            ],
            trusted: true,
        });

        // 4. File Browser
        manifests.insert("com.yfitops.files".to_string(), ModuleManifest {
            id: "com.yfitops.files".to_string(),
            name: "File Browser".to_string(),
            permissions: vec![
                Permission::FileRead,
                Permission::FileWrite,
                Permission::FileWatch,
                Permission::ClipboardWrite,
            ],
            trusted: true,
        });

        // 5. Settings
        manifests.insert("com.yfitops.settings".to_string(), ModuleManifest {
            id: "com.yfitops.settings".to_string(),
            name: "Settings".to_string(),
            permissions: vec![
                Permission::FileRead,
                Permission::NetworkRead,
                Permission::PrivacyControl,
                Permission::DataRead,
                Permission::DataWrite,
            ],
            trusted: true,
        });

        // 6. Notes
        manifests.insert("com.yfitops.notes".to_string(), ModuleManifest {
            id: "com.yfitops.notes".to_string(),
            name: "Notes".to_string(),
            permissions: vec![
                Permission::FileRead,
                Permission::ClipboardWrite,
                Permission::DataRead,
                Permission::DataWrite,
            ],
            trusted: true,
        });

        // 7. Audit Logs
        manifests.insert("com.yfitops.logs".to_string(), ModuleManifest {
            id: "com.yfitops.logs".to_string(),
            name: "Audit Logs".to_string(),
            permissions: vec![
                Permission::AuditRead,
            ],
            trusted: true,
        });

        // 8. Package Manager
        manifests.insert("com.yfitops.packages".to_string(), ModuleManifest {
            id: "com.yfitops.packages".to_string(),
            name: "Package Manager".to_string(),
            permissions: vec![
                Permission::PackageInstall,
                Permission::FileRead,
            ],
            trusted: true,
        });

        // 9. Clipboard
        manifests.insert("com.yfitops.clipboard".to_string(), ModuleManifest {
            id: "com.yfitops.clipboard".to_string(),
            name: "Clipboard".to_string(),
            permissions: vec![
                Permission::ClipboardRead,
                Permission::ClipboardWrite,
            ],
            trusted: true,
        });

        PermissionGuard { 
            manifests,
            sandbox_root: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).to_string_lossy().to_string(),
        }
    }

    pub fn exec_whitelist(&self) -> Vec<String> {
        vec!["ls".to_string(), "cat".to_string(), "pwd".to_string(), "whoami".to_string(), "hostname".to_string(), "date".to_string(), "uptime".to_string(), "free".to_string(), "df".to_string(), "ps".to_string()]
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
