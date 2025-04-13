// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
pub mod constants;
mod filesystem;
mod state;

use serde::{Deserialize, Serialize};
use tauri::ipc::Invoke;
use crate::commands::file_system_operation_commands::{create_directory, create_file, move_file_to_trash, open_directory, open_file, rename_file};
use crate::commands::meta_data_commands::get_meta_data;

#[derive(Serialize, Deserialize)]
pub struct CachedPath {
    #[serde(rename = "p")]
    file_path: String,
    #[serde(rename = "t")]
    file_type: String,
}

fn all_commands() -> fn(Invoke) -> bool {
    tauri::generate_handler![
        open_file,
        open_directory,
        create_file,
        create_directory,
        rename_file,
        move_file_to_trash,
        get_meta_data,
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
