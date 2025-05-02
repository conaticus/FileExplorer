use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::SystemTime;
use crate::log_info;
use crate::log_error;

// Enhanced radix node that supports adaptive segmentation
pub struct AdaptiveRadixNode {
    // Maps segments to child nodes
    pub children: HashMap<String, Arc<RwLock<AdaptiveRadixNode>>>,
    // Terminal node value (if this node represents a complete path)
    pub value: Option<PathBuf>,
    // Tracking access patterns
    pub frequency: u32,
    pub last_accessed: SystemTime,
    // The segment this node represents (could be a directory name or file name)
    pub key_segment: String,
    // Whether this segment is a directory separator
    pub is_separator: bool,
}

impl AdaptiveRadixNode {
    pub fn new(segment: String, is_separator: bool) -> Self {
        Self {
            children: HashMap::new(),
            value: None,
            frequency: 0,
            last_accessed: SystemTime::now(),
            key_segment: segment,
            is_separator: is_separator,
        }
    }

    pub fn new_root() -> Self {
        Self::new(String::new(), false)
    }
}

pub struct AdaptiveRadixTrie {
    pub root: Arc<RwLock<AdaptiveRadixNode>>,
    // Configuration for segmentation
    min_segment_length: usize,
    // Whether to use directory separators as segment boundaries
    use_path_separators: bool,
}

impl AdaptiveRadixTrie {
    pub fn new() -> Self {
        log_info!("Creating new AdaptiveRadixTrie");
        Self {
            root: Arc::new(RwLock::new(AdaptiveRadixNode::new_root())),
            min_segment_length: 2,
            use_path_separators: true,
        }
    }

    pub fn insert(&self, path_str: &str, path: PathBuf) {
        // Process the path with adaptive segmentation
        let segments = self.segment_path(path_str);

        if let Ok(mut root) = self.root.write() {
            self.insert_segments(&mut root, &segments, 0, path);
        } else {
            log_error!(&format!("Failed to acquire write lock for inserting path: {}", path_str));
        }
    }

    fn insert_segments(
        &self,
        node: &mut AdaptiveRadixNode,
        segments: &[String],
        index: usize,
        path: PathBuf
    ) {
        if index >= segments.len() {
            // We've reached the end of the path, mark as terminal node
            node.value = Some(path);
            node.frequency += 1;
            node.last_accessed = SystemTime::now();
            return;
        }

        let segment = &segments[index];
        let is_separator = segment == "/" || segment == "\\";

        // Find or create child node for this segment
        if !node.children.contains_key(segment) {
            let new_node = AdaptiveRadixNode::new(segment.clone(), is_separator);
            node.children.insert(segment.clone(), Arc::new(RwLock::new(new_node)));
        }

        // Continue insertion with next segment
        if let Some(child_arc) = node.children.get(segment) {
            if let Ok(mut child) = child_arc.write() {
                self.insert_segments(&mut child, segments, index + 1, path);
            }
        }
    }

    // The key method that implements adaptive segmentation
    fn segment_path(&self, path: &str) -> Vec<String> {
        if !self.use_path_separators {
            // Character-by-character segmentation - useful for very short paths or names
            return path.chars().map(|c| c.to_string()).collect();
        }

        let mut segments = Vec::new();
        let mut current_segment = String::new();

        // Normalize path separators for consistent handling
        let normalized_path = path.replace('\\', "/");

        for c in normalized_path.chars() {
            if c == '/' {
                // Add current segment if not empty
                if !current_segment.is_empty() {
                    segments.push(current_segment);
                    current_segment = String::new();
                }

                // Add separator as its own segment - using normalized separator
                segments.push("/".to_string());
            } else {
                current_segment.push(c);
            }
        }

        // Add the final segment if not empty
        if !current_segment.is_empty() {
            segments.push(current_segment);
        }

        // Adaptive logic: if the path is very short and doesn't have separators,
        // we can further segment by characters or n-grams
        if segments.len() <= 1 && path.len() > self.min_segment_length {
            // Segment by n-grams for very short paths without directory structure
            let n = 2; // Use 2-grams
            let chars: Vec<char> = path.chars().collect();

            segments.clear();

            for i in 0..chars.len() {
                if i + n <= chars.len() {
                    let ngram: String = chars[i..i+n].iter().collect();
                    segments.push(ngram);
                } else if i < chars.len() {
                    // Remaining chars
                    let remaining: String = chars[i..].iter().collect();
                    segments.push(remaining);
                    break;
                }
            }
        }

        segments
    }

    pub fn find_with_prefix(&self, prefix: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();

        // Segment the prefix query
        let segments = self.segment_path(prefix);

        if let Ok(root) = self.root.read() {
            log_info!(&format!("Searching with prefix: {}", prefix));
            self.find_with_segments(&root, &segments, 0, &mut results);
            
            // If an exact path match is expected but no results were found
            // try again with normalized path separators
            if results.is_empty() && prefix.contains('\\') {
                let normalized_prefix = prefix.replace('\\', "/");
                let normalized_segments = self.segment_path(&normalized_prefix);
                self.find_with_segments(&root, &normalized_segments, 0, &mut results);
            }
            
            // Special case: if we're searching for an exact path and it's not already in results,
            // try to find it directly
            if results.is_empty() || (results.len() == 1 && results[0].to_string_lossy() != prefix) {
                // Try both original and normalized versions of the path
                if let Some(child_arc) = self.find_exact_path(&root, prefix) {
                    if let Ok(child) = child_arc.read() {
                        if let Some(path) = &child.value {
                            if !results.contains(path) {
                                results.push(path.clone());
                            }
                        }
                    }
                }
                
                // Try with normalized path
                let normalized_prefix = prefix.replace('\\', "/");
                if normalized_prefix != prefix {
                    if let Some(child_arc) = self.find_exact_path(&root, &normalized_prefix) {
                        if let Ok(child) = child_arc.read() {
                            if let Some(path) = &child.value {
                                if !results.contains(path) {
                                    results.push(path.clone());
                                }
                            }
                        }
                    }
                }
            }
        } else {
            log_error!(&format!("Failed to acquire read lock for searching prefix: {}", prefix));
        }

        log_info!(&format!("Found {} results for prefix: {}", results.len(), prefix));
        results
    }
    
    // Helper method to find an exact path in the trie
    fn find_exact_path(&self, node: &AdaptiveRadixNode, path: &str) -> Option<Arc<RwLock<AdaptiveRadixNode>>> {
        let segments = self.segment_path(path);
        
        // Start with children of the provided node
        let mut current_arc_opt: Option<Arc<RwLock<AdaptiveRadixNode>>> = None;
        
        // Initialize with the first segment
        if let Some(first_segment) = segments.first() {
            current_arc_opt = node.children.get(first_segment).cloned();
            
            // Try alternative separator if this is a separator segment
            if current_arc_opt.is_none() && (first_segment == "/" || first_segment == "\\") {
                let alt_segment = if first_segment == "/" { "\\" } else { "/" };
                current_arc_opt = node.children.get(alt_segment).cloned();
            }
        }
        
        // Process remaining segments
        for segment in segments.iter().skip(1) {
            // Get the current node if it exists
            let current_arc = match &current_arc_opt {
                Some(arc) => arc.clone(),
                None => return None, // Segment not found
            };
            
            // Try to read the current node
            let current_node = match current_arc.read() {
                Ok(node) => node,
                Err(_) => return None, // Failed to acquire read lock
            };
            
            // Update current_arc_opt to the next segment
            current_arc_opt = current_node.children.get(segment).cloned();
            
            // Try with alternative separator if needed
            if current_arc_opt.is_none() && (segment == "/" || segment == "\\") {
                let alt_segment = if segment == "/" { "\\" } else { "/" };
                current_arc_opt = current_node.children.get(alt_segment).cloned();
            }
            
            // If we couldn't find the next segment, stop early
            if current_arc_opt.is_none() {
                return None;
            }
        }
        
        // Check if the final node represents a complete path
        if let Some(current_arc) = &current_arc_opt.clone() {
            if let Ok(current_node) = current_arc.read() {
                if current_node.value.is_some() {
                    return current_arc_opt;
                }
            }
        }
        
        None
    }

    fn find_with_segments(
        &self,
        node: &AdaptiveRadixNode,
        segments: &[String],
        index: usize,
        results: &mut Vec<PathBuf>
    ) {
        // If we've consumed all segments, this is a prefix match
        if index >= segments.len() {
            // Add this node's value if it exists
            if let Some(path) = &node.value {
                results.push(path.clone());
            }

            // Recursively add all children's values
            for (_, child_arc) in &node.children {
                if let Ok(child) = child_arc.read() {
                    self.collect_all_values(&child, results);
                }
            }

            return;
        }

        let segment = &segments[index];

        // Special case: if the last segment is only a partial match,
        // we need to find all children that start with this segment
        if index == segments.len() - 1 {
            for (child_segment, child_arc) in &node.children {
                if child_segment.starts_with(segment) || segment.starts_with(child_segment) {
                    if let Ok(child) = child_arc.read() {
                        // For directory separator matches, be more flexible
                        let is_separator_match = (segment == "/" || segment == "\\") && 
                                                (child_segment == "/" || child_segment == "\\");
                        
                        if child_segment.starts_with(segment) || is_separator_match {
                            // Add this node's value if it exists
                            if let Some(path) = &child.value {
                                results.push(path.clone());
                            }

                            // Recursively add all children's values
                            self.collect_all_values(&child, results);
                        }
                    }
                }
            }
        } else {
            // Exact match for a complete segment
            // Try with both normalized and original separators
            let alt_segment = if segment == "/" { "\\".to_string() } else if segment == "\\" { "/".to_string() } else { segment.clone() };
            
            // First try with the original segment
            if let Some(child_arc) = node.children.get(segment) {
                if let Ok(child) = child_arc.read() {
                    self.find_with_segments(&child, segments, index + 1, results);
                }
            } 
            // Then try with the alternative separator if needed
            else if segment != &alt_segment {
                if let Some(child_arc) = node.children.get(&alt_segment) {
                    if let Ok(child) = child_arc.read() {
                        self.find_with_segments(&child, segments, index + 1, results);
                    }
                }
            }
        }
    }

    fn collect_all_values(&self, node: &AdaptiveRadixNode, results: &mut Vec<PathBuf>) {
        if let Some(path) = &node.value {
            results.push(path.clone());
        }

        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                self.collect_all_values(&child, results);
            }
        }
    }

    pub fn remove(&self, path_str: &str) -> bool {
        let segments = self.segment_path(path_str);

        if let Ok(mut root) = self.root.write() {
            // Only return true if a value was actually removed
            log_info!(&format!("Removing path: {}", path_str));
            let (removed_value, _) = self.remove_segments(&mut root, &segments, 0);
            if removed_value {
                log_info!(&format!("Successfully removed path: {}", path_str));
            } else {
                log_info!(&format!("Path not found for removal: {}", path_str));
            }
            return removed_value;
        } else {
            log_error!(&format!("Failed to acquire write lock for removing path: {}", path_str));
            false
        }
    }

    fn remove_segments(
        &self,
        node: &mut AdaptiveRadixNode,
        segments: &[String],
        index: usize
    ) -> (bool, bool) {
        if index >= segments.len() {
            // Path found, remove the value
            let had_value = node.value.is_some();
            if had_value {
                node.value = None;
                return (true, node.children.is_empty());
            }
            return (false, false);
        }

        let segment = &segments[index];
        let mut value_removed = false;
        let mut should_remove_child = false;

        if let Some(child_arc) = node.children.get(segment) {
            if let Ok(mut child) = child_arc.write() {
                let (removed, empty) = self.remove_segments(&mut child, segments, index + 1);
                value_removed = removed;
                should_remove_child = empty;
            }

            if should_remove_child {
                node.children.remove(segment);
            }
        }

        // Return if a value was removed and if this node can be removed
        (value_removed, node.value.is_none() && node.children.is_empty())
    }

    pub fn get_path_count(&self) -> usize {
        let mut count = 0;
        if let Ok(root) = self.root.read() {
            count = self.count_all_values(&root);
            log_info!(&format!("Current path count: {}", count));
        } else {
            log_error!("Failed to acquire read lock for counting paths");
        }
        count
    }

    fn count_all_values(&self, node: &AdaptiveRadixNode) -> usize {
        let mut count = if node.value.is_some() { 1 } else { 0 };

        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                count += self.count_all_values(&child);
            }
        }

        count
    }
}

#[cfg(test)]
mod tests_adaptive_radix_tree {
    use super::*;
    use std::time::{Duration, Instant};
    use crate::search_engine::{Entry, generate_test_data, index_given_path, index_given_path_parallel};

    // Helper function to get the test data path and verify it exists
    fn get_test_data_path() -> PathBuf {
        let mut path = PathBuf::from(".");
        path = path.join("test-data-for-search-engine");
        if !path.exists() {
            panic!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path);
        }
        path
    }

    #[test]
    #[ignore]
    fn test_generate_test_data() {
        match generate_test_data(get_test_data_path()) {
            Ok(path) => {
                assert!(path.exists(), "Test data directory should exist");

                // Test sequential vs parallel indexing of the test data
                let start = std::time::Instant::now();
                let seq_entries = index_given_path(path.clone());
                let seq_duration = start.elapsed();

                let start = std::time::Instant::now();
                let par_entries = index_given_path_parallel(path.clone());
                let par_duration = start.elapsed();

                log_info!(&format!("Test data indexed:"));
                log_info!(&format!("  - Sequential: {} entries in {:?}", seq_entries.len(), seq_duration));
                log_info!(&format!("  - Parallel: {} entries in {:?}", par_entries.len(), par_duration));

                assert!(!seq_entries.is_empty(), "Should have indexed some entries");
                assert_eq!(seq_entries.len(), par_entries.len(),
                           "Sequential and parallel indexing should find the same number of entries");
            },
            Err(e) => {
                panic!("Failed to generate test data: {}", e);
            }
        }
    }

    #[test]
    #[cfg(feature = "generate-test-data")]
    fn create_test_data() {
        match generate_test_data(get_test_data_path()) {
            Ok(path) => log_info!(&format!("Test data created at: {:?}", path)),
            Err(e) => panic!("Failed to generate test data: {}", e)
        }
    }

    #[test]
    fn test_index_real_test_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Get paths from test data
        let entries = index_given_path(test_path);
        let total_entries = entries.len();
        log_info!(&format!("Indexing {} real test paths into trie", total_entries));

        let start = Instant::now();
        for entry in &entries {
            match entry {
                Entry::FILE(file) => {
                    let path_str = file.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                },
                Entry::DIRECTORY(dir) => {
                    let path_str = dir.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                }
            }
        }
        let insertion_time = start.elapsed();

        log_info!(&format!("Inserted {} real paths in {:?}", total_entries, insertion_time));

        // Verify count matches
        assert_eq!(trie.get_path_count(), total_entries,
            "Trie should contain exactly {} paths", total_entries);

        // Test some prefix searches
        if total_entries > 0 {
            // Try a generic search that should return some results
            let first_entry = &entries[0];
            let path = match first_entry {
                Entry::FILE(file) => &file.path,
                Entry::DIRECTORY(dir) => &dir.path,
            };
            let prefix = if path.contains('/') { "/" } else { "C:\\" };
            let start = Instant::now();
            let results = trie.find_with_prefix(prefix);
            let search_time = start.elapsed();

            log_info!(&format!("Found {} paths with prefix '{}' in {:?}",
                results.len(), prefix, search_time));

            assert!(!results.is_empty(), "Should find at least some paths with generic prefix");
        }
    }

    #[test]
    fn test_parallel_vs_sequential_indexing_performance() {
        let test_path = get_test_data_path();

        // Gather test data paths using both methods
        let start = Instant::now();
        let seq_results = index_given_path(test_path.clone());
        let seq_time = start.elapsed();
        log_info!(&format!("Sequential indexing found {} paths in {:?}",
            seq_results.len(), seq_time));

        let start = Instant::now();
        let par_results = index_given_path_parallel(test_path.clone());
        let par_time = start.elapsed();
        log_info!(&format!("Parallel indexing found {} paths in {:?}",
            par_results.len(), par_time));

        // Compare results
        assert_eq!(seq_results.len(), par_results.len(),
            "Both methods should find the same number of files");

        // Create tries and compare insert performance
        let trie_seq = AdaptiveRadixTrie::new();
        let trie_par = AdaptiveRadixTrie::new();

        let start = Instant::now();
        for entry in &seq_results {
            match entry {
                Entry::FILE(file) => {
                    let path_str = file.path.clone();
                    trie_seq.insert(&path_str, PathBuf::from(&path_str));
                },
                Entry::DIRECTORY(dir) => {
                    let path_str = dir.path.clone();
                    trie_seq.insert(&path_str, PathBuf::from(&path_str));
                }
            }
        }
        let seq_insert_time = start.elapsed();
        log_info!(&format!("Sequential trie insertion: {:?}", seq_insert_time));

        let start = Instant::now();
        for entry in &par_results {
            match entry {
                Entry::FILE(file) => {
                    let path_str = file.path.clone();
                    trie_par.insert(&path_str, PathBuf::from(&path_str));
                },
                Entry::DIRECTORY(dir) => {
                    let path_str = dir.path.clone();
                    trie_par.insert(&path_str, PathBuf::from(&path_str));
                }
            }
        }
        let par_insert_time = start.elapsed();
        log_info!(&format!("Parallel data trie insertion: {:?}", par_insert_time));

        // The counts should match
        assert_eq!(trie_seq.get_path_count(), trie_par.get_path_count());
    }

    #[test]
    fn test_trie_with_real_world_queries() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the test data
        let entries = index_given_path(test_path);
        if entries.is_empty() {
            log_info!("No test paths found, skipping test");
            return;
        }

        // Insert all paths
        for entry in &entries {
            match entry {
                Entry::FILE(file) => {
                    let path_str = file.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                },
                Entry::DIRECTORY(dir) => {
                    let path_str = dir.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                }
            }
        }

        // Extract some realistic prefixes from the data
        let mut prefixes = Vec::new();
        for entry in &entries {
            let path_str = match entry {
                Entry::FILE(file) => &file.path,
                Entry::DIRECTORY(dir) => &dir.path,
            };

            // Add partial path prefixes - use platform-agnostic path handling
            let path_buf = PathBuf::from(path_str);
            if let Some(parent) = path_buf.parent() {
                if !parent.as_os_str().is_empty() {
                    let prefix1 = parent.to_string_lossy().to_string();
                    if !prefix1.is_empty() && !prefixes.contains(&prefix1) {
                        prefixes.push(prefix1);
                    }

                    // For the second level, try to go one level deeper if possible
                    if let Some(grandparent) = parent.parent() {
                        if !grandparent.as_os_str().is_empty() {
                            let prefix2 = format!("{}{}{}",
                                grandparent.to_string_lossy(),
                                std::path::MAIN_SEPARATOR,
                                parent.file_name().unwrap_or_default().to_string_lossy());
                            if !prefixes.contains(&prefix2) {
                                prefixes.push(prefix2);
                            }
                        }
                    }
                }
            }

            // Add filename prefix (first few chars)
            if let Some(filename) = PathBuf::from(path_str).file_name() {
                let filename_str = filename.to_string_lossy();
                if filename_str.len() >= 3 {
                    let filename_prefix = filename_str.chars().take(3).collect::<String>();
                    if !prefixes.contains(&filename_prefix) {
                        prefixes.push(filename_prefix);
                    }
                }
            }

            // Limit number of test prefixes
            if prefixes.len() >= 20 {
                break;
            }
        }

        log_info!(&format!("Testing {} real-world prefixes", prefixes.len()));

        let mut total_results = 0;
        let start = Instant::now();

        for prefix in &prefixes {
            let results = trie.find_with_prefix(prefix);
            log_info!(&format!("Prefix '{}' returned {} results", prefix, results.len()));
            total_results += results.len();
        }

        let total_time = start.elapsed();
        let avg_time = if prefixes.len() > 0 {
            total_time.div_f32(prefixes.len() as f32)
        } else {
            total_time
        };

        log_info!(&format!("Average query time for real-world prefixes: {:?}", avg_time));
        log_info!(&format!("Total results across all queries: {}", total_results));

        // Ensure we found something
        assert!(total_results > 0, "Should find at least some results with real-world prefixes");
    }

    #[test]
    fn test_remove_with_real_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the test data
        let entries = index_given_path(test_path);
        if entries.is_empty() {
            log_info!("No test paths found, skipping test");
            return;
        }

        // Insert all paths
        for entry in &entries {
            match entry {
                Entry::FILE(file) => {
                    let path_str = file.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                },
                Entry::DIRECTORY(dir) => {
                    let path_str = dir.path.clone();
                    trie.insert(&path_str, PathBuf::from(&path_str));
                }
            }
        }

        let total_count = trie.get_path_count();
        log_info!(&format!("Indexed {} real paths", total_count));

        // Remove about 10% of paths
        let remove_count = std::cmp::max(1, entries.len() / 10);
        
        // Create a vector of paths to remove
        let mut paths_to_remove = Vec::with_capacity(remove_count);
        for (i, entry) in entries.iter().enumerate() {
            if i % 10 == 0 && paths_to_remove.len() < remove_count {
                match entry {
                    Entry::FILE(file) => {
                        paths_to_remove.push(PathBuf::from(&file.path));
                    },
                    Entry::DIRECTORY(dir) => {
                        paths_to_remove.push(PathBuf::from(&dir.path));
                    }
                }
            }
        }

        log_info!(&format!("Removing {} paths...", paths_to_remove.len()));

        let start = Instant::now();
        for path in &paths_to_remove {
            let path_str = path.to_string_lossy().to_string();
            trie.remove(&path_str);
        }
        let remove_time = start.elapsed();

        let remaining_count = trie.get_path_count();
        log_info!(&format!("Removed {} paths in {:?}. {} paths remain.",
            paths_to_remove.len(), remove_time, remaining_count));

        // Verify count
        assert_eq!(total_count - paths_to_remove.len(), remaining_count,
            "Count after removal should be original count minus removed count");

        // Verify removed paths are actually gone
        for path in &paths_to_remove {
            let path_str = path.to_string_lossy().to_string();
            let results = trie.find_with_prefix(&path_str);

            // The exact path should not be in results
            assert!(!results.contains(path),
                "Removed path {} should not be found in trie", path_str);
        }
    }

    #[test]
    fn test_trie_basic_operations() {
        let trie = AdaptiveRadixTrie::new();
        
        // Test insertion
        trie.insert("/home/user/documents", PathBuf::from("/home/user/documents"));
        trie.insert("/home/user/pictures", PathBuf::from("/home/user/pictures"));
        trie.insert("/home/user/videos", PathBuf::from("/home/user/videos"));
        
        // Test count
        assert_eq!(trie.get_path_count(), 3, "Trie should contain 3 paths");
        
        // Test finding with prefix
        let results = trie.find_with_prefix("/home/user");
        assert_eq!(results.len(), 3, "Should find 3 paths with prefix '/home/user'");
        
        // Test exact path match
        let results = trie.find_with_prefix("/home/user/documents");
        assert_eq!(results.len(), 1, "Should find exact path match");
        assert_eq!(results[0], PathBuf::from("/home/user/documents"));
        
        // Test removal
        assert!(trie.remove("/home/user/documents"), "Removal should return true for existing path");
        assert_eq!(trie.get_path_count(), 2, "Path count should be reduced after removal");
        
        // Test finding after removal
        let results = trie.find_with_prefix("/home/user");
        assert_eq!(results.len(), 2, "Should find 2 paths after removal");
        assert!(!results.contains(&PathBuf::from("/home/user/documents")), "Removed path should not be found");
        
        // Test removing non-existent path
        assert!(!trie.remove("/home/user/nonexistent"), "Removing non-existent path should return false");
        assert_eq!(trie.get_path_count(), 2, "Path count should remain unchanged");
    }

    #[test]
    fn test_trie_empty_and_edge_cases() {
        let trie = AdaptiveRadixTrie::new();
        
        // Test empty trie operations
        assert_eq!(trie.get_path_count(), 0, "New trie should be empty");
        assert!(trie.find_with_prefix("/any/path").is_empty(), "Empty trie should return no results");
        assert!(!trie.remove("/nonexistent/path"), "Removing from empty trie should return false");
        
        // Test with empty string
        trie.insert("", PathBuf::from(""));
        assert_eq!(trie.get_path_count(), 1, "Trie should accept empty string");
        let results = trie.find_with_prefix("");
        assert_eq!(results.len(), 1, "Should find empty string path");
        assert!(trie.remove(""), "Should successfully remove empty string path");
        
        // Test with very long path
        let long_path = "/a/".repeat(100) + "file.txt";
        trie.insert(&long_path, PathBuf::from(&long_path));
        assert_eq!(trie.get_path_count(), 1, "Trie should accept very long path");
        assert!(trie.find_with_prefix("/a/").len() > 0, "Should find long path with prefix");
        assert!(trie.remove(&long_path), "Should successfully remove long path");
    }

    #[test]
    fn test_trie_segmentation_strategies() {
        let trie = AdaptiveRadixTrie::new();
        
        // Test with different types of paths to test segmentation logic
        let windows_path = "C:\\Users\\username\\Documents\\file.txt";
        let unix_path = "/home/username/documents/file.txt";
        let mixed_path = "C:/Users/username/Documents/file.txt";
        
        trie.insert(windows_path, PathBuf::from(windows_path));
        trie.insert(unix_path, PathBuf::from(unix_path));
        trie.insert(mixed_path, PathBuf::from(mixed_path));
        
        assert_eq!(trie.get_path_count(), 3, "All paths should be inserted regardless of separator style");
        
        // Test finding with different prefix styles
        let windows_results = trie.find_with_prefix("C:\\Users");
        let unix_results = trie.find_with_prefix("/home");
        let mixed_results = trie.find_with_prefix("C:/Users");
        
        assert!(!windows_results.is_empty(), "Should find Windows-style paths");
        assert!(!unix_results.is_empty(), "Should find Unix-style paths");
        assert!(!mixed_results.is_empty(), "Should find mixed-style paths");
        
        // Test that segmentation handles file names correctly
        let file_results = trie.find_with_prefix("file.txt");
        assert!(file_results.len() > 0, "Should find paths by filename");
    }
}

