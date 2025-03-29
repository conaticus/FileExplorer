use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    pub name: String,
    pub path: String,
    pub access_rights_as_string: String,
    pub access_rights_as_number: u32,
    pub size_in_bytes: u64,
    pub file_count: usize,
    pub sub_dir_count: usize,
    pub created: String,
    pub last_modified: String,
    pub accessed: String,
}