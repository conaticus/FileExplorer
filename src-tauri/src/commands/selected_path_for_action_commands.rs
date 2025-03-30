use std::sync::Mutex;
use tauri::{command, State};
use crate::state::SelectedPathForAction;
#[command]
fn set_selected_path_for_action(state: State<Mutex<SelectedPathForAction>>, path: String) {
    let mut selected_file = state.lock().unwrap();
    selected_file.abs_file_path = Some(path.parse().expect("Failed to parse path"));
    selected_file.time_of_selection = Some(chrono::Utc::now().timestamp() as u64);
}

#[command]
fn get_selected_path_for_action(state: State<Mutex<SelectedPathForAction>>) -> String {
    let selected_file = state.lock().unwrap();
    if let Some(path) = &selected_file.abs_file_path {
        path.to_string_lossy().to_string()
    } else {
        "<no path selected>".to_string()
    }
}