// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::toggle_console,
            commands::change_language
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}