pub mod search;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, FileType, read_dir};
use std::io::{Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use sysinfo::{DiskExt, System, SystemExt};
use tauri::State;
use walkdir::WalkDir;
use crate::{CachedPath, StateSafe};
use crate::util::conversions::{bytes_to_gb, os_to_string, path_to_string, ostr_to_string};
use rayon::prelude::*;

const CACHE_FILE_PATH: &str = "./disk_cache.json";

#[derive(Serialize)]
pub struct Drive {
    name: String,
    available_gb: u16,
    used_gb: u16,
    total_gb: u16,
    letter: String,
}

#[derive(Serialize, Clone)]
pub enum DirectoryChild {
    File(String, String), // Name of file, path to file
    Directory(String, String)
}

pub fn save_cache_to_disk(state_mux: &StateSafe) {
    let state = &mut state_mux.lock().unwrap();
    let serialized_cache = serde_json::to_string(&state.drive_cache).unwrap();

    let mut file = fs::OpenOptions::new().write(true).open(CACHE_FILE_PATH).unwrap();
    file.write_all(serialized_cache.as_bytes()).unwrap();
}

pub fn cache_disk(state_mux: &StateSafe, path: &Path, letter: String) {
    let state = &mut state_mux.lock().unwrap();

    let disk_cache = state.drive_cache
        .entry(letter.clone())
        .or_insert_with(HashMap::new);

    let disk_cache = Arc::new(Mutex::new(disk_cache));

    WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .filter_map(|entry| entry.ok())
        .for_each(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_lowercase();
            let file_path = entry.path().to_string_lossy().to_string();

            let walkdir_filetype = entry.file_type();
            let mut file_type = String::from("file");
            if FileType::is_dir(&walkdir_filetype) {
                file_type = String::from("directory");
            }

            let cache_guard = &mut disk_cache.lock().unwrap();
            cache_guard.entry(file_name)
                .or_insert_with(|| Vec::new())
                .push(CachedPath { file_path, file_type });
        });
}

pub fn load_cache(state_mux: &StateSafe) {
    let state = &mut state_mux.lock().unwrap();
    let file_contents = fs::read_to_string(CACHE_FILE_PATH).unwrap();
    state.drive_cache = serde_json::from_str(&file_contents).unwrap();
}

#[tauri::command]
pub fn get_disks(state_mux: State<'_, StateSafe>) -> Vec<Drive> {
    let mut disks = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    let cache_exists = fs::metadata(CACHE_FILE_PATH).is_ok();
    if cache_exists {
        load_cache(&state_mux);
    } else {
        File::create(CACHE_FILE_PATH).unwrap();
    }

    for disk in sys.disks() {
        let used_bytes= disk.total_space() - disk.available_space();
        let available_gb = bytes_to_gb(disk.available_space());
        let used_gb = bytes_to_gb(used_bytes);
        let total_gb = bytes_to_gb(disk.total_space());

        let mut name = ostr_to_string(disk.name());
        if name.len() == 0 {
            name = String::from("Local Disk");
        }

        let mnt_point = disk.mount_point();

        let letter = path_to_string(mnt_point);
        if !cache_exists {
            cache_disk(&state_mux, mnt_point, letter.clone());
            save_cache_to_disk(&state_mux);
        }

        disks.push(Drive{name, available_gb, used_gb, total_gb, letter});
    }

    disks
}

#[tauri::command]
pub fn open_directory(path: String) -> Vec<DirectoryChild> {
    let mut dir_children = Vec::new();

    let directory = read_dir(path);
    if !directory.is_ok() {
        return dir_children;
    }

    for entry in directory.unwrap() {
        let entry = entry.unwrap();
        let file_name = os_to_string(entry.file_name());

        if entry.file_type().unwrap().is_file() {
            dir_children.push(DirectoryChild::File(file_name, entry.path().to_string_lossy().to_string()));
            continue;
        }

        dir_children.push(DirectoryChild::Directory(file_name, entry.path().to_string_lossy().to_string()));
    }

    dir_children
}
