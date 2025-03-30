use crate::filesystem::models;
use chrono::{DateTime, Utc};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::panic::resume_unwind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tauri::command;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entries {
    directories: Vec<models::Directory>,
    files: Vec<models::File>
}

#[command]
fn get_entries_for_directory(directory: PathBuf) -> Entries {
    let mut result = Entries {
        directories: Vec::new(),
        files: Vec::new(),
    };
    
    let entries = fs::read_dir(directory).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        
        if metadata.is_dir() {
            let path = entry.path();
            let dir_struct = models::Directory {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path.to_str().unwrap().to_string(),
                is_symlink: path.is_symlink(),
                access_rights_as_string: access_rights_to_string(metadata.permissions().mode()),
                access_rights_as_number: metadata.permissions().mode(),
                size_in_bytes: get_directory_size_in_bytes(path.to_str().unwrap()),
                sub_file_count: count_subfiles_and_directories(path.to_str().unwrap()).0,
                sub_dir_count: count_subfiles_and_directories(path.to_str().unwrap()).1,
                created: format_system_time(metadata.created().unwrap()),
                last_modified: format_system_time(metadata.modified().unwrap()),
                accessed: format_system_time(metadata.accessed().unwrap()),
            };
            
            result.directories.push(dir_struct);
        } else {
            let path = entry.path();
            let file_struct = models::File {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path.to_str().unwrap().to_string(),
                is_symlink: path.is_symlink(),
                access_rights_as_string: access_rights_to_string(metadata.permissions().mode()),
                access_rights_as_number: metadata.permissions().mode(),
                size_in_bytes: metadata.len(),
                created: format_system_time(metadata.created().unwrap()),
                last_modified: format_system_time(metadata.modified().unwrap()),
                accessed: format_system_time(metadata.accessed().unwrap()),
            };
            
            result.files.push(file_struct);
        }
    }

    result
}

fn access_rights_to_string(mode: u32) -> String {
    let mut result = String::new();

    // User permissions
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Others permissions
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_directory_size_in_bytes(path: &str) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok) // Ignore errors
        .filter(|entry| entry.path().is_file()) // Only count files
        .map(|entry| {
            fs::metadata(entry.path())
                .map(|meta| meta.len())
                .unwrap_or(0)
        }) // Get file sizes
        .sum()
}

//The first is the number of files and the second is the number of directories
fn count_subfiles_and_directories(path: &str) -> (usize, usize) {
    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.path().is_file() {
            file_count += 1;
        } else if entry.path().is_dir() {
            dir_count += 1;
        }
    }

    (file_count, dir_count)
}

#[cfg(test)]
mod tests {
    use crate::commands::fs_dir_loader::get_entries_for_directory;
    use std::env;

    #[test]
    fn execute() {
        let directory = env::current_dir().unwrap();
        let entries = get_entries_for_directory(directory);
        println!("{}", serde_json::to_string(&entries).unwrap());
    }
}
