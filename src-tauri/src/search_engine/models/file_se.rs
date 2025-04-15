use std::fs::DirEntry;
use serde::{Deserialize, Serialize};
use crate::models::{format_system_time, get_access_permission_number, get_access_permission_string};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct File_SE {
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


impl File_SE {
    /// Creates a new File struct from a DirEntry
    ///
    /// # Arguments
    /// * `entry` - The DirEntry to convert
    ///
    /// # Returns
    /// * `Result<File>` - The created File or an error
    pub fn from_dir_entry(entry: DirEntry) -> std::io::Result<Self> {
        let path_of_entry = entry.path();
        let metadata = entry.metadata()?;

        Ok(File_SE {
            name: entry.file_name().to_str().unwrap_or("").to_string(),
            path: path_of_entry.to_str().unwrap_or("").to_string(),
            is_symlink: path_of_entry.is_symlink(),
            access_rights_as_string: get_access_permission_string(metadata.permissions(), false),
            access_rights_as_number: get_access_permission_number(metadata.permissions()),
            size_in_bytes: metadata.len(),
            created: format_system_time(metadata.created()?),
            last_modified: format_system_time(metadata.modified()?),
            accessed: format_system_time(metadata.accessed()?),
        })
    }
}