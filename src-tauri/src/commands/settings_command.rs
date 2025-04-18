use std::io;
use crate::state::SettingsState;
use std::sync::{Arc, Mutex};
use serde_json::{to_string, Value};
use tauri::State;

#[tauri::command]
pub fn get_settings_as_json(state: State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap().0.clone();
    to_string(&settings_state).unwrap().to_string()
}

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

#[tauri::command]
pub fn get_setting_field(
    settings_state: tauri::State<'_, SettingsState>,
    key: String,
) -> Result<serde_json::Value, String> {
    settings_state
        .get_setting_field(&key)
        .map_err(|e| e.to_string())
}
