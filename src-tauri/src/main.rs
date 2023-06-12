// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Serialize)]
struct Drive {
    name: String,
    used_capacity: u64,
    total_capacity: u64,
}

enum DirectoryChild {
    File(String), // String is the path to the file
    Directory(String)
}

#[tauri::command]
fn get_disks() -> Vec<Drive> {
    let mut disks = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    for disk in sys.disks() {
        let available_capacity = disk.available_space();
        let total_capacity = disk.total_space();
        let used_capacity = total_capacity - available_capacity;

        disks.push(Drive{name: disk.name().to_string_lossy().to_string(), used_capacity, total_capacity});
    }

    disks
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_disks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
