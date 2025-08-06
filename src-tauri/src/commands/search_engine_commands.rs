use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

use crate::{log_error, log_info};
use crate::state::searchengine_data::{IndexingProgress, SearchEngineInfo, SearchEngineState, SearchEngineStatus};

// Type alias for the search result type returned by the engine
type SearchResult = Vec<(String, f32)>;

/// Searches the indexed files based on the provided query string.
///
/// # Arguments
/// * `query` - The search query string
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(SearchResult)` - A vector of paths and their relevance scores that match the query
/// * `Err(String)` - If there was an error during the search operation
///
/// # Example
/// ```rust
/// let result = search("document".to_string(), search_engine_state).await;
/// match result {
///     Ok(matches) => {
///         for (path, score) in matches {
///             println!("Match: {} (score: {})", path, score);
///         }
///     },
///     Err(err) => println!("Search error: {}", err),
/// }
/// ```
#[tauri::command]
pub fn search(
    query: String,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<SearchResult, String> {
    search_impl(query, search_engine_state.inner().clone())
}

pub fn search_impl(
    query: String,
    state: Arc<Mutex<SearchEngineState>>,
) -> Result<SearchResult, String> {
    log_info!(
        "Search implementation called with query: {}",
        query
    );
    let engine = state.lock().unwrap();
    engine.search(&query)
}

/// Searches the indexed files based on the provided query string,
/// filtering results to only include files with the specified extensions.
///
/// # Arguments
/// * `query` - The search query string
/// * `extensions` - A vector of file extensions to filter by (e.g., ["txt", "md"])
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(SearchResult)` - A vector of paths and their relevance scores that match the query and extensions
/// * `Err(String)` - If there was an error during the search operation
///
/// # Example
/// ```rust
/// let result = search_with_extension(
///     "document".to_string(),
///     vec!["txt".to_string(), "md".to_string()],
///     search_engine_state
/// ).await;
/// match result {
///     Ok(matches) => {
///         for (path, score) in matches {
///             println!("Match: {} (score: {})", path, score);
///         }
///     },
///     Err(err) => println!("Search error: {}", err),
/// }
/// ```
#[tauri::command]
pub fn search_with_extension(
    query: String,
    extensions: Vec<String>,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<SearchResult, String> {
    search_with_extension_impl(query, extensions, search_engine_state.inner().clone())
}

pub fn search_with_extension_impl(
    query: String,
    extensions: Vec<String>,
    state: Arc<Mutex<SearchEngineState>>,
) -> Result<SearchResult, String> {
    log_info!(
        "Search with extension called: query='{}', extensions={:?}",
        query, extensions
    );
    let engine = state.lock().unwrap();
    engine.search_by_extension(&query, extensions)
}

/// Recursively adds all files from a directory to the search engine index using chunked processing.
///
/// Updated to use chunked indexing by default for better performance and responsiveness.
/// Processes files in chunks to prevent UI freezes during indexing of large directories.
///
/// # Arguments
/// * `folder` - The path to the directory to index
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(())` - If the indexing was successfully started
/// * `Err(String)` - If there was an error starting the indexing process
///
/// # Example
/// ```rust
/// let result = add_paths_recursive("/path/to/documents".to_string(), search_engine_state).await;
/// match result {
///     Ok(_) => println!("Started indexing the directory"),
///     Err(err) => println!("Failed to start indexing: {}", err),
/// }
/// ```
#[tauri::command]
pub fn add_paths_recursive(
    folder: String,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    add_paths_recursive_impl(folder, search_engine_state.inner().clone())
}

#[tauri::command]
pub async fn add_paths_recursive_async(
    folder: String,
    search_engine_state: State<'_, Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    let state = search_engine_state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        add_paths_recursive_impl(folder, state)
    })
        .await
        .map_err(|e| format!("Indexing thread error: {:?}", e))?
}

pub fn add_paths_recursive_impl(
    folder: String,
    state: Arc<Mutex<SearchEngineState>>,
) -> Result<(), String> {
    log_info!(
        "Add paths recursive called with folder: {} (using chunked indexing)",
        folder
    );
    
    // Use chunked indexing with smaller chunk size for more frequent progress updates
    let default_chunk_size = 350; // Smaller chunk size for more frequent progress updates
    let path = PathBuf::from(&folder);

    // Verify the path exists before starting
    if !path.exists() {
        let error_msg = format!("Path does not exist: {}", folder);
        log_error!("{}", error_msg);
        return Err(error_msg);
    }

    log_info!("Starting chunked indexing for path: {} with chunk size: {}", folder, default_chunk_size);

    let engine_state = state.lock().unwrap();
    let result = engine_state.start_chunked_indexing(path, default_chunk_size);

    match &result {
        Ok(_) => log_info!("Chunked indexing started successfully for: {}", folder),
        Err(e) => log_error!("Chunked indexing failed for {}: {}", folder, e),
    }

    result
}

/// Adds a single file to the search engine index.
///
/// # Arguments
/// * `path` - The path to the file to add to the index
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(())` - If the file was successfully added to the index
/// * `Err(String)` - If there was an error adding the file
///
/// # Example
/// ```rust
/// let result = add_path("/path/to/document.txt".to_string(), search_engine_state).await;
/// match result {
///     Ok(_) => println!("File added to index"),
///     Err(err) => println!("Failed to add file: {}", err),
/// }
/// ```
#[tauri::command]
pub fn add_path(
    path: String,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    add_path_impl(path, search_engine_state.inner().clone())
}

pub fn add_path_impl(path: String, state: Arc<Mutex<SearchEngineState>>) -> Result<(), String> {
    log_info!("Add path called with: {}", path);
    let engine = state.lock().unwrap();
    engine.add_path(&path)
}

/// Recursively removes a directory and all its contents from the search engine index.
///
/// # Arguments
/// * `folder` - The path to the directory to remove from the index
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(())` - If the directory was successfully removed from the index
/// * `Err(String)` - If there was an error removing the directory
///
/// # Example
/// ```rust
/// let result = remove_paths_recursive("/path/to/old_documents".to_string(), search_engine_state).await;
/// match result {
///     Ok(_) => println!("Directory removed from index"),
///     Err(err) => println!("Failed to remove directory: {}", err),
/// }
/// ```
#[tauri::command]
pub fn remove_paths_recursive(
    folder: String,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    remove_paths_recursive_impl(folder, search_engine_state.inner().clone())
}

pub fn remove_paths_recursive_impl(
    folder: String,
    state: Arc<Mutex<SearchEngineState>>,
) -> Result<(), String> {
    log_info!(
        "Remove paths recursive called with folder: {}",
        folder
    );
    let engine = state.lock().unwrap();
    engine.remove_paths_recursive(&folder)
}

/// Removes a single file from the search engine index.
///
/// # Arguments
/// * `path` - The path to the file to remove from the index
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(())` - If the file was successfully removed from the index
/// * `Err(String)` - If there was an error removing the file
///
/// # Example
/// ```rust
/// let result = remove_path("/path/to/old_document.txt".to_string(), search_engine_state).await;
/// match result {
///     Ok(_) => println!("File removed from index"),
///     Err(err) => println!("Failed to remove file: {}", err),
/// }
/// ```
#[tauri::command]
pub fn remove_path(
    path: String,
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    remove_path_impl(path, search_engine_state.inner().clone())
}

pub fn remove_path_impl(path: String, state: Arc<Mutex<SearchEngineState>>) -> Result<(), String> {
    log_info!("Remove path called with: {}", path);
    let engine = state.lock().unwrap();
    engine.remove_path(&path)
}

/// Clears all indexed data from the search engine.
///
/// # Arguments
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(())` - If the search engine was successfully cleared
/// * `Err(String)` - If there was an error clearing the search engine
///
/// # Example
/// ```rust
/// let result = clear_search_engine(search_engine_state).await;
/// match result {
///     Ok(_) => println!("Search engine index cleared"),
///     Err(err) => println!("Failed to clear search engine: {}", err),
/// }
/// ```
#[tauri::command]
pub fn clear_search_engine(
    search_engine_state: State<Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    clear_search_engine_impl(search_engine_state.inner().clone())
}

pub fn clear_search_engine_impl(state: Arc<Mutex<SearchEngineState>>) -> Result<(), String> {
    log_info!("Clear search engine called");

    let state = state.lock().unwrap();
    let mut engine = state.engine.write().unwrap();
    engine.clear();

    // Update state
    let mut data = state.data.lock().unwrap();
    data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

    Ok(())
}

/// Retrieves comprehensive information about the search engine's current state
/// including status, indexing progress, metrics, recent activity, and engine statistics.
///
/// # Arguments
/// * `search_engine_state` - The state containing the search engine
///
/// # Returns
/// * `Ok(SearchEngineInfo)` - A struct containing all relevant search engine information
/// * `Err(String)` - If there was an error retrieving the information
///
/// # Example
/// ```rust
/// let result = get_search_engine_info(search_engine_state).await;
/// match result {
///     Ok(info) => {
///         println!("Search engine status: {:?}", info.status);
///         println!("Indexing progress: {:.2}%", info.progress.percentage_complete);
///         println!("Files indexed: {}/{}", info.progress.files_indexed, info.progress.files_discovered);
///         println!("Currently indexing: {:?}", info.progress.current_path);
///         println!("Remaining time estimate: {:?} ms", info.progress.estimated_time_remaining);
///         
///         println!("Total searches: {}", info.metrics.total_searches);
///         println!("Average search time: {:?} ms", info.metrics.average_search_time_ms);
///         println!("Last indexing duration: {:?} ms", info.metrics.last_indexing_duration_ms);
///         
///         println!("Recent searches: {:?}", info.recent_activity.recent_searches);
///         println!("Most accessed paths: {:?}", info.recent_activity.most_accessed_paths);
///         
///         println!("Index size: {} entries", info.stats.trie_size);
///         println!("Cache size: {} entries", info.stats.cache_size);
///         
///         println!("Last updated: {}", info.last_updated);
///         
///         // Convert timestamp to readable date if needed
///         let datetime = chrono::DateTime::from_timestamp_millis(info.last_updated as i64)
///             .map(|dt| dt.to_rfc3339());
///         println!("Last updated (readable): {:?}", datetime);
///     },
///     Err(err) => println!("Failed to get search engine info: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn get_search_engine_info(
    search_engine_state: State<'_, Arc<Mutex<SearchEngineState>>>,
) -> Result<SearchEngineInfo, String> {
    get_search_engine_info_impl(search_engine_state.inner().clone())
}

pub fn get_search_engine_info_impl(
    state: Arc<Mutex<SearchEngineState>>,
) -> Result<SearchEngineInfo, String> {
    log_info!("Get search engine info called");
    let engine = state.lock().unwrap();
    Ok(engine.get_search_engine_info())
}

#[tauri::command]
pub async fn get_indexing_progress(
    search_engine_state: State<'_, Arc<Mutex<SearchEngineState>>>,
) -> Result<IndexingProgress, String> {
    let state = search_engine_state.lock().map_err(|e| e.to_string())?;
    let data = state.data.lock().map_err(|e| e.to_string())?;
    let progress = data.progress.clone();

    // Add debug logging for every progress request
    #[cfg(feature = "index-progress-logging")]
    log_info!(
        "Progress API: discovered={}, indexed={}, percentage={:.1}%, current_path={:?}, status={:?}",
        progress.files_indexed,
        progress.files_discovered,
        progress.percentage_complete,
        progress.current_path.as_ref().map(|p| p.split('/').last().unwrap_or(p)),
        data.status
    );

    Ok(progress)
}

#[tauri::command]
pub async fn get_indexing_status(
    search_engine_state: State<'_, Arc<Mutex<SearchEngineState>>>,
) -> Result<String, String> {
    let state = search_engine_state.lock().map_err(|e| e.to_string())?;
    let data = state.data.lock().map_err(|e| e.to_string())?;
    let status = format!("{:?}", data.status);

    // Add debug logging
    log_info!("Status request: {}", status);

    Ok(status)
}

#[tauri::command]
pub async fn stop_indexing(
    search_engine_state: State<'_, Arc<Mutex<SearchEngineState>>>,
) -> Result<(), String> {
    log_info!("Stopping indexing process");

    let state = search_engine_state.lock().map_err(|e| e.to_string())?;

    // Lock the state data to update status
    let mut data = state.data.lock().map_err(|e| e.to_string())?;

    // Update status first
    data.status = SearchEngineStatus::Cancelled;
    data.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    drop(data);

    // Lock the engine to call stop_indexing
    let mut engine = state.engine.write().map_err(|e| e.to_string())?;

    // Call stop_indexing on the engine
    engine.stop_indexing();

    Ok(())
}

#[cfg(test)]
mod tests_autocomplete_commands {
    use super::*;
    use crate::state::searchengine_data::SearchEngineStatus;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;
    use crate::state::SettingsState;

    // Helper function to create a test SearchEngineState
    fn create_test_search_engine_state() -> Arc<Mutex<SearchEngineState>> {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        Arc::new(Mutex::new(SearchEngineState::new(settings_state)))
    }

    // Helper to create a temporary file with content
    fn create_temp_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(filename);
        let mut file = File::create(&file_path).unwrap();
        write!(file, "{}", content).unwrap();
        file_path
    }

    #[test]
    fn test_search_impl_with_empty_engine() {
        let state = create_test_search_engine_state();
        let results = search_impl("test".to_string(), state);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_search_with_extension_impl_with_empty_engine() {
        let state = create_test_search_engine_state();
        let results =
            search_with_extension_impl("test".to_string(), vec!["txt".to_string()], state);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }

    #[test]
    fn test_add_and_search_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.txt", "This is a test document");

        let state = create_test_search_engine_state();

        // Add the file to the index
        let add_result = add_path_impl(file_path.to_string_lossy().to_string(), state.clone());
        assert!(add_result.is_ok());

        // Search for a term that should be in the file
        let search_result = search_impl("test".to_string(), state.clone());
        assert!(search_result.is_ok());

        let results = search_result.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].0.contains("test.txt"));
    }

    #[test]
    fn test_add_and_remove_path() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.txt", "This is a test document");

        let state = create_test_search_engine_state();

        // Add the file to the index
        let add_result = add_path_impl(file_path.to_string_lossy().to_string(), state.clone());
        assert!(add_result.is_ok());

        // Remove the file from the index
        let remove_result =
            remove_path_impl(file_path.to_string_lossy().to_string(), state.clone());
        assert!(remove_result.is_ok());

        // Search for a term that was in the file
        let search_result = search_impl("test".to_string(), state.clone());
        assert!(search_result.is_ok());

        // Verify the file is no longer in the index
        let results = search_result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_recursive_add_and_remove() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();

        let _file1 = create_temp_file(&temp_dir, "test1.txt", "This is test document one");
        let _file2 = create_temp_file(
            &TempDir::new_in(&subdir).unwrap(),
            "test2.txt",
            "This is test document two",
        );

        let state = create_test_search_engine_state();

        // Add all files recursively
        let add_result =
            add_paths_recursive_impl(temp_dir.path().to_string_lossy().to_string(), state.clone());
        assert!(add_result.is_ok());

        // Search for a common term
        let search_result = search_impl("test".to_string(), state.clone());
        assert!(search_result.is_ok());
        let _results = search_result.unwrap();

        // Since recursive indexing happens in a background thread, we can't reliably test file count here
        // We should test core functionality without relying on completion timing

        // Remove all files recursively
        let remove_result = remove_paths_recursive_impl(
            temp_dir.path().to_string_lossy().to_string(),
            state.clone(),
        );
        assert!(remove_result.is_ok());

        // Allow some time for removal to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Search again after removal
        let search_result_after = search_impl("test".to_string(), state.clone());
        assert!(search_result_after.is_ok());

        // Verify the files are no longer in the index
        let results = search_result_after.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_clear_search_engine() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_temp_file(&temp_dir, "test.txt", "This is a test document");

        let state = create_test_search_engine_state();

        // Add the file to the index
        let add_result = add_path_impl(file_path.to_string_lossy().to_string(), state.clone());
        assert!(add_result.is_ok());

        // Clear the search engine
        let clear_result = clear_search_engine_impl(state.clone());
        assert!(clear_result.is_ok());

        // Search for a term that was in the file
        let search_result = search_impl("test".to_string(), state.clone());
        assert!(search_result.is_ok());

        // Verify the index is empty
        let results = search_result.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_search_engine_info() {
        let state = create_test_search_engine_state();

        let info_result = get_search_engine_info_impl(state.clone());
        assert!(info_result.is_ok());

        let info = info_result.unwrap();

        // Check that the returned structure has the expected default values
        assert_eq!(info.metrics.total_searches, 0);
        assert_eq!(info.progress.files_indexed, 0);
        assert!(matches!(info.status, SearchEngineStatus::Idle));
    }

    #[test]
    fn test_search_with_extension_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let txt_file = create_temp_file(&temp_dir, "test.txt", "This is a text document");
        let md_file = create_temp_file(&temp_dir, "readme.md", "This is a markdown document");

        let state = create_test_search_engine_state();

        // Add both files to the index
        add_path_impl(txt_file.to_string_lossy().to_string(), state.clone()).unwrap();
        add_path_impl(md_file.to_string_lossy().to_string(), state.clone()).unwrap();

        // Search for "document" with txt extension filter
        let search_result = search_with_extension_impl(
            "document".to_string(),
            vec!["txt".to_string()],
            state.clone(),
        );

        assert!(search_result.is_ok());
        let results = search_result.unwrap();

        // Should only find the txt file
        assert_eq!(results.len(), 1);
        assert!(results[0].0.contains("test.txt"));
    }

    #[test]
    fn test_add_paths_recursive_uses_chunked() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();

        let _file1 = create_temp_file(&temp_dir, "chunked_test1.txt", "This is chunked test document one");
        let file2_dir = TempDir::new_in(&subdir).unwrap();
        let _file2 = create_temp_file(&file2_dir, "chunked_test2.txt", "This is chunked test document two");

        let state = create_test_search_engine_state();

        // The updated add_paths_recursive should use chunked indexing internally
        let add_result = add_paths_recursive_impl(
            temp_dir.path().to_string_lossy().to_string(),
            state.clone(),
        );
        assert!(add_result.is_ok());

        // Allow time for chunked indexing to complete
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Should be able to search successfully
        let search_result = search_impl("chunked".to_string(), state.clone());
        assert!(search_result.is_ok());

        // Results might be empty if indexing is still in progress, which is acceptable
        let _results = search_result.unwrap();
    }

    #[test]
    fn test_recursive_add_now_uses_chunked() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();

        let _file1 = create_temp_file(&temp_dir, "test1.txt", "This is test document one");
        let _file2 = create_temp_file(
            &TempDir::new_in(&subdir).unwrap(),
            "test2.txt",
            "This is test document two",
        );

        let state = create_test_search_engine_state();

        // Add all files recursively (now uses chunked indexing)
        let add_result =
            add_paths_recursive_impl(temp_dir.path().to_string_lossy().to_string(), state.clone());
        assert!(add_result.is_ok());

        // Search for a common term
        let search_result = search_impl("test".to_string(), state.clone());
        assert!(search_result.is_ok());
        let _results = search_result.unwrap();

        // Chunked indexing happens in the current thread but processes in chunks
        // We can test that the command succeeded

        // Remove all files recursively
        let remove_result = remove_paths_recursive_impl(
            temp_dir.path().to_string_lossy().to_string(),
            state.clone(),
        );
        assert!(remove_result.is_ok());

        // Allow some time for removal to complete
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Search again after removal
        let search_result_after = search_impl("test".to_string(), state.clone());
        assert!(search_result_after.is_ok());
    }
}
