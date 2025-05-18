use std::collections::HashMap;
use crate::log_warn;

pub struct ART {
    root: Option<Box<Node>>,
    // Number of paths stored
    path_count: usize,
    max_results: usize,
}

#[allow(dead_code)] // remove later when used
struct Node {
    /// Character represented by this node
    character: Option<char>,
    /// Score associated with this path (if terminal)
    score: Option<f32>,
    /// Children nodes mapped by their characters
    children: HashMap<char, Node>,
    /// Flag indicating if this node represents the end of a path
    is_terminal: bool,
}

#[allow(dead_code)] // remove later when used
impl ART {
    pub fn new(max_results: usize) -> Self {
        ART {
            root: None,
            path_count: 0,
            max_results,
        }
    }

    pub fn insert(&mut self, path: &str, score: f32) -> bool {
        let normalized_path = self.normalize_path(path);
        let chars: Vec<char> = normalized_path.chars().collect();

        // Create root node if it doesn't exist
        if self.root.is_none() {
            self.root = Some(Box::new(Node {
                character: None,
                score: None,
                children: HashMap::new(),
                is_terminal: false,
            }));
        }

        // Directly use the result from insert_internal
        let root = self.root.as_mut().unwrap();
        let (changed, new_path) = Self::insert_internal(&chars, 0, root, score);

        // Update path count if this is a new path
        if new_path {
            self.path_count += 1;
        }

        // Return whether the trie was modified
        changed
    }

    fn insert_internal(chars: &[char], index: usize, node: &mut Node, score: f32) -> (bool, bool) {
        // If we've reached the end of the path, mark this node as terminal
        if index == chars.len() {
            let mut changed = false;
            let mut new_path = false;

            // Check if this is a new path
            if !node.is_terminal {
                node.is_terminal = true;
                new_path = true;
                changed = true;
            }

            // Check if the score is different
            if node.score != Some(score) {
                node.score = Some(score);
                changed = true;
            }

            return (changed, new_path);
        }

        let current_char = chars[index];

        // Create a new node if the character doesn't exist
        if !node.children.contains_key(&current_char) {
            node.children.insert(current_char, Node {
                character: Some(current_char),
                score: None,
                children: HashMap::new(),
                is_terminal: false,
            });
        }

        // Continue insertion with the next character
        let next_node = node.children.get_mut(&current_char).unwrap();
        Self::insert_internal(chars, index + 1, next_node, score)
    }

    pub fn find_completions(&self, prefix: &str) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        // Handle empty trie case
        if self.root.is_none() {
            return results;
        }

        // Navigate to the node corresponding to the prefix
        let normalized_prefix = self.normalize_path(prefix);
        let root_node = self.root.as_ref().unwrap();
        let mut current_node = root_node.as_ref();
        let mut found = true;

        for ch in normalized_prefix.chars() {
            if let Some(next) = current_node.children.get(&ch) {
                current_node = next;
            } else {
                // Prefix not found in the trie
                found = false;
                break;
            }
        }

        if !found {
            return results;
        }

        // Start collecting completions from the prefix node
        self.collect_completions_with_parent_char(current_node, normalized_prefix, &mut results);

        // Sort results by score (highest first) and limit to max_results
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }

    fn collect_completions_with_parent_char(&self, node: &Node, prefix: String, results: &mut Vec<(String, f32)>) {
        // If this node represents a complete path, add it to results
        if node.is_terminal {
            if let Some(score) = node.score {
                results.push((prefix.clone(), score));
            }
        }

        // Traverse all children, adding their characters to the path
        for (ch, child) in &node.children {
            let mut new_prefix = prefix.clone();
            new_prefix.push(*ch);
            self.collect_completions_with_parent_char(child, new_prefix, results);
        }
    }

    pub fn search(&self, query: &str, current_dir: Option<&str>, allow_partial_components: bool) -> Vec<(String, f32)> {
        // If query is empty, return empty results
        if query.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // Case 1: Direct prefix search (standard behavior)
        let direct_matches = self.find_completions(query);
        results.extend(direct_matches);

        // Case 2: If current directory is provided, search within that context
        if let Some(dir) = current_dir {
            let normalized_dir = self.normalize_path(dir);
            let combined_path = if normalized_dir.ends_with('/') {
                format!("{}{}", normalized_dir, query)
            } else {
                format!("{}/{}", normalized_dir, query)
            };

            let context_matches = self.find_completions(&combined_path);
            results.extend(context_matches);
        }

        // Case 3: If partial component matching is enabled, search for components
        if allow_partial_components {
            self.find_component_matches(query, current_dir, &mut results);
        }

        // Sort by score and deduplicate (keep highest score version of duplicates)
        self.sort_and_deduplicate_results(&mut results);

        // Limit to max results
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }

    /// Finds paths where any component matches the query
    fn find_component_matches(&self, query: &str, current_dir: Option<&str>, results: &mut Vec<(String, f32)>) {
        // Skip if root is None
        if self.root.is_none() {
            return;
        }

        let normalized_query = self.normalize_path(query);

        // Don't process empty queries
        if normalized_query.is_empty() {
            return;
        }

        // Normalize current directory path if provided
        let normalized_dir = current_dir.map(|dir| self.normalize_path(dir));

        // Get all paths in the trie (only if needed - this is expensive!)
        let mut all_paths = Vec::new();
        if let Some(root) = &self.root {
            // Start collection from the root node
            self.collect_all_paths(root.as_ref(), String::new(), &mut all_paths);
        }

        // Find paths where any component contains the query
        for (path, score) in all_paths {
            // Skip paths that don't match the current directory context
            if let Some(ref dir) = normalized_dir {
                // Only consider paths that are under the current directory
                if !path.starts_with(dir) && !path.starts_with(&format!("{}/", dir)) {
                    continue;
                }
            }

            let components: Vec<&str> = path.split('/').collect();

            // Check if any component contains or starts with the query
            for component in components {
                if component.contains(&normalized_query) {
                    // Reduce score slightly for partial component matches
                    // that aren't at the start of the component
                    let adjusted_score = if component.starts_with(&normalized_query) {
                        score * 0.95 // Small penalty for component prefix match
                    } else {
                        score * 0.9  // Bigger penalty for substring match
                    };

                    results.push((path.clone(), adjusted_score));
                    break; // Only count each path once
                }
            }
        }
    }

    /// Collect all paths in the trie
    /// Takes a reference to Node (not Box<Node>)
    fn collect_all_paths(&self, node: &Node, current_path: String, results: &mut Vec<(String, f32)>) {
        // If this node is terminal, add the current path to results
        if node.is_terminal {
            if let Some(score) = node.score {
                results.push((current_path.clone(), score));
            }
        }

        // Traverse all children
        for (ch, child) in &node.children {
            let mut next_path = current_path.clone();
            next_path.push(*ch);
            self.collect_all_paths(child, next_path, results);
        }
    }

    /// Sort results by score and remove duplicates
    fn sort_and_deduplicate_results(&self, results: &mut Vec<(String, f32)>) {
        // Sort descending by score
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate keeping highest score for each path
        let mut seen_paths = std::collections::HashSet::new();
        results.retain(|(path, _)| seen_paths.insert(path.clone()));
    }

    pub fn remove(&mut self, path: &str) -> bool {
        // Handle empty trie case
        if self.root.is_none() {
            return false;
        }

        let normalized_path = self.normalize_path(path);
        let chars: Vec<char> = normalized_path.chars().collect();

        // Call the internal removal function
        let (removed, should_remove_node) = Self::remove_internal(&chars, 0, self.root.as_mut().unwrap());

        // If the root node should be removed, set root to None
        if should_remove_node {
            self.root = None;
        }

        // Update path count if a path was removed
        if removed {
            // Safety check to avoid underflow
            if self.path_count > 0 {
                self.path_count -= 1;
            } else {
                // This is an inconsistency in the trie state
                log_warn!("Removing a path when path_count is already 0");
            }
        }

        removed
    }

    fn remove_internal(chars: &[char], index: usize, node: &mut Node) -> (bool, bool) {
        // If we've reached the end of the path
        if index == chars.len() {
            // Check if this node is terminal
            if node.is_terminal {
                // Mark it as non-terminal
                node.is_terminal = false;
                node.score = None;

                // If the node has no children, it should be removed
                let should_remove = node.children.is_empty();
                return (true, should_remove);
            } else {
                // Path not found (node exists but isn't terminal)
                return (false, false);
            }
        }

        let current_char = chars[index];

        // If the character doesn't exist in children, path not found
        if !node.children.contains_key(&current_char) {
            return (false, false);
        }

        // Recursively remove from child
        let (removed, should_remove_child) = {
            let child = node.children.get_mut(&current_char).unwrap();
            Self::remove_internal(chars, index + 1, child)
        };

        // If child should be removed, remove it
        if should_remove_child {
            node.children.remove(&current_char);
        }

        // This node should be removed if:
        // 1. It's not terminal (doesn't represent a path end)
        // 2. It has no children after potential child removal
        // 3. It's not the root node (which has None character)
        let should_remove_this_node = !node.is_terminal &&
            node.children.is_empty() &&
            node.character.is_some();

        (removed, should_remove_this_node)
    }

    /// Get the number of paths in the trie
    pub fn len(&self) -> usize {
        self.path_count
    }

    /// Check if the trie is empty
    pub fn is_empty(&self) -> bool {
        self.path_count == 0
    }

    /// Clear the trie
    pub fn clear(&mut self) {
        self.root = None;
        self.path_count = 0;
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

    /// Fast check if a path exists in the trie
    pub fn contains(&self, path: &str) -> bool {
        if self.root.is_none() {
            return false;
        }

        let normalized = self.normalize_path(path);
        if normalized.is_empty() {
            return false;
        }

        let mut current = self.root.as_ref().unwrap().as_ref();

        for ch in normalized.chars() {
            match current.children.get(&ch) {
                Some(child) => current = child,
                None => return false, // Path prefix not found
            }
        }

        // We've traversed the entire path, check if it's a terminal node
        current.is_terminal
    }
}

#[cfg(test)]
mod tests_art_v3 {
    use super::*;
    use std::time::Instant;
    #[cfg(feature = "long-tests")]
    use std::time::Duration;
    use std::path::{Path, PathBuf, MAIN_SEPARATOR};
    use crate::{log_info, log_warn};

    // Helper function to get test data directory
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
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

        fn add_paths_recursively(dir: &Path, paths: &mut Vec<String>, limit: Option<usize>) {
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
            // Generate paths with the correct separator
            return (0..100).map(|i| format!("{}path{}to{}file{}.txt",
                                            MAIN_SEPARATOR, MAIN_SEPARATOR, MAIN_SEPARATOR, i)).collect();
        }

        paths
    }

    /// Normalize paths with special handling for spaces and backslashes
    fn normalize_path(path: &str) -> String {
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

    // Basic functionality tests
    #[test]
    fn test_basic_insert_and_find() {
        log_info!("Starting basic insert and find test");
        let mut trie = ART::new(10);

        // Use platform-agnostic paths by joining components
        let docs_path = std::path::Path::new("C:").join("Users").join("Documents").to_string_lossy().to_string();
        let downloads_path = std::path::Path::new("C:").join("Users").join("Downloads").to_string_lossy().to_string();
        let pictures_path = std::path::Path::new("C:").join("Users").join("Pictures").to_string_lossy().to_string();

        let docs_path = normalize_path(&docs_path);
        let downloads_path = normalize_path(&downloads_path);
        let pictures_path = normalize_path(&pictures_path);

        // Insert some paths
        assert!(trie.insert(&docs_path, 1.0));
        assert!(trie.insert(&downloads_path, 0.8));
        assert!(trie.insert(&pictures_path, 0.6));

        // Check the count
        assert_eq!(trie.len(), 3);
        log_info!(&format!("Trie contains {} paths", trie.len()));

        // Find completions
        let prefix = std::path::Path::new("C:").join("Users").to_string_lossy().to_string();
        let completions = trie.find_completions(&prefix);
        assert_eq!(completions.len(), 3);
        log_info!(&format!("Found {} completions for '{}'", completions.len(), prefix));

        // Check specific completion
        let docs = completions.iter().find(|(path, _)| path == &docs_path);
        assert!(docs.is_some());
        log_info!("Successfully found 'Documents' in completions");
    }

    #[test]
    fn test_empty_trie() {
        log_info!("Testing empty trie behavior");
        let trie = ART::new(5);

        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());

        let completions = trie.find_completions("anything");
        assert_eq!(completions.len(), 0);
        log_info!("Empty trie returns empty completions as expected");
    }

    #[test]
    fn test_complete_filenames_v3() {
        let mut trie = ART::new(10);

        // The exact paths from your example
        let paths = vec![
            "./test-data-for-fuzzy-search/airplane.mp4",
            "./test-data-for-fuzzy-search/ambulance",
            "./test-data-for-fuzzy-search/apple.pdf"
        ];

        // Insert all paths
        for path in &paths {
            trie.insert(path, 1.0);
        }

        // Search with base directory
        let results = trie.find_completions("./test-data-for-fuzzy-search");

        // Check that each path is complete with the correct filename
        assert_eq!(results.len(), 3, "Should find all 3 paths");

        // Each original path should be in the results - EXACT match
        for path in &paths {
            let found = results.iter().any(|(p, _)| p == path);
            assert!(found, "Complete path should be found: {}", path);
        }

        // Check that filenames still start with 'a'
        for (path, _) in &results {
            let last_slash = path.rfind('/').unwrap_or(0);
            let filename = &path[last_slash+1..];
            assert!(filename.starts_with('a'),
                    "Filename should start with 'a': {}", filename);
        }
    }

    #[test]
    fn debug_byte_representation() {
        log_info!("===== BYTE REPRESENTATION DEBUG TEST =====");
        let mut trie = ART::new(10);

        // Create a simple test path
        let test_path = "test_path";

        // 1. Log the bytes directly
        log_info!(&format!("Original path: '{}'", test_path));
        log_info!(&format!("Original bytes: {:?}", test_path.as_bytes()));

        // 2. Insert the path
        let success = trie.insert(test_path, 1.0);
        log_info!(&format!("Insertion success: {}", success));

        // 3. Try to find the path
        let completions = trie.find_completions(test_path);
        log_info!(&format!("Found {} completions", completions.len()));

        // 4. Directly examine normalized versions
        let normalized_for_insert = trie.normalize_path(test_path);
        log_info!(&format!("Normalized for insert: '{}'", normalized_for_insert));
        log_info!(&format!("Normalized bytes: {:?}", normalized_for_insert.as_bytes()));

        // 5. Add debug to your normalize_path method
        // Add this temporarily to your normalize_path method:
        /*
        log_info!("NORMALIZING: '{}' -> '{}'", path, normalized);
        log_info!("BYTES BEFORE: {:?}", path.as_bytes());
        log_info!("BYTES AFTER: {:?}", normalized.as_bytes());
        */

        // 6. Test with a path containing backslashes
        let backslash_path = r"dir1\file2.txt";
        log_info!(&format!("Backslash path: '{}'", backslash_path));
        log_info!(&format!("Backslash path bytes: {:?}", backslash_path.as_bytes()));

        let normalized_bs = trie.normalize_path(backslash_path);
        log_info!(&format!("Normalized backslash path: '{}'", normalized_bs));
        log_info!(&format!("Normalized backslash bytes: {:?}", normalized_bs.as_bytes()));
    }

    #[test]
    fn test_component_split() {
        let mut trie = ART::new(10);

        // The exact paths from your logs that are causing issues
        let path1 = "./test-data-for-fuzzy-search/airplane.mp4";
        let path2 = "./test-data-for-fuzzy-search/ambulance";
        let path3 = "./test-data-for-fuzzy-search/apple.pdf";

        // Insert first path
        assert!(trie.insert(path1, 1.0), "Should insert first path");

        // Verify first path was added correctly
        let results1 = trie.find_completions(path1);
        assert_eq!(results1.len(), 1, "Should find the first path");
        assert_eq!(results1[0].0, path1, "Path should match exactly");

        // Now insert second path - this triggers the split within a component
        assert!(trie.insert(path2, 0.9), "Should insert second path");

        // The critical test - verify second path was added correctly
        let results2 = trie.find_completions(path2);
        assert_eq!(results2.len(), 1, "Should find the second path");
        assert_eq!(results2[0].0, path2, "Second path should match exactly");

        // Verify first path is still findable
        let still_find1 = trie.find_completions(path1);
        assert_eq!(still_find1.len(), 1, "Should still find first path");
        assert_eq!(still_find1[0].0, path1, "First path should still match exactly");

        // Add third path
        assert!(trie.insert(path3, 0.8), "Should insert third path");

        // Verify prefix search works for all paths
        let prefix = "./test-data-for-fuzzy-search/a";
        let prefix_results = trie.find_completions(prefix);
        assert_eq!(prefix_results.len(), 3, "Should find all three paths");

        // Verify each path is in the results
        let has_path1 = prefix_results.iter().any(|(p, _)| p == path1);
        let has_path2 = prefix_results.iter().any(|(p, _)| p == path2);
        let has_path3 = prefix_results.iter().any(|(p, _)| p == path3);

        assert!(has_path1, "Prefix search should find path1");
        assert!(has_path2, "Prefix search should find path2");
        assert!(has_path3, "Prefix search should find path3");
    }

    #[test]
    fn test_multiple_files_with_similar_names() {
        let mut trie = ART::new(10);

        // Very similar filenames
        let path1 = "a/b/file1.txt";
        let path2 = "a/b/file2.txt";

        // Insert in sequence - log extensively
        log_info!("===================== INSERTING FIRST PATH =====================");
        assert!(trie.insert(path1, 1.0), "Should insert first path");

        // Verify path1 can be found
        let found1 = trie.find_completions(path1);
        assert_eq!(found1.len(), 1, "Should find path1 after first insertion");
        assert_eq!(found1[0].0, path1, "Should match exact path");

        log_info!("===================== INSERTING SECOND PATH =====================");
        assert!(trie.insert(path2, 0.9), "Should insert second path");

        // Now verify BOTH paths can be found
        let found1_again = trie.find_completions(path1);
        assert_eq!(found1_again.len(), 1, "Should still find path1 after second insertion");
        assert_eq!(found1_again[0].0, path1, "Should still match exact path1");

        let found2 = trie.find_completions(path2);
        assert_eq!(found2.len(), 1, "Should find path2");
        assert_eq!(found2[0].0, path2, "Should match exact path2");

        // Check prefix search - should find both
        let prefix_results = trie.find_completions("a/b/file");
        assert_eq!(prefix_results.len(), 2, "Prefix search should find both files");
    }

    #[test]
    fn test_remove_path() {
        log_info!("Testing path removal with multiple related paths");
        let mut trie = ART::new(10);

        // Create paths as literal strings - no helpers or conversions
        let path1 = "a/b/file1.txt";
        let path2 = "home/user/file2.txt";
        let path3 = "home/other/file3.txt";

        // Insert them with standard syntax
        trie.insert(path1, 1.0);
        trie.insert(path2, 1.0);
        trie.insert(path3, 1.0);

        assert_eq!(trie.len(), 3, "Should have 3 paths after insertion");

        // Check that path1 exists - use the same string reference
        let before_completions = trie.find_completions(path1);
        log_info!(&format!("Before removal: found {} completions for '{}'",
                      before_completions.len(), path1));
        log_info!(&format!("is_in_trie: {}", trie.find_completions(path1).len() > 0));
        assert_eq!(before_completions.len(), 1, "Path1 should be found before removal");

        // If needed, verify the exact string (for debugging)
        if !before_completions.is_empty() {
            let found_path = &before_completions[0].0;
            log_info!(&format!("Found path: '{}', Expected: '{}'", found_path, path1));
            log_info!(&format!("Path bytes: {:?}", found_path.as_bytes()));
            log_info!(&format!("Expected bytes: {:?}", path1.as_bytes()));
        }

        // Remove path1
        let removed = trie.remove(path1);
        assert!(removed, "Path1 should be successfully removed");
        assert_eq!(trie.len(), 2, "Should have 2 paths after removal");

        // Verify path1 is gone
        let after_completions = trie.find_completions(path1);
        assert_eq!(after_completions.len(), 0, "Path1 should be gone after removal");

        // Check that we still find path2 with a common prefix search
        let user_prefix = "home/user/";
        let user_paths = trie.find_completions(user_prefix);
        assert_eq!(user_paths.len(), 1, "Should find only 1 user path after removal");
        assert_eq!(user_paths[0].0, path2, "The remaining user path should be path2");
    }

    #[test]
    fn test_prefix_matching() {
        log_info!("Testing prefix matching functionality");
        let mut trie = ART::new(100);

        // Insert paths with common prefixes
        let path1 = normalize_path("/usr/local/bin/program1");
        let path2 = normalize_path("/usr/local/bin/program2");
        let path3 = normalize_path("/usr/local/lib/library1");
        let path4 = normalize_path("/usr/share/doc/readme");

        trie.insert(&path1, 1.0);
        trie.insert(&path2, 0.9);
        trie.insert(&path3, 0.8);
        trie.insert(&path4, 0.7);

        // Test various prefix lengths
        let test_cases = vec![
            (normalize_path("/usr"), 4),
            (normalize_path("/usr/local"), 3),
            (normalize_path("/usr/local/bin"), 2),
            (normalize_path("/usr/local/bin/program"), 2),
            (normalize_path("/usr/share"), 1),
            (normalize_path("/nonexistent"), 0),
        ];

        for (prefix, expected_count) in test_cases {
            let completions = trie.find_completions(&prefix);
            assert_eq!(completions.len(), expected_count, "Failed for prefix: {}", prefix);
            log_info!(&format!("Prefix '{}' returned {} completions", prefix, completions.len()));
        }
    }

    #[test]
    fn test_clear_trie() {
        log_info!("Testing trie clearing");
        let mut trie = ART::new(10);

        // Insert some paths
        trie.insert(&normalize_path("/path1"), 1.0);
        trie.insert(&normalize_path("/path2"), 0.9);

        assert_eq!(trie.len(), 2);

        // Clear the trie
        trie.clear();

        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());

        let completions = trie.find_completions(&normalize_path("/"));
        assert_eq!(completions.len(), 0);
        log_info!("Trie successfully cleared");

        // Insert after clearing
        trie.insert(&normalize_path("/new_path"), 1.0);
        assert_eq!(trie.len(), 1);
        log_info!("Successfully inserted after clearing");
    }

    #[test]
    fn test_file_extensions() {
        let mut trie = ART::new(10);

        // Paths with file extensions
        let path1 = "a/b/file1.txt";
        let path2 = "a/b/file2.txt";

        // Insert path
        trie.insert(path1, 1.0);
        trie.insert(path2, 1.0);

        // Check exact match
        let found = trie.find_completions(path1);
        assert_eq!(found.len(), 1, "Should find the exact path with extension");

        // Log for debugging
        log_info!(&format!("Paths found for '{}': {}", path1, found.len()));
        for (i, (path, score)) in found.iter().enumerate() {
            log_info!(&format!("  Path {}: {} (score: {})", i, path, score));
        }
    }

    #[test]
    fn test_scoring_and_sorting() {
        log_info!("Testing score-based sorting of completions");
        let mut trie = ART::new(10);

        // Insert paths with different scores
        trie.insert(&normalize_path("/docs/low"), 0.1);
        trie.insert(&normalize_path("/docs/medium"), 0.5);
        trie.insert(&normalize_path("/docs/high"), 0.9);

        // Get completions and verify sorting
        let completions = trie.find_completions(&normalize_path("/docs/"));

        assert_eq!(completions.len(), 3);
        assert!(completions[0].0.ends_with(&normalize_path("/high")));
        assert!(completions[1].0.ends_with(&normalize_path("/medium")));
        assert!(completions[2].0.ends_with(&normalize_path("/low")));

        log_info!(&format!("Completions correctly sorted by score: {:.1} > {:.1} > {:.1}",
            completions[0].1, completions[1].1, completions[2].1));
    }

    // Performance tests with real-world data
    #[test]
    fn test_insertion_performance() {
        log_info!("Testing insertion performance with real paths");
        let mut trie = ART::new(100);

        // Get real-world paths from test data
        let paths = collect_test_paths(Some(500));
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Measure time to insert all paths
        let start = Instant::now();
        for (i, path) in paths.iter().enumerate() {
            trie.insert(path, 1.0 - (i as f32 * 0.001));
        }
        let elapsed = start.elapsed();

        log_info!(&format!("Inserted {} paths in {:?} ({:.2} paths/ms)",
            paths.len(), elapsed, paths.len() as f64 / elapsed.as_millis() as f64));

        assert_eq!(trie.len(), paths.len());
    }

    #[test]
    fn test_completion_performance() {
        log_info!("Testing completion performance with real paths");
        let mut trie = ART::new(1000);

        // Get real-world paths from test data
        let paths = collect_test_paths(Some(1000));
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Insert all paths
        for (i, path) in paths.iter().enumerate() {
            trie.insert(path, 1.0 - (i as f32 * 0.0001));
        }

        // Extract some prefixes to test from the actual data
        let test_prefixes: Vec<String> = if !paths.is_empty() {
            let mut prefixes = Vec::new();

            // Use the first character of the first path
            if let Some(first_path) = paths.first() {
                if !first_path.is_empty() {
                    prefixes.push(first_path[0..1].to_string());
                }
            }

            // Use the directory portion of some paths
            for path in paths.iter().take(5) {
                if let Some(last_sep) = path.rfind(MAIN_SEPARATOR) {
                    prefixes.push(path[0..last_sep+1].to_string());
                }
            }

            // If we couldn't extract enough prefixes, add some generic ones
            if prefixes.len() < 3 {
                prefixes.push(normalize_path("/"));
                prefixes.push(normalize_path("/usr"));
                prefixes.push(normalize_path("/home"));
            }

            prefixes
        } else {
            vec![
                normalize_path("/"),
                normalize_path("/usr"),
                normalize_path("/home")
            ]
        };

        for prefix in test_prefixes {
            let start = Instant::now();
            let completions = trie.find_completions(&prefix);
            let elapsed = start.elapsed();

            log_info!(&format!("Found {} completions for '{}' in {:?}",
                completions.len(), prefix, elapsed));

            if completions.len() > 0 {
                log_info!(&format!("First completion: {} (score: {:.1})",
                    completions[0].0, completions[0].1));
            }
        }
    }

    #[test]
    fn test_specific_path_cases() {
        let mut trie = ART::new(10);

        // Test the specific cases from your logs
        let base_path = "./test-data-for-fuzzy-search";
        let files = vec![
            "/airplane.mp4",
            "/ambulance",
            "/apple.pdf"
        ];

        // Insert each file path
        for file in &files {
            let full_path = format!("{}{}", base_path, file);
            trie.insert(&full_path, 1.0);

            // Immediately verify it was added correctly
            let found = trie.find_completions(&full_path);
            assert_eq!(found.len(), 1, "Path should be found");
            assert_eq!(found[0].0, full_path, "Path should match exactly");

            // Log the path for verification
            log_info!(&format!("Inserted and verified path: {}", full_path));
        }

        // Test base path search
        let completions = trie.find_completions(base_path);

        // Check each completion against expected paths
        for (i, file) in files.iter().enumerate() {
            let expected_path = format!("{}{}", base_path, file);
            let found = completions.iter().any(|(path, _)| path == &expected_path);

            assert!(found, "Path {} should be found in completions", expected_path);
            log_info!(&format!("Found expected path {}: {}", i, expected_path));
        }

        // Test partially matching path
        let partial_path = format!("{}/a", base_path);
        let partial_completions = trie.find_completions(&partial_path);

        assert!(partial_completions.len() >= 2,
                "Should find at least airplane.mp4 and apple.pdf");

        // Verify no character splitting
        for (path, _) in &partial_completions {
            // Check no character was incorrectly split
            assert!(!path.contains("/i/rplane"), "No character splitting in airplane");
            assert!(!path.contains("/m/bulance"), "No character splitting in ambulance");
            assert!(!path.contains("/a/pple"), "No character splitting in apple");
        }
    }

    #[test]
    fn test_node_sizing_and_shrinking() {
        log_info!("Testing node sizing and automatic shrinking");
        let mut trie = ART::new(100);

        // Create a common prefix path
        let prefix = normalize_path("/common/prefix/path_");

        // Insert enough paths to force node growth
        for i in 0..100 {
            // Create paths with the same prefix but different last bytes
            // to force node growth at the same level
            let path = format!("{}{:03}", prefix, i);
            trie.insert(&path, 1.0);
        }

        log_info!(&format!("Inserted {} paths with common prefix", trie.len()));

        // Check that we get all the completions
        let completions = trie.find_completions(&prefix);
        assert_eq!(completions.len(), 100);
        log_info!("Successfully retrieved all completions after node growth");

        // Now remove paths to force node shrinking
        for i in 0..90 {
            let path = format!("{}{:03}", prefix, i);
            assert!(trie.remove(&path));
        }

        log_info!(&format!("Removed 90 paths, trie now contains {} paths", trie.len()));

        // Check we can still find the remaining paths
        let completions = trie.find_completions(&prefix);
        assert_eq!(completions.len(), 10);
        log_info!("Successfully retrieved remaining completions after node shrinking");
    }

    #[test]
    fn test_duplicate_insertion() {
        let mut trie = ART::new(10);
        let test_path = normalize_path("/path/to/file");

        assert!(trie.insert(&test_path, 1.0));
        // Second insertion should either return false or update the score
        assert!(!trie.insert(&test_path, 0.8) || trie.find_completions(&test_path)[0].1 == 0.8);
        assert_eq!(trie.len(), 1); // Length should still be 1
    }

    #[test]
    fn debug_test() {
        let mut trie = ART::new(10);
        let path = "a/b/file1.txt";
        let path2 = "a/b/file2.txt";
        let path3 = "a/b/d";
        trie.insert(path, 1.0);
        trie.insert(path2, 1.0);
        trie.insert(path3, 1.0);
        let found = trie.find_completions(path);
        assert_eq!(found.len(), 1, "Should find the exact path with extension");
        trie.remove(path);
        log_info!(&format!("is_in_trie: {}", trie.find_completions(path).len() == 0));
    }
    #[test]
    fn test_long_path() {
        let mut trie = ART::new(10);
        let long_path = normalize_path("/very/long/path/").repeat(20) + "file.txt";
        assert!(trie.insert(&long_path, 1.0));
        let completions = trie.find_completions(&normalize_path("/very/long"));
        assert_eq!(completions.len(), 1);
    }

    #[test]
    fn test_search_with_current_directory() {
        let mut trie = ART::new(10);

        // Insert test paths
        trie.insert("home/user/documents/important.txt", 1.0);
        trie.insert("home/user/pictures/vacation.jpg", 0.9);
        trie.insert("home/other/documents/report.pdf", 0.8);

        // Test 1: Direct prefix search
        let results1 = trie.search("home", None, false);
        assert_eq!(results1.len(), 3);

        // Test 2: Search with current directory context
        let results2 = trie.search("doc", Some("home/user"), true);
        assert_eq!(results2.len(), 1, "Should only find documents in home/user");
        assert_eq!(results2[0].0, "home/user/documents/important.txt");

        // Test 3: Search with different current directory context
        let results3 = trie.search("doc", Some("home/other"), true);
        assert_eq!(results3.len(), 1, "Should only find documents in home/other");
        assert_eq!(results3[0].0, "home/other/documents/report.pdf");

        // Test 4: Partial component matching without directory context
        let results4 = trie.search("doc", None, true);
        assert_eq!(results4.len(), 2, "Should find all paths with 'doc' component");

        // Test 5: Search for component that's not in the path
        let results5 = trie.search("missing", Some("home/user"), true);
        assert_eq!(results5.len(), 0, "Should find no results for non-existent component");
    }

    #[test]
    fn test_prefix_compression() {
        let mut trie = ART::new(10);

        let path1 = normalize_path("/common/prefix/path/file1.txt");
        let path2 = normalize_path("/common/prefix/path/file2.txt");
        let path3 = normalize_path("/common/prefix/other/file3.txt");

        trie.insert(&path1, 1.0);
        trie.insert(&path2, 0.9);
        trie.insert(&path3, 0.8);

        // Memory usage would be lower with compression than without
        let completions = trie.find_completions(&normalize_path("/common/prefix"));
        assert_eq!(completions.len(), 3);
    }

    #[test]
    fn test_with_real_world_data_art_v3() {
        log_info!("Testing ART with real-world data");
        let mut trie = ART::new(100);

        // Get all available test paths
        let paths = collect_test_paths(Some(500));
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Insert paths with slightly decreasing scores
        for (i, path) in paths.iter().enumerate() {
            trie.insert(path, 1.0 - (i as f32 * 0.001));
        }

        log_info!(&format!("Inserted {} paths into trie", trie.len()));

        // Extract some common prefixes from the data for testing
        let mut test_prefixes: Vec<String> = if !paths.is_empty() {
            let mut prefixes = Vec::new();

            // Try to find common directory components
            let mut common_dirs = std::collections::HashMap::new();
            for path in &paths {
                let components: Vec<&str> = path.split(MAIN_SEPARATOR).collect();
                for (i, component) in components.iter().enumerate() {
                    if !component.is_empty() {
                        let prefix_path = components[0..=i].join(&MAIN_SEPARATOR.to_string());
                        *common_dirs.entry(prefix_path).or_insert(0) += 1;
                    }
                }
            }

            // Use the most common prefixes
            let mut prefix_counts: Vec<(String, usize)> = common_dirs.into_iter().collect();
            prefix_counts.sort_by(|a, b| b.1.cmp(&a.1));

            for (prefix, _count) in prefix_counts.into_iter().take(5) {
                prefixes.push(prefix);
            }

            if prefixes.is_empty() {
                // Fallback if we couldn't extract common prefixes
                prefixes.push(paths[0].chars().take(3).collect());
            }

            prefixes
        } else {
            vec![normalize_path("/usr"), normalize_path("/home")]
        };

        // Add partial prefix matches to test
        let mut partial_prefixes = Vec::new();

        for prefix in &test_prefixes {
            // Add first few characters of each prefix
            if prefix.len() >= 3 {
                partial_prefixes.push(prefix.chars().take(2).collect::<String>());
                partial_prefixes.push(prefix.chars().take(3).collect::<String>());
            }

            // Add partial directory path if it contains separators
            if let Some(last_sep_pos) = prefix.rfind(MAIN_SEPARATOR) {
                if last_sep_pos > 0 && last_sep_pos < prefix.len() - 1 {
                    // Add partial component after the last separator
                    let component = &prefix[last_sep_pos+1..];
                    if component.len() >= 2 {
                        partial_prefixes.push(format!("{}{}",
                                                      &prefix[..=last_sep_pos],
                                                      &component[..component.len().min(2)]));
                    }
                }
            }
        }

        // Combine exact and partial prefixes
        test_prefixes.extend(partial_prefixes);

        // Test searching with all the prefixes
        for original_prefix in test_prefixes {
            // Create a temporary ART instance for path normalization
            let temp_art = ART::new(1);
            let normalized_prefix = temp_art.normalize_path(&original_prefix);

            let start = Instant::now();
            let completions = trie.find_completions(&original_prefix);
            let elapsed = start.elapsed();

            log_info!(&format!("Found {} completions for prefix '{}' in {:?}",
                  completions.len(), original_prefix, elapsed));

            if !completions.is_empty() {
                log_info!(&format!("First result: {} (score: {:.2})",
                      completions[0].0, completions[0].1));

                // Verify that results actually match the normalized prefix
                let valid_matches = completions.iter()
                    .filter(|(path, _)| path.starts_with(&normalized_prefix))
                    .count();

                log_info!(&format!("{} of {} results are valid prefix matches for '{}' (normalized: '{}')",
                      valid_matches, completions.len(), original_prefix, normalized_prefix));

                assert!(valid_matches > 0, "No valid matches found for prefix '{}' (normalized: '{}')",
                        original_prefix, normalized_prefix);
            }
        }

        // Test removing a subset of paths
        let to_remove = paths.len().min(50);
        let mut removed = 0;

        for i in 0..to_remove {
            if trie.remove(&paths[i]) {
                removed += 1;
            }
        }

        log_info!(&format!("Successfully removed {} paths", removed));
        assert_eq!(trie.len(), paths.len() - removed);
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn benchmark_prefix_search_with_all_paths_art_v3() {
        log_info!("Benchmarking prefix search with thousands of real-world paths");

        // 1. Collect all available paths
        let paths = collect_test_paths(None); // Get all available paths
        let path_count = paths.len();

        log_info!(&format!("Collected {} test paths", path_count));

        // If we don't have enough paths, generate more synthetic ones
        let all_paths = paths.clone();

        // 2. Create ART and insert all paths
        let start_insert = Instant::now();
        let mut trie = ART::new(100);

        for (i, path) in all_paths.iter().enumerate() {
            // Use varying scores based on position
            let score = 1.0 - (i as f32 * 0.0001).min(0.99);
            trie.insert(path, score);
        }

        let insert_time = start_insert.elapsed();
        log_info!(&format!("Inserted {} paths in {:?} ({:.2} paths/ms)",
            all_paths.len(), insert_time,
            all_paths.len() as f64 / insert_time.as_millis().max(1) as f64));

        // 3. Generate diverse test prefixes
        let mut test_prefixes = Vec::new();

        // a. Most common directory components
        let mut prefix_counts = std::collections::HashMap::new();
        for path in &all_paths {
            let components: Vec<&str> = path.split(MAIN_SEPARATOR).collect();
            for i in 1..components.len() {
                let prefix = components[0..i].join(&MAIN_SEPARATOR.to_string());
                *prefix_counts.entry(prefix).or_insert(0) += 1;
            }
        }

        // Use the most common prefixes
        let mut common_prefixes: Vec<(String, usize)> = prefix_counts.into_iter().collect();
        common_prefixes.sort_by(|a, b| b.1.cmp(&a.1));

        for (prefix, _) in common_prefixes.into_iter().take(10) {
            if !prefix.is_empty() {
                test_prefixes.push(prefix);
            }
        }

        // b. Add some partial prefix matches
        if !all_paths.is_empty() {
            for i in 0..5 {
                let path_idx = (i * all_paths.len() / 5) % all_paths.len();
                let path = &all_paths[path_idx];

                if let Some(last_sep_pos) = path.rfind(MAIN_SEPARATOR) {
                    if last_sep_pos > 0 {
                        // Add full directory
                        test_prefixes.push(path[..last_sep_pos].to_string());

                        // Add partial directory name
                        if last_sep_pos + 2 < path.len() {
                            test_prefixes.push(path[..last_sep_pos+2].to_string());
                        }
                    }
                }

                // Add first few characters
                if path.len() >= 3 {
                    test_prefixes.push(path.chars().take(3).collect::<String>());
                }
            }
        }

        // c. Add short and very specific prefixes
        test_prefixes.extend(vec![
            "./t".to_string(),
            "./".to_string(),
        ]);

        // Remove duplicates
        test_prefixes.sort();
        test_prefixes.dedup();

        // 4. Benchmark searches with different batch sizes
        let batch_sizes = [10, 100, 1000, 10000, all_paths.len()];

        for &batch_size in &batch_sizes {
            // Create a subset trie with the specified number of paths
            let subset_size = batch_size.min(all_paths.len());
            let mut subset_trie = ART::new(100);

            for i in 0..subset_size {
                subset_trie.insert(&all_paths[i], 1.0 - (i as f32 * 0.0001));
            }

            log_info!(&format!("\n=== BENCHMARK WITH {} PATHS ===", subset_size));

            let mut total_time = Duration::new(0, 0);
            let mut total_results = 0;
            let mut times = Vec::new();

            for prefix in &test_prefixes {
                let normalized_prefix = normalize_path(prefix);
                let start = Instant::now();
                let completions = subset_trie.find_completions(&normalized_prefix);
                let elapsed = start.elapsed();

                total_time += elapsed;
                total_results += completions.len();
                times.push((prefix.clone(), elapsed, completions.len()));
            }

            // 5. Report statistics for this batch size
            times.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by time, slowest first

            let avg_time = if !test_prefixes.is_empty() {
                total_time / test_prefixes.len() as u32
            } else {
                Duration::new(0, 0)
            };

            let avg_results = if !test_prefixes.is_empty() {
                total_results / test_prefixes.len()
            } else {
                0
            };

            log_info!(&format!("Ran {} prefix searches", test_prefixes.len()));
            log_info!(&format!("Average search time: {:?}", avg_time));
            log_info!(&format!("Average results per search: {}", avg_results));

            // Log the slowest searches
            log_info!("Slowest searches:");
            for (i, (prefix, time, count)) in times.iter().take(3).enumerate() {
                log_info!(&format!("  #{}: '{:40}' - {:?} ({} results)",
                    i+1, prefix, time, count));
            }

            // Log the fastest searches
            log_info!("Fastest searches:");
            for (i, (prefix, time, count)) in times.iter().rev().take(3).enumerate() {
                log_info!(&format!("  #{}: '{:40}' - {:?} ({} results)",
                    i+1, prefix, time, count));
            }

            // Log search times for different result sizes
            let mut by_result_count = Vec::new();
            for &count in &[0, 1, 10, 100] {
                let matching: Vec<_> = times.iter()
                    .filter(|(_, _, c)| *c >= count)
                    .collect();

                if !matching.is_empty() {
                    let total = matching.iter()
                        .fold(Duration::new(0, 0), |sum, (_, time, _)| sum + *time);
                    let avg = total / matching.len() as u32;

                    by_result_count.push((count, avg, matching.len()));
                }
            }

            log_info!("Average search times by result count:");
            for (count, avg_time, num_searches) in by_result_count {
                log_info!(&format!("  ≥ {:3} results: {:?} (from {} searches)",
                    count, avg_time, num_searches));
            }
        }
    }
}
