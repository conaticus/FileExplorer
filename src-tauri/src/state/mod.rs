pub mod meta_data;
pub(crate) mod settings_data;

use meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::{Builder, Wry};
use crate::state::settings_data::SettingsState;

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry> {
    //To add more just .manage 
    app.manage(Arc::new(Mutex::new(MetaDataState::new())))
        .manage(Arc::new(Mutex::new(SettingsState::new())))
}