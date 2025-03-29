use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::filesystem::models;
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

enum EntryTypes {
    Directory(models::Directory),
    File(models::File)
}

fn get_entries_for_directory(directory: PathBuf) -> Vec<EntryTypes> {
    let entries = fs::read_dir(directory).unwrap();
    
    for entry in entries {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        if metadata.is_dir() {
            let name = entry.file_name();
            let path = entry.path();
            let access_rights = metadata.permissions();
            let size_in_bytes = get_directory_size_in_bytes(path.to_str().unwrap());
            let (file_count, sub_dir_count) = count_subfiles_and_directories(path.to_str().unwrap());
            
            let created = metadata.created().unwrap();
            let last_modified = metadata.modified().unwrap();
            let accessed = metadata.accessed().unwrap();
            
            let dir_struct = models::Directory{
                name: name.to_str().unwrap().to_string(),
                path: path.to_str().unwrap().to_string(),
                access_rights_as_string: access_rights_to_string(access_rights.mode()),
                access_rights_as_number: access_rights.mode(),
                size_in_bytes,
                file_count,
                sub_dir_count,
                created: format_system_time(created),
                last_modified: format_system_time(last_modified),
                accessed: format_system_time(accessed),
            };
            println!("{:?}", dir_struct);
        }else { 
            
        }
        //println!("{:?}", entry.metadata().unwrap());
    }
    
    todo!()
}

fn access_rights_to_string(mode: u32) -> String {
    let mut result = String::new();

    // User permissions
    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    // Group permissions
    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    // Others permissions
    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

fn format_system_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn get_directory_size_in_bytes(path: &str) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok) // Ignore errors
        .filter(|entry| entry.path().is_file()) // Only count files
        .map(|entry| fs::metadata(entry.path()).map(|meta| meta.len()).unwrap_or(0)) // Get file sizes
        .sum()
}

//The first is the number of files and the second is the number of directories
fn count_subfiles_and_directories(path: &str) -> (usize, usize) {
    let mut file_count = 0;
    let mut dir_count = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.path().is_file() {
            file_count += 1;
        } else if entry.path().is_dir() {
            dir_count += 1;
        }
    }

    (file_count, dir_count)
}

#[cfg(test)]
mod tests{
    use std::env;
    use crate::filesystem::fs_dir_loader::get_entries_for_directory;

    #[test]
    fn execute(){
        let directory = env::current_dir().unwrap();
        get_entries_for_directory(directory);
    }
}