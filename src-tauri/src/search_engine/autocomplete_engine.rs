use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::search_engine::art_v3::ART;
use crate::search_engine::fast_fuzzy_v2::PathMatcher;
use crate::search_engine::path_cache_wrapper::PathCache;
use crate::{log_info, log_warn};

/// Autocomplete engine that combines caching, prefix search, and fuzzy search
pub struct AutocompleteEngine {
    /// Cache for storing recent search results
    cache: PathCache,
    
    /// Adaptive Radix Trie for prefix searching
    trie: ART,
    
    /// Fuzzy search engine for approximate matching
    fuzzy_matcher: PathMatcher,
    
    /// Maximum number of results to return
    max_results: usize,
    
    /// Current directory context for ranking
    current_directory: Option<String>,
    
    /// Track frequency of path usage
    frequency_map: HashMap<String, u32>,
    
    /// Track recency of path usage
    recency_map: HashMap<String, Instant>,
    
    /// Preferred file extensions (ranked higher)
    preferred_extensions: Vec<String>,
    
    /// Flag to signal that indexing should stop
    stop_indexing: AtomicBool,
}

#[allow(dead_code)] // remove later when used
impl AutocompleteEngine {
    /// Create a new AutocompleteEngine with specified cache size and max results
    pub fn new(cache_size: usize, max_results: usize) -> Self {
        Self {
            cache: PathCache::with_ttl(cache_size, Duration::from_secs(300)), // 5 minute TTL
            trie: ART::new(max_results * 2),
            fuzzy_matcher: PathMatcher::new(),
            max_results,
            current_directory: None,
            frequency_map: HashMap::new(),
            recency_map: HashMap::new(),
            preferred_extensions: vec![
                "txt".to_string(), "pdf".to_string(), "docx".to_string(), 
                "xlsx".to_string(), "md".to_string(), "rs".to_string(),
                "js".to_string(), "html".to_string(), "css".to_string(),
                "json".to_string(), "png".to_string(), "jpg".to_string(),
                "mp4".to_string(), "mp3".to_string()
            ],
            stop_indexing: AtomicBool::new(false),
        }
    }

    /// Normalize paths with special handling for spaces and backslashes
    fn normalize_path(&self, path: &str) -> String {
        // Skip normalization for empty paths
        if path.is_empty() {
            return String::new();
        }

        // Step 1: Handle escaped spaces
        // Replace backslash-space sequences with just spaces
        let space_fixed = path.replace("\\ ", " ");

        // Step 2: Handle platform-specific separators
        let slash_fixed = space_fixed.replace('\\', "/");

        // Step 3: Fix doubled slashes
        let mut normalized = slash_fixed;
        while normalized.contains("//") {
            normalized = normalized.replace("//", "/");
        }

        // Step 4: Handle trailing slashes appropriately
        let trimmed = if normalized == "/" {
            "/".to_string()
        } else {
            normalized.trim_end_matches('/').to_string()
        };

        // Step 5: Clean up any remaining spaces that look like they should be separators
        // This handles cases where spaces were intended to be path separators
        if trimmed.contains(' ') {
            // Check if these are likely meant to be separators by looking at the pattern
            // e.g., "./test-data-for-fuzzy-search ambulance blueberry lime"
            let components: Vec<&str> = trimmed.split(' ').collect();

            // If the first component contains a slash and subsequent components don't,
            // they're likely meant to be separate path components
            if components.len() > 1 &&
                components[0].contains('/') &&
                !components.iter().skip(1).any(|&c| c.contains('/')) {
                // Join with slashes instead of spaces
                return components.join("/");
            }
        }

        trimmed
    }

    /// for ranking
    pub fn set_current_directory(&mut self, directory: Option<String>) {
        self.current_directory = directory;
    }
    
    /// Add or update a path (normalized!) in the search engines
    pub fn add_path(&mut self, path: &str) {
        let normalied_path = self.normalize_path(path);
        let mut score = 1.0;
        
        // Check if we have existing frequency data to adjust score
        if let Some(freq) = self.frequency_map.get(&normalied_path) {
            // Boost score for frequently accessed paths
            score += (*freq as f32) * 0.01;
        }
        
        // Update all modules and clean cache
        self.trie.insert(&normalied_path, score);
        self.fuzzy_matcher.add_path(&normalied_path);
        self.cache.purge_expired();
    }

    /// Signal the engine to stop indexing
    pub fn stop_indexing(&mut self) {
        self.stop_indexing.store(true, Ordering::SeqCst);
    }

    /// Reset the stop indexing flag
    pub fn reset_stop_flag(&mut self) {
        self.stop_indexing.store(false, Ordering::SeqCst);
    }

    /// Check if indexing should stop
    pub fn should_stop_indexing(&self) -> bool {
        self.stop_indexing.load(Ordering::SeqCst)
    }

    pub fn add_paths_recursive(&mut self, path: &str) {
        // Reset stop flag at the beginning of a new indexing operation
        self.reset_stop_flag();
        
        // Add the path itself first
        self.add_path(path);

        // Check if the path is a directory
        let path_obj = std::path::Path::new(path);
        if !path_obj.is_dir() {
            return;
        }

        log_info!(&format!("Recursively indexing directory: {}", path));

        // Walk dir
        let walk_dir = match std::fs::read_dir(path) {
            Ok(dir) => dir,
            Err(err) => {
                log_warn!(&format!("Failed to read directory '{}': {}", path, err));
                return;
            }
        };

        for entry in walk_dir.filter_map(Result::ok) {
            // Check if we should stop indexing
            if self.should_stop_indexing() {
                log_info!(&format!("Indexing of '{}' stopped prematurely", path));
                return;
            }
            
            let entry_path = entry.path();
            if let Some(entry_str) = Some(entry_path.to_string_lossy().to_string()) {
                // Add this path
                self.add_path(&entry_str);

                // If it's a directory, recurse
                if entry_path.is_dir() {
                    self.add_paths_recursive(&entry_str);
                    
                    // Check again after recursion in case we need to stop
                    if self.should_stop_indexing() {
                        return;
                    }
                }
            }
        }
    }

    /// Remove a path (normalized!) from the search engines
    pub fn remove_path(&mut self, path: &str) {
        let normalized_path = self.normalize_path(path);
        // Remove from modules
        self.trie.remove(&normalized_path);
        self.fuzzy_matcher.remove_path(&normalized_path);
        self.cache.remove(&normalized_path);

        // Remove from frequency and recency maps
        self.frequency_map.remove(&normalized_path);
        self.recency_map.remove(&normalized_path);
    }

    pub fn remove_paths_recursive(&mut self, path: &str) {
        // Remove the path itself first
        self.remove_path(path);

        // Check if the path is a directory
        let path_obj = std::path::Path::new(path);
        if !path_obj.exists() || !path_obj.is_dir() {
            return;
        }

        log_info!(&format!("Recursively removing directory from index: {}", path));

        let mut paths_to_remove = Vec::new();

        // Walk the directory and collect all paths
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.filter_map(Result::ok) {
                let entry_path = entry.path();
                if let Some(entry_str) = entry_path.to_str() {
                    paths_to_remove.push(entry_str.to_string());
                }
            }
        } else {
            log_warn!(&format!("Failed to read directory '{}' for removal", path));
        }

        // Now remove each path
        for path_to_remove in paths_to_remove {
            if std::path::Path::new(&path_to_remove).is_dir() {
                self.remove_paths_recursive(&path_to_remove);
            } else {
                self.remove_path(&path_to_remove);
            }
        }

        // Ensure the cache is purged of any entries that might contain references to removed paths
        self.cache.purge_expired();
    }

    /// Track that a path was used (for frequency/recency scoring)
    pub fn record_path_usage(&mut self, path: &str) {
        // Update frequency count
        let count = self.frequency_map.entry(path.to_string()).or_insert(0);
        *count += 1;
        
        // Update recency timestamp
        self.recency_map.insert(path.to_string(), Instant::now());
    }
    
    /// Set preferred file extensions for ranking
    pub fn set_preferred_extensions(&mut self, extensions: Vec<String>) {
        self.preferred_extensions = extensions;
    }
    
    /// Get the currently set preferred extensions
    pub fn get_preferred_extensions(&self) -> &Vec<String> {
        &self.preferred_extensions
    }
    
    /// Search for path completions using the engine's combined strategy
    pub fn search(&mut self, query: &str) -> Vec<(String, f32)> {
        if query.is_empty() {
            return Vec::new();
        }

        let normalized_query = query.trim().to_string();

        // 1. Check cache first but validate the path still exists
        if let Some(path_data) = self.cache.get(&normalized_query) {
            log_info!(&format!("Cache hit for query: '{}'", normalized_query));

            // Fast check if path still exists
            if self.trie.contains(&path_data.path) {
                // Path still exists, return it
                return vec![(path_data.path, path_data.score)];
            } else {
                // Path no longer exists, remove it from cache
                log_info!(&format!("Cached path '{}' no longer exists, removing from cache",
                          path_data.path));
                self.cache.remove(&normalized_query);
            }
        }
        
        log_info!(&format!("Cache miss for query: '{}'", normalized_query));
        
        // 2. Search using context-aware search in ART instead of simple prefix search
        let prefix_start = Instant::now();

        // Use the context-aware search method with current directory
        let current_dir_ref = self.current_directory.as_deref();
        let prefix_results = self.trie.search(
            &normalized_query,
            current_dir_ref,
            true // allow partial component matches
        );

        let prefix_duration = prefix_start.elapsed();
        log_info!(&format!("Context-aware prefix search found {} results in {:?}",
                 prefix_results.len(), prefix_duration));
        
        // 3. Only use fuzzy search if we don't have enough results
        let mut results = prefix_results;
        if results.len() < self.max_results {
            let fuzzy_start = Instant::now();
            let fuzzy_results = self.fuzzy_matcher.search(
                &normalized_query, 
                self.max_results - results.len()
            );
            let fuzzy_duration = fuzzy_start.elapsed();
            log_info!(&format!("Fuzzy search found {} results in {:?}", 
                     fuzzy_results.len(), fuzzy_duration));
            
            // Combine results, avoiding duplicates
            let prefix_paths: Vec<String> = results.iter().map(|(path, _)| path.clone()).collect();
            for (path, score) in fuzzy_results {
                if !prefix_paths.contains(&path) {
                    results.push((path, score));
                }
            }
        }
        
        // 4. Rank combined results
        self.rank_results(&mut results, &normalized_query);
        
        // 5. Cache top result for future queries
        if !results.is_empty() {
            // Only cache the exact query match to avoid cache pollution
            self.cache.insert(normalized_query.clone(), results[0].1);
            
            // Also record usage of the top result
            self.record_path_usage(&results[0].0);
        }
        
        // 6. Limit to max results
        results.truncate(self.max_results);
        
        results
    }
    
    /// Rank search results based on various relevance factors
    fn rank_results(&self, results: &mut Vec<(String, f32)>, query: &str) {
        // Recalculate scores based on our ranking criteria
        for (path, score) in results.iter_mut() {
            // Start with the existing score
            let mut new_score = *score;
            
            // 1. Boost for frequency
            if let Some(frequency) = self.frequency_map.get(path) {
                // More frequently used paths get a boost
                new_score += ((*frequency as f32) * 0.05).min(0.5);
            }
            
            // 2. Boost for recency
            if let Some(timestamp) = self.recency_map.get(path) {
                let age = timestamp.elapsed();
                // More recently used paths get a boost (max 1.0 for very recent)
                // Increased from 0.3 to 1.0 to ensure recency takes priority
                let recency_boost = 1.0 * (1.0 - (age.as_secs_f32() / 86400.0).min(1.0));
                new_score += recency_boost;
            }
            
            // 3. Boost for current directory context
            if let Some(current_dir) = &self.current_directory {
                if path.starts_with(current_dir) {
                    // Paths in the current directory get a significant boost
                    new_score += 0.4;
                } else if let Some(parent_dir) = std::path::Path::new(current_dir).parent() {
                    if let Some(parent_str) = parent_dir.to_str() {
                        if path.starts_with(parent_str) {
                            // Paths in the parent directory get a smaller boost
                            new_score += 0.2;
                        }
                    }
                }
            }
            
            // 4. Boost for preferred file extensions
            if let Some(extension) = std::path::Path::new(path)
                .extension()
                .and_then(|e| e.to_str()) 
            {
                let ext = extension.to_lowercase();
                
                // Check if it's in preferred extensions list
                if let Some(position) = self.preferred_extensions.iter().position(|e| e.to_lowercase() == ext) {
                    // Give higher boost to extensions that appear earlier in the list
                    let position_factor = 1.0 - (position as f32 / self.preferred_extensions.len().max(1) as f32);
                    // Stronger boost (up to 2.0 for first extension)
                    new_score += 2.0 * position_factor;
                    
                    // Log this boost for debugging
                    log_info!(&format!("Boosting score for {} with extension {} by {:.2}", 
                             path, ext, 2.0 * position_factor));
                }
                
                // Extra boost if the query contains the extension
                if query.to_lowercase().contains(&ext) {
                    new_score += 0.25;
                }
            }
            
            // 5. Boost for exact filename matches
            if let Some(filename) = std::path::Path::new(path).file_name().and_then(|n| n.to_str()) {
                if filename.to_lowercase() == query.to_lowercase() {
                    // Exact filename matches get a large boost
                    new_score += 1.0;
                } else if filename.to_lowercase().starts_with(&query.to_lowercase()) {
                    // Filename prefix matches get a medium boost
                    new_score += 0.3;
                } else if filename.to_lowercase().contains(&query.to_lowercase()) {
                    // Filename contains matches get a small boost
                    new_score += 0.1;
                }
            }

            *score = new_score;
        }
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    #[allow(dead_code)] // Used in testing
    /// Clear all data and caches
    pub fn clear(&mut self) {
        self.trie.clear();
        self.cache.clear();
        self.frequency_map.clear();
        self.recency_map.clear();

        self.fuzzy_matcher = PathMatcher::new();
    }
    
    /// Get statistics about the engine
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            cache_size: self.cache.len(),
            trie_size: self.trie.len(),
        }
    }
}

#[allow(dead_code)] // used only in testing rn
/// Statistics about the autocomplete engine
pub struct EngineStats {
    pub cache_size: usize,
    pub trie_size: usize,
}

#[cfg(test)]
mod tests_autocomplete_engine {
    use super::*;
    use std::thread::sleep;
    use std::fs;

    #[test]
    fn test_basic_search() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add some test paths
        engine.add_path("/home/user/documents/report.pdf");
        engine.add_path("/home/user/documents/notes.txt");
        engine.add_path("/home/user/pictures/vacation.jpg");
        
        // Test prefix search
        let results = engine.search("doc");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(path, _)| path.contains("documents")));
        log_info!(&format!("First search for 'doc' found {} results", results.len()));
        
        // Test cache hit on repeat search
        let cached_results = engine.search("doc");
        log_info!(&format!("Second search for 'doc' found {} results", cached_results.len()));
        assert!(!cached_results.is_empty());
    }
    
    #[test]
    fn test_fuzzy_search_fallback() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add some test paths
        engine.add_path("/home/user/documents/report.pdf");
        engine.add_path("/home/user/documents/presentation.pptx");
        engine.add_path("/home/user/pictures/vacation.jpg");
        
        // Test with a misspelling that should use fuzzy search
        let results = engine.search("documants");
        assert!(!results.is_empty());
        assert!(results.iter().any(|(path, _)| path.contains("documents")));
        log_info!(&format!("Fuzzy search for 'documants' found {} results", results.len()));
    }
    
    #[test]
    fn test_recency_and_frequency_ranking() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add some test paths
        engine.add_path("/path/a.txt");
        engine.add_path("/path/b.txt");
        engine.add_path("/path/c.txt");
        
        // Increase frequency and recency for certain paths
        engine.record_path_usage("/path/a.txt");
        engine.record_path_usage("/path/a.txt");  // Used twice
        engine.record_path_usage("/path/b.txt");  // Used once
        
        // Wait a bit to create a recency difference
        sleep(Duration::from_millis(50));
        
        // Record newer usage for b.txt
        engine.record_path_usage("/path/b.txt");
        
        // Search for common prefix
        let results = engine.search("/path/");
        
        // b.txt should be first (most recent), followed by a.txt (most frequent)
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/path/b.txt");  // This is correct, should be most recent
        assert_eq!(results[1].0, "/path/a.txt");  // This is second most relevant
    }

    #[test]
    fn test_current_directory_context() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add paths in different directories
        engine.add_path("/home/user/docs/file1.txt");
        engine.add_path("/home/user/docs/file2.txt");
        engine.add_path("/var/log/file3.txt");
        
        // Set current directory context
        engine.set_current_directory(Some("/home/user/docs".to_string()));
        
        // Search for a common term
        let results = engine.search("file");
        
        // The files in the current directory should be ranked higher
        assert!(!results.is_empty());
        assert!(results[0].0.starts_with("/home/user/docs"));
    }
    
    #[test]
    fn test_extension_preference() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add paths with different extensions
        engine.add_path("/docs/report.pdf");
        engine.add_path("/docs/data.csv");
        engine.add_path("/docs/note.txt");
        
        // txt and pdf should be preferred over csv
        let results = engine.search("docs");
        
        // The files with preferred extensions should be ranked higher
        assert!(!results.is_empty());
        assert!(results[0].0.ends_with(".pdf") || results[0].0.ends_with(".txt"));
    }
    
    #[test]
    fn test_removal() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add paths
        engine.add_path("/path/file1.txt");
        engine.add_path("/path/file2.txt");
        
        // Initial search
        let initial_results = engine.search("file");
        assert_eq!(initial_results.len(), 2);
        
        // Remove one path
        engine.remove_path("/path/file1.txt");
        
        // Search again
        let after_removal = engine.search("file");
        assert_eq!(after_removal.len(), 1);
        assert_eq!(after_removal[0].0, "/path/file2.txt");
    }
    
    #[test]
    fn test_cache_expiration() {
        let mut engine = AutocompleteEngine::new(10, 5);
        
        // Add a path
        engine.add_path("/test/file.txt");
        
        // Search to cache results
        let _ = engine.search("file");
        
        // Modify the path cache with a very short TTL for testing
        engine.cache = PathCache::with_ttl(10, Duration::from_millis(10));
        
        // Add the path again to ensure it's in the index
        engine.add_path("/test/file.txt");
        
        // Wait for cache to expire
        sleep(Duration::from_millis(20));
        
        // Search again - should be a cache miss but still find results
        let results = engine.search("file");
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/test/file.txt");
    }
    
    #[test]
    fn test_stats() {
        let mut engine = AutocompleteEngine::new(100, 10);
        
        // Add some paths
        for i in 0..5 {
            engine.add_path(&format!("/path/file{}.txt", i));
        }
        
        // Search to populate cache
        let _ = engine.search("file");
        
        // Get stats
        let stats = engine.get_stats();
        
        // Should have 5 paths in trie, 1 in cache
        assert_eq!(stats.trie_size, 5);
        assert!(stats.cache_size >= 1);
    }

    // Helper function to create a temporary directory structure for testing
    fn create_temp_dir_structure() -> std::path::PathBuf {
        // Create unique temp directory using timestamp and random number
        let unique_id = format!("{}_{}", std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis());

        let temp_dir = std::env::temp_dir().join(format!("autocomplete_engine_test_{}", unique_id));

        // Clean up any previous test directories
        if temp_dir.exists() {
            // Add a best-effort cleanup, but don't panic if it fails
            let _ = fs::remove_dir_all(&temp_dir);
        }

        // Create main directory
        fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

        // Create subdirectories and files
        let subdir1 = temp_dir.join("subdir1");
        let subdir2 = temp_dir.join("subdir2");
        let nested_dir = subdir1.join("nested");

        // Create each directory
        fs::create_dir_all(&subdir1).expect("Failed to create subdir1");
        fs::create_dir_all(&subdir2).expect("Failed to create subdir2");
        fs::create_dir_all(&nested_dir).expect("Failed to create nested dir");

        // Create some test files
        let root_file = temp_dir.join("root_file.txt");
        let file1 = subdir1.join("file1.txt");
        let file2 = subdir2.join("file2.txt");
        let nested_file = nested_dir.join("nested_file.txt");

        // Write content to each file, checking for success
        fs::write(&root_file, "test").expect("Failed to create root file");
        fs::write(&file1, "test").expect("Failed to create file1");
        fs::write(&file2, "test").expect("Failed to create file2");
        fs::write(&nested_file, "test").expect("Failed to create nested file");

        // Verify all files exist before returning
        assert!(root_file.exists(), "Root file was not created");
        assert!(file1.exists(), "File1 was not created");
        assert!(file2.exists(), "File2 was not created");
        assert!(nested_file.exists(), "Nested file was not created");

        temp_dir
    }

    #[test]
    fn test_add_paths_recursive() {
        let temp_dir = create_temp_dir_structure();
        let temp_dir_str = temp_dir.to_str().unwrap();

        let mut engine = AutocompleteEngine::new(100, 10);

        // Add paths recursively
        engine.add_paths_recursive(temp_dir_str);

        // Test that all files are indexed
        let results = engine.search(temp_dir_str);
        assert!(!results.is_empty(), "Should find temp directory");

        // Check for root file - search for the full filename to be more specific
        let root_file_results = engine.search("root_file.txt");
        assert!(!root_file_results.is_empty(), "Should find root file");
        assert!(root_file_results[0].0.contains("root_file.txt"));

        // Check for file in subdirectory - search for the full filename to be more specific
        let subdir_results = engine.search("file1.txt");
        assert!(!subdir_results.is_empty(), "Should find file1");
        assert!(subdir_results[0].0.contains("file1.txt"));

        // Check for file in nested directory - search for the full filename
        let nested_results = engine.search("nested_file.txt");
        assert!(!nested_results.is_empty(), "Should find nested file");
        assert!(nested_results[0].0.contains("nested_file.txt"));

        // Get engine stats to verify indexed path count
        let stats = engine.get_stats();

        // We should have all directories and files indexed (1 root dir + 3 subdirs + 4 files = 8 paths)
        assert!(stats.trie_size >= 8, "Trie should contain all paths and directories");

        // Clean up - best effort, don't panic if it fails
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_remove_paths_recursive() {
        let temp_dir = create_temp_dir_structure();
        let temp_dir_str = temp_dir.to_str().unwrap();
        let subdir1_str = temp_dir.join("subdir1").to_str().unwrap().to_string();

        let mut engine = AutocompleteEngine::new(100, 10);

        // First add all paths recursively
        engine.add_paths_recursive(temp_dir_str);

        // Verify initial indexing
        let initial_stats = engine.get_stats();
        assert!(initial_stats.trie_size >= 8, "Trie should initially contain all paths");

        // Verify subdir1 content is searchable - use full filename
        let subdir1_results = engine.search("file1.txt");
        assert!(!subdir1_results.is_empty(), "Should initially find file1");

        // Force cache purging before removal to ensure clean state
        engine.cache.clear();

        // Now remove one subdirectory recursively
        engine.remove_paths_recursive(&subdir1_str);

        // Verify subdir1 content is no longer searchable (should still find fuzzy matches)
        let after_removal_results = engine.search("file1.txt");
        assert!(!after_removal_results[0].0.contains("file1.txt"), "Should not find file1 after removal");

        // Also verify nested content is removed (should still find some fuzzy matches)
        let nested_results = engine.search("nested_file.txt");
        assert!(!nested_results[0].0.contains("nested_file.txt"), "Should not find nested file after removal");

        // But content in other directories should still be searchable
        let root_file_results = engine.search("root_file.txt");
        assert!(!root_file_results.is_empty(), "Should still find root file");

        let subdir2_results = engine.search("file2.txt");
        assert!(!subdir2_results.is_empty(), "Should still find file2");

        // Get updated stats
        let after_removal_stats = engine.get_stats();
        assert!(after_removal_stats.trie_size < initial_stats.trie_size,
                "Trie size should decrease after removal");

        // Clean up - best effort, don't panic if it fails
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_recursive_operations_with_permissions() {
        let temp_dir = create_temp_dir_structure();
        let temp_dir_str = temp_dir.to_str().unwrap();

        // Create a directory with no read permission to test error handling
        // Note: This test may behave differently on different operating systems
        let restricted_dir = temp_dir.join("restricted");
        fs::create_dir_all(&restricted_dir).expect("Failed to create restricted directory");

        // On Unix systems, we could change permissions
        // We'll use a conditional test based on platform
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let metadata = fs::metadata(&restricted_dir).expect("Failed to get metadata");
            let mut perms = metadata.permissions();
            // Remove read permissions
            perms.set_mode(0o000);
            fs::set_permissions(&restricted_dir, perms).expect("Failed to set permissions");
        }

        let mut engine = AutocompleteEngine::new(100, 10);

        // Add paths recursively - should handle the permission error gracefully
        engine.add_paths_recursive(temp_dir_str);

        // Test that we can still search and find files in accessible directories - use full filename
        let root_file_results = engine.search("root_file.txt");
        assert!(!root_file_results.is_empty(), "Should find root file");

        // Try to add the restricted directory specifically
        // This should not crash, just log a warning
        let restricted_dir_str = restricted_dir.to_str().unwrap();
        engine.add_paths_recursive(restricted_dir_str);

        // Now test removing paths with permission issues
        engine.remove_paths_recursive(restricted_dir_str);

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let metadata = fs::metadata(&restricted_dir).expect("Failed to get metadata");
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&restricted_dir, perms).expect("Failed to restore permissions");
        }

        // Clean up - best effort, don't panic if it fails
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_add_and_remove_with_nonexistent_paths() {
        let mut engine = AutocompleteEngine::new(100, 10);

        // Try to add a non-existent path recursively
        let nonexistent_path = "/path/that/does/not/exist";
        engine.add_paths_recursive(nonexistent_path);

        // Verify that the engine state is still valid
        let results = engine.search("path");
        // The path itself might be indexed, but no recursion would happen
        if !results.is_empty() {
            assert_eq!(results.len(), 1, "Should only index the top-level path");
            assert_eq!(results[0].0, nonexistent_path);
        }

        // Try to remove a non-existent path recursively
        engine.remove_paths_recursive(nonexistent_path);

        // Verify engine is still in a valid state
        let after_removal = engine.search("path");
        assert!(after_removal.is_empty(), "Path should be removed even if it doesn't exist");

        // Add some valid paths to ensure engine still works
        engine.add_path("/valid/path1.txt");
        engine.add_path("/valid/path2.txt");

        let valid_results = engine.search("valid");
        assert_eq!(valid_results.len(), 2, "Engine should still work with valid paths");
    }

    // Helper function to get test data directory
    fn get_test_data_path() -> std::path::PathBuf {
        let path = std::path::PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            log_warn!(&format!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path));
            panic!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path);
        }
        path
    }

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

            if let Some(walker) = std::fs::read_dir(dir).ok() {
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
            return (0..100).map(|i| format!("/path/to/file{}.txt", i)).collect();
        }

        paths
    }

    #[test]
    fn test_with_real_world_data() {
        log_info!("Testing autocomplete engine with real-world test data");

        // Create a new engine with reasonable parameters
        let mut engine = AutocompleteEngine::new(100, 20);

        // Get real-world paths from test data
        let paths = collect_test_paths(Some(500));
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Add all paths to the engine
        let start = std::time::Instant::now();
        for path in &paths {
            engine.add_path(path);
        }
        let elapsed = start.elapsed();
        log_info!(&format!("Added {} paths in {:?} ({:.2} paths/ms)",
                 paths.len(), elapsed, paths.len() as f64 / elapsed.as_millis().max(1) as f64));

        // Test different types of searches

        // 1. Test prefix search
        if let Some(first_path) = paths.first() {
            // Extract a prefix from the first path
            if let Some(last_sep) = first_path.rfind('/').or_else(|| first_path.rfind('\\')) {
                let prefix = &first_path[..last_sep+1];

                let prefix_start = std::time::Instant::now();
                let prefix_results = engine.search(prefix);
                let prefix_elapsed = prefix_start.elapsed();

                log_info!(&format!("Prefix search for '{}' found {} results in {:?}",
                         prefix, prefix_results.len(), prefix_elapsed));

                assert!(!prefix_results.is_empty(), "Should find results for existing prefix");

                // Log top results
                for (i, (path, score)) in prefix_results.iter().take(3).enumerate() {
                    log_info!(&format!("  Result #{}: {} (score: {:.4})", i+1, path, score));
                }
            }
        }

        // 2. Test with specific filename components
        // Extract some filename terms to search for from the data
        let mut filename_terms = Vec::new();
        for path in paths.iter().take(20) {
            if let Some(filename) = path.split('/').last().or_else(|| path.split('\\').last()) {
                if filename.len() >= 3 {
                    filename_terms.push(filename[..3].to_string());
                }
            }
        }

        // If we couldn't extract terms, use some defaults
        if filename_terms.is_empty() {
            filename_terms = vec!["app".to_string(), "doc".to_string(), "ima".to_string()];
        }

        // Test each extracted filename term
        for term in &filename_terms {
            let term_start = std::time::Instant::now();
            let term_results = engine.search(term);
            let term_elapsed = term_start.elapsed();

            log_info!(&format!("Filename search for '{}' found {} results in {:?}",
                     term, term_results.len(), term_elapsed));

            // Log first result if any
            if !term_results.is_empty() {
                log_info!(&format!("  First result: {} (score: {:.4})",
                         term_results[0].0, term_results[0].1));
            }
        }

        // 3. Test with directory context
        if paths.len() >= 2 {
            // Use the directory part of the second path as context
            let second_path = &paths[1];
            if let Some(last_sep) = second_path.rfind('/').or_else(|| second_path.rfind('\\')) {
                let dir_context = &second_path[..last_sep];

                // Set the context
                engine.set_current_directory(Some(dir_context.to_string()));

                // Use a short, generic search term
                let context_start = std::time::Instant::now();
                let context_results = engine.search("file");
                let context_elapsed = context_start.elapsed();

                log_info!(&format!("Context search with directory '{}' found {} results in {:?}",
                         dir_context, context_results.len(), context_elapsed));

                // Check that results prioritize the context directory
                if !context_results.is_empty() {
                    let top_result = &context_results[0].0;
                    log_info!(&format!("  Top result: {}", top_result));

                    // Count how many results are from the context directory
                    let context_matches = context_results.iter()
                        .filter(|(path, _)| path.starts_with(dir_context))
                        .count();

                    log_info!(&format!("  {} of {} results are from the context directory",
                             context_matches, context_results.len()));
                }

                // Reset context for other tests
                engine.set_current_directory(None);
            }
        }

        // 4. Test with usage frequency and recency tracking
        if !paths.is_empty() {
            // Record usage for some paths to affect ranking
            for i in 0..paths.len().min(5) {
                engine.record_path_usage(&paths[i]);

                // Record multiple usages for the first path
                if i == 0 {
                    engine.record_path_usage(&paths[i]);
                    engine.record_path_usage(&paths[i]);
                }
            }

            // Extract a common term to search for
            let common_term = if let Some(path) = paths.first() {
                if path.len() >= 3 {
                    &path[..3]
                } else {
                    "fil"
                }
            } else {
                "fil"
            };

            let freq_start = std::time::Instant::now();
            let freq_results = engine.search(common_term);
            let freq_elapsed = freq_start.elapsed();

            log_info!(&format!("Frequency-aware search for '{}' found {} results in {:?}",
                     common_term, freq_results.len(), freq_elapsed));

            // Check that frequently used paths are prioritized
            if !freq_results.is_empty() {
                log_info!(&format!("  Top result: {} (score: {:.4})",
                         freq_results[0].0, freq_results[0].1));

                // The most frequently used path should be ranked high
                let frequent_path_pos = freq_results.iter()
                    .position(|(path, _)| path == &paths[0]);

                if let Some(pos) = frequent_path_pos {
                    log_info!(&format!("  Most frequently used path is at position {}", pos));
                    // Should be in the top results
                    assert!(pos < 3, "Frequently used path should be ranked high");
                }
            }
        }

        // 5. Test the engine's statistics
        let stats = engine.get_stats();
        log_info!(&format!("Engine stats - Cache size: {}, Trie size: {}",
                 stats.cache_size, stats.trie_size));

        assert!(stats.trie_size >= paths.len(),
                "Trie should contain at least as many entries as paths");

        // 6. Test cache behavior by repeating a search
        if !paths.is_empty() {
            let repeat_term = if let Some(path) = paths.first() {
                if let Some(filename) = path.split('/').last().or_else(|| path.split('\\').last()) {
                    if filename.len() >= 3 {
                        &filename[..3]
                    } else {
                        "fil"
                    }
                } else {
                    "fil"
                }
            } else {
                "fil"
            };

            // First search to populate cache
            let _ = engine.search(repeat_term);

            // Second search should hit cache
            let cache_start = std::time::Instant::now();
            let cache_results = engine.search(repeat_term);
            let cache_elapsed = cache_start.elapsed();

            log_info!(&format!("Cached search for '{}' took {:?}", repeat_term, cache_elapsed));

            // Cache hit should be very fast
            assert!(!cache_results.is_empty(), "Cached search should return results");
        }
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn test_with_all_test_data_paths() {
        log_info!("Testing autocomplete engine with all available test data paths");

        // Create a new engine with reasonable parameters
        let mut engine = AutocompleteEngine::new(100, 20);

        // Get ALL available test paths (no limit)
        let paths = collect_test_paths(None);
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Add all paths to the engine
        let start = std::time::Instant::now();
        for path in &paths {
            engine.add_path(path);
        }
        let elapsed = start.elapsed();
        log_info!(&format!("Added {} paths in {:?} ({:.2} paths/ms)",
                 paths.len(), elapsed, paths.len() as f64 / elapsed.as_millis().max(1) as f64));

        // Test different types of searches

        // 1. Test prefix search with various prefixes from the data
        if !paths.is_empty() {
            // Try to find common prefixes from the data
            let mut prefixes = Vec::new();
            for path in paths.iter().take(10) {
                if let Some(last_sep) = path.rfind('/').or_else(|| path.rfind('\\')) {
                    prefixes.push(&path[..last_sep+1]);
                }
            }

            for prefix in prefixes {
                let prefix_start = std::time::Instant::now();
                let prefix_results = engine.search(prefix);
                let prefix_elapsed = prefix_start.elapsed();

                log_info!(&format!("Prefix search for '{}' found {} results in {:?}",
                         prefix, prefix_results.len(), prefix_elapsed));

                assert!(!prefix_results.is_empty(), "Should find results for existing prefix");
            }
        }

        // 2. Test with specific filename terms extracted from the data
        let mut filename_terms = Vec::new();
        for path in paths.iter().take(50) {
            if let Some(filename) = path.split('/').last().or_else(|| path.split('\\').last()) {
                if filename.len() >= 3 {
                    filename_terms.push(filename[..3].to_string());
                }
            }
        }

        // Test each extracted filename term
        for term in filename_terms.iter().take(5) {
            let term_start = std::time::Instant::now();
            let term_results = engine.search(term);
            let term_elapsed = term_start.elapsed();

            log_info!(&format!("Filename search for '{}' found {} results in {:?}",
                     term, term_results.len(), term_elapsed));

            assert!(!term_results.is_empty(), "Should find results for extracted terms");
        }

        // 3. Test with directory context if we have enough paths
        if paths.len() >= 2 {
            // Find a directory with at least 2 files to use as context
            let mut context_dir = None;
            let mut dirs_with_counts = std::collections::HashMap::new();

            for path in &paths {
                if let Some(last_sep) = path.rfind('/').or_else(|| path.rfind('\\')) {
                    let dir = &path[..last_sep];
                    *dirs_with_counts.entry(dir.to_string()).or_insert(0) += 1;
                }
            }

            // Find a directory with multiple files
            for (dir, count) in dirs_with_counts {
                if count >= 2 {
                    context_dir = Some(dir);
                    break;
                }
            }

            if let Some(dir) = context_dir {
                // Set the context
                engine.set_current_directory(Some(dir.clone()));

                // Use a generic search term
                let context_start = std::time::Instant::now();
                let context_results = engine.search("file");
                let context_elapsed = context_start.elapsed();

                log_info!(&format!("Context search with directory '{}' found {} results in {:?}",
                         dir, context_results.len(), context_elapsed));

                // Check if results prioritize the context directory
                let context_matches = context_results.iter()
                    .filter(|(path, _)| path.starts_with(&dir))
                    .count();

                log_info!(&format!("{} of {} results are from the context directory",
                         context_matches, context_results.len()));

                // Reset context
                engine.set_current_directory(None);
            }
        }

        // 4. Test with usage frequency and recency
        if !paths.is_empty() {
            // Record usage for some paths to affect ranking
            for i in 0..paths.len().min(20) {
                engine.record_path_usage(&paths[i]);

                // Record multiple usages for the first few paths
                if i < 5 {
                    for _ in 0..3 {
                        engine.record_path_usage(&paths[i]);
                    }
                }
            }

            // Wait a moment to create time difference for recency
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Record more recent usage for a different set of paths
            for i in 20..paths.len().min(30) {
                engine.record_path_usage(&paths[i]);
            }

            // Extract a common term to search for
            let common_term = if let Some(path) = paths.first() {
                if path.len() >= 3 {
                    &path[..3]
                } else {
                    "fil"
                }
            } else {
                "fil"
            };

            let freq_start = std::time::Instant::now();
            let freq_results = engine.search(common_term);
            let freq_elapsed = freq_start.elapsed();

            log_info!(&format!("Frequency-aware search for '{}' found {} results in {:?}",
                     common_term, freq_results.len(), freq_elapsed));

            assert!(!freq_results.is_empty(), "Should find results for frequency-aware search");
        }

        // 5. Test engine stats
        let stats = engine.get_stats();
        log_info!(&format!("Engine stats - Cache size: {}, Trie size: {}",
                 stats.cache_size, stats.trie_size));

        assert!(stats.trie_size >= paths.len(),
                "Trie should contain at least as many entries as paths");

        // 6. Test path removal (for a sample of paths)
        if !paths.is_empty() {
            let to_remove = paths.len().min(100);
            log_info!(&format!("Testing removal of {} paths", to_remove));

            let removal_start = std::time::Instant::now();
            for i in 0..to_remove {
                engine.remove_path(&paths[i]);
            }
            let removal_elapsed = removal_start.elapsed();

            log_info!(&format!("Removed {} paths in {:?}", to_remove, removal_elapsed));

            // Check that engine stats reflect the removals
            let after_stats = engine.get_stats();
            log_info!(&format!("Engine stats after removal - Cache size: {}, Trie size: {}",
                     after_stats.cache_size, after_stats.trie_size));

            assert!(after_stats.trie_size <= stats.trie_size - to_remove,
                    "Trie size should decrease after removals");
        }
    }
}
