use serde::{Deserialize, Serialize};
use crate::models::LoggingLevel;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub logging_level: LoggingLevel,
    pub json_log: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            logging_level: LoggingLevel::Full,
            json_log: false,
        }
    }
}