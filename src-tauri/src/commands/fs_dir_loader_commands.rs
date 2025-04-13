use crate::filesystem::models;
use crate::filesystem::models::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tauri::command;

//TODO maybe redundant to file_system_operation_commands

/// This function retrieves the entries (files and directories) for a given directory.
/// It returns a struct containing the entries, including their metadata such as name, path, access rights, size, and timestamps.
/// Parameters are
/// - `directory`: The path of the directory to retrieve entries from. Example: "/home/user/documents". or "C:\\Users\\user\\Documents" or C:/Users/user/Documents/subdir"
/// Returns a struct containing the entries, including their metadata such as name, path, access rights, size, and timestamps.
/// # Example
///
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_entries_for_directory;
/// use std::env;
///
/// fn main() {
///   let directory = env::current_dir().unwrap().to_str().unwrap().to_string();
///   let entries = get_entries_for_directory(directory);
///   println!("{}", serde_json::to_string(&entries).unwrap());
/// }
/// ```
#[command]
fn get_entries_for_directory(directory: String) -> Entries {
    //TODO adequate error handling
    let directory = PathBuf::from(directory);

    let mut directories = Vec::new();
    let mut files = Vec::new();

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
                access_rights_as_string: get_access_permission_string(metadata.permissions(), true),
                access_rights_as_number: get_access_permission_number(metadata.permissions(), true),
                size_in_bytes: get_directory_size_in_bytes(path.to_str().unwrap()),
                sub_file_count: count_subfiles_and_directories(path.to_str().unwrap()).0,
                sub_dir_count: count_subfiles_and_directories(path.to_str().unwrap()).1,
                created: format_system_time(metadata.created().unwrap()),
                last_modified: format_system_time(metadata.modified().unwrap()),
                accessed: format_system_time(metadata.accessed().unwrap()),
            };

            directories.push(dir_struct);
        } else {
            let path = entry.path();
            let file_struct = models::File {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path.to_str().unwrap().to_string(),
                is_symlink: path.is_symlink(),
                access_rights_as_string: get_access_permission_string(
                    metadata.permissions(),
                    false,
                ),
                access_rights_as_number: get_access_permission_number(
                    metadata.permissions(),
                    false,
                ),
                size_in_bytes: metadata.len(),
                created: format_system_time(metadata.created().unwrap()),
                last_modified: format_system_time(metadata.modified().unwrap()),
                accessed: format_system_time(metadata.accessed().unwrap()),
            };

            files.push(file_struct);
        }
    }

    Entries { directories, files }
}

#[cfg(test)]
mod tests {
    use crate::commands::fs_dir_loader_commands::get_entries_for_directory;
    use std::env;

    #[test]
    fn execute_dir_loader() {
        env_logger::init();
        //directory of execution
        let directory = env::current_dir().unwrap().to_str().unwrap().to_string();
        println!("Execution in directory: {}", directory);
        let entries = get_entries_for_directory(directory);
        println!("{}", serde_json::to_string(&entries).unwrap());

        //let string = serde_json::to_string(&entries).unwrap();

        //println!("directory-path: {}", serde_json::from_str::<Entries>(&string).unwrap().directories[0].path);
    }
}
