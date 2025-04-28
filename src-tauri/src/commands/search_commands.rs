//TODO
#[tauri::command]
pub async fn _search_command(snipped: String) -> Result<String, String> {
    // Simulate a search operation
    if snipped.is_empty() {
        return Err("Search term cannot be empty".to_string());
    }
    
    // Here you would typically perform the search operation
    // For demonstration, we will just return a success message
    Ok(format!("Search results for: {}", snipped))
}