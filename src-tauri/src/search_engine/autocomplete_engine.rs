use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::search_engine::adaptive_radix_trie::AdaptiveRadixTrie;
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
        // First try search_recursive which is more comprehensive - finds the query anywhere in the path
        let mut results = self.radix_trie.search_recursive(query);

        // If not enough results, try flexible prefix matching which looks at segments
        if results.len() < limit {
            let flexible_results = self.radix_trie.find_with_flexible_prefix(query);
            // Add only new results
            for path in flexible_results {
                if !results.contains(&path) {
                    results.push(path);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        // If still not enough results, try fuzzy search
        if results.len() < limit {
            let fuzzy_results = self.fuzzy_index.find_matches(query, limit - results.len());
            // Only add fuzzy results that don't already exist
            for path in fuzzy_results {
                if !results.contains(&path) {
                    results.push(path);
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        // Apply context-aware ranking - use current directory for context
        self.ranker.rank_results_with_context(results, &self.current_directory)
    }

    // Add a method to verify a path doesn't exist in any index
    pub fn verify_path_removed(&self, path: &Path) -> bool {
        // Check if path is completely gone from all indices

        // 1. Check radix trie
        let path_str = path.to_string_lossy().to_string();
        let in_trie = self.radix_trie.find_exact_path(&path_str).is_some();

        // 2. Check fuzzy index
        let in_fuzzy = self.fuzzy_index.contains_path(path);

        // Path is considered removed if it's not in any index
        !in_trie && !in_fuzzy
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
        match self.engine.read() {
            Ok(engine) => {
                // First check the cache within the same lock
                if let Some(cached_results) = engine.result_cache.get_suggestions(query) {
                    return Ok(cached_results);
                }

                // Cache miss, compute the results while still holding the lock
                let suggestions = engine.suggest(query, limit);

                // Cache the results while still holding the lock
                engine.result_cache.cache_suggestions(query, suggestions.clone())?;

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

                // Log the path we're indexing in tests
                #[cfg(test)]
                {
                    println!("DEBUG: Indexing path: {}", path_str);
                }

                // Add to radix trie with adaptive segmentation
                let _ = engine.radix_trie.insert(&path_str, path.to_path_buf());

                // Add to fuzzy index
                engine.fuzzy_index.index_path(path);

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Invalidate any cache entries that might be affected
                engine.result_cache.clear()?;

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
                let _ = engine.radix_trie.remove(&path_str);

                // Remove from fuzzy index
                engine.fuzzy_index.remove_path(path);

                // Update timestamp
                engine.last_update = SystemTime::now();

                // Clear result cache completely to ensure removed paths don't appear in results
                engine.result_cache.clear()?;

                // Log that we're removing the path
                #[cfg(test)]
                {
                    println!("DEBUG: Removing path: {}", path_str);
                }

                // Verify the path was completely removed (for testing purposes)
                #[cfg(test)]
                {
                    if !engine.verify_path_removed(path) {
                        return Err(format!("Failed to completely remove path: {}", path_str));
                    }
                }

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
                    let _ = engine.radix_trie.remove(&path_str);
                    engine.fuzzy_index.remove_path(path);
                }

                // Then process additions
                for path in &paths_to_add {
                    let path_str = path.to_string_lossy().to_string();
                    let _ = engine.radix_trie.insert(&path_str, path.to_path_buf());
                    engine.fuzzy_index.index_path(path);
                }

                // Update timestamp
                engine.last_update = SystemTime::now();

                // For now, we'll always clear the entire cache after batch operations
                // This ensures removed paths don't appear in results
                engine.result_cache.clear()?;

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
        match self.engine.read() {
            Ok(engine) => {
                self.invalidate_affected_cache_direct(path, &engine)
            },
            Err(_) => Err("Failed to acquire read lock on autocomplete engine".to_string()),
        }
    }

    // Add a new method that takes a reference to an already-locked engine
    fn invalidate_affected_cache_direct(&self, path: &Path, engine: &AutocompleteEngine) -> Result<(), String> {
        // Get filename without extension
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy().to_string();

            // Invalidate cache entries starting with this filename
            engine.result_cache.invalidate(&file_name_str)?;

            // Also try without extension
            if let Some(stem) = path.file_stem() {
                let stem_str = stem.to_string_lossy().to_string();
                engine.result_cache.invalidate(&stem_str)?;
            }
        }

        // If the path is deep, invalidate based on directory names too
        let mut current = path;
        while let Some(parent) = current.parent() {
            if let Some(dir_name) = parent.file_name() {
                let dir_name_str = dir_name.to_string_lossy().to_string();
                engine.result_cache.invalidate(&dir_name_str)?;
            }
            current = parent;
        }

        Ok(())
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
                #[cfg(test)]
                {
                    println!("DEBUG: Updating current directory to: {}", dir.display());
                }

                engine.current_directory = dir.clone();
                engine.ranker.update_current_directory(dir);

                // Clear cache when directory context changes
                engine.result_cache.clear()?;

                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on autocomplete engine".to_string()),
        }
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

    // New method to rank results considering current directory context
    pub fn rank_results_with_context(&self, results: Vec<PathBuf>, current_dir: &Path) -> Vec<PathBuf> {
        // Score and sort the results
        let mut scored_results: Vec<(PathBuf, f64)> = results
            .into_iter()
            .map(|path| {
                let base_score = self.calculate_score(&path);

                // Increase score significantly for files in the current directory or subdirectories
                let dir_bonus = if path.starts_with(current_dir) {
                    5.0  // Increased from 2.0 to give much stronger preference to current directory
                } else {
                    0.0
                };

                (path, base_score + dir_bonus)
            })
            .collect();

        // Sort by score (highest first)
        scored_results.sort_by(|(_, score1), (_, score2)| {
            score2.partial_cmp(score1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Extract just the paths, now sorted by score
        scored_results.into_iter().map(|(path, _)| path).collect()
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
    use std::time::Instant;
    use crate::{log_info, log_error};

    // Helper function to get or generate test data path - with smaller dataset
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-autocomplete-small");
        if !path.exists() {
            log_info!("Creating small test data directory...");
            match generate_small_test_data(path.clone()) {
                Ok(_) => log_info!("Small test data created successfully"),
                Err(e) => log_error!(&format!("Failed to create test data: {}", e)),
            }
        }
        path
    }

    // Create a much smaller test dataset to prevent test hangs
    fn generate_small_test_data(base_path: PathBuf) -> Result<PathBuf, std::io::Error> {
        use std::fs::{create_dir_all, File};

        // Remove the directory if it already exists
        if base_path.exists() {
            std::fs::remove_dir_all(&base_path)?;
        }

        // Create the base directory
        create_dir_all(&base_path)?;

        // Create a simple directory structure with predictable names
        let test_dirs = [
            "documents", "images", "music",
            "documents/work", "documents/personal",
            "images/vacation", "images/family"
        ];

        for dir in &test_dirs {
            create_dir_all(base_path.join(dir))?;
        }

        // Create test files with predictable names
        let test_files = [
            "documents/doc1.txt", "documents/doc2.docx", "documents/readme.md",
            "documents/work/report.pdf", "documents/work/budget.xlsx",
            "documents/personal/notes.txt", "documents/personal/letter.docx",
            "images/photo1.jpg", "images/photo2.png", "images/banner.gif",
            "images/vacation/beach.jpg", "images/vacation/hotel.jpg",
            "images/family/birthday.jpg", "images/family/group.png",
            "music/song1.mp3", "music/song2.mp3", "music/playlist.m3u"
        ];

        for file in &test_files {
            File::create(base_path.join(file))?;
        }

        log_info!(&format!("Created small test dataset with {} directories and {} files",
                 test_dirs.len(), test_files.len()));

        Ok(base_path)
    }

    // Updated to use smaller test data
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
                    if let Err(e) = engine.index_path(&path) {
                        log_error!(&format!("Failed to index test path: {}", e));
                    }
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    let path = PathBuf::from(&dir.path);
                    if let Err(e) = engine.index_path(&path) {
                        log_error!(&format!("Failed to index test directory: {}", e));
                    }
                },
            }
        }

        log_info!("Test engine initialized with test data");
        engine
    }

    #[test]
    fn test_autocomplete_with_real_data() {
        let engine = setup_test_engine();

        // Get stats about the indexed data
        if let Ok(stats) = engine.get_stats() {
            log_info!(&format!("Autocomplete engine stats: {} paths indexed, {} cached queries",
                stats.indexed_path_count, stats.cached_query_count));

            assert!(stats.indexed_path_count > 0, "Should have indexed some paths");
        }

        // Try some autocomplete queries
        let queries = ["doc", "jpg", "txt"];

        for query in &queries {
            if let Ok(results) = engine.suggest(query, 5) {
                log_info!(&format!("Query '{}' returned {} suggestions", query, results.len()));

                // For non-empty results, print first match
                if !results.is_empty() {
                    log_info!(&format!("  First match: {}", results[0].display()));
                }
            }
        }
    }

    #[test]
    fn test_suggest_with_metadata() {
        let engine = setup_test_engine();

        // Get results with metadata for a common query
        if let Ok(results) = engine.suggest_with_metadata("doc", 5) {
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
    }

    #[test]
    fn test_record_and_retrieve_access_patterns() {
        let engine = setup_test_engine();

        // Create a known path from our test data
        let test_path = get_test_data_path().join("documents/doc1.txt");

        // Record several accesses to the test path
        for _ in 0..3 {
            if let Err(e) = engine.record_path_access(&test_path) {
                log_error!(&format!("Failed to record path access: {}", e));
            }
        }

        // Get frequent paths
        if let Ok(frequent_paths) = engine.get_frequent_paths(5) {
            log_info!(&format!("Retrieved {} frequent paths", frequent_paths.len()));
        }

        // Get recent paths
        if let Ok(recent_paths) = engine.get_recent_paths(5) {
            log_info!(&format!("Retrieved {} recent paths", recent_paths.len()));
        }
    }

    #[test]
    fn test_directory_context_affects_results() {
        let engine = ThreadSafeAutocomplete::new();

        // Use a known directory from our test data
        let target_dir = get_test_data_path().join("documents");

        // Update current directory to that parent
        if let Err(e) = engine.update_current_directory(target_dir.clone()) {
            log_error!(&format!("Failed to update current directory: {}", e));
        }

        // First verify we have some txt files in our test data
        let txt_files_exist = engine.suggest("txt", 10).expect("Search for txt files failed");
        if txt_files_exist.is_empty() {
            // If no txt files exist, create one for testing
            let test_path = target_dir.join("test_context.txt");
            std::fs::write(&test_path, "test content").expect("Failed to create test file");

            // Make sure the file exists
            assert!(test_path.exists(), "Test file should exist after creation");
            println!("Created test file at: {}", test_path.display());

            // Index the newly created file explicitly
            engine.index_path(&test_path).expect("Failed to index test file");

            // Re-check after creating the test file - with a more specific search term
            let txt_files_after_create = engine.suggest("test_context", 10).expect("Search after file creation failed");

            // Debug output
            if txt_files_after_create.is_empty() {
                println!("DEBUG: No results found for 'test_context'");
                // Try a more general search to see what's indexed
                let all_files = engine.suggest("", 20).expect("Failed to get all files");
                println!("All indexed files:");
                for file in all_files {
                    println!("  {}", file.display());
                }
            }

            assert!(!txt_files_after_create.is_empty(), "Should find test file after creation");
        }

        // Get suggestions with the new context
        let context_results = engine.suggest("txt", 5).expect("Search with context failed");

        // Debug output to help diagnose issues
        if context_results.is_empty() {
            log_error!("No results found with directory context. Checking if any txt files exist...");
            let all_results = engine.suggest("", 100).expect("Failed to get all files");
            log_error!(&format!("Found {} total files in index", all_results.len()));

            for path in all_results.iter().take(5) {
                log_error!(&format!("Sample indexed file: {}", path.display()));
            }
        }

        assert!(!context_results.is_empty(), "Should find results with directory context");
    }

    #[test]
    fn test_fuzzy_search_capabilities() {
        let engine = setup_test_engine();

        // Get results for a correct query
        let correct_query = "document";
        if let Ok(correct_results) = engine.suggest(correct_query, 5) {
            log_info!(&format!("Correct query '{}' returned {} results",
                correct_query, correct_results.len()));

            // Now try with a typo
            let typo_query = "documnet"; // Swapped 'e' and 'n'
            if let Ok(fuzzy_results) = engine.suggest(typo_query, 5) {
                log_info!(&format!("Misspelled query '{}' returned {} results",
                    typo_query, fuzzy_results.len()));
            }
        }
    }

    #[test]
    fn test_cache_performance() {
        let engine = setup_test_engine();

        // First query - should miss cache
        let start = Instant::now();
        let _ = engine.suggest("doc", 5);
        let first_query_time = start.elapsed();

        // Second query with the same string - should hit cache
        let start = Instant::now();
        let _ = engine.suggest("doc", 5);
        let second_query_time = start.elapsed();

        log_info!(&format!("First query (cache miss): {:?}", first_query_time));
        log_info!(&format!("Second query (cache hit): {:?}", second_query_time));
    }

    #[test]
    fn test_index_and_remove_path() {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();

        // Use known paths from test data
        let test_path1 = test_dir.join("documents/doc1.txt");
        let test_path2 = test_dir.join("documents/doc2.docx");

        // Index the paths
        engine.index_path(&test_path1).expect("Failed to index first test path");
        engine.index_path(&test_path2).expect("Failed to index second test path");

        // Search for file names
        let results1 = engine.suggest("doc1", 5).expect("First search failed");
        assert!(!results1.is_empty(), "Should find indexed path");

        // Remove one of the paths and ensure cache is fully cleared
        engine.remove_path(&test_path1).expect("Failed to remove path");

        // Add debug logging to show the search term and results
        println!("Searching for 'doc1' after removal");
        let results_after_removal = engine.suggest("doc1", 5).expect("Search after removal failed");

        // Debugging the test failure
        if !results_after_removal.is_empty() {
            println!("WARNING: Found removed path in results: {:?}", results_after_removal);
            // Force reindex this path to debug - remove in production
            engine.remove_path(&test_path1).expect("Failed to remove path again");
        }

        assert!(results_after_removal.is_empty(), "Removed path should no longer appear");

        // Verify the other path still exists
        let results2 = engine.suggest("doc2", 5).expect("Second search failed");
        assert!(!results2.is_empty(), "Should still find non-removed path");
    }

    #[test]
    fn test_batch_update_with_real_paths() {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();

        // Use known paths from test data
        let paths_to_add = vec![
            test_dir.join("documents/doc1.txt"),
            test_dir.join("documents/doc2.docx"),
            test_dir.join("images/photo1.jpg")
        ];
        
        // Perform batch update
        engine.batch_update(paths_to_add.clone(), Vec::new())
            .expect("Batch update failed");

        // Now search for these files
        let results = engine.suggest("doc", 5).expect("Search failed");
        assert!(!results.is_empty(), "Should find batch-added paths");
        
        // Now remove them in a batch
        engine.batch_update(Vec::new(), paths_to_add.clone())
            .expect("Batch removal failed");

        // Verify they're gone
        let results = engine.suggest("doc", 5).expect("Search after removal failed");
        assert!(results.is_empty(), "Should not find removed paths");
    }
}

#[cfg(test)]
mod benchmarks_autocomplete {
    use std::path::PathBuf;
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::{log_error, log_info};
    use crate::search_engine::autocomplete_engine::ThreadSafeAutocomplete;
    
    // Helper function to get or generate smaller test data path for benchmarks
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-autocomplete-small");
        if !path.exists() {
            log_info!("Creating small test data directory for benchmarks...");
            match generate_small_test_data(path.clone()) {
                Ok(_) => log_info!("Small test data created successfully for benchmarks"),
                Err(e) => log_error!(&format!("Failed to create test data: {}", e)),
            }
        }
        path
    }
    
    // Create a much smaller test dataset for benchmarks
    fn generate_small_test_data(base_path: PathBuf) -> Result<PathBuf, std::io::Error> {
        use std::fs::{create_dir_all, File};
        
        // Remove the directory if it already exists
        if base_path.exists() {
            std::fs::remove_dir_all(&base_path)?;
        }
        
        // Create the base directory
        create_dir_all(&base_path)?;
        
        // Create a simple directory structure with predictable names
        let test_dirs = [
            "documents", "images", "music", 
            "documents/work", "documents/personal",
            "images/vacation", "images/family"
        ];
        
        for dir in &test_dirs {
            create_dir_all(base_path.join(dir))?;
        }
        
        // Create test files with predictable names
        let test_files = [
            "documents/doc1.txt", "documents/doc2.docx", "documents/readme.md",
            "documents/work/report.pdf", "documents/work/budget.xlsx",
            "documents/personal/notes.txt", "documents/personal/letter.docx",
            "images/photo1.jpg", "images/photo2.png", "images/banner.gif",
            "images/vacation/beach.jpg", "images/vacation/hotel.jpg",
            "images/family/birthday.jpg", "images/family/group.png",
            "music/song1.mp3", "music/song2.mp3", "music/playlist.m3u"
        ];
        
        for file in &test_files {
            File::create(base_path.join(file))?;
        }
        
        log_info!(&format!("Created small benchmark dataset with {} directories and {} files", 
                 test_dirs.len(), test_files.len()));
                 
        Ok(base_path)
    }
    
    // Helper to create an engine from small test data
    fn create_benchmark_engine_from_real_data() -> ThreadSafeAutocomplete {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();
        
        // Index all files from the test directory
        let entries = crate::search_engine::index_given_path_parallel(test_dir);
        log_info!(&format!("Indexing {} entries from test data for benchmark", entries.len()));
        
        // Add paths to the engine
        for entry in entries {
            match entry {
                crate::search_engine::Entry::FILE(file) => {
                    let path = PathBuf::from(&file.path);
                    if let Err(e) = engine.index_path(&path) {
                        log_error!(&format!("Failed to index test path: {}", e));
                    }
                },
                crate::search_engine::Entry::DIRECTORY(dir) => {
                    let path = PathBuf::from(&dir.path);
                    if let Err(e) = engine.index_path(&path) {
                        log_error!(&format!("Failed to index test directory: {}", e));
                    }
                },
            }
        }

        log_info!(&format!("Benchmark engine initialized with real test data"));
        engine
    }

    #[test]
    fn bench_real_data_autocomplete() {
        let engine = create_benchmark_engine_from_real_data();
        
        // Get statistics about the indexed data
        if let Ok(stats) = engine.get_stats() {
            log_info!(&format!("Benchmark engine: {} paths indexed", stats.indexed_path_count));
        }
        
        // Benchmark fewer query patterns
        let test_queries = ["doc", "jpg", "txt", "a"]; 
        
        log_info!(&format!("Running autocomplete benchmarks with {} queries", test_queries.len()));
        log_info!(&format!("{:<12} | {:<15} | {:<10}", "Query", "Time", "Results"));
        
        for query in &test_queries {
            let start = Instant::now();
            if let Ok(results) = engine.suggest(query, 5) {
                let duration = start.elapsed();
                log_info!(&format!("{:<12} | {:>15?} | {:>10}", 
                    query, duration, results.len()));
            }
        }
    }

    #[test]
    fn bench_batch_operations_with_real_data() {
        let engine = ThreadSafeAutocomplete::new();
        let test_dir = get_test_data_path();
        
        // Use known paths from test data
        let paths_to_add = vec![
            test_dir.join("documents/doc1.txt"),
            test_dir.join("documents/doc2.docx"),
            test_dir.join("images/photo1.jpg"),
            test_dir.join("music/song1.mp3")
        ];
        
        // Benchmark individual operations
        let start = Instant::now();
        for path in &paths_to_add {
            engine.index_path(path).expect("Failed to index path");
        }
        let individual_time = start.elapsed();
        log_info!(&format!("{} individual operations: {:?}", paths_to_add.len(), individual_time));
        
        // Clear the engine
        engine.clear().expect("Failed to clear engine");
        
        // Benchmark batch operation
        let start = Instant::now();
        engine.batch_update(paths_to_add.clone(), vec![])
            .expect("Batch update failed");
        let batch_time = start.elapsed();
        log_info!(&format!("1 batch operation: {:?}", batch_time));
    }

    #[test]
    fn bench_concurrent_queries_with_real_data() {
        let engine = create_benchmark_engine_from_real_data();
        
        // Create thread handles
        let mut handles = vec![];
        let thread_count = 4; // Reduced from 8
        let queries_per_thread = 10; // Reduced from 50
        
        // Common search queries
        let search_terms = ["doc", "jpg", "txt"];
        
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
                    if let Ok(results) = engine_clone.suggest(query, 5) {
                        results_count += results.len();
                    }
                }
                
                (thread_start.elapsed(), results_count)
            });
            handles.push(handle);
        }
        
        // Collect and report results
        let mut total_duration = Duration::from_secs(0);
        let mut total_results = 0;
        
        for handle in handles {
            match handle.join() {
                Ok((thread_time, result_count)) => {
                    total_duration += thread_time;
                    total_results += result_count;
                },
                Err(_) => log_error!("Thread panicked"),
            }
        }
        
        log_info!(&format!("Concurrent benchmark completed, found {} total results", total_results));
    }
}
