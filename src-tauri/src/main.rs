// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;
mod file_explorer;

use file_explorer::{get_disks, open_directory};
use file_explorer::search::{search_directory};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_disks, open_directory, search_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
