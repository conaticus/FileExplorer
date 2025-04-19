use std::io;
use crate::state::SettingsState;
use std::sync::{Arc, Mutex};
use serde_json::{to_string, Value};
use tauri::State;

/// Retrieves the current application settings as a JSON string.
///
/// This command provides access to the entire settings state, serialized to a JSON string.
///
/// # Arguments
///
/// * `state` - A Tauri state containing a thread-safe reference to the application's settings.
///
/// # Returns
///
/// * A JSON string representation of the current settings.
///
/// # Example
///
/// ```rust
/// let settings_json = get_settings_as_json(state);
/// println!("Current settings: {}", settings_json);
/// ```
#[tauri::command]
pub fn get_settings_as_json(state: State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap().0.clone();
    to_string(&settings_state).unwrap().to_string()
}


/// Updates a specific setting field with a new value.
///
/// This command allows changing a single setting identified by its key.
///
/// # Arguments
///
/// * `state` - A Tauri state containing a thread-safe reference to the application's settings.
/// * `key` - A string representing the setting key to update.
/// * `value` - The new value to assign to the setting.
///
/// # Returns
///
/// * `Ok(String)` - A JSON string representation of the updated settings if successful.
/// * `Err(String)` - An error message if the update operation failed.
///
/// # Example
///
/// ```rust
/// let result = update_settings_field(state, "theme".to_string(), json!("dark"));
/// match result {
///     Ok(updated_settings) => println!("Updated settings: {}", updated_settings),
///     Err(err) => println!("Failed to update setting: {}", err),
/// }
/// ```
#[tauri::command]
pub fn update_settings_field(
    state: State<Arc<Mutex<SettingsState>>>,
    key: String,
    value: Value,
) -> Result<String, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .update_setting_field(&key, value)
        .and_then(|updated| to_string(&updated).map_err(|e| io::Error::new(io::ErrorKind::Other, e)))
        .map_err(|e| e.to_string())
}

/// Retrieves the value of a specific setting field.
///
/// This command allows accessing a single setting value identified by its key.
///
/// # Arguments
///
/// * `state` - A Tauri state containing a thread-safe reference to the application's settings.
/// * `key` - A string representing the setting key to retrieve.
///
/// # Returns
///
/// * `Ok(Value)` - The value of the requested setting if found.
/// * `Err(String)` - An error message if the setting key doesn't exist or another error occurred.
///
/// # Example
///
/// ```rust
/// let result = get_setting_field(state, "theme".to_string());
/// match result {
///     Ok(value) => println!("Theme setting: {}", value),
///     Err(err) => println!("Failed to get setting: {}", err),
/// }
/// ```
#[tauri::command]
pub fn get_setting_field(
    state: State<Arc<Mutex<SettingsState>>>,
    key: String,
) -> Result<Value, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .get_setting_field(&key)
        .map_err(|e| e.to_string())
}

/// Updates multiple settings fields at once.
///
/// This command allows batch updating of multiple settings in a single operation.
///
/// # Arguments
///
/// * `state` - A Tauri state containing a thread-safe reference to the application's settings.
/// * `updates` - A map of setting keys to their new values.
///
/// # Returns
///
/// * `Ok(String)` - A JSON string representation of the updated settings if successful.
/// * `Err(String)` - An error message if the update operation failed.
///
/// # Example
///
/// ```rust
/// let mut updates = serde_json::Map::new();
/// updates.insert("theme".to_string(), json!("dark"));
/// updates.insert("notifications".to_string(), json!(true));
///
/// let result = update_multiple_settings_command(state, updates);
/// match result {
///     Ok(updated_settings) => println!("Updated settings: {}", updated_settings),
///     Err(err) => println!("Failed to update settings: {}", err),
/// }
/// ```
#[tauri::command]
pub fn update_multiple_settings_command(
    state: State<Arc<Mutex<SettingsState>>>,
    updates: serde_json::Map<String, serde_json::Value>,
) -> Result<String, String> {
    let settings_state = state.lock().unwrap();

    settings_state
        .update_multiple_settings(&updates)
        .and_then(|updated| serde_json::to_string(&updated).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
        .map_err(|e| e.to_string())
}
