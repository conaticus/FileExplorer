pub mod meta_data;
use meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::{Builder, Wry};

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry> {
    //To add more just .manage 
    app.manage(Arc::new(Mutex::new(MetaDataState::new())))
}
