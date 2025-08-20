use std::io::{Read, Write};
use ssh2::{Session, Sftp};
use std::net::TcpStream;
use std::path::Path;
use std::fs;
use crate::models::SFTPDirectory;
use crate::commands::preview_commands::PreviewPayload;
use base64::Engine;

fn connect_to_sftp_via_password(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<Sftp, String> {
    // Create the TCP connection string
    let connection_string = format!("{}:{}", host, port);
    // Connect to the SSH server
    let tcp = TcpStream::connect(connection_string).map_err(|e| e.to_string())?;
    let mut session = Session::new().map_err(|_| "Could not initialize session".to_string())?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| e.to_string())?;

    // Generates a unique SFTP destination path by appending a number if the path already exists on the remote server.
    // For example: "file.txt" -> "file (1).txt" -> "file (2).txt"
    // For directories: "folder" -> "folder (1)" -> "folder (2)"
    // Authenticate
    session.userauth_password(&username, &password).map_err(|e| e.to_string())?;
    
    // Check if authentication was successful
    if !session.authenticated() {
        return Err("Authentication failed".to_string());
    }
    
    // Open an SFTP session
    session.sftp().map_err(|e| e.to_string()).map_err(|e| e.to_string())
}

#[allow(dead_code)]
#[tauri::command]
pub fn connect_to_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
) -> Result<Sftp, String> {
    connect_to_sftp_via_password(host, port, username, password)
}

#[tauri::command]
pub fn load_dir(
    host: String,
    port: u16,
    username: String,
    password: String,
    directory: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Read the directory entries
    let entries = sftp.readdir(&directory).map_err(|e| e.to_string())?;
    
    // Convert entries to SFTPDirectory format
    let files: Vec<String> = entries.iter()
        .filter_map(|(path, stat)| {
            if stat.is_file() {
                Some(path.to_str().unwrap_or("").to_string())
            } else {
                None
            }
        })
        .collect();
    
    let directories: Vec<String> = entries.iter()
        .filter_map(|(path, stat)| {
            if stat.is_dir() {
                Some(path.to_str().unwrap_or("").to_string())
            } else {
                None
            }
        })
        .collect();
    
    let sftp_directory = SFTPDirectory {
        sftp_directory: directory,
        files,
        directories,
    };
    
    // Serialize the SFTPDirectory to JSON
    serde_json::to_string(&sftp_directory).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    file_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Open the file
    let mut file = sftp.open(&file_path).map_err(|e| e.to_string())?;
    
    // Read the file content
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
    
    Ok(contents)
}

#[tauri::command]
pub fn create_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    file_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Create the file
    sftp.create(file_path.as_ref()).map_err(|e| e.to_string())?;
    
    Ok(format!("File created at: {}", file_path))
}

#[tauri::command]
pub fn delete_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    file_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Delete the file
    sftp.unlink(file_path.as_ref()).map_err(|e| e.to_string())?;
    
    Ok(format!("File deleted at: {}", file_path))
}

#[tauri::command]
pub fn rename_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    old_path: String,
    new_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Rename the file
    sftp.rename(old_path.as_ref(), new_path.as_ref(), None).map_err(|e| e.to_string())?;
    
    Ok(format!("File renamed from {} to {}", old_path, new_path))
}

#[tauri::command]
pub fn copy_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    source_path: String,
    destination_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Copy the file
    let mut source_file = sftp.open(&source_path).map_err(|e| e.to_string())?;
    let mut destination_file = sftp.create(destination_path.as_ref()).map_err(|e| e.to_string())?;
    
    let mut buffer = Vec::new();
    source_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
    destination_file.write_all(&buffer).map_err(|e| e.to_string())?;
    
    Ok(format!("File copied from {} to {}", source_path, destination_path))
}

#[tauri::command]
pub fn move_file_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    source_path: String,
    destination_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Move the file
    sftp.rename(source_path.as_ref(), destination_path.as_ref(), None).map_err(|e| e.to_string())?;
    
    Ok(format!("File moved from {} to {}", source_path, destination_path))
}

#[tauri::command]
pub fn create_directory_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    directory_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Create the directory
    sftp.mkdir(directory_path.as_ref(), 0o755).map_err(|e| e.to_string())?;
    
    Ok(format!("Directory created at: {}", directory_path))
}

#[tauri::command]
pub fn delete_directory_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    directory_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Delete the directory
    sftp.rmdir(directory_path.as_ref()).map_err(|e| e.to_string())?;
    
    Ok(format!("Directory deleted at: {}", directory_path))
}

#[tauri::command]
pub fn rename_directory_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    old_path: String,
    new_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Rename the directory
    sftp.rename(old_path.as_ref(), new_path.as_ref(), None).map_err(|e| e.to_string())?;
    
    Ok(format!("Directory renamed from {} to {}", old_path, new_path))
}

#[tauri::command]
pub fn copy_directory_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    source_path: String,
    destination_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host.clone(), port, username.clone(), password.clone())?;
    
    // Create the destination directory
    sftp.mkdir(destination_path.as_ref(), 0o755).map_err(|e| e.to_string())?;
    
    // Read the source directory entries
    let entries = sftp.readdir(&source_path).map_err(|e| e.to_string())?;
    
    for (path, stat) in entries {
        let file_name = path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("[invalid_filename]");
        let new_path = format!("{}/{}", destination_path, file_name);
        
        if stat.is_file() {
            // Copy file
            let mut source_file = sftp.open(&path).map_err(|e| e.to_string())?;
            let mut destination_file = sftp.create(new_path.as_ref()).map_err(|e| e.to_string())?;
            
            let mut buffer = Vec::new();
            source_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            destination_file.write_all(&buffer).map_err(|e| e.to_string())?;
        } else if stat.is_dir() {
            // Recursively copy directory
            let path_str = path.to_str().unwrap_or("[invalid_path]").to_string();
            copy_directory_sftp(host.clone(), port, username.clone(), password.clone(), path_str, new_path)?;
        }
    }
    
    Ok(format!("Directory copied from {} to {}", source_path, destination_path))
}

#[tauri::command]
pub fn move_directory_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    source_path: String,
    destination_path: String,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Move the directory
    sftp.rename(source_path.as_ref(), destination_path.as_ref(), None).map_err(|e| e.to_string())?;
    
    Ok(format!("Directory moved from {} to {}", source_path, destination_path))
}

fn filename_from_path(path: &str) -> String {
    if let Some(name) = path.split('/').last() {
        if !name.is_empty() {
            return name.to_string();
        }
    }
    "file".to_string()
}

fn detect_mime_sftp(path: &str, head: &[u8]) -> Option<&'static str> {
    if let Some(kind) = infer::get(head) {
        return Some(kind.mime_type());
    }
    
    if let Some(ext) = path.split('.').last().map(|s| s.to_lowercase()) {
        return Some(match ext.as_str() {
            "md" | "rs" | "ts" | "tsx" | "js" | "jsx" | "json" | "txt" | "log" | "toml" | "yaml" | "yml" | "xml" | "ini" | "csv" => "text/plain",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            _ => "application/octet-stream",
        });
    }
    None
}

fn read_sftp_prefix(sftp: &Sftp, path: &str, max_bytes: usize) -> Result<Vec<u8>, String> {
    let mut file = sftp.open(Path::new(path)).map_err(|e| e.to_string())?;
    let mut buf = Vec::with_capacity(max_bytes.min(1024 * 1024));
    let mut temp_buf = vec![0u8; max_bytes.min(8192)];
    let mut total_read = 0;
    
    while total_read < max_bytes {
        let chunk_size = std::cmp::min(temp_buf.len(), max_bytes - total_read);
        match file.read(&mut temp_buf[..chunk_size]) {
            Ok(0) => break, // EOF
            Ok(n) => {
                buf.extend_from_slice(&temp_buf[..n]);
                total_read += n;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    Ok(buf)
}

#[tauri::command]
pub fn build_preview_sftp(
    host: String,
    port: u16,
    username: String,
    password: String,
    file_path: String,
) -> Result<PreviewPayload, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    let name = filename_from_path(&file_path);
    
    // Get file stats to check if it's a directory or file
    let stat = sftp.stat(Path::new(&file_path)).map_err(|e| e.to_string())?;
    
    // Handle directories
    if stat.is_dir() {
        // Count items (files + dirs, not recursive)
        let mut item_count = 0;
        let mut size: u64 = 0;
        let mut latest_modified: Option<u64> = None;
        
        if let Ok(entries) = sftp.readdir(Path::new(&file_path)) {
            for (_, entry_stat) in entries {
                item_count += 1;
                if let Some(entry_size) = entry_stat.size {
                    size += entry_size;
                }
                if let Some(mtime) = entry_stat.mtime {
                    let mtime_u64 = mtime as u64;
                    latest_modified = match latest_modified {
                        Some(current) if current > mtime_u64 => Some(current),
                        _ => Some(mtime_u64),
                    };
                }
            }
        }
        
        // Use folder's own modified time if no children
        let folder_modified = stat.mtime;
        let modified_time = latest_modified.or(folder_modified);
        let modified_str = modified_time.map(|t| {
            chrono::DateTime::from_timestamp(t as i64, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| "unknown".to_string())
        });
        
        return Ok(PreviewPayload::Folder {
            name,
            size,
            item_count,
            modified: modified_str,
        });
    }
    
    // Files
    let bytes = stat.size.unwrap_or(0) as usize;
    // Read a small head for detection + maybe text
    let head = read_sftp_prefix(&sftp, &file_path, 256 * 1024).map_err(|e| e.to_string())?;
    let mime = detect_mime_sftp(&file_path, &head).unwrap_or("application/octet-stream");
    
    // Branch by mime top-level type - exactly like original
    if mime.starts_with("image/") {
        // Encode entire file only if small; else just the head (fast path)
        let cap = 6 * 1024 * 1024;
        let data = if bytes <= cap {
            let mut full_file = sftp.open(Path::new(&file_path)).map_err(|e| e.to_string())?;
            let mut full_data = Vec::new();
            full_file.read_to_end(&mut full_data).map_err(|e| e.to_string())?;
            full_data
        } else {
            head.clone()
        };
        let data_uri = format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(data));
        return Ok(PreviewPayload::Image { name, data_uri, bytes });
    }
    
    if mime == "application/pdf" {
        // Encode entire file only if small; else just the head (fast path)
        let cap = 12 * 1024 * 1024; // Allow larger PDFs than images
        let data = if bytes <= cap {
            let mut full_file = sftp.open(Path::new(&file_path)).map_err(|e| e.to_string())?;
            let mut full_data = Vec::new();
            full_file.read_to_end(&mut full_data).map_err(|e| e.to_string())?;
            full_data
        } else {
            head.clone()
        };
        let data_uri = format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(data));
        return Ok(PreviewPayload::Pdf { name, data_uri, bytes });
    }

    if mime.starts_with("video/") {
        // For SFTP videos, treat as unknown since we can't stream remote files
        return Ok(PreviewPayload::Unknown { name });
    }

    if mime.starts_with("audio/") {
        // For SFTP audio, treat as unknown since we can't stream remote files
        return Ok(PreviewPayload::Unknown { name });
    }

    // Heuristic: treat smallish or text‑ish files as text
    let looks_texty = mime.starts_with("text/") || head.iter().all(|&b| b == 9 || b == 10 || b == 13 || (b >= 32 && b < 0xF5));
    if looks_texty || bytes <= 2 * 1024 * 1024 {
        let mut det = chardetng::EncodingDetector::new();
        det.feed(&head, true);
        let enc = det.guess(None, true);
        let (cow, _, _) = enc.decode(&head);
        let mut text = cow.to_string();
        let mut truncated = false;
        if text.len() > 200_000 {
            text.truncate(200_000);
            text.push_str("\n…(truncated)");
            truncated = true;
        }
        return Ok(PreviewPayload::Text { name, text, truncated });
    }

    Ok(PreviewPayload::Unknown { name })
}

#[tauri::command]
pub fn download_and_open_sftp_file(
    host: String,
    port: u16,
    username: String,
    password: String,
    file_path: String,
    open_file: Option<bool>,
) -> Result<String, String> {
    let sftp = connect_to_sftp_via_password(host, port, username, password)?;
    
    // Get the filename from the path
    let filename = filename_from_path(&file_path);
    
    // Create a temporary directory if it doesn't exist
    let temp_dir = std::env::temp_dir().join("file_explorer_sftp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp directory: {}", e))?;
    }
    
    // Create a unique temporary file path
    let temp_file_path = temp_dir.join(&filename);
    
    // Download the file from SFTP
    let mut remote_file = sftp.open(Path::new(&file_path)).map_err(|e| e.to_string())?;
    let mut local_file = fs::File::create(&temp_file_path).map_err(|e| e.to_string())?;
    
    // Copy the file content
    std::io::copy(&mut remote_file, &mut local_file).map_err(|e| e.to_string())?;
    
    // Only open the file if explicitly requested (default is true for backward compatibility)
    let should_open = open_file.unwrap_or(true);
    
    if should_open {
        // Open the file with the default application
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(&["/C", "start", "", &temp_file_path.to_string_lossy()])
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open")
                .arg(&temp_file_path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open")
                .arg(&temp_file_path)
                .spawn()
                .map_err(|e| format!("Failed to open file: {}", e))?;
        }
        
        Ok(format!("File downloaded to {} and opened", temp_file_path.to_string_lossy()))
    } else {
        // Return the temporary file path without opening
        Ok(temp_file_path.to_string_lossy().to_string())
    }
}

#[tauri::command]
pub fn cleanup_sftp_temp_files() -> Result<String, String> {
    let temp_dir = std::env::temp_dir().join("file_explorer_sftp");
    
    if !temp_dir.exists() {
        return Ok("No temporary directory to clean".to_string());
    }
    
    let mut cleaned_count = 0;
    
    match fs::read_dir(&temp_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            // Delete files older than 24 hours
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() > 24 * 60 * 60 {
                                    if fs::remove_file(entry.path()).is_ok() {
                                        cleaned_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read temp directory: {}", e)),
    }
    
    Ok(format!("Cleaned {} old temporary files", cleaned_count))
}

#[cfg(test)]
#[cfg(feature = "sftp-tests")]
mod sftp_file_system_operation_commands_tests {
    use super::*;

    // Test data
    const TEST_HOST: &str = "localhost";
    const TEST_PORT: u16 = 2222;
    const TEST_USERNAME: &str = "explorer";
    const TEST_PASSWORD: &str = "explorer";
    const TEST_WRONG_PASSWORD: &str = "wrong_password";
    const TEST_WRONG_HOST: &str = "nonexistent.host";

    // Helper function to create test file content
    #[allow(dead_code)]
    fn get_test_file_content() -> &'static str {
        "This is a test file content for SFTP operations."
    }

    #[test]
    fn test_connect_to_sftp_via_password_success() {
        let result = connect_to_sftp_via_password(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully connect to SFTP server");
    }

    #[test]
    fn test_connect_to_sftp_via_password_failure_wrong_password() {
        let result = connect_to_sftp_via_password(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_WRONG_PASSWORD.to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong password");
    }

    #[test]
    fn test_connect_to_sftp_via_password_failure_wrong_host() {
        let result = connect_to_sftp_via_password(
            TEST_WRONG_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong host");
    }

    #[test]
    fn test_connect_to_sftp_success() {
        let result = connect_to_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully connect to SFTP server");
    }

    #[test]
    fn test_connect_to_sftp_failure() {
        let result = connect_to_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_WRONG_PASSWORD.to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong credentials");
    }

    #[test]
    fn test_load_dir_success() {
        let result = load_dir(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            ".".to_string(),
        );
        
        match result {
            Ok(json) => {
                println!("SFTP Directory JSON: {}", json);
                assert!(!json.is_empty(), "JSON should not be empty");
                // Try to parse the JSON to ensure it's valid
                let parsed: Result<SFTPDirectory, _> = serde_json::from_str(&json);
                assert!(parsed.is_ok(), "Should be valid JSON");
            },
            Err(e) => {
                panic!("Should successfully load directory: {}", e);
            }
        }
    }

    #[test]
    fn test_load_dir_failure_wrong_credentials() {
        let result = load_dir(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_WRONG_PASSWORD.to_string(),
            ".".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong credentials");
    }

    #[test]
    fn test_load_dir_failure_nonexistent_directory() {
        let result = load_dir(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "/nonexistent/directory".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent directory");
    }

    #[test]
    fn test_create_file_sftp_success() {
        let test_file = "test_create_file.txt";
        
        let result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully create file");
        
        // Clean up - delete the test file
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
    }

    #[test]
    fn test_create_file_sftp_failure() {
        let result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_WRONG_PASSWORD.to_string(),
            "test_file.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong credentials");
    }

    #[test]
    fn test_delete_file_sftp_success() {
        let test_file = "test_delete_file.txt";
        
        // First create a file
        let create_result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
        assert!(create_result.is_ok(), "Should create test file first");
        
        // Then delete it
        let result = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully delete file");
    }

    #[test]
    fn test_delete_file_sftp_failure_nonexistent_file() {
        let result = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_file.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_rename_file_sftp_success() {
        let original_file = "test_rename_original.txt";
        let renamed_file = "test_rename_new.txt";
        
        // First create a file
        let create_result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            original_file.to_string(),
        );
        assert!(create_result.is_ok(), "Should create test file first");
        
        // Then rename it
        let result = rename_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            original_file.to_string(),
            renamed_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully rename file");
        
        // Clean up
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            renamed_file.to_string(),
        );
    }

    #[test]
    fn test_rename_file_sftp_failure() {
        let result = rename_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_file.txt".to_string(),
            "new_name.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_copy_file_sftp_success() {
        let source_file = "test_copy_source.txt";
        let dest_file = "test_copy_dest.txt";
        
        // First create a source file
        let create_result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_file.to_string(),
        );
        assert!(create_result.is_ok(), "Should create source file first");
        
        // Then copy it
        let result = copy_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_file.to_string(),
            dest_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully copy file");
        
        // Clean up
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_file.to_string(),
        );
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            dest_file.to_string(),
        );
    }

    #[test]
    fn test_copy_file_sftp_failure() {
        let result = copy_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_source.txt".to_string(),
            "dest.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent source file");
    }

    #[test]
    fn test_move_file_sftp_success() {
        let source_file = "test_move_source.txt";
        let dest_file = "test_move_dest.txt";
        
        // First create a source file
        let create_result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_file.to_string(),
        );
        assert!(create_result.is_ok(), "Should create source file first");
        
        // Then move it
        let result = move_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_file.to_string(),
            dest_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully move file");
        
        // Clean up
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            dest_file.to_string(),
        );
    }

    #[test]
    fn test_move_file_sftp_failure() {
        let result = move_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_file.txt".to_string(),
            "dest.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent file");
    }

    #[test]
    fn test_create_directory_sftp_success() {
        let test_dir = "test_create_directory";
        
        let result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_dir.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully create directory");
        
        // Clean up
        let _ = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_dir.to_string(),
        );
    }

    #[test]
    fn test_create_directory_sftp_failure() {
        let result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_WRONG_PASSWORD.to_string(),
            "test_dir".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with wrong credentials");
    }

    #[test]
    fn test_delete_directory_sftp_success() {
        let test_dir = "test_delete_directory";
        
        // First create a directory
        let create_result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_dir.to_string(),
        );
        assert!(create_result.is_ok(), "Should create test directory first");
        
        // Then delete it
        let result = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_dir.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully delete directory");
    }

    #[test]
    fn test_delete_directory_sftp_failure() {
        let result = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_directory".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent directory");
    }

    #[test]
    fn test_rename_directory_sftp_success() {
        let original_dir = "test_rename_dir_original";
        let renamed_dir = "test_rename_dir_new";
        
        // First create a directory
        let create_result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            original_dir.to_string(),
        );
        assert!(create_result.is_ok(), "Should create test directory first");
        
        // Then rename it
        let result = rename_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            original_dir.to_string(),
            renamed_dir.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully rename directory");
        
        // Clean up
        let _ = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            renamed_dir.to_string(),
        );
    }

    #[test]
    fn test_rename_directory_sftp_failure() {
        let result = rename_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_directory".to_string(),
            "new_name".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent directory");
    }

    #[test]
    fn test_move_directory_sftp_success() {
        let source_dir = "test_move_dir_source";
        let dest_dir = "test_move_dir_dest";
        
        // First create a source directory
        let create_result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_dir.to_string(),
        );
        assert!(create_result.is_ok(), "Should create source directory first");
        
        // Then move it
        let result = move_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_dir.to_string(),
            dest_dir.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully move directory");
        
        // Clean up
        let _ = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            dest_dir.to_string(),
        );
    }

    #[test]
    fn test_move_directory_sftp_failure() {
        let result = move_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_directory".to_string(),
            "dest_dir".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent directory");
    }

    #[test]
    fn test_copy_directory_sftp_success() {
        let source_dir = "test_copy_dir_source";
        let dest_dir = "test_copy_dir_dest";
        
        // First create a source directory
        let create_result = create_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_dir.to_string(),
        );
        assert!(create_result.is_ok(), "Should create source directory first");
        
        // Then copy it
        let result = copy_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_dir.to_string(),
            dest_dir.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully copy directory");
        
        // Clean up
        let _ = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            source_dir.to_string(),
        );
        let _ = delete_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            dest_dir.to_string(),
        );
    }

    #[test]
    fn test_copy_directory_sftp_failure() {
        let result = copy_directory_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_directory".to_string(),
            "dest_dir".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent directory");
    }

    #[test]
    fn test_open_file_sftp_success() {
        // Test with an existing file - let's assume there's at least one file in the test directory
        // We'll create a file first, then read it
        let test_file = "test_read_file.txt";
        
        // First create a file
        let create_result = create_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
        assert!(create_result.is_ok(), "Should create test file first");
        
        // Then try to read it
        let result = open_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
        
        assert!(result.is_ok(), "Should successfully read file");
        
        // Clean up
        let _ = delete_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            test_file.to_string(),
        );
    }

    #[test]
    fn test_open_file_sftp_failure() {
        let result = open_file_sftp(
            TEST_HOST.to_string(),
            TEST_PORT,
            TEST_USERNAME.to_string(),
            TEST_PASSWORD.to_string(),
            "nonexistent_file.txt".to_string(),
        );
        
        assert!(result.is_err(), "Should fail with nonexistent file");
    }
}