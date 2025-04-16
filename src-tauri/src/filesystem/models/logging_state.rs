use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum LoggingState {
    Full,
    Partial,
    None
}