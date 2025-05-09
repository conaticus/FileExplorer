use serde::{Deserialize, Serialize};
use crate::search_engine;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct DirectorySe {
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
    pub sub_entries: Vec<search_engine::Entry>
}

use std::fs::{DirEntry, read_dir};
use std::io::Result;
use crate::models::{format_system_time, get_access_permission_number, get_access_permission_string};
use crate::search_engine::Entry;
use crate::search_engine::models::file_se::FileSe;

impl DirectorySe {
    /// Creates a new Directory struct from a DirEntry
    ///
    /// # Arguments
    /// * `entry` - The DirEntry to convert
    ///
    /// # Returns
    /// * `Result<Directory>` - The created Directory or an error
    pub fn from_dir_entry(entry: &DirEntry) -> Result<Self> {
        let path_of_entry = entry.path();
        let metadata = entry.metadata()?;
        let mut sub_entries = Vec::new();
        let mut sub_file_count = 0;
        let mut sub_dir_count = 0;
    
        // Read and process subdirectories and files
        if let Ok(entries) = read_dir(&path_of_entry) {
            for sub_entry_result in entries {
                if let Ok(sub_entry) = sub_entry_result {
                    if let Ok(sub_metadata) = sub_entry.metadata() {
                        if sub_metadata.is_file() {
                            // Handle files
                            match FileSe::from_dir_entry(&sub_entry) {
                                Ok(file) => {
                                    sub_entries.push(Entry::FILE(file));
                                    sub_file_count += 1;
                                }
                                Err(_) => continue
                            }
                        } else if sub_metadata.is_dir() {
                            // For directories, process recursively
                            match DirectorySe::from_dir_entry(&sub_entry) {
                                Ok(dir) => {
                                    sub_entries.push(Entry::DIRECTORY(dir));
                                    sub_dir_count += 1;
                                }
                                Err(_) => continue
                            }
                        }
                    }
                }
            }
        }
    
        Ok(DirectorySe {
            name: entry.file_name().to_str().unwrap_or("").to_string(),
            path: path_of_entry.to_str().unwrap_or("").to_string(),
            is_symlink: path_of_entry.is_symlink(),
            access_rights_as_string: get_access_permission_string(metadata.permissions(), false),
            access_rights_as_number: get_access_permission_number(metadata.permissions(), false),
            size_in_bytes: metadata.len(),
            sub_file_count,
            sub_dir_count,
            created: format_system_time(metadata.created()?),
            last_modified: format_system_time(metadata.modified()?),
            accessed: format_system_time(metadata.accessed()?),
            sub_entries,
        })
    }
}