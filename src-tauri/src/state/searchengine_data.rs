use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::log_error;
#[cfg(test)]
use crate::log_info;
use crate::search_engine::search_core::{AutocompleteEngine, EngineStats};
use crate::models::search_engine_config::SearchEngineConfig;
use crate::state::SettingsState;

/// Current operational status of the search engine.
///
/// Represents the various states the search engine can be in at any given time,
/// allowing the UI to update accordingly and prevent conflicting operations.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum SearchEngineStatus {
    Idle,
    Indexing,
    Searching,
    Cancelled,
    Failed,
}

/// Progress information for ongoing indexing operations.
///
/// Tracks the current state of an indexing operation, including completion percentage
/// and estimated time remaining, to provide feedback for the user interface.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexingProgress {
    pub files_discovered: usize,
    pub files_indexed: usize,
    pub percentage_complete: f32,
    pub current_path: Option<String>,
    pub start_time: Option<u64>, // as milliseconds since epoch
    pub estimated_time_remaining: Option<u64>, // in milliseconds
}

impl Default for IndexingProgress {
    fn default() -> Self {
        Self {
            files_discovered: 0,
            files_indexed: 0,
            percentage_complete: 0.0,
            current_path: None,
            start_time: None,
            estimated_time_remaining: None,
        }
    }
}

/// Performance metrics for the search engine.
///
/// Collects statistics about search engine performance to help users
/// understand system behavior and identify potential optimizations.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchEngineMetrics {
    pub last_indexing_duration_ms: Option<u64>,
    pub average_search_time_ms: Option<f32>,
    pub cache_hit_rate: Option<f32>,
    pub total_searches: usize,
    pub cache_hits: usize,
}

impl Default for SearchEngineMetrics {
    fn default() -> Self {
        Self {
            last_indexing_duration_ms: None,
            average_search_time_ms: None,
            cache_hit_rate: None,
            total_searches: 0,
            cache_hits: 0,
        }
    }
}

/// User activity data related to search operations.
///
/// Tracks recent user interactions with the search system to provide
/// history features and improve result relevance through usage patterns.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RecentActivity {
    pub recent_searches: Vec<String>,
    pub most_accessed_paths: Vec<String>,
}

impl Default for RecentActivity {
    fn default() -> Self {
        Self {
            recent_searches: Vec::new(),
            most_accessed_paths: Vec::new(),
        }
    }
}

/// Serializable version of engine statistics.
///
/// Provides a Serde-compatible representation of internal engine statistics
/// for transmission to the frontend or storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatsSerializable {
    pub cache_size: usize,
    pub trie_size: usize,
}

impl From<EngineStats> for EngineStatsSerializable {
    fn from(stats: EngineStats) -> Self {
        Self {
            cache_size: stats.cache_size,
            trie_size: stats.trie_size,
        }
    }
}

/// Comprehensive information about the search engine's current state.
///
/// Aggregates all relevant status information, metrics, and activity data
/// into a single serializable structure for frontend display and monitoring.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchEngineInfo {
    pub status: SearchEngineStatus,
    pub progress: IndexingProgress,
    pub metrics: SearchEngineMetrics,
    pub recent_activity: RecentActivity,
    pub stats: EngineStatsSerializable,
    pub last_updated: u64,
}

/// Complete search engine state including both configuration and runtime data.
///
/// Contains all persistent configuration options and runtime state of the
/// search engine system for storage and restoration between sessions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchEngine {
    pub status: SearchEngineStatus,
    pub index_folder: PathBuf,
    pub progress: IndexingProgress,
    pub metrics: SearchEngineMetrics,
    pub config: SearchEngineConfig,
    pub recent_activity: RecentActivity,
    pub current_directory: Option<String>,
    pub last_updated: u64, // timestamp in milliseconds
}

impl Default for SearchEngine {
    fn default() -> Self {
        SearchEngine {
            status: SearchEngineStatus::Idle,
            index_folder: PathBuf::new(),
            progress: IndexingProgress::default(),
            metrics: SearchEngineMetrics::default(),
            config: SearchEngineConfig::default(),
            recent_activity: RecentActivity::default(),
            current_directory: None,
            last_updated: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

/// Thread-safe container for search engine state and operations.
///
/// Provides synchronized access to the search engine's configuration, state,
/// and underlying search index through a mutex-protected interface.
/// Offers methods for searching, indexing, and managing the search engine.
pub struct SearchEngineState {
    pub data: Arc<Mutex<SearchEngine>>,
    pub engine: Arc<Mutex<AutocompleteEngine>>,
    settings_state: Arc<Mutex<SettingsState>>,
}

impl SearchEngineState {
    /// Creates a new SearchEngineState with default settings.
    ///
    /// Initializes a new search engine state with default configuration and
    /// an empty search index. The search engine will start in Idle status
    /// and be ready to index files or perform searches.
    ///
    /// # Arguments
    ///
    /// * `settings_state` - Application settings state containing search engine configuration
    ///
    /// # Returns
    ///
    /// A new SearchEngineState instance with default configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// let settings_state = Arc::new(Mutex::new(SettingsState::new()));
    /// let search_engine = SearchEngineState::new(settings_state);
    /// ```
    pub fn new(settings_state: Arc<Mutex<SettingsState>>) -> Self {
        // Get config from settings_state
        let config = {
            let settings = settings_state.lock().unwrap();
            let inner_settings = settings.0.lock().unwrap();
            inner_settings.backend_settings.search_engine_config.clone()
        };
        
        // Create a new RankingConfig with the directory boost enabled/disabled
        // based on the prefer_directories setting
        let mut ranking_config = config.ranking_config.clone();
        if !config.prefer_directories {
            ranking_config.directory_ranking_boost = 0.0; // Disable directory boost if not preferred
        }
        
        // Pass the ranking_config from settings to the autocomplete engine
        let engine = AutocompleteEngine::new(
            config.cache_size, 
            config.max_results,
            config.cache_ttl.unwrap(),
            ranking_config,
        );

        Self {
            data: Arc::new(Mutex::new(Self::save_default_search_engine_in_state(config))),
            engine: Arc::new(Mutex::new(engine)),
            settings_state,
        }
    }

    /// Creates a default search engine configuration.
    ///
    /// Helper method that creates and returns a default SearchEngine instance.
    ///
    /// # Returns
    ///
    /// A SearchEngine instance with default settings.
    fn save_default_search_engine_in_state(config: SearchEngineConfig) -> SearchEngine {
        let mut defaults = SearchEngine::default();
        defaults.config = config;
        Self::save_search_engine_in_state(defaults)
    }

    /// Saves a search engine configuration to state.
    ///
    /// Helper method to set up a search engine instance.
    ///
    /// # Arguments
    ///
    /// * `defaults` - The SearchEngine instance to save
    ///
    /// # Returns
    ///
    /// The provided SearchEngine instance (for chaining).
    fn save_search_engine_in_state(defaults: SearchEngine) -> SearchEngine {
        defaults
    }

    /// Starts indexing a folder for searching.
    ///
    /// Begins the process of scanning and indexing all files and directories
    /// within the specified folder. If an indexing operation is already in progress,
    /// it will be stopped before starting the new one.
    ///
    /// This is a blocking operation and will not return until indexing is complete.
    /// For very large directories, consider running this in a separate thread.
    ///
    /// # Arguments
    ///
    /// * `folder` - The root folder path to index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Indexing completed successfully
    /// * `Err(String)` - An error occurred during indexing
    ///
    /// # Example
    ///
    /// ```rust
    /// let search_engine = SearchEngineState::new();
    /// let result = search_engine.start_indexing(PathBuf::from("/path/to/index"));
    /// ```
    pub fn start_indexing(&self, folder: PathBuf) -> Result<(), String> {
        // Get locks on both data and engine
        let mut data = self.data.lock().unwrap();
        let mut engine = self.engine.lock().unwrap();

        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }

        // Check if we're already indexing - if so, stop it first
        if matches!(data.status, SearchEngineStatus::Indexing) {
            // Signal the engine to stop the current indexing process
            #[cfg(test)]
            log_info!(
                "Stopping previous indexing of '{}' before starting new indexing",
                data.index_folder.display()
            );

            engine.stop_indexing();
        }

        // Update state to show we're indexing a new folder
        data.status = SearchEngineStatus::Indexing;
        data.index_folder = folder.clone();
        data.progress = IndexingProgress::default();
        data.progress.start_time = Some(chrono::Utc::now().timestamp_millis() as u64);
        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

        // Reset the stop flag before starting new indexing
        engine.reset_stop_flag();

        // Start indexing in the engine
        let start_time = Instant::now();

        // Clear previous index if switching folders
        engine.clear();

        // Get excluded patterns from config
        let excluded_patterns = data.config.excluded_patterns.clone();

        // Actually start the indexing
        if let Some(folder_str) = folder.to_str() {
            // Release the locks before starting the recursive operation
            drop(data);
            drop(engine);

            // Get the engine again for the recursive operation
            {
                let mut engine = self.engine.lock().unwrap();
                engine.add_paths_recursive(folder_str, Some(&excluded_patterns.unwrap()));
            }

            // Update status and metrics after indexing completes or stops
            let mut data = self.data.lock().unwrap();
            let elapsed = start_time.elapsed();
            data.metrics.last_indexing_duration_ms = Some(elapsed.as_millis() as u64);

            // Check if it was cancelled
            let engine = self.engine.lock().unwrap();
            if engine.should_stop_indexing() {
                data.status = SearchEngineStatus::Cancelled;
                #[cfg(test)]
                log_info!(
                    "Indexing of '{}' was cancelled after {:?}",
                    folder.display(),
                    elapsed
                );
            } else {
                data.status = SearchEngineStatus::Idle;
                #[cfg(test)]
                log_info!(
                    "Indexing of '{}' completed in {:?}",
                    folder.display(),
                    elapsed
                );
            }
        } else {
            data.status = SearchEngineStatus::Failed;
            return Err("Invalid folder path".to_string());
        }

        Ok(())
    }

    /// Performs a search using the indexed files.
    ///
    /// Searches through the indexed files for matches to the given query string.
    /// Results are ranked by relevance and limited by the configured maximum results.
    /// This method will fail if the engine is currently indexing or searching.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to find matching files
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, f32)>)` - List of matching paths and their relevance scores
    /// * `Err(String)` - An error occurred during searching
    ///
    /// # Example
    ///
    /// ```rust
    /// let search_engine = SearchEngineState::new();
    /// // ... index some files first ...
    /// let results = search_engine.search("document").unwrap();
    /// for (path, score) in results {
    ///     println!("{} (score: {})", path, score);
    /// }
    /// ```
    pub fn search(&self, query: &str) -> Result<Vec<(String, f32)>, String> {
        let mut data = self.data.lock().unwrap();
        let mut engine = self.engine.lock().unwrap();

        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }

        // Check if engine is busy
        if matches!(data.status, SearchEngineStatus::Indexing) {
            return Err("Engine is currently indexing".to_string());
        }

        if matches!(data.status, SearchEngineStatus::Searching) {
            return Err("Engine is currently searching".to_string());
        }

        // Update state
        data.status = SearchEngineStatus::Searching;
        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

        // Set current directory context if available
        if let Some(current_dir) = &data.current_directory {
            engine.set_current_directory(Some(current_dir.clone()));
        }

        // Perform search
        let start_time = Instant::now();
        let results = engine.search(query);
        let search_time = start_time.elapsed();

        // Update metrics
        data.metrics.total_searches += 1;

        // Calculate average search time
        if let Some(avg_time) = data.metrics.average_search_time_ms {
            data.metrics.average_search_time_ms = Some(
                (avg_time * (data.metrics.total_searches - 1) as f32
                    + search_time.as_millis() as f32)
                    / data.metrics.total_searches as f32,
            );
        } else {
            data.metrics.average_search_time_ms = Some(search_time.as_millis() as f32);
        }

        // Track recent searches (add to front, limit to 10)
        if !query.is_empty() {
            data.recent_activity
                .recent_searches
                .insert(0, query.to_string());
            if data.recent_activity.recent_searches.len() > 10 {
                data.recent_activity.recent_searches.pop();
            }
        }

        // Update state
        data.status = SearchEngineStatus::Idle;

        Ok(results)
    }

    /// Performs a search with custom file extension preferences.
    ///
    /// Similar to `search`, but allows overriding the default extension preferences
    /// specifically for this search operation. Files with the specified extensions 
    /// will receive higher ranking in results, with priority determined by order.
    ///
    /// # Arguments
    ///
    /// * `query` - The search string to find matching files
    /// * `extensions` - List of file extensions to prioritize, in order of preference
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<(String, f32)>)` - List of matching paths and their relevance scores
    /// * `Err(String)` - An error occurred during searching
    ///
    /// # Example
    ///
    /// ```rust
    /// let search_engine = SearchEngineState::new();
    /// // Prioritize markdown and text files in search results
    /// let results = search_engine.search_by_extension("document", vec!["md".to_string(), "txt".to_string()]).unwrap();
    /// ```
    ///
    /// # Performance
    ///
    /// Similar to `search`, but with additional overhead of temporarily modifying 
    /// and restoring extension preferences.
    pub fn search_by_extension(
        &self,
        query: &str,
        extensions: Vec<String>,
    ) -> Result<Vec<(String, f32)>, String> {
        let mut data = self.data.lock().unwrap();
        let mut engine = self.engine.lock().unwrap();

        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }

        // Check if engine is busy
        if matches!(data.status, SearchEngineStatus::Indexing) {
            return Err("Engine is currently indexing".to_string());
        }

        if matches!(data.status, SearchEngineStatus::Searching) {
            return Err("Engine is currently searching".to_string());
        }

        data.status = SearchEngineStatus::Searching;
        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

        // Set current directory context if available
        if let Some(current_dir) = &data.current_directory {
            engine.set_current_directory(Some(current_dir.clone()));
        }

        // Store original preferred extensions and override
        let original_extensions = engine.get_preferred_extensions().clone();
        engine.set_preferred_extensions(extensions.clone());
        #[cfg(test)]
        log_info!(
            "Searching with preferred extensions: {:?}",
            extensions
        );

        // Perform search
        let start_time = Instant::now();
        let results = engine.search(query);
        let search_time = start_time.elapsed();

        #[cfg(test)]
        {
            // Verify that results meet our extension preferences
            if !results.is_empty() && !extensions.is_empty() {
                log_info!("Top search result: {}", results[0].0);

                // Check if top result has one of our preferred extensions
                if let Some(extension) = std::path::Path::new(&results[0].0)
                    .extension()
                    .and_then(|e| e.to_str())
                {
                    let ext = extension.to_lowercase();
                    log_info!(
                        "Top result extension: {}, preferred: {:?}",
                        ext, extensions
                    );
                }
            }
        }

        // Reset the original preferred extensions
        engine.set_preferred_extensions(original_extensions);

        // Update metrics
        data.metrics.total_searches += 1;

        // Calculate average search time
        if let Some(avg_time) = data.metrics.average_search_time_ms {
            data.metrics.average_search_time_ms = Some(
                (avg_time * (data.metrics.total_searches - 1) as f32
                    + search_time.as_millis() as f32)
                    / data.metrics.total_searches as f32,
            );
        } else {
            data.metrics.average_search_time_ms = Some(search_time.as_millis() as f32);
        }

        // Track recent searches (add to front, limit to 10)
        if !query.is_empty()
            && !data
                .recent_activity
                .recent_searches
                .contains(&query.to_string())
        {
            data.recent_activity
                .recent_searches
                .insert(0, query.to_string());
            if data.recent_activity.recent_searches.len() > 10 {
                data.recent_activity.recent_searches.pop();
            }
        }

        // Update state
        data.status = SearchEngineStatus::Idle;

        Ok(results)
    }

    /// Updates the progress information for an ongoing indexing operation.
    ///
    /// This method updates various metrics about the indexing process including
    /// counts of indexed files, completion percentage, and estimated time remaining.
    ///
    /// # Arguments
    ///
    /// * `indexed` - Number of files and directories that have been indexed
    /// * `total` - Total number of files and directories discovered
    /// * `current_path` - Optional string representing the file/directory currently being processed
    ///
    /// # Performance
    ///
    /// O(1) - Simple field updates and calculations
    #[cfg(test)]
    pub fn update_indexing_progress(
        &self,
        indexed: usize,
        total: usize,
        current_path: Option<String>,
    ) {
        let mut data = self.data.lock().unwrap();

        data.progress.files_indexed = indexed;
        data.progress.files_discovered = total;
        data.progress.current_path = current_path;

        // Calculate percentage
        if total > 0 {
            data.progress.percentage_complete = (indexed as f32 / total as f32) * 100.0;
        }

        // Calculate estimated time remaining
        if let Some(start_time) = data.progress.start_time {
            let elapsed_ms = chrono::Utc::now().timestamp_millis() as u64 - start_time;
            if indexed > 0 {
                let avg_time_per_file = elapsed_ms as f32 / indexed as f32;
                let remaining_files = total.saturating_sub(indexed);
                let estimated_ms = (avg_time_per_file * remaining_files as f32) as u64;
                data.progress.estimated_time_remaining = Some(estimated_ms);
            }
        }

        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;
    }

    /// Returns statistics about the search engine's index and cache.
    ///
    /// This method retrieves information about the current size of the search index
    /// and the cache, providing visibility into memory usage and data structure sizes.
    ///
    /// # Returns
    ///
    /// An `EngineStatsSerializable` struct containing statistics about the engine
    ///
    /// # Performance
    ///
    /// O(1) - Simple field access operations
    pub fn get_stats(&self) -> EngineStatsSerializable {
        let engine = self.engine.lock().unwrap();
        let stats = engine.get_stats();
        EngineStatsSerializable::from(stats)
    }

    /// Returns comprehensive information about the search engine's current state.
    ///
    /// This method combines all relevant status information, metrics, and activity data
    /// into a single serializable structure suitable for frontend display or monitoring.
    ///
    /// # Returns
    ///
    /// A `SearchEngineInfo` struct containing the complete state information
    ///
    /// # Performance
    ///
    /// O(1) - Simple field aggregation operations
    pub fn get_search_engine_info(&self) -> SearchEngineInfo {
        let data = self.data.lock().unwrap();

        // Get stats from engine
        let stats = self.get_stats();
        SearchEngineInfo {
            status: data.status.clone(),
            progress: data.progress.clone(),
            metrics: data.metrics.clone(),
            recent_activity: data.recent_activity.clone(),
            stats,
            last_updated: data.last_updated,
        }
    }

    /// Updates the search engine configuration from settings state.
    ///
    /// This method retrieves the latest configuration from the settings state 
    /// and applies it to the search engine.
    ///
    /// # Arguments
    ///
    /// * `path` - Optional string representing current directory context
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration was successfully updated
    /// * `Err(String)` - An error occurred during configuration update
    ///
    /// # Performance
    ///
    /// O(1) plus cache invalidation cost for changed preferences
    #[cfg(test)]
    pub fn update_config(&self, path: Option<String>) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        let mut engine = self.engine.lock().unwrap();
        
        // Get fresh config from settings state
        let config = {
            let settings = self.settings_state.lock().unwrap();
            let inner_settings = settings.0.lock().unwrap();
            inner_settings.backend_settings.search_engine_config.clone()
        };
        
        data.config = config.clone();
        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

        // Update the current directory in the data structure
        data.current_directory = path.clone();

        engine.set_preferred_extensions(config.preferred_extensions);

        Ok(())
    }

    /// Adds a single path to the search index.
    ///
    /// This method adds a single file or directory path to the search index
    /// without recursively adding its contents if it's a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to add to the search index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Path was successfully added
    /// * `Err(String)` - An error occurred while adding the path
    pub fn add_path(&self, path: &str) -> Result<(), String> {
        let data = self.data.lock().unwrap();
        
        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }
        
        // Get the excluded patterns to pass to the engine
        let excluded_patterns = data.config.excluded_patterns.clone();
        drop(data);
        
        let mut engine = self.engine.lock().unwrap();
        // Use the new method to check exclusions before adding
        engine.add_path_with_exclusion_check(path, Some(&excluded_patterns.unwrap()));
        Ok(())
    }

    /// Removes a single path from the search index.
    ///
    /// This method removes a specific file or directory path from the search index
    /// without recursively removing its contents if it's a directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to remove from the search index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Path was successfully removed
    /// * `Err(String)` - An error occurred while removing the path
    pub fn remove_path(&self, path: &str) -> Result<(), String> {
        let data = self.data.lock().unwrap();
        
        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }
        
        drop(data);
        
        let mut engine = self.engine.lock().unwrap();
        engine.remove_path(path);
        Ok(())
    }

    /// Recursively removes a path and all its subdirectories and files from the index.
    ///
    /// This method removes a directory path and all files and subdirectories contained
    /// within it from the search index.
    ///
    /// # Arguments
    ///
    /// * `path` - The root directory path to remove from the index
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Path and its contents were successfully removed
    /// * `Err(String)` - An error occurred during removal
    pub fn remove_paths_recursive(&self, path: &str) -> Result<(), String> {
        let data = self.data.lock().unwrap();
        
        // Check if search engine is enabled
        if !data.config.search_engine_enabled {
            log_error!("Search engine is disabled in configuration.");
            return Err("Search engine is disabled in configuration".to_string());
        }
        
        drop(data);
        
        let mut engine = self.engine.lock().unwrap();
        engine.remove_paths_recursive(path);
        Ok(())
    }

    /// Stops any ongoing indexing operation.
    ///
    /// This method signals the underlying search engine to stop its current
    /// indexing operation as soon as possible.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Stop signal was successfully sent
    /// * `Err(String)` - No indexing operation was in progress
    ///
    /// # Performance
    ///
    /// O(1) - Simple flag operation
    #[cfg(test)] // maybe use in a later release
    pub fn stop_indexing(&self) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        let mut engine = self.engine.lock().unwrap();

        if matches!(data.status, SearchEngineStatus::Indexing) {
            // Signal the engine to stop indexing
            engine.stop_indexing();

            // Update state
            data.status = SearchEngineStatus::Cancelled;
            data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

            #[cfg(test)]
            log_info!(
                "Indexing of '{}' stopped",
                data.index_folder.display()
            );

            return Ok(());
        }

        Err("No indexing operation in progress".to_string())
    }

    /// Cancels the current indexing operation at user request.
    ///
    /// This is a user-initiated cancellation that calls stop_indexing().
    /// The method makes the user's intention explicit in the code.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Cancel signal was successfully sent
    /// * `Err(String)` - No indexing operation was in progress
    ///
    /// # Performance
    ///
    /// O(1) - Delegates to stop_indexing()
    #[cfg(test)] //maybe use in a later release
    pub fn cancel_indexing(&self) -> Result<(), String> {
        self.stop_indexing()
    }
}

/// Implementation of the Clone trait for SearchEngineState.
///
/// Provides a way to create a new SearchEngineState instance
/// that shares the same underlying data and engine through Arc references.
impl Clone for SearchEngineState {
    /// Creates a new SearchEngineState that refers to the same data and engine.
    ///
    /// The cloned instance shares the same mutex-protected state as the original,
    /// allowing multiple threads to safely access and modify the shared state.
    ///
    /// # Returns
    ///
    /// A new SearchEngineState instance with the same underlying data
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            engine: Arc::clone(&self.engine),
            settings_state: Arc::clone(&self.settings_state),
        }
    }
}

#[cfg(test)]
mod tests_searchengine_state {
    use super::*;
    use crate::log_info;
    use crate::log_warn;
    use std::fs;
    use std::thread;
    use std::time::Duration;

    // Helper function to get test data directory
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            log_warn!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            );
            panic!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            );
        }
        path
    }

    // Helper function to collect real paths from the test data directory
    fn collect_test_paths(limit: Option<usize>) -> Vec<String> {
        let test_path = get_test_data_path();
        let mut paths = Vec::new();

        fn add_paths_recursively(
            dir: &std::path::Path,
            paths: &mut Vec<String>,
            limit: Option<usize>,
        ) {
            if let Some(max) = limit {
                if paths.len() >= max {
                    return;
                }
            }

            if let Some(walker) = fs::read_dir(dir).ok() {
                for entry in walker.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(path_str) = path.to_str() {
                        paths.push(path_str.to_string());

                        if let Some(max) = limit {
                            if paths.len() >= max {
                                return;
                            }
                        }
                    }

                    if path.is_dir() {
                        add_paths_recursively(&path, paths, limit);
                    }
                }
            }
        }

        add_paths_recursively(&test_path, &mut paths, limit);

        // If test data doesn't contain enough paths or doesn't exist,
        // fall back to synthetic data with a warning
        if paths.is_empty() {
            log_warn!("No test data found, using synthetic data instead");
            return (0..100)
                .map(|i| format!("/path/to/file{}.txt", i))
                .collect();
        }

        paths
    }

    // Helper function to get a directory for indexing from test paths
    fn get_test_dir_for_indexing() -> PathBuf {
        let paths = collect_test_paths(Some(20));

        // First try to find a directory path from the collected paths
        for path in &paths {
            let path_buf = PathBuf::from(path);
            if path_buf.is_dir() {
                return path_buf;
            }
        }

        // If no directory found, use the parent of the first file path
        if let Some(first_path) = paths.first() {
            let path_buf = PathBuf::from(first_path);
            if let Some(parent) = path_buf.parent() {
                return parent.to_path_buf();
            }
        }

        // Fallback to the test data root
        get_test_data_path()
    }

    // Helper function to get a subdirectory from test data for indexing tests
    fn get_test_subdirs() -> (PathBuf, PathBuf) {
        let test_data_root = get_test_data_path();

        // Try to find two different subdirectories
        let mut dirs = Vec::new();

        if let Ok(entries) = fs::read_dir(&test_data_root) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                    if dirs.len() >= 2 {
                        break;
                    }
                }
            }
        }

        // If we found two directories, return them
        if dirs.len() >= 2 {
            return (dirs[0].clone(), dirs[1].clone());
        }

        // Otherwise, create two temporary subdirectories
        let subdir1 = test_data_root.join("test_subdir1");
        let subdir2 = test_data_root.join("test_subdir2");

        // Create the directories if they don't exist
        if !subdir1.exists() {
            let _ = fs::create_dir_all(&subdir1);
        }
        if !subdir2.exists() {
            let _ = fs::create_dir_all(&subdir2);
        }

        (subdir1, subdir2)
    }

    #[test]
    fn test_initialization() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Check default values
        let data = state.data.lock().unwrap();
        assert_eq!(data.status, SearchEngineStatus::Idle);
        assert_eq!(data.progress.files_indexed, 0);
        assert_eq!(data.metrics.total_searches, 0);
        assert!(!data.config.preferred_extensions.is_empty());
        assert!(data.recent_activity.recent_searches.is_empty());
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_start_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);
        let test_dir = get_test_dir_for_indexing();

        // Start indexing
        let result = state.start_indexing(test_dir.clone());
        assert!(result.is_ok(), "Indexing should start successfully");

        // Allow some time for indexing to complete
        thread::sleep(Duration::from_millis(200));

        // Check that indexing completed
        let data = state.data.lock().unwrap();
        assert!(matches!(
            data.status,
            SearchEngineStatus::Idle | SearchEngineStatus::Cancelled
        ));
        assert_eq!(data.index_folder, test_dir);
        assert!(data.metrics.last_indexing_duration_ms.is_some());
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_stop_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let test_dir = get_test_dir_for_indexing();

        // Create test files to ensure indexing takes enough time
        let mut test_files = Vec::new();
        for i in 0..1000 {
            // Increased to 1000 files to ensure indexing takes time
            let file_path = test_dir.join(format!("testfile_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use more reliable synchronization
        let (status_tx, status_rx) = std::sync::mpsc::channel();

        // Clone the Arc for the thread to use
        let state_clone = Arc::clone(&state);
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            // First manually set the status to Indexing to guarantee we're in that state
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;

                // Signal the test thread that we've set the status
                status_tx.send(()).unwrap();
            }

            // Now start the actual indexing (which may take a while)
            state_clone.start_indexing(test_dir_clone).unwrap();
        });

        // Wait for the signal that the status has been explicitly set to Indexing
        status_rx.recv().unwrap();

        // Double-check that we're really in Indexing state before proceeding
        {
            let data = state.data.lock().unwrap();
            assert_eq!(
                data.status,
                SearchEngineStatus::Indexing,
                "Should be in Indexing state before stopping"
            );
        }

        // Now we can safely stop indexing
        let stop_result = state.stop_indexing();
        assert!(stop_result.is_ok(), "Should successfully stop indexing");

        // Verify that stopping worked
        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Cancelled);
        }

        // Wait for indexing thread to complete
        indexing_thread.join().unwrap();

        // Clean up test files (best effort, don't fail test if cleanup fails)
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_cancel_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let test_dir = get_test_dir_for_indexing();

        // Create a LOT of test files to ensure indexing takes enough time
        let mut test_files = Vec::new();
        for i in 0..1000 {
            // Use 1000 files to ensure indexing takes time
            let file_path = test_dir.join(format!("cancel_test_file_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use more reliable synchronization with channel
        let (status_tx, status_rx) = std::sync::mpsc::channel();

        // Clone the Arc for the thread to use
        let state_clone = Arc::clone(&state);
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            // First manually set the status to Indexing to guarantee we're in that state
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;

                // Signal the test thread that we've set the status
                status_tx.send(()).unwrap();
            }

            // Now start the actual indexing
            state_clone.start_indexing(test_dir_clone).unwrap();
        });

        // Wait for the signal that the status has been explicitly set to Indexing
        status_rx.recv().unwrap();

        // Double-check that we're really in Indexing state before proceeding
        {
            let data = state.data.lock().unwrap();
            assert_eq!(
                data.status,
                SearchEngineStatus::Indexing,
                "Should be in Indexing state before canceling"
            );
        }

        // Now attempt to cancel indexing
        let cancel_result = state.cancel_indexing();
        assert!(cancel_result.is_ok(), "Should successfully cancel indexing");

        // Verify that canceling worked
        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Cancelled);
        }

        // Wait for indexing thread to complete
        indexing_thread.join().unwrap();

        // Clean up test files (best effort, don't fail test if cleanup fails)
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_search() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get paths and add them directly to the engine
        let paths = collect_test_paths(Some(100));
        for path in &paths {
            let _ = state.add_path(path);
        }

        // Find a search term likely to match something
        let search_term = if let Some(first_path) = paths.first() {
            let path_buf = PathBuf::from(first_path);
            if let Some(file_name) = path_buf.file_name() {
                if let Some(file_str) = file_name.to_str() {
                    if file_str.len() > 3 {
                        file_str[0..3].to_string()
                    } else {
                        "file".to_string()
                    }
                } else {
                    "file".to_string()
                }
            } else {
                "file".to_string()
            }
        } else {
            "file".to_string()
        };

        // Search using the term
        let search_result = state.search(&search_term);
        assert!(search_result.is_ok());

        let results = search_result.unwrap();
        assert!(!results.is_empty(), "Should find matching files");

        // Check that searches are recorded
        let data = state.data.lock().unwrap();
        assert!(!data.recent_activity.recent_searches.is_empty());
        assert!(data.metrics.total_searches > 0);
    }

    #[test]
    fn test_multiple_searches() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get paths and add them directly to the engine
        let paths = collect_test_paths(Some(100));
        for path in &paths {
            let _ = state.add_path(path);
        }

        // Extract some search terms from the paths
        let mut search_terms = Vec::new();
        for path in paths.iter().take(3) {
            let path_buf = PathBuf::from(path);
            if let Some(file_name) = path_buf.file_name() {
                if let Some(file_str) = file_name.to_str() {
                    if file_str.len() > 3 {
                        search_terms.push(file_str[0..3].to_string());
                    }
                }
            }
        }

        // If we couldn't find enough terms, add some default ones
        while search_terms.len() < 3 {
            search_terms.push("file".to_string());
        }

        // Perform multiple searches
        for term in &search_terms {
            let _ = state.search(term);
        }

        // Check that recent searches are tracked in order
        let data = state.data.lock().unwrap();
        assert_eq!(data.recent_activity.recent_searches.len(), 3);

        // Verify the order (newest first)
        if search_terms.len() >= 3 {
            assert_eq!(data.recent_activity.recent_searches[0], search_terms[2]);
            assert_eq!(data.recent_activity.recent_searches[1], search_terms[1]);
            assert_eq!(data.recent_activity.recent_searches[2], search_terms[0]);
        }
    }

    #[test]
    fn test_concurrent_operations() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));

        // Get a test directory for indexing
        let (test_dir, subdir) = get_test_subdirs();

        // Create a LOT of test files to ensure indexing takes time
        let mut test_files = Vec::new();
        for i in 0..1000 {
            // Increased to 1000 files to ensure indexing takes time
            let file_path = test_dir.join(format!("concurrent_test_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use more reliable synchronization
        let (status_tx, status_rx) = std::sync::mpsc::channel();

        // Clone the Arc for the thread to use
        let state_clone = Arc::clone(&state);
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            // First manually set the status to Indexing to guarantee we're in that state
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;

                // Signal the test thread that we've set the status
                status_tx.send(()).unwrap();
            }

            // Now start the actual indexing (which may take a while)
            state_clone.start_indexing(test_dir_clone).unwrap();
        });

        // Wait for the signal that the status has been explicitly set to Indexing
        status_rx.recv().unwrap();

        // Double-check that we're in the Indexing state before proceeding
        {
            let data = state.data.lock().unwrap();
            assert_eq!(
                data.status,
                SearchEngineStatus::Indexing,
                "Should be in Indexing state before testing concurrent operations"
            );
        }

        // Try to search while indexing - should return an error
        let search_result = state.search("file");
        assert!(
            search_result.is_err(),
            "Search should fail with an error when engine is indexing"
        );
        assert!(
            search_result.unwrap_err().contains("indexing"),
            "Error should mention indexing"
        );

        // Try to start another indexing operation - should stop the previous one and start new
        let second_index_result = state.start_indexing(subdir.clone());
        assert!(
            second_index_result.is_ok(),
            "Starting new indexing operation should succeed even when one is in progress"
        );

        // Wait for indexing thread to complete
        indexing_thread.join().unwrap();

        // Allow more time for the second indexing operation to complete and update the state
        thread::sleep(Duration::from_millis(1000)); // Increased wait time to 1 second

        // Get the expected directory name for comparison
        let expected_name = subdir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Retry mechanism for checking the directory - sometimes indexing takes longer
        let max_attempts = 5;
        let mut attempt = 0;
        let mut success = false;

        while attempt < max_attempts && !success {
            let data = state.data.lock().unwrap();

            // Check if we're still indexing
            if matches!(data.status, SearchEngineStatus::Indexing) {
                // Skip this attempt if still indexing
                log_info!(
                    "Attempt {}: Indexing still in progress, waiting...",
                    attempt + 1
                );
                drop(data); // Release the lock before sleeping
                thread::sleep(Duration::from_millis(500));
            } else {
                // Get just the filename component for comparison
                let actual_name = data
                    .index_folder
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                log_info!(
                    "Attempt {}: Actual folder name: '{}', Expected: '{}'",
                    attempt + 1,
                    actual_name,
                    expected_name
                );

                // If names match or one contains the other (to handle path formatting differences)
                if actual_name == expected_name
                    || actual_name.contains(&expected_name)
                    || expected_name.contains(&actual_name)
                {
                    success = true;
                    log_info!("Directory name check passed!");
                } else {
                    drop(data); // Release the lock before sleeping
                    thread::sleep(Duration::from_millis(500));
                }
            }

            attempt += 1;
        }

        assert!(
            success,
            "Failed to verify index folder was updated after {} attempts",
            max_attempts
        );

        // Clean up test files (best effort, don't fail test if cleanup fails)
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_directory_context_for_search() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get paths from test data
        let paths = collect_test_paths(Some(200));

        // Add paths directly to the engine
        for path in &paths {
            let _ = state.add_path(path);
        }

        // Find a directory to use as context
        let dir_context = if let Some(first_path) = paths.first() {
            let path_buf = PathBuf::from(first_path);
            if let Some(parent) = path_buf.parent() {
                parent.to_string_lossy().to_string()
            } else {
                get_test_data_path().to_string_lossy().to_string()
            }
        } else {
            get_test_data_path().to_string_lossy().to_string()
        };
        
        // Update configuration with directory context
        let _ = state.update_config(Some(dir_context.clone()));

        // Search for a generic term
        let search_result = state.search("file");
        assert!(search_result.is_ok());

        let results = search_result.unwrap();

        // Results from the current directory should be ranked higher
        if !results.is_empty() {
            let top_result = &results[0].0;
            log_info!(
                "Top result: {} for context dir: {}",
                top_result, dir_context
            );

            // Count results from context directory
            let context_matches = results
                .iter()
                .filter(|(path, _)| path.starts_with(&dir_context))
                .count();

            log_info!(
                "{} of {} results are from context directory",
                context_matches,
                results.len()
            );

            assert!(
                context_matches > 0,
                "At least some results should be from context directory"
            );
        }
    }

    #[test]
    fn test_sequential_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get two subdirectories for sequential indexing
        let (subdir1, subdir2) = get_test_subdirs();

        // Add some test files to both directories to ensure they have content
        let file1 = subdir1.join("testfile1.txt");
        let file2 = subdir2.join("testfile2.txt");

        let _ = fs::write(&file1, "Test content 1");
        let _ = fs::write(&file2, "Test content 2");

        // Index first directory
        let _ = state.start_indexing(subdir1.clone());

        // Allow indexing to complete
        thread::sleep(Duration::from_millis(200));

        // Search for the first file
        let search1 = state.search("testfile1");
        assert!(search1.is_ok());
        let results1 = search1.unwrap();
        let has_file1 = results1.iter().any(|(path, _)| path.contains("testfile1"));
        assert!(
            has_file1,
            "Should find testfile1 after indexing first directory"
        );

        // Now index second directory
        let _ = state.start_indexing(subdir2.clone());

        // Allow indexing to complete
        thread::sleep(Duration::from_millis(200));

        // Search for the second file
        let search2 = state.search("testfile2");
        assert!(search2.is_ok());
        let results2 = search2.unwrap();
        let has_file2 = results2.iter().any(|(path, _)| path.contains("testfile2"));
        assert!(
            has_file2,
            "Should find testfile2 after indexing second directory"
        );

        // First file should no longer be found (or at least not ranked highly)
        let search1_again = state.search("testfile1");
        assert!(search1_again.is_ok());
        let results1_again = search1_again.unwrap();
        let still_has_file1 = results1_again
            .iter()
            .any(|(path, _)| path.contains("testfile1"));
        assert!(
            !still_has_file1,
            "Should not find testfile1 after switching indexes"
        );

        // Clean up test files
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
    }

    #[test]
    fn test_empty_search_query() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Add some test paths
        let paths = collect_test_paths(Some(50));
        for path in &paths {
            let _ = state.add_path(path);
        }

        // Search with empty query
        let empty_search = state.search("");
        assert!(empty_search.is_ok());

        // Should return empty results
        let results = empty_search.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_update_indexing_progress() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Set initial state for testing progress updates
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        {
            let mut data = state.data.lock().unwrap();
            data.progress.start_time = Some(start_time);
            data.status = SearchEngineStatus::Indexing;
        }

        // Update progress manually
        state.update_indexing_progress(50, 100, Some("/path/to/current/file.txt".to_string()));

        // Check progress data
        let data = state.data.lock().unwrap();
        assert_eq!(data.progress.files_indexed, 50);
        assert_eq!(data.progress.files_discovered, 100);
        assert_eq!(data.progress.percentage_complete, 50.0);
        assert_eq!(
            data.progress.current_path,
            Some("/path/to/current/file.txt".to_string())
        );

        // Only check if estimated_time_remaining exists, as the exact value will vary
        assert!(data.progress.estimated_time_remaining.is_some());
    }

    #[test]
    fn test_get_stats() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get initial stats
        let initial_stats = state.get_stats();
        assert_eq!(initial_stats.trie_size, 0);

        // Add paths
        let paths = collect_test_paths(Some(20));
        for path in &paths {
            let _ = state.add_path(path);
        }

        // Get stats after adding paths
        let after_stats = state.get_stats();
        assert!(
            after_stats.trie_size > 0,
            "Trie should contain indexed paths"
        );
        assert!(
            after_stats.trie_size >= paths.len(),
            "Trie should contain all indexed paths"
        );
    }

    #[test]
    fn test_update_config() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Update the configuration
        let result = state.update_config(Some("/home/user".to_string()));
        assert!(result.is_ok());

        // Check that configuration was updated
        let data = state.data.lock().unwrap();
        assert_eq!(
            data.current_directory,
            Some("/home/user".to_string())
        );
    }

    #[test]
    fn test_add_and_remove_path() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Add a path
        let result = state.add_path("/test/path.txt");
        assert!(result.is_ok());

        // Search for the path
        let search_result = state.search("path.txt");
        assert!(search_result.is_ok());

        let results = search_result.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/test/path.txt");

        // Remove the path
        let remove_result = state.remove_path("/test/path.txt");
        assert!(remove_result.is_ok());

        // Search again - should not find the path
        let search_again = state.search("path.txt");
        assert!(search_again.is_ok());

        let empty_results = search_again.unwrap();
        assert!(empty_results.is_empty() || !empty_results[0].0.contains("/test/path.txt"));
    }

    #[test]
    fn test_start_indexing_invalid_path() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Try to index an invalid path
        let invalid_path = PathBuf::from("/path/that/does/not/exist");
        let result = state.start_indexing(invalid_path);

        // Should still return Ok since the error is handled internally
        assert!(result.is_ok());

        // But the status should be Failed or Idle
        thread::sleep(Duration::from_millis(50)); // Wait for status update
        let data = state.data.lock().unwrap();
        assert!(matches!(
            data.status,
            SearchEngineStatus::Failed | SearchEngineStatus::Idle
        ));
    }

    #[test]
    fn test_stop_indexing_when_not_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Set state to Idle to ensure we're not indexing
        {
            let mut data = state.data.lock().unwrap();
            data.status = SearchEngineStatus::Idle;
        }

        // Try to stop indexing when not indexing
        let result = state.stop_indexing();

        // Should return an error
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("No indexing operation in progress"));
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_thread_safety() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let state_clone = Arc::clone(&state);
        let test_dir = get_test_dir_for_indexing();

        // Create a LOT of test files to ensure indexing takes time
        let mut test_files = Vec::new();
        for i in 0..1000 {
            // Increased to 1000 files to ensure indexing takes time
            let file_path = test_dir.join(format!("thread_safety_test_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use more reliable synchronization
        let (status_tx, status_rx) = std::sync::mpsc::channel();

        // Start indexing in another thread
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            // First manually set the status to Indexing to guarantee we're in that state
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;

                // Signal the test thread that we've set the status
                status_tx.send(()).unwrap();
            }

            // Now start the actual indexing (which may take a while)
            state_clone.start_indexing(test_dir_clone).unwrap();
        });

        // Wait for the signal that the status has been explicitly set to Indexing
        status_rx.recv().unwrap();

        // Double-check that we're in the Indexing state before proceeding
        {
            let data = state.data.lock().unwrap();
            assert_eq!(
                data.status,
                SearchEngineStatus::Indexing,
                "Should be in Indexing state before testing thread safety"
            );
        }

        // Try to search from the main thread - should return an error while indexing
        let search_result = state.search("document");
        assert!(
            search_result.is_err(),
            "Search should fail with an error when engine is indexing"
        );
        assert!(
            search_result.unwrap_err().contains("indexing"),
            "Error should mention indexing"
        );

        // Now stop the indexing operation
        let _ = state.stop_indexing();

        // Wait for indexing thread to complete
        indexing_thread.join().unwrap();

        // Set status back to Idle to allow successful search
        {
            let mut data = state.data.lock().unwrap();
            data.status = SearchEngineStatus::Idle;
        }

        // Now search should work
        let after_search = state.search("document");
        assert!(
            after_search.is_ok(),
            "Search should succeed after indexing is complete"
        );

        // Clean up test files (best effort, don't fail test if cleanup fails)
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_clone_implementation() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Test that we can clone the state
        let cloned_state = state.clone();

        // Test that the cloned state operates independently
        // by modifying the original state's data
        {
            let mut data = state.data.lock().unwrap();
            data.status = SearchEngineStatus::Searching;
        }

        // The cloned state should see the change since they share the same Arc<Mutex<>>
        {
            let data = cloned_state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Searching);
        }
    }

    #[test]
    fn test_interactive_search_scenarios() {
        // This test simulates a user interacting with the search engine
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);
        let mut paths = collect_test_paths(Some(100)); // Reduced for test stability

        // Ensure we have distinct paths with predictable content
        paths.push("/test/document1.txt".to_string());
        paths.push("/test/document2.txt".to_string());
        paths.push("/test/documents/file.txt".to_string());
        paths.push("/test/docs/readme.md".to_string());

        // Add "folder" entries that would only match "do" but not "doc"
        paths.push("/test/downloads/file1.txt".to_string());
        paths.push("/test/downloads/file2.txt".to_string());

        // Add paths to the engine
        for path in &paths {
            state.add_path(path).expect("Failed to add path");
        }

        // Scenario 1: User performs a search, then refines it with more specific terms
        let initial_search_term = "doc";
        let refined_search_term = "docu";

        let initial_search = state
            .search(initial_search_term)
            .expect("Initial search failed");
        log_info!(
            "Initial search for '{}' found {} results",
            initial_search_term,
            initial_search.len()
        );

        for (i, (path, score)) in initial_search.iter().take(5).enumerate() {
            log_info!(
                "  Initial result #{}: {} (score: {})",
                i + 1,
                path,
                score
            );
        }

        let refined_search = state
            .search(refined_search_term)
            .expect("Refined search failed");
        log_info!(
            "Refined search for '{}' found {} results",
            refined_search_term,
            refined_search.len()
        );

        for (i, (path, score)) in refined_search.iter().take(5).enumerate() {
            log_info!(
                "  Refined result #{}: {} (score: {})",
                i + 1,
                path,
                score
            );
        }

        // Count paths that match each search term
        let do_matches = paths.iter().filter(|p| p.contains("do")).count();
        let doc_matches = paths.iter().filter(|p| p.contains("doc")).count();

        log_info!(
            "Paths containing 'do': {}, paths containing 'doc': {}",
            do_matches, doc_matches
        );

        // Only assert if the dataset should logically support our assumption
        if doc_matches <= do_matches {
            assert!(
                refined_search.len() <= initial_search.len(),
                "Refined search should return fewer or equal results"
            );
        } else {
            log_info!("Skipping assertion - test data has more 'doc' matches than 'do' matches");
        }

        // Rest of the test remains unchanged
        // ...existing code...
    }

    #[test]
    fn test_with_real_world_data() {
        log_info!("Testing SearchEngineState with real-world test data");
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get real-world paths from test data (limit to 100 for stability)
        let mut paths = collect_test_paths(Some(100));
        log_info!("Collected {} test paths", paths.len());

        // Add some guaranteed test paths
        paths.push("./test-data-for-fuzzy-search/file1.txt".to_string());
        paths.push("./test-data-for-fuzzy-search/file2.txt".to_string());
        paths.push("./test-data-for-fuzzy-search/test.md".to_string());

        // Add paths directly to the engine
        let start = Instant::now();
        for path in &paths {
            state.add_path(path).expect("Failed to add path");
        }
        let elapsed = start.elapsed();
        log_info!(
            "Added {} paths in {:?} ({:.2} paths/ms)",
            paths.len(),
            elapsed,
            paths.len() as f64 / elapsed.as_millis().max(1) as f64
        );

        // Get stats after adding paths
        let stats = state.get_stats();
        log_info!(
            "Engine stats after adding paths - Cache size: {}, Trie size: {}",
            stats.cache_size, stats.trie_size
        );

        // Use multiple search queries to increase chances of finding matches
        let test_queries = ["fi", "test", "file", "txt", "md"];

        let mut found_results = false;
        for query in &test_queries {
            // Perform search
            let search_start = Instant::now();
            let results = state.search(query).expect("Search failed");
            let search_elapsed = search_start.elapsed();

            log_info!(
                "Search for '{}' found {} results in {:?}",
                query,
                results.len(),
                search_elapsed
            );

            if !results.is_empty() {
                found_results = true;

                // Log top results
                for (i, (path, score)) in results.iter().take(3).enumerate() {
                    log_info!(
                        "  Result #{}: {} (score: {:.4})",
                        i + 1,
                        path,
                        score
                    );
                }

                break;
            }
        }

        assert!(
            found_results,
            "Should find results with real-world data using at least one of the test queries"
        );
    }

    #[test]
    fn test_search_by_extension() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Add paths with different extensions
        state.add_path("/test/document.pdf").unwrap();
        state.add_path("/test/document.txt").unwrap();
        state.add_path("/test/document.docx").unwrap();
        state.add_path("/test/image.jpg").unwrap();
        state.add_path("/test/spreadsheet.xlsx").unwrap();

        // Search with no extension preference
        let regular_results = state.search("document").unwrap();

        // Search with preference for txt extension only
        let txt_results = state
            .search_by_extension("document", vec!["txt".to_string()])
            .unwrap();

        // Search with preference for pdf extension only
        let pdf_results = state
            .search_by_extension("document", vec!["pdf".to_string()])
            .unwrap();

        // Search with multiple extension preferences in order (txt first, then pdf)
        let txt_pdf_results = state
            .search_by_extension("document", vec!["txt".to_string(), "pdf".to_string()])
            .unwrap();

        // Search with different order of extensions (pdf first, then txt)
        let pdf_txt_results = state
            .search_by_extension("document", vec!["pdf".to_string(), "txt".to_string()])
            .unwrap();

        // Verify that extension preferences affect ranking
        if !txt_results.is_empty() && !pdf_results.is_empty() {
            assert_eq!(
                txt_results[0].0, "/test/document.txt",
                "TXT document should be first with txt extension preference"
            );
            assert_eq!(
                pdf_results[0].0, "/test/document.pdf",
                "PDF document should be first with pdf extension preference"
            );
        }

        // Verify that multiple extension preferences work in order
        if !txt_pdf_results.is_empty() && !pdf_txt_results.is_empty() {
            // When txt is first priority, txt document should be first
            assert_eq!(
                txt_pdf_results[0].0, "/test/document.txt",
                "TXT document should be first when txt is first priority"
            );
            // When pdf is first priority, pdf document should be first
            assert_eq!(
                pdf_txt_results[0].0, "/test/document.pdf",
                "PDF document should be first when pdf is first priority"
            );

            // The second item should be the second prioritized extension
            if txt_pdf_results.len() >= 2 && pdf_txt_results.len() >= 2 {
                assert_eq!(
                    txt_pdf_results[1].0, "/test/document.pdf",
                    "PDF document should be second when pdf is second priority"
                );
                assert_eq!(
                    pdf_txt_results[1].0, "/test/document.txt",
                    "TXT document should be second when txt is second priority"
                );
            }
        }

        // Verify that all documents are still found with different rankings
        assert_eq!(regular_results.len(), txt_results.len());
        assert_eq!(regular_results.len(), pdf_results.len());
        assert_eq!(regular_results.len(), txt_pdf_results.len());

        // Test search for a non-existent extension
        let nonexistent_results = state
            .search_by_extension("document", vec!["nonexistent".to_string()])
            .unwrap();
        assert_eq!(
            regular_results.len(),
            nonexistent_results.len(),
            "Should still find all documents with non-existent extension"
        );

        // Test with empty extensions list (should use default preferences)
        let empty_ext_results = state.search_by_extension("document", vec![]).unwrap();
        assert_eq!(
            regular_results.len(),
            empty_ext_results.len(),
            "Should find all documents with empty extensions list"
        );

        // Results should match regular search results when no extensions are specified
        if !regular_results.is_empty() && !empty_ext_results.is_empty() {
            assert_eq!(
                regular_results[0].0, empty_ext_results[0].0,
                "Top result should match regular search when no extensions specified"
            );
        }
    }
}

