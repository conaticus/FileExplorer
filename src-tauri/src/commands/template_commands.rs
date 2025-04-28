use crate::log_info;

#[tauri::command]
pub fn get_template_paths() -> Result<String, String> {
    // This function should return a JSON string containing the templates
    // For now, we will return a dummy JSON string
    log_info!("Getting template paths");
    Ok(r#"{"template1", "template2", "template3"}"#.to_string())
}

#[tauri::command]
pub fn add_template(template_path: &str) -> Result<String, String> {
    // This function should add a template and return a success message
    // For now, we will return a dummy success message
    log_info!("Adding template");
    Ok(format!("Template '{}' added successfully", template_path))
}

#[tauri::command]
pub fn use_template(template_path: &str) -> Result<String, String> {
    // This function should apply a template and return a success message
    // For now, we will return a dummy success message
    log_info!("Using template");
    Ok(format!("Template '{}' applied successfully", template_path))
}

#[tauri::command]
pub fn remove_template(template_path: &str) -> Result<String, String> {
    // This function should remove a template and return a success message
    // For now, we will return a dummy success message
    log_info!("Removing template");
    Ok(format!("Template '{}' removed successfully", template_path))
}