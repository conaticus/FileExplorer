use crate::file_explorer::DirectoryChild;

#[tauri::command]
pub fn search_directory(query: String, search_directory: String, extension: String, accept_files: bool, accept_directories: bool) -> Vec<DirectoryChild> {
    return walk(query, search_directory, extension, accept_files, accept_directories);
}

fn walk(query: String, directory: String, extension: String, accept_files: bool, accept_directories: bool) -> Vec<DirectoryChild> {
    Vec::new()
}