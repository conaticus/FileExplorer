use serde::{Deserialize, Serialize};
use crate::models::LoggingLevel;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub logging_level: LoggingLevel,
    pub json_log: bool,
    pub max_log_size: Option<u64>,
    pub max_log_files: Option<usize>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            logging_level: LoggingLevel::Full,
            json_log: false,
            max_log_size: Some(5 * 1024 * 1024), //max log size in Megabytes (5 MB)
            max_log_files: Some(3), // max number of log files to keep
        }
    }
}