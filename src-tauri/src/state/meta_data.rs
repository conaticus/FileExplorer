use crate::constants;
use crate::AppState;
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::filesystem::models::VolumeInformation;
use crate::filesystem::volume_operations;

#[derive(Debug, Deserialize, Serialize)]
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
            all_volumes_with_information: volume_operations::get_system_volumes_information(),
        }
    }
}

pub struct MetaDataState(pub Arc<Mutex<MetaData>>);
impl MetaDataState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(
            Self::load_and_store_meta_data(),
        )))
    }

    fn load_and_store_meta_data() -> MetaData {
        let user_config_file_path = &*crate::constants::META_DATA_CONFIG_ABS_PATH;
        
        Self::write_meta_data_to_file(user_config_file_path)
    }

    fn write_meta_data_to_file(file_path: &PathBuf) -> MetaData {
        let defaults = MetaData::default();
        let serialized = serde_json::to_string_pretty(&defaults).unwrap();

        //makes sure the parent dire exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Could not create parent directories for config file");
        }
        
        //write to the file
        let mut file = File::create(file_path).expect("Could not create file");
        file.write_all(serialized.as_bytes()).expect("Could not write to file");
        
        defaults
    }
}

#[cfg(test)]
mod tests {
    use crate::state::meta_data::MetaDataState;

    #[test]
    fn test_meta_data_creation_and_leave_files() {
        let meta_data_state = MetaDataState::new();
    }
}
