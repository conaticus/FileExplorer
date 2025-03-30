pub mod meta_data;
mod selected_path_for_action;
use crate::AppState;
use meta_data::MetaDataState;
pub use selected_path_for_action::SelectedPathForAction;
use std::sync::{Arc, Mutex};
use tauri::{App, Builder, Manager, State, Wry};

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry> {
    app.manage(Mutex::new(SelectedPathForAction::default()))
        .manage(Arc::new(Mutex::new(MetaDataState::new())))
}
