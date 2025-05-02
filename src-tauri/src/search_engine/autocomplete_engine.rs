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
                    engine.radix_trie.insert(&path_str, file.to_path_buf());
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
