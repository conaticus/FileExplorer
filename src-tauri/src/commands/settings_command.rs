use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;
use crate::state::settings_data::{SettingsState, download_location};


#[tauri::command]
pub fn get_settings_as_json(state: State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap().0.clone();
    serde_json::to_string(&settings_state).unwrap().to_string()
}

#[tauri::command]
pub fn set_darkmode(state: tauri::State<Arc<Mutex<SettingsState>>>, enabled: bool) {
    let settings_state = state.lock().unwrap();
    let mut settings = settings_state.0.lock().unwrap();
    settings.set_darkmode(enabled);
}

#[tauri::command]
pub fn get_darkmode(state: tauri::State<Arc<Mutex<SettingsState>>>) -> bool {
    let settings_state = state.lock().unwrap();
    let settings = settings_state.0.lock().unwrap();
    settings.darkmode()
}

#[tauri::command]
pub fn set_default_theme(state: tauri::State<Arc<Mutex<SettingsState>>>, theme: String) {
    let settings_state = state.lock().unwrap();
    let mut settings = settings_state.0.lock().unwrap();
    settings.set_default_theme(theme);
}

#[tauri::command]
pub fn get_default_theme(state: tauri::State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap();
    let settings = settings_state.0.lock().unwrap();
    settings.default_theme().clone()
}

#[tauri::command]
pub fn set_folder_path_on_open(state: tauri::State<Arc<Mutex<SettingsState>>>, path: String) {
    let settings_state = state.lock().unwrap();
    let mut settings = settings_state.0.lock().unwrap();
    settings.set_default_folder_path_on_opening(PathBuf::from(path));
}

#[tauri::command]
pub fn get_folder_path_on_open(state: tauri::State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap();
    let settings = settings_state.0.lock().unwrap();
    settings.default_folder_path_on_opening().to_str().unwrap().to_string()
}

#[tauri::command]
pub fn add_download_location(
    state: tauri::State<Arc<Mutex<SettingsState>>>,
    name: String,
    url: String,
    path: String
) {
    let settings_state = state.lock().unwrap();
    let mut settings = settings_state.0.lock().unwrap();
    let new_location = download_location::DefaultDownloadLocation {
        name,
        url,
        path: PathBuf::from(path),
    };
    settings.add_download_location(new_location);
}

#[tauri::command]
pub fn get_download_locations(state: tauri::State<Arc<Mutex<SettingsState>>>) -> Vec<download_location::DefaultDownloadLocation> {
    let settings_state = state.lock().unwrap();
    let settings = settings_state.0.lock().unwrap();
    settings.default_download_location().clone()
}