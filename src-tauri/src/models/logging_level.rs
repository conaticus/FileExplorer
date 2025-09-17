use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoggingLevel {
    Full,
    Partial,
    Minimal,
    OFF,
}
