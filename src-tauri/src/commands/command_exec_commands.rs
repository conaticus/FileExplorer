use crate::error_handling::{Error, ErrorCode};
use crate::log_info;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
struct CommandResponse {
    stdout: String,
    stderr: String,
    status: i32,
    exec_time_in_ms: u128,
}

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
    let program = parts.next();

    match program {
        Some(p) => {
            if p.is_empty() {
                return Err(Error::new(
                    ErrorCode::InvalidInput,
                    "Command is empty (before exec)".to_string(),
                )
                .to_json());
            }
        }
        None => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "Command is empty or invalid input (before exec)".to_string(),
            )
            .to_json());
        }
    }

    let start_time = std::time::Instant::now();
    
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", &command])
            .output()
    } else {
        let program = program.unwrap();
        let args: Vec<&str> = parts.collect();
        // Original Unix approach
        Command::new(program)
            .args(args)
            .output()
    }.map_err(|e| Error::new(ErrorCode::InvalidInput, e.to_string()).to_json())?;
    
    let exec_time = start_time.elapsed().as_millis();

    let res = CommandResponse {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status: output.status.code().unwrap_or(-1),
        exec_time_in_ms: exec_time,
    };

    serde_json::to_string(&res).map_err(|e| {
        Error::new(
            ErrorCode::InternalError,
            format!("Error serializing command response: {}", e),
        )
        .to_json()
    })
}

#[cfg(test)]
mod command_exec_tests {
    use crate::commands::command_exec_commands::{execute_command, CommandResponse};
    #[cfg(windows)]
    use serde_json::from_str;

    #[cfg(unix)]
    #[tokio::test]
    async fn echo_command_test_unix() {
        let result = execute_command("echo hello world".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().trim(), "hello world");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn ls_command_test_unix() {
        let result = execute_command("ls -la".to_string()).await;
        assert!(result.is_ok());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn echo_command_test_windows() {
        let result = execute_command("echo hello world".to_string()).await;
        assert!(result.is_ok());

        let json_result = result.unwrap();
        let command_response: CommandResponse = from_str(&json_result).unwrap();

        assert_eq!(command_response.stdout.trim(), "hello world");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn dir_command_test_windows() {
        let result = execute_command("dir".to_string()).await;
        assert!(result.is_ok());
    }
}
