pub mod cache;
pub mod volume;

use std::fs::{read_dir};
use crate::filesystem::volume::DirectoryChild;

pub const DIRECTORY: &str = "directory";
pub const FILE: &str = "file";

pub const fn bytes_to_gb(bytes: u64) -> u16 { (bytes / (1e+9 as u64)) as u16 }

/// Searches and returns the files in a given directory. This is not recursive.
#[tauri::command]
pub fn open_directory(path: String) -> Vec<DirectoryChild> {
    let mut dir_children = Vec::new();

    let Ok(directory) = read_dir(path) else {
        return dir_children;
    };

    for entry in directory {
        let entry = entry.unwrap();

        let file_name = entry.file_name().to_str().unwrap().to_string();
        let entry_is_file = entry.file_type().unwrap().is_file();
        let entry = entry.path().to_str().unwrap().to_string();

        if entry_is_file {
            dir_children.push(DirectoryChild::File(file_name, entry));
            continue;
        }

        dir_children.push(DirectoryChild::Directory(file_name, entry));
    }

    dir_children
}
