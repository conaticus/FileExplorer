use crate::models;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::Permissions;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Entries {
    pub(crate) directories: Vec<models::Directory>,
    pub(crate) files: Vec<models::File>,
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
pub fn get_access_permission_number(permissions: Permissions, _is_directory: bool) -> u32 {
    #[cfg(windows)]
    {
        // Unix-like octal for Windows-permissions
        if permissions.readonly() {
            return 0o444; // r--r--r--
        } else if _is_directory {
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
#[allow(unused_variables)]
pub fn get_access_permission_string(permissions: Permissions, is_directory: bool) -> String {
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
#[allow(dead_code)]
pub fn access_permission_string_windows(permission: Permissions, is_directory: bool) -> String {
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
pub fn access_rights_to_string_unix(permissions: Permissions) -> String {
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
pub fn format_system_time(system_time: SystemTime) -> String {
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
pub fn get_directory_size_in_bytes(path: &str) -> u64 {
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
pub fn count_subfiles_and_subdirectories(path: &str) -> (usize, usize) {
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
