use crate::constants;
use serde::{Deserialize, Serialize};

use crate::filesystem::models::VolumeInformation;
use crate::commands::volume_operations_commands;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaData {
    version: String,
    abs_file_path_buf: PathBuf,
    all_volumes_with_information: Vec<VolumeInformation>,
}
impl Default for MetaData {
    fn default() -> Self {
        MetaData {
            version: constants::VERSION.to_owned(),
            abs_file_path_buf: constants::META_DATA_CONFIG_ABS_PATH.to_path_buf(),
            all_volumes_with_information: volume_operations_commands::get_system_volumes_information(),
        }
    }
}

pub struct MetaDataState(pub Arc<Mutex<MetaData>>);
impl MetaDataState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Self::write_default_meta_data_to_file_and_save_in_state())))
    }

    /// Updates the volume information in the metadata
    pub fn refresh_volumes(&self) -> io::Result<()> {
        let mut meta_data = self.0.lock().unwrap();
        meta_data.all_volumes_with_information = volume_operations_commands::get_system_volumes_information();
        self.write_meta_data_to_file(&meta_data)
    }

    /// Writes the current metadata to file
    fn write_meta_data_to_file(&self, meta_data: &MetaData) -> io::Result<()> {
        let user_config_file_path = &*constants::META_DATA_CONFIG_ABS_PATH;
        let serialized = serde_json::to_string_pretty(&meta_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Makes sure the parent directory exists
        if let Some(parent) = user_config_file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to the file
        let mut file = File::create(user_config_file_path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    fn write_default_meta_data_to_file_and_save_in_state() -> MetaData {
        let defaults = MetaData::default();
        let meta_data_state = Self(Arc::new(Mutex::new(defaults.clone())));
        
        if let Err(e) = meta_data_state.write_meta_data_to_file(&defaults) {
            eprintln!("Error writing metadata to file: {}", e);
        }
        
        defaults
    }
}

#[cfg(test)]
mod tests {
    use crate::state::meta_data::MetaDataState;

    #[test]
    fn test_meta_data_creation_and_leave_files() {
        let _meta_data_state = MetaDataState::new();
    }
}
