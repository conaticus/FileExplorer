use crate::file_explorer::DirectoryChild;

#[tauri::command]
fn search_directory(query: String, search_directory: String) -> Vec<DirectoryChild> {
    return walk(query, search_directory);
}

fn walk(query: String, directory: String) -> Vec<DirectoryChild> {
    Vec::new()
}