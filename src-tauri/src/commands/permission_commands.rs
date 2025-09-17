use std::process::Command;

#[tauri::command]
pub fn request_full_disk_access() -> Result<(), String> {
    // Check if we're on macOS
    #[cfg(target_os = "macos")]
    {
        // Open System Preferences to Full Disk Access
        let result = Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
            .output();
            
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to open System Preferences: {}", e))
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}

#[tauri::command]
pub fn check_directory_access(path: String) -> Result<bool, String> {
    use std::fs;
    
    match fs::read_dir(&path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false)
    }
}

#[cfg(test)]
mod permission_commands_tests {
    use super::*;

    #[test]
    fn test_check_directory_access_existing() {
        // Should succeed for a directory that exists and is accessible
        let path = std::env::temp_dir().to_string_lossy().to_string();
        let result = check_directory_access(path);
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_check_directory_access_nonexistent() {
        // Should return Ok(false) for a directory that does not exist
        let path = "/nonexistent/directory/for/test".to_string();
        let result = check_directory_access(path);
        assert_eq!(result, Ok(false));
    }

    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_request_full_disk_access_noop_non_macos() {
        // On non-macOS, should always return Ok(();
        {
            let result = request_full_disk_access();
            assert_eq!(result, Ok(()));
        }
    }
}

