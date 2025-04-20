use crate::filesystem::models;
use crate::filesystem::models::{
    count_subfiles_and_subdirectories, format_system_time, get_access_permission_number,
    get_access_permission_string, get_directory_size_in_bytes, Entries,
};
use std::{fs};
use tokio::task;
use std::fs::read_dir;
use std::future::Future;
use std::io::Write;
use std::path::Path;
use std::pin::Pin;
use rand::Rng;

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
            count_subfiles_and_subdirectories(path_of_entry.to_str().unwrap());

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
                access_rights_as_number: get_access_permission_number(metadata.permissions(), false),
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
pub async fn create_file(folder_path_abs: &str, file_name: &str) -> Result<(), String> {
    // Check if the folder path exists and is valid
    let path = Path::new(folder_path_abs);
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", folder_path_abs));
    }
    if !path.is_dir() {
        return Err(format!("Path is no directory: {}", folder_path_abs));
    }

    // Concatenate the folder path and filename
    let file_path = path.join(file_name);

    // Create the file
    match fs::File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("File could not be created: {}", err)),
    }
}

/// Creates a directory at the given absolute path. Returns a string if there was an error.
/// This function does not create any parent directories.
/// 
/// # Arguments
/// - `folder_path_abs` - A string slice that holds the absolute path to the directory to be created.
/// 
/// # Returns
/// - `Ok(())` if the directory was successfully created.
/// - `Err(String)` if there was an error during the creation process.
/// 
/// # Example
/// ```rust
/// let result = create_directory("/path/to/directory", "new_folder").await;
/// match result {
///     Ok(_) => println!("Directory created successfully!"),
///     Err(err) => println!("Error creating directory: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn create_directory(folder_path_abs: &str, folder_name: &str) -> Result<(), String> {
    // Check if the folder path exists and is valid
    let parent_path = Path::new(folder_path_abs);
    if !parent_path.exists() {
        return Err(format!("Parent directory does not exist: {}", folder_path_abs));
    }
    if !parent_path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path_abs));
    }

    // Concatenate the parent path and new directory name
    let dir_path = parent_path.join(folder_name);

    // Create the directory
    match fs::create_dir(&dir_path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to create directory: {}", err)),
    }
}

/// Renames a file or directory at the given path.
///
/// # Arguments
/// - `path` - The current path of the file or directory
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
pub async fn rename(old_path: &str, new_path: &str) -> Result<(), String> {
    let old_path_obj = Path::new(old_path);
    let new_path_obj = Path::new(new_path);

    // Check if the old path exists
    if !old_path_obj.exists() {
        return Err(format!("File does not exist: {}", old_path));
    }

    // Check if the new path is valid
    if new_path_obj.exists() {
        return Err(format!("New path already exists: {}", new_path));
    }

    // Rename the file or directory
    fs::rename(old_path, new_path).map_err(|err| format!("Failed to rename: {}", err))
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
pub async fn move_to_trash(path: &str) -> Result<(), String> {
    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Failed to move file or directory to trash: {}", err)),
    }
}

/// Copies a file or directory from the source path to the destination path.
/// This function does not create any parent directories.
/// It will overwrite the destination if it already exists.
/// If the source is a directory, it will recursively copy all files and subdirectories.
/// 
/// # Arguments
/// - `source_path` - A string slice that holds the path to the source file or directory.
/// - `destination_path` - A string slice that holds the path to the destination.
/// 
/// # Returns
/// - `Ok(u64)` - The total size of copied files in bytes.
/// - `Err(String)` - If there was an error during the copy process.
/// 
/// # Example
/// ```rust
/// let result = copy_file_or_dir("/path/to/source.txt", "/path/to/destination.txt").await;
/// match result {
///     Ok(size) => println!("File copied successfully! Size: {} bytes", size),
///     Err(err) => println!("Error copying file: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn copy_file_or_dir(source_path: &str, destination_path: &str) -> Result<u64, String> {
    // Check if the source path exists
    if !Path::new(source_path).exists() {
        return Err(format!("Source path does not exist: {}", source_path));
    }

    // Check if the destination path is valid
    if Path::new(destination_path).exists() {
        return Err(format!("Destination path already exists: {}", destination_path));
    }
    
    if Path::new(source_path).is_dir() {
        // If the source is a directory, recursively copy it
        let mut total_size = 0;
        
        // Create the destination directory
        fs::create_dir_all(destination_path)
            .map_err(|err| format!("Failed to create destination directory: {}", err))?;
        
        // Read all entries in the source directory
        for entry in fs::read_dir(source_path)
            .map_err(|err| format!("Failed to read source directory: {}", err))? {
            
            let entry = entry.map_err(|err| format!("Failed to read directory entry: {}", err))?;
            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = Path::new(destination_path).join(file_name);
            
            if entry_path.is_file() {
                // Copy file
                let size = fs::copy(&entry_path, &dest_path)
                    .map_err(|err| format!("Failed to copy file '{}': {}", entry_path.display(), err))?;
                total_size += size;
            } else if entry_path.is_dir() {
                // Recursively copy subdirectory
                let sub_size = Box::pin(copy_file_or_dir(
                    entry_path.to_str().unwrap(),
                    dest_path.to_str().unwrap()
                )).await?;
                total_size += sub_size;
            }
        }
        
        Ok(total_size)
    } else {
        // Copy a single file
        let size = fs::copy(source_path, destination_path)
            .map_err(|err| format!("Failed to copy file: {}", err))?;
        Ok(size)
    }
}

// checks whether path is system relevant
#[cfg(target_os = "windows")]
fn is_protected_path(path: &str) -> bool {
    let protected_paths = [
        "C:\\Windows",
        "C:\\Program Files",
        "C:\\Program Files (x86)",
        "C:\\__PROTECTED_TEST_PATH__",
    ];

    for protected_path in protected_paths.iter() {
        if path.starts_with(protected_path) {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
fn is_protected_path(path: &str) -> bool {
    let protected_paths = [
        "/etc",
        "/bin",
        "/sys",
        "/usr",
        "/__protected_test_path__",
    ];

    for protected_path in protected_paths.iter() {
        if path.starts_with(protected_path) {
            return true;
        }
    }

    false
}

#[cfg(target_os = "macos")]
fn is_protected_path(path: &str) -> bool {
    let protected_paths = [
        "/System",
        "/Applications",
        "/usr",
        "/bin",
        "/Library",
        "/__protected_test_path__",
    ];

    for protected_path in protected_paths.iter() {
        if path.starts_with(protected_path) {
            return true;
        }
    }

    false
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
fn is_protected_path(path: &str) -> bool {
    // For other OSes, treat all paths as unprotected by default
    false
}

/// Securely deletes a file or directory by overwriting its contents multiple times and then removing it.
/// This function allows you to securely delete a file or a directory and its contents by overwriting
/// the data with ones, zeros, and random data. It supports both files and directories.
/// If the path is a directory, it will recursively delete all files and subdirectories before deleting the directory itself.
///
/// # Arguments
/// - `path` - A string slice that holds the path to the file or directory to be deleted.
/// - `passes` - An optional u64 specifying the number of overwrite passes. Default is 3 passes if not provided.
///
/// # Returns
/// - `Ok(())` - If the file or directory is securely deleted.
/// - `Err(String)` - If there was an error during the deletion process.
///
/// # Example
/// ```rust
/// let result = safe_delete_file_or_dir("/path/to/file_or_dir", Some(3)).await;
/// match result {
///     Ok(_) => println!("File/Directory securely deleted!"),
///     Err(err) => println!("Error deleting file/dir: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn safe_delete_file_or_dir(path: &str, passes: Option<u64>) -> Result<(), String> {
    if is_protected_path(&path) {
        return Err(format!("Refusing to delete system-protected path: {}", path));
    }
    // TODO: Maybe add option to choose between methods for safe delete (e.g. zero-fill, one-fill, random-fill) currently used a method of first writing zeros then ones and then random stuff but idk tho
    let path = path.to_string();
    task::spawn_blocking(move || {
        let path_obj = Path::new(&path);

        if !path_obj.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        // If the path is a file, delete it securely
        if path_obj.is_file() {
            let result = tokio::runtime::Handle::current().block_on(secure_delete_file(path.clone(), passes));
            return result.map_err(|e| e.to_string());
        }

        // If the path is a directory, delete it securely, including all contents
        if path_obj.is_dir() {
            let result = tokio::runtime::Handle::current().block_on(secure_delete_directory(path.clone(), passes));
            return result.map_err(|e| e.to_string());
        }

        Err(format!("Path is neither a file nor a directory: {}", path))
    }).await.map_err(|e| e.to_string())?
}

/// Securely deletes a file by overwriting it multiple times with random data.
async fn secure_delete_file(path: String, passes: Option<u64>) -> Result<(), String> {
    // Default to 3 passes if none is provided
    let passes = passes.unwrap_or(3);

    // Define the block size (1 MB per block)
    let block_size: u64 = 1024 * 1024;

    // Open file
    let mut file = match fs::File::create(&path) {
        Ok(f) => f,
        Err(err) => return Err(format!("Failed to open file for overwriting: {}", err)),
    };

    // First pass: overwrite with zeros (0x00)
    let zero_block = vec![0u8; block_size as usize];
    if let Err(e) = file.write_all(&zero_block) {
        return Err(format!("Failed to overwrite with zeros: {}", e));
    }

    if passes == 1 {
        // remove file
        if let Err(e) = fs::remove_file(&path) {
            return Err(format!("Failed to delete file: {}", e));
        }
        return Ok(());
    }

    // Second pass: overwrite with ones (0xFF)
    let one_block = vec![255u8; block_size as usize];
    if let Err(e) = file.write_all(&one_block) {
        return Err(format!("Failed to overwrite with ones: {}", e));
    }

    if passes == 2 {
        // remove file
        if let Err(e) = fs::remove_file(&path) {
            return Err(format!("Failed to delete file: {}", e));
        }
        return Ok(());
    }

    // Third and until passes: overwrite with random data
    let mut rng = rand::thread_rng();
    for _ in 2..passes {
        let random_block: Vec<u8> = (0..block_size)
            .map(|_| rng.gen())
            .collect();
        if let Err(e) = file.write_all(&random_block) {
            return Err(format!("Failed to overwrite with random data: {}", e));
        }
    }

    // remove file
    if let Err(e) = fs::remove_file(&path) {
        return Err(format!("Failed to delete file: {}", e));
    }

    Ok(())
}

/// Securely deletes a directory by recursively deleting all its files and subdirectories securely, and then deleting the directory itself.
async fn secure_delete_directory(path: String, passes: Option<u64>) -> Result<(), String> {
    let path_obj = Path::new(&path);

    // Recursively read all files and subdirectories
    let entries = match fs::read_dir(path_obj) {
        Ok(entries) => entries,
        Err(err) => return Err(format!("Failed to read directory: {}", err)),
    };

    let mut tasks: Vec<Pin<Box<dyn Future<Output = Result<(), String>>>>> = vec![];

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => return Err(format!("Failed to read directory entry: {}", err)),
        };
        let entry_path = entry.path();


        // If it's a file, securely delete it
        if entry_path.is_file() {
            let task = secure_delete_file(entry_path.to_str().unwrap().to_string(), passes);
            tasks.push(Box::pin(task));
        }
        // If it's a subdirectory, securely delete it
        else if entry_path.is_dir() {
            let task = secure_delete_directory(entry_path.to_str().unwrap().to_string(), passes);
            tasks.push(Box::pin(task));
        }
    }

    // Await all tasks (deleting files and subdirectories)
    for task in tasks {
        if let Err(err) = task.await {
            return Err(format!("Error while deleting: {}", err));
        }
    }

    // remove the directory itself
    if let Err(err) = fs::remove_dir(&path) {
        return Err(format!("Failed to delete directory {}: {}", path, err));
    }

    Ok(())
}

#[cfg(test)]
mod tests_file_system_operation_commands {
    use super::*;
    use tempfile::tempdir;

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
        let result = move_to_trash(test_path.to_str().unwrap()).await;

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
    async fn create_directory_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory path in the temporary directory
        let test_path = temp_dir.path().join("create_directory_test");

        // Call the function to create the directory
        let result = create_directory(temp_dir.path().to_str().unwrap(), "create_directory_test").await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to create directory: {:?}", result);

        // Verify that the directory exists at the specified path
        assert!(test_path.exists(), "Directory should exist after creation");
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
    
    #[tokio::test]
    async fn rename_file_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("rename_file_test.txt");

        // Create the test file
        fs::File::create(&test_path).unwrap();

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before renaming");

        // Rename the file
        let new_name = "renamed_file.txt";
        let new_path = temp_dir.path().join(new_name);
        let result = rename(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to rename file: {:?}", result);

        // Verify that the file exists at the new path
        assert!(new_path.exists(), "File should exist at the new path");
    }
    
    #[tokio::test]
    async fn rename_directory_test(){
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("rename_directory_test");

        // Create the test directory
        fs::create_dir(&test_path).unwrap();

        // Ensure the directory exists
        assert!(test_path.exists(), "Test directory should exist before renaming");

        // Rename the directory
        let new_name = "renamed_directory";
        let new_path = temp_dir.path().join(new_name);
        let result = rename(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to rename directory: {:?}", result);

        // Verify that the directory exists at the new path
        assert!(new_path.exists(), "Directory should exist at the new path");
    }
    
    #[tokio::test]
    async fn copy_file_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("copy_file_test.txt");

        // Create the test file
        fs::File::create(&test_path).unwrap();

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before copying");

        // Copy the file
        let new_name = "copied_file.txt";
        let new_path = temp_dir.path().join(new_name);
        let result = copy_file_or_dir(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to copy file: {:?}", result);

        // Verify that the copied file exists at the new path
        assert!(new_path.exists(), "Copied file should exist at the new path");
        
        // Verify the old file still exists
        assert!(test_path.exists(), "Original file should still exist");
    }
    
    #[tokio::test]
    async fn copy_directory_test() {
        use std::io::Write;
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory in the temporary directory
        let test_path = temp_dir.path().join("copy_directory_test");
        fs::create_dir(&test_path).unwrap();

        // Create a file in the test directory
        let file_in_dir_path = test_path.join("file_in_dir.txt");
        let mut file_in_dir = fs::File::create(&file_in_dir_path).expect("Failed to create file in directory");
        writeln!(file_in_dir, "Content of file in directory").expect("Failed to write to file");

        // Create a subdirectory
        let subdir_path = test_path.join("subdir");
        fs::create_dir(&subdir_path).unwrap();

        // Create a file in the subdirectory
        let file_in_subdir_path = subdir_path.join("file_in_subdir.txt");
        let mut file_in_subdir = fs::File::create(&file_in_subdir_path).expect("Failed to create file in subdirectory");
        writeln!(file_in_subdir, "Content of file in subdirectory").expect("Failed to write to file");

        // Ensure the directory structure exists
        assert!(test_path.exists(), "Test directory should exist before copying");
        assert!(file_in_dir_path.exists(), "File in directory should exist before copying");
        assert!(subdir_path.exists(), "Subdirectory should exist before copying");
        assert!(file_in_subdir_path.exists(), "File in subdirectory should exist before copying");

        // Copy the directory
        let copied_dir_name = "copied_directory";
        let copied_dir_path = temp_dir.path().join(copied_dir_name);
        let result = copy_file_or_dir(test_path.to_str().unwrap(), copied_dir_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to copy directory: {:?}", result);

        // Verify that the copied directory exists
        assert!(copied_dir_path.exists(), "Copied directory should exist");

        // Verify that the file in the copied directory exists
        let copied_file_in_dir_path = copied_dir_path.join("file_in_dir.txt");
        assert!(copied_file_in_dir_path.exists(), "Copied file in directory should exist");

        // Verify that the subdirectory in the copied directory exists
        let copied_subdir_path = copied_dir_path.join("subdir");
        assert!(copied_subdir_path.exists(), "Copied subdirectory should exist");

        // Verify that the file in the copied subdirectory exists
        let copied_file_in_subdir_path = copied_subdir_path.join("file_in_subdir.txt");
        assert!(copied_file_in_subdir_path.exists(), "Copied file in subdirectory should exist");
        
        // Verify the original directory structure still exists
        assert!(test_path.exists(), "Original directory should still exist");
        assert!(file_in_dir_path.exists(), "Original file in directory should still exist");
        assert!(subdir_path.exists(), "Original subdirectory should still exist");
        assert!(file_in_subdir_path.exists(), "Original file in subdirectory should still exist");
    }

    // Test for secure deletion of a single file with 1 pass (just overwrite with zeros and delete)
    #[tokio::test]
    async fn secure_delete_file_one_pass_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("secure_delete_file_one_pass.txt");

        fs::File::create(&test_file_path).unwrap();
        assert!(test_file_path.exists(), "Test file should exist before deletion");

        let result = secure_delete_file(test_file_path.to_str().unwrap().to_string(), Some(1)).await;
        assert!(result.is_ok(), "Failed to delete file: {:?}", result);
        assert!(!test_file_path.exists(), "File should no longer exist after deletion");
    }

    // Test for secure deletion of a single file with 3 passes
    #[tokio::test]
    async fn secure_delete_file_three_passes_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("secure_delete_file_three_passes.txt");

        fs::File::create(&test_file_path).unwrap();
        assert!(test_file_path.exists(), "Test file should exist before deletion");

        let result = secure_delete_file(test_file_path.to_str().unwrap().to_string(), Some(3)).await;
        assert!(result.is_ok(), "Failed to delete file: {:?}", result);
        assert!(!test_file_path.exists(), "File should no longer exist after deletion");
    }

    // Test for secure deletion of a directory with files inside
    #[tokio::test]
    async fn secure_delete_directory_with_files_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory
        let test_dir = temp_dir.path().join("test_directory");
        fs::create_dir(&test_dir).unwrap();

        let file1 = test_dir.join("file1.txt");
        let file2 = test_dir.join("file2.txt");
        fs::File::create(&file1).unwrap();
        fs::File::create(&file2).unwrap();
        assert!(file1.exists(), "file1.txt should exist before deletion");
        assert!(file2.exists(), "file2.txt should exist before deletion");

        // Securely delete the directory with files inside
        let result = secure_delete_directory(test_dir.to_str().unwrap().to_string(), Some(3)).await;
        assert!(result.is_ok(), "Failed to delete directory: {:?}", result);

        // Verify that the directory and its files no longer exist
        assert!(!test_dir.exists(), "Directory should no longer exist after deletion");
        assert!(!file1.exists(), "file1.txt should no longer exist after deletion");
        assert!(!file2.exists(), "file2.txt should no longer exist after deletion");
    }

    // Test for secure deletion of a nested directory structure
    #[tokio::test]
    async fn secure_delete_nested_directory_structure_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a nested directory structure with files
        let nested_dir = temp_dir.path().join("nested_dir");
        fs::create_dir(&nested_dir).unwrap();
        let nested_file1 = nested_dir.join("nested_file1.txt");
        let nested_file2 = nested_dir.join("nested_file2.txt");
        fs::File::create(&nested_file1).unwrap();
        fs::File::create(&nested_file2).unwrap();
        assert!(nested_file1.exists(), "nested_file1.txt should exist before deletion");
        assert!(nested_file2.exists(), "nested_file2.txt should exist before deletion");

        // Create a subdirectory inside the nested directory and file
        let sub_dir = nested_dir.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        let sub_file = sub_dir.join("sub_file.txt");
        fs::File::create(&sub_file).unwrap();
        assert!(sub_file.exists(), "sub_file.txt should exist before deletion");

        // Securely delete the nested directory with files and subdirectory
        let result = secure_delete_directory(nested_dir.to_str().unwrap().to_string(), Some(3)).await;
        assert!(result.is_ok(), "Failed to delete nested directory structure: {:?}", result);

        // Verify that the directory and its contents no longer exist
        assert!(!nested_dir.exists(), "Nested directory should no longer exist after deletion");
        assert!(!nested_file1.exists(), "nested_file1.txt should no longer exist after deletion");
        assert!(!nested_file2.exists(), "nested_file2.txt should no longer exist after deletion");
        assert!(!sub_dir.exists(), "Subdirectory should no longer exist after deletion");
        assert!(!sub_file.exists(), "sub_file.txt should no longer exist after deletion");
    }

    // Test for secure deletion of a directory with multiple levels of nesting
    #[tokio::test]
    async fn secure_delete_multiple_nested_directories_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create multiple nested directories
        let level1_dir = temp_dir.path().join("level1");
        let level2_dir = level1_dir.join("level2");
        let level3_dir = level2_dir.join("level3");
        fs::create_dir_all(&level3_dir).unwrap();

        // Create files in the nested directories
        let file_in_level3 = level3_dir.join("file_in_level3.txt");
        fs::File::create(&file_in_level3).unwrap();
        assert!(file_in_level3.exists(), "file_in_level3.txt should exist before deletion");

        // Securely delete the directory with multiple nested levels (3 passes)
        let result = secure_delete_directory(temp_dir.path().to_str().unwrap().to_string(), Some(3)).await;
        assert!(result.is_ok(), "Failed to delete multiple nested directories: {:?}", result);

        // Verify that the directories and files no longer exist
        assert!(!level1_dir.exists(), "level1 directory should no longer exist after deletion");
        assert!(!level2_dir.exists(), "level2 directory should no longer exist after deletion");
        assert!(!level3_dir.exists(), "level3 directory should no longer exist after deletion");
        assert!(!file_in_level3.exists(), "file_in_level3.txt should no longer exist after deletion");
    }

    // Test for secure deletion of a directory that is system relevant (same name)
    #[tokio::test]
    async fn secure_delete_sys_relevant_path_test() {
        #[cfg(target_os = "windows")]
        let test_path = "C:\\__PROTECTED_TEST_PATH__\\file.txt";

        #[cfg(target_os = "linux")]
        let test_path = "/__protected_test_path__/file.txt";

        let test_dir = Path::new(test_path).parent().unwrap();

        fs::create_dir_all(test_dir).unwrap_or_else(|_| ());
        fs::write(test_path, b"test").unwrap_or_else(|_| ());

        let result = safe_delete_file_or_dir(test_path, Some(1)).await;

        // Clean up after the test (since the protected check should prevent deletion)
        if result.is_ok() {
            let _ = fs::remove_file(test_path);
            let _ = fs::remove_dir_all(test_dir);
        }

        // Check that deletion was denied due to protection logic
        match result {
            Err(err) => {
                // Ensure the error message contains the expected protection warning
                assert!(err.contains("Refusing to delete system-protected path"), "Expected protection error, got: {}", err);
            }
            Ok(_) => {
                // This case should never happen because the path is protected
                panic!("Expected error for protected path, but deletion succeeded");
            }
        }
    }
}
