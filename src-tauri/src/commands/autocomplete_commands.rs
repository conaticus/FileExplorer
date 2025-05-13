use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use serde::{Deserialize, Serialize};

use crate::state::searchengine_data::{SearchEngineState, EngineStatsSerializable, SearchEngineInfo};
use crate::log_info;

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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<SearchResult, String> {
    log_info!(&format!("Search command called with query: {}", query));
    search_engine_state.search(&query)
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<SearchResult, String> {
    log_info!(&format!("Search with extension called: query='{}', extensions={:?}", query, extensions));
    search_engine_state.search_by_extension(&query, extensions)
}

/// Recursively adds all files from a directory to the search engine index.
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<(), String> {
    log_info!(&format!("Add paths recursive called with folder: {}", folder));
    let path = PathBuf::from(folder);
    search_engine_state.start_indexing(path)
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<(), String> {
    log_info!(&format!("Add path called with: {}", path));
    search_engine_state.add_path(&path)
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<(), String> {
    log_info!(&format!("Remove paths recursive called with folder: {}", folder));
    search_engine_state.remove_paths_recursive(&folder)
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<(), String> {
    log_info!(&format!("Remove path called with: {}", path));
    search_engine_state.remove_path(&path)
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
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<(), String> {
    log_info!("Clear search engine called");
    
    // Get engine lock and clear it
    let mut engine = search_engine_state.engine.lock().unwrap();
    engine.clear();
    
    // Update state
    let mut data = search_engine_state.data.lock().unwrap();
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
pub fn get_search_engine_info(
    search_engine_state: State<Arc<SearchEngineState>>
) -> Result<SearchEngineInfo, String> {
    log_info!("Get search engine info called");
    Ok(search_engine_state.get_search_engine_info())
}
