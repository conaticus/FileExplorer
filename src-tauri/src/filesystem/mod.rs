pub mod cache;
pub mod volume;

use crate::filesystem::volume::DirectoryChild;
use std::fs::read_dir;
use std::io;
use std::io::{ErrorKind, Error};
use std::process::ExitStatus;

pub const DIRECTORY: &str = "directory";
pub const FILE: &str = "file";

pub const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}

/// Opens a file at the given path. Returns true if was successful
#[tauri::command]
pub async fn open_file(path: String) -> Result<bool, ()> {
    Ok(tokio::task::spawn_blocking(move || {
        open::commands(path)[0].status().is_ok()
    }).await.unwrap_or(false)) // NOTE(conaticus): I tried handling the errors nicely here but Tauri was mega cringe and wouldn't let me nest results in async functiona
}

/// Searches and returns the files in a given directory. This is not recursive.
#[tauri::command]
pub async fn open_directory(path: String) -> Result<Vec<DirectoryChild>, ()> {
    let Ok(directory) = read_dir(path) else {
        return Ok(Vec::new());
    };

    Ok(directory
        .map(|entry| {
            let entry = entry.unwrap();

            let file_name = entry.file_name().to_string_lossy().to_string();
            let entry_is_file = entry.file_type().unwrap().is_file();
            let entry = entry.path().to_string_lossy().to_string();

            if entry_is_file {
                return DirectoryChild::File(file_name, entry);
            }

            DirectoryChild::Directory(file_name, entry)
        })
        .collect())
}
