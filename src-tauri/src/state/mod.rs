pub mod meta_data;
mod selected_file_for_action;
use crate::AppState;
use meta_data::MetaDataState;
pub use selected_file_for_action::SelectedFileForAction;
use std::sync::{Arc, Mutex};
use tauri::{App, Builder, Manager, State, Wry};

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry> {
    app.manage(Mutex::new(SelectedFileForAction::default()))
        .manage(Arc::new(Mutex::new(MetaDataState::new())))
}
