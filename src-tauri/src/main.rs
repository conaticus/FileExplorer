// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
pub mod constants;
mod filesystem;
mod state;

use tauri::ipc::Invoke;
use crate::commands::{file_system_operation_commands, meta_data_commands, settings_commands, volume_operations_commands,};

fn all_commands() -> fn(Invoke) -> bool {
    tauri::generate_handler![
        // Filesystem commands
        file_system_operation_commands::open_file,
        file_system_operation_commands::open_directory,
        file_system_operation_commands::create_file,
        file_system_operation_commands::create_directory,
        file_system_operation_commands::rename,
        file_system_operation_commands::move_to_trash,
        file_system_operation_commands::copy_file_or_dir,
        file_system_operation_commands::zip,
        file_system_operation_commands::unzip,

        // Metadata commands
        meta_data_commands::get_meta_data_as_json,
        meta_data_commands::update_meta_data,

        // Volume commands
        volume_operations_commands::get_system_volumes_information_as_json,
        volume_operations_commands::get_system_volumes_information,

        // Settings commands
        settings_commands::get_settings_as_json,
        settings_commands::update_settings_field,
        settings_commands::get_setting_field,
        settings_commands::update_multiple_settings_command,
        settings_commands::reset_settings_command,
    ]
}

#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(all_commands());

    // State-Setup ausgelagert in eigene Funktion
    let app = state::setup_app_state(app);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
