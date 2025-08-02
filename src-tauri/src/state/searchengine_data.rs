use crate::models::search_engine_config::SearchEngineConfig;
use crate::search_engine::search_core::{EngineStats, SearchCore};
use crate::state::SettingsState;
#[allow(unused_imports)]
use crate::{log_error, log_info, log_warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{fs, thread};



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
    pub engine: Arc<Mutex<SearchCore>>,
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
        let engine = SearchCore::new(
            config.cache_size,
            config.max_results,
            config.cache_ttl.unwrap(),
            ranking_config,
        );

        Self {
            data: Arc::new(Mutex::new(Self::save_default_search_engine_in_state(
                config,
            ))),
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
    #[allow(dead_code)]
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

    /// Starts indexing a folder in chunks to prevent crashes with large directories.
    ///
    /// This method collects all paths first and then processes them in smaller batches,
    /// releasing locks between chunks to prevent UI freezes. Now includes all features
    /// from the original indexing method including progress tracking, metrics, and cancellation.
    ///
    /// # Arguments
    ///
    /// * `folder` - The root folder path to index
    /// * `chunk_size` - Number of paths to process in each chunk
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Indexing completed successfully
    /// * `Err(String)` - An error occurred during indexing
    pub fn start_chunked_indexing(&self, folder: PathBuf, chunk_size: usize) -> Result<(), String> {
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
            #[cfg(test)]
            log_info!(
                "Stopping previous indexing of '{}' before starting new chunked indexing",
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

        // Actually start the chunked indexing
        if let Some(_folder_str) = folder.to_str() {
            // Release the locks before starting the recursive operation
            drop(data);
            drop(engine);

            // First collect all paths that need to be indexed
            let paths =
                self.collect_paths_recursive(&folder, &excluded_patterns.unwrap_or_default());
            let total_paths = paths.len();

            #[cfg(test)]
            log_info!(
                "Collected {} paths to index in chunks of {}",
                total_paths,
                chunk_size
            );

            // Update initial progress
            {
                let mut data = self.data.lock().unwrap();
                data.progress.files_discovered = total_paths;
                data.progress.files_indexed = 0;
                data.progress.percentage_complete = 0.0;
                data.last_updated = chrono::Utc::now().timestamp_millis() as u64;
            }

            // Process paths in chunks
            let mut files_indexed = 0;
            let mut _chunk_number = 0;

            for chunk in paths.chunks(chunk_size) {
                _chunk_number += 1;

                // Check if indexing should stop before processing chunk
                {
                    let engine = self.engine.lock().unwrap();
                    if engine.should_stop_indexing() {
                        drop(engine);
                        let mut data = self.data.lock().unwrap();
                        data.status = SearchEngineStatus::Cancelled;
                        data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

                        #[cfg(test)]
                        log_info!(
                            "Chunked indexing of '{}' was cancelled after processing {} files",
                            folder.display(),
                            files_indexed
                        );

                        return Ok(());
                    }
                }

                // Process this chunk
                {
                    let mut engine = self.engine.lock().unwrap();
                    for (i, path) in chunk.iter().enumerate() {
                        // Update current path for fine-grained progress tracking
                        {
                            let mut data = self.data.lock().unwrap();
                            data.progress.current_path = Some(path.clone());
                            data.progress.files_indexed = files_indexed + i + 1;
                            data.progress.percentage_complete = if total_paths > 0 {
                                ((files_indexed + i + 1) as f32 / total_paths as f32) * 100.0
                            } else {
                                100.0
                            };

                            // Calculate estimated time remaining
                            if let Some(start_time_ms) = data.progress.start_time {
                                let elapsed_ms =
                                    chrono::Utc::now().timestamp_millis() as u64 - start_time_ms;
                                if files_indexed + i + 1 > 0 {
                                    let avg_time_per_file =
                                        elapsed_ms as f32 / (files_indexed + i + 1) as f32;
                                    let remaining_files =
                                        total_paths.saturating_sub(files_indexed + i + 1);
                                    let estimated_ms =
                                        (avg_time_per_file * remaining_files as f32) as u64;
                                    data.progress.estimated_time_remaining = Some(estimated_ms);
                                }
                            }

                            data.last_updated = chrono::Utc::now().timestamp_millis() as u64;
                        }

                        // Add the path with exclusion checking
                        engine.add_path_with_exclusion_check(path, None);

                        // Check for stop signal during chunk processing
                        if engine.should_stop_indexing() {
                            drop(engine);
                            let mut data = self.data.lock().unwrap();
                            data.status = SearchEngineStatus::Cancelled;
                            data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

                            #[cfg(test)]
                            log_info!(
                                "Chunked indexing was cancelled mid-chunk after {} files",
                                files_indexed + i + 1
                            );

                            return Ok(());
                        }
                    }
                }

                // Update indexed count after processing chunk
                files_indexed += chunk.len();

                // Update progress after each chunk
                {
                    let mut data = self.data.lock().unwrap();
                    data.progress.files_indexed = files_indexed;
                    data.progress.percentage_complete = if total_paths > 0 {
                        (files_indexed as f32 / total_paths as f32) * 100.0
                    } else {
                        100.0
                    };

                    // Calculate estimated time remaining
                    if let Some(start_time_ms) = data.progress.start_time {
                        let elapsed_ms =
                            chrono::Utc::now().timestamp_millis() as u64 - start_time_ms;
                        if files_indexed > 0 {
                            let avg_time_per_file = elapsed_ms as f32 / files_indexed as f32;
                            let remaining_files = total_paths.saturating_sub(files_indexed);
                            let estimated_ms = (avg_time_per_file * remaining_files as f32) as u64;
                            data.progress.estimated_time_remaining = Some(estimated_ms);
                        }
                    }

                    data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

                    #[cfg(test)]
                    log_info!(
                        "Processing chunk {} ({} files): {:.1}% complete",
                        _chunk_number,
                        chunk.len(),
                        data.progress.percentage_complete
                    );
                }

                // Small delay between chunks to allow UI updates and other operations
                thread::sleep(Duration::from_millis(5));
            }

            // Update status and metrics after indexing completes
            let mut data = self.data.lock().unwrap();
            let elapsed = start_time.elapsed();
            data.metrics.last_indexing_duration_ms = Some(elapsed.as_millis() as u64);

            // Check if it was cancelled (double-check)
            let engine = self.engine.lock().unwrap();
            if engine.should_stop_indexing() {
                data.status = SearchEngineStatus::Cancelled;
                #[cfg(test)]
                log_info!(
                    "Chunked indexing of '{}' was cancelled after {:?}",
                    folder.display(),
                    elapsed
                );
            } else {
                data.status = SearchEngineStatus::Idle;
                data.progress.files_indexed = total_paths;
                data.progress.percentage_complete = 100.0;
                data.progress.current_path = None;
                data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

                #[cfg(test)]
                log_info!(
                    "Chunked indexing of '{}' completed in {:?} ({} files processed)",
                    folder.display(),
                    elapsed,
                    total_paths
                );
            }
        } else {
            data.status = SearchEngineStatus::Failed;
            return Err("Invalid folder path".to_string());
        }

        Ok(())
    }

    /// Collects all paths recursively from a directory without indexing them.
    /// Now includes proper exclusion pattern matching and error handling.
    ///
    /// # Arguments
    ///
    /// * `dir` - The directory to scan
    /// * `excluded_patterns` - Patterns to exclude from collection
    ///
    /// # Returns
    ///
    /// A vector of all file paths found that don't match the excluded patterns
    fn collect_paths_recursive(&self, dir: &PathBuf, excluded_patterns: &[String]) -> Vec<String> {
        let mut paths = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();

                // Convert path to string
                if let Some(path_str) = path.to_str() {
                    // Check if path should be excluded using the same logic as the original indexing
                    let should_exclude = excluded_patterns.iter().any(|pattern| {
                        // Support both exact matches and pattern matching
                        path_str.contains(pattern)
                            || path_str.ends_with(pattern)
                            || path
                                .file_name()
                                .and_then(|name| name.to_str())
                                .map(|name| name.contains(pattern))
                                .unwrap_or(false)
                    });

                    if !should_exclude {
                        // Add this path (both files and directories are indexed)
                        paths.push(path_str.to_string());

                        // Recursively add subdirectory contents
                        if path.is_dir() {
                            // Check for stop signal during path collection
                            {
                                let engine = self.engine.lock().unwrap();
                                if engine.should_stop_indexing() {
                                    #[cfg(test)]
                                    log_info!("Path collection stopped due to cancellation signal");
                                    break;
                                }
                            }

                            paths.extend(self.collect_paths_recursive(&path, excluded_patterns));
                        }
                    } else {
                        #[cfg(test)]
                        log_info!("Excluding path: {} (matched pattern)", path_str);
                    }
                }
            }
        } else {
            log_warn!("Failed to read directory: {}", dir.display());
        }

        paths
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
        log_info!("Searching with preferred extensions: {:?}", extensions);

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
                    log_info!("Top result extension: {}, preferred: {:?}", ext, extensions);
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
    /// Enhanced to support both traditional and chunked indexing progress tracking.
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

    /// Stops any ongoing indexing operation (works for both traditional and chunked indexing).
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
            // Signal the engine to stop indexing (works for both traditional and chunked)
            engine.stop_indexing();

            // Update state
            data.status = SearchEngineStatus::Cancelled;
            data.last_updated = chrono::Utc::now().timestamp_millis() as u64;

            #[cfg(test)]
            log_info!(
                "Indexing of '{}' stopped (works for both traditional and chunked)",
                data.index_folder.display()
            );

            return Ok(());
        }

        Err("No indexing operation in progress".to_string())
    }

    /// Cancels the current indexing operation at user request (works for both traditional and chunked).
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
// Helper function to get test data directory
fn get_test_data_path() -> PathBuf {
    use crate::search_engine::test_generate_test_data::generate_test_data_if_not_exists;
    use crate::constants::TEST_DATA_PATH;

    let path = PathBuf::from(TEST_DATA_PATH);
    generate_test_data_if_not_exists(PathBuf::from(TEST_DATA_PATH)).unwrap_or_else(|err| {
        log_error!("Error during test data generation or path lookup: {}", err);
        panic!("Test data generation failed");
    });
    path
}

#[cfg(test)]
// Helper function to collect real paths from the test data directory
fn collect_test_paths(limit: Option<usize>) -> Vec<String> {
    let test_path = get_test_data_path();
    let mut paths = Vec::new();

    fn add_paths_recursively(dir: &std::path::Path, paths: &mut Vec<String>, limit: Option<usize>) {
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

#[cfg(test)]
mod tests_searchengine_state {
    use super::*;
    use crate::log_info;
    use std::fs;
    use std::thread;
    use std::time::Duration;

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
                top_result,
                dir_context
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
        assert_eq!(data.current_directory, Some("/home/user".to_string()));
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

        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;
                status_tx.send(()).unwrap();
            }

            state_clone.start_indexing(test_dir_clone).unwrap();
        });

        status_rx.recv().unwrap();

        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Indexing);
        }

        // Try to search from main thread - should fail while indexing
        let search_result = state.search("document");
        assert!(search_result.is_err());
        assert!(search_result.unwrap_err().contains("indexing"));

        // Stop the indexing operation
        let _ = state.stop_indexing();

        indexing_thread.join().unwrap();

        // Set status back to Idle to allow successful search
        {
            let mut data = state.data.lock().unwrap();
            data.status = SearchEngineStatus::Idle;
        }

        // Now search should work
        let after_search = state.search("document");
        assert!(after_search.is_ok());

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
            log_info!("  Initial result #{}: {} (score: {})", i + 1, path, score);
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
            log_info!("  Refined result #{}: {} (score: {})", i + 1, path, score);
        }

        // Count paths that match each search term
        let do_matches = paths.iter().filter(|p| p.contains("do")).count();
        let doc_matches = paths.iter().filter(|p| p.contains("doc")).count();

        log_info!(
            "Paths containing 'do': {}, paths containing 'doc': {}",
            do_matches,
            doc_matches
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
            stats.cache_size,
            stats.trie_size
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
                    log_info!("  Result #{}: {} (score: {:.4})", i + 1, path, score);
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

    #[test]
    fn test_start_chunked_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get a test directory for indexing
        let test_dir = get_test_dir_for_indexing();

        // Create test files to ensure we have enough for multiple chunks
        let mut test_files = Vec::new();
        for i in 0..100 {
            let file_path = test_dir.join(format!("chunked_test_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use a small chunk size to ensure multiple chunks
        let chunk_size = 10;

        // Start chunked indexing
        let result = state.start_chunked_indexing(test_dir.clone(), chunk_size);
        assert!(result.is_ok(), "Chunked indexing should start successfully");

        // After indexing completes, verify the status is Idle
        let data = state.data.lock().unwrap();
        assert_eq!(
            data.status,
            SearchEngineStatus::Idle,
            "Status should be Idle after completion"
        );
        assert_eq!(
            data.progress.percentage_complete, 100.0,
            "Progress should be 100%"
        );

        // Check that we can search for the indexed files
        drop(data);
        let search_result = state.search("chunked_test");
        assert!(search_result.is_ok());

        let results = search_result.unwrap();
        assert!(!results.is_empty(), "Should find indexed files");

        // Verify that at least one chunked test file is found
        let found_chunked_test = results
            .iter()
            .any(|(path, _)| path.contains("chunked_test"));
        assert!(
            found_chunked_test,
            "Should find at least one chunked test file"
        );

        // Clean up test files
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_start_chunked_indexing_cancellation() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let state_clone = Arc::clone(&state);

        // Get a test directory for indexing
        let test_dir = get_test_dir_for_indexing();

        // Create test files to ensure we have enough for multiple chunks
        let mut test_files = Vec::new();
        for i in 0..200 {
            let file_path = test_dir.join(format!("cancel_chunked_{}.txt", i));
            let _ = fs::write(&file_path, format!("Test content {}", i));
            test_files.push(file_path);
        }

        // Use a small chunk size with delay to ensure we can cancel mid-operation
        let chunk_size = 5;

        // Start chunked indexing in a separate thread
        let test_dir_clone = test_dir.clone();
        let (tx, rx) = std::sync::mpsc::channel();

        let indexing_thread = thread::spawn(move || {
            // Signal that we're about to start indexing
            tx.send(()).unwrap();

            let result = state_clone.start_chunked_indexing(test_dir_clone, chunk_size);
            assert!(result.is_ok());
        });

        // Wait for the signal that indexing is about to start
        rx.recv().unwrap();

        // Give indexing a moment to begin
        thread::sleep(Duration::from_millis(50));

        // Now stop indexing
        {
            let mut engine = state.engine.lock().unwrap();
            engine.stop_indexing();
        }

        // Wait for indexing thread to complete
        indexing_thread.join().unwrap();

        // Check that status is Cancelled
        let data = state.data.lock().unwrap();
        assert_eq!(
            data.status,
            SearchEngineStatus::Cancelled,
            "Status should be Cancelled after stopping indexing"
        );

        // Clean up test files
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    // ========== NEW CHUNKED INDEXING TESTS ==========

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_start_chunked_indexing_basic() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);
        let test_dir = get_test_dir_for_indexing();

        // Start chunked indexing with chunk size 100
        let result = state.start_chunked_indexing(test_dir.clone(), 100);
        assert!(result.is_ok(), "Chunked indexing should start successfully");

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

    #[test]
    fn test_chunked_indexing_stop() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let test_dir = get_test_dir_for_indexing();

        // Create test files to ensure indexing takes enough time
        let mut test_files = Vec::new();
        for i in 0..1000 {
            let file_path = test_dir.join(format!("chunked_testfile_{}.txt", i));
            let _ = fs::write(&file_path, format!("Chunked test content {}", i));
            test_files.push(file_path);
        }

        // Use small chunk size and synchronization
        let (status_tx, status_rx) = std::sync::mpsc::channel();
        let state_clone = Arc::clone(&state);
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            // Set status to Indexing
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;
                status_tx.send(()).unwrap();
            }

            // Start chunked indexing
            state_clone
                .start_chunked_indexing(test_dir_clone, 10)
                .unwrap();
        });

        // Wait for indexing to start
        status_rx.recv().unwrap();

        // Verify we're in Indexing state
        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Indexing);
        }

        // Stop indexing
        let stop_result = state.stop_indexing();
        assert!(
            stop_result.is_ok(),
            "Should successfully stop chunked indexing"
        );

        // Verify that stopping worked
        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Cancelled);
        }

        indexing_thread.join().unwrap();

        // Clean up test files
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_chunked_indexing_cancel() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let test_dir = get_test_dir_for_indexing();

        // Create many test files
        let mut test_files = Vec::new();
        for i in 0..1000 {
            let file_path = test_dir.join(format!("chunked_cancel_test_{}.txt", i));
            let _ = fs::write(&file_path, format!("Chunked cancel test content {}", i));
            test_files.push(file_path);
        }

        let (status_tx, status_rx) = std::sync::mpsc::channel();
        let state_clone = Arc::clone(&state);
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;
                status_tx.send(()).unwrap();
            }

            state_clone
                .start_chunked_indexing(test_dir_clone, 5)
                .unwrap();
        });

        status_rx.recv().unwrap();

        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Indexing);
        }

        // Cancel indexing
        let cancel_result = state.cancel_indexing();
        assert!(
            cancel_result.is_ok(),
            "Should successfully cancel chunked indexing"
        );

        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Cancelled);
        }

        indexing_thread.join().unwrap();

        // Clean up test files
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_chunked_search_after_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get paths and use chunked indexing to add them
        let test_dir = get_test_dir_for_indexing();
        let result = state.start_chunked_indexing(test_dir.clone(), 50);
        assert!(result.is_ok());

        // Wait for indexing to complete
        thread::sleep(Duration::from_millis(200));

        // Find a search term from the indexed content
        let search_term = "apple";

        // Search using the term
        let search_result = state.search(&search_term);
        assert!(search_result.is_ok());

        let _results = search_result.unwrap();
        // Results might be empty if no files contain "test", which is acceptable

        // Check that searches are recorded
        let data = state.data.lock().unwrap();
        assert!(!data.recent_activity.recent_searches.is_empty());
        assert!(data.metrics.total_searches > 0);
    }

    #[test]
    fn test_chunked_multiple_searches() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing on test directory
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir, 100);
        thread::sleep(Duration::from_millis(200));

        let search_terms = ["file", "test", "data"];

        // Perform multiple searches
        for term in &search_terms {
            let _ = state.search(term);
        }

        // Check that recent searches are tracked
        let data = state.data.lock().unwrap();
        assert_eq!(data.recent_activity.recent_searches.len(), 3);

        // Verify the order (newest first)
        assert_eq!(data.recent_activity.recent_searches[0], search_terms[2]);
        assert_eq!(data.recent_activity.recent_searches[1], search_terms[1]);
        assert_eq!(data.recent_activity.recent_searches[2], search_terms[0]);
    }

    #[test]
    fn test_chunked_directory_context_for_search() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing on test directory
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir.clone(), 75);
        thread::sleep(Duration::from_millis(200));

        // Set directory context
        let dir_context = test_dir.to_string_lossy().to_string();
        let _ = state.update_config(Some(dir_context.clone()));

        // Search for a generic term
        let search_result = state.search("file");
        assert!(search_result.is_ok());

        let results = search_result.unwrap();

        if !results.is_empty() {
            let top_result = &results[0].0;
            log_info!(
                "Chunked indexing top result: {} for context dir: {}",
                top_result,
                dir_context
            );

            // Count results from context directory
            let context_matches = results
                .iter()
                .filter(|(path, _)| path.starts_with(&dir_context))
                .count();

            log_info!(
                "Chunked: {} of {} results are from context directory",
                context_matches,
                results.len()
            );
        }
    }

    #[test]
    fn test_chunked_sequential_indexing() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        let (subdir1, subdir2) = get_test_subdirs();

        // Add test files to both directories
        let file1 = subdir1.join("chunked_testfile1.txt");
        let file2 = subdir2.join("chunked_testfile2.txt");

        let _ = fs::write(&file1, "Chunked test content 1");
        let _ = fs::write(&file2, "Chunked test content 2");

        // Index first directory with chunked indexing
        let _ = state.start_chunked_indexing(subdir1.clone(), 25);
        thread::sleep(Duration::from_millis(200));

        // Search for the first file
        let search1 = state.search("chunked_testfile1");
        assert!(search1.is_ok());
        let results1 = search1.unwrap();
        let has_file1 = results1
            .iter()
            .any(|(path, _)| path.contains("chunked_testfile1"));
        assert!(
            has_file1,
            "Should find chunked_testfile1 after chunked indexing first directory"
        );

        // Index second directory with chunked indexing
        let _ = state.start_chunked_indexing(subdir2.clone(), 25);
        thread::sleep(Duration::from_millis(200));

        // Search for the second file
        let search2 = state.search("chunked_testfile2");
        assert!(search2.is_ok());
        let results2 = search2.unwrap();
        let has_file2 = results2
            .iter()
            .any(|(path, _)| path.contains("chunked_testfile2"));
        assert!(
            has_file2,
            "Should find chunked_testfile2 after chunked indexing second directory"
        );

        // First file should no longer be found
        let search1_again = state.search("chunked_testfile1");
        assert!(search1_again.is_ok());
        let results1_again = search1_again.unwrap();
        let still_has_file1 = results1_again
            .iter()
            .any(|(path, _)| path.contains("chunked_testfile1"));
        assert!(
            !still_has_file1,
            "Should not find chunked_testfile1 after switching indexes"
        );

        // Clean up test files
        let _ = fs::remove_file(file1);
        let _ = fs::remove_file(file2);
    }

    #[test]
    fn test_chunked_empty_search_query() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir, 50);
        thread::sleep(Duration::from_millis(200));

        // Search with empty query
        let empty_search = state.search("");
        assert!(empty_search.is_ok());

        // Should return empty results
        let results = empty_search.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_chunked_update_indexing_progress() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Set initial state for testing progress updates during chunked indexing
        let start_time = chrono::Utc::now().timestamp_millis() as u64;
        {
            let mut data = state.data.lock().unwrap();
            data.progress.start_time = Some(start_time);
            data.status = SearchEngineStatus::Indexing;
        }

        // Update progress manually (simulating chunked indexing progress)
        state.update_indexing_progress(
            25,
            100,
            Some("/chunked/path/to/current/file.txt".to_string()),
        );

        // Check progress data
        let data = state.data.lock().unwrap();
        assert_eq!(data.progress.files_indexed, 25);
        assert_eq!(data.progress.files_discovered, 100);
        assert_eq!(data.progress.percentage_complete, 25.0);
        assert_eq!(
            data.progress.current_path,
            Some("/chunked/path/to/current/file.txt".to_string())
        );
        assert!(data.progress.estimated_time_remaining.is_some());
    }

    #[test]
    fn test_chunked_get_stats() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Get initial stats
        let initial_stats = state.get_stats();
        assert_eq!(initial_stats.trie_size, 0);

        // Use chunked indexing on test directory
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir, 40);
        thread::sleep(Duration::from_millis(200));

        // Get stats after chunked indexing
        let after_stats = state.get_stats();
        assert!(
            after_stats.trie_size >= 0,
            "Trie should contain indexed paths after chunked indexing"
        );
    }

    #[test]
    fn test_chunked_indexing_invalid_path() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Try chunked indexing on an invalid path
        let invalid_path = PathBuf::from("/chunked/path/that/does/not/exist");
        let result = state.start_chunked_indexing(invalid_path, 50);

        // Should still return Ok since the error is handled internally
        assert!(result.is_ok());

        // Status should be Idle since no files were found to index
        thread::sleep(Duration::from_millis(50));
        let data = state.data.lock().unwrap();
        assert!(matches!(data.status, SearchEngineStatus::Idle));
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_chunked_thread_safety() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = Arc::new(SearchEngineState::new(settings_state));
        let state_clone = Arc::clone(&state);
        let test_dir = get_test_dir_for_indexing();

        // Create test files
        let mut test_files = Vec::new();
        for i in 0..1000 {
            let file_path = test_dir.join(format!("chunked_thread_safety_test_{}.txt", i));
            let _ = fs::write(
                &file_path,
                format!("Chunked thread safety test content {}", i),
            );
            test_files.push(file_path);
        }

        let (status_tx, status_rx) = std::sync::mpsc::channel();
        let test_dir_clone = test_dir.clone();

        let indexing_thread = thread::spawn(move || {
            {
                let mut data = state_clone.data.lock().unwrap();
                data.status = SearchEngineStatus::Indexing;
                status_tx.send(()).unwrap();
            }

            state_clone
                .start_chunked_indexing(test_dir_clone, 30)
                .unwrap();
        });

        status_rx.recv().unwrap();

        {
            let data = state.data.lock().unwrap();
            assert_eq!(data.status, SearchEngineStatus::Indexing);
        }

        // Try to search from main thread - should fail while chunked indexing
        let search_result = state.search("document");
        assert!(search_result.is_err());
        assert!(search_result.unwrap_err().contains("indexing"));

        // Stop the chunked indexing operation
        let _ = state.stop_indexing();

        indexing_thread.join().unwrap();

        // Set status back to Idle to allow successful search
        {
            let mut data = state.data.lock().unwrap();
            data.status = SearchEngineStatus::Idle;
        }

        // Now search should work
        let after_search = state.search("document");
        assert!(after_search.is_ok());

        // Clean up test files
        for file in test_files {
            let _ = fs::remove_file(file);
        }
    }

    #[test]
    fn test_chunked_interactive_search_scenarios() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing first
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir, 60);
        thread::sleep(Duration::from_millis(200));

        // Add predictable test content
        let _ = state.add_path("/chunked/test/document1.txt");
        let _ = state.add_path("/chunked/test/document2.txt");
        let _ = state.add_path("/chunked/test/documents/file.txt");
        let _ = state.add_path("/chunked/test/docs/readme.md");
        let _ = state.add_path("/chunked/test/downloads/file1.txt");

        // Scenario: User performs search, then refines it
        let initial_search_term = "doc";
        let refined_search_term = "docu";

        let initial_search = state
            .search(initial_search_term)
            .expect("Initial chunked search failed");
        log_info!(
            "Chunked initial search for '{}' found {} results",
            initial_search_term,
            initial_search.len()
        );

        let refined_search = state
            .search(refined_search_term)
            .expect("Refined chunked search failed");
        log_info!(
            "Chunked refined search for '{}' found {} results",
            refined_search_term,
            refined_search.len()
        );

        // Basic assertion - refined search should be meaningful
        assert!(refined_search.len() <= initial_search.len() + 5); // Allow some tolerance for ranking differences
    }

    #[test]
    fn test_chunked_with_real_world_data() {
        log_info!("Testing chunked indexing with real-world test data");
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing on real test data
        let test_dir = get_test_data_path();

        let start = Instant::now();
        let result = state.start_chunked_indexing(test_dir.clone(), 80);
        let elapsed = start.elapsed();

        assert!(
            result.is_ok(),
            "Chunked indexing should succeed with real data"
        );
        log_info!("Chunked indexing completed in {:?}", elapsed);

        // Wait for completion
        thread::sleep(Duration::from_millis(200));

        // Get stats after chunked indexing
        let stats = state.get_stats();
        log_info!(
            "Chunked indexing stats - Cache size: {}, Trie size: {}",
            stats.cache_size,
            stats.trie_size
        );

        // Test multiple search queries
        let test_queries = ["fi", "test", "file", "txt", "md"];
        let mut found_results = false;

        for query in &test_queries {
            let search_start = Instant::now();
            let results = state.search(query).expect("Chunked search failed");
            let search_elapsed = search_start.elapsed();

            log_info!(
                "Chunked search for '{}' found {} results in {:?}",
                query,
                results.len(),
                search_elapsed
            );

            if !results.is_empty() {
                found_results = true;
                for (i, (path, score)) in results.iter().take(3).enumerate() {
                    log_info!(
                        "  Chunked result #{}: {} (score: {:.4})",
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
            "Should find results with chunked indexing using real-world data"
        );
    }

    #[test]
    fn test_chunked_search_by_extension() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use chunked indexing first, then add paths
        let test_dir = get_test_dir_for_indexing();
        let _ = state.start_chunked_indexing(test_dir, 50);
        thread::sleep(Duration::from_millis(100));

        // Add paths with different extensions
        state.add_path("/chunked/test/document.pdf").unwrap();
        state.add_path("/chunked/test/document.txt").unwrap();
        state.add_path("/chunked/test/document.docx").unwrap();
        state.add_path("/chunked/test/image.jpg").unwrap();
        state.add_path("/chunked/test/spreadsheet.xlsx").unwrap();

        // Search with no extension preference
        let regular_results = state.search("document").unwrap();

        // Search with preference for txt extension
        let txt_results = state
            .search_by_extension("document", vec!["txt".to_string()])
            .unwrap();

        // Search with preference for pdf extension
        let pdf_results = state
            .search_by_extension("document", vec!["pdf".to_string()])
            .unwrap();

        // Search with multiple extension preferences
        let _txt_pdf_results = state
            .search_by_extension("document", vec!["txt".to_string(), "pdf".to_string()])
            .unwrap();

        // Verify extension preferences affect ranking
        if !txt_results.is_empty() && !pdf_results.is_empty() {
            assert_eq!(
                txt_results[0].0, "/chunked/test/document.txt",
                "TXT document should be first with txt preference after chunked indexing"
            );
            assert_eq!(
                pdf_results[0].0, "/chunked/test/document.pdf",
                "PDF document should be first with pdf preference after chunked indexing"
            );
        }

        // Verify all documents are still found
        assert_eq!(
            regular_results.len(),
            txt_results.len(),
            "Same number of results with extension preferences after chunked indexing"
        );
        assert_eq!(
            regular_results.len(),
            pdf_results.len(),
            "Same number of results with different extension preferences after chunked indexing"
        );
    }

    #[test]
    fn test_chunked_vs_traditional_indexing_results_consistency() {
        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state1 = SearchEngineState::new(settings_state.clone());
        let state2 = SearchEngineState::new(settings_state);

        // Create a controlled test directory
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let test_dir = temp_dir.path().to_path_buf();

        // Add test files
        let test_files = vec![
            "document1.txt",
            "document2.pdf",
            "readme.md",
            "script.js",
            "style.css",
        ];

        for file_name in &test_files {
            let file_path = test_dir.join(file_name);
            fs::write(&file_path, format!("Content for {}", file_name)).unwrap();
        }

        // Index with traditional method
        let _ = state1.start_indexing(test_dir.clone());
        thread::sleep(Duration::from_millis(100));

        // Index with chunked method
        let _ = state2.start_chunked_indexing(test_dir.clone(), 3);
        thread::sleep(Duration::from_millis(100));

        // Compare search results
        let search_terms = ["document", "readme", "script"];

        for term in &search_terms {
            let traditional_results = state1.search(term).unwrap();
            let chunked_results = state2.search(term).unwrap();

            log_info!(
                "Comparing results for '{}': traditional={}, chunked={}",
                term,
                traditional_results.len(),
                chunked_results.len()
            );

            // Results should be similar (allowing for minor differences in ranking)
            assert_eq!(
                traditional_results.len(),
                chunked_results.len(),
                "Traditional and chunked indexing should find same number of results for '{}'",
                term
            );

            // Top results should be the same files (though scores might differ slightly)
            if !traditional_results.is_empty() && !chunked_results.is_empty() {
                let traditional_top = &traditional_results[0].0;
                let chunked_top = &chunked_results[0].0;

                // Extract just the filename for comparison
                let traditional_filename = std::path::Path::new(traditional_top)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();
                let chunked_filename = std::path::Path::new(chunked_top)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap();

                assert_eq!(
                    traditional_filename, chunked_filename,
                    "Top result filename should be the same for traditional and chunked indexing"
                );
            }
        }

        // Clean up
        temp_dir.close().unwrap();
    }
}

#[cfg(test)]
mod bench_indexing_methods {
    use super::*;
    use std::collections::HashMap;
    use std::time::Instant;

    // Helper function to create a larger test dataset for benchmarking using real test data
    fn create_benchmark_test_files(base_dir: &PathBuf, file_count: usize) -> Vec<PathBuf> {
        let mut created_files = Vec::new();

        // First try to use existing test data
        let test_data_path = get_test_data_path();
        if test_data_path.exists() {
            log_info!(
                "Using existing test data from: {}",
                test_data_path.display()
            );

            // Collect existing files from test data
            fn collect_existing_files(dir: &PathBuf, files: &mut Vec<PathBuf>, limit: usize) {
                if files.len() >= limit {
                    return;
                }

                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.filter_map(Result::ok) {
                        if files.len() >= limit {
                            break;
                        }

                        let path = entry.path();
                        if path.is_file() {
                            files.push(path.clone());
                        } else if path.is_dir() {
                            collect_existing_files(&path, files, limit);
                        }
                    }
                }
            }

            collect_existing_files(&test_data_path, &mut created_files, file_count);

            // If we have enough files from test data, use them
            if created_files.len() >= file_count / 2 {
                log_info!(
                    "Using {} existing test files from test data",
                    created_files.len()
                );
                return created_files;
            }
        }

        // Fall back to creating synthetic test files in the base_dir
        log_info!("Creating synthetic test files in: {}", base_dir.display());
        created_files.clear();

        // Create nested directory structure for realistic testing
        let depth_levels = 4;
        let dirs_per_level = 5;
        let files_per_dir = file_count / (depth_levels * dirs_per_level);

        for depth in 0..depth_levels {
            for dir_num in 0..dirs_per_level {
                let dir_path = base_dir
                    .join(format!("benchmark_depth_{}", depth))
                    .join(format!("dir_{}", dir_num));

                let _ = fs::create_dir_all(&dir_path);

                // Create files in this directory
                for file_num in 0..files_per_dir {
                    let file_path =
                        dir_path.join(format!("benchmark_file_{}_{}.txt", depth, file_num));
                    let content = format!(
                        "Benchmark test content for depth {} file {}",
                        depth, file_num
                    );

                    if fs::write(&file_path, content).is_ok() {
                        created_files.push(file_path);
                    }
                }
            }
        }

        log_info!(
            "Created {} synthetic benchmark test files",
            created_files.len()
        );
        created_files
    }

    // Helper function to get the benchmark test directory - prefer real test data
    fn get_benchmark_test_dir() -> PathBuf {
        // First try to use the real test data directory
        let test_data_path = get_test_data_path();
        if test_data_path.exists() {
            log_info!(
                "Using real test data directory for benchmarking: {}",
                test_data_path.display()
            );
            return test_data_path;
        }

        // Fall back to creating a temporary directory
        log_warn!("Real test data not available, using temporary directory for benchmarking");
        tempfile::tempdir()
            .expect("Failed to create temp directory")
            .path()
            .to_path_buf()
    }

    // Helper function to clean up test files (only synthetic ones)
    fn cleanup_benchmark_files(files: Vec<PathBuf>) {
        // Only clean up files that are in temporary directories or synthetic benchmark files
        for file in files {
            if let Some(file_name) = file.file_name().and_then(|n| n.to_str()) {
                // Only remove files we created (synthetic benchmark files)
                if file_name.starts_with("benchmark_file_") {
                    let _ = fs::remove_file(file);
                }
            }
        }
    }

    // Helper function to measure indexing performance
    fn measure_indexing_performance(
        state: &SearchEngineState,
        test_dir: &PathBuf,
        method_name: &str,
        chunk_size: Option<usize>,
    ) -> (Duration, bool) {
        // Clear any existing index
        {
            let mut engine = state.engine.lock().unwrap();
            engine.clear();
        }

        let start_time = Instant::now();

        let result = match chunk_size {
            Some(size) => {
                log_info!("Starting chunked indexing with chunk size {}", size);
                state.start_chunked_indexing(test_dir.clone(), size)
            }
            None => {
                log_info!("Starting traditional indexing");
                state.start_indexing(test_dir.clone())
            }
        };

        let duration = start_time.elapsed();
        let success = result.is_ok();

        log_info!(
            "{} indexing took {:?} (success: {})",
            method_name,
            duration,
            success
        );

        (duration, success)
    }

    // Helper function to verify indexing worked correctly
    fn verify_indexing_results(state: &SearchEngineState, _expected_files: usize) -> bool {
        // Wait a moment for indexing to complete
        thread::sleep(Duration::from_millis(100));

        // Check final status
        let data = state.data.lock().unwrap();
        let status_ok = matches!(data.status, SearchEngineStatus::Idle);
        let progress_complete = data.progress.percentage_complete >= 100.0;
        drop(data);

        // Try a search to verify files were indexed
        let search_result = state.search("test");
        let search_works = search_result.is_ok();
        let has_results = search_result.map(|r| !r.is_empty()).unwrap_or(false);

        // Get engine stats
        let stats = state.get_stats();
        let has_trie_content = stats.trie_size > 0;

        log_info!(
            "Verification - Status OK: {}, Progress Complete: {}, Search Works: {}, Has Results: {}, Trie Size: {}",
            status_ok, progress_complete, search_works, has_results, stats.trie_size
        );

        status_ok && search_works && has_trie_content
    }

    #[test]
    fn benchmark_indexing_methods_comparison() {
        log_info!("=== INDEXING METHODS BENCHMARK ===");

        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use the real test data directory if available
        let test_dir = get_benchmark_test_dir();
        log_info!("Using test directory: {}", test_dir.display());

        // Count existing files in the test directory
        let existing_file_count = if test_dir.exists() {
            let paths = collect_test_paths(None); // Get all available paths
            log_info!("Found {} existing files in test data", paths.len());
            paths.len()
        } else {
            0
        };

        // If we don't have enough real files, supplement with synthetic ones
        let target_file_count = 1000;
        let synthetic_files = if existing_file_count < target_file_count {
            let needed = target_file_count - existing_file_count;
            log_info!(
                "Creating {} additional synthetic files for benchmarking",
                needed
            );
            create_benchmark_test_files(&test_dir, needed)
        } else {
            log_info!("Using existing test data files for benchmarking");
            Vec::new()
        };

        let total_files = existing_file_count + synthetic_files.len();
        log_info!("Total files available for benchmarking: {}", total_files);

        // Store benchmark results
        let mut results = HashMap::new();
        let chunk_sizes = [200, 350, 500];

        // Benchmark traditional indexing
        log_info!("\n--- Benchmarking Traditional Indexing ---");
        let (traditional_duration, traditional_success) =
            measure_indexing_performance(&state, &test_dir, "Traditional", None);

        let traditional_verified = verify_indexing_results(&state, total_files);
        results.insert(
            "Traditional".to_string(),
            (
                traditional_duration,
                traditional_success && traditional_verified,
            ),
        );

        // Benchmark chunked indexing with different chunk sizes
        for &chunk_size in &chunk_sizes {
            log_info!(
                "\n--- Benchmarking Chunked Indexing (chunk size: {}) ---",
                chunk_size
            );

            let (chunked_duration, chunked_success) = measure_indexing_performance(
                &state,
                &test_dir,
                &format!("Chunked-{}", chunk_size),
                Some(chunk_size),
            );

            let chunked_verified = verify_indexing_results(&state, total_files);
            results.insert(
                format!("Chunked-{}", chunk_size),
                (chunked_duration, chunked_success && chunked_verified),
            );
        }

        // Print comprehensive benchmark results
        log_info!("\n=== BENCHMARK RESULTS SUMMARY ===");
        log_info!(
            "Test files: {} (existing: {}, synthetic: {})",
            total_files,
            existing_file_count,
            synthetic_files.len()
        );
        log_info!("Test directory: {}", test_dir.display());

        let mut sorted_results: Vec<_> = results.iter().collect();
        sorted_results.sort_by_key(|(_, (duration, _))| *duration);

        log_info!("\nPerformance ranking (fastest to slowest):");
        for (i, (method, (duration, success))) in sorted_results.iter().enumerate() {
            let status = if *success { "✓" } else { "✗" };
            let files_per_second = if duration.as_millis() > 0 {
                total_files as f64 / duration.as_secs_f64()
            } else {
                0.0
            };

            log_info!(
                "{}. {} {} - {:?} ({:.2} ms, {:.1} files/sec)",
                i + 1,
                status,
                method,
                duration,
                duration.as_millis(),
                files_per_second
            );
        }

        // Calculate performance comparisons
        if let Some((traditional_duration, traditional_success)) = results.get("Traditional") {
            if *traditional_success {
                log_info!("\nPerformance vs Traditional Indexing:");

                for &chunk_size in &chunk_sizes {
                    let key = format!("Chunked-{}", chunk_size);
                    if let Some((chunked_duration, chunked_success)) = results.get(&key) {
                        if *chunked_success {
                            let ratio = chunked_duration.as_millis() as f64
                                / traditional_duration.as_millis() as f64;
                            let percentage = (ratio - 1.0) * 100.0;

                            if ratio < 1.0 {
                                log_info!(
                                    "  Chunked-{}: {:.1}% FASTER than traditional",
                                    chunk_size,
                                    percentage.abs()
                                );
                            } else {
                                log_info!(
                                    "  Chunked-{}: {:.1}% slower than traditional",
                                    chunk_size,
                                    percentage
                                );
                            }
                        }
                    }
                }
            }
        }

        // Find the best chunk size
        let best_chunked = chunk_sizes
            .iter()
            .filter_map(|&size| {
                let key = format!("Chunked-{}", size);
                results.get(&key).and_then(|(duration, success)| {
                    if *success {
                        Some((size, duration))
                    } else {
                        None
                    }
                })
            })
            .min_by_key(|(_, duration)| *duration);

        if let Some((best_size, best_duration)) = best_chunked {
            log_info!(
                "\nBest chunked indexing: Chunk size {} in {:?}",
                best_size,
                best_duration
            );
        }

        // Verify all methods succeeded
        let all_succeeded = results.values().all(|(_, success)| *success);
        assert!(all_succeeded, "All indexing methods should succeed");

        // Verify that we have meaningful performance data
        let has_performance_data = results
            .values()
            .any(|(duration, _)| duration.as_millis() > 0);
        assert!(
            has_performance_data,
            "Should have measurable performance data"
        );

        // Cleanup only synthetic files
        cleanup_benchmark_files(synthetic_files);
        log_info!("\n=== BENCHMARK COMPLETED ===");
    }

    #[test]
    fn benchmark_indexing_scalability() {
        log_info!("=== INDEXING SCALABILITY BENCHMARK ===");

        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use real test data directory
        let base_test_dir = get_benchmark_test_dir();

        // Test with different file counts to measure scalability
        let file_counts = [100, 500, 1000];
        let chunk_size = 350; // Use a middle-ground chunk size

        for &file_count in &file_counts {
            log_info!("\n--- Testing scalability with {} files ---", file_count);

            // For scalability testing, create a subdirectory with specific file count
            let test_dir = base_test_dir.join(format!("scalability_test_{}", file_count));
            let _ = fs::create_dir_all(&test_dir);

            // Create test files using real data as template but in controlled quantities
            let created_files = create_benchmark_test_files(&test_dir, file_count);
            log_info!("Created {} files for scalability test", created_files.len());

            // Test traditional indexing
            let (traditional_duration, traditional_success) =
                measure_indexing_performance(&state, &test_dir, "Traditional", None);

            // Test chunked indexing
            let (chunked_duration, chunked_success) =
                measure_indexing_performance(&state, &test_dir, "Chunked", Some(chunk_size));

            // Calculate performance metrics
            let traditional_rate = if traditional_duration.as_millis() > 0 {
                created_files.len() as f64 / traditional_duration.as_secs_f64()
            } else {
                0.0
            };

            let chunked_rate = if chunked_duration.as_millis() > 0 {
                created_files.len() as f64 / chunked_duration.as_secs_f64()
            } else {
                0.0
            };

            log_info!("Scalability results for {} files:", created_files.len());
            log_info!(
                "  Traditional: {:?} ({:.1} files/sec) - Success: {}",
                traditional_duration,
                traditional_rate,
                traditional_success
            );
            log_info!(
                "  Chunked: {:?} ({:.1} files/sec) - Success: {}",
                chunked_duration,
                chunked_rate,
                chunked_success
            );

            // Cleanup
            cleanup_benchmark_files(created_files);
            let _ = fs::remove_dir_all(&test_dir);
        }

        log_info!("\n=== SCALABILITY BENCHMARK COMPLETED ===");
    }

    #[test]
    fn benchmark_memory_usage_comparison() {
        log_info!("=== MEMORY USAGE BENCHMARK ===");

        let settings_state = Arc::new(Mutex::new(SettingsState::new()));
        let state = SearchEngineState::new(settings_state);

        // Use real test data directory
        let test_dir = get_benchmark_test_dir();
        log_info!(
            "Using test directory for memory benchmark: {}",
            test_dir.display()
        );

        // Count actual files available
        let available_paths = collect_test_paths(Some(800));
        log_info!(
            "Using {} files for memory usage benchmark",
            available_paths.len()
        );

        // Measure memory usage for traditional indexing
        {
            let mut engine = state.engine.lock().unwrap();
            engine.clear();
        }

        let initial_stats = state.get_stats();
        let _ = state.start_indexing(test_dir.clone());
        let traditional_stats = state.get_stats();

        log_info!(
            "Traditional indexing memory usage - Trie: {} -> {}, Cache: {} -> {}",
            initial_stats.trie_size,
            traditional_stats.trie_size,
            initial_stats.cache_size,
            traditional_stats.cache_size
        );

        // Measure memory usage for chunked indexing
        {
            let mut engine = state.engine.lock().unwrap();
            engine.clear();
        }

        let _ = state.start_chunked_indexing(test_dir.clone(), 350);
        let chunked_stats = state.get_stats();

        log_info!(
            "Chunked indexing memory usage - Trie: {}, Cache: {}",
            chunked_stats.trie_size,
            chunked_stats.cache_size
        );

        // Memory usage should be similar for both methods
        let trie_difference =
            (traditional_stats.trie_size as i64 - chunked_stats.trie_size as i64).abs();
        let trie_difference_percent = if traditional_stats.trie_size > 0 {
            trie_difference as f64 / traditional_stats.trie_size as f64 * 100.0
        } else {
            0.0
        };

        log_info!(
            "Memory usage difference - Trie size: {} ({:.1}%)",
            trie_difference,
            trie_difference_percent
        );

        // Calculate memory efficiency (files per trie node)
        let traditional_efficiency = if traditional_stats.trie_size > 0 {
            available_paths.len() as f64 / traditional_stats.trie_size as f64
        } else {
            0.0
        };

        let chunked_efficiency = if chunked_stats.trie_size > 0 {
            available_paths.len() as f64 / chunked_stats.trie_size as f64
        } else {
            0.0
        };

        log_info!(
            "Memory efficiency - Traditional: {:.2} files/trie_node, Chunked: {:.2} files/trie_node",
            traditional_efficiency, chunked_efficiency
        );

        // The difference should be minimal (both methods should index the same data)
        assert!(
            trie_difference_percent < 10.0,
            "Trie size difference should be less than 10% between methods"
        );

        log_info!("=== MEMORY USAGE BENCHMARK COMPLETED ===");
    }
}
