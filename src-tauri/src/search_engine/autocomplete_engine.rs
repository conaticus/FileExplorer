use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::search_engine::art_v3::ART;
use crate::search_engine::fast_fuzzy_v2::PathMatcher;
use crate::search_engine::path_cache_wrapper::{PathCache, PathData};
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
}

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
        }
    }
    
    /// Set the current directory context for improved ranking
    pub fn set_current_directory(&mut self, directory: Option<String>) {
        self.current_directory = directory;
    }
    
    /// Add or update a path in the search engines
    pub fn add_path(&mut self, path: &str) {
        // Calculate initial score (can be adjusted based on various factors)
        let mut score = 1.0;
        
        // Check if we have existing frequency data to adjust score
        if let Some(freq) = self.frequency_map.get(path) {
            // Boost score for frequently accessed paths
            score += (*freq as f32) * 0.01;
        }
        
        // Update the trie
        self.trie.insert(path, score);
        
        // Update the fuzzy matcher
        self.fuzzy_matcher.add_path(path);
        
        // Clean the cache when adding new paths to ensure fresh results
        self.cache.purge_expired();
    }
    
    /// Remove a path from the search engines
    pub fn remove_path(&mut self, path: &str) {
        // Remove from trie
        self.trie.remove(path);
        
        // For fuzzy matcher, we would ideally remove it, but the current API 
        // doesn't support removal, so we'll have to rebuild the index periodically
        
        // Remove from cache
        self.cache.remove(path);
        
        // Remove from frequency and recency maps
        self.frequency_map.remove(path);
        self.recency_map.remove(path);
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
    
    /// Search for path completions using the engine's combined strategy
    pub fn search(&mut self, query: &str) -> Vec<(String, f32)> {
        if query.is_empty() {
            return Vec::new();
        }
        
        let normalized_query = query.trim().to_string();
        
        // 1. Check cache first
        if let Some(path_data) = self.cache.get(&normalized_query) {
            log_info!(&format!("Cache hit for query: '{}'", normalized_query));
            // Return cached results
            return vec![(path_data.path, path_data.score)];
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
                // More recently used paths get a boost (max 0.3 for very recent)
                let recency_boost = 0.3 * (1.0 - (age.as_secs_f32() / 86400.0).min(1.0));
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
                if self.preferred_extensions.contains(&ext) {
                    // Preferred file types get a boost
                    new_score += 0.15;
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
                    new_score += 0.5;
                } else if filename.to_lowercase().starts_with(&query.to_lowercase()) {
                    // Filename prefix matches get a medium boost
                    new_score += 0.3;
                } else if filename.to_lowercase().contains(&query.to_lowercase()) {
                    // Filename contains matches get a small boost
                    new_score += 0.1;
                }
            }
            
            // Update the score
            *score = new_score;
        }
        
        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    /// Clear all data and caches
    pub fn clear(&mut self) {
        self.trie.clear();
        self.cache.clear();
        self.frequency_map.clear();
        self.recency_map.clear();
        
        // For the fuzzy matcher, we'll need to rebuild it from scratch
        self.fuzzy_matcher = PathMatcher::new();
    }
    
    /// Get statistics about the engine
    pub fn get_stats(&self) -> EngineStats {
        EngineStats {
            cache_size: self.cache.len(),
            trie_size: self.trie.len(),
            tracked_paths: self.frequency_map.len(),
        }
    }
}

/// Statistics about the autocomplete engine
pub struct EngineStats {
    pub cache_size: usize,
    pub trie_size: usize,
    pub tracked_paths: usize,
}

#[cfg(test)]
mod tests_autocomplete_engine {
    use super::*;
    use std::thread::sleep;
    
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
        sleep(Duration::from_millis(10));
        
        // Record newer usage for b.txt
        engine.record_path_usage("/path/b.txt");
        
        // Search for common prefix
        let results = engine.search("/path/");
        
        // b.txt should be first (most recent), followed by a.txt (most frequent)
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/path/b.txt");
        assert_eq!(results[1].0, "/path/a.txt");
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

    #[test]
    fn test_directory_aware_search() {
        let mut engine = AutocompleteEngine::new(100, 10);

        // Add some test paths
        engine.add_path("/home/user/documents/report.pdf");
        engine.add_path("/home/user/documents/presentation.pptx");
        engine.add_path("/home/user/pictures/vacation.jpg");
        engine.add_path("/var/log/system.log");

        // Set current directory context
        engine.set_current_directory(Some("/home/user".to_string()));

        // Test searching for a relative path from current directory
        let results = engine.search("doc");

        // Should find documents directory entries
        assert!(!results.is_empty());
        log_info!(&format!("Directory-aware search for 'doc' found {} results", results.len()));

        // Verify the correct files were found
        let doc_paths = results.iter()
            .filter(|(path, _)| path.contains("documents"))
            .collect::<Vec<_>>();

        assert!(!doc_paths.is_empty(), "Should find paths containing 'documents'");
        assert_eq!(doc_paths.len(), 2, "Should find both document files");

        // Make sure files from other directories aren't prioritized
        assert!(results[0].0.contains("/home/user/documents/"),
                "First result should be from documents directory");

        // Test with a different current directory
        engine.set_current_directory(Some("/var".to_string()));
        let var_results = engine.search("log");

        assert!(!var_results.is_empty());
        assert!(var_results[0].0.contains("/var/log/"),
                "First result should be from log directory");
    }
}
