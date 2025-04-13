use crate::filesystem::models;
use crate::filesystem::models::{
    count_subfiles_and_directories, format_system_time, get_access_permission_number,
    get_access_permission_string, get_directory_size_in_bytes, Entries,
};
use std::fs;
use std::fs::read_dir;
use std::path::Path;

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

/// Opens a directory at the given path and returns its contents as a json string.
///
/// # Arguments
/// - `path` - A string slice that holds the path to the directory to be opened.
///
/// # Returns
/// - `Ok(Entries)` - If the directory was successfully opened and read.
/// - `Err(String)` - If there was an error during the opening or reading process.
///
/// # Example
/// ```rust
/// let result = open_directory("/path/to/directory").await;
/// match result {
///    Ok(entries) => {
///       for dir in entries.directories {
///          println!("Directory: {}", dir.name);
///       }
///      for file in entries.files {
///         println!("File: {}", file.name);
///      }
///   },
///   Err(err) => println!("Error opening directory: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn open_directory(path: String) -> Result<String, String> {
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
        let file_type = entry
            .file_type()
            .map_err(|err| format!("Failed to get file type: {}", err))?;
        let path_of_entry = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|err| format!("Failed to get metadata: {}", err))?;

        let (subfile_count, subdir_count) =
            count_subfiles_and_directories(path_of_entry.to_str().unwrap());

        if file_type.is_dir() {
            directories.push(models::Directory {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path_of_entry.to_str().unwrap().to_string(),
                is_symlink: path_of_entry.is_symlink(),
                access_rights_as_string: get_access_permission_string(metadata.permissions(), true),
                access_rights_as_number: get_access_permission_number(metadata.permissions(), true),
                size_in_bytes: get_directory_size_in_bytes(path_of_entry.to_str().unwrap()),
                sub_file_count: subfile_count,
                sub_dir_count: subdir_count,
                created: format_system_time(metadata.created().unwrap()),
                last_modified: format_system_time(metadata.modified().unwrap()),
                accessed: format_system_time(metadata.accessed().unwrap()),
            });
        } else if file_type.is_file() {
            files.push(models::File {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path_of_entry.to_str().unwrap().to_string(),
                is_symlink: path_of_entry.is_symlink(),
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
            });
        }
    }

    let entries = Entries { directories, files };

    // Convert the Entries struct to a JSON string
    let json = serde_json::to_string(&entries)
        .map_err(|err| format!("Failed to serialize entries: {}", err))?;
    Ok(json)
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
pub async fn create_directory(path: &str, name: &str) -> Result<(), String> {
    // Check if the folder path exists and is valid
    let parent_path = Path::new(path);
    if !parent_path.exists() {
        return Err(format!("Parent directory does not exist: {}", path));
    }
    if !parent_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    // Concatenate the parent path and new directory name
    let dir_path = parent_path.join(name);

    // Create the directory
    match fs::create_dir(&dir_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to create directory: {}", err)),
    }
}

/// Renames a file or directory at the given path.
///
/// # Arguments
/// - `old_path` - The current path of the file or directory
/// - `new_path` - The new path for the file or directory
///
/// # Returns
/// - `Ok(())` if the rename operation was successful
/// - `Err(Error)` if there was an error during the operation
///
/// # Example
/// ```rust
/// let result = rename_file("/path/to/old_file.txt", "/path/to/new_file.txt").await;
/// match result {
///     Ok(_) => println!("File renamed successfully!"),
///     Err(err) => println!("Error renaming file: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn rename_file(old_path: &str, new_path: &str) -> Result<(), String> {
    // Check if the old path exists
    let old_path_obj = Path::new(old_path);
    if !old_path_obj.exists() {
        return Err(format!("File does not exist: {}", old_path));
    }

    // Check if the new path's parent directory exists
    let new_path_obj = Path::new(new_path);
    if let Some(parent) = new_path_obj.parent() {
        if !parent.exists() {
            return Err(format!(
                "Parent directory of destination path does not exist: {}",
                parent.display()
            ));
        }
    }

    // Perform the rename operation
    match fs::rename(old_path, new_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("File could not be renamed: {}", err)),
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

    #[tokio::test]
    async fn open_directory_test() {
        use std::io::Write;
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        println!("Temporary directory created: {:?}", temp_dir.path());

        // Create a subdirectory
        let sub_dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir_path).expect("Failed to create subdirectory");
        println!("Temporary subdirectory created: {:?}", sub_dir_path);

        // Create files in the root directory
        let file1_path = temp_dir.path().join("file1.txt");
        let mut file1 = fs::File::create(&file1_path).expect("Failed to create file1");
        writeln!(file1, "File 1 content").expect("Failed to write to file1");
        println!("File 1 created: {:?}", file1_path);

        let file2_path = temp_dir.path().join("file2.txt");
        let mut file2 = fs::File::create(&file2_path).expect("Failed to create file2");
        writeln!(file2, "File 2 content").expect("Failed to write to file2");
        println!("File 2 created: {:?}", file2_path);

        // Create files in the subdirectory
        let sub_file1_path = sub_dir_path.join("sub_file1.txt");
        let mut sub_file1 = fs::File::create(&sub_file1_path).expect("Failed to create sub_file1");
        writeln!(sub_file1, "Sub File 1 content").expect("Failed to write to sub_file1");
        println!("Sub File 1 created: {:?}", sub_file1_path);

        let sub_file2_path = sub_dir_path.join("sub_file2.txt");
        let mut sub_file2 = fs::File::create(&sub_file2_path).expect("Failed to create sub_file2");
        writeln!(sub_file2, "Sub File 2 content").expect("Failed to write to sub_file2");
        println!("Sub File 2 created: {:?}", sub_file2_path);

        // Call the open_directory function
        let result = open_directory(temp_dir.path().to_str().unwrap().to_string()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to open directory: {:?}", result);

        let entries = result.unwrap();
        let entries: Entries = serde_json::from_str(&entries).expect("Failed to parse JSON");

        // Verify directories
        assert_eq!(entries.directories.len(), 1, "Expected 1 subdirectory");
        assert_eq!(
            entries.directories[0].name, "subdir",
            "Subdirectory name does not match"
        );

        // Verify files in the root directory
        assert_eq!(
            entries.files.len(),
            2,
            "Expected 2 files in the root directory"
        );
        let file_names: Vec<String> = entries.files.iter().map(|f| f.name.clone()).collect();
        assert!(
            file_names.contains(&"file1.txt".to_string()),
            "file1.txt not found"
        );
        assert!(
            file_names.contains(&"file2.txt".to_string()),
            "file2.txt not found"
        );

        // Verify subdirectory contents
        let subdir_result = open_directory(sub_dir_path.to_str().unwrap().to_string()).await;
        assert!(
            subdir_result.is_ok(),
            "Failed to open subdirectory: {:?}",
            subdir_result
        );

        let subdir_entries = subdir_result.unwrap();
        let subdir_entries: Entries =
            serde_json::from_str(&subdir_entries).expect("Failed to parse JSON");
        assert_eq!(
            subdir_entries.files.len(),
            2,
            "Expected 2 files in the subdirectory"
        );
        let sub_file_names: Vec<String> = subdir_entries
            .files
            .iter()
            .map(|f| f.name.clone())
            .collect();
        assert!(
            sub_file_names.contains(&"sub_file1.txt".to_string()),
            "sub_file1.txt not found"
        );
        assert!(
            sub_file_names.contains(&"sub_file2.txt".to_string()),
            "sub_file2.txt not found"
        );
    }
}
