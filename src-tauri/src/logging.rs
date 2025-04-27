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
//! ## Notes
//!
//! - Log files are automatically truncated when they exceed the maximum file size (`MAX_FILE_SIZE`).
//! - Error and critical logs are also written to a separate error log file for easier debugging.
//! - Ensure that the `SettingsState` is properly initialized and shared across the application to manage logging behavior effectively.

use std::fs::{OpenOptions, File};
use std::io::{Write, BufReader, BufRead};
use std::path::PathBuf;
use chrono::Local;
use std::fs;
use std::fmt;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use crate::constants::{LOG_FILE_NAME, ERROR_LOG_FILE_NAME, MAX_FILE_SIZE};
use crate::filesystem::models::LoggingState;
use crate::state::SettingsState;

pub enum LogLevel {
    Info,
    Warn,
    Error,
    Critical,
}

fn get_logging_state(state: Arc<Mutex<SettingsState>>) -> Result<LoggingState, String> {
    let settings_state = state.lock().unwrap();
    let inner_settings = settings_state.0.lock().map_err(|_| "Error while getting Logging state")?;
    Ok(inner_settings.logging_state.clone())
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

static LOGGER: Lazy<Logger> = Lazy::new(|| {
    let state = SettingsState::new(); // Initialize the SettingsState
    Logger::new(Arc::new(Mutex::new(state)))
});

impl Logger {
    pub fn new(state: Arc<Mutex<SettingsState>>) -> Self {
        Logger {
            log_path: PathBuf::from(LOG_FILE_NAME),
            error_log_path: PathBuf::from(ERROR_LOG_FILE_NAME),
            state,
        }
    }

    pub fn global() -> &'static Logger {
        &LOGGER
    }

    pub fn log(
        &self,
        level: LogLevel,
        file: &str,
        function: &str,
        message: &str,
        line: u32,
    ) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Retrieve the logging state
        let logging_state = match get_logging_state(self.state.clone()) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Failed to retrieve logging state: {}", e);
                return;
            }
        };

        if logging_state == LoggingState::OFF {
            return;
        }

        let entry = match logging_state {
            LoggingState::Full => format!(
                "{timestamp} - file: {file} - fn: {function} - line: {line} - {level} - {message}"
            ),
            LoggingState::Partial => format!("{timestamp} - {level} - {message}"),
            LoggingState::Minimal => format!("{level} - {message}"),
            LoggingState::OFF => return, // redundant due to early return, but kept for safety
        };

        self.write_log(&entry);

        if matches!(level, LogLevel::Error | LogLevel::Critical) {
            self.write_error_log(&entry);
        }
    }

    fn write_log(&self, entry: &str) {
        self.write_to_file(&self.log_path, entry);
    }

    fn write_error_log(&self, entry: &str) {
        self.write_to_file(&self.error_log_path, entry);
    }

    fn write_to_file(&self, path: &PathBuf, entry: &str) {
        let metadata = fs::metadata(path).ok();
        let file_size = metadata.map(|m| m.len()).unwrap_or(0);

        if file_size > MAX_FILE_SIZE {
            self.truncate_oldest_entry_from(path);
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("Failed to open log file");

        writeln!(file, "{}", entry).expect("Failed to write log");
    }

    fn truncate_oldest_entry_from(&self, path: &PathBuf) {
        let file = File::open(path).expect("Failed to open log file");
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

        if !lines.is_empty() {
            lines.remove(0);
            let mut file = File::create(path).expect("Failed to truncate log file");
            for line in lines {
                writeln!(file, "{}", line).expect("Failed to rewrite log file");
            }
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($msg:expr) => {
        $crate::logging::Logger::global().log(
            $crate::logging::LogLevel::Info,
            file!(),
            module_path!(),
            $msg,
            line!(),
        );
    };
}

#[macro_export]
macro_rules! log_warn {
    ($msg:expr) => {
        $crate::logging::Logger::global().log(
            $crate::logging::LogLevel::Warn,
            file!(),
            module_path!(),
            $msg,
            line!(),
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($msg:expr) => {
        $crate::logging::Logger::global().log(
            $crate::logging::LogLevel::Error,
            file!(),
            module_path!(),
            $msg,
            line!(),
        );
    };
}

#[macro_export]
macro_rules! log_critical {
    ($msg:expr) => {
        $crate::logging::Logger::global().log(
            $crate::logging::LogLevel::Critical,
            file!(),
            module_path!(),
            $msg,
            line!(),
        );
    };
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
        assert!(logger.log_path.exists(), "Log file should exist after logging");
    }

    #[test]
    fn test_log_warn() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_log("Test warning message");
        assert!(logger.log_path.exists(), "Log file should exist after logging");
    }

    #[test]
    fn test_log_error() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_log("Test error message");
        assert!(logger.log_path.exists(), "Log file should exist after logging");
    }

    #[test]
    fn test_error_log_creation() {
        let (logger, _temp_dir) = setup_test_logger();
        logger.write_error_log("Test error message");
        assert!(logger.error_log_path.exists(), "Error log file should exist after logging an error");
    }

    #[test]
    fn test_logging_state_full() {
        let (mut logger, _temp_dir) = setup_test_logger();
        {
            let mut state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.logging_state = LoggingState::Full;
        }

        logger.log(LogLevel::Info, "test_file.rs", "test_function", "Full logging test", 42);

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(log_content.contains("test_file.rs"), "Full logging should include file name");
        assert!(log_content.contains("test_function"), "Full logging should include function name");
        assert!(log_content.contains("line: 42"), "Full logging should include line number");
        assert!(log_content.contains("INFO"), "Full logging should include log level");
        assert!(log_content.contains("Full logging test"), "Full logging should include the message");
    }

    #[test]
    fn test_logging_state_partial() {
        let (mut logger, _temp_dir) = setup_test_logger();
        {
            let mut state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.logging_state = LoggingState::Partial;
        }

        logger.log(LogLevel::Warn, "test_file.rs", "test_function", "Partial logging test", 42);

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(!log_content.contains("test_file.rs"), "Partial logging should not include file name");
        assert!(!log_content.contains("test_function"), "Partial logging should not include function name");
        assert!(!log_content.contains("line 42"), "Partial logging should not include line number");
        assert!(log_content.contains("WARN"), "Partial logging should include log level");
        assert!(log_content.contains("Partial logging test"), "Partial logging should include the message");
    }

    #[test]
    fn test_logging_state_minimal() {
        let (mut logger, _temp_dir) = setup_test_logger();
        {
            let mut state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.logging_state = LoggingState::Minimal;
        }

        logger.log(LogLevel::Error, "test_file.rs", "test_function", "Minimal logging test", 42);

        let log_content = fs::read_to_string(&logger.log_path).expect("Failed to read log file");
        assert!(!log_content.contains("test_file.rs"), "Minimal logging should not include file name");
        assert!(!log_content.contains("test_function"), "Minimal logging should not include function name");
        assert!(!log_content.contains("line 42"), "Minimal logging should not include line number");
        assert!(log_content.contains("ERROR"), "Minimal logging should include log level");
        assert!(log_content.contains("Minimal logging test"), "Minimal logging should include the message");
    }

    #[test]
    fn test_logging_state_off() {
        let (mut logger, _temp_dir) = setup_test_logger();
        {
            let mut state = logger.state.lock().unwrap();
            let mut inner_settings = state.0.lock().unwrap();
            inner_settings.logging_state = LoggingState::OFF;
        }

        logger.log(LogLevel::Critical, "test_file.rs", "test_function", "Logging OFF test", 42);

        let log_content = fs::read_to_string(&logger.log_path).unwrap_or_default();
        assert!(log_content.is_empty(), "Logging should not occur when state is OFF");
    }
}
