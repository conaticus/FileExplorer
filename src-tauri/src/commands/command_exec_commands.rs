use std::process::Command;
use crate::log_info;

/// Executes a shell command and returns its output as a string.
///
/// # Arguments
///
/// * `command` - A string representing the command to execute
///
/// # Returns
///
/// * `Ok(String)` - The combined stdout and stderr output from the command
/// * `Err(String)` - If there was an error executing the command
///
/// # Example
///
/// ```rust
/// let result = execute_command("ls -la").await;
/// match result {
///     Ok(output) => println!("Command output: {}", output),
///     Err(err) => println!("Error executing command: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn execute_command(command: String) -> Result<String, String> {
    log_info!(format!("Command: {}", command).as_str());
    
    // Split the command string into program and arguments
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or_else(|| "Empty command".to_string())?;
    let args: Vec<&str> = parts.collect();

    // Execute the command
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    // Combine stdout and stderr
    let mut result = String::new();
    
    if !output.stdout.is_empty() {
        result.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    
    if !output.stderr.is_empty() {
        if !result.is_empty() {
            result.push_str("\n");
        }
        result.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    // Add exit status if the command failed
    if !output.status.success() {
        if !result.is_empty() {
            result.push_str("\n");
        }
        result.push_str(&format!("Exit status: {}", output.status));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_command_success() {
        let result = execute_command("echo hello world".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
    }
}
