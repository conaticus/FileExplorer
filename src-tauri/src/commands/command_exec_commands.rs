use crate::error_handling::{Error, ErrorCode};
use crate::log_info;
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::env;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;
use tokio::process::Command as TokioCommand;

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
/// * `working_directory` - Optional working directory to run the command in
///
/// # Returns
///
/// * `Ok(String)` - The combined stdout and stderr output from the command
/// * `Err(String)` - If there was an error executing the command
///
/// # Example
///
/// ```rust
/// let result = execute_command("ls -la".to_string(), Some("/home/user".to_string())).await;
/// match result {
///     Ok(output) => println!("Command output: {}", output),
///     Err(err) => println!("Error executing command: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn execute_command(command: String, working_directory: Option<String>) -> Result<String, String> {
    log_info!("Command: {}", command);

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

    // Get the shell to use
    let shell_path = if cfg!(target_os = "windows") {
        "cmd".to_string()
    } else {
        // Prefer user's shell, fallback to sh
        env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    };
    
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    let mut cmd = Command::new(&shell_path);
    cmd.arg(shell_arg).arg(&command);
    
    // Set working directory if provided, with validation
    if let Some(ref wd) = working_directory {
        let path = Path::new(wd);
        if path.exists() && path.is_dir() {
            cmd.current_dir(wd);
        } else {
            // If working directory doesn't exist, try to use home directory
            if let Ok(home_dir) = env::var("HOME") {
                cmd.current_dir(home_dir);
            }
        }
    } else {
        // Set a reasonable default working directory
        if let Ok(home_dir) = env::var("HOME") {
            cmd.current_dir(home_dir);
        }
    }
    
    // Set up environment variables for better compatibility
    cmd.env("TERM", "xterm-256color");
    if !cfg!(target_os = "windows") {
        cmd.env("PATH", env::var("PATH").unwrap_or_default());
    }
    
    // Configure stdio for proper output capture
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| {
            let error_msg = match e.kind() {
                std::io::ErrorKind::NotFound => format!("Command '{}' not found. Make sure it's installed and in your PATH.", program.unwrap_or("unknown")),
                std::io::ErrorKind::PermissionDenied => format!("Permission denied executing command '{}'. Check file permissions.", program.unwrap_or("unknown")),
                _ => format!("Failed to execute command '{}': {}", program.unwrap_or("unknown"), e)
            };
            Error::new(ErrorCode::InvalidInput, error_msg).to_json()
        })?;

    let exec_time = start_time.elapsed().as_millis();

    // Handle output with proper encoding
    let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();
    
    // Get proper exit code
    let status_code = if let Some(code) = output.status.code() {
        code
    } else {
        // Process was terminated by signal on Unix
        if cfg!(unix) {
            128 + 9 // SIGKILL equivalent
        } else {
            -1
        }
    };
    
    let res = CommandResponse {
        stdout,
        stderr,
        status: status_code,
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

/// Executes a shell command with better error handling and environment setup.
/// This is an improved version of execute_command with streaming capabilities.
#[tauri::command]
pub async fn execute_command_improved(
    command: String,
    working_directory: Option<String>,
) -> Result<String, String> {
    log_info!("Improved Command: {}", command);

    // Validate command
    let mut parts = command.split_whitespace();
    let program = parts.next();

    match program {
        Some(p) => {
            if p.is_empty() {
                return Err(Error::new(
                    ErrorCode::InvalidInput,
                    "Command is empty".to_string(),
                )
                .to_json());
            }
        }
        None => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "No command provided".to_string(),
            )
            .to_json());
        }
    }

    let start_time = std::time::Instant::now();

    // Get the appropriate shell
    let shell_path = if cfg!(target_os = "windows") {
        "powershell".to_string()
    } else {
        // Use user's preferred shell or fallback to bash/sh
        env::var("SHELL").unwrap_or_else(|_| {
            if Path::new("/bin/bash").exists() {
                "/bin/bash".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })
    };
    
    let shell_arg = if cfg!(target_os = "windows") {
        "-Command"
    } else {
        "-c"
    };

    let mut cmd = Command::new(&shell_path);
    cmd.arg(shell_arg).arg(&command);
    
    // Set working directory with validation
    if let Some(ref wd) = working_directory {
        let path = Path::new(wd);
        if path.exists() && path.is_dir() {
            cmd.current_dir(wd);
        } else {
            log_info!("Working directory '{}' not found, using default", wd);
            // Use home directory as fallback
            if let Ok(home_dir) = env::var("HOME") {
                cmd.current_dir(home_dir);
            }
        }
    }
    
    // Set up proper environment
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    
    if !cfg!(target_os = "windows") {
        // Preserve PATH and add common binary directories
        let current_path = env::var("PATH").unwrap_or_default();
        let extended_path = format!("{}:/usr/local/bin:/usr/bin:/bin", current_path);
        cmd.env("PATH", extended_path);
        
        // Set locale for proper character encoding
        cmd.env("LC_ALL", "en_US.UTF-8");
        cmd.env("LANG", "en_US.UTF-8");
    }
    
    // Configure stdio
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd
        .output()
        .map_err(|e| {
            let error_msg = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    format!("Command '{}' not found. Please check if it's installed and in your PATH.", program.unwrap_or("unknown"))
                },
                std::io::ErrorKind::PermissionDenied => {
                    format!("Permission denied executing '{}'. Check file permissions or run with appropriate privileges.", program.unwrap_or("unknown"))
                },
                std::io::ErrorKind::InvalidInput => {
                    format!("Invalid command format: '{}'", command)
                },
                _ => format!("Failed to execute '{}': {}", program.unwrap_or("unknown"), e)
            };
            Error::new(ErrorCode::InvalidInput, error_msg).to_json()
        })?;

    let exec_time = start_time.elapsed().as_millis();

    // Handle output with proper encoding and cleanup
    let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();
    
    // Get proper exit code with signal handling
    let status_code = if let Some(code) = output.status.code() {
        code
    } else {
        // Process was terminated by signal (Unix only)
        if cfg!(unix) {
            128 + 15 // SIGTERM equivalent
        } else {
            -1
        }
    };
    
    let res = CommandResponse {
        stdout,
        stderr,
        status: status_code,
        exec_time_in_ms: exec_time,
    };

    serde_json::to_string(&res).map_err(|e| {
        Error::new(
            ErrorCode::InternalError,
            format!("Error serializing response: {}", e),
        )
        .to_json()
    })
}

/// Executes a shell command with timeout support for long-running commands.
/// This version handles commands like ping that might run indefinitely.
#[tauri::command]
pub async fn execute_command_with_timeout(
    command: String,
    working_directory: Option<String>,
    timeout_seconds: Option<u64>,
) -> Result<String, String> {
    log_info!("Command with timeout: {}", command);

    // Validate command
    let mut parts = command.split_whitespace();
    let program = parts.next();

    match program {
        Some(p) => {
            if p.is_empty() {
                return Err(Error::new(
                    ErrorCode::InvalidInput,
                    "Command is empty".to_string(),
                )
                .to_json());
            }
        }
        None => {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "No command provided".to_string(),
            )
            .to_json());
        }
    }

    let start_time = std::time::Instant::now();

    // Auto-modify certain commands to prevent infinite running
    let modified_command = if command.starts_with("ping ") && !command.contains(" -c ") && !command.contains(" -n ") {
        if cfg!(target_os = "windows") {
            format!("{} -n 4", command) // Windows: send 4 packets
        } else {
            format!("{} -c 4", command) // Unix: send 4 packets
        }
    } else {
        command.clone()
    };

    // Get the appropriate shell
    let shell_path = if cfg!(target_os = "windows") {
        "powershell".to_string()
    } else {
        env::var("SHELL").unwrap_or_else(|_| {
            if Path::new("/bin/bash").exists() {
                "/bin/bash".to_string()
            } else {
                "/bin/sh".to_string()
            }
        })
    };
    
    let shell_arg = if cfg!(target_os = "windows") {
        "-Command"
    } else {
        "-c"
    };

    let mut cmd = TokioCommand::new(&shell_path);
    cmd.arg(shell_arg).arg(&modified_command);
    
    // Set working directory
    if let Some(ref wd) = working_directory {
        let path = Path::new(wd);
        if path.exists() && path.is_dir() {
            cmd.current_dir(wd);
        } else if let Ok(home_dir) = env::var("HOME") {
            cmd.current_dir(home_dir);
        }
    }
    
    // Set environment
    cmd.env("TERM", "xterm-256color");
    if !cfg!(target_os = "windows") {
        let current_path = env::var("PATH").unwrap_or_default();
        let extended_path = format!("{}:/usr/local/bin:/usr/bin:/bin", current_path);
        cmd.env("PATH", extended_path);
    }
    
    // Configure stdio
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Set timeout (default 30 seconds for potentially long-running commands)
    let timeout_duration = Duration::from_secs(timeout_seconds.unwrap_or(30));

    let result = timeout(timeout_duration, cmd.output()).await;

    let output = match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            let error_msg = match e.kind() {
                std::io::ErrorKind::NotFound => {
                    format!("Command '{}' not found", program.unwrap_or("unknown"))
                },
                std::io::ErrorKind::PermissionDenied => {
                    format!("Permission denied: '{}'", program.unwrap_or("unknown"))
                },
                _ => format!("Failed to execute: {}", e)
            };
            return Err(Error::new(ErrorCode::InvalidInput, error_msg).to_json());
        },
        Err(_) => {
            // Timeout occurred
            return Err(Error::new(
                ErrorCode::InvalidInput, 
                format!("Command '{}' timed out after {} seconds. Use Ctrl+C to cancel long-running commands.", 
                       modified_command, timeout_duration.as_secs())
            ).to_json());
        }
    };

    let exec_time = start_time.elapsed().as_millis();
    let stdout = String::from_utf8_lossy(&output.stdout).trim_end().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim_end().to_string();
    let status_code = output.status.code().unwrap_or(-1);

    let res = CommandResponse {
        stdout,
        stderr,
        status: status_code,
        exec_time_in_ms: exec_time,
    };

    serde_json::to_string(&res).map_err(|e| {
        Error::new(
            ErrorCode::InternalError,
            format!("Error serializing response: {}", e),
        )
        .to_json()
    })
}


#[cfg(test)]
mod command_exec_tests {
    use crate::commands::command_exec_commands::{execute_command, CommandResponse};
    use serde_json::from_str;

    #[cfg(unix)]
    #[tokio::test]
    async fn echo_command_test_unix() {
        let result = execute_command("echo hello world".to_string(), None).await;
        assert!(result.is_ok());

        let json_result = result.unwrap();
        let command_response: CommandResponse = from_str(&json_result).unwrap();
        assert_eq!(command_response.stdout.trim(), "hello world");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn ls_command_test_unix() {
        let result = execute_command("ls -la".to_string(), None).await;
        assert!(result.is_ok());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn working_directory_test_unix() {
        let result = execute_command("pwd".to_string(), Some("/tmp".to_string())).await;
        assert!(result.is_ok());

        let json_result = result.unwrap();
        let command_response: CommandResponse = from_str(&json_result).unwrap();
        assert!(command_response.stdout.contains("/tmp"));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn echo_command_test_windows() {
        let result = execute_command("echo hello world".to_string(), None).await;
        assert!(result.is_ok());

        let json_result = result.unwrap();
        let command_response: CommandResponse = from_str(&json_result).unwrap();

        assert_eq!(command_response.stdout.trim(), "hello world");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn dir_command_test_windows() {
        let result = execute_command("dir".to_string(), None).await;
        assert!(result.is_ok());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn working_directory_test_windows() {
        let result = execute_command("cd".to_string(), Some("C:\\".to_string())).await;
        assert!(result.is_ok());
    }
}
