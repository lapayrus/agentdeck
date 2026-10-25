#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::load())
        .invoke_handler(tauri::generate_handler![
            commands::scan,
            commands::set_project,
            commands::get_settings,
            commands::save_settings,
            commands::read_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running AgentDeck");
}
