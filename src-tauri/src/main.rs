// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod filesystem;
mod search;

use filesystem::{ get_volumes, open_directory };
use search::search_directory;
use serde::{ Deserialize, Serialize };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use tauri::Manager;
use window_vibrancy::{ apply_blur, apply_vibrancy, NSVisualEffectMaterial };

#[derive(Serialize, Deserialize)]
pub struct CachedPath {
    file_path: String,
    file_type: String,
}

pub type VolumeCache = HashMap<String, Vec<CachedPath>>;

#[derive(Default)]
pub struct AppState {
    system_cache: HashMap<String, VolumeCache>,
}

pub type StateSafe = Arc<Mutex<AppState>>;

fn main() {
    tauri::Builder
        ::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();

            #[cfg(target_os = "macos")]
            apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, None).expect(
                "Unsupported platform! 'apply_vibrancy' is only supported on macOS"
            );

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((18, 18, 18, 125))).expect(
                "Unsupported platform! 'apply_blur' is only supported on Windows"
            );

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_volumes, open_directory, search_directory])
        .manage(Arc::new(Mutex::new(AppState::default())))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
