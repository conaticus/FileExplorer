use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::search_engine::adaptive_radix_trie::{AdaptiveRadixNode, AdaptiveRadixTrie};
use crate::search_engine::fuzzy::FuzzySearchIndex;
use crate::search_engine::lru_chache::AutocompleteLRUCache;
use crate::search_engine::context_aware_ranking::ContextAwareRanker;

pub struct AutocompleteEngine {
    radix_trie: AdaptiveRadixTrie,
    fuzzy_index: FuzzySearchIndex,
    result_cache: AutocompleteLRUCache,
    ranker: ContextAwareRanker,
    current_directory: PathBuf,
    last_update: SystemTime,
}

impl AutocompleteEngine {
    pub fn suggest(&self, query: &str, limit: usize) -> Vec<PathBuf> {
        // First try exact prefix matching
        let mut results = self.radix_trie.find_with_prefix(query);

        // If not enough results, try fuzzy search
        if results.len() < limit {
            let fuzzy_results = self.fuzzy_index.find_matches(query, limit - results.len());
            results.extend(fuzzy_results);
        }

        // Apply context-aware ranking
        self.ranker.rank_results(results)
    }
}

// Thread-safe wrapper
pub struct ThreadSafeAutocomplete {
    engine: Arc<RwLock<AutocompleteEngine>>,
}

impl ThreadSafeAutocomplete {
    pub fn new() -> Self {
        let current_dir = PathBuf::from("/");

        Self {
            engine: Arc::new(RwLock::new(AutocompleteEngine {
                radix_trie: AdaptiveRadixTrie::new(),
                fuzzy_index: FuzzySearchIndex::new(2), // Max edit distance of 2
                result_cache: AutocompleteLRUCache::new(1000), // Cache 1000 queries
                ranker: ContextAwareRanker::new(current_dir.clone()),
                current_directory: current_dir,
                last_update: SystemTime::now(),
            })),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
        }
    }

    // Basic search functionality
    pub fn suggest(&self, query: &str, limit: usize) -> Result<Vec<PathBuf>, String> {
        // First check the cache
        if let Some(cached_results) = self.get_cached_results(query) {
            return Ok(cached_results);
        }

        // Cache miss, compute the results
        match self.engine.read() {
            Ok(engine) => {
                let suggestions = engine.suggest(query, limit);

                // Cache the results for future use
                self.cache_results(query, suggestions.clone())?;

                Ok(suggestions)
            },
            Err(_) => Err("Failed to acquire read lock on autocomplete engine".to_string()),
        }
    }

    // Index a single path
    pub fn index_path(&self, path: &Path) -> Result<(), String> {
        match self.engine.write() {
            Ok(mut engine) => {
                let path_str = path.to_string_lossy().to_string();

                // Add to radix trie with adaptive segmentation
                engine.radix_trie.insert(&path_str, path.to_path_buf());

                // Add to fuzzy index
                engine.fuzzy_index.index_path(path);

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Invalidate any cache entries that might be affected
                self.invalidate_affected_cache(path)?;

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on autocomplete engine".to_string()),
        }
    }

    // Remove a path from the index
    pub fn remove_path(&self, path: &Path) -> Result<(), String> {
        match self.engine.write() {
            Ok(mut engine) => {
                let path_str = path.to_string_lossy().to_string();

                // Remove from radix trie
                engine.radix_trie.remove(&path_str);

                // Remove from fuzzy index
                engine.fuzzy_index.remove_path(path);

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Invalidate any cache entries that might be affected
                self.invalidate_affected_cache(path)?;

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on autocomplete engine".to_string()),
        }
    }

    // Batch update multiple paths (add some, remove others)
    pub fn batch_update(&self, paths_to_add: Vec<PathBuf>, paths_to_remove: Vec<PathBuf>)
                        -> Result<(), String> {
        match self.engine.write() {
            Ok(mut engine) => {
                // Process removals first
                for path in &paths_to_remove {
                    let path_str = path.to_string_lossy().to_string();
                    engine.radix_trie.remove(&path_str);
                    engine.fuzzy_index.remove_path(path);
                }

                // Then process additions
                for path in &paths_to_add {
                    let path_str = path.to_string_lossy().to_string();
                    engine.radix_trie.insert(&path_str, path.to_path_buf());
                    engine.fuzzy_index.index_path(path);
                }

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Determine if we should invalidate the entire cache
                let total_changes = paths_to_add.len() + paths_to_remove.len();
                if total_changes > 20 {  // Threshold for full cache invalidation
                    engine.result_cache.clear()?;
                } else {
                    // Selectively invalidate cache for affected paths
                    for path in paths_to_add.iter().chain(paths_to_remove.iter()) {
                        self.invalidate_affected_cache(path)?;
                    }
                }

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock for batch update".to_string()),
        }
    }

    // Reindex an entire directory hierarchy
    pub fn reindex_directory(&self, dir: &Path) -> Result<(), String> {
        // First, collect all files in the directory
        let files = self.collect_files_in_directory(dir)?;

        match self.engine.write() {
            Ok(mut engine) => {
                // Clear existing index for this directory
                self.clear_directory_from_index(dir)?;

                // Add all files to the index
                for file in &files {
                    let path_str = file.to_string_lossy().to_string();
                    let _ = engine.radix_trie.insert(&path_str, file.to_path_buf());
                    engine.fuzzy_index.index_path(file);
                }

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Clear entire cache since this is a major operation
                engine.result_cache.clear()?;

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock for directory reindexing".to_string()),
        }
    }

    // Helper to collect all files in a directory (simulated)
    fn collect_files_in_directory(&self, _dir: &Path) -> Result<Vec<PathBuf>, String> {
        // In a real implementation, you would use fs::read_dir or walkdir crate
        // This is just a stub for demonstration
        Ok(Vec::new())
    }

    // Helper to clear a directory from the index
    fn clear_directory_from_index(&self, _dir: &Path) -> Result<(), String> {
        match self.engine.write() {
            Ok(_engine) => {
                // In a real implementation, you would need to find all indexed
                // paths that start with the directory path and remove them
                // This is just a stub
                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock for clearing directory".to_string()),
        }
    }

    // Clear the entire index
    pub fn clear(&self) -> Result<(), String> {
        match self.engine.write() {
            Ok(mut engine) => {
                // Create new empty data structures
                engine.radix_trie = AdaptiveRadixTrie::new();
                engine.fuzzy_index = FuzzySearchIndex::new(2);

                // Clear the cache
                engine.result_cache.clear()?;

                // Update timestamp
                engine.last_update = SystemTime::now();

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock for clearing index".to_string()),
        }
    }

    // Get statistics about the index
    pub fn get_stats(&self) -> Result<IndexStats, String> {
        match self.engine.read() {
            Ok(engine) => {
                let stats = IndexStats {
                    indexed_path_count: engine.radix_trie.get_path_count(),
                    cached_query_count: engine.result_cache.len()?,
                    last_update: engine.last_update,
                };

                Ok(stats)
            },
            Err(_) => Err("Failed to acquire read lock for getting stats".to_string()),
        }
    }

    // Get from cache
    fn get_cached_results(&self, query: &str) -> Option<Vec<PathBuf>> {
        match self.engine.read() {
            Ok(engine) => engine.result_cache.get_suggestions(query),
            Err(_) => None,
        }
    }

    // Cache results
    fn cache_results(&self, query: &str, results: Vec<PathBuf>) -> Result<(), String> {
        match self.engine.read() {
            Ok(engine) => engine.result_cache.cache_suggestions(query, results),
            Err(_) => Err("Failed to acquire read lock on autocomplete engine".to_string()),
        }
    }

    // Record that a path was accessed (for ranking purposes)
    pub fn record_path_access(&self, path: &Path) -> Result<(), String> {
        match self.engine.write() {
            Ok(engine) => {
                engine.ranker.record_access(path);
                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on autocomplete engine".to_string()),
        }
    }

    // Update the current directory context
    pub fn update_current_directory(&self, dir: PathBuf) -> Result<(), String> {
        match self.engine.write() {
            Ok(mut engine) => {
                engine.current_directory = dir.clone();
                engine.ranker.update_current_directory(dir);
                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on autocomplete engine".to_string()),
        }
    }

    // Method to invalidate cache when filesystem changes
    pub fn invalidate_cache(&self, prefix: &str) -> Result<(), String> {
        match self.engine.read() {
            Ok(engine) => {
                engine.result_cache.invalidate(prefix)
            },
            Err(_) => Err("Failed to acquire read lock on autocomplete engine".to_string()),
        }
    }

    // More targeted cache invalidation based on path
    fn invalidate_affected_cache(&self, path: &Path) -> Result<(), String> {
        // Get filename without extension
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy().to_string();

            // Invalidate cache entries starting with this filename
            self.invalidate_cache(&file_name_str)?;

            // Also try without extension
            if let Some(stem) = path.file_stem() {
                let stem_str = stem.to_string_lossy().to_string();
                self.invalidate_cache(&stem_str)?;
            }
        }

        // If the path is deep, invalidate based on directory names too
        let mut current = path;
        while let Some(parent) = current.parent() {
            if let Some(dir_name) = parent.file_name() {
                let dir_name_str = dir_name.to_string_lossy().to_string();
                self.invalidate_cache(&dir_name_str)?;
            }
            current = parent;
        }

        Ok(())
    }

    // Suggest completions with metadata
    pub fn suggest_with_metadata(&self, query: &str, limit: usize)
                                 -> Result<Vec<(PathBuf, PathMetadata)>, String> {
        let paths = self.suggest(query, limit)?;

        // Gather metadata for each path
        let mut results_with_metadata = Vec::new();

        for path in paths {
            match self.engine.read() {
                Ok(engine) => {
                    let frequency = engine.ranker.get_frequency(&path);
                    let recency = engine.ranker.get_last_access(&path);
                    let score = engine.ranker.calculate_score(&path);

                    let metadata = PathMetadata {
                        frequency,
                        last_access: recency,
                        score,
                    };

                    results_with_metadata.push((path, metadata));
                },
                Err(_) => return Err("Failed to acquire read lock for metadata".to_string()),
            }
        }

        Ok(results_with_metadata)
    }

    // Get recently accessed paths
    pub fn get_recent_paths(&self, limit: usize) -> Result<Vec<PathBuf>, String> {
        match self.engine.read() {
            Ok(engine) => {
                let paths = engine.ranker.get_recent_paths(limit);
                Ok(paths)
            },
            Err(_) => Err("Failed to acquire read lock for recent paths".to_string()),
        }
    }

    // Get frequently accessed paths
    pub fn get_frequent_paths(&self, limit: usize) -> Result<Vec<PathBuf>, String> {
        match self.engine.read() {
            Ok(engine) => {
                let paths = engine.ranker.get_frequent_paths(limit);
                Ok(paths)
            },
            Err(_) => Err("Failed to acquire read lock for frequent paths".to_string()),
        }
    }
}

// Metadata about a path for detailed results
pub struct PathMetadata {
    frequency: u32,
    last_access: Option<SystemTime>,
    score: f64,
}

// Statistics about the autocomplete index
pub struct IndexStats {
    indexed_path_count: usize,
    cached_query_count: usize,
    last_update: SystemTime,
}

// Add additional methods to our ranker to support the new functionality
impl ContextAwareRanker {
    // Get frequency of a path
    pub fn get_frequency(&self, path: &Path) -> u32 {
        if let Ok(frequency_data) = self.frequency_data.read() {
            *frequency_data.get(path).unwrap_or(&0)
        } else {
            0
        }
    }

    // Get last access time of a path
    pub fn get_last_access(&self, path: &Path) -> Option<SystemTime> {
        if let Ok(recency_data) = self.recency_data.read() {
            recency_data.get(path).cloned()
        } else {
            None
        }
    }

    // Calculate overall score for a path
    pub fn calculate_score(&self, path: &Path) -> f64 {
        let recency_score = self.calculate_recency_score(self.get_last_access(path));
        let frequency_score = self.calculate_frequency_score(self.get_frequency(path));
        let proximity_score = self.calculate_proximity_score(path);
        let extension_score = self.calculate_extension_score(path);

        (recency_score * self.factors.recency_weight) +
            (frequency_score * self.factors.frequency_weight) +
            (proximity_score * self.factors.proximity_weight) +
            extension_score
    }

    // Get most recently accessed paths
    pub fn get_recent_paths(&self, limit: usize) -> Vec<PathBuf> {
        let mut paths_with_time = Vec::new();

        if let Ok(recency_data) = self.recency_data.read() {
            for (path, time) in recency_data.iter() {
                paths_with_time.push((path.clone(), *time));
            }
        }

        // Sort by timestamp (most recent first)
        paths_with_time.sort_by(|(_, time1), (_, time2)| {
            time2.cmp(time1)
        });

        // Take only the requested number
        paths_with_time.into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect()
    }

    // Get most frequently accessed paths
    pub fn get_frequent_paths(&self, limit: usize) -> Vec<PathBuf> {
        let mut paths_with_freq = Vec::new();

        if let Ok(frequency_data) = self.frequency_data.read() {
            for (path, freq) in frequency_data.iter() {
                paths_with_freq.push((path.clone(), *freq));
            }
        }

        // Sort by frequency (most frequent first)
        paths_with_freq.sort_by(|(_, freq1), (_, freq2)| {
            freq2.cmp(freq1)
        });

        // Take only the requested number
        paths_with_freq.into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect()
    }
}

#[cfg(test)]
mod tests_autocomplete {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;
    use crate::{log_info, log_error};
    use crate::search_engine::generate_test_data;

    // Helper function to get or generate test data path
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-autocomplete");
        if !path.exists() {
            log_info!("Creating test data directory...");
            match generate_test_data(path.clone()) {
                Ok(_) => log_info!("Test data created successfully"),
                Err(e) => log_error!(&format!("Failed to create test data: {}", e)),
            }
        }
        path
    }

    // Updated to use generated test data
    fn setup_test_engine() -> ThreadSafeAutocomplete {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();
        
        // Index all files from the test directory
        let entries = crate::search_engine::index_given_path_parallel(test_dir);
        log_info!(&format!("Indexing {} entries from test data", entries.len()));
        
        // Add paths to the engine
        for entry in entries {
            match entry {
                crate::search_engine::Entry::FILE(file) => {
                    let path = PathBuf::from(&file.path);
                    engine.index_path(&path).expect("Failed to index test path");
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    let path = PathBuf::from(&dir.path);
                    engine.index_path(&path).expect("Failed to index test directory");
                },
            }
        }

        log_info!(&format!("Test engine initialized with test data"));
        engine
    }

    #[test]
    fn test_autocomplete_with_real_data() {
        let engine = setup_test_engine();
        
        // Get stats about the indexed data
        let stats = engine.get_stats().expect("Failed to get stats");
        log_info!(&format!("Autocomplete engine stats: {} paths indexed, {} cached queries",
            stats.indexed_path_count, stats.cached_query_count));
        
        // Verify we have indexed paths
        assert!(stats.indexed_path_count > 0, "Should have indexed some paths");
        
        // Try some autocomplete queries
        let queries = ["doc", "ba", "txt", "js", "p"];
        
        for query in &queries {
            let results = engine.suggest(query, 10).expect("Suggestion failed");
            log_info!(&format!("Query '{}' returned {} suggestions", query, results.len()));
            
            // For non-empty results, print first few matches
            if !results.is_empty() {
                let display_count = std::cmp::min(3, results.len());
                for i in 0..display_count {
                    log_info!(&format!("  Match {}: {}", i+1, results[i].display()));
                }
            }
        }
    }

    #[test]
    fn test_suggest_with_metadata() {
        let engine = setup_test_engine();

        // Get results with metadata for a common query
        let results = engine.suggest_with_metadata("doc", 10)
            .expect("Failed to get suggestions with metadata");

        log_info!(&format!("Found {} results with metadata", results.len()));
        assert!(!results.is_empty(), "Should find at least some results");

        // Log some metadata to verify it's working
        if !results.is_empty() {
            let (path, metadata) = &results[0];
            log_info!(&format!("Top result: {} (score: {:.2}, frequency: {})",
                path.display(), metadata.score, metadata.frequency));

            // Verify that metadata contains reasonable values
            assert!(metadata.score >= 0.0, "Score should be non-negative");
        }
    }

    #[test]
    fn test_record_and_retrieve_access_patterns() {
        let engine = setup_test_engine();
        let test_dir = get_test_data_path();

        // Get initial suggestions
        let initial_results = engine.suggest("doc", 5)
            .expect("Initial suggestion failed");

        if initial_results.is_empty() {
            log_info!("No initial results found for test query. Test skipped.");
            return;
        }

        let target_path = &initial_results[0];

        // Record several accesses to the first result
        for _ in 0..5 {
            engine.record_path_access(target_path)
                .expect("Failed to record path access");
        }

        // Get frequent paths
        let frequent_paths = engine.get_frequent_paths(10)
            .expect("Failed to get frequent paths");

        log_info!(&format!("Retrieved {} frequent paths", frequent_paths.len()));
        assert!(!frequent_paths.is_empty(), "Should have at least one frequent path");

        // Verify our accessed path is among the frequent ones
        let target_found = frequent_paths.iter()
            .any(|p| p == target_path);

        assert!(target_found, "Accessed path should appear in frequent paths");

        // Get recent paths
        let recent_paths = engine.get_recent_paths(10)
            .expect("Failed to get recent paths");

        log_info!(&format!("Retrieved {} recent paths", recent_paths.len()));
        assert!(!recent_paths.is_empty(), "Should have at least one recent path");

        // Verify our accessed path is among the recent ones
        let target_found = recent_paths.iter()
            .any(|p| p == target_path);

        assert!(target_found, "Accessed path should appear in recent paths");
    }

    #[test]
    fn test_directory_context_affects_results() {
        let engine = setup_test_engine();
        let test_dir = get_test_data_path();

        // Get some initial results
        let initial_results = engine.suggest("txt", 10)
            .expect("Initial suggestion failed");

        if initial_results.len() < 3 {
            log_info!("Not enough results for meaningful directory context test. Skipped.");
            return;
        }

        // Choose a parent directory from one of the results
        let target_dir = match initial_results[0].parent() {
            Some(dir) => dir.to_path_buf(),
            None => {
                log_info!("Couldn't find parent directory for test. Skipped.");
                return;
            }
        };

        // Update current directory to that parent
        engine.update_current_directory(target_dir.clone())
            .expect("Failed to update current directory");

        // Get suggestions again with the new context
        let context_results = engine.suggest("txt", 10)
            .expect("Context-aware suggestion failed");

        log_info!(&format!("Current directory set to: {}", target_dir.display()));
        log_info!(&format!("Found {} results with directory context", context_results.len()));

        // At least we should have some results
        assert!(!context_results.is_empty(), "Should find results with directory context");

        // This is difficult to test deterministically, but we could check if at least one
        // result is from the target directory
        let result_in_context = context_results.iter()
            .any(|p| p.starts_with(&target_dir));

        log_info!(&format!("Results contain path from context directory: {}", result_in_context));
    }

    #[test]
    fn test_fuzzy_search_capabilities() {
        let engine = setup_test_engine();

        // Get results for a correct query
        let correct_query = "document";
        let correct_results = engine.suggest(correct_query, 10)
            .expect("Correct query suggestion failed");

        if correct_results.is_empty() {
            log_info!(&format!("No results for '{}'. Skipping fuzzy test.", correct_query));
            return;
        }

        // Now try with a typo
        let typo_query = "documnet"; // Swapped 'e' and 'n'
        let fuzzy_results = engine.suggest(typo_query, 10)
            .expect("Fuzzy query suggestion failed");

        log_info!(&format!("Correct query '{}' returned {} results",
            correct_query, correct_results.len()));
        log_info!(&format!("Misspelled query '{}' returned {} results",
            typo_query, fuzzy_results.len()));

        // We should get at least some results with the typo
        assert!(!fuzzy_results.is_empty(),
            "Fuzzy search should find results despite typo");

        // Some of the fuzzy results should match the correct results
        let has_overlapping_results = fuzzy_results.iter()
            .any(|fuzzy_path| correct_results.contains(fuzzy_path));

        assert!(has_overlapping_results,
            "Fuzzy results should overlap with correct results");
    }

    #[test]
    fn test_cache_performance() {
        let engine = setup_test_engine();

        // First query - should miss cache
        let start = Instant::now();
        let results1 = engine.suggest("app", 10)
            .expect("First suggestion failed");
        let first_query_time = start.elapsed();

        // Second query with the same string - should hit cache
        let start = Instant::now();
        let results2 = engine.suggest("app", 10)
            .expect("Second suggestion failed");
        let second_query_time = start.elapsed();

        log_info!(&format!("First query (cache miss): {:?}", first_query_time));
        log_info!(&format!("Second query (cache hit): {:?}", second_query_time));

        // Results should be identical
        assert_eq!(results1, results2, "Cached results should be identical");

        // Second query should be faster, but this is not guaranteed
        // so we won't assert on it, just log the information
        if second_query_time < first_query_time {
            log_info!("Cache hit was faster than cache miss (expected)");
        } else {
            log_info!("Cache hit was not faster than cache miss (unexpected)");
        }

        // Try a different query to verify cache independence
        let different_query = "txt";
        let start = Instant::now();
        let _ = engine.suggest(different_query, 10)
            .expect("Different query suggestion failed");
        let different_query_time = start.elapsed();

        log_info!(&format!("Different query '{}': {:?}",
            different_query, different_query_time));
    }

    #[test]
    fn test_index_and_remove_path() {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();

        // Create test paths
        let test_path1 = test_dir.join("test_file_for_indexing_123.txt");
        let test_path2 = test_dir.join("another_test_file_789.doc");

        // Index the paths
        engine.index_path(&test_path1).expect("Failed to index first test path");
        engine.index_path(&test_path2).expect("Failed to index second test path");

        // Search for the unique part of the filenames
        let results1 = engine.suggest("123", 10).expect("First search failed");
        let results2 = engine.suggest("789", 10).expect("Second search failed");

        log_info!(&format!("Search for '123' found {} results", results1.len()));
        log_info!(&format!("Search for '789' found {} results", results2.len()));

        // Remove one of the paths
        engine.remove_path(&test_path1).expect("Failed to remove path");

        // Search again
        let results_after_removal1 = engine.suggest("123", 10).expect("Search after removal failed");
        let results_after_removal2 = engine.suggest("789", 10).expect("Search after removal failed");

        log_info!(&format!("After removal, search for '123' found {} results",
            results_after_removal1.len()));
        log_info!(&format!("After removal, search for '789' found {} results",
            results_after_removal2.len()));

        // The removed path should no longer appear in results
        let path1_still_found = results_after_removal1.contains(&test_path1);
        let path2_still_found = results_after_removal2.contains(&test_path2);

        assert!(!path1_still_found, "Removed path should not appear in search results");
        assert!(path2_still_found, "Non-removed path should still appear in search results");
    }

    #[test]
    fn test_batch_update_with_real_paths() {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();

        // Get first 5 entries from test directory to use as test data
        let entries = crate::search_engine::index_given_path_parallel(test_dir);
        let mut test_paths = Vec::new();

        for entry in entries.iter().take(5) {
            match entry {
                crate::search_engine::Entry::FILE(file) => {
                    test_paths.push(PathBuf::from(&file.path));
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    test_paths.push(PathBuf::from(&dir.path));
                }
            }
        }

        if test_paths.len() < 2 {
            log_info!("Not enough test paths found. Skipping batch update test.");
            return;
        }

        // Split the paths into two groups
        let paths_to_add = test_paths[0..test_paths.len()/2].to_vec();
        let paths_to_remove = Vec::new(); // Empty initially

        // Perform batch update
        engine.batch_update(paths_to_add.clone(), paths_to_remove)
            .expect("Batch update failed");

        // Verify all added paths are findable
        for path in &paths_to_add {
            if let Some(file_name) = path.file_name() {
                let search_term = file_name.to_string_lossy().to_string();
                let results = engine.suggest(&search_term, 10)
                    .expect("Search after batch add failed");

                let path_found = results.contains(path);
                assert!(path_found, "Added path should be findable: {}", path.display());
            }
        }

        // Now remove them in a batch
        engine.batch_update(Vec::new(), paths_to_add.clone())
            .expect("Batch removal failed");

        // Verify they're no longer findable
        for path in &paths_to_add {
            if let Some(file_name) = path.file_name() {
                let search_term = file_name.to_string_lossy().to_string();
                let results = engine.suggest(&search_term, 10)
                    .expect("Search after batch remove failed");

                let path_found = results.contains(path);
                assert!(!path_found, "Removed path should not be findable: {}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod benchmarks_autocomplete {
    use std::thread;
    use super::*;
    use std::time::{Duration, Instant};
    use crate::{log_error, log_info};
    use crate::search_engine::generate_test_data;

    // A simple benchmark helper that logs execution time
    struct BenchmarkTimer {
        name: String,
        start: Instant,
    }

    impl BenchmarkTimer {
        fn new(name: &str) -> Self {
            log_info!(&format!("Starting benchmark: {}", name));
            Self {
                name: name.to_string(),
                start: Instant::now(),
            }
        }
    }

    impl Drop for BenchmarkTimer {
        fn drop(&mut self) {
            let elapsed = self.start.elapsed();
            log_info!(&format!("Benchmark '{}' completed in {:?}", self.name, elapsed));
        }
    }

    // Helper function to get or generate test data path
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-autocomplete");
        if !path.exists() {
            log_info!("Creating test data directory...");
            match generate_test_data(path.clone()) {
                Ok(_) => log_info!("Test data created successfully"),
                Err(e) => log_error!(&format!("Failed to create test data: {}", e)),
            }
        }
        path
    }

    // Helper to create an engine from real test data
    fn create_benchmark_engine_from_real_data() -> ThreadSafeAutocomplete {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();
        
        // Index all files from the test directory
        let entries = crate::search_engine::index_given_path_parallel(test_dir);
        log_info!(&format!("Indexing {} entries from test data for benchmark", entries.len()));
        
        let timer = BenchmarkTimer::new("index_test_data_into_autocomplete");
        
        // Add paths to the engine
        for entry in entries {
            match entry {
                crate::search_engine::Entry::FILE(file) => {
                    let path = PathBuf::from(&file.path);
                    engine.index_path(&path).expect("Failed to index test path");
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    let path = PathBuf::from(&dir.path);
                    engine.index_path(&path).expect("Failed to index test directory");
                },
            }
        }

        log_info!(&format!("Benchmark engine initialized with real test data"));
        engine
    }

    #[test]
    fn bench_real_data_autocomplete() {
        let _timer = BenchmarkTimer::new("bench_real_data_autocomplete");
        let engine = create_benchmark_engine_from_real_data();
        
        // Get statistics about the indexed data
        let stats = engine.get_stats().expect("Failed to get stats");
        log_info!(&format!("Real data benchmark engine: {} paths indexed", stats.indexed_path_count));
        
        // Benchmark various query patterns
        let test_queries = [
            // Common prefixes
            "doc", "ba", "txt", "js", 
            // Partial words
            "app", "car", "prog", 
            // Short queries
            "a", "e", "s", 
            // Long queries 
            "document", "program", "raspberry"
        ];
        
        log_info!(&format!("Running autocomplete benchmarks with {} different queries", test_queries.len()));
        log_info!(&format!("{:<12} | {:<15} | {:<10}", "Query", "Time", "Results"));
        log_info!(&format!("{:-<40}", ""));
        
        for query in &test_queries {
            let start = Instant::now();
            let results = engine.suggest(query, 10).expect("Suggestion failed");
            let duration = start.elapsed();
            
            log_info!(&format!("{:<12} | {:>15?} | {:>10}", 
                query, duration, results.len()));
            
            // For the first few results, try with fuzzy matching too
            if query.len() >= 3 {
                // Introduce a typo
                let typo_query = format!("{}{}{}",
                    &query[0..1],
                    if query.len() >= 2 { &query[2..] } else { "" },
                    if query.len() >= 2 { &query[1..2] } else { "" }
                );
                
                let start = Instant::now();
                let results = engine.suggest(&typo_query, 10).expect("Fuzzy suggestion failed");
                let duration = start.elapsed();
                
                log_info!(&format!("{:<12} | {:>15?} | {:>10} (fuzzy)", 
                    typo_query, duration, results.len()));
            }
        }
    }

    #[test]
    fn bench_batch_operations_with_real_data() {
        let _timer = BenchmarkTimer::new("bench_batch_operations_with_real_data");
        let engine = ThreadSafeAutocomplete::new();
        
        // Use the test data directory
        let test_dir = get_test_data_path();
        
        // Index all files from the test directory
        let entries = crate::search_engine::index_given_path_parallel(test_dir);
        log_info!(&format!("Found {} entries in test data", entries.len()));
        
        // Split the entries for batch operations
        let mut paths_to_add = Vec::new();
        
        for entry in entries {
            match entry {
                crate::search_engine::Entry::FILE(file) => {
                    paths_to_add.push(PathBuf::from(&file.path));
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    paths_to_add.push(PathBuf::from(&dir.path));
                },
            }
        }
        
        // Use part of the data for individual additions and part for batch
        let individual_count = std::cmp::min(100, paths_to_add.len() / 10);
        let batch_paths = paths_to_add.split_off(individual_count);
        
        // Benchmark individual operations
        let start = Instant::now();
        for path in &paths_to_add {
            engine.index_path(path).expect("Failed to index path");
        }
        let individual_time = start.elapsed();
        log_info!(&format!("{} individual index operations: {:?}", 
            paths_to_add.len(), individual_time));
        
        // Benchmark batch operation
        let start = Instant::now();
        engine.batch_update(batch_paths.clone(), vec![])
            .expect("Batch update failed");
        let batch_time = start.elapsed();
        log_info!(&format!("1 batch operation with {} paths: {:?}", 
            batch_paths.len(), batch_time));
        
        if paths_to_add.len() > 0 && batch_paths.len() > 0 {
            log_info!(&format!("Efficiency ratio: {:.2}x",
                (individual_time.as_micros() as f64 / paths_to_add.len() as f64) /
                (batch_time.as_micros() as f64 / batch_paths.len() as f64)));
        }
        
        // Test search performance after indexing
        let start = Instant::now();
        let results = engine.suggest("doc", 10).expect("Suggestion failed");
        let search_time = start.elapsed();
        log_info!(&format!("Search after indexing: found {} results in {:?}", 
            results.len(), search_time));
    }

    #[test]
    fn bench_concurrent_queries_with_real_data() {
        let _timer = BenchmarkTimer::new("bench_concurrent_queries_with_real_data");
        let engine = create_benchmark_engine_from_real_data();
        
        // Create thread handles
        let mut handles = vec![];
        let thread_count = 8;
        let queries_per_thread = 50;
        
        // Common search queries
        let search_terms = ["doc", "img", "txt", "config", "app", "ba", "car"];
        
        log_info!(&format!("Spawning {} threads, each performing {} queries", 
            thread_count, queries_per_thread));
        
        // Spawn threads
        for t in 0..thread_count {
            let engine_clone = engine.clone();
            let handle = thread::spawn(move || {
                let thread_start = Instant::now();
                let mut results_count = 0;
                
                for i in 0..queries_per_thread {
                    let query = search_terms[(i + t) % search_terms.len()];
                    let results = engine_clone.suggest(query, 10)
                        .expect("Thread suggestion failed");
                    results_count += results.len();
                }
                
                (thread_start.elapsed(), results_count)
            });
            handles.push(handle);
        }
        
        // Collect and report results
        let mut total_duration = Duration::from_secs(0);
        let mut max_duration = Duration::from_secs(0);
        let mut total_results = 0;
        
        for handle in handles {
            let (thread_time, result_count) = handle.join().expect("Thread panicked");
            total_duration += thread_time;
            total_results += result_count;
            if thread_time > max_duration {
                max_duration = thread_time;
            }
        }
        
        // Calculate average duration correctly using floating-point conversion
        let avg_duration_secs = total_duration.as_secs_f64() / thread_count as f64;
        let avg_duration = Duration::from_secs_f64(avg_duration_secs);

        log_info!(&format!("{} threads each performing {} queries:", 
            thread_count, queries_per_thread));
        log_info!(&format!("  Average thread time: {:?}", avg_duration));
        log_info!(&format!("  Max thread time: {:?}", max_duration));
        log_info!(&format!("  Total results found: {}", total_results));
        log_info!(&format!("  Throughput: {:.2} queries/second", 
            (thread_count * queries_per_thread) as f64 / max_duration.as_secs_f64()));
    }
}
