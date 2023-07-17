use std::error::Error;
use std::fs::copy;
use std::path::{Path, PathBuf};
use tauri::State;
use std::fs;

/// Copies a file from source to destination.

#[tauri::command]
pub async fn paste(source: String, destination: String) -> Result<String, String> {
    let final_destination = format!("{}/{}", destination, Path::new(&source).file_name().unwrap().to_string_lossy());

    let source_path = Path::new(&source);
    let destination_path = Path::new(&final_destination);

    let result = if source_path.is_dir() {
        match copy_dir_recursive(source_path, destination_path) {
            Ok(_) => Ok(("Directory".to_string(), String::from("Copied"))),
            Err(err) => Err(err.to_string()),
        }
    } else if source_path.is_file() {
        match fs::copy(source_path, destination_path) {
            Ok(_) => Ok(("File".to_string(), String::from("Copied"))),
            Err(err) => Err(err.to_string()),
        }
    } else {
        Err(String::from("Source is neither a file nor a directory."))
    };

    // Send the result back to JavaScript
    match result {
        Ok(value) => Ok(value.0),
        Err(error) => Err(error),
    }
}

/// Get file type, which is File or Directory (or error)

#[tauri::command]
pub async fn get_file_type(path: String) -> Result<String, String> {
    let metadata_result = std::fs::metadata(&path);
    match metadata_result {
        Ok(metadata) => {
            if metadata.is_file() {
                Ok(String::from("File"))
            } else if metadata.is_dir() {
                Ok(String::from("Directory"))
            } else {
                Err(String::from("Path is neither a file nor a directory."))
            }
        }
        Err(error) => Err(format!("Failed to get metadata: {}", error)),
    }
}


fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    if source.is_dir() {
        fs::create_dir(destination)?;

        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let destination_file = destination.join(entry.file_name());

            if entry_path.is_dir() {
                copy_dir_recursive(&entry_path, &destination_file)?;
            } else if entry_path.is_file() {
                fs::copy(&entry_path, &destination_file)?;
            } else {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Source entry is neither a file nor a directory.",
                )));
            }
        }
    }
    Ok(())
}
