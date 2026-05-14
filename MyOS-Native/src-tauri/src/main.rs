// Yfitops OS v8.0 Native Vision - Core Gateway (Rust)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod state;
mod commands;
mod db;

use crate::state::AppState;
use crate::commands::{window, health, terminal, files, settings, logs, notes, workspace, session, theme, packages, privacy, ai};

fn main() {
    // Database Initialization
    let db_path = "yfitops.db"; 
    let db_conn = db::init_db(std::path::Path::new(db_path)).expect("Failed to initialize database");

    tauri::Builder::default()
        .manage(AppState::new(db_conn))
        .invoke_handler(tauri::generate_handler![
            window::open_module,
            health::get_system_health,
            terminal::exec_command,
            files::list_dir,
            files::read_text_file,
            settings::get_setting,
            settings::set_setting,
            settings::list_settings,
            logs::write_audit,
            logs::query_audit,
            notes::list_notes,
            notes::save_note,
            notes::delete_note,
            notes::search_notes,
            workspace::save_workspace_state,
            workspace::load_workspace_state,
            workspace::toggle_focus_mode,
            session::set_session_mode,
            theme::set_wallpaper,
            packages::install_package,
            packages::search_system_packages,
            packages::install_system_package,
            privacy::get_privacy_status,
            privacy::get_privacy_heatmap,
            privacy::toggle_network_guardian,
            ai::query_assistant,
            ai::get_ai_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
