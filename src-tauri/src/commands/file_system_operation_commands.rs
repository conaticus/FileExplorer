use crate::error_handling::{Error, ErrorCode};
use crate::models::{
    count_subdirectories, count_subfiles, format_system_time, get_access_permission_number,
    get_access_permission_string, Entries,
};
use crate::{log_error, models};
use std::fs;
use std::fs::read_dir;
use std::io::Write;
use std::path::Path;
use zip::write::FileOptions;
use zip::ZipWriter;

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
#[allow(dead_code)] //remove once the command is used again
#[tauri::command]
pub async fn open_file(path: &str) -> Result<String, String> {
    let path_obj = Path::new(path);

    // Check if path exists
    if !path_obj.exists() {
        log_error!(format!("File does not exist: {}", path).as_str());
        return Err(Error::new(
            ErrorCode::ResourceNotFound,
            format!("File does not exist: {}", path),
        )
        .to_json());
    }

    // Check if path is a file
    if !path_obj.is_file() {
        log_error!(format!("Path is not a file: {}", path).as_str());
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("Path is not a file: {}", path),
        )
        .to_json());
    }

    // Read the file
    //fs::read_to_string(path).map_err(|err| format!("Failed to read file: {}", err))
    fs::read_to_string(path).map_err(|err| {
        log_error!(format!("Failed to open file: {}", err).as_str());
        Error::new(
            ErrorCode::InternalError,
            format!("Failed to read file: {}", err),
        )
        .to_json()
    })
}

#[tauri::command]
pub async fn open_in_default_app(path: &str) -> Result<(), String> {
    let path_obj = Path::new(path);

    // Check if path exists
    if !path_obj.exists() {
        log_error!(format!("File does not exist: {}", path).as_str());
        return Err(format!("File does not exist: {}", path));
    }

    // Open the file in the default application
    open::that(path).map_err(|err| format!("Failed to open file: {}", err))
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
        log_error!(format!("Directory does not exist: {}", path).as_str());
        return Err(Error::new(
            ErrorCode::ResourceNotFound,
            format!("Directory does not exist: {}", path),
        )
        .to_json());
    }

    // Check if path is a directory
    if !path_obj.is_dir() {
        log_error!(format!("Path is not a directory: {}", path).as_str());
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("Path is not a directory: {}", path),
        )
        .to_json());
    }

    let mut directories = Vec::new();
    let mut files = Vec::new();

    for entry in read_dir(path_obj).map_err(|err| {
        log_error!(format!("Failed to read directory: {}", err).as_str());
        Error::new(
            ErrorCode::InternalError,
            format!("Failed to read directory: {}", err),
        )
        .to_json()
    })? {
        let entry = entry.map_err(|err| {
            log_error!(format!("Failed to read entry: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to read entry: {}", err),
            )
            .to_json()
        })?;

        let file_type = entry.file_type().map_err(|err| {
            log_error!(format!("Failed to get file type: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to get file type: {}", err),
            )
            .to_json()
        })?;

        let path_of_entry = entry.path();
        let metadata = entry.metadata().map_err(|err| {
            log_error!(format!("Failed to get metadata: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to get metadata: {}", err),
            )
            .to_json()
        })?;

        if file_type.is_dir() {
            directories.push(models::Directory {
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path_of_entry.to_str().unwrap().to_string(),
                is_symlink: path_of_entry.is_symlink(),
                access_rights_as_string: get_access_permission_string(metadata.permissions(), true),
                access_rights_as_number: get_access_permission_number(metadata.permissions(), true),
                size_in_bytes: 0,
                sub_file_count: count_subfiles(path_of_entry.to_str().unwrap()),
                sub_dir_count: count_subdirectories(path_of_entry.to_str().unwrap()),
                created: metadata
                    .created()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
                last_modified: metadata
                    .modified()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
                accessed: metadata
                    .accessed()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
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
                created: metadata
                    .created()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
                last_modified: metadata
                    .modified()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
                accessed: metadata
                    .accessed()
                    .map_or("1970-01-01 00:00:00".to_string(), |time| {
                        format_system_time(time)
                    }),
            });
        }
    }

    let entries = Entries { directories, files };

    // Convert the Entries struct to a JSON string
    let json = serde_json::to_string(&entries).map_err(|err| {
        log_error!(format!("Failed to serialize entries: {}", err).as_str());
        Error::new(
            ErrorCode::InternalError,
            format!("Failed to serialize entries: {}", err),
        )
        .to_json()
    })?;
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
        log_error!(format!("Directory does not exist: {}", folder_path_abs).as_str());
        // Check if the folder path exists
        return Err(Error::new(
            ErrorCode::ResourceNotFound,
            format!("Directory does not exist: {}", folder_path_abs),
        )
        .to_json());
    }
    if !path.is_dir() {
        log_error!(format!("Path is no directory: {}", folder_path_abs).as_str());
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("Path is no directory: {}", folder_path_abs),
        )
        .to_json());
    }

    // Concatenate the folder path and filename
    let file_path = path.join(file_name);

    // Create the file
    match fs::File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            log_error!(format!(
                "File could not be created: {} error: {}",
                folder_path_abs, err
            )
            .as_str());
            Err(Error::new(
                ErrorCode::InternalError,
                format!(
                    "File could not be created: {} error: {}",
                    folder_path_abs, err
                ),
            )
            .to_json())
        }
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
        log_error!(format!("Parent directory does not exist: {}", folder_path_abs).as_str());
        return Err(Error::new(
            ErrorCode::ResourceNotFound,
            format!("Parent directory does not exist: {}", folder_path_abs),
        )
        .to_json());
    }

    if !parent_path.is_dir() {
        log_error!(format!("Path is no directory: {}", folder_path_abs).as_str());
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("Path is no directory: {}", folder_path_abs),
        )
        .to_json());
    }

    // Concatenate the parent path and new directory name
    let dir_path = parent_path.join(folder_name);

    // Create the directory
    match fs::create_dir(&dir_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            log_error!(format!(
                "Failed to create directory: {} err: {}",
                folder_path_abs, err
            )
            .as_str());
            Err(Error::new(
                ErrorCode::InternalError,
                format!(
                    "Failed to create directory: {} err: {}",
                    folder_path_abs, err
                ),
            )
            .to_json())
        }
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
        log_error!(format!("File does not exist: {}", old_path).as_str());
        return Err(Error::new(
            ErrorCode::ResourceNotFound,
            format!("File does not exist: {}", old_path),
        )
        .to_json());
    }

    // Check if the new path is valid
    if new_path_obj.exists() {
        log_error!(format!("New path already exists: {}", new_path).as_str());
        return Err(Error::new(
            ErrorCode::ResourceAlreadyExists,
            format!("New path already exists: {}", new_path),
        )
        .to_json());
    }

    // Rename the file or directory
    match fs::rename(old_path, new_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            log_error!(format!("Failed to rename: {}", err).as_str());
            Err(Error::new(
                ErrorCode::InternalError,
                format!("Failed to rename: {}", err),
            )
            .to_json())
        }
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
pub async fn move_to_trash(path: &str) -> Result<(), String> {
    match trash::delete(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            log_error!(format!("Failed to move file or directory to trash: {}", err).as_str());
            Err(Error::new(
                ErrorCode::InternalError,
                format!("Failed to move file or directory to trash: {}", err),
            )
            .to_json())
        }
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
        log_error!(format!("Source path does not exist: {}", source_path).as_str());
        return Err(Error::new(
            ErrorCode::InvalidInput,
            format!("Source path does not exist: {}", source_path),
        )
        .to_json());
    }

    // Check if the destination path is valid
    if Path::new(destination_path).exists() {
        log_error!(format!("Destination path already exists: {}", destination_path).as_str());
        return Err(Error::new(
            ErrorCode::ResourceAlreadyExists,
            format!("Destination path already exists: {}", destination_path),
        )
        .to_json());
    }

    if Path::new(source_path).is_dir() {
        // If the source is a directory, recursively copy it
        let mut total_size = 0;

        // Create the destination directory
        fs::create_dir_all(destination_path).map_err(|err| {
            log_error!(format!("Failed to create destination directory: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to create destination directory: {}", err),
            )
            .to_json()
        })?;

        // Read all entries in the source directory
        for entry in fs::read_dir(source_path).map_err(|err| {
            log_error!(format!("Failed to read source directory: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to read source directory: {}", err),
            )
            .to_json()
        })? {
            let entry = entry.map_err(|err| {
                log_error!(format!("Failed to read directory entry: {}", err).as_str());
                Error::new(
                    ErrorCode::InternalError,
                    format!("Failed to read directory entry: {}", err),
                )
                .to_json()
            })?;

            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = Path::new(destination_path).join(file_name);

            if entry_path.is_file() {
                // Copy file
                let size = fs::copy(&entry_path, &dest_path).map_err(|err| {
                    log_error!(format!("Failed to copy file: {}", err).as_str());
                    Error::new(
                        ErrorCode::InternalError,
                        format!("Failed to copy file '{}': {}", entry_path.display(), err),
                    )
                    .to_json()
                })?;
                total_size += size;
            } else if entry_path.is_dir() {
                // Recursively copy subdirectory
                let sub_size = Box::pin(copy_file_or_dir(
                    entry_path.to_str().unwrap(),
                    dest_path.to_str().unwrap(),
                ))
                .await?;
                total_size += sub_size;
            }
        }

        Ok(total_size)
    } else {
        // Copy a single file
        let size = fs::copy(source_path, destination_path).map_err(|err| {
            log_error!(format!("Failed to copy file: {}", err).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to copy file: {}", err),
            )
            .to_json()
        })?;
        Ok(size)
    }
}
/// Zips files and directories to a destination zip file.
/// If only one source path is provided and no destination is specified, creates a zip file with the same name.
/// For multiple source paths, the destination path must be specified.
///
/// # Arguments
/// * `source_paths` - Vector of paths to files/directories to be zipped
/// * `destination_path` - Optional destination path for the zip file
///
/// # Returns
/// * `Ok(())` - If the zip file was successfully created
/// * `Err(String)` - If there was an error during the zipping process
///
/// # Example
/// ```rust
/// // Single file/directory with auto destination
/// let result = zip(vec!["/path/to/file.txt"], None).await;
///
/// // Multiple files to specific destination
/// let result = zip(
///     vec!["/path/to/file1.txt", "/path/to/dir1"],
///     Some("/path/to/archive.zip")
/// ).await;
/// ```
#[tauri::command]
pub async fn zip(
    source_paths: Vec<String>,
    destination_path: Option<String>,
) -> Result<(), String> {
    if source_paths.is_empty() {
        log_error!("No source paths provided");
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "No source paths provided".to_string(),
        )
        .to_json());
    }

    // If single source and no destination, use source name with .zip
    let zip_path = if source_paths.len() == 1 && destination_path.is_none() {
        Path::new(&source_paths[0]).with_extension("zip")
    } else if let Some(dest) = destination_path {
        Path::new(&dest).to_path_buf()
    } else {
        log_error!("Destination path required for multiple sources");
        return Err(Error::new(
            ErrorCode::InvalidInput,
            "Destination path required for multiple sources".to_string(),
        )
        .to_json());
    };

    // Create zip file
    let zip_file = fs::File::create(&zip_path).map_err(|e| {
        log_error!(format!("Failed to create zip file: {}", e).as_str());
        Error::new(
            ErrorCode::InternalError,
            format!("Failed to create zip file: {}", e),
        )
        .to_json()
    })?;

    let mut zip = ZipWriter::new(zip_file);
    let options: FileOptions<()> = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Process each source path
    for source_path in source_paths {
        let source = Path::new(&source_path);
        if !source.exists() {
            log_error!(format!("Source path does not exist: {}", source_path).as_str());
            return Err(Error::new(
                ErrorCode::ResourceNotFound,
                format!("Source path does not exist: {}", source_path),
            )
            .to_json());
        }

        let base_name = source
            .file_name()
            .ok_or_else(|| "Invalid source name".to_string())?
            .to_str()
            .ok_or_else(|| "Invalid characters in source name".to_string())?;

        if source.is_file() {
            zip.start_file(base_name, options).map_err(|e| {
                let err_msg = format!("Error adding file to zip: {}", e);
                log_error!(&err_msg);
                err_msg
            })?;
            let content = fs::read(source).map_err(|e| {
                let err_msg = format!("Error reading file: {}", e);
                log_error!(&err_msg);
                err_msg
            })?;
            zip.write_all(&content).map_err(|e| {
                let err_msg = format!("Error writing to zip: {}", e);
                log_error!(&err_msg);
                err_msg
            })?;
        } else if source.is_dir() {
            for entry in walkdir::WalkDir::new(source) {
                let entry = entry.map_err(|e| {
                    let err_msg = format!("Error reading directory: {}", e);
                    log_error!(&err_msg);
                    err_msg
                })?;
                let path = entry.path();

                if path.is_file() {
                    let relative = path.strip_prefix(source).map_err(|e| {
                        log_error!(format!("Failed to strip prefix: {}", e).as_str());
                        Error::new(
                            ErrorCode::InternalError,
                            format!("Error creating relative path: {}", e),
                        )
                        .to_json()
                    })?;
                    let name = format!(
                        "{}/{}",
                        base_name,
                        relative
                            .to_str()
                            .ok_or_else(|| "Invalid characters in path".to_string())?
                            .replace('\\', "/")
                    );

                    zip.start_file(&name, options).map_err(|e| {
                        log_error!(format!("Error adding file to zip: {}", e).as_str());
                        Error::new(
                            ErrorCode::InternalError,
                            format!("Error adding file to zip: {}", e),
                        )
                        .to_json()
                    })?;
                    let content = fs::read(path).map_err(|e| {
                        log_error!(format!("Error reading file: {}", e).as_str());
                        Error::new(
                            ErrorCode::InternalError,
                            format!("Error reading file: {}", e),
                        )
                        .to_json()
                    })?;
                    zip.write_all(&content).map_err(|e| {
                        log_error!(format!("Error writing to zip: {}", e).as_str());
                        Error::new(
                            ErrorCode::InternalError,
                            format!("Error writing to zip: {}", e),
                        )
                        .to_json()
                    })?;
                }
            }
        }
    }

    zip.finish().map_err(|e| {
        log_error!(format!("Error finalizing zip file: {}", e).as_str());
        Error::new(
            ErrorCode::InternalError,
            format!("Error finalizing zip file: {}", e),
        )
        .to_json()
    })?;
    Ok(())
}

/// Extracts zip files to specified destinations.
/// If extracting a single zip file without a specified destination,
/// extracts to a directory with the same name as the zip file.
///
/// # Arguments
/// * `zip_paths` - Vector of paths to zip files
/// * `destination_path` - Optional destination directory for extraction
///
/// # Returns
/// * `Ok(())` - If all zip files were successfully extracted
/// * `Err(String)` - If there was an error during extraction
///
/// # Example
/// ```rust
/// // Single zip with auto destination
/// let result = unzip(vec!["/path/to/archive.zip"], None).await;
///
/// // Multiple zips to specific destination
/// let result = unzip(
///     vec!["/path/to/zip1.zip", "/path/to/zip2.zip"],
///     Some("/path/to/extracted")
/// ).await;
/// ```
#[tauri::command]
pub async fn unzip(zip_paths: Vec<String>, destination_path: Option<String>) -> Result<(), String> {
    if zip_paths.is_empty() {
        log_error!("No zip files provided");
        return Err(
            Error::new(ErrorCode::InvalidInput, "No zip files provided".to_string()).to_json(),
        );
    }

    for zip_path in zip_paths.clone() {
        let zip_path = Path::new(&zip_path);
        if !zip_path.exists() {
            log_error!(format!("Zip file does not exist: {}", zip_path.display()).as_str());
            return Err(Error::new(
                ErrorCode::ResourceNotFound,
                format!("Zip file does not exist: {}", zip_path.display()),
            )
            .to_json());
        }

        // Determine extraction path for this zip
        let extract_path = if zip_paths.len() == 1 && destination_path.is_none() {
            // For single zip without destination, use zip name without extension
            zip_path.with_extension("")
        } else if let Some(dest) = &destination_path {
            // For multiple zips or specified destination, create subdirectory for each zip
            let zip_name = zip_path
                .file_stem()
                .ok_or_else(|| "Invalid zip filename".to_string())?;
            Path::new(dest).join(zip_name)
        } else {
            log_error!("Destination path required for multiple zip files");
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "Destination path required for multiple zip files".to_string(),
            )
            .to_json());
        };

        // Create extraction directory
        fs::create_dir_all(&extract_path).map_err(|e| {
            log_error!(format!("Failed to create extraction directory: {}", e).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to create extraction directory: {}", e),
            )
            .to_json()
        })?;

        // Open and extract zip file
        let file = fs::File::open(zip_path).map_err(|e| {
            log_error!(format!("Failed to open zip file: {}", e).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to open zip file: {}", e),
            )
            .to_json()
        })?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            log_error!(format!("Failed to read zip archive: {}", e).as_str());
            Error::new(
                ErrorCode::InternalError,
                format!("Failed to read zip archive: {}", e),
            )
            .to_json()
        })?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                log_error!(format!("Failed to read zip entry: {}", e).as_str());
                Error::new(
                    ErrorCode::InternalError,
                    format!("Failed to read zip entry: {}", e),
                )
                .to_json()
            })?;
            let outpath = extract_path.join(file.mangled_name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath).map_err(|e| {
                    log_error!(format!("Failed to create directory: {}", e).as_str());
                    Error::new(
                        ErrorCode::InternalError,
                        format!("Failed to create directory '{}': {}", outpath.display(), e),
                    )
                    .to_json()
                })?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).map_err(|e| {
                            log_error!(format!("Failed to create parent directory: {}", e).as_str());
                            Error::new(
                                ErrorCode::InternalError,
                                format!(
                                    "Failed to create parent directory '{}': {}",
                                    parent.display(),
                                    e
                                ),
                            )
                            .to_json()
                        })?;
                    }
                }
                let mut outfile = fs::File::create(&outpath).map_err(|e| {
                    log_error!(format!("Failed to create file: {}", e).as_str());
                    Error::new(
                        ErrorCode::InternalError,
                        format!("Failed to create file {}': {}", outpath.display(), e),
                    )
                    .to_json()
                })?;
                std::io::copy(&mut file, &mut outfile).map_err(|e| {
                    log_error!(format!("Failed to write file: {}", e).as_str());
                    Error::new(
                        ErrorCode::InternalError,
                        format!("Failed to write file '{}': {}", outpath.display(), e),
                    )
                    .to_json()
                })?;
            }
        }
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
    #[cfg(feature = "open-file-in-app")]
    async fn open_in_default_app_test() {
        use std::env;
        let current_dir = env::current_dir().expect("Failed to get current directory");

        let file_extensions = vec!["txt", "pdf", "mp4", "jpg", "png", "html"];

        for file_extension in file_extensions {
            let test_path = current_dir
                .join("assets")
                .join(format!("dummy.{}", file_extension));

            // Ensure the file exists
            assert!(test_path.exists(), "Test file should exist before opening");

            // Open the file in the default application
            let result = open_in_default_app(test_path.to_str().unwrap()).await;

            // Verify that the operation was successful
            assert!(
                result.is_ok(),
                "Failed to open file in default app: {:?}",
                result
            );
        }
    }

    #[tokio::test]
    async fn failed_to_open_file_because_file_not_exists_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("open_file_test.txt");

        // Open the file and read its contents
        let result = open_file(test_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );
        assert!(
            result.clone().unwrap_err().contains("File does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_open_file_because_path_is_not_a_file_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("open_file_test.txt");

        // Create the test file
        fs::File::create(&test_path).unwrap();

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before reading");

        // Open the file and read its contents
        let result = open_file(temp_dir.path().to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result.clone().unwrap_err().contains("Path is not a file"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
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
    async fn failed_move_to_trash_because_invalid_resource_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut invalid_test_path = temp_dir.path().to_path_buf();
        invalid_test_path.push("move_to_trash_test.txt");

        // Ensure the file does not exist
        assert!(
            !invalid_test_path.exists(),
            "Test file should not exist before deletion"
        );

        eprintln!("Test file exists: {:?}", invalid_test_path);

        // Move the file to the trash
        let result = move_to_trash(invalid_test_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Failed to move file or directory to trash"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("500"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InternalError"),
            "Error message does not match expected value"
        );
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
    async fn failed_to_create_file_because_directory_does_not_exist_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file path in the temporary directory
        let test_path = temp_dir.path().join("missing_dir");

        // Call the function to create the file
        let result = create_file(test_path.to_str().unwrap(), "create_file_test.txt").await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Directory does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_create_file_because_path_is_no_directory_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let test_path = temp_dir.path().join("not_a_folder.txt");
        fs::File::create(&test_path).unwrap();

        // Call the function to create the file
        let result = create_file(test_path.to_str().unwrap(), "create_file_test.txt").await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result.clone().unwrap_err().contains("Path is no directory"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn create_directory_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory path in the temporary directory
        let test_path = temp_dir.path().join("create_directory_test");

        // Call the function to create the directory
        let result =
            create_directory(temp_dir.path().to_str().unwrap(), "create_directory_test").await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to create directory: {:?}", result);

        // Verify that the directory exists at the specified path
        assert!(test_path.exists(), "Directory should exist after creation");
    }

    #[tokio::test]
    async fn failed_to_create_directory_because_parent_directory_does_not_exist_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory path in the temporary directory
        let test_path = temp_dir.path().join("missing_dir");

        // Call the function to create the directory
        let result = create_directory(
            test_path.join("not_a_parent_directory").to_str().unwrap(),
            "create_directory_test",
        )
        .await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Parent directory does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
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
    async fn failed_to_open_directory_because_directory_does_not_exist_test() {
        use super::*;
        use tempfile::tempdir;
        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("open_directory_test.txt");

        // Open the file and read its contents
        let result = open_directory(test_path.to_str().unwrap().to_string()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Directory does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_open_directory_because_path_is_not_a_directory_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("open_directory_test.txt");

        // Create the test file
        fs::File::create(&test_path).unwrap();

        // Ensure the file exists
        assert!(test_path.exists(), "Test file should exist before reading");

        // Open the file and read its contents
        let result = open_directory(test_path.to_str().unwrap().to_string()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Path is not a directory"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
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
    async fn failed_to_rename_because_file_does_not_exist_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("rename_file_test.txt");

        // Rename the file
        let new_name = "renamed_file.txt";
        let new_path = temp_dir.path().join(new_name);
        let result = rename(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result.clone().unwrap_err().contains("File does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_rename_because_new_path_already_exists_test() {
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
        fs::File::create(&new_path).unwrap(); // Create the new path to simulate conflict

        let result = rename(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("New path already exists"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("409"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceAlreadyExists"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn rename_directory_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test directory in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("rename_directory_test");

        // Create the test directory
        fs::create_dir(&test_path).unwrap();

        // Ensure the directory exists
        assert!(
            test_path.exists(),
            "Test directory should exist before renaming"
        );

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
        let result =
            copy_file_or_dir(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to copy file: {:?}", result);

        // Verify that the copied file exists at the new path
        assert!(
            new_path.exists(),
            "Copied file should exist at the new path"
        );

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
        let mut file_in_dir =
            fs::File::create(&file_in_dir_path).expect("Failed to create file in directory");
        writeln!(file_in_dir, "Content of file in directory").expect("Failed to write to file");

        // Create a subdirectory
        let subdir_path = test_path.join("subdir");
        fs::create_dir(&subdir_path).unwrap();

        // Create a file in the subdirectory
        let file_in_subdir_path = subdir_path.join("file_in_subdir.txt");
        let mut file_in_subdir =
            fs::File::create(&file_in_subdir_path).expect("Failed to create file in subdirectory");
        writeln!(file_in_subdir, "Content of file in subdirectory")
            .expect("Failed to write to file");

        // Ensure the directory structure exists
        assert!(
            test_path.exists(),
            "Test directory should exist before copying"
        );
        assert!(
            file_in_dir_path.exists(),
            "File in directory should exist before copying"
        );
        assert!(
            subdir_path.exists(),
            "Subdirectory should exist before copying"
        );
        assert!(
            file_in_subdir_path.exists(),
            "File in subdirectory should exist before copying"
        );

        // Copy the directory
        let copied_dir_name = "copied_directory";
        let copied_dir_path = temp_dir.path().join(copied_dir_name);
        let result = copy_file_or_dir(
            test_path.to_str().unwrap(),
            copied_dir_path.to_str().unwrap(),
        )
        .await;

        // Verify that the operation was successful
        assert!(result.is_ok(), "Failed to copy directory: {:?}", result);

        // Verify that the copied directory exists
        assert!(copied_dir_path.exists(), "Copied directory should exist");

        // Verify that the file in the copied directory exists
        let copied_file_in_dir_path = copied_dir_path.join("file_in_dir.txt");
        assert!(
            copied_file_in_dir_path.exists(),
            "Copied file in directory should exist"
        );

        // Verify that the subdirectory in the copied directory exists
        let copied_subdir_path = copied_dir_path.join("subdir");
        assert!(
            copied_subdir_path.exists(),
            "Copied subdirectory should exist"
        );

        // Verify that the file in the copied subdirectory exists
        let copied_file_in_subdir_path = copied_subdir_path.join("file_in_subdir.txt");
        assert!(
            copied_file_in_subdir_path.exists(),
            "Copied file in subdirectory should exist"
        );

        // Verify the original directory structure still exists
        assert!(test_path.exists(), "Original directory should still exist");
        assert!(
            file_in_dir_path.exists(),
            "Original file in directory should still exist"
        );
        assert!(
            subdir_path.exists(),
            "Original subdirectory should still exist"
        );
        assert!(
            file_in_subdir_path.exists(),
            "Original file in subdirectory should still exist"
        );
    }

    #[tokio::test]
    async fn failed_to_copy_file_or_dir_because_source_path_does_not_exist_test() {
        use tempfile::tempdir;

        // Create a temporary directory (automatically deleted when out of scope)
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test file in the temporary directory
        let mut test_path = temp_dir.path().to_path_buf();
        test_path.push("copy_file_test.txt");

        // Copy the file to a non-existing path
        let new_name = "copy_file_test.txt";
        let new_path = temp_dir.path().join(new_name);

        let result =
            copy_file_or_dir(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Source path does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }
    #[tokio::test]
    async fn failed_to_copy_file_or_dir_because_destination_path_already_exists_test() {
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

        // Copy the file to an existing path
        let new_name = "copy_file_test.txt";
        let new_path = temp_dir.path().join(new_name);
        fs::File::create(&new_path).unwrap(); // Create the new path to simulate conflict

        let result =
            copy_file_or_dir(test_path.to_str().unwrap(), new_path.to_str().unwrap()).await;

        // Verify that the operation was successful
        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Destination path already exists"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("409"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceAlreadyExists"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn zip_single_file_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_file.txt");

        // Create and write to test file
        fs::write(&test_file_path, "Test content").expect("Failed to write test file");
        assert!(
            test_file_path.exists(),
            "Test file should exist before zipping"
        );

        // Zip the file
        let result = zip(vec![test_file_path.to_str().unwrap().to_string()], None).await;
        assert!(result.is_ok(), "Failed to zip file: {:?}", result);

        // Check if zip file was created
        let zip_path = test_file_path.with_extension("zip");
        assert!(zip_path.exists(), "Zip file should exist after operation");

        // Verify zip contents
        let zip_file = fs::File::open(&zip_path).expect("Failed to open zip file");
        let mut archive = zip::ZipArchive::new(zip_file).expect("Failed to read zip archive");
        assert_eq!(archive.len(), 1, "Zip should contain exactly one file");

        let file = archive.by_index(0).expect("Failed to read file from zip");
        assert_eq!(file.name(), "test_file.txt", "Incorrect filename in zip");
    }

    #[tokio::test]
    async fn failed_to_zip_because_no_source_paths_provided_test() {
        let result = zip(vec![], None).await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("No source paths provided"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_zip_because_destination_path_required_for_multiple_sources_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        fs::write(&file1_path, "Content 1").expect("Failed to write file1");
        fs::write(&file2_path, "Content 2").expect("Failed to write file2");

        // Zip multiple files without specifying destination
        let result = zip(
            vec![
                file1_path.to_str().unwrap().to_string(),
                file2_path.to_str().unwrap().to_string(),
            ],
            None,
        )
        .await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );
        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Destination path required for multiple sources"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_zip_because_source_path_does_not_exist_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        let result_zip = Some(
            temp_dir
                .path()
                .join("result.zip")
                .to_str()
                .unwrap()
                .to_string(),
        );

        // Create a test file
        let test_file_path = temp_dir.path().join("test_file.txt");
        fs::write(&test_file_path, "Test content").expect("Failed to write test file");

        // Attempt to zip a non-existing file
        let non_existing_file_path = temp_dir.path().join("non_existing_file.txt");
        let result = zip(
            vec![
                test_file_path.to_str().unwrap().to_string(),
                non_existing_file_path.to_str().unwrap().to_string(),
            ],
            result_zip,
        )
        .await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Source path does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn unzip_single_file_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test zip file
        let zip_path = temp_dir.path().join("test.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());

        zip.start_file::<_, ()>("test.txt", FileOptions::default())
            .unwrap();
        zip.write_all(b"Hello, World!").unwrap();
        zip.finish().unwrap();

        // Test extraction without specifying destination
        let result = unzip(vec![zip_path.to_str().unwrap().to_string()], None).await;

        assert!(result.is_ok(), "Failed to extract zip: {:?}", result);

        // Verify extracted contents
        let extract_path = zip_path.with_extension("");
        let test_file = extract_path.join("test.txt");

        assert!(test_file.exists(), "Extracted test.txt should exist");
        assert_eq!(
            fs::read_to_string(test_file).unwrap(),
            "Hello, World!",
            "Extracted content should match"
        );
    }

    #[tokio::test]
    async fn failed_to_unzip_because_no_zip_files_provided_test() {
        let result = unzip(vec![], None).await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("No zip files provided"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_unzip_because_zip_file_does_not_exist_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test zip file
        let zip_path = temp_dir.path().join("non_existing.zip");

        // Test extraction of a non-existing zip file
        let result = unzip(vec![zip_path.to_str().unwrap().to_string()], None).await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Zip file does not exist"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("405"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("ResourceNotFound"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_unzip_because_destination_path_required_for_multiple_zip_files_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create test zip files
        let zip1_path = temp_dir.path().join("test1.zip");
        let zip2_path = temp_dir.path().join("test2.zip");

        // Create content for first zip
        let mut zip1 = zip::ZipWriter::new(fs::File::create(&zip1_path).unwrap());
        zip1.start_file::<_, ()>("file1.txt", FileOptions::default())
            .unwrap();
        zip1.write_all(b"Content 1").unwrap();
        zip1.finish().unwrap();

        // Create content for second zip
        let mut zip2 = zip::ZipWriter::new(fs::File::create(&zip2_path).unwrap());
        zip2.start_file::<_, ()>("file2.txt", FileOptions::default())
            .unwrap();
        zip2.write_all(b"Content 2").unwrap();
        zip2.finish().unwrap();

        // Test extraction of multiple zips without specifying destination
        let result = unzip(
            vec![
                zip1_path.to_str().unwrap().to_string(),
                zip2_path.to_str().unwrap().to_string(),
            ],
            None,
        )
        .await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Destination path required for multiple zip files"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("408"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InvalidInput"),
            "Error message does not match expected value"
        );
    }

    #[tokio::test]
    async fn failed_to_unzip_because_failed_to_create_extraction_directory_test() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");

        // Create a test zip file
        let zip_path = temp_dir.path().join("test.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());

        zip.start_file::<_, ()>("test.txt", FileOptions::default())
            .unwrap();
        zip.write_all(b"Hello, World!").unwrap();
        zip.finish().unwrap();

        // Attempt to unzip to an invalid destination path
        let invalid_dest = "/invalid/path/extracted";
        let result = unzip(
            vec![zip_path.to_str().unwrap().to_string()],
            Some(invalid_dest.to_string()),
        )
        .await;

        assert!(
            result.is_err(),
            "Failed test (should throw an error): {:?}",
            result
        );

        assert!(
            result
                .clone()
                .unwrap_err()
                .contains("Failed to create extraction directory"),
            "Error message does not match expected value"
        );

        assert!(
            result.clone().unwrap_err().contains("500"),
            "Error message does not match expected value"
        );

        assert!(
            result.unwrap_err().contains("InternalError"),
            "Error message does not match expected value"
        );
    }
}
