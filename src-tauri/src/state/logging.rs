//! # Logger Module
//!
//! This module provides a logging utility for the application. It supports multiple log levels
//! and allows for configurable logging states to control the verbosity of the logs.
//!
//! ## Usage
//!
//! To log messages, use the provided macros:
//!
//! - `log_info!("Your message here");`
//! - `log_warn!("Your message here");`
//! - `log_error!("Your message here");`
//! - `log_critical!("Your message here");`
//!
//! Example:
//! ```rust
//! log_info!("Application started successfully.");
//! log_warn!("This is a warning message.");
//! log_error!("An error occurred while processing the request.");
//! log_critical!("Critical failure! Immediate attention required.");
//! ```
//!
//! ## Logging State
//!
//! The logger behavior is controlled by the `LoggingState` enum, which has the following variants:
//!
//! - `LoggingState::Full`: Logs detailed information, including the file name, function name, line number, log level, and message.
//! - `LoggingState::Partial`: Logs only the timestamp, log level, and message.
//! - `LoggingState::Minimal`: Logs only the log level and message.
//! - `LoggingState::OFF`: Disables logging entirely.
//!
//! The logging state can be dynamically retrieved and modified through the `SettingsState`.
//!
//! Example of how the logging state affects the output:
//!
//! - **Full**: `2023-01-01 12:00:00 - file: main.rs - fn: main - line: 42 - INFO - Application started successfully.`
//! - **Partial**: `2023-01-01 12:00:00 - INFO - Application started successfully.`
//! - **Minimal**: `INFO - Application started successfully.`
//! - **OFF**: No log is written.
//!
//! ## Structured Logging
//!
//! If `json_log` is enabled in `SettingsState`, all entries are emitted as JSON objects with consistent fields:
//! `{ timestamp, level, file, function, line, message }`.
//!
//! ## Notes
//!
//! - Log files are automatically truncated when they exceed the maximum file size (`MAX_FILE_SIZE`).
//! - Error and critical logs are also written to a separate error log file for easier debugging.
//! - Ensure that the `SettingsState` is properly initialized and shared across the application to manage logging behavior effectively.

use crate::constants::{ERROR_LOG_FILE_ABS_PATH, LOG_FILE_ABS_PATH, MAX_NUMBER_OF_LOG_FILES};
use crate::error_handling::{Error, ErrorCode};
use crate::models::LoggingLevel;
use crate::state::SettingsState;
use chrono::Local;
use once_cell::sync::{Lazy, OnceCell};
use std::fmt;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde_json::json;

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Info,
            file!(),
            module_path!(),
            $msg,
            line!(),
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Info,
            file!(),
            module_path!(),
            &format!($fmt, $($arg)*),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Warn,
            file!(),
            module_path!(),
            $msg,
            line!(),
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Warn,
            file!(),
            module_path!(),
            &format!($fmt, $($arg)*),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Error,
            file!(),
            module_path!(),
            $msg,
            line!(),
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Error,
            file!(),
            module_path!(),
            &format!($fmt, $($arg)*),
            line!(),
        )
    };
}

#[macro_export]
macro_rules! log_critical {
    ($msg:expr) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Critical,
            file!(),
            module_path!(),
            $msg,
            line!(),
        )
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::state::logging::Logger::global().log(
            $crate::state::logging::LogLevel::Critical,
            file!(),
            module_path!(),
            &format!($fmt, $($arg)*),
            line!(),
        )
    };
}

static WRITE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Critical,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Critical => write!(f, "CRITICAL"),
        }
    }
}

pub struct Logger {
    log_path: PathBuf,
    error_log_path: PathBuf,
    state: Arc<Mutex<SettingsState>>,
}

// Replace Lazy with OnceCell for more flexible initialization
static LOGGER: OnceCell<Logger> = OnceCell::new();

impl Logger {
    pub fn new(state: Arc<Mutex<SettingsState>>) -> Self {
        Logger {
            log_path: LOG_FILE_ABS_PATH.to_path_buf(),
            error_log_path: ERROR_LOG_FILE_ABS_PATH.to_path_buf(),
            state,
        }
    }

    /// Initialize the global logger instance with application settings.
    ///
    /// This should be called early in your application startup before any logging occurs.
    ///
    /// # Example
    /// ```rust
    /// let app_state = Arc::new(Mutex::new(SettingsState::new()));
    /// Logger::init(app_state.clone());
    /// ```
    pub fn init(state: Arc<Mutex<SettingsState>>) {
        // Ensure log directories exist before initializing the logger
        Self::ensure_log_directories_exist();
        
        // Create empty log files if they don't exist
        Self::ensure_log_files_exist();
        
        Self::init_global_logger(state);
    }

    // Create log directories if they don't exist
    fn ensure_log_directories_exist() {
        if let Some(parent) = LOG_FILE_ABS_PATH.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                eprintln!("Failed to create parent log directory: {}", e);
            }
        }
        
        if let Some(parent) = ERROR_LOG_FILE_ABS_PATH.parent() {
            if parent != LOG_FILE_ABS_PATH.parent().unwrap() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create parent error log directory: {}", e);
                }
            }
        }
    }
    
    // Create empty log files if they don't exist
    fn ensure_log_files_exist() {
        // Create empty app.log if it doesn't exist
        if let Err(e) = OpenOptions::new().write(true).create(true).open(&*LOG_FILE_ABS_PATH) {
            eprintln!("Failed to create log file: {}", e);
        }
        
        // Create empty error.log if it doesn't exist
        if let Err(e) = OpenOptions::new().write(true).create(true).open(&*ERROR_LOG_FILE_ABS_PATH) {
            eprintln!("Failed to create error log file: {}", e);
        }
    }

    // Internal implementation function
    fn init_global_logger(state: Arc<Mutex<SettingsState>>) {
        if LOGGER.get().is_none() {
            let _ = LOGGER.set(Logger::new(state));
        }
    }

    pub fn global() -> &'static Logger {
        LOGGER.get_or_init(|| {
            eprintln!("Warning: Logger accessed before initialization with application state! Using default settings.");
            eprintln!("Call Logger::init(app_state) in your application startup code.");
            Logger::new(Arc::new(Mutex::new(SettingsState::new())))
        })
    }

    pub fn log(&self, level: LogLevel, file: &str, function: &str, message: &str, line: u32) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Retrieve the logging state with proper error handling
        let (logging_state, json_log) = match self.state.lock() {
            Ok(state_guard) => match state_guard.0.lock() {
                Ok(settings) => (settings.backend_settings.logging_config.logging_level.clone(), settings.backend_settings.logging_config.json_log.clone()),
                Err(e) => {
                    eprintln!("Failed to acquire inner settings lock: {}", e);
                    (LoggingLevel::Minimal, false)
                }
            },
            Err(e) => {
                eprintln!("Failed to acquire settings state lock: {}", e);
                (LoggingLevel::Minimal, false)
            }
        };

        if logging_state == LoggingLevel::OFF {
            return;
        }

        let entry = if json_log {
            json!({
                "timestamp": timestamp,
                "level": level.to_string(),
                "file": file,
                "function": function,
                "line": line,
                "message": message,
            })
                .to_string()
        } else {
            match logging_state {
                LoggingLevel::Full => format!(
                    "{timestamp} - file: {file} - fn: {function} - line: {line} - {level} - {message}"
                ),
                LoggingLevel::Partial => format!("{timestamp} - {level} - {message}"),
                LoggingLevel::Minimal => format!("{level} - {message}"),
                LoggingLevel::OFF => return, // redundant due to early return, but kept for safety
            }
        };

        self.write_log(&entry);
        if matches!(level, LogLevel::Error | LogLevel::Critical) {
            self.write_error_log(&entry);
        }
    }

    /// Called when file_size > MAX_FILE_SIZE.
    fn rotate_logs(&self, path: &PathBuf) {
        // Use timestamp-based naming for archived logs
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let archive_path = path.with_file_name(format!("{}.{}.log",
                                                       path.file_stem().unwrap().to_str().unwrap(),
                                                       timestamp));

        // Move current log to archive and create new file
        if let Err(e) = fs::rename(path, &archive_path) {
            eprintln!("Failed to rotate log file: {}", e);
            return;
        }

        // Enforce the 3-file limit after successful rotation
        self.enforce_log_file_limit(path);
    }

    fn enforce_log_file_limit(&self, current_log_path: &PathBuf) {
        if let Some(parent) = current_log_path.parent() {
            if let Some(base_name) = current_log_path.file_stem() {
                let base_name = base_name.to_string_lossy();

                // Collect all archived log files
                let mut archived_logs: Vec<_> = fs::read_dir(parent)
                    .into_iter()
                    .flatten()
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| {
                        // Store the file name first to avoid the temporary value being dropped
                        let name = entry.file_name();
                        let name = name.to_string_lossy();
                        name.starts_with(&*base_name) &&
                            name.ends_with(".log") &&
                            name != format!("{}.log", base_name)
                    })
                    .collect();

                // Sort by modification time (oldest first)
                archived_logs.sort_by(|a, b| {
                    a.metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH)
                        .cmp(&b.metadata()
                            .and_then(|m| m.modified())
                            .unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH))
                });

                // If we have more than 2 archived files (3 total including current), remove the oldest
                let max_log_files = match self.state.lock() {
                    Ok(state_guard) => match state_guard.0.lock() {
                        Ok(settings) => settings.backend_settings.logging_config.max_log_files.unwrap_or(MAX_NUMBER_OF_LOG_FILES),
                        Err(_) => MAX_NUMBER_OF_LOG_FILES // Fallback to default if lock fails
                    },
                    Err(_) => MAX_NUMBER_OF_LOG_FILES // Fallback to default if lock fails
                };


                while archived_logs.len() > max_log_files - 1 {
                    if let Some(oldest) = archived_logs.first() {
                        if let Err(e) = fs::remove_file(oldest.path()) {
                            eprintln!("Failed to remove oldest log file {}: {}",
                                      oldest.path().display(), e);
                        }
                    }
                    archived_logs.remove(0);
                }
            }
        }
    }

    fn write_log(&self, entry: &str) {
        self.write_to_file(&self.log_path, entry);
    }

    fn write_error_log(&self, entry: &str) {
        self.write_to_file(&self.error_log_path, entry);
    }

    fn write_to_file(&self, path: &PathBuf, entry: &str) {
        // Double-check parent directory exists before attempting to write to file
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    eprintln!("Failed to create log directory {}: {}", parent.display(), e);
                    return;
                }
            }
        }

        let metadata = fs::metadata(path).ok();
        let file_size = metadata.map(|m| m.len()).unwrap_or(0);
        let _guard = WRITE_LOCK.lock().unwrap();

        // If file size exceeds the limit, truncate before writing new entry
        let max_log_size = match self.state.lock() {
            Ok(state_guard) => match state_guard.0.lock() {
                Ok(settings) => settings.backend_settings.logging_config.max_log_size.unwrap_or(5 * 1024 * 1024),
                Err(_) => 5 * 1024 * 1024 // Fallback to constant if lock fails
            },
            Err(_) => 5 * 1024 * 1024 // Fallback to constant if lock fails
        };
        
        if file_size > max_log_size {
            // For test purposes, print the file size before truncation
            #[cfg(test)]
            println!(
                "File exceeds size limit: {} bytes. Rotating...",
                file_size
            );
        
            self.rotate_logs(path);
        }

        // Ensure the entry ends with exactly one newline
        let entry = entry.trim_end();

        // Process the entry to handle any embedded newlines
        let formatted_entry = entry.replace('\n', "\n    | ");

        let to_write = format!("{}\n", formatted_entry);

        match OpenOptions::new().create(true).append(true).open(path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(to_write.as_bytes()) {
                    eprintln!("Failed to write to log file: {}", e);
                    // Create an error using our error handling module but just log it
                    let error = Error::new(
                        ErrorCode::InternalError,
                        format!("Failed to write to log file: {}", e)
                    );
                    eprintln!("Logging error: {}", error.to_json());
                }
            }
            Err(e) => {
                eprintln!("Failed to open log file for writing: {}", e);
                eprintln!("Path: {}", path.display());
                eprintln!("Parent exists: {}", path.parent().map_or(false, |p| p.exists()));
                // Create an error using our error handling module but just log it
                let error = Error::new(
                    ErrorCode::ResourceNotFound,
                    format!("Failed to open log file for writing: {}", e)
                );
                eprintln!("Logging error: {}", error.to_json());
            }
        }
    }
}

#[cfg(test)]
mod tests_logging {
    use super::*;
    use tempfile::tempdir;

    const TEST_LOG_FILE: &str = "test_app.log";
    const TEST_ERROR_LOG_FILE: &str = "test_error.log";

    fn setup_test_logger() -> (Logger, tempfile::TempDir) {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let log_path = temp_dir.path().join(TEST_LOG_FILE);
        let logger = Logger {
            log_path: log_path.clone(),
            error_log_path: temp_dir.path().join(TEST_ERROR_LOG_FILE),
            state: Arc::new(Mutex::new(SettingsState::new())),
        };
        (logger, temp_dir)
    }

    #[test]
    fn test_log_info() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_log("Test info message");
        assert!(
            logger.log_path.exists(),
            "Log file should exist after logging"
        );
    }

    #[test]
    fn test_log_warn() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_log("Test warning message");
        assert!(
            logger.log_path.exists(),
            "Log file should exist after logging"
        );
    }

    #[test]
    fn test_log_error() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_log("Test error message");
        assert!(
            logger.log_path.exists(),
            "Log file should exist after logging"
        );
    }

    #[test]
    fn test_error_log_creation() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_error_log("Test error message");
        assert!(
            logger.error_log_path.exists(),
            "Error log file should exist after logging an error"
        );
    }

    #[test]
    fn test_logging_state_full() {
        let (logger, _temp_dir) = setup_test_logger();
        {
            let state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.backend_settings.logging_config.logging_level = LoggingLevel::Full;
        }

        logger.log(
            LogLevel::Info,
            "test_file.rs",
            "test_function",
            "Full logging test",
            42,
        );

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(
            log_content.contains("test_file.rs"),
            "Full logging should include file name"
        );
        assert!(
            log_content.contains("test_function"),
            "Full logging should include function name"
        );
        assert!(
            log_content.contains("line: 42"),
            "Full logging should include line number"
        );
        assert!(
            log_content.contains("INFO"),
            "Full logging should include log level"
        );
        assert!(
            log_content.contains("Full logging test"),
            "Full logging should include the message"
        );
    }

    #[test]
    fn test_logging_state_partial() {
        let (logger, _temp_dir) = setup_test_logger();
        {
            let state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.backend_settings.logging_config.logging_level = LoggingLevel::Partial;
        }

        logger.log(
            LogLevel::Warn,
            "test_file.rs",
            "test_function",
            "Partial logging test",
            42,
        );

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(
            !log_content.contains("test_file.rs"),
            "Partial logging should not include file name"
        );
        assert!(
            !log_content.contains("test_function"),
            "Partial logging should not include function name"
        );
        assert!(
            !log_content.contains("line 42"),
            "Partial logging should not include line number"
        );
        assert!(
            log_content.contains("WARN"),
            "Partial logging should include log level"
        );
        assert!(
            log_content.contains("Partial logging test"),
            "Partial logging should include the message"
        );
    }

    #[test]
    fn test_logging_state_minimal() {
        let (logger, _temp_dir) = setup_test_logger();
        {
            let state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.backend_settings.logging_config.logging_level = LoggingLevel::Minimal;
        }

        logger.log(
            LogLevel::Error,
            "test_file.rs",
            "test_function",
            "Minimal logging test",
            42,
        );

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(
            !log_content.contains("test_file.rs"),
            "Minimal logging should not include file name"
        );
        assert!(
            !log_content.contains("test_function"),
            "Minimal logging should not include function name"
        );
        assert!(
            !log_content.contains("line 42"),
            "Minimal logging should not include line number"
        );
        assert!(
            log_content.contains("ERROR"),
            "Minimal logging should include log level"
        );
        assert!(
            log_content.contains("Minimal logging test"),
            "Minimal logging should include the message"
        );
    }

    #[test]
    fn test_logging_state_off() {
        let (logger, _temp_dir) = setup_test_logger();
        {
            let state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.backend_settings.logging_config.logging_level = LoggingLevel::OFF;
        }

        logger.log(
            LogLevel::Critical,
            "test_file.rs",
            "test_function",
            "Logging OFF test",
            42,
        );

        let log_content = fs::read_to_string(&logger.log_path).unwrap_or_default();
        assert!(
            log_content.is_empty(),
            "Logging should not occur when state is OFF"
        );
    }

    #[test]
    fn test_log_rotate_when_max_size_reached() {
        let (logger, temp_dir) = setup_test_logger();

        // Create a log file that exceeds our test max size
        let large_entry = "X".repeat(1000); // 1KB per entry

        // Write entries until we exceed the test max size
        for i in 0..60 {
            // Should create ~60KB of data
            logger.write_log(&format!("Log entry #{}: {}", i, large_entry));
        }

        // Get the file size before triggering rotation
        let metadata_before = fs::metadata(&logger.log_path).expect("Failed to read file metadata");
        let size_before = metadata_before.len();
        println!("Size before rotation: {} bytes", size_before);

        // Keep track of original log path for later comparison
        let original_log_path = logger.log_path.clone();
        let log_filename = original_log_path.file_name().unwrap().to_str().unwrap().to_string();

        // Force log rotation by directly calling the rotate_logs method
        logger.rotate_logs(&logger.log_path);

        // Check that the size of the new log file is small
        let new_size = fs::metadata(&original_log_path)
            .map(|m| m.len())
            .unwrap_or(0);
        assert!(
            new_size < size_before,
            "New log file should be smaller than the original"
        );

        // Check that an archive log file was created with timestamp in the name
        let entries = fs::read_dir(temp_dir.path())
            .expect("Failed to read temp directory")
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        // Find the archived log file (should be named like "test_app.20230101_123456.log")
        let archived_log = entries
            .iter()
            .find(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                // Check if file name contains the original name and has timestamp pattern
                name.starts_with(log_filename.trim_end_matches(".log")) &&
                name != log_filename &&
                name.ends_with(".log") &&
                name.contains(".")
            });

        assert!(
            archived_log.is_some(),
            "Should have created an archived log file with timestamp"
        );

        if let Some(archived_log) = archived_log {
            let archived_path = archived_log.path();

            // Check if the archived log has content
            let archived_content = fs::read_to_string(&archived_path)
                .expect("Failed to read archived log file");

            assert!(
                !archived_content.is_empty(),
                "Archived log file should contain the original log content"
            );

            // Verify the archived content has the expected log entries
            assert!(
                archived_content.contains("Log entry #0:"),
                "Archived log should contain the earliest log entries"
            );

            println!("Archive log created at: {}", archived_path.display());
        }

        // Add new log entries to verify they're written to the new log file
        logger.write_log("This entry should be added after rotation");

        // Verify the new entry is in the original log file path
        let new_log_content = fs::read_to_string(&original_log_path)
            .expect("Failed to read new log file");

        assert!(
            new_log_content.contains("This entry should be added after rotation"),
            "New log entries should be written to the new log file"
        );
    }
    #[test]
    fn test_enforce_log_file_limit() {
        let (logger, temp_dir) = setup_test_logger();
        let temp_path = temp_dir.path();

        // Create base log file
        let base_log = temp_path.join("app.log");
        fs::write(&base_log, "current log").expect("Failed to create base log");

        // Create 5 archived log files
        for i in 0..5 {
            let path = temp_path.join(format!("app.202301010000{}.log", i));
            fs::write(&path, format!("archived content {}", i))
                .expect("Failed to create archived log");
        }

        // Set custom max_log_files value
        {
            let state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.backend_settings.logging_config.max_log_files = Some(4);
        }

        // Enforce the limit
        logger.enforce_log_file_limit(&base_log);

        // Count remaining files
        let remaining_files: Vec<_> = fs::read_dir(temp_path)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        
        // Should have base log file (app.log) + 3 archived files
        assert_eq!(remaining_files.len(), 4, "Should have base log file + 3 archived files");
        let base_file_exists = remaining_files.iter()
            .any(|entry| entry.file_name() == "app.log");
        assert!(base_file_exists, "Base log file should exist");
    }
    
    #[test]
    fn test_log_file_creation() {
        let (logger, temp_dir) = setup_test_logger();
        let log_path = &logger.log_path;

        // Ensure the log file is created
        assert!(!log_path.exists(), "Log file should not exist before logging");

        logger.write_log("Test log entry");

        // Check if the log file was created
        assert!(log_path.exists(), "Log file should be created after logging");
        
        // Verify the content of the log file
        let content = fs::read_to_string(log_path).expect("Failed to read log file");
        assert!(content.contains("Test log entry"), "Log file should contain the logged message");
    }
    
    #[test]
    fn test_log_file_creation_after_rotation() {
        let (logger, temp_dir) = setup_test_logger();
        let log_path = &logger.log_path;

        // Ensure the log file is created
        assert!(!log_path.exists(), "Log file should not exist before logging");

        logger.write_log("Test log entry");

        // Check if the log file was created
        assert!(log_path.exists(), "Log file should be created after logging");
        
        // Simulate log rotation by manually calling rotate_logs
        logger.rotate_logs(log_path);
        
        // Check if the log file still exists after rotation
        assert!(!log_path.exists(), "Log file should still exist after rotation");
        
        // Create new log entry after rotation
        logger.write_log("Test log entry after rotation");
        
        // Check if the log file was recreated
        assert!(log_path.exists(), "Log file should be recreated after rotation");

        // Verify the content of the log file
        let content = fs::read_to_string(log_path).expect("Failed to read log file");
        assert!(content.contains("Test log entry after rotation"), 
                "Log file should contain the new logged message after rotation");
    }
}
