// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod errors;
mod filesystem;
mod search;

use filesystem::explorer::{
    create_directory, create_file, delete_file, open_directory, open_file, rename_file,
};
use filesystem::volume::get_volumes;
use search::search_directory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct CachedPath {
    #[serde(rename = "p")]
    file_path: String,
    #[serde(rename = "t")]
    file_type: String,
}

pub type VolumeCache = HashMap<String, Vec<CachedPath>>;

#[derive(Default)]
pub struct AppState {
    system_cache: HashMap<String, VolumeCache>,
}

pub type StateSafe = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_volumes,
            open_directory,
            search_directory,
            open_file,
            create_file,
            create_directory,
            rename_file,
            delete_file
        ])
        .manage(Arc::new(Mutex::new(AppState::default())))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
