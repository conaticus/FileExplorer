use crate::{log_critical, log_info};

log_critical!("Running commands:");

#[tauri::command]
pub fn get_templates() -> String {
    // This function should return a JSON string containing the templates
    // For now, we will return a dummy JSON string
    r#"{"templates": ["template1", "template2", "template3"]}"#.to_string()
}

#[tauri::command]
pub fn add_template(template: String) -> String {
    // This function should add a template and return a success message
    // For now, we will return a dummy success message
    format!("Template '{}' added successfully", template)
}

#[tauri::command]
pub fn remove_template(template: String) -> String {
    // This function should remove a template and return a success message
    // For now, we will return a dummy success message
    format!("Template '{}' removed successfully", template)
}