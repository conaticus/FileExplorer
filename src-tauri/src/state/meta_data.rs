use crate::commands::volume_operations_commands;
use crate::models::VolumeInformation;
use crate::{constants, log_error};
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, io};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaData {
    version: String,
    abs_file_path_buf: PathBuf,
    abs_file_path_for_settings_json: PathBuf,
    pub abs_folder_path_buf_for_templates: PathBuf,
    pub template_paths: Vec<PathBuf>,
    all_volumes_with_information: Vec<VolumeInformation>,
    current_running_os: String,
    current_cpu_architecture: String,
    user_home_dir: String,
}
impl Default for MetaData {
    fn default() -> Self {
        MetaData {
            version: constants::VERSION.to_owned(),
            abs_file_path_buf: constants::META_DATA_CONFIG_ABS_PATH.to_path_buf(),
            abs_file_path_for_settings_json: constants::SETTINGS_CONFIG_ABS_PATH.to_path_buf(),
            abs_folder_path_buf_for_templates: constants::TEMPLATES_ABS_PATH_FOLDER.to_path_buf(),
            template_paths: load_templates(),
            all_volumes_with_information:
                volume_operations_commands::get_system_volumes_information(),
            current_running_os: std::env::consts::OS.to_string(),
            current_cpu_architecture: std::env::consts::ARCH.to_string(),
            user_home_dir: home_dir()
                .unwrap_or(PathBuf::from(""))
                .to_string_lossy()
                .to_string(),
        }
    }
}

fn load_templates() -> Vec<PathBuf> {
    let templates_path = constants::TEMPLATES_ABS_PATH_FOLDER.to_path_buf();
    if templates_path.exists() {
        std::fs::read_dir(templates_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect()
    } else {
        //create the empty folder
        fs::create_dir_all(templates_path)
            .map_err(|e| {
                log_error!(format!("Failed to create templates folder. Error: {}", e).as_str());
            })
            .unwrap();
        vec![]
    }
}

pub struct MetaDataState(pub Arc<Mutex<MetaData>>);
impl MetaDataState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(
            Self::write_default_meta_data_to_file_and_save_in_state(),
        )))
    }

    // For testing - allows creating a MetaDataState with a custom path
    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        let mut defaults = MetaData::default();
        defaults.abs_file_path_buf = path;
        Self(Arc::new(Mutex::new(
            Self::write_meta_data_to_file_and_save_in_state(defaults),
        )))
    }

    /// Updates the volume information in the metadata
    pub fn refresh_volumes(&self) -> io::Result<()> {
        let mut meta_data = self.0.lock().unwrap();
        meta_data.all_volumes_with_information =
            volume_operations_commands::get_system_volumes_information();
        self.write_meta_data_to_file(&meta_data)
    }

    pub fn update_template_paths(&self) -> io::Result<()> {
        let mut meta_data = self.0.lock().unwrap();
        meta_data.template_paths = load_templates();
        self.write_meta_data_to_file(&meta_data)
    }

    /// Writes the current metadata to file
    pub fn write_meta_data_to_file(&self, meta_data: &MetaData) -> io::Result<()> {
        let user_config_file_path = &meta_data.abs_file_path_buf;
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
        Self::write_meta_data_to_file_and_save_in_state(defaults)
    }

    // Helper method to write metadata to a file
    fn write_meta_data_to_file_and_save_in_state(defaults: MetaData) -> MetaData {
        let meta_data_state = Self(Arc::new(Mutex::new(defaults.clone())));

        if let Err(e) = meta_data_state.write_meta_data_to_file(&defaults) {
            eprintln!("Error writing metadata to file: {}", e);
        }

        defaults
    }

    // For testing - read metadata from file
    #[cfg(test)]
    pub fn read_meta_data_from_file(path: &PathBuf) -> io::Result<MetaData> {
        use std::io::Read;
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use tempfile::tempdir;

    //test the default values of the metadata
    #[test]
    fn test_default_meta_data() {
        let meta_data = MetaData::default();
        assert_eq!(meta_data.version, constants::VERSION);
        assert_eq!(
            meta_data.abs_file_path_buf,
            constants::META_DATA_CONFIG_ABS_PATH.to_path_buf()
        );
        // Check the OS and architecture fields
        assert_eq!(
            meta_data.current_running_os,
            std::env::consts::OS.to_string()
        );
        assert_eq!(
            meta_data.current_cpu_architecture,
            std::env::consts::ARCH.to_string()
        );
        // Check the home directory field
        assert_eq!(
            meta_data.user_home_dir,
            home_dir()
                .unwrap_or(PathBuf::from(""))
                .to_string_lossy()
                .to_string()
        );
        // Cannot test volume information directly as it depends on the system
    }

    #[test]
    fn test_meta_data_state_creation() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("meta_data.json");

        // Create a new MetaDataState with our test path
        let _meta_data_state = MetaDataState::new_with_path(test_path.clone());

        // Verify the file was created
        assert!(
            test_path.exists(),
            "Metadata file should exist after creation"
        );

        // Read the file and verify its contents
        let read_result = MetaDataState::read_meta_data_from_file(&test_path);
        assert!(read_result.is_ok(), "Should be able to read metadata file");

        let meta_data = read_result.unwrap();
        assert_eq!(meta_data.version, constants::VERSION);
        assert_eq!(meta_data.abs_file_path_buf, test_path);
        assert_eq!(
            meta_data.abs_file_path_for_settings_json,
            constants::SETTINGS_CONFIG_ABS_PATH.to_path_buf()
        );
        // Check the OS and architecture fields
        assert_eq!(
            meta_data.current_running_os,
            std::env::consts::OS.to_string()
        );
        assert_eq!(
            meta_data.current_cpu_architecture,
            std::env::consts::ARCH.to_string()
        );
        // Check the home directory field
        assert_eq!(
            meta_data.user_home_dir,
            home_dir()
                .unwrap_or(PathBuf::from(""))
                .to_string_lossy()
                .to_string()
        );
    }

    #[test]
    fn test_refresh_volumes() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("meta_data.json");

        // Create a new MetaDataState
        let meta_data_state = MetaDataState::new_with_path(test_path.clone());

        // Get the initial volumes count
        let initial_volumes = {
            let meta_data = meta_data_state.0.lock().unwrap();
            meta_data.all_volumes_with_information.len()
        };

        // Refresh volumes
        let refresh_result = meta_data_state.refresh_volumes();
        assert!(refresh_result.is_ok(), "Volume refresh should succeed");

        // Verify the file still exists and can be read
        assert!(
            test_path.exists(),
            "Metadata file should exist after refresh"
        );

        // Get the volumes after refresh
        let refreshed_volumes = {
            let meta_data = meta_data_state.0.lock().unwrap();
            meta_data.all_volumes_with_information.len()
        };

        // The number of volumes should be the same after refresh since we're on the same system
        assert_eq!(
            initial_volumes, refreshed_volumes,
            "Volume count should remain the same after refresh"
        );
    }

    #[test]
    fn test_write_meta_data_to_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("meta_data.json");

        // Create a custom metadata object
        let mut meta_data = MetaData::default();
        meta_data.abs_file_path_buf = test_path.clone();
        meta_data.version = "test-version".to_string();

        // Customize the OS, CPU, and home directory fields for testing
        let test_os = "test-os".to_string();
        let test_arch = "test-arch".to_string();
        let test_home = "/test/home/path".to_string();

        meta_data.current_running_os = test_os.clone();
        meta_data.current_cpu_architecture = test_arch.clone();
        meta_data.user_home_dir = test_home.clone();

        // Create a MetaDataState and write the custom metadata
        // Construct a MetaDataState with the custom metadata (is the struct from above)
        let meta_data_state = MetaDataState(Arc::new(Mutex::new(meta_data.clone())));
        let write_result = meta_data_state.write_meta_data_to_file(&meta_data);
        assert!(write_result.is_ok(), "Writing metadata should succeed");

        // Read back the file and verify contents
        let read_result = MetaDataState::read_meta_data_from_file(&test_path);
        assert!(read_result.is_ok(), "Should be able to read metadata file");

        let read_meta_data = read_result.unwrap();
        assert_eq!(read_meta_data.version, "test-version");
        assert_eq!(read_meta_data.abs_file_path_buf, test_path);
        assert_eq!(
            meta_data.abs_file_path_for_settings_json,
            constants::SETTINGS_CONFIG_ABS_PATH.to_path_buf()
        );

        // Verify the custom OS, CPU, and home directory fields
        assert_eq!(read_meta_data.current_running_os, test_os);
        assert_eq!(read_meta_data.current_cpu_architecture, test_arch);
        assert_eq!(read_meta_data.user_home_dir, test_home);
    }
}
