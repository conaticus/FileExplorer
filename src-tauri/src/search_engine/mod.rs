mod models;

use std::path::PathBuf;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::read_dir;
use crate::models::{format_system_time, get_access_permission_number, get_access_permission_string, Directory, File};
use models::file_se::File_SE;
use crate::search_engine::models::directory_se::Directory_SE;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Entry {
    FILE(File_SE),
    DIRECTORY(Directory_SE),
}

pub fn start_indexing_home_dir() -> Vec<Entry> {
    let home_dir = home_dir().unwrap_or_default();
    index_given_path(home_dir)
}

fn index_given_path(path: PathBuf) -> Vec<Entry> {
    let mut result: Vec<Entry> = Vec::new();
    
    // Return empty vector if we can't read the directory
    let read_dir_result = match read_dir(&path) {
        Ok(dir) => dir,
        Err(_) => return result,
    };
    
    // Process each entry in the directory
    for entry_result in read_dir_result {
        // Skip entries that we can't access
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        
        // Skip entries that we can't get metadata for
        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        
        if metadata.is_file() {
            // Process files
            match File_SE::from_dir_entry(entry) {
                Ok(file) => result.push(Entry::FILE(file)),
                Err(_) => continue,
            }
        } else if metadata.is_dir() {
            // Process directories
            match Directory_SE::from_dir_entry(entry) {
                Ok(dir) => result.push(Entry::DIRECTORY(dir)),
                Err(_) => continue,
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn start_indexing_home_dir_test() {
        let entries = start_indexing_home_dir();
        assert!(!entries.is_empty(), "Home directory should contain at least some entries");
        
        let result = start_indexing_home_dir();
        
        //write the result to a file as json 
        let json_result = serde_json::to_string_pretty(&result).unwrap();
        let path = PathBuf::from("home_dir_index.json");
        std::fs::write(&path, json_result).expect("Unable to write file");
        println!("Home directory indexed and saved to {:?}", path);
        
     
    }
}
