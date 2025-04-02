use crate::filesystem::models;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::SystemTime;
use tauri::command;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entries {
    directories: Vec<models::Directory>,
    files: Vec<models::File>,
}

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
    let directory = PathBuf::from(directory);
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
                access_rights_as_string: get_access_permission_string(metadata.permissions(), true),
                access_rights_as_number: get_access_permission_number(metadata.permissions(), true),
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

            result.files.push(file_struct);
        }
    }

    result
}

/// This function retrieves the access permissions of a file or directory.
/// It returns the permissions as a number.
/// It takes into account the platform (Windows or Unix) and formats the permissions accordingly.
///
/// # Parameters
/// - `permissions`: The permissions of the file or directory.
/// - `is_directory`: A boolean indicating whether the entry is a directory or not.
///
/// # Returns
/// A u32 representing the access permissions.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_access_permission_number;
/// use std::fs::Permissions;
/// use std::os::unix::fs::PermissionsExt;
///
/// fn main() {
///  let permissions = Permissions::from_mode(0o755);
///  let is_directory = true;
///  let permission_number = get_access_permission_number(permissions, is_directory);
///  println!("Access permissions number: {}", permission_number);
/// }
fn get_access_permission_number(permissions: Permissions, is_directory: bool) -> u32 {
    #[cfg(windows)]
    {
        // Unix-like Oktale für Windows-Berechtigungen
        if permissions.readonly() {
            return 0o444; // r--r--r--
        } else if is_directory {
            return 0o755; // rwxr-xr-x
        } else {
            return 0o666; // rw-rw-rw-
        }
    }
    #[cfg(unix)]
    {
        let mode = permissions.mode();
        mode
    }
}

/// This function converts the access permissions of a file or directory into a human-readable string.
/// It takes into account the platform (Windows or Unix) and formats the permissions accordingly.
///
/// # Parameters
/// - `permissions`: The permissions of the file or directory.
/// - `is_directory`: A boolean indicating whether the entry is a directory or not.
///
/// # Returns
/// A string representing the access permissions in a human-readable format.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_access_permission_string;
/// use std::fs::Permissions;
/// use std::os::unix::fs::PermissionsExt;
///
/// fn main() {
///   let permissions = Permissions::from_mode(0o755);
///   let is_directory = true;
///   let permission_string = get_access_permission_string(permissions, is_directory);
///   println!("Access permissions: {}", permission_string);
/// }
/// ```
fn get_access_permission_string(permissions: Permissions, is_directory: bool) -> String {
    #[cfg(windows)]
    {
        access_permission_string_windows(permissions, is_directory)
    }
    #[cfg(unix)]
    {
        access_rights_to_string_unix(permissions)
    }
}

/// This function converts the access permissions of a file or directory into a human-readable string.
/// It takes into account the platform (Windows or Unix) and formats the permissions accordingly.
///
/// # Parameters
/// - `permissions`: The permissions of the file or directory.
/// - `is_directory`: A boolean indicating whether the entry is a directory or not.
///
/// # Returns
/// A string representing the access permissions in a human-readable format.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_access_permission_string;
/// use std::fs::Permissions;
/// use std::os::unix::fs::PermissionsExt;
///
/// fn main() {
///  let permissions = Permissions::from_mode(0o755);
///  let is_directory = true;
///  let permission_string = get_access_permission_string(permissions, is_directory);
/// println!("Access permissions: {}", permission_string);
/// }
/// ```
fn access_permission_string_windows(permission: Permissions, is_directory: bool) -> String {
    // Standardmäßig Leserechte für alle
    let mut result = String::from("r--r--r--");

    // Wenn nicht schreibgeschützt, füge Schreibrechte für alle hinzu
    if !permission.readonly() {
        result = String::from("rw-rw-rw-");
    }

    // Wenn es sich um ein Verzeichnis handelt, füge Ausführungsrechte hinzu
    if is_directory {
        result = String::from("rwxr-xr-x");
    }

    result
}

/// This function converts the access permissions of a file or directory into a human-readable string.
/// It takes into account the platform (Windows or Unix) and formats the permissions accordingly.
///
/// # Parameters
/// - `permissions`: The permissions of the file or directory.
/// - `is_directory`: A boolean indicating whether the entry is a directory or not.
///
/// # Returns
/// A string representing the access permissions in a human-readable format.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_access_permission_string;
/// use std::fs::Permissions;
/// use std::os::unix::fs::PermissionsExt;
///
/// fn main() {
///  let permissions = Permissions::from_mode(0o755);
///  let is_directory = true;
///  let permission_string = get_access_permission_string(permissions, is_directory);
///  println!("Access permissions: {}", permission_string);
/// }
/// ```
#[cfg(unix)]
fn access_rights_to_string_unix(permissions: Permissions) -> String {
    let mut result = String::new();
    let mode = permissions.mode();

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

/// This function formats a SystemTime object into a human-readable string.
/// It converts the SystemTime into a DateTime<Utc> object and then formats it into a string.
///
/// # Parameters
/// - `system_time`: The SystemTime object to be formatted.
///
/// # Returns
/// A string representing the formatted date and time.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::format_system_time;
/// use std::time::SystemTime;
///
/// fn main() {
///  let system_time = SystemTime::now();
///  let formatted_time = format_system_time(system_time);
///  println!("Formatted time: {}", formatted_time);
/// }
fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// This function calculates the size of a directory in bytes.
/// It uses the WalkDir crate to recursively walk through the directory and sum up the sizes of all files.
///
/// # Parameters
/// - `path`: The path of the directory to calculate the size for.
///
/// # Returns
/// The total size of the directory in bytes.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::get_directory_size_in_bytes;
/// use std::fs;
///
/// fn main() {
///  let path = "/path/to/directory";
///  let size = get_directory_size_in_bytes(path);
///  println!("Directory size: {} bytes", size);
/// }
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

/// This function counts the number of files and directories in a given path.
/// It uses the WalkDir crate to recursively walk through the directory and count the entries.
///
/// # Parameters
/// - `path`: The path of the directory to count the entries for.
///
/// # Returns
/// A tuple containing the number of files and directories. Where the first is the number of files and the second is the number of directories.
///
/// # Example
/// ```rust
/// use crate::commands::fs_dir_loader_commands::count_subfiles_and_directories;
/// use std::env;
///
/// fn main() {
///  let path = env::current_dir().unwrap().to_str().unwrap().to_string();
///  let (file_count, dir_count) = count_subfiles_and_directories(&path);
///  println!("Files: {}, Directories: {}", file_count, dir_count);
/// }
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
