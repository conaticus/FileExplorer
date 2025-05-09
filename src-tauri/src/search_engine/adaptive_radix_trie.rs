use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[allow(dead_code)]
/// A node in the Adaptive Radix Trie specifically optimized for file paths
pub struct AdaptiveRadixNode {
    /// Maps segments to child nodes
    children: HashMap<String, Arc<RwLock<AdaptiveRadixNode>>>,
    /// The complete path if this node is a terminal node
    path: Option<PathBuf>,
    /// The segment this node represents (e.g., directory or file name)
    segment: String,
    /// Frequency counter for this node
    frequency: u32,
    /// Last access time for ranking
    last_accessed: SystemTime,
    /// Parent node reference for hierarchical traversal
    parent: Option<Arc<RwLock<AdaptiveRadixNode>>>,
    /// Depth in the path hierarchy (root = 0)
    depth: usize,
    /// Whether this segment is case-sensitive
    case_sensitive: bool,
}

#[allow(dead_code)]
/// Adaptive Radix Trie optimized for file paths across platforms
pub struct AdaptiveRadixTrie {
    /// Root node of the trie
    root: Arc<RwLock<AdaptiveRadixNode>>,
    /// Configuration flag for default case sensitivity
    default_case_sensitive: bool,
    /// Total number of paths indexed
    path_count: Arc<RwLock<usize>>,
}

#[allow(dead_code)]
impl AdaptiveRadixNode {
    /// Create a new PathNode
    fn new(
        segment: String,
        case_sensitive: bool,
        parent: Option<Arc<RwLock<AdaptiveRadixNode>>>,
        depth: usize,
    ) -> Self {
        Self {
            children: HashMap::new(),
            path: None,
            segment,
            frequency: 0,
            last_accessed: SystemTime::now(),
            parent,
            depth,
            case_sensitive,
        }
    }

    /// Create a new root node
    fn new_root(case_sensitive: bool) -> Self {
        Self::new(String::new(), case_sensitive, None, 0)
    }

    /// Get normalized segment for comparison (handles case sensitivity)
    fn normalized_segment(&self, segment: &str) -> String {
        if self.case_sensitive {
            segment.to_string()
        } else {
            segment.to_lowercase()
        }
    }

    /// Check if this segment matches the provided segment (respecting case sensitivity)
    fn segment_matches(&self, other: &str) -> bool {
        if self.case_sensitive {
            self.segment == other
        } else {
            self.segment.to_lowercase() == other.to_lowercase()
        }
    }

    /// Increment access frequency of this node
    fn record_access(&mut self) {
        self.frequency += 1;
        self.last_accessed = SystemTime::now();
    }
}

#[allow(dead_code)]
impl AdaptiveRadixTrie {
    /// Create a new AdaptivePathTrie with specific case sensitivity
    pub fn new_with_arg(case_sensitive: bool) -> Self {
        Self {
            root: Arc::new(RwLock::new(AdaptiveRadixNode::new_root(case_sensitive))),
            default_case_sensitive: case_sensitive,
            path_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a new AdaptivePathTrie with default settings (case insensitive)
    pub fn new_default() -> Self {
        Self::new_with_arg(false)
    }

    /// No-parameter version of new() for backward compatibility
    pub fn new() -> Self {
        Self::new_with_arg(false)
    }

    /// Default implementation
    pub fn default() -> Self {
        Self::new_default()
    }

    /// Normalize a path for internal storage and comparison
    pub fn normalize_path(&self, path: &str) -> String {
        // Replace backslashes with forward slashes for uniform handling
        let normalized = path.replace('\\', "/");

        // Ensure drive letters are consistently formatted (lowercase for case-insensitive)
        let normalized = if normalized.len() >= 2 && normalized.chars().nth(1) == Some(':') {
            let mut chars: Vec<char> = normalized.chars().collect();
            if !self.default_case_sensitive {
                chars[0] = chars[0].to_lowercase().next().unwrap_or(chars[0]);
            }
            chars.into_iter().collect()
        } else {
            normalized
        };

        // Remove trailing slashes for consistency
        let normalized = normalized.trim_end_matches('/').to_string();

        // Handle case sensitivity based on configuration
        if self.default_case_sensitive {
            normalized
        } else {
            normalized.to_lowercase()
        }
    }

    /// Segment a path into components for trie storage
    pub fn segment_path(&self, path: &str) -> Vec<String> {
        let normalized = self.normalize_path(path);

        // Split by both forward slashes and backslashes
        let segments: Vec<String> = normalized
            .split(|c| c == '/' || c == '\\')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        // Handle drive letters for Windows paths
        if normalized.len() >= 2 && normalized.chars().nth(1) == Some(':') {
            let drive = normalized.chars().take(2).collect::<String>();
            let mut result = Vec::with_capacity(segments.len() + 1);
            result.push(drive.clone());
            result.extend(segments.into_iter().filter(|s| s != &drive));
            return result;
        }

        // Handle root directory case
        if normalized.starts_with('/') {
            let mut result = Vec::with_capacity(segments.len() + 1);
            result.push("/".to_string());
            result.extend(segments);
            return result;
        }

        segments
    }

    /// Insert a path into the trie
    pub fn insert(&self, path_str: &str, path: PathBuf) -> Result<(), String> {
        let segments = self.segment_path(path_str);

        if segments.is_empty() {
            return Err("Cannot insert empty path".to_string());
        }

        if let Ok(mut root) = self.root.write() {
            self.insert_segments(&mut root, &segments, 0, path.clone(), None)?;

            // Increment path count
            if let Ok(mut count) = self.path_count.write() {
                *count += 1;
            } else {
                return Err("Failed to acquire write lock on path count".to_string());
            }

            Ok(())
        } else {
            Err("Failed to acquire write lock on root node".to_string())
        }
    }

    /// Recursively insert path segments
    fn insert_segments(
        &self,
        node: &mut AdaptiveRadixNode,
        segments: &[String],
        index: usize,
        path: PathBuf,
        parent: Option<Arc<RwLock<AdaptiveRadixNode>>>,
    ) -> Result<(), String> {
        if index >= segments.len() {
            // We've reached the end of the path, mark as terminal node
            node.path = Some(path);
            node.record_access();
            return Ok(());
        }

        let segment = &segments[index];
        let normalized_segment = node.normalized_segment(segment);

        // Find or create child node for this segment
        if !node.children.contains_key(&normalized_segment) {
            let new_node = AdaptiveRadixNode::new(
                segment.clone(),
                self.default_case_sensitive,
                parent.clone(),
                index + 1,
            );

            node.children
                .insert(normalized_segment.clone(), Arc::new(RwLock::new(new_node)));
        }

        // Continue insertion with next segment
        if let Some(child_arc) = node.children.get(&normalized_segment) {
            let parent_arc = Arc::clone(child_arc);

            if let Ok(mut child) = child_arc.write() {
                self.insert_segments(&mut child, segments, index + 1, path, Some(parent_arc))?;
            } else {
                return Err("Failed to acquire write lock on child node".to_string());
            }
        }

        Ok(())
    }

    /// Find paths with the given prefix
    /// Note: This method only finds paths that START WITH the given prefix.
    /// For substring matching anywhere in the path, use search_recursive() instead.
    pub fn find_with_prefix(&self, prefix: &str) -> Vec<PathBuf> {
        let segments = self.segment_path(prefix);
        let mut results = Vec::new();

        if segments.is_empty() {
            return results;
        }

        if let Ok(root) = self.root.read() {
            if let Err(_) = self.find_with_segments(&root, &segments, 0, &mut results) {
                return Vec::new();
            }
        } else {
            return Vec::new();
        }

        results
    }

    /// Recursively find paths with given segments
    fn find_with_segments(
        &self,
        node: &AdaptiveRadixNode,
        segments: &[String],
        index: usize,
        results: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        // If we've consumed all segments, this is a prefix match
        if index >= segments.len() {
            // Add this node's path if it exists
            if let Some(path) = &node.path {
                results.push(path.clone());
            }

            // Recursively add all children's paths
            for (_, child_arc) in &node.children {
                if let Ok(child) = child_arc.read() {
                    self.collect_all_paths(&child, results)?;
                } else {
                    return Err("Failed to acquire read lock on child node".to_string());
                }
            }

            return Ok(());
        }

        let segment = &segments[index];
        let is_last_segment = index == segments.len() - 1;

        // Handle all children that might match
        for (normalized_key, child_arc) in &node.children {
            let key_matches = if node.case_sensitive {
                if is_last_segment {
                    // For last segment, do prefix matching
                    normalized_key.starts_with(segment)
                } else {
                    // For middle segments, do exact matching
                    normalized_key == segment
                }
            } else {
                if is_last_segment {
                    // For last segment, do case-insensitive prefix matching
                    normalized_key
                        .to_lowercase()
                        .starts_with(&segment.to_lowercase())
                } else {
                    // For middle segments, do case-insensitive exact matching
                    normalized_key.to_lowercase() == segment.to_lowercase()
                }
            };

            if key_matches {
                if let Ok(child) = child_arc.read() {
                    self.find_with_segments(&child, segments, index + 1, results)?;
                } else {
                    return Err("Failed to acquire read lock on child node".to_string());
                }
            }
        }

        Ok(())
    }

    /// Find paths where any segment contains the given prefix
    /// This is more flexible than find_with_prefix() as it looks for the term
    /// as a substring within path segments rather than requiring exact prefix matches.
    pub fn find_with_flexible_prefix(&self, prefix: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        if let Ok(root) = self.root.read() {
            // Start the recursive search from the root
            let _ = self.find_with_flexible_prefix_recursive(
                &root,
                prefix,
                &mut results,
                &mut seen_paths,
            );
        }

        results
    }

    /// Recursively find paths containing the prefix in any segment
    fn find_with_flexible_prefix_recursive(
        &self,
        node: &AdaptiveRadixNode,
        prefix: &str,
        results: &mut Vec<PathBuf>,
        seen_paths: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<(), String> {
        // Check if the current node's segment contains the prefix
        let segment_matches = if node.case_sensitive {
            node.segment.contains(prefix)
        } else {
            node.segment.to_lowercase().contains(&prefix.to_lowercase())
        };

        // If this node matches and has a path, add it
        if segment_matches && node.path.is_some() {
            if let Some(path) = &node.path {
                if !seen_paths.contains(path) {
                    seen_paths.insert(path.clone());
                    results.push(path.clone());
                }
            }
        }

        // If this is a terminal node and the full path contains the prefix
        if let Some(path) = &node.path {
            let path_str = path.to_string_lossy();
            let path_contains = if node.case_sensitive {
                path_str.contains(prefix)
            } else {
                path_str.to_lowercase().contains(&prefix.to_lowercase())
            };

            if path_contains && !seen_paths.contains(path) {
                seen_paths.insert(path.clone());
                results.push(path.clone());
            }
        }

        // Continue searching in all children
        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                self.find_with_flexible_prefix_recursive(&child, prefix, results, seen_paths)?;
            } else {
                return Err("Failed to acquire read lock on child node".to_string());
            }
        }

        Ok(())
    }

    /// Search recursively through all paths for a substring
    /// This method finds ANY path containing the search term anywhere (most similar to
    /// standard file search implementations) and is recommended for general-purpose searches.
    pub fn search_recursive(&self, search_term: &str) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        if let Ok(root) = self.root.read() {
            // Start the recursive search from the root
            let _ = self.search_recursive_impl(&root, search_term, &mut results, &mut seen_paths);
        }

        results
    }

    /// Implementation of recursive search
    fn search_recursive_impl(
        &self,
        node: &AdaptiveRadixNode,
        search_term: &str,
        results: &mut Vec<PathBuf>,
        seen_paths: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<(), String> {
        // Check if the current node's segment contains the search term
        let segment_matches = if node.case_sensitive {
            node.segment.contains(search_term)
        } else {
            node.segment
                .to_lowercase()
                .contains(&search_term.to_lowercase())
        };

        // If this node has a path and it matches or hasn't been checked yet
        if let Some(path) = &node.path {
            let path_str = path.to_string_lossy();
            let path_contains = if node.case_sensitive {
                path_str.contains(search_term)
            } else {
                path_str
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
            };

            if segment_matches || path_contains {
                if !seen_paths.contains(path) {
                    seen_paths.insert(path.clone());
                    results.push(path.clone());
                }
            }
        }

        // Continue searching in all children
        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                self.search_recursive_impl(&child, search_term, results, seen_paths)?;
            } else {
                return Err("Failed to acquire read lock on child node".to_string());
            }
        }

        Ok(())
    }

    /// Search for paths containing all specified components
    pub fn search_by_components(&self, components: &[&str]) -> Vec<PathBuf> {
        if components.is_empty() {
            return Vec::new();
        }

        // Convert components to String for consistent processing
        let components: Vec<String> = components.iter().map(|&s| s.to_string()).collect();

        let mut results = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        if let Ok(root) = self.root.read() {
            // Start with an empty vector of matched components
            let _ = self.search_components_recursive(
                &root,
                &components,
                &Vec::new(),
                &mut results,
                &mut seen_paths,
            );
        }

        results
    }

    /// Recursively search for components
    fn search_components_recursive(
        &self,
        node: &AdaptiveRadixNode,
        components: &[String],
        matched_components: &Vec<String>,
        results: &mut Vec<PathBuf>,
        seen_paths: &mut std::collections::HashSet<PathBuf>,
    ) -> Result<(), String> {
        // If we've matched all components and this is a terminal node, add the path
        if matched_components.len() >= components.len() && node.path.is_some() {
            if let Some(path) = &node.path {
                if !seen_paths.contains(path) {
                    seen_paths.insert(path.clone());
                    results.push(path.clone());
                }
            }
            return Ok(());
        }

        // Check if the current segment matches any remaining component
        let mut new_matched = matched_components.clone();
        let current_segment = &node.segment;

        if !current_segment.is_empty() {
            for (_i, component) in components.iter().enumerate() {
                // Skip components we've already matched
                if matched_components.iter().any(|m| m == component) {
                    continue;
                }

                let matches = if node.case_sensitive {
                    current_segment.contains(component)
                } else {
                    current_segment
                        .to_lowercase()
                        .contains(&component.to_lowercase())
                };

                if matches {
                    new_matched.push(component.clone());
                    break; // Only match one component per segment
                }
            }
        }

        // Check if we matched all components with the current path
        if new_matched.len() >= components.len() && node.path.is_some() {
            if let Some(path) = &node.path {
                if !seen_paths.contains(path) {
                    seen_paths.insert(path.clone());
                    results.push(path.clone());
                }
            }
        }

        // Continue search in all children
        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                // Try with the new matched components
                self.search_components_recursive(
                    &child,
                    components,
                    &new_matched,
                    results,
                    seen_paths,
                )?;
            } else {
                return Err("Failed to acquire read lock on child node".to_string());
            }
        }

        Ok(())
    }

    /// Helper to collect all paths from a node and its descendants
    fn collect_all_paths(
        &self,
        node: &AdaptiveRadixNode,
        results: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        // First add the current node's path if it exists
        if let Some(path) = &node.path {
            results.push(path.clone());
        }

        // Then recursively collect from all children
        for (_, child_arc) in &node.children {
            if let Ok(child) = child_arc.read() {
                self.collect_all_paths(&child, results)?;
            } else {
                return Err("Failed to acquire read lock on child node".to_string());
            }
        }

        Ok(())
    }

    /// Find a specific exact path and return it if it exists
    pub fn find_exact_path(&self, path_str: &str) -> Option<PathBuf> {
        let segments = self.segment_path(path_str);

        if segments.is_empty() {
            return None;
        }

        if let Ok(root) = self.root.read() {
            if let Ok(result) = self.find_exact_path_impl(&root, &segments, 0) {
                return result;
            }
        }

        None
    }

    /// Implementation to find an exact path
    fn find_exact_path_impl(
        &self,
        node: &AdaptiveRadixNode,
        segments: &[String],
        index: usize,
    ) -> Result<Option<PathBuf>, String> {
        // If we've reached the end of the segments, check if this is a terminal node
        if index >= segments.len() {
            return Ok(node.path.clone());
        }

        let segment = &segments[index];
        let normalized_segment = node.normalized_segment(segment);

        // Check if this segment exists
        if let Some(child_arc) = node.children.get(&normalized_segment) {
            if let Ok(child) = child_arc.read() {
                return self.find_exact_path_impl(&child, segments, index + 1);
            } else {
                return Err("Failed to acquire read lock on child node".to_string());
            }
        }

        // No matching segment found
        Ok(None)
    }

    /// Remove a path from the trie - improved version to ensure complete removal
    pub fn remove(&self, path_str: &str) -> Result<bool, String> {
        let segments = self.segment_path(path_str);

        if segments.is_empty() {
            return Ok(false);
        }

        let removed;

        if let Ok(mut root) = self.root.write() {
            removed = self.remove_path(&mut root, &segments, 0)?;

            // Decrement path count if a path was removed
            if removed {
                if let Ok(mut count) = self.path_count.write() {
                    if *count > 0 {
                        *count -= 1;
                    }
                } else {
                    return Err("Failed to acquire write lock on path count".to_string());
                }

                // Double-check the path is gone by attempting to find it
                if self.find_exact_path(path_str).is_some() {
                    return Err(format!(
                        "Path removal verification failed for: {}",
                        path_str
                    ));
                }
            }
        } else {
            return Err("Failed to acquire write lock on root node".to_string());
        }

        Ok(removed)
    }

    /// Recursively remove path segments
    fn remove_path(
        &self,
        node: &mut AdaptiveRadixNode,
        segments: &[String],
        index: usize,
    ) -> Result<bool, String> {
        if index >= segments.len() {
            // Path found, remove the value
            if node.path.is_some() {
                node.path = None;
                return Ok(true);
            }
            return Ok(false);
        }

        let segment = &segments[index];

        // Find all matching children (considering case sensitivity)
        let mut removed = false;
        let mut children_to_check = Vec::new();

        for key in node.children.keys() {
            if (node.case_sensitive && key == segment)
                || (!node.case_sensitive && key.to_lowercase() == segment.to_lowercase())
            {
                children_to_check.push(key.clone());
            }
        }

        for key in children_to_check {
            if let Some(child_arc) = node.children.get(&key).cloned() {
                if let Ok(mut child) = child_arc.write() {
                    let result = self.remove_path(&mut child, segments, index + 1)?;

                    if result {
                        // Mark as removed
                        removed = true;

                        // If the child has no path and no children, remove it
                        if child.path.is_none() && child.children.is_empty() {
                            node.children.remove(&key);
                        }
                    }
                } else {
                    return Err("Failed to acquire write lock on child node".to_string());
                }
            }
        }

        Ok(removed)
    }

    /// Get the total number of indexed paths
    pub fn path_count(&self) -> usize {
        if let Ok(count) = self.path_count.read() {
            *count
        } else {
            0
        }
    }

    /// Alias for path_count for backward compatibility
    pub fn get_path_count(&self) -> usize {
        self.path_count()
    }
}

#[cfg(test)]
mod tests_art {
    use super::*;
    use crate::search_engine::generate_test_data;
    use crate::log_info;
    use std::fs::read_dir;
    use std::path::Path;
    use std::time::{Duration, Instant};

    // Helper function to get the test data path, creating it if needed
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-art");
        if !path.exists() {
            log_info!("Test data doesn't exist, generating it now");
            generate_test_data(path.clone()).expect("Failed to generate test data");
            log_info!("Test data generation complete");
        }
        path
    }

    // Helper function to index all files in a directory for testing
    fn index_directory_in_trie(trie: &AdaptiveRadixTrie, dir_path: &Path) -> usize {
        let mut count = 0;
        if let Ok(entries) = read_dir(dir_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();

                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        trie.insert(&path_str, path.clone()).unwrap();
                        count += 1;
                    } else if metadata.is_dir() {
                        count += index_directory_in_trie(trie, &path);
                    }
                }
            }
        }
        count
    }

    #[test]
    fn test_creation() {
        let trie = AdaptiveRadixTrie::new();
        assert_eq!(trie.path_count(), 0);
        assert!(!trie.default_case_sensitive);

        let case_sensitive_trie = AdaptiveRadixTrie::new_with_arg(true);
        assert!(case_sensitive_trie.default_case_sensitive);
    }

    #[test]
    fn test_insert_and_count() {
        let trie = AdaptiveRadixTrie::new();

        // Insert some paths
        trie.insert(
            "/home/user/documents/file1.txt",
            PathBuf::from("/home/user/documents/file1.txt"),
        )
        .unwrap();
        trie.insert(
            "/home/user/documents/file2.txt",
            PathBuf::from("/home/user/documents/file2.txt"),
        )
        .unwrap();
        trie.insert(
            "/home/user/pictures/img1.jpg",
            PathBuf::from("/home/user/pictures/img1.jpg"),
        )
        .unwrap();

        assert_eq!(trie.path_count(), 3);
    }

    #[test]
    fn test_normalize_and_segment_paths() {
        let trie = AdaptiveRadixTrie::new();

        // Test normalization (backslash to forward slash)
        let normalized = trie.normalize_path(r"C:\Users\Documents\file.txt");
        assert_eq!(normalized, "c:/users/documents/file.txt");

        // Test segmentation
        let segments = trie.segment_path("/home/user/file.txt");
        assert_eq!(segments, vec!["/", "home", "user", "file.txt"]);

        let windows_segments = trie.segment_path(r"C:\Users\Documents\file.txt");
        assert_eq!(
            windows_segments,
            vec!["c:", "users", "documents", "file.txt"]
        );
    }

    #[test]
    fn test_find_with_prefix() {
        let trie = AdaptiveRadixTrie::new();

        // Insert some paths
        trie.insert(
            "/home/user/documents/report.pdf",
            PathBuf::from("/home/user/documents/report.pdf"),
        )
        .unwrap();
        trie.insert(
            "/home/user/documents/notes.txt",
            PathBuf::from("/home/user/documents/notes.txt"),
        )
        .unwrap();
        trie.insert(
            "/home/user/pictures/vacation.jpg",
            PathBuf::from("/home/user/pictures/vacation.jpg"),
        )
        .unwrap();
        trie.insert(
            "/home/user/pictures/family.jpg",
            PathBuf::from("/home/user/pictures/family.jpg"),
        )
        .unwrap();

        // Test exact prefix
        let docs = trie.find_with_prefix("/home/user/documents");
        assert_eq!(docs.len(), 2);
        assert!(docs.contains(&PathBuf::from("/home/user/documents/report.pdf")));
        assert!(docs.contains(&PathBuf::from("/home/user/documents/notes.txt")));

        // Test broader prefix
        let all = trie.find_with_prefix("/home/user");
        assert_eq!(all.len(), 4);

        // Test specific file prefix
        let report = trie.find_with_prefix("/home/user/documents/report");
        assert_eq!(report.len(), 1);
        assert_eq!(report[0], PathBuf::from("/home/user/documents/report.pdf"));
    }

    #[test]
    fn test_find_with_flexible_prefix() {
        let trie = AdaptiveRadixTrie::new();

        // Insert some paths with different styles
        trie.insert(
            "/home/user/documents/report.pdf",
            PathBuf::from("/home/user/documents/report.pdf"),
        )
        .unwrap();
        trie.insert(
            "C:/Users/Documents/notes.txt",
            PathBuf::from("C:/Users/Documents/notes.txt"),
        )
        .unwrap();
        trie.insert(
            r"D:\Projects\code.rs",
            PathBuf::from(r"D:\Projects\code.rs"),
        )
        .unwrap();

        // Test flexible prefix matching
        let docs1 = trie.find_with_flexible_prefix("documents");
        assert!(docs1.contains(&PathBuf::from("/home/user/documents/report.pdf")));

        let docs2 = trie.find_with_flexible_prefix("Documents");
        assert!(docs2.contains(&PathBuf::from("C:/Users/Documents/notes.txt")));

        // Test with partial path
        let projects = trie.find_with_flexible_prefix("Projects");
        assert!(projects.contains(&PathBuf::from(r"D:\Projects\code.rs")));
    }

    #[test]
    fn test_search_recursive() {
        let trie = AdaptiveRadixTrie::new();

        // Insert paths with common patterns
        trie.insert(
            "/home/user/work/project1/source.rs",
            PathBuf::from("/home/user/work/project1/source.rs"),
        )
        .unwrap();
        trie.insert(
            "/home/user/work/project2/source.rs",
            PathBuf::from("/home/user/work/project2/source.rs"),
        )
        .unwrap();
        trie.insert(
            "/home/user/personal/notes.txt",
            PathBuf::from("/home/user/personal/notes.txt"),
        )
        .unwrap();
        trie.insert(
            "/opt/data/backup/config.bak",
            PathBuf::from("/opt/data/backup/config.bak"),
        )
        .unwrap();

        // Test recursive search for pattern
        let source_files = trie.search_recursive("source");
        assert_eq!(source_files.len(), 2);

        let rs_files = trie.search_recursive(".rs");
        assert_eq!(rs_files.len(), 2);

        let project_files = trie.search_recursive("project");
        assert_eq!(project_files.len(), 2);

        // Test that it finds substrings within segments
        let backup_files = trie.search_recursive("back");
        assert_eq!(backup_files.len(), 1);
        assert!(backup_files.contains(&PathBuf::from("/opt/data/backup/config.bak")));
    }

    #[test]
    fn test_search_by_components() {
        let trie = AdaptiveRadixTrie::new();

        // Insert paths
        trie.insert(
            "/usr/local/bin/program",
            PathBuf::from("/usr/local/bin/program"),
        )
        .unwrap();
        trie.insert(
            "/usr/share/doc/manual.pdf",
            PathBuf::from("/usr/share/doc/manual.pdf"),
        )
        .unwrap();
        trie.insert(
            "/home/user/Downloads/app.dmg",
            PathBuf::from("/home/user/Downloads/app.dmg"),
        )
        .unwrap();

        // Test component search
        let results = trie.search_by_components(&["usr", "bin"]);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&PathBuf::from("/usr/local/bin/program")));

        // Test component search with non-adjacent terms
        let results = trie.search_by_components(&["usr", "manual"]);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&PathBuf::from("/usr/share/doc/manual.pdf")));
    }

    #[test]
    fn test_case_sensitivity() {
        // Default case insensitive
        let case_insensitive = AdaptiveRadixTrie::new();
        case_insensitive
            .insert(
                "/Home/User/Documents/Report.pdf",
                PathBuf::from("/Home/User/Documents/Report.pdf"),
            )
            .unwrap();

        // Find with different case
        let results1 = case_insensitive.find_with_prefix("/home/user");
        assert_eq!(results1.len(), 1);

        let results2 = case_insensitive.search_recursive("report");
        assert_eq!(results2.len(), 1);

        // Case sensitive
        let case_sensitive = AdaptiveRadixTrie::new_with_arg(true);
        case_sensitive
            .insert(
                "/Home/User/Documents/Report.pdf",
                PathBuf::from("/Home/User/Documents/Report.pdf"),
            )
            .unwrap();

        // Should not find with different case
        let results3 = case_sensitive.find_with_prefix("/home/user");
        assert_eq!(results3.len(), 0);

        // Should find with exact case
        let results4 = case_sensitive.find_with_prefix("/Home/User");
        assert_eq!(results4.len(), 1);
    }

    #[test]
    fn test_remove() {
        let trie = AdaptiveRadixTrie::new();

        // Insert some paths
        trie.insert("/tmp/file1.txt", PathBuf::from("/tmp/file1.txt"))
            .unwrap();
        trie.insert("/tmp/file2.txt", PathBuf::from("/tmp/file2.txt"))
            .unwrap();
        trie.insert(
            "/tmp/subdir/file3.txt",
            PathBuf::from("/tmp/subdir/file3.txt"),
        )
        .unwrap();

        assert_eq!(trie.path_count(), 3);

        // Remove a path
        let removed = trie.remove("/tmp/file1.txt").unwrap();
        assert!(removed);
        assert_eq!(trie.path_count(), 2);

        // Verify it's gone
        let results = trie.find_with_prefix("/tmp/file1");
        assert_eq!(results.len(), 0);

        // Other files still exist
        let remaining = trie.find_with_prefix("/tmp");
        assert_eq!(remaining.len(), 2);

        // Remove non-existent path
        let removed = trie.remove("/nonexistent/path").unwrap();
        assert!(!removed);
        assert_eq!(trie.path_count(), 2);
    }

    #[test]
    fn test_windows_paths() {
        let trie = AdaptiveRadixTrie::new();

        // Insert Windows paths
        trie.insert(
            r"C:\Program Files\App\program.exe",
            PathBuf::from(r"C:\Program Files\App\program.exe"),
        )
        .unwrap();
        trie.insert(
            r"C:\Users\User\Documents\file.docx",
            PathBuf::from(r"C:\Users\User\Documents\file.docx"),
        )
        .unwrap();

        // Find with Windows path syntax
        let results1 = trie.find_with_prefix(r"C:\Program Files");
        assert_eq!(results1.len(), 1);

        // Find with forward slashes
        let results2 = trie.find_with_prefix("C:/Users");
        assert_eq!(results2.len(), 1);

        // Search by component
        let results3 = trie.search_by_components(&["Program Files", "program.exe"]);
        assert_eq!(results3.len(), 1);
    }

    #[test]
    fn test_comprehensive() {
        let trie = AdaptiveRadixTrie::new();

        // Insert a variety of paths
        let paths = [
            "/usr/bin/gcc",
            "/usr/local/bin/python",
            "/home/user/Documents/report.docx",
            "/home/user/Documents/presentation.pptx",
            "/home/user/Pictures/vacation/beach.jpg",
            "/home/user/Pictures/vacation/sunset.jpg",
            "/home/user/Pictures/family.jpg",
            "/var/log/syslog",
            "/etc/config/settings.conf",
            r"C:\Windows\System32\cmd.exe",
        ];

        for path in &paths {
            trie.insert(path, PathBuf::from(path)).unwrap();
        }

        assert_eq!(trie.path_count(), paths.len());

        // Test various search functions
        let bin_files = trie.find_with_prefix("/usr");
        assert_eq!(bin_files.len(), 2);

        let doc_files = trie.find_with_prefix("/home/user/Documents");
        assert_eq!(doc_files.len(), 2);

        let pic_files = trie.search_recursive("Pictures");
        assert_eq!(pic_files.len(), 3);

        let vacation_files = trie.find_with_flexible_prefix("vacation");
        assert_eq!(vacation_files.len(), 2);

        // Remove some paths
        trie.remove("/home/user/Pictures/vacation/beach.jpg")
            .unwrap();
        assert_eq!(trie.path_count(), paths.len() - 1);

        // Verify removed path is gone but others remain
        let remaining_vacation = trie.find_with_prefix("/home/user/Pictures/vacation");
        assert_eq!(remaining_vacation.len(), 1);
        assert!(
            remaining_vacation.contains(&PathBuf::from("/home/user/Pictures/vacation/sunset.jpg"))
        );
    }

    #[test]
    fn test_performance_insertion() {
        let trie = AdaptiveRadixTrie::new();
        let start_time = Instant::now();
        let count = 1000;

        for i in 0..count {
            let path = format!("/test/path/to/file_{}.txt", i);
            trie.insert(&path, PathBuf::from(&path)).unwrap();
        }

        let duration = start_time.elapsed();
        log_info!(&format!(
            "Performance: Inserted {} paths in {:?} ({:?} per insertion)",
            count,
            duration,
            duration / count as u32
        ));

        assert_eq!(trie.path_count(), count);
    }

    #[test]
    fn test_performance_search() {
        let trie = AdaptiveRadixTrie::new();
        let count = 1000;

        // Insert paths first
        for i in 0..count {
            let path = format!("/test/path/level{}/file_{}.txt", i % 10, i);
            trie.insert(&path, PathBuf::from(&path)).unwrap();
        }

        // Test prefix search
        let start_prefix = Instant::now();
        let prefix_results = trie.find_with_prefix("/test/path/level5");
        let prefix_duration = start_prefix.elapsed();
        log_info!(&format!(
            "Performance: Prefix search found {} paths in {:?}",
            prefix_results.len(),
            prefix_duration
        ));

        // Test recursive search
        let start_recursive = Instant::now();
        let recursive_results = trie.search_recursive("file_50");
        let recursive_duration = start_recursive.elapsed();
        log_info!(&format!(
            "Performance: Recursive search found {} paths in {:?}",
            recursive_results.len(),
            recursive_duration
        ));

        // Test flexible search
        let start_flexible = Instant::now();
        let flexible_results = trie.find_with_flexible_prefix("level3");
        let flexible_duration = start_flexible.elapsed();
        log_info!(&format!(
            "Performance: Flexible search found {} paths in {:?}",
            flexible_results.len(),
            flexible_duration
        ));

        // Test component search
        let start_component = Instant::now();
        let component_results = trie.search_by_components(&["level2", "file"]);
        let component_duration = start_component.elapsed();
        log_info!(&format!(
            "Performance: Component search found {} paths in {:?}",
            component_results.len(),
            component_duration
        ));
    }

    #[test]
    fn test_performance_large_dataset() {
        let trie = AdaptiveRadixTrie::new();
        let count = 10000;

        // Create a larger dataset with varied paths
        log_info!("Building large test dataset...");
        let insert_start = Instant::now();

        for i in 0..count {
            let folder = match i % 5 {
                0 => "documents",
                1 => "pictures",
                2 => "videos",
                3 => "music",
                _ => "projects",
            };

            let depth = i % 4 + 1;
            let mut path = format!("/home/user/{}", folder);

            for d in 0..depth {
                path = format!("{}/subfolder_{}", path, d);
            }

            let extension = match i % 8 {
                0 => "txt",
                1 => "doc",
                2 => "jpg",
                3 => "png",
                4 => "mp4",
                5 => "mp3",
                6 => "rs",
                _ => "json",
            };

            path = format!("{}/file_{}_{}.{}", path, folder, i, extension);
            trie.insert(&path, PathBuf::from(&path)).unwrap();
        }

        let insert_duration = insert_start.elapsed();
        log_info!(&format!(
            "Performance: Inserted {} varied paths in {:?} ({:?} per insertion)",
            count,
            insert_duration,
            insert_duration / count as u32
        ));

        // Measure search performance with different patterns
        let search_terms = [
            "documents",
            "projects/subfolder",
            "file_pictures",
            ".json",
            "music/subfolder_2",
            "videos/subfolder_3/file",
        ];

        for term in &search_terms {
            let search_start = Instant::now();
            let results = trie.search_recursive(term);
            let search_duration = search_start.elapsed();
            log_info!(&format!(
                "Performance: Searching for '{}' found {} results in {:?}",
                term,
                results.len(),
                search_duration
            ));
        }
    }

    #[test]
    fn test_with_generated_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the generated test data
        let start_time = Instant::now();
        let indexed_count = index_directory_in_trie(&trie, &test_path);
        let index_duration = start_time.elapsed();

        log_info!(&format!(
            "Indexed {} entries from generated test data in {:?}",
            indexed_count, index_duration
        ));
        assert!(
            indexed_count > 0,
            "Should have indexed some entries from test data"
        );
        assert_eq!(trie.path_count(), indexed_count);

        // Test search on the generated data
        let search_terms = ["banana", "apple", "txt", "jpg"];

        for term in &search_terms {
            let search_start = Instant::now();
            let results = trie.search_recursive(term);
            let search_duration = search_start.elapsed();

            log_info!(&format!(
                "Searching for '{}' in generated data found {} results in {:?}",
                term,
                results.len(),
                search_duration
            ));
        }
    }

    #[test]
    fn test_prefix_search_with_generated_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the generated test data
        let indexed_count = index_directory_in_trie(&trie, &test_path);
        assert!(
            indexed_count > 0,
            "Should have indexed some entries from test data"
        );

        // Get some directories from the test data to use as prefixes
        let mut test_prefixes = Vec::new();
        if let Ok(entries) = read_dir(&test_path) {
            for entry in entries.filter_map(Result::ok).take(3) {
                if entry.path().is_dir() {
                    test_prefixes.push(entry.path().to_string_lossy().to_string());
                }
            }
        }

        // If we found directories, test prefix search with them
        if !test_prefixes.is_empty() {
            for prefix in &test_prefixes {
                let start_time = Instant::now();
                let results = trie.find_with_prefix(prefix);
                let duration = start_time.elapsed();

                log_info!(&format!(
                    "Prefix search for '{}' found {} results in {:?}",
                    prefix,
                    results.len(),
                    duration
                ));
            }
        } else {
            // Fallback if no directories were found
            log_info!("No directories found in test data for prefix search test");
        }
    }

    #[test]
    fn test_component_search_with_generated_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the generated test data
        let indexed_count = index_directory_in_trie(&trie, &test_path);
        assert!(
            indexed_count > 0,
            "Should have indexed some entries from test data"
        );

        // Test component search with common components from the generated data
        let components = [
            &["banana", "apple"],
            &["orange", "grape"],
            &["car", "truck"],
            &["txt", "pdf"],
            &["json", "png"],
        ];

        for comp_set in &components {
            let start_time = Instant::now();
            let results = trie.search_by_components(*comp_set);
            let duration = start_time.elapsed();

            log_info!(&format!(
                "Component search for {:?} found {} results in {:?}",
                comp_set,
                results.len(),
                duration
            ));
        }
    }

    #[test]
    fn test_performance_comparison_with_generated_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the generated test data
        log_info!("Indexing generated test data...");
        let start_time = Instant::now();
        let indexed_count = index_directory_in_trie(&trie, &test_path);
        let index_duration = start_time.elapsed();

        log_info!(&format!(
            "Indexed {} entries in {:?} ({:?} per entry)",
            indexed_count,
            index_duration,
            if indexed_count > 0 {
                index_duration / indexed_count as u32
            } else {
                Duration::from_secs(0)
            }
        ));

        // Test different search methods and compare performance
        let search_terms = ["txt", "json", "banana", "car"];

        log_info!(&format!(
            "\n{:<10} | {:<15} | {:<15} | {:<15}",
            "Term", "Recursive Time", "Prefix Time", "Component Time"
        ));
        log_info!(&format!("{:-<60}", ""));

        for term in &search_terms {
            // Test recursive search
            let recursive_start = Instant::now();
            let recursive_results = trie.search_recursive(term);
            let recursive_duration = recursive_start.elapsed();

            // Test prefix search
            let prefix_start = Instant::now();
            let prefix_results = trie.find_with_prefix(term);
            let prefix_duration = prefix_start.elapsed();

            // Test component search
            let component_start = Instant::now();
            let component_results = trie.search_by_components(&[term]);
            let component_duration = component_start.elapsed();

            log_info!(&format!(
                "{:<10} | {:>15?} | {:>15?} | {:>15?}",
                term, recursive_duration, prefix_duration, component_duration
            ));

            log_info!(&format!(
                "  Found: {} (recursive) | {} (prefix) | {} (component)",
                recursive_results.len(),
                prefix_results.len(),
                component_results.len()
            ));
        }
    }

    #[test]
    fn test_remove_with_generated_data() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index the generated test data
        let indexed_count = index_directory_in_trie(&trie, &test_path);
        assert!(
            indexed_count > 0,
            "Should have indexed some entries from test data"
        );

        // Find some paths to remove
        let search_results = trie.search_recursive("txt");

        if !search_results.is_empty() {
            // Take 5 paths to remove (or fewer if we don't have 5)
            let paths_to_remove: Vec<PathBuf> = search_results.into_iter().take(5).collect();
            let initial_count = trie.path_count();

            log_info!(&format!("Starting with {} indexed paths", initial_count));

            // Remove each path and verify the count decreases
            for path in &paths_to_remove {
                let path_str = path.to_string_lossy().to_string();

                let removed = trie.remove(&path_str).unwrap();
                assert!(removed, "Path should have been successfully removed");

                // Verify the path is gone
                let search_after = trie.find_with_prefix(&path_str);
                assert!(
                    search_after.is_empty(),
                    "Path should no longer be found after removal"
                );
            }

            let final_count = trie.path_count();
            assert_eq!(
                final_count,
                initial_count - paths_to_remove.len(),
                "Path count should decrease by the number of paths removed"
            );

            log_info!(&format!(
                "Successfully removed {} paths, count reduced from {} to {}",
                paths_to_remove.len(),
                initial_count,
                final_count
            ));
        } else {
            log_info!("No .txt files found in test data to test removal");
        }
    }

    #[test]
    fn test_prefix_search_basic() {
        let trie = AdaptiveRadixTrie::new();

        // Insert paths with clear prefix relationships
        let base_paths = [
            "/test/documents/file1.txt",
            "/test/documents/file2.txt",
            "/test/documents/subfolder/file3.txt",
            "/test/images/photo1.jpg",
            "/test/images/photo2.jpg",
        ];

        for path in &base_paths {
            trie.insert(path, PathBuf::from(path)).unwrap();
        }

        // Test specific prefixes that should definitely have results
        let test_cases = [
            ("/test/documents", 3),
            ("/test/documents/", 3),
            ("/test/documents/file", 2),
            ("/test/documents/subfolder", 1),
            ("/test/images", 2),
            ("/test", 5),
        ];

        for (prefix, expected_count) in &test_cases {
            let results = trie.find_with_prefix(prefix);
            assert_eq!(
                results.len(),
                *expected_count,
                "Prefix '{}' should return {} results, got {}",
                prefix,
                expected_count,
                results.len()
            );

            // Verify the paths actually start with this prefix
            for path in &results {
                let path_str = path.to_string_lossy();
                let normalized_prefix = trie.normalize_path(prefix);
                let normalized_path = trie.normalize_path(&path_str);
                assert!(
                    normalized_path.starts_with(&normalized_prefix),
                    "Result '{}' should start with prefix '{}'",
                    path_str,
                    prefix
                );
            }
        }
    }

    #[test]
    fn test_prefix_search_with_real_inserted_paths() {
        let test_path = get_test_data_path();
        let trie = AdaptiveRadixTrie::new();

        // Index files and collect the actual paths we inserted
        let mut inserted_paths = Vec::new();

        if let Ok(entries) = read_dir(&test_path) {
            for entry in entries.filter_map(Result::ok).take(20) {
                let path = entry.path();
                let path_str = path.to_string_lossy().to_string();

                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        trie.insert(&path_str, path.clone()).unwrap();
                        inserted_paths.push(path_str);
                    }
                }
            }
        }

        assert!(
            !inserted_paths.is_empty(),
            "Should have inserted some test paths"
        );

        // Test prefix search with prefixes derived from actual inserted paths
        for inserted_path in &inserted_paths {
            // Create a prefix from the path by taking the parent directory
            let path = Path::new(inserted_path);
            if let Some(parent) = path.parent() {
                let prefix = parent.to_string_lossy().to_string();

                let results = trie.find_with_prefix(&prefix);

                log_info!(&format!(
                    "Prefix search for '{}' found {} results",
                    prefix,
                    results.len()
                ));

                // The prefix search should at least find the path we derived the prefix from
                assert!(
                    results.contains(&PathBuf::from(inserted_path)),
                    "Prefix '{}' should at least find path '{}'",
                    prefix,
                    inserted_path
                );
            }
        }
    }

    #[test]
    fn test_prefix_case_sensitivity() {
        // Test with case-insensitive trie (default)
        let case_insensitive = AdaptiveRadixTrie::new();
        case_insensitive
            .insert(
                "/Home/User/Documents/Report.pdf",
                PathBuf::from("/Home/User/Documents/Report.pdf"),
            )
            .unwrap();

        // Should find with different case prefixes
        let results1 = case_insensitive.find_with_prefix("/home/user");
        assert_eq!(
            results1.len(),
            1,
            "Case-insensitive trie should find path with different case prefix"
        );

        let results2 = case_insensitive.find_with_prefix("/HOME/USER");
        assert_eq!(
            results2.len(),
            1,
            "Case-insensitive trie should find path with uppercase prefix"
        );

        // Test with case-sensitive trie
        let case_sensitive = AdaptiveRadixTrie::new_with_arg(true);
        case_sensitive
            .insert(
                "/Home/User/Documents/Report.pdf",
                PathBuf::from("/Home/User/Documents/Report.pdf"),
            )
            .unwrap();

        // Should only find with exact case prefix
        let results3 = case_sensitive.find_with_prefix("/Home/User");
        assert_eq!(
            results3.len(),
            1,
            "Case-sensitive trie should find path with exact case prefix"
        );

        let results4 = case_sensitive.find_with_prefix("/home/user");
        assert_eq!(
            results4.len(),
            0,
            "Case-sensitive trie should not find path with different case prefix"
        );
    }
}
