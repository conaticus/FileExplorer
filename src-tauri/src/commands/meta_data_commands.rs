use crate::state::meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::State;


//TODO implement error handling and update docs
//TODO implement tests
#[tauri::command]
pub fn get_meta_data_as_json(state: State<Arc<Mutex<MetaDataState>>>) -> String {
    let meta_dat_state = state.lock().unwrap().0.clone();
    serde_json::to_string(&meta_dat_state).unwrap().to_string()
}
#[tauri::command]
pub fn update_meta_data(state: State<Arc<Mutex<MetaDataState>>>) -> Result<(), String> {
    match state.lock().unwrap().refresh_volumes() {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

