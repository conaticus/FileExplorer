use crate::error_handling::{Error, ErrorCode};
use crate::state::meta_data::MetaDataState;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Retrieves system metadata information as a JSON string.
/// This includes information about volumes, drives, and storage devices.
/// Updates the metadata state before returning the JSON.
///
/// # Arguments
/// * `state` - The application state containing metadata information.
///
/// # Returns
/// * `Ok(String)` - A JSON string containing the metadata if successful.
/// * `Err(String)` - If there was an error retrieving or serializing the metadata.
///
/// # Example
/// ```javascript
/// invoke('get_meta_data_as_json')
///   .then((response) => {
///     // Process the metadata JSON
///     console.log('Metadata:', response);
///     const metadata = JSON.parse(response);
///     // Use the metadata in the UI
///   })
///   .catch((error) => {
///     console.error('Error retrieving metadata:', error);
///   });
/// ```
#[tauri::command]
pub fn get_meta_data_as_json(state: State<Arc<Mutex<MetaDataState>>>) -> Result<String, String> {
    let meta_data = state.lock().map_err(|_| {
        Error::new(
            ErrorCode::InternalError,
            "Failed to acquire lock on metadata state".to_string(),
        ).to_json()
    })?.refresh_volumes();

    if let Err(e) = meta_data {
        return Err(Error::new(
            ErrorCode::InternalError,
            format!("Error refreshing volumes: {}", e),
        )
        .to_json());
    }

    let meta_data = state.lock().map_err(|_| {
        Error::new(
            ErrorCode::InternalError,
            "Failed to acquire lock on metadata state".to_string(),
        ).to_json()
    })?.0.clone();

    serde_json::to_string(&meta_data).map_err(|e| {
        Error::new(
            ErrorCode::InternalError,
            format!("Error serializing metadata: {}", e),
        )
        .to_json()
    })
}

#[cfg(test)]
pub fn get_meta_data_as_json_impl(state: Arc<Mutex<MetaDataState>>) -> Result<String, String> {
    let meta_data = state.lock().map_err(|_| {
        Error::new(
            ErrorCode::InternalError,
            "Failed to acquire lock on metadata state".to_string(),
        ).to_json()
    })?.refresh_volumes();

    if let Err(e) = meta_data {
        return Err(Error::new(
            ErrorCode::InternalError,
            format!("Error refreshing volumes: {}", e),
        )
        .to_json());
    }

    let meta_data = state.lock().map_err(|_| {
        Error::new(
            ErrorCode::InternalError,
            "Failed to acquire lock on metadata state".to_string(),
        ).to_json()
    })?.0.clone();

    serde_json::to_string(&meta_data).map_err(|e| {
        Error::new(
            ErrorCode::InternalError,
            format!("Error serializing metadata: {}", e),
        )
        .to_json()
    })
}

#[tauri::command]
pub fn update_meta_data(state: State<Arc<Mutex<MetaDataState>>>) -> Result<(), String> {
    match state.lock().unwrap().refresh_volumes() {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorCode::InternalError, format!("Error: {}", e)).to_json()),
    }
}

#[cfg(test)]
mod tests_meta_data_commands {
    use super::*;

    // Helper function to create a test MetaDataState
    fn create_test_meta_data_state() -> Arc<Mutex<MetaDataState>> {
        // Create a temporary MetaDataState for testing
        Arc::new(Mutex::new(MetaDataState::new()))
    }

    #[test]
    fn test_get_meta_data_as_json_success() {
        let state = create_test_meta_data_state();

        // Call the implementation function with our test state
        let result = get_meta_data_as_json_impl(state.clone());

        // Check that we got a successful result
        assert!(result.is_ok());

        // Verify the JSON contains expected metadata structure
        let json = result.unwrap();
        assert!(json.contains("volumes"));
        // Add more specific assertions based on your expected data structure
    }

    #[test]
    fn test_get_meta_data_as_json_contains_volumes() {
        let state = create_test_meta_data_state();

        // First update the metadata to ensure we have fresh data
        state
            .lock()
            .unwrap()
            .refresh_volumes()
            .expect("Failed to refresh volumes");

        // Call the implementation function
        let result = get_meta_data_as_json_impl(state);

        // Verify the result contains volumes information
        assert!(result.is_ok());
        let json = result.unwrap();

        // Parse the JSON to check its structure
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Failed to parse JSON");

        // Check that the parsed JSON has the expected structure
        assert!(parsed.is_object());
        assert!(parsed
            .as_object()
            .unwrap()
            .contains_key("all_volumes_with_information"));
    }
}
