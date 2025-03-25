use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::AppState;
use crate::constants;

pub struct MetaData {
    abs_file_path_buf: PathBuf
}
impl Default for MetaData {
    fn default() -> Self {
        MetaData { abs_file_path_buf: constants::META_DATA_CONFIG_ABS_PATH.to_path_buf() }
    }
}

pub struct MetaDataState (pub Arc<Mutex<MetaData>>);
impl MetaDataState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Self::load_from_file_if_exists_or_default())))
    }

    fn load_from_file_if_exists_or_default() -> MetaData {
        let _ = *constants::META_DATA_CONFIG_ABS_PATH;
        let mut path = env::current_dir().expect("could not determine current path");
        let user_config_file = &*crate::constants::META_DATA_CONFIG_ABS_PATH;
        let exists = constants::META_DATA_CONFIG_ABS_PATH.clone().exists();

        match exists {
            true => {}
            false => {
                let file = File::create(user_config_file).expect("could not create config file");
                let meta_data = MetaData { abs_file_path_buf: path };
                Self::write_defaults(&file, meta_data);
                
            }
        }

    }

    fn write_defaults(file: &File, meta_data: MetaData) {
        todo!()
    }
}



#[cfg(test)]
mod tests {
    use crate::state::meta_data::MetaDataState;

    #[test]
    fn test_meta_data() {
        let meta_data_state = MetaDataState::new();
    }
}