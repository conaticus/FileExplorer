// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod util;

use std::fs::{FileType, read_dir};
use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};
use util::conversions::{bytes_to_gb, os_to_string, path_to_string, ostr_to_string};

// struct State {
// }

#[derive(Serialize)]
struct Drive {
    name: String,
    used_gb: u16,
    total_gb: u16,
    letter: String,
}

#[derive(Serialize)]
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
        let used_bytes= disk.total_space() - disk.available_space();
        let used_gb = bytes_to_gb(used_bytes);
        let total_gb = bytes_to_gb(disk.total_space());

        let mut name = ostr_to_string(disk.name());
        if name.len() == 0 {
            name = String::from("Local Disk");
        }

        let letter = path_to_string(disk.mount_point());

        disks.push(Drive{name, used_gb, total_gb, letter});
    }

    disks
}

#[tauri::command]
fn open_directory(path: String) -> Vec<DirectoryChild> {
    let mut dir_children = Vec::new();

    let directory = read_dir(path);
    if !directory.is_ok() {
        return dir_children; // TODO(conaticus): handle error
    }

    for entry in directory.unwrap() {
        let entry = entry.unwrap();
        let file_name = os_to_string(entry.file_name());

        if entry.file_type().unwrap().is_file() {
            dir_children.push(DirectoryChild::File(file_name));
            continue;
        }

        dir_children.push(DirectoryChild::Directory(file_name));
    }

    dir_children
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_disks, open_disk])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
