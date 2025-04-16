use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DefaultDownloadLocation {
    pub name: String,
    pub url: String,
    pub path: PathBuf,
}