//! # Application State Management
//!
//! This module handles the application state through Tauri's state management system.
//! States defined here are automatically autowired and managed by Tauri's dependency
//! injection system, making them available throughout the application.
//!
//! ## How it works
//!
//! 1. State structs are defined in submodules (e.g., `meta_data`, `settings_data`)
//! 2. The `setup_app_state` function registers these states with Tauri
//! 3. States are wrapped in `Arc<Mutex<T>>` to allow safe concurrent access
//! 4. Tauri's `.manage()` function is used to register states with the application
//!
//! ## Adding a new state
//!
//! To add a new state:
//! 1. Create a new module with your state struct
//! 2. Add it to the imports in this file
//! 3. Add it to the `setup_app_state` function using `.manage(Arc::new(Mutex::new(YourState::new())))`
//!
//! States can then be accessed in command handlers using the `#[tauri::command]` macro
//! and appropriate state parameters.

pub mod meta_data;
pub mod searchengine_data;
pub mod settings_data;
pub mod logging;

pub use settings_data::*;

use logging::Logger;
use crate::state::searchengine_data::SearchEngineState;
use meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::{Builder, Wry};

pub fn setup_app_state(app: Builder<Wry>) -> Builder<Wry> {
    // Create our shared state instances
    let meta_data_state = Arc::new(Mutex::new(MetaDataState::new()));
    let settings_state = Arc::new(Mutex::new(SettingsState::new()));
    let search_engine_state = Arc::new(Mutex::new(SearchEngineState::new(settings_state.clone())));
    
    // Initialize the logger with the settings state
    Logger::init(settings_state.clone());
    
    //To add more just .manage
    app.manage(meta_data_state)
        .manage(settings_state)
        .manage(search_engine_state)
}
