use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct File {
    pub name: String,
    pub path: String,
    pub is_symlink: bool,
    pub access_rights_as_string: String,
    pub access_rights_as_number: u32,
    pub size_in_bytes: u64,
    pub created: String,
    pub last_modified: String,
    pub accessed: String,
}
