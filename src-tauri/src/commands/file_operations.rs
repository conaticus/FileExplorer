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


//TODO: impelemnt
/// Opens a file at the given path. Returns a string if there was an error.
// NOTE(conaticus): I tried handling the errors nicely here but Tauri was mega cringe and wouldn't let me nest results in async functions, so used string error messages instead.
#[tauri::command]
pub async fn open_file(path: String) -> Result<(), Error> {
    let output_res = open::commands(path)[0].output();
    let output = match output_res {
        Ok(output) => output,
        Err(err) => {
            let err_msg = format!("Failed to get open command output: {}", err);
            return Err(Error::Custom(err_msg));
        }
    };

    if output.status.success() {
        return Ok(());
    }

    let err_msg = String::from_utf8(output.stderr)
        .unwrap_or(String::from("Failed to open file and deserialize stderr."));
    Err(Error::Custom(err_msg))
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
pub async fn create_file(file_path_abs: &str) -> Result<(), String> {
    //write a simple function, which gets an abs filepath and creates a file at this path 
    match fs::File::create(&file_path_abs) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to create file: {}", err)),
    }
}

//TODO: impelemnt
#[tauri::command]
pub async fn create_directory(state_mux: State<'_, StateSafe>, path: String) -> Result<(), Error> {
    let mount_point_str = get_mount_point(path.clone()).unwrap_or_default();

    let fs_event_manager = FsEventHandler::new(state_mux.deref().clone(), mount_point_str.into());
    fs_event_manager.handle_create(CreateKind::Folder, Path::new(&path));

    let res = fs::create_dir(path);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Custom(err.to_string())),
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