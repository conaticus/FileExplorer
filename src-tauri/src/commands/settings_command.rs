use crate::state::SettingsState;
use std::sync::{Arc, Mutex};
use tauri::State;

#[tauri::command]
pub fn get_settings_as_json(state: State<Arc<Mutex<SettingsState>>>) -> String {
    let settings_state = state.lock().unwrap().0.clone();
    serde_json::to_string(&settings_state).unwrap().to_string()
}

//TODO implement a method which takes a string as an argument of what should be changed in the json file. 
// Then as a second parameter of type T the value for the specific field. Update the settings and return 
// the new ones
