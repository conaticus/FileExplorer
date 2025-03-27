use crate::constants;
use crate::AppState;
use serde::{Deserialize, Serialize};

use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::filesystem::infos::volume_information::VolumeInformation;
use crate::filesystem::volume_operations;

#[derive(Debug, Deserialize, Serialize)]
pub struct MetaData {
    abs_file_path_buf: PathBuf,
    all_volumes_with_information: Vec<VolumeInformation>,
}
impl Default for MetaData {
    fn default() -> Self {
        MetaData {
            abs_file_path_buf: constants::META_DATA_CONFIG_ABS_PATH.to_path_buf(),
            all_volumes_with_information: volume_operations::get_system_volumes_information(),
        }
    }
}

pub struct MetaDataState(pub Arc<Mutex<MetaData>>);
impl MetaDataState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(
            Self::load_from_file_if_exists_or_default(),
        )))
    }

    fn load_from_file_if_exists_or_default() -> MetaData {
        let user_config_file_path = &*crate::constants::META_DATA_CONFIG_ABS_PATH;
        let exists = constants::META_DATA_CONFIG_ABS_PATH.clone().exists();

        if exists {
            let mut opened_file =
                File::open(&user_config_file_path).expect("Could not open user config file");
            let mut content = String::new();
            opened_file
                .read_to_string(&mut content)
                .expect("Could not read user config file to string");
            return serde_json::from_str::<MetaData>(&content).expect("Could not deserialize user config");
        }
        Self::write_defaults(user_config_file_path)
    }

    fn write_defaults(file_path: &PathBuf) -> MetaData {
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
