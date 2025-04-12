use crate::errors::Error;
use crate::filesystem::cache::FsEventHandler;
use crate::filesystem::fs_utils::get_mount_point;
use crate::filesystem::volume::DirectoryChild;
use crate::StateSafe;
use notify::event::CreateKind;
use std::fs;
use std::fs::read_dir;
use std::ops::Deref;
use std::path::{Path};
use tauri::State;


/// Opens a file at the given path and returns its contents as a string.
/// Should only be used for text files.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the file to be opened.
///
/// # Returns
///
/// * `Ok(String)` - If the file was successfully opened and read.
/// * `Err(String)` - If there was an error during the opening or reading process.
///
/// # Example
///
/// ```rust
/// let result = open_file("/path/to/file.txt").await;
/// match result {
///     Ok(contents) => println!("File contents: {}", contents),
///     Err(err) => println!("Error opening file: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn open_file(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);
    
    // Check if path exists
    if !path_obj.exists() {
        return Err(format!("File does not exist: {}", path));
    }
    
    // Check if path is a file
    if !path_obj.is_file() {
        return Err(format!("Path is not a file: {}", path));
    }
    
    // Read the file
    fs::read_to_string(path)
        .map_err(|err| format!("Failed to read file: {}", err))
}

//TODO: impelemnt
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

/// Creates a file at the given absolute path. Returns a string if there was an error.
/// This function does not create any parent directories.
/// 
/// # Arguments
/// - `file_path_abs` - A string slice that holds the absolute path to the file to be created.
/// 
/// # Returns
/// - `Ok(())` if the file was successfully created.
/// - `Err(String)` if there was an error during the creation process.
/// 
/// # Example
/// ```rust
/// let result = create_file("/path/to/file.txt").await;
/// match result {
///     Ok(_) => println!("File created successfully!"),
///     Err(err) => println!("Error creating file: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn create_file(folder_path_abs: &str, filename: &str) -> Result<(), String> {
    // Check if the folder path exists and is valid
    let path = Path::new(folder_path_abs);
    if !path.exists() {
        return Err(format!("Verzeichnis existiert nicht: {}", folder_path_abs));
    }
    if !path.is_dir() {
        return Err(format!("Pfad ist kein Verzeichnis: {}", folder_path_abs));
    }
    
    // Concatenate the folder path and filename
    let file_path = path.join(filename);
    
    // Create the file
    match fs::File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Datei konnte nicht erstellt werden: {}", err)),
    }
}

#[tauri::command]
pub async fn create_directory(path: &str, name: &str) -> Result<(), Error> {
    // Check if the folder path exists and is valid
    let parent_path = Path::new(path);
    if !parent_path.exists() {
        return Err(Error::Custom(format!("Parent directory does not exist: {}", path)));
    }
    if !parent_path.is_dir() {
        return Err(Error::Custom(format!("Path is not a directory: {}", path)));
    }
    
    // Concatenate the parent path and new directory name
    let dir_path = parent_path.join(name);
    
    // Create the directory
    match fs::create_dir(&dir_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Io(err)),
    }
}

//TODO: impelemnt
#[tauri::command]
pub async fn rename_file(
    state_mux: State<'_, StateSafe>,
    old_path: String,
    new_path: String,
) -> Result<(), Error> {
    let mount_point_str = get_mount_point(old_path.clone()).unwrap_or_default();

    let mut fs_event_manager =
        FsEventHandler::new(state_mux.deref().clone(), mount_point_str.into());
    fs_event_manager.handle_rename_from(Path::new(&old_path));
    fs_event_manager.handle_rename_to(Path::new(&new_path));

    let res = fs::rename(old_path, new_path);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Custom(err.to_string())),
    }
}


/// Deletes a file at the given path. Returns a string if there was an error.
/// This function moves the file to the trash instead of deleting it permanently.
/// 
/// # Arguments
/// - `path` - A string slice that holds the path to the file to be deleted.
/// 
/// # Returns
/// - `Ok(())` if the file was successfully deleted.
/// - `Err(String)` if there was an error during the deletion process.
/// 
/// # Example
/// ```rust
/// let result = delete_file("/path/to/file.txt").await;
/// match result {
///   Ok(_) => println!("File deleted successfully!"),
///   Err(err) => println!("Error deleting file: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn move_file_to_trash(path: &str) -> Result<(), String> {
    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to delete (move to trash)file: {}", err)),
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn move_file_to_trash_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("move_to_trash_test.txt");

        // Create the test file
        fs::File::create(&test_path).unwrap();

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before deletion");

        eprintln!("Test file exists: {:?}", test_path);
        
        // Move the file to the trash
        let result = move_file_to_trash(test_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to move file to trash: {:?}", result);

        // Verify that the file no longer exists at the original path
        assert!(!test_path.exists(), "File should no longer exist at the original path");

        // No manual cleanup needed, as the temporary directory is automatically deleted
    }
    
    #[tokio::test]
    async fn create_file_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file path in the temporary directory
        let test_path = temp_dir.path().join("create_file_test.txt");

        // Call the function to create the file
        let result = create_file(test_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to create file: {:?}", result);

        // Verify that the file exists at the specified path
        assert!(test_path.exists(), "File should exist after creation");
    }
}