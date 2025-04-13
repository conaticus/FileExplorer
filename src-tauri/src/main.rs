// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
pub mod constants;
mod filesystem;
mod search;
mod state;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::ipc::Invoke;

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

fn all_commands() -> fn(Invoke) -> bool {
    tauri::generate_handler![]
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
