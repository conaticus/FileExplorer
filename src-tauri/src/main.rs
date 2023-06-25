// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod file_explorer;
mod util;

use file_explorer::filesystem_ops::{get_disks, open_directory};
use file_explorer::search::search_directory;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct CachedPath {
    file_path: String,
    file_type: String,
}

pub type DiskCache = HashMap<String, Vec<CachedPath>>;

#[derive(Default)]
pub struct AppState {
    disk_cache: HashMap<String, DiskCache>,
}

pub type StateSafe = Arc<Mutex<AppState>>;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_disks,
            open_directory,
            search_directory
        ])
        .manage(Arc::new(Mutex::new(AppState::default())))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

