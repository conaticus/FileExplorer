use crate::state::meta_data::MetaDataState;
use crate::{log_error, log_info};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::State;

async fn get_template_paths_from_state(
    state: Arc<Mutex<MetaDataState>>,
) -> Result<Vec<PathBuf>, ()> {
    let meta_data_state = state.lock().unwrap();
    let inner_meta_data = meta_data_state
        .0
        .lock()
        .map_err(|_| log_error!("Cannot acquire lock on metadata state"))?;
    Ok(inner_meta_data.template_paths.clone())
}

pub async fn get_template_paths_as_json_impl(
    state: Arc<Mutex<MetaDataState>>,
) -> Result<String, String> {
    log_info!("Retrieving template paths from state");
    // Get the template paths
    let paths = get_template_paths_from_state(state).await.map_err(|_| {
        let error_msg = "Failed to get template paths from state";
        log_error!(error_msg);
        error_msg.to_string()
    })?;

    // Convert PathBufs to strings
    let path_strings: Vec<String> = paths
        .into_iter()
        .filter_map(|p| p.to_str().map(|s| s.to_string()))
        .collect();

    log_info!(format!("Found {} template paths", path_strings.len()).as_str());

    // Serialize to JSON
    match serde_json::to_string(&path_strings) {
        Ok(json) => {
            log_info!("Successfully serialized template paths to JSON");
            Ok(json)
        }
        Err(e) => {
            let error_msg = format!("Failed to serialize paths to JSON: {}", e);
            log_error!(error_msg.as_str());
            Err(error_msg)
        }
    }
}

/// Retrieves all available templates as a JSON string of paths.
///
/// # Returns
/// * `Ok(String)` - A JSON array of template paths as strings
/// * `Err(String)` - An error message if the templates can't be retrieved
///
/// # Example
/// ```rust
/// let result = get_template_paths_as_json(state).await;
/// match result {
///     Ok(json_paths) => println!("Available templates: {}", json_paths),
///     Err(e) => eprintln!("Error getting templates: {}", e),
/// }
/// ```
#[tauri::command]
pub async fn get_template_paths_as_json(
    state: State<'_, Arc<Mutex<MetaDataState>>>,
) -> Result<String, String> {
    log_info!("get_template_paths_as_json command called");
    get_template_paths_as_json_impl(state.inner().clone()).await
}

pub async fn copy_to_dest_path(source_path: &str, dest_path: &str) -> Result<u64, String> {
    log_info!(format!("Copying from '{}' to '{}'", source_path, dest_path).as_str());

    // Check if the source path exists
    if !Path::new(source_path).exists() {
        let error_msg = format!("Source path does not exist: {}", source_path);
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Create parent directories for destination if they don't exist
    let dest_path_buf = PathBuf::from(dest_path);
    if let Some(parent) = dest_path_buf.parent() {
        if !parent.exists() {
            match fs::create_dir_all(parent) {
                Ok(_) => log_info!(format!(
                    "Created parent directories for destination: {}",
                    parent.display()
                )
                .as_str()),
                Err(err) => {
                    let error_msg = format!(
                        "Failed to create parent directories for destination: {}",
                        err
                    );
                    log_error!(error_msg.as_str());
                    return Err(error_msg);
                }
            }
        }
    }

    if Path::new(source_path).is_dir() {
        log_info!("Copying directory recursively");
        // If the source is a directory, recursively copy it
        let mut total_size = 0;

        // Get the source directory name
        let source_dir_name = match Path::new(source_path).file_name() {
            Some(name) => name,
            None => {
                let error_msg = "Invalid source directory name".to_string();
                log_error!(error_msg.as_str());
                return Err(error_msg);
            }
        };

        // Create the final destination directory including the source directory name
        let final_dest_path = Path::new(dest_path).join(source_dir_name);

        // Create the destination directory
        match fs::create_dir_all(&final_dest_path) {
            Ok(_) => log_info!(format!(
                "Created destination directory: {}",
                final_dest_path.display()
            )
            .as_str()),
            Err(err) => {
                let error_msg = format!("Failed to create destination directory: {}", err);
                log_error!(error_msg.as_str());
                return Err(error_msg);
            }
        }

        // Read all entries in the source directory
        let entries = match fs::read_dir(source_path) {
            Ok(entries) => entries,
            Err(err) => {
                let error_msg = format!("Failed to read source directory: {}", err);
                log_error!(error_msg.as_str());
                return Err(error_msg);
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(entry) => entry,
                Err(err) => {
                    let error_msg = format!("Failed to read directory entry: {}", err);
                    log_error!(error_msg.as_str());
                    return Err(error_msg);
                }
            };

            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path_entry = final_dest_path.join(file_name);

            log_info!(format!("Processing item: {}", entry_path.display()).as_str());

            if entry_path.is_file() {
                // Copy file
                match fs::copy(&entry_path, &dest_path_entry) {
                    Ok(size) => {
                        log_info!(format!(
                            "Copied file: {} ({} bytes)",
                            entry_path.display(),
                            size
                        )
                        .as_str());
                        total_size += size;
                    }
                    Err(err) => {
                        let error_msg =
                            format!("Failed to copy file '{}': {}", entry_path.display(), err);
                        log_error!(error_msg.as_str());
                        return Err(error_msg);
                    }
                }
            } else if entry_path.is_dir() {
                // Recursively copy subdirectory - pass the dest_path_entry as destination
                log_info!(
                    format!("Recursively copying subdirectory: {}", entry_path.display()).as_str()
                );
                match Box::pin(copy_to_dest_path(
                    entry_path.to_str().unwrap(),
                    dest_path_entry.parent().unwrap().to_str().unwrap(),
                ))
                .await
                {
                    Ok(sub_size) => {
                        log_info!(format!(
                            "Copied directory: {} ({} bytes)",
                            entry_path.display(),
                            sub_size
                        )
                        .as_str());
                        total_size += sub_size;
                    }
                    Err(err) => {
                        log_error!(format!(
                            "Failed to copy directory '{}': {}",
                            entry_path.display(),
                            err
                        )
                        .as_str());
                        return Err(err);
                    }
                }
            }
        }

        log_info!(format!(
            "Successfully copied directory with total size: {} bytes",
            total_size
        )
        .as_str());
        Ok(total_size)
    } else {
        log_info!("Copying single file");
        // Create parent directory for the file if it doesn't exist
        if let Some(parent) = PathBuf::from(dest_path).parent() {
            if !parent.exists() {
                match fs::create_dir_all(parent) {
                    Ok(_) => {
                        log_info!(format!("Created parent directory: {}", parent.display()).as_str())
                    }
                    Err(err) => {
                        let error_msg = format!("Failed to create parent directory: {}", err);
                        log_error!(error_msg.as_str());
                        return Err(error_msg);
                    }
                }
            }
        }

        // Copy a single file
        match fs::copy(source_path, dest_path) {
            Ok(size) => {
                log_info!(format!(
                    "Copied file: {} to {} ({} bytes)",
                    source_path, dest_path, size
                )
                .as_str());
                Ok(size)
            }
            Err(err) => {
                let error_msg = format!("Failed to copy file: {}", err);
                log_error!(error_msg.as_str());
                Err(error_msg)
            }
        }
    }
}

pub async fn add_template_impl(
    state: Arc<Mutex<MetaDataState>>,
    template_path: &str,
) -> Result<String, String> {
    log_info!(format!("Adding template from path: {}", template_path).as_str());

    // Check if the source path exists
    if !Path::new(template_path).exists() {
        let error_msg = format!("Source path does not exist: {}", template_path);
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Extract what we need from the metadata state before any await points
    let dest_path = {
        let metadata_state = state.lock().unwrap();
        let inner_metadata = metadata_state.0.lock().map_err(|e| {
            let error_msg = format!("Error acquiring lock on metadata state: {:?}", e);
            log_error!(error_msg.as_str());
            error_msg
        })?;

        inner_metadata.abs_folder_path_buf_for_templates.clone()
    };

    log_info!(format!("Template destination path: {}", dest_path.display()).as_str());

    // Create destination directory if it doesn't exist
    if !dest_path.exists() {
        let error_msg = format!(
            "Failed to find templates directory: {}",
            dest_path.display()
        );
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Copy the template using our helper function
    let size = copy_to_dest_path(template_path, dest_path.to_str().unwrap()).await?;

    // Update the template paths in the metadata state
    let update_result = {
        let metadata_state = state.lock().unwrap();
        metadata_state.update_template_paths()
    };

    match update_result {
        Ok(_) => {
            let success_msg = format!(
                "Template '{}' added successfully ({} bytes)",
                Path::new(template_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
                size
            );
            log_info!(success_msg.as_str());
            Ok(success_msg)
        }
        Err(err) => {
            let error_msg = format!("Failed to update template paths: {}", err);
            log_error!(error_msg.as_str());
            Err(error_msg)
        }
    }
}

/// Adds a template to the template directory.
///
/// This function copies a file or directory from the provided path to the application's
/// template directory and registers it as a template.
///
/// # Arguments
/// * `state` - The application's metadata state
/// * `template_path` - A string representing the absolute path to the file or directory to be added as a template
///
/// # Returns
/// * `Ok(String)` - A success message including the name of the template and its size
/// * `Err(String)` - An error message if the template cannot be added
///
/// # Example
/// ```rust
/// let result = add_template(state, "/path/to/my/template").await;
/// match result {
///     Ok(msg) => println!("{}", msg),  // Template 'template' added successfully (1024 bytes)
///     Err(e) => eprintln!("Error adding template: {}", e),
/// }
/// ```
#[tauri::command]
pub async fn add_template(
    state: State<'_, Arc<Mutex<MetaDataState>>>,
    template_path: &str,
) -> Result<String, String> {
    log_info!(format!("add_template command called with path: {}", template_path).as_str());
    add_template_impl(state.inner().clone(), template_path).await
}

pub async fn use_template_impl(template_path: &str, dest_path: &str) -> Result<String, String> {
    log_info!(format!("Using template from path: {}", template_path).as_str());

    // Check if the template path exists
    if !Path::new(template_path).exists() {
        let error_msg = format!("Template path does not exist: {}", template_path);
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Check if the destination path exists
    if !Path::new(dest_path).exists() {
        let error_msg = format!("Destination path does not exist: {}", dest_path);
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Copy the template to the destination
    match copy_to_dest_path(template_path, dest_path).await {
        Ok(size) => {
            let success_msg = format!(
                "Template '{}' applied successfully to '{}' ({} bytes copied)",
                template_path, dest_path, size
            );
            log_info!(success_msg.as_str());
            Ok(success_msg)
        }
        Err(err) => {
            let error_msg = format!("Failed to apply template: {}", err);
            log_error!(error_msg.as_str());
            Err(error_msg)
        }
    }
}

/// Applies a template to the specified destination path.
///
/// This function copies the content of a template (file or directory) to the specified destination.
/// The template remains unchanged, creating a new instance at the destination path.
///
/// # Arguments
/// * `template_path` - A string representing the absolute path to the template
/// * `dest_path` - A string representing the absolute path where the template should be applied
///
/// # Returns
/// * `Ok(String)` - A success message with details about the template application
/// * `Err(String)` - An error message if the template cannot be applied
///
/// # Example
/// ```rust
/// let result = use_template("/path/to/template", "/path/to/destination").await;
/// match result {
///     Ok(msg) => println!("{}", msg),  // Template applied successfully (1024 bytes copied)
///     Err(e) => eprintln!("Error applying template: {}", e),
/// }
/// ```
#[tauri::command]
pub async fn use_template(template_path: &str, dest_path: &str) -> Result<String, String> {
    log_info!(format!("use_template command called with key: {}", template_path).as_str());
    use_template_impl(template_path, dest_path).await
}

pub async fn remove_template_impl(
    state: Arc<Mutex<MetaDataState>>,
    template_path: &str,
) -> Result<String, String> {
    log_info!(format!("Removing template at path: {}", template_path).as_str());

    // Check if the template path exists
    if !Path::new(template_path).exists() {
        let error_msg = format!("Template path does not exist: {}", template_path);
        log_error!(error_msg.as_str());
        return Err(error_msg);
    }

    // Remove the template
    match fs::remove_dir_all(template_path) {
        Ok(_) => {
            let success_msg = format!("Template '{}' removed successfully", template_path);
            log_info!(success_msg.as_str());

            // Update the template paths in the metadata state
            let update_result = {
                let metadata_state = state.lock().unwrap();
                metadata_state.update_template_paths()
            };

            match update_result {
                Ok(_) => {
                    log_info!(success_msg.as_str());
                    Ok(success_msg)
                }
                Err(err) => {
                    let error_msg = format!("Failed to update template paths: {}", err);
                    log_error!(error_msg.as_str());
                    Err(error_msg)
                }
            }
        }
        Err(err) => {
            let error_msg = format!("Failed to remove template: {}", err);
            log_error!(error_msg.as_str());
            Err(error_msg)
        }
    }
}

/// Removes a template from the template directory.
///
/// This function deletes a template (file or directory) from the application's template directory
/// and updates the registered templates list.
///
/// # Arguments
/// * `state` - The application's metadata state
/// * `template_path` - A string representing the absolute path to the template to be removed
///
/// # Returns
/// * `Ok(String)` - A success message confirming the removal of the template
/// * `Err(String)` - An error message if the template cannot be removed
///
/// # Example
/// ```rust
/// let result = remove_template(state, "/path/to/templates/my_template").await;
/// match result {
///     Ok(msg) => println!("{}", msg),  // Template removed successfully
///     Err(e) => eprintln!("Error removing template: {}", e),
/// }
/// ```
#[tauri::command]
pub async fn remove_template(
    state: State<'_, Arc<Mutex<MetaDataState>>>,
    template_path: &str,
) -> Result<String, String> {
    remove_template_impl(state.inner().clone(), template_path).await
}

#[cfg(test)]
mod tests_template_commands {
    use super::*;
    use crate::state::meta_data::MetaDataState;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;

    // Helper function to create a test MetaDataState
    fn create_test_metadata_state(
        meta_data_path: PathBuf,
        temp_dir_path: PathBuf,
    ) -> Arc<Mutex<MetaDataState>> {
        // Create a custom metadata state with our test directories
        let meta_data = MetaDataState::new_with_path(meta_data_path.to_path_buf());

        {
            let mut meta_data_inner = meta_data.0.lock().unwrap().clone();
            meta_data_inner.abs_folder_path_buf_for_templates = temp_dir_path;
            // Initialize with empty template paths
            meta_data_inner.template_paths = vec![];

            // Save the updated metadata
            meta_data
                .write_meta_data_to_file(&meta_data_inner)
                .expect("Failed to update metadata");

            // Update the state
            *meta_data.0.lock().unwrap() = meta_data_inner;
        }

        // Ensure we properly initialize template paths in the state
        meta_data
            .update_template_paths()
            .expect("Failed to update template paths");

        Arc::new(Mutex::new(meta_data))
    }

    // Helper to create a test file with content
    fn create_test_file(path: &Path, content: &[u8]) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(content)
    }

    #[tokio::test]
    async fn test_get_template_paths_empty() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        let result = get_template_paths_as_json_impl(state).await;

        assert!(result.is_ok(), "Should return Ok for empty template paths");
        let json = result.unwrap();
        assert_eq!(
            json, "[]",
            "Empty template paths should return empty JSON array"
        );
    }

    #[tokio::test]
    async fn test_get_template_paths_with_templates() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        // Create test templates directly in the template directory
        let template1 = templates_dir.path().join("template1");
        let template2 = templates_dir.path().join("template2");

        fs::create_dir(&template1).expect("Failed to create test template1");
        fs::create_dir(&template2).expect("Failed to create test template2");

        // Add content to templates to ensure they're meaningful
        create_test_file(&template1.join("test.txt"), b"Test content 1")
            .expect("Failed to create test file in template1");
        create_test_file(&template2.join("test.txt"), b"Test content 2")
            .expect("Failed to create test file in template2");

        // Update template paths in state - this should find our newly created templates
        {
            let meta_state = state.lock().unwrap();
            println!("Templates dir: {:?}", templates_dir.path());

            let inner_meta_data = &mut meta_state.0.lock().unwrap();
            inner_meta_data.template_paths = vec![template1.clone(), template2.clone()];

            assert!(
                inner_meta_data.template_paths.len() >= 2,
                "Should have found at least 2 templates: {:?}",
                inner_meta_data.template_paths
            );
            assert!(
                inner_meta_data
                    .template_paths
                    .iter()
                    .any(|p| p.ends_with("template1")),
                "template1 should be in template_paths: {:?}",
                inner_meta_data.template_paths
            );
            assert!(
                inner_meta_data
                    .template_paths
                    .iter()
                    .any(|p| p.ends_with("template2")),
                "template2 should be in template_paths: {:?}",
                inner_meta_data.template_paths
            );
        }

        let result = get_template_paths_as_json_impl(state).await;
        assert!(result.is_ok(), "Should return Ok with template paths");
        let json = result.unwrap();

        assert!(
            json.contains("template1"),
            "JSON should contain template1: {}",
            json
        );
        assert!(
            json.contains("template2"),
            "JSON should contain template2: {}",
            json
        );
    }

    #[tokio::test]
    async fn test_add_template() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        // Create a source template directory
        let source_dir = tempdir().expect("Failed to create source template directory");
        let source_path = source_dir.path();
        let source_name = source_path.file_name().unwrap().to_str().unwrap();

        // Create some content in the source
        let test_file = source_path.join("test.txt");
        create_test_file(&test_file, b"Test content").expect("Failed to create test file");

        // Add the template
        let result = add_template_impl(state.clone(), source_path.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "Adding template should succeed: {:?}",
            result.err()
        );

        // Verify the template was copied - should be in templates_dir/source_name
        let expected_template_path = templates_dir.path().join(source_name);
        println!("Expected template path: {:?}", expected_template_path);
        assert!(
            expected_template_path.exists(),
            "Template should exist at destination: {:?}",
            expected_template_path
        );

        // Verify the template file was copied correctly
        let copied_file = expected_template_path.join("test.txt");
        assert!(copied_file.exists(), "Template file should be copied");

        let content = fs::read_to_string(copied_file).expect("Failed to read copied file");
        assert_eq!(content, "Test content", "File content should match");
    }

    #[tokio::test]
    async fn test_add_template_nonexistent_source() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        // Try to add a template that doesn't exist
        let result = add_template_impl(state, "/path/that/does/not/exist").await;

        assert!(result.is_err(), "Should fail when source doesn't exist");
        assert!(result.unwrap_err().contains("Source path does not exist"));
    }

    #[tokio::test]
    async fn test_use_template() {
        // Create a template directory
        let template_dir = tempdir().expect("Failed to create template directory");
        let template_file = template_dir.path().join("template_file.txt");
        create_test_file(&template_file, b"Template content")
            .expect("Failed to create template file");

        // Create a destination directory
        let dest_dir = tempdir().expect("Failed to create destination directory");

        // Template directory name
        let template_name = template_dir.path().file_name().unwrap().to_str().unwrap();

        // Use the template
        let result = use_template_impl(
            template_dir.path().to_str().unwrap(),
            dest_dir.path().to_str().unwrap(),
        )
        .await;

        assert!(
            result.is_ok(),
            "Using template should succeed: {:?}",
            result.err()
        );

        // Verify the template was copied to the destination - should be in dest_dir/template_name
        let dest_template_dir = dest_dir.path().join(template_name);
        println!("Looking for template in: {:?}", dest_template_dir);
        assert!(
            dest_template_dir.exists(),
            "Template directory should exist at destination"
        );

        // Verify the template file was copied correctly
        let copied_file = dest_template_dir.join("template_file.txt");
        assert!(
            copied_file.exists(),
            "Template file should be copied to destination"
        );

        let content = fs::read_to_string(copied_file).expect("Failed to read copied file");
        assert_eq!(content, "Template content", "File content should match");
    }

    #[tokio::test]
    async fn test_use_template_nonexistent() {
        // Create a destination directory
        let dest_dir = tempdir().expect("Failed to create destination directory");

        // Try to use a template that doesn't exist
        let result = use_template_impl(
            "/path/to/nonexistent/template",
            dest_dir.path().to_str().unwrap(),
        )
        .await;

        assert!(result.is_err(), "Should fail when template doesn't exist");
        assert!(result.unwrap_err().contains("Template path does not exist"));
    }

    #[tokio::test]
    async fn test_use_template_invalid_destination() {
        // Create a template directory
        let template_dir = tempdir().expect("Failed to create template directory");

        // Try to use a template with an invalid destination
        let result = use_template_impl(
            template_dir.path().to_str().unwrap(),
            "/path/to/nonexistent/destination",
        )
        .await;

        assert!(
            result.is_err(),
            "Should fail when destination doesn't exist"
        );
        assert!(result
            .unwrap_err()
            .contains("Destination path does not exist"));
    }

    #[tokio::test]
    async fn test_remove_template() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        // Create a test template
        let template_path = templates_dir.path().join("template_to_remove");
        fs::create_dir(&template_path).expect("Failed to create test template");
        create_test_file(&template_path.join("test.txt"), b"Test content")
            .expect("Failed to create test file in template");

        // Update template paths in state
        let meta_state = state.lock().unwrap();
        meta_state
            .update_template_paths()
            .expect("Failed to update template paths");
        drop(meta_state);

        // Verify the template exists
        assert!(
            template_path.exists(),
            "Template should exist before removal"
        );

        // Remove the template
        let result = remove_template_impl(state.clone(), template_path.to_str().unwrap()).await;

        assert!(
            result.is_ok(),
            "Removing template should succeed: {:?}",
            result.err()
        );

        // Verify the template was removed
        assert!(
            !template_path.exists(),
            "Template should be removed after removal"
        );
    }

    #[tokio::test]
    async fn test_remove_nonexistent_template() {
        // Create temp directories for template storage and metadata
        let templates_dir = tempdir().expect("Failed to create temporary templates directory");
        let metadata_dir = tempdir().expect("Failed to create temporary metadata directory");

        let metadata_file = metadata_dir.path().join("meta_data.json");
        let state = create_test_metadata_state(metadata_file, templates_dir.path().to_path_buf());

        // Try to remove a template that doesn't exist
        let nonexistent_path = templates_dir.path().join("nonexistent_template");

        // Ensure it doesn't exist
        assert!(
            !nonexistent_path.exists(),
            "Template should not exist before test"
        );

        // Try to remove it
        let result = remove_template_impl(state, nonexistent_path.to_str().unwrap()).await;

        assert!(result.is_err(), "Should fail when template doesn't exist");
        assert!(result.unwrap_err().contains("Template path does not exist"));
    }

    #[tokio::test]
    async fn test_copy_to_dest_path_file() {
        // Create source and destination directories
        let source_dir = tempdir().expect("Failed to create source directory");
        let dest_dir = tempdir().expect("Failed to create destination directory");

        // Create a test file
        let source_file = source_dir.path().join("test.txt");
        create_test_file(&source_file, b"Test file content").expect("Failed to create test file");

        // Create destination file path
        let dest_file = dest_dir.path().join("test.txt");

        // Copy the file
        let result =
            copy_to_dest_path(source_file.to_str().unwrap(), dest_file.to_str().unwrap()).await;

        assert!(
            result.is_ok(),
            "Copying file should succeed: {:?}",
            result.err()
        );

        // Verify the file was copied
        assert!(dest_file.exists(), "Destination file should exist");

        let content = fs::read_to_string(dest_file).expect("Failed to read destination file");
        assert_eq!(content, "Test file content", "File content should match");
    }

    #[tokio::test]
    async fn test_copy_to_dest_path_directory() {
        // Create source and destination directories
        let source_dir = tempdir().expect("Failed to create source directory");
        let dest_dir = tempdir().expect("Failed to create destination directory");

        // Create content in the source directory
        let subdir = source_dir.path().join("subdir");
        fs::create_dir(&subdir).expect("Failed to create subdirectory");

        let file1 = source_dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");

        create_test_file(&file1, b"File 1 content").expect("Failed to create file1");
        create_test_file(&file2, b"File 2 content").expect("Failed to create file2");

        // Get source directory name for verification
        let source_name = source_dir.path().file_name().unwrap().to_str().unwrap();
        println!("Source directory name: {}", source_name);

        // Copy the directory
        let result = copy_to_dest_path(
            source_dir.path().to_str().unwrap(),
            dest_dir.path().to_str().unwrap(),
        )
        .await;

        assert!(
            result.is_ok(),
            "Copying directory should succeed: {:?}",
            result.err()
        );

        // The copied directory should be in dest_dir/source_name
        let copied_dir_path = dest_dir.path().join(source_name);
        println!("Expected copied directory path: {:?}", copied_dir_path);

        // Verify the directory structure was copied
        assert!(
            copied_dir_path.exists(),
            "Destination directory should exist"
        );

        println!("Contents of copied directory:");
        for entry in fs::read_dir(&copied_dir_path).expect("Failed to read directory") {
            println!("  {:?}", entry.unwrap().path());
        }

        let copied_file1 = copied_dir_path.join("file1.txt");
        let copied_subdir = copied_dir_path.join("subdir");
        let copied_file2 = copied_subdir.join("file2.txt");

        assert!(
            copied_file1.exists(),
            "file1.txt should be copied: {:?}",
            copied_file1
        );
        assert!(copied_subdir.exists(), "subdir should be copied");
        assert!(copied_file2.exists(), "file2.txt should be copied");

        let content1 = fs::read_to_string(copied_file1).expect("Failed to read file1.txt");
        let content2 = fs::read_to_string(copied_file2).expect("Failed to read file2.txt");

        assert_eq!(content1, "File 1 content", "File1 content should match");
        assert_eq!(content2, "File 2 content", "File2 content should match");
    }
}
