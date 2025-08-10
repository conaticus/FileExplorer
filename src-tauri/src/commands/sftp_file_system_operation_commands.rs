use std::io::{Read, Write};
use ssh2::{Session, Sftp};
use std::net::TcpStream;
use crate::models::SFTPDirectory;

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
        let new_path = format!("{}/{}", destination_path, path.file_name().unwrap().to_str().unwrap());
        
        if stat.is_file() {
            // Copy file
            let mut source_file = sftp.open(&path).map_err(|e| e.to_string())?;
            let mut destination_file = sftp.create(new_path.as_ref()).map_err(|e| e.to_string())?;
            
            let mut buffer = Vec::new();
            source_file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            destination_file.write_all(&buffer).map_err(|e| e.to_string())?;
        } else if stat.is_dir() {
            // Recursively copy directory
            copy_directory_sftp(host.clone(), port, username.clone(), password.clone(), path.to_str().unwrap().to_string(), new_path)?;
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

#[cfg(test)]
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