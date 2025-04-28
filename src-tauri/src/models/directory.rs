use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Directory {
    pub name: String,
    pub path: String,
    pub is_symlink: bool,
    pub access_rights_as_string: String,
    pub access_rights_as_number: u32,
    pub size_in_bytes: u64,
    pub sub_file_count: usize,
    pub sub_dir_count: usize,
    pub created: String,
    pub last_modified: String,
    pub accessed: String,
}
