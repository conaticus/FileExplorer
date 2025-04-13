use crate::state::meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::{command, State};

#[command]
pub fn get_meta_data(state: State<Arc<Mutex<MetaDataState>>>) -> String {
    let meta_dat_state = state.lock().unwrap().0.clone();
    serde_json::to_string(&meta_dat_state).unwrap().to_string()
}
