use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LoggingState {
    Full,
    Partial,
    Minimal,
    OFF
}