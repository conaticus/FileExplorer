use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SFTPDirectory {
    pub sftp_directory: String,
    pub files: Vec<String>,
    pub directories: Vec<String>,
}