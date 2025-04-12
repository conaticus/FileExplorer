use crate::errors::Error;
use crate::filesystem::cache::FsEventHandler;
use crate::filesystem::fs_utils::get_mount_point;
use crate::filesystem::volume::DirectoryChild;
use crate::StateSafe;
use std::fs;
use std::fs::read_dir;
use std::ops::Deref;
use std::path::Path;
use tauri::State;
use crate::filesystem::models::{Entries, File};

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
    fs::read_to_string(path).map_err(|err| format!("Failed to read file: {}", err))
}

//TODO: impelemnt
#[tauri::command]
pub async fn open_directory(path: String) -> Result<Entries, String> {
    let path_obj = Path::new(&path);

    // Check if path exists
    if !path_obj.exists() {
        return Err(format!("Directory does not exist: {}", path));
    }

    // Check if path is a directory
    if !path_obj.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry in read_dir(path_obj).map_err(|err| format!("Failed to read directory: {}", err))? {
        let entry = entry.map_err(|err| format!("Failed to read entry: {}", err))?;
        let file_type = entry.file_type().map_err(|err| format!("Failed to get file type: {}", err))?;
        let path_of_entry = entry.path();
        
        if file_type.is_dir() {
            directories.push(File{
                name: entry.file_name().to_str().unwrap().to_string(),
                path: entry.path().to_str().unwrap().to_string(),
                is_symlink: path_of_entry.is_symlink(),
                access_rights_as_string: "".to_string(),
                access_rights_as_number: 0,
                size_in_bytes: 0,
                created: "".to_string(),
                last_modified: "".to_string(),
                accessed: "".to_string(),
            });
        } else if file_type.is_file() {
            files.push(DirectoryChild::new(entry.path()));
        }
    }

    Ok(Entries { directories, files })

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
        return Err(format!("Directory does not exist: {}", folder_path_abs));
    }
    if !path.is_dir() {
        return Err(format!("Path is no directory: {}", folder_path_abs));
    }

    // Concatenate the folder path and filename
    let file_path = path.join(filename);

    // Create the file
    match fs::File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("File could not be created: {}", err)),
    }
}

#[tauri::command]
pub async fn create_directory(path: &str, name: &str) -> Result<(), Error> {
    // Check if the folder path exists and is valid
    let parent_path = Path::new(path);
    if !parent_path.exists() {
        return Err(Error::Custom(format!(
            "Parent directory does not exist: {}",
            path
        )));
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
    async fn open_file_test() {
        use std::io::Write;
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("open_file_test.txt");

        // Write some content to the test file
        let mut test_file = fs::File::create(&test_path).expect("Failed to create test file");
        writeln!(test_file, "Hello, world!").expect("Failed to write to test file");

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before reading");

        // Open the file and read its contents
        let result = open_file(test_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to open file: {:?}", result);

        // Verify the file contents
        assert_eq!(
            result.unwrap(),
            "Hello, world!\n",
            "File contents do not match expected value"
        );
    }

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
        assert!(
            !test_path.exists(),
            "File should no longer exist at the original path"
        );

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
        let result = create_file(temp_dir.path().to_str().unwrap(), "create_file_test.txt").await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to create file: {:?}", result);

        // Verify that the file exists at the specified pat´ßp0
        assert!(test_path.exists(), "File should exist after creation");
    }
}
