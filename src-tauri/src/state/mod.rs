pub mod meta_data;
use meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::{Manager, State, App, Builder, Wry};
use crate::AppState;

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry>{
    return app.manage(Arc::new(Mutex::new(MetaDataState::new())));
}