use std::sync::Mutex;
use tauri::{command, State};
use crate::state::SelectedFileForAction;
#[command]
fn set_selected_file_for_action(state: State<Mutex<SelectedFileForAction>>, file_path: String) {
    let mut selected_file = state.lock().unwrap();
    selected_file.abs_file_path = Some(file_path.parse().expect("Failed to parse path"));
    selected_file.time_of_selection = Some(chrono::Utc::now().timestamp() as u64);
}

#[command]
fn get_selected_file_for_action(state: State<Mutex<SelectedFileForAction>>) -> String {
    let selected_file = state.lock().unwrap();
    if let Some(path) = &selected_file.abs_file_path {
        path.to_string_lossy().to_string()
    } else {
        "<no file selected>".to_string()
    }
}