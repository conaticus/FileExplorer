use crate::state::SettingsState;
use serde_json::{to_string, Value};
use std::io;
use std::sync::{Arc, Mutex};
use tauri::State;

pub fn get_settings_as_json_impl(state: Arc<Mutex<SettingsState>>) -> String {
    let settings_inner = state.lock().unwrap().0.clone();
    to_string(&settings_inner).unwrap()
}

pub fn get_setting_field_impl(
    state: Arc<Mutex<SettingsState>>,
    key: String,
) -> Result<Value, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .get_setting_field(&key)
        .map_err(|e| e.to_string())
}

pub fn update_settings_field_impl(
    state: Arc<Mutex<SettingsState>>,
    key: String,
    value: Value,
) -> Result<String, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .update_setting_field(&key, value)
        .and_then(|updated| {
            to_string(&updated).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })
        .map_err(|e| e.to_string())
}

pub fn update_multiple_settings_impl(
    state: Arc<Mutex<SettingsState>>,
    updates: serde_json::Map<String, Value>,
) -> Result<String, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .update_multiple_settings(&updates)
        .and_then(|updated| {
            to_string(&updated).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })
        .map_err(|e| e.to_string())
}

pub fn reset_settings_impl(state: Arc<Mutex<SettingsState>>) -> Result<String, String> {
    let settings_state = state.lock().unwrap();
    settings_state
        .reset_settings()
        .and_then(|updated| {
            to_string(&updated).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })
        .map_err(|e| e.to_string())
}

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
    get_settings_as_json_impl(state.inner().clone())
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
    get_setting_field_impl(state.inner().clone(), key)
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
    update_settings_field_impl(state.inner().clone(), key, value)
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
    update_multiple_settings_impl(state.inner().clone(), updates)
}

/// Resets the current settings file and resets settings to their default values.
///
/// reinitializes the in-memory settings state to default values by reusing the default state logic.
///
/// # Arguments
///
/// * `settings_state` - A Tauri state containing a thread-safe reference to the application's settings.
///
/// # Returns
///
/// * `Ok(())` - If the settings file was successfully deleted and the state reset.
/// * `Err(String)` - An error message if deletion or reset fails.
///
/// # Example
///
/// ```rust
/// let result = reset_settings(state);
/// match result {
///     Ok(_) => println!("Settings were reset to default."),
///     Err(err) => println!("Failed to reset settings: {}", err),
/// }
/// ```
#[tauri::command]
pub fn reset_settings_command(
    state: State<Arc<Mutex<SettingsState>>>
) -> Result<String, String>{
    reset_settings_impl(state.inner().clone())
}

#[cfg(test)]
mod tests_settings_commands {
    use std::path::{Path, PathBuf};
    use super::*;
    use serde_json::json;

    // Testing: Helper function to create a test SettingsState
    fn create_test_settings_state() -> Arc<Mutex<SettingsState>> {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create a settings state with a temporary file path
        Arc::new(Mutex::new(SettingsState::new_with_path(path)))
    }

    fn create_test_settings_state_with_temp_file(temp_file: PathBuf) -> Arc<Mutex<SettingsState>> {
        // Create a settings state with a temporary file path
        Arc::new(Mutex::new(SettingsState::new_with_path(temp_file)))
    }

    #[test]
    fn test_get_settings_as_json_contains_default() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();

        let state = create_test_settings_state_with_temp_file(temp_file.path().to_path_buf());
        let json = get_settings_as_json_impl(state);
        assert!(json.contains("\"darkmode\":false"));
        assert!(json.contains("\"logging_level\":\"Full\""));
    }

    #[test]
    fn test_get_setting_field_existing_key() {
        let state = create_test_settings_state();
        let value = get_setting_field_impl(state.clone(), "darkmode".to_string()).unwrap();
        assert_eq!(value, json!(false));
    }

    #[test]
    fn test_get_setting_field_invalid_key() {
        let state = create_test_settings_state();
        let result = get_setting_field_impl(state.clone(), "invalid_key".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_update_settings_field_success() {
        let state = create_test_settings_state();
        let result = update_settings_field_impl(state.clone(), "darkmode".to_string(), json!(true));
        assert!(result.is_ok());

        let updated = get_setting_field_impl(state.clone(), "darkmode".to_string()).unwrap();
        assert_eq!(updated, json!(true));
    }

    #[test]
    fn test_update_settings_field_invalid_key() {
        let state = create_test_settings_state();
        let result =
            update_settings_field_impl(state.clone(), "nonexistent".to_string(), json!(123));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_multiple_settings_success() {
        let state = create_test_settings_state();

        let mut updates = serde_json::Map::new();
        updates.insert("darkmode".to_string(), json!(true));
        updates.insert("default_theme".to_string(), json!("solarized"));

        let result = update_multiple_settings_impl(state.clone(), updates);
        assert!(result.is_ok());

        let darkmode = get_setting_field_impl(state.clone(), "darkmode".to_string()).unwrap();
        let theme = get_setting_field_impl(state.clone(), "default_theme".to_string()).unwrap();

        assert_eq!(darkmode, json!(true));
        assert_eq!(theme, json!("solarized"));
    }

    #[test]
    fn test_update_multiple_settings_with_invalid_key() {
        let state = create_test_settings_state();

        let mut updates = serde_json::Map::new();
        updates.insert("nonexistent".to_string(), json!("oops"));

        let result = update_multiple_settings_impl(state.clone(), updates);
        assert!(result.is_err());
    }

    #[test]
    fn test_reset_settings_command_success() {
        let state = create_test_settings_state();
        // Prefix unused variable with underscore
        let _updated_data = update_settings_field_impl(state.clone(), "darkmode".to_string(), json!(true));

        let result = reset_settings_impl(state.clone());
        assert!(result.is_ok());

        let darkmode = get_setting_field_impl(state.clone(), "darkmode".to_string()).unwrap();
        assert_eq!(darkmode, json!(false));
    }
}

