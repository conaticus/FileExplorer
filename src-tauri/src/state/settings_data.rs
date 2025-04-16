use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::filesystem::models::LoggingState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub darkmode: bool,
    pub custom_themes: Vec<String>,
    pub default_theme: String,
    pub default_themes_path: PathBuf,
    pub default_folder_path_on_opening: PathBuf,
    pub default_checksum_hash: String,
    pub logging_state: LoggingState,
    pub abs_file_path_buf: PathBuf,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            darkmode: false,
            custom_themes: vec![],
            default_theme: "".to_string(),
            default_themes_path: Default::default(),
            default_folder_path_on_opening: Default::default(),
            default_checksum_hash: "".to_string(),
            logging_state: LoggingState::Full,
            abs_file_path_buf: Default::default(),
        }
    }
}

pub struct SettingsState(pub Arc<Mutex<Settings>>);


impl SettingsState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Self::write_default_settings_to_file_and_save_in_state())))
    }

    pub fn update_setting_field(
        &self,
        key: &str,
        value: Value,
    ) -> Result<Settings, io::Error> {
        let mut settings = self.0.lock().unwrap();

        match key {
            "darkmode" => {
                settings.darkmode = value.as_bool().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Expected a boolean for 'darkmode'",
                ))?;
            }
            "custom_themes" => {
                settings.custom_themes = serde_json::from_value::<Vec<String>>(value)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            }
            "default_theme" => {
                settings.default_theme = value.as_str().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Expected a string for 'default_theme'",
                ))?.to_string();
            }
            "default_themes_path" => {
                settings.default_themes_path = PathBuf::from(
                    value.as_str().ok_or_else(|| io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Expected a string for 'default_themes_path'",
                    ))?,
                );
            }
            "default_folder_path_on_opening" => {
                settings.default_folder_path_on_opening = PathBuf::from(
                    value.as_str().ok_or_else(|| io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Expected a string for 'default_folder_path_on_opening'",
                    ))?,
                );
            }
            "default_checksum_hash" => {
                settings.default_checksum_hash = value.as_str().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Expected a string for 'default_checksum_hash'",
                ))?.to_string();
            }
            "logging_state" => {
                settings.logging_state = serde_json::from_value::<LoggingState>(value)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
            }
            "abs_file_path_buf" => {
                settings.abs_file_path_buf = PathBuf::from(
                    value.as_str().ok_or_else(|| io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Expected a string for 'abs_file_path_buf'",
                    ))?,
                );
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown settings key: {}", key),
                ));
            }
        }

        self.write_settings_to_file(&settings)?;
        Ok(settings.clone())
    }
    // For testing - allows creating a SettingsState with a custom path
    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        let mut defaults = Settings::default();
        defaults.abs_file_path_buf = path;
        Self(Arc::new(Mutex::new(Self::write_settings_to_file_and_save_in_state(defaults))))
    }

    /// Writes the current settings to file
    fn write_settings_to_file(&self, settings: &Settings) -> io::Result<()> {
        let user_config_file_path = &settings.abs_file_path_buf;
        let serialized = serde_json::to_string_pretty(&settings)
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

    fn write_default_settings_to_file_and_save_in_state() -> Settings {
        let defaults = Settings::default();
        Self::write_settings_to_file_and_save_in_state(defaults)
    }

    // Helper method to write settings to a file
    fn write_settings_to_file_and_save_in_state(defaults: Settings) -> Settings {
        let settings_state = Self(Arc::new(Mutex::new(defaults.clone())));

        if let Err(e) = settings_state.write_settings_to_file(&defaults) {
            eprintln!("Error writing settings to file: {}", e);
        }

        defaults
    }

    // For testing - read settings from file
    #[cfg(test)]
    pub fn read_settings_from_file(path: &PathBuf) -> io::Result<Settings> {
        use std::io::Read;
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    #[cfg(test)]
    fn test_update_darkmode_field() {
        let state = Self::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("darkmode", json!(true));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().darkmode, true);
    }

    #[cfg(test)]
    fn test_update_default_theme_field() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("default_theme", json!("ocean"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().default_theme, "ocean");
    }

    #[cfg(test)]
    fn test_update_default_checksum_hash_field() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("default_checksum_hash", json!("abc123"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().default_checksum_hash, "abc123");
    }

    #[cfg(test)]
    fn test_update_custom_themes_field() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let themes = vec!["dark".to_string(), "light".to_string()];
        let result = state.update_setting_field("custom_themes", json!(themes.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().custom_themes, themes);
    }

    #[cfg(test)]
    fn test_update_path_fields() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let path = "/some/path";
        let result1 = state.update_setting_field("default_themes_path", json!(path));
        let result2 = state.update_setting_field("default_folder_path_on_opening", json!(path));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap().default_themes_path, PathBuf::from(path));
        assert_eq!(result2.unwrap().default_folder_path_on_opening, PathBuf::from(path));
    }

    #[cfg(test)]
    fn test_update_logging_state_field() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("logging_state", json!("Minimal"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().logging_state, LoggingState::Minimal);
    }

    #[cfg(test)]
    fn test_invalid_key() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("non_existing_key", json!("value"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown settings key"));
    }

    #[cfg(test)]
    fn test_invalid_type_for_darkmode() {
        let state = SettingsState::new_with_path(tempfile::NamedTempFile::new().unwrap().path().to_path_buf());

        let result = state.update_setting_field("darkmode", json!("not_a_bool"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected a boolean"));
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    //test the default values of the settings
    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.darkmode, false);
        //assert_eq!(settings.custom_themes, vec![]);
        assert_eq!(settings.default_theme, "".to_string());
        //assert_eq!(settings.default_themes_path, Default::default());
        //assert_eq!(settings.default_folder_path_on_opening, Default::default());
        assert_eq!(settings.default_checksum_hash, "".to_string());
        assert_eq!(settings.logging_state, LoggingState::Full);
        //assert_eq!(settings.abs_file_path_buf, Default::default());
    }

    #[test]
    fn test_settings_state_creation() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("settings.json");

        // Create a new Settings with our test path
        let _settings_state = SettingsState::new_with_path(test_path.clone());

        // Verify the file was created
        assert!(test_path.exists(), "Settings file should exist after creation");

        // Read the file and verify its contents
        let read_result = SettingsState::read_settings_from_file(&test_path);
        assert!(read_result.is_ok(), "Should be able to read settings file");

        let settings = read_result.unwrap();
        assert_eq!(settings.darkmode, false);
        assert_eq!(settings.default_theme, "".to_string());
        //assert_eq!(settings.default_themes_path, Default::default());
        //assert_eq!(settings.default_folder_path_on_opening, Default::default());
        assert_eq!(settings.abs_file_path_buf, test_path);
    }

    #[test]
    fn test_write_settings_to_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("settings.json");

        // Create a custom metadata object
        let mut settings = Settings::default();
        settings.abs_file_path_buf = test_path.clone();
        settings.logging_state = LoggingState::Partial;
        settings.default_folder_path_on_opening = PathBuf::from("temp_dir");

        // Create a MetaDataState and write the custom metadata
        // Construct a MetaDataState with the custom metadata (is the struct from above)
        let settings_state = SettingsState(Arc::new(Mutex::new(settings.clone())));
        let write_result = settings_state.write_settings_to_file(&settings);
        assert!(write_result.is_ok(), "Writing settings should succeed");

        // Read back the file and verify contents
        let read_result = SettingsState::read_settings_from_file(&test_path);
        assert!(read_result.is_ok(), "Should be able to read metadata file");

        let read_settings = read_result.unwrap();
        assert_eq!(read_settings.default_folder_path_on_opening, PathBuf::from("temp_dir"));
    }
}
