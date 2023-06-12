// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::ffi::{OsStr};
use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};

#[derive(Serialize)]
struct Drive {
    name: String,
    used_gb: u16,
    total_gb: u16,
}

enum DirectoryChild {
    File(String), // String is the path to the file
    Directory(String)
}

fn os_to_string(os_string: &OsStr) -> String {
   os_string.to_string_lossy().to_string()
}

fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}

#[tauri::command]
fn get_disks() -> Vec<Drive> {
    let mut disks = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    for disk in sys.disks() {
        let used_bytes= disk.total_space() - disk.available_space();
        let used_gb = bytes_to_gb(used_bytes);
        let total_gb = bytes_to_gb(disk.total_space());

        let mut name = os_to_string(disk.name());
        if name.len() == 0 {
            name = String::from("Local Disk");
        }

        disks.push(Drive{name, used_gb, total_gb});
    }

    disks
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_disks])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
