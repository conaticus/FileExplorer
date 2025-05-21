use crate::{constants, log_error};
use crate::models::LoggingLevel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io;
use std::io::{Error, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::commands::hash_commands::ChecksumMethod;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DefaultView {
    Grid,
    List,
    Details,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum FontSize {
    Small,
    Medium,
    Large,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SortDirection {
    Acscending,
    Descending,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum SortBy {
    Name,
    Size,
    Date,
    Type,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DoubleClick {
    OpenFilesAndFolders,
    SelectFilesAndFolders,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub darkmode: bool,
    pub custom_themes: Vec<String>,
    pub default_theme: String,
    pub default_themes_path: PathBuf,
    pub default_folder_path_on_opening: PathBuf,
    pub default_checksum_hash: ChecksumMethod,
    pub logging_level: LoggingLevel,
    pub default_view: DefaultView,
    pub font_size: FontSize,
    pub show_hidden_files_and_folders: bool,
    pub show_details_panel: bool,
    pub accent_color: String,
    pub confirm_delete: bool,
    pub auto_refresh_dir: bool,
    pub sort_direction: SortDirection,
    pub sort_by: SortBy,
    pub double_click: DoubleClick,
    pub show_file_extensions: bool,
    pub terminal_height: u32,
    pub enable_animations_and_transitions: bool,
    pub enable_virtual_scroll_for_large_directories: bool,
    pub abs_file_path_buf: PathBuf,
}

//TODO implement the default settings -> talk to Lauritz for further more information
impl Default for Settings {
    fn default() -> Self {
        Settings {
            darkmode: false,
            custom_themes: vec![],
            default_theme: "".to_string(),
            default_themes_path: Default::default(),
            default_folder_path_on_opening: Default::default(),
            default_checksum_hash: ChecksumMethod::SHA256,
            logging_level: LoggingLevel::Full,
            abs_file_path_buf: constants::SETTINGS_CONFIG_ABS_PATH.to_path_buf(),
            default_view: DefaultView::Grid,
            font_size: FontSize::Medium,
            show_hidden_files_and_folders: false,
            show_details_panel: false,
            accent_color: "#000000".to_string(),
            confirm_delete: true,
            auto_refresh_dir: true,
            sort_direction: SortDirection::Acscending,
            sort_by: SortBy::Name,
            double_click: DoubleClick::OpenFilesAndFolders,
            show_file_extensions: true,
            terminal_height: 240,
            enable_animations_and_transitions: true,
            enable_virtual_scroll_for_large_directories: false,
        }
    }
}

pub struct SettingsState(pub Arc<Mutex<Settings>>);

impl SettingsState {
    pub fn new() -> Self {
        let path = Settings::default().abs_file_path_buf.to_path_buf();

        let settings = if path.exists() {
            match Self::read_settings_from_file(&path) {
                Ok(s) => s,
                Err(_) => Self::write_default_settings_to_file_and_save_in_state()
            }
        } else {
            Self::write_default_settings_to_file_and_save_in_state()
        };
        Self(Arc::new(Mutex::new(settings)))
    }

    /// Converts a Settings struct to a JSON map representation.
    ///
    /// This function serializes the settings object into a serde_json Map structure
    /// for easier manipulation of individual fields.
    ///
    /// # Arguments
    ///
    /// * `settings` - A reference to the Settings struct to be converted.
    ///
    /// # Returns
    ///
    /// * `Ok(Map<String, Value>)` - A map of setting keys to their values if successful.
    /// * `Err(Error)` - If serialization fails or the result is not a JSON object.
    ///
    /// # Example
    ///
    /// ```rust
    /// let settings = Settings::default();
    /// let map = settings_to_json_map(&settings)?;
    /// println!("Settings map: {:?}", map);
    /// ```
    pub fn settings_to_json_map(
        settings: &Settings,
    ) -> Result<serde_json::Map<String, Value>, Error> {
        let settings_value = serde_json::to_value(settings)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        settings_value.as_object().cloned().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Settings is not a JSON object",
            )
        })
    }

    /// Converts a JSON map back to a Settings struct.
    ///
    /// This function deserializes a map of settings values into a Settings struct.
    ///
    /// # Arguments
    ///
    /// * `map` - A serde_json Map containing setting keys and their values.
    ///
    /// # Returns
    ///
    /// * `Ok(Settings)` - The deserialized Settings struct if successful.
    /// * `Err(io::Error)` - If deserialization fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut map = serde_json::Map::new();
    /// map.insert("theme".to_string(), json!("dark"));
    ///
    /// let settings = json_map_to_settings(map)?;
    /// println!("Converted settings: {:?}", settings);
    /// ```
    pub fn json_map_to_settings(
        map: serde_json::Map<String, Value>,
    ) -> Result<Settings, io::Error> {
        serde_json::from_value(Value::Object(map))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Updates a single setting field with a new value.
    ///
    /// This method updates a specific setting identified by its key, validates that the
    /// key exists, and writes the updated settings to file.
    ///
    /// # Arguments
    ///
    /// * `&self` - Reference to the settings state.
    /// * `key` - A string slice identifying the setting to update.
    /// * `value` - The new value to assign to the setting.
    ///
    /// # Returns
    ///
    /// * `Ok(Settings)` - The updated Settings struct if successful.
    /// * `Err(io::Error)` - If the key doesn't exist or there's an error saving the settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// let result = settings_state.update_setting_field("theme", json!("dark"))?;
    /// println!("Updated settings: {:?}", result);
    /// ```
    pub fn update_setting_field(&self, key: &str, value: Value) -> Result<Settings, io::Error> {
        let mut settings = self.0.lock().unwrap();

        let mut settings_map = Self::settings_to_json_map(&settings)?;

        // Update the field
        if settings_map.contains_key(key) {
            settings_map.insert(key.to_string(), value);
        } else {
            return Err(Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown settings key: {}", key),
            ));
        }

        let updated_settings = Self::json_map_to_settings(settings_map)?;
        *settings = updated_settings.clone();
        self.write_settings_to_file(&updated_settings)?;

        Ok(updated_settings)
    }

    /// Retrieves the value of a specific setting field.
    ///
    /// This method gets the value of a setting identified by its key.
    ///
    /// # Arguments
    ///
    /// * `&self` - Reference to the settings state.
    /// * `key` - A string slice identifying the setting to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Value)` - The value of the requested setting if found.
    /// * `Err(Error)` - If the key doesn't exist or there's an error accessing the settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// let theme = settings_state.get_setting_field("theme")?;
    /// println!("Current theme: {}", theme);
    /// ```
    pub fn get_setting_field(&self, key: &str) -> Result<Value, Error> {
        let settings = self.0.lock().unwrap();
        let settings_value =
            serde_json::to_value(&*settings).map_err(|e| Error::new(io::ErrorKind::Other, e))?;

        if let Some(obj) = settings_value.as_object() {
            obj.get(key).cloned().ok_or_else(|| {
                Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown settings key: {}", key),
                )
            })
        } else {
            Err(Error::new(
                io::ErrorKind::InvalidData,
                "Failed to serialize settings to object",
            ))
        }
    }

    /// Updates multiple settings fields at once.
    ///
    /// This method applies a batch of updates to the settings in a single operation,
    /// writing the updated settings to file.
    ///
    /// # Arguments
    ///
    /// * `&self` - Reference to the settings state.
    /// * `updates` - A map of setting keys to their new values.
    ///
    /// # Returns
    ///
    /// * `Ok(Settings)` - The final updated Settings struct if successful.
    /// * `Err(io::Error)` - If any key doesn't exist, no updates were provided, or there's an error saving the settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut updates = serde_json::Map::new();
    /// updates.insert("theme".to_string(), json!("dark"));
    /// updates.insert("notifications".to_string(), json!(true));
    ///
    /// let result = settings_state.update_multiple_settings(&updates)?;
    /// println!("Updated settings: {:?}", result);
    /// ```
    pub fn update_multiple_settings(
        &self,
        updates: &serde_json::Map<String, Value>,
    ) -> Result<Settings, io::Error> {
        let mut last_updated_settings = None;

        for (key, value) in updates {
            // We reuse the existing function here
            let updated = self.update_setting_field(key, value.clone())?;
            last_updated_settings = Some(updated);
        }

        // Return the last successful update
        last_updated_settings
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No settings were provided"))
    }

    /// resets the settings file, effectively removing all saved settings.
    ///
    /// This method deletes the settings file from the disk, meaning the application will
    /// lose all settings
    ///
    /// # Arguments
    ///
    /// * `&self` - Reference to the settings state.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings file was successfully deleted.
    /// * `Err(io::Error)` - If there was an error during the deletion process.
    ///
    /// # Example
    ///
    /// ```rust
    /// let result = settings_state.delete_settings();
    /// match result {
    ///     Ok(_) => println!("Settings file has been deleted."),
    ///     Err(e) => eprintln!("Failed to delete settings: {}", e),
    /// }
    /// ```
    pub fn reset_settings(&self) -> Result<Settings, io::Error> {
        let mut settings = self.0.lock().unwrap();

        let default_settings = Settings::default();
        *settings = default_settings.clone();
        self.write_settings_to_file(&default_settings)?;

        Ok(default_settings)
    }

    /// Creates a new SettingsState with a custom path for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where settings will be stored.
    ///
    /// # Returns
    ///
    /// A new SettingsState instance configured with the specified path.
    ///
    /// # Example
    ///
    /// ```rust
    /// let test_path = PathBuf::from("test_settings.json");
    /// let settings_state = SettingsState::new_with_path(test_path);
    /// ```
    // For testing - allows creating a SettingsState with a custom path
    #[cfg(test)]
    pub fn new_with_path(path: PathBuf) -> Self {
        let mut defaults = Settings::default();
        defaults.abs_file_path_buf = path;
        Self(Arc::new(Mutex::new(
            Self::write_settings_to_file_and_save_in_state(defaults),
        )))
    }

    /// Writes the current settings to the configured file path.
    ///
    /// This method serializes the settings to JSON and saves them to disk.
    ///
    /// # Arguments
    ///
    /// * `&self` - Reference to the settings state.
    /// * `settings` - A reference to the Settings struct to be saved.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the settings were successfully written to file.
    /// * `Err(io::Error)` - If there was an error creating directories, opening the file, or writing to it.
    ///
    /// # Example
    ///
    /// ```rust
    /// let settings = Settings::default();
    /// settings_state.write_settings_to_file(&settings)?;
    /// ```
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

    /// Creates a default settings instance and writes it to file.
    ///
    /// This method initializes a new Settings with default values and saves it to disk.
    ///
    /// # Returns
    ///
    /// The created Settings instance with default values.
    ///
    /// # Example
    ///
    /// ```rust
    /// let default_settings = SettingsState::write_default_settings_to_file_and_save_in_state();
    /// ```
    fn write_default_settings_to_file_and_save_in_state() -> Settings {
        let defaults = Settings::default();
        Self::write_settings_to_file_and_save_in_state(defaults)
    }

    /// Helper method to write settings to a file and return the settings instance.
    ///
    /// This method creates a settings state with the provided defaults, writes them to file,
    /// and returns the settings instance.
    ///
    /// # Arguments
    ///
    /// * `defaults` - The Settings instance to be written to file.
    ///
    /// # Returns
    ///
    /// The provided Settings instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// let settings = Settings::default();
    /// let saved_settings = SettingsState::write_settings_to_file_and_save_in_state(settings);
    /// ```
    fn write_settings_to_file_and_save_in_state(defaults: Settings) -> Settings {
        let settings_state = Self(Arc::new(Mutex::new(defaults.clone())));

        if let Err(e) = settings_state.write_settings_to_file(&defaults) {
            log_error!(&format!("Error writing settings to file: {}", e));
        }

        defaults
    }

    /// Reads settings from a file path for testing purposes.
    ///
    /// This method loads and deserializes Settings from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path from which to read the settings.
    ///
    /// # Returns
    ///
    /// * `Ok(Settings)` - The deserialized Settings struct if successful.
    /// * `Err(io::Error)` - If there was an error reading or parsing the file.
    ///
    /// # Example
    ///
    /// ```rust
    /// let test_path = PathBuf::from("test_settings.json");
    /// let settings = SettingsState::read_settings_from_file(&test_path)?;
    /// println!("Read settings: {:?}", settings);
    /// ```
    pub fn read_settings_from_file(path: &PathBuf) -> io::Result<Settings> {
        use std::io::Read;
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

#[cfg(test)]
mod tests_settings {
    use super::*;
    use serde_json::{json, Map, Value};
    use tempfile::tempdir;

    /// Tests that the default settings have the expected initial values.
    ///
    /// Verifies that a newly created Settings instance has the correct
    /// default values for all properties.
    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.darkmode, false);
        //assert_eq!(settings.custom_themes, vec![]);
        assert_eq!(settings.default_theme, "".to_string());
        //assert_eq!(settings.default_themes_path, Default::default());
        //assert_eq!(settings.default_folder_path_on_opening, Default::default());
        assert_eq!(settings.default_checksum_hash, ChecksumMethod::SHA256);
        assert_eq!(settings.logging_level, LoggingLevel::Full);
        assert_eq!(
            settings.abs_file_path_buf,
            constants::SETTINGS_CONFIG_ABS_PATH.to_path_buf()
        );
    }

    /// Tests the creation of a new SettingsState with a custom path.
    ///
    /// Verifies that:
    /// 1. The settings file is created at the specified path
    /// 2. The file can be read back
    /// 3. The read settings have the expected default values
    /// 4. The path in the settings matches the custom path
    #[test]
    fn test_settings_state_creation() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("settings.json");

        // Create a new Settings with our test path
        let _settings_state = SettingsState::new_with_path(test_path.clone());

        // Verify the file was created
        assert!(
            test_path.exists(),
            "Settings file should exist after creation"
        );

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
    fn test_init_settings_json_exists() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("settings.json");

        // Step 1: Create the first SettingsState and update some values
        let settings_state = SettingsState::new_with_path(test_path.clone());

        let mut updates = Map::new();
        updates.insert("darkmode".to_string(), json!(true));
        updates.insert("default_theme".to_string(), json!("solarized"));

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_ok(), "Settings update should succeed");

        // Step 2: Drop the first state and reinitialize from file
        drop(settings_state);

        let loaded = SettingsState::read_settings_from_file(&test_path);
        assert!(loaded.is_ok(), "Should load settings from file after reload");

        let loaded_settings = loaded.unwrap();
        assert_eq!(loaded_settings.darkmode, true);
        assert_eq!(loaded_settings.default_theme, "solarized");
    }

    /// Tests writing custom settings to a file.
    ///
    /// Verifies that:
    /// 1. Modified settings can be written to disk successfully
    /// 2. The written settings can be read back correctly
    /// 3. The read settings match the original modified values
    #[test]
    fn test_write_settings_to_file() {
        // Create a temporary directory
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_path = temp_dir.path().join("settings.json");

        // Create a custom metadata object
        let mut settings = Settings::default();
        settings.abs_file_path_buf = test_path.clone();
        settings.logging_level = LoggingLevel::Partial;
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
        assert_eq!(
            read_settings.default_folder_path_on_opening,
            PathBuf::from("temp_dir")
        );
    }

    /// Tests updating the darkmode setting field.
    ///
    /// Verifies that:
    /// 1. The darkmode field can be updated to true
    /// 2. The returned settings object reflects the updated value
    #[test]
    fn test_update_darkmode_field() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("darkmode", json!(true));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().darkmode, true);
    }

    /// Tests updating the default_theme setting field.
    ///
    /// Verifies that:
    /// 1. The default_theme field can be updated to a new string value
    /// 2. The returned settings object reflects the updated value
    #[test]
    fn test_update_default_theme_field() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("default_theme", json!("ocean"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().default_theme, "ocean");
    }

    /// Tests updating the default_checksum_hash setting field.
    ///
    /// Verifies that:
    /// 1. The default_checksum_hash field can be updated to a new string value
    /// 2. The returned settings object reflects the updated value
    #[test]
    fn test_update_default_checksum_hash_field() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("default_checksum_hash", json!("MD5"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().default_checksum_hash, ChecksumMethod::MD5);
    }

    /// Tests updating the custom_themes setting field.
    ///
    /// Verifies that:
    /// 1. The custom_themes field can be updated to an array of strings
    /// 2. The returned settings object reflects the updated values
    #[test]
    fn test_update_custom_themes_field() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let themes = vec!["dark".to_string(), "light".to_string()];
        let result = state.update_setting_field("custom_themes", json!(themes.clone()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().custom_themes, themes);
    }

    /// Tests updating path-type settings fields.
    ///
    /// Verifies that:
    /// 1. The default_themes_path field can be updated with a path string
    /// 2. The default_folder_path_on_opening field can be updated with a path string
    /// 3. Both fields are properly converted to PathBuf values
    #[test]
    fn test_update_path_fields() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let path = "/some/path";
        let result1 = state.update_setting_field("default_themes_path", json!(path));
        let result2 = state.update_setting_field("default_folder_path_on_opening", json!(path));

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap().default_themes_path, PathBuf::from(path));
        assert_eq!(
            result2.unwrap().default_folder_path_on_opening,
            PathBuf::from(path)
        );
    }

    /// Tests updating the logging_state setting field.
    ///
    /// Verifies that:
    /// 1. The logging_state field can be updated to a different enum value
    /// 2. The returned settings object reflects the updated enum value
    #[test]
    fn test_update_logging_level_field() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("logging_level", json!("Minimal"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().logging_level, LoggingLevel::Minimal);
    }

    /// Tests error handling when attempting to update a non-existent key.
    ///
    /// Verifies that:
    /// 1. Attempting to update a non-existent key results in an error
    /// 2. The error message contains "Unknown settings key"
    #[test]
    fn test_invalid_key() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("non_existing_key", json!("value"));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown settings key"));
    }

    /// Tests type validation when updating the darkmode field.
    ///
    /// Verifies that:
    /// 1. Attempting to update the darkmode field with a non-boolean value results in an error
    /// 2. The error message indicates the type mismatch
    #[test]
    fn test_invalid_type_for_darkmode() {
        let state = SettingsState::new_with_path(
            tempfile::NamedTempFile::new().unwrap().path().to_path_buf(),
        );

        let result = state.update_setting_field("darkmode", json!("not_a_bool"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("expected a boolean") || err.contains("invalid type"));
    }

    /// Tests retrieving an existing setting field.
    ///
    /// Verifies that:
    /// 1. A previously set field can be retrieved successfully
    /// 2. The retrieved value matches what was set
    #[test]
    fn test_get_existing_field() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        // Set a known value
        settings_state
            .update_setting_field("darkmode", json!(true))
            .unwrap();

        // Call get_setting_field
        let result = settings_state.get_setting_field("darkmode");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(true));
    }

    /// Tests error handling when retrieving a non-existent key.
    ///
    /// Verifies that:
    /// 1. Attempting to get a non-existent key results in an error
    /// 2. The error message contains "Unknown settings key"
    #[test]
    fn test_get_invalid_key() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let result = settings_state.get_setting_field("non_existing_key");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown settings key"));
    }

    /// Tests retrieving a complex field (array type).
    ///
    /// Verifies that:
    /// 1. A complex field (array of strings) can be retrieved successfully
    /// 2. The retrieved value matches what was set
    #[test]
    fn test_get_complex_field() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        settings_state
            .update_setting_field("custom_themes", json!(["dark", "light"]))
            .unwrap();

        let result = settings_state.get_setting_field("custom_themes");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), json!(["dark", "light"]));
    }

    /// Tests updating multiple valid settings fields at once.
    ///
    /// Verifies that:
    /// 1. Multiple fields can be updated in a single operation
    /// 2. All updated fields have the expected values in the returned settings
    #[test]
    fn test_update_multiple_valid_fields() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let mut updates: Map<String, Value> = Map::new();
        updates.insert("darkmode".into(), Value::Bool(true));
        updates.insert("default_theme".into(), Value::String("gruvbox".into()));

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.darkmode, true);
        assert_eq!(updated.default_theme, "gruvbox");
    }

    /// Tests error handling when updating with an invalid key.
    ///
    /// Verifies that:
    /// 1. Attempting to update multiple settings with a non-existent key results in an error
    /// 2. The error message identifies the specific invalid key
    #[test]
    fn test_update_with_invalid_key() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let mut updates: Map<String, Value> = Map::new();
        updates.insert("non_existing_field".into(), Value::String("value".into()));

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown settings key: non_existing_field"));
    }

    /// Tests error handling when updating with a mix of valid and invalid keys.
    ///
    /// Verifies that:
    /// 1. When attempting to update multiple settings with both valid and invalid keys,
    ///    the operation fails with an error
    /// 2. No partial updates are applied (all-or-nothing behavior)
    /// 3. The error message identifies the specific invalid key
    #[test]
    fn test_update_with_mixed_valid_and_invalid_keys() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let mut updates: Map<String, Value> = Map::new();
        updates.insert("darkmode".into(), Value::Bool(false));
        updates.insert("unknown".into(), Value::String("oops".into()));

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown settings key: unknown"));
    }

    /// Tests error handling when updating with an empty updates map.
    ///
    /// Verifies that:
    /// 1. Attempting to update with an empty map results in an error
    /// 2. The error message indicates that no settings were provided
    #[test]
    fn test_update_with_empty_updates_map() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let updates: Map<String, Value> = Map::new();

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "No settings were provided");
    }

    /// Tests type validation when updating with an invalid value type.
    ///
    /// Verifies that:
    /// 1. Attempting to update a field with a value of the wrong type results in an error
    /// 2. The error message indicates the type mismatch
    #[test]
    fn test_update_with_invalid_value_type() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let settings_state = SettingsState::new_with_path(temp_file.path().to_path_buf());

        let mut updates: Map<String, Value> = Map::new();
        updates.insert("darkmode".into(), Value::String("not_a_bool".into())); // darkmode expects bool

        let result = settings_state.update_multiple_settings(&updates);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid type: string"));
    }
}
