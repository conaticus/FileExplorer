use std::cmp;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use crate::log_info;

/// An implementation of an Adaptive Radix Trie for fast path autocompletion
pub struct ART {
    // Root nodes - one per root path component
    root: Option<Box<Node>>,

    // Number of paths stored
    path_count: usize,

    // Maximum number of results to return
    max_results: usize,
}

/// Node in the ART with different capacities based on the number of children
#[derive(Clone)]
enum Node {
    // Node with up to 4 children (most common case)
    Node4 {
        // Common node data
        is_terminal: bool,
        score: f32,
        last_accessed: Instant,

        // Path compression - store shared prefix once
        prefix: Vec<u8>,

        // Child nodes
        keys: Vec<u8>,
        children: Vec<Option<Box<Node>>>,
    },

    // Node with up to 16 children
    Node16 {
        is_terminal: bool,
        score: f32,
        last_accessed: Instant,
        prefix: Vec<u8>,
        keys: Vec<u8>,
        children: Vec<Option<Box<Node>>>,
    },

    // Node with up to 48 children
    Node48 {
        is_terminal: bool,
        score: f32,
        last_accessed: Instant,
        prefix: Vec<u8>,
        // Fixed size index array for O(1) lookups
        indices: [Option<usize>; 256],
        children: Vec<Option<Box<Node>>>,
    },

    // Node with up to 256 children (full byte range)
    Node256 {
        is_terminal: bool,
        score: f32,
        last_accessed: Instant,
        prefix: Vec<u8>,
        children: [Option<Box<Node>>; 256],
    },
}

impl ART {
    /// Create a new Adaptive Radix Trie
    pub fn new(max_results: usize) -> Self {
        Self {
            root: None,
            path_count: 0,
            max_results,
        }
    }

    /// Insert a path into the trie with a relevance score
    pub fn insert(&mut self, path: &str, score: f32) -> bool {
        let normalized_path = self.normalize_path(path);
        let bytes = normalized_path.as_bytes();

        if bytes.is_empty() {
            return false;
        }

        // Create root node if it doesn't exist
        if self.root.is_none() {
            self.root = Some(Box::new(Node::new_node4()));
        }

        let mut changed = false;
        if let Some(root) = &mut self.root {
            changed = Self::insert_recursive(root, bytes, 0, score);
        }

        if changed {
            self.path_count += 1;
        }

        changed
    }

    /// Find completions with fixed path handling
    pub fn find_completions(&mut self, prefix: &str) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        if self.root.is_none() {
            return results;
        }

        // Normalize the prefix
        let normalized_prefix = self.normalize_path(prefix);
        let prefix_bytes = normalized_prefix.as_bytes();

        // Find the node corresponding to the prefix
        if let Some(root) = &mut self.root {
            if let Some(node) = Self::find_prefix_node(root, prefix_bytes, 0) {
                // Get node prefix
                let node_prefix = node.get_prefix();

                // CRITICAL IMPROVEMENT: Analyze if this node might contain split characters
                let mut parent_last_char = None;
                if prefix_bytes.len() >= 2 &&
                    prefix_bytes[prefix_bytes.len() - 2] == b'/' &&
                    prefix_bytes[prefix_bytes.len() - 1].is_ascii_alphanumeric() {
                    // This looks like we're at a prefix ending with a single char
                    // after a separator (like "/a")
                    parent_last_char = Some(prefix_bytes[prefix_bytes.len() - 1]);
                }

                // Build the path to this point
                let mut current_path = Vec::with_capacity(normalized_prefix.len());
                current_path.extend_from_slice(prefix_bytes);

                // Check if the prefix itself is a valid path
                if node.is_terminal() {
                    node.update_access_time();
                    results.push((normalized_prefix.clone(), node.score()));
                }

                // Collect completions with the parent character info
                Self::collect_completions_with_parent_char(
                    node,
                    &mut current_path,
                    &mut results,
                    parent_last_char
                );

                // Sort and limit results
                results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(cmp::Ordering::Equal));
                if results.len() > self.max_results {
                    results.truncate(self.max_results);
                }
            }
        }

        results
    }

    /// Helper that passes parent character information
    fn collect_completions_with_parent_char(
        node: &mut Box<Node>,
        prefix: &mut Vec<u8>,
        results: &mut Vec<(String, f32)>,
        parent_last_char: Option<u8>
    ) {
        let mut stack = Vec::new();
        stack.push((node as *mut Box<Node>, prefix.len(), parent_last_char));

        while let Some((node_ptr, prefix_len, parent_last_char)) = stack.pop() {
            let node = unsafe { &mut *node_ptr };
            node.update_access_time();

            // Restore prefix to parent's state
            prefix.truncate(prefix_len);

            // Check if this node is a terminal node
            if node.is_terminal() {
                let path_str = String::from_utf8_lossy(prefix).to_string();
                results.push((path_str, node.score()));
            }

            // Analyze the current prefix
            let is_path = prefix.contains(&b'/');
            let ends_with_separator = prefix.ends_with(&[b'/']);

            // Process children in reverse (for stack order matching DFS)
            for byte in (0..=255u8).rev() {
                if let Some(child) = node.get_child_mut(byte) {
                    // Get child's prefix
                    let child_prefix = child.get_prefix();
                    let child_starts_with_separator = !child_prefix.is_empty() && child_prefix[0] == b'/';

                    // Save original length
                    let original_len = prefix.len();

                    // CRITICAL PART 1: Determine if we need a separator
                    let needs_separator = !prefix.is_empty() &&
                        !ends_with_separator &&
                        !child_starts_with_separator &&
                        is_path;

                    if needs_separator {
                        prefix.push(b'/');
                    }

                    // CRITICAL PART 2: Check if parent has a split character
                    // we need to add to the path
                    if let Some(last_char) = parent_last_char {
                        // Add the parent's last character if it's a split character
                        // Only add if we're building a filename (after a separator)
                        if ends_with_separator || needs_separator {
                            prefix.push(last_char);
                        }
                    }

                    // Add the byte
                    prefix.push(byte);

                    // Add child's prefix
                    prefix.extend_from_slice(child_prefix);

                    // Log reconstructed path
                    log_info!(&format!("Collecting: added byte={}, prefix={}",
                        byte as char, String::from_utf8_lossy(prefix)));

                    // CRITICAL PART 3: Check if this child's path needs special handling
                    // in the next iteration because we're at a split component
                    let mut next_parent_char = None;

                    // If prefix ends with one character after a separator,
                    // this might be a split character
                    if prefix.len() >= 2 &&
                        prefix[prefix.len() - 2] == b'/' &&
                        prefix[prefix.len() - 1].is_ascii_alphanumeric() {
                        // Remember this character for children
                        next_parent_char = Some(prefix[prefix.len() - 1]);
                    }

                    // Push child to stack with parent char info
                    stack.push((child as *mut Box<Node>, prefix.len(), next_parent_char));

                    // Restore prefix
                    prefix.truncate(original_len);
                }
            }
        }
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

    /// Remove a path from the trie
    pub fn remove(&mut self, path: &str) -> bool {
        // Step 1: Normalize the path exactly the same way as during insertion
        let normalized_path = self.normalize_path(path);
        log_info!(&format!("Attempting to remove normalized path: '{}'", normalized_path));

        // Step 2: First verify the path exists using find_completions
        let exists = self.find_completions(&normalized_path)
            .iter()
            .any(|(p, _)| p == &normalized_path);

        if !exists {
            log_info!(&format!("Path not found for removal: {}", normalized_path));
            return false;
        }

        log_info!(&format!("Path exists, proceeding with removal: {}", normalized_path));

        // Step 3: Path exists, proceed with removal
        let bytes = normalized_path.as_bytes();

        if bytes.is_empty() || self.root.is_none() {
            return false;
        }

        let mut changed = false;
        if let Some(root) = &mut self.root {
            changed = Self::remove_recursive(root, bytes, 0);
        }

        if changed {
            self.path_count -= 1;
            log_info!(&format!("Successfully removed path: {}", normalized_path));
        }

        changed
    }

    // ===== Helper methods =====

    /// Recursively insert a path with path-aware splitting
    fn insert_recursive(
        node: &mut Box<Node>,
        key: &[u8],
        depth: usize,
        score: f32
    ) -> bool {
        // Check prefix match
        let (match_len, match_exact) = node.check_prefix(key, depth);

        if !match_exact {
            // Handle split
            let was_terminal = node.is_terminal();
            let original_score = node.score();
            let original_prefix = node.get_prefix().to_vec();

            // Store children
            let mut original_children = Vec::new();
            for b in 0..=255u8 {
                if let Some(child) = node.get_child_mut(b) {
                    let child_owned = std::mem::replace(child, Box::new(Node::new_node4()));
                    original_children.push((b, child_owned));
                }
            }

            // Split the node
            node.split_prefix(match_len);

            if match_len < original_prefix.len() {
                // Create child for original path
                let mut original_child = Node::new_node4();
                original_child.set_terminal(was_terminal);
                if was_terminal {
                    original_child.set_score(original_score);
                }

                // CRITICAL FIX: Character at split position handling
                if match_len + 1 < original_prefix.len() {
                    // The character at the split position is handled by the node's child key
                    // so we start from match_len+1 in the original prefix
                    original_child.add_to_prefix(&original_prefix[match_len+1..]);
                }

                // Add original children
                for (byte, child) in original_children {
                    original_child.add_child(byte, child);
                }

                // Add to parent - using the character at the split position as the key
                node.add_child(original_prefix[match_len], Box::new(original_child));
            }

            // Handle the new key path
            if depth + match_len < key.len() {
                // Create child for new key - using the character at the split as the key
                let key_byte = key[depth + match_len];
                let mut new_child = Node::new_node4();
                new_child.set_terminal(true);
                new_child.set_score(score);

                // Add the remaining key part - again, starting AFTER the split character
                if depth + match_len + 1 < key.len() {
                    new_child.add_to_prefix(&key[depth + match_len + 1..]);
                }

                node.add_child(key_byte, Box::new(new_child));
                node.set_terminal(false);
            } else {
                node.set_terminal(true);
                node.set_score(score);
            }

            return true;
        }

        // Rest of function unchanged...
        let new_depth = depth + node.prefix_len();

        if new_depth == key.len() {
            let is_new = !node.is_terminal();
            node.set_terminal(true);
            node.set_score(score);
            return is_new;
        }

        let byte = key[new_depth];

        if let Some(child) = node.get_child_mut(byte) {
            Self::insert_recursive(child, key, new_depth + 1, score)
        } else {
            let mut child = Node::new_node4();
            child.set_terminal(true);
            child.set_score(score);

            if new_depth + 1 < key.len() {
                child.add_to_prefix(&key[new_depth + 1..]);
            }

            node.add_child(byte, Box::new(child));
            true
        }
    }

    /// Find the node corresponding to a prefix
    fn find_prefix_node<'a>(
        node: &'a mut Box<Node>,
        key: &[u8],
        depth: usize
    ) -> Option<&'a mut Box<Node>> {
        let mut current_node = node;
        let mut current_depth = depth;

        loop {
            if current_depth >= key.len() {
                return Some(current_node);
            }

            // Check prefix match
            let (match_len, match_exact) = current_node.check_prefix(key, current_depth);

            if !match_exact && current_depth + match_len < key.len() {
                // Prefix mismatch - key not in trie
                return None;
            }

            // If we've exhausted the key during prefix matching, this node is a match
            if current_depth + match_len >= key.len() {
                return Some(current_node);
            }

            // Move to next node
            let new_depth = current_depth + current_node.prefix_len();

            if new_depth >= key.len() {
                return Some(current_node);
            }

            let byte = key[new_depth];

            if let Some(child) = current_node.get_child_mut(byte) {
                // Continue with child
                current_node = child;
                current_depth = new_depth + 1;
            } else {
                return None;
            }
        }
    }

    /// Complete solution for correct path reconstruction including split characters
    fn collect_completions(
        node: &mut Box<Node>,
        prefix: &mut Vec<u8>,
        results: &mut Vec<(String, f32)>
    ) {
        // Use a stack for traversal
        let mut stack = Vec::new();

        // Each entry: (node pointer, parent prefix length, parent's last character if relevant)
        stack.push((node as *mut Box<Node>, prefix.len(), None));

        while let Some((node_ptr, prefix_len, parent_last_char)) = stack.pop() {
            let node = unsafe { &mut *node_ptr };
            node.update_access_time();

            // Restore prefix to parent's state
            prefix.truncate(prefix_len);

            // Check if this node is a terminal node
            if node.is_terminal() {
                let path_str = String::from_utf8_lossy(prefix).to_string();
                results.push((path_str, node.score()));
            }

            // Analyze the current prefix
            let is_path = prefix.contains(&b'/');
            let ends_with_separator = prefix.ends_with(&[b'/']);

            // Process children in reverse (for stack order matching DFS)
            for byte in (0..=255u8).rev() {
                if let Some(child) = node.get_child_mut(byte) {
                    // Get child's prefix
                    let child_prefix = child.get_prefix();
                    let child_starts_with_separator = !child_prefix.is_empty() && child_prefix[0] == b'/';

                    // Save original length
                    let original_len = prefix.len();

                    // CRITICAL PART 1: Determine if we need a separator
                    let needs_separator = !prefix.is_empty() &&
                        !ends_with_separator &&
                        !child_starts_with_separator &&
                        is_path;

                    if needs_separator {
                        prefix.push(b'/');
                    }

                    // CRITICAL PART 2: Check if parent has a split character
                    // we need to add to the path
                    if let Some(last_char) = parent_last_char {
                        // Add the parent's last character if it's a split character
                        // Only add if we're building a filename (after a separator)
                        if ends_with_separator || needs_separator {
                            prefix.push(last_char);
                        }
                    }

                    // Add the byte
                    prefix.push(byte);

                    // Add child's prefix
                    prefix.extend_from_slice(child_prefix);

                    // Log reconstructed path
                    log_info!(&format!("Collecting: added byte={}, prefix={}",
                        byte as char, String::from_utf8_lossy(prefix)));

                    // CRITICAL PART 3: Check if this child's path needs special handling
                    // in the next iteration because we're at a split component
                    let mut next_parent_char = None;

                    // If prefix ends with one character after a separator,
                    // this might be a split character
                    if prefix.len() >= 2 &&
                        prefix[prefix.len() - 2] == b'/' &&
                        prefix[prefix.len() - 1].is_ascii_alphanumeric() {
                        // Remember this character for children
                        next_parent_char = Some(prefix[prefix.len() - 1]);
                    }

                    // Push child to stack with parent char info
                    stack.push((child as *mut Box<Node>, prefix.len(), next_parent_char));

                    // Restore prefix
                    prefix.truncate(original_len);
                }
            }
        }
    }

    /// Check if a byte is a valid path character
    fn is_path_char(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-' ||
            byte == b'_' || byte == b'~' || byte == b'+'
    }

    /// Helper method for recursive removal
    fn remove_recursive(node: &mut Box<Node>, key: &[u8], depth: usize) -> bool {
        // Check prefix match
        let (match_len, match_exact) = node.check_prefix(key, depth);

        // For removal, we need to match the entire prefix
        if match_len < node.prefix_len() {
            log_info!(&format!("Path removal failed: prefix not fully matched at depth {}", depth));
            return false;
        }

        // Skip the matched prefix
        let new_depth = depth + match_len;

        // Check if we've reached the end of the key
        if new_depth == key.len() {
            // Found the node to remove
            if !node.is_terminal() {
                log_info!(&format!("Path removal failed: found node but it's not terminal"));
                return false;
            }

            // Mark as non-terminal
            node.set_terminal(false);
            log_info!(&format!("Successfully removed path (marked node as non-terminal)"));

            return true;
        }

        // Continue to child node
        let byte = key[new_depth];

        if let Some(child) = node.get_child_mut(byte) {
            // Recursively remove from child
            let removed = Self::remove_recursive(child, key, new_depth + 1);

            // If child is empty after removal, remove it
            if removed && !child.is_terminal() && child.child_count() == 0 {
                node.remove_child(byte);
                log_info!(&format!("Removed empty child node for byte {}", byte));
            }

            return removed;
        }

        log_info!(&format!("Path removal failed: no child for byte {} at depth {}", byte, new_depth));
        false
    }

    /// Normalize paths with improved separator handling
    fn normalize_path(&self, path: &str) -> String {
        // Handle platform-specific separators
        let mut normalized = path.replace('\\', "/");

        // Fix doubled slashes
        while normalized.contains("//") {
            normalized = normalized.replace("//", "/");
        }

        // Remove trailing separators
        let trimmed = normalized.trim_end_matches('/');

        // Detect and fix potentially malformed paths
        let components: Vec<&str> = trimmed.split('/').collect();
        let mut suspicious = false;

        // Check for suspicious long components
        for component in &components {
            if component.len() > 20 && !component.contains('.') {
                log_info!(&format!(
                "WARNING: Long path component ({} chars) may indicate missing separator: {}",
                component.len(), component));
                suspicious = true;
                break;
            }
        }

        // For suspicious paths, we'll try to infer separators
        if suspicious {
            // Use heuristics to detect common patterns that indicate missing separators
            // This is non-specific and works for any path structure
            let mut fixed = String::new();
            let mut last_slash = true;

            for (i, c) in trimmed.chars().enumerate() {
                // Add the character
                fixed.push(c);

                // Check for potential missing separators
                if !last_slash && c != '/' {
                    // If we see a transition from lowercase to uppercase, it might
                    // indicate a new component
                    if i > 0 && i < trimmed.len() - 1 {
                        let prev = trimmed.chars().nth(i - 1).unwrap();
                        if prev.is_lowercase() && c.is_uppercase() {
                            fixed.insert(fixed.len() - 1, '/');
                            last_slash = true;
                            continue;
                        }
                    }
                }

                last_slash = c == '/';
            }

            return fixed;
        }

        trimmed.to_string()
    }
}

impl Node {
    /// Create a new Node4
    fn new_node4() -> Self {
        Node::Node4 {
            is_terminal: false,
            score: 0.0,
            last_accessed: Instant::now(),
            prefix: Vec::new(),
            keys: Vec::with_capacity(4),
            children: Vec::with_capacity(4),
        }
    }

    /// Create a path node - stack-friendly version
    fn create_path(key: &[u8], depth: usize, score: f32) -> Node {
        let mut node = Node::new_node4();
        node.set_terminal(true);
        node.set_score(score);

        // Add the remaining key as prefix (avoiding recursion)
        if depth < key.len() {
            node.add_to_prefix(&key[depth..]);
        }

        node
    }

    /// Check if this node is a terminal node (complete path)
    fn is_terminal(&self) -> bool {
        match self {
            Node::Node4 { is_terminal, .. } => *is_terminal,
            Node::Node16 { is_terminal, .. } => *is_terminal,
            Node::Node48 { is_terminal, .. } => *is_terminal,
            Node::Node256 { is_terminal, .. } => *is_terminal,
        }
    }

    /// Set terminal status
    fn set_terminal(&mut self, value: bool) {
        match self {
            Node::Node4 { is_terminal, .. } => *is_terminal = value,
            Node::Node16 { is_terminal, .. } => *is_terminal = value,
            Node::Node48 { is_terminal, .. } => *is_terminal = value,
            Node::Node256 { is_terminal, .. } => *is_terminal = value,
        }
    }

    /// Get the score for this node
    fn score(&self) -> f32 {
        match self {
            Node::Node4 { score, .. } => *score,
            Node::Node16 { score, .. } => *score,
            Node::Node48 { score, .. } => *score,
            Node::Node256 { score, .. } => *score,
        }
    }

    /// Set the score for this node
    fn set_score(&mut self, value: f32) {
        match self {
            Node::Node4 { score, .. } => *score = value,
            Node::Node16 { score, .. } => *score = value,
            Node::Node48 { score, .. } => *score = value,
            Node::Node256 { score, .. } => *score = value,
        }
    }

    /// Update the access time
    fn update_access_time(&mut self) {
        let now = Instant::now();
        match self {
            Node::Node4 { last_accessed, .. } => *last_accessed = now,
            Node::Node16 { last_accessed, .. } => *last_accessed = now,
            Node::Node48 { last_accessed, .. } => *last_accessed = now,
            Node::Node256 { last_accessed, .. } => *last_accessed = now,
        }
    }

    /// Get the prefix length
    fn prefix_len(&self) -> usize {
        match self {
            Node::Node4 { prefix, .. } => prefix.len(),
            Node::Node16 { prefix, .. } => prefix.len(),
            Node::Node48 { prefix, .. } => prefix.len(),
            Node::Node256 { prefix, .. } => prefix.len(),
        }
    }

    /// Get the prefix
    fn get_prefix(&self) -> &[u8] {
        match self {
            Node::Node4 { prefix, .. } => prefix,
            Node::Node16 { prefix, .. } => prefix,
            Node::Node48 { prefix, .. } => prefix,
            Node::Node256 { prefix, .. } => prefix,
        }
    }

    /// Add to the prefix
    fn add_to_prefix(&mut self, bytes: &[u8]) {
        match self {
            Node::Node4 { prefix, .. } => prefix.extend_from_slice(bytes),
            Node::Node16 { prefix, .. } => prefix.extend_from_slice(bytes),
            Node::Node48 { prefix, .. } => prefix.extend_from_slice(bytes),
            Node::Node256 { prefix, .. } => prefix.extend_from_slice(bytes),
        }
    }

    /// Check prefix match with proper handling of path component splits
    fn check_prefix(&self, key: &[u8], depth: usize) -> (usize, bool) {
        let prefix = self.get_prefix();

        let remaining_key_len = key.len().saturating_sub(depth);
        let max_cmp = cmp::min(prefix.len(), key.len() - depth);

        // Log the actual prefix being compared
        log_info!(&format!("check_prefix: comparing prefix={:?} with key={:?} at depth={}",
     String::from_utf8_lossy(prefix),
     String::from_utf8_lossy(&key[depth..]),
     depth));

        // Compare bytes
        for i in 0..max_cmp {
            if prefix[i] != key[depth + i] {
                // Mismatch at position i
                log_info!(&format!("Prefix mismatch at position {} (key={}, prefix={})",
                 i,
                 key[depth + i] as char,
                 prefix[i] as char));

                // Determine if this mismatch is within a path component
                let in_path_component = i > 0 && prefix[i-1] != b'/' && key[depth+i-1] != b'/';
                log_info!(&format!("Mismatch within path component: {}", in_path_component));

                return (i, false);
            }
        }

        // For prefix search, we need to check if the remainder is compatible
        let prefix_consumed = max_cmp == prefix.len();
        let key_consumed = max_cmp == remaining_key_len;

        // CRITICAL: For path component splits, check character compatibility
        (max_cmp, prefix_consumed || key_consumed)
    }

    /// Get a reference to a child node (non-mutable)
    pub fn get_child(&self, byte: u8) -> Option<&Box<Node>> {
        match self {
            Node::Node4 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    children.get(pos).and_then(|c| c.as_ref())
                } else {
                    None
                }
            },
            Node::Node16 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    children.get(pos).and_then(|c| c.as_ref())
                } else {
                    None
                }
            },
            Node::Node48 { indices, children, .. } => {
                if let Some(pos) = indices[byte as usize] {
                    children.get(pos).and_then(|c| c.as_ref())
                } else {
                    None
                }
            },
            Node::Node256 { children, .. } => {
                children[byte as usize].as_ref()
            }
        }
    }

    /// Split the prefix at the given position with path awareness for all node types
    fn split_prefix(&mut self, pos: usize) {
        log_info!(&format!("Splitting prefix at position {}", pos));

        if pos == 0 {
            // No shared prefix
            return;
        }

        match self {
            Node::Node4 { prefix, .. } => {
                if pos < prefix.len() {
                    // CRITICAL FIX: Correctly handle the split character
                    let remaining = prefix.split_off(pos);

                    // When splitting a path component, don't include the split character
                    // in either the parent or child prefix
                    log_info!(&format!("Split prefix: old={:?}, new={:?}, remaining={:?}",
                    String::from_utf8_lossy(prefix),
                    String::from_utf8_lossy(&prefix[0..pos]),
                    String::from_utf8_lossy(&remaining)));

                    *prefix = prefix[0..pos].to_vec();
                } else {
                    log_info!("No split needed - pos >= prefix length");
                }
            },
            Node::Node16 { prefix, .. } => {
                if pos < prefix.len() {
                    // Apply the same logic as Node4
                    let remaining = prefix.split_off(pos);

                    log_info!(&format!("Split prefix (Node16): old={:?}, new={:?}, remaining={:?}",
                    String::from_utf8_lossy(prefix),
                    String::from_utf8_lossy(&prefix[0..pos]),
                    String::from_utf8_lossy(&remaining)));

                    *prefix = prefix[0..pos].to_vec();
                } else {
                    log_info!("No split needed for Node16 - pos >= prefix length");
                }
            },
            Node::Node48 { prefix, .. } => {
                if pos < prefix.len() {
                    // Apply the same logic as Node4
                    let remaining = prefix.split_off(pos);

                    log_info!(&format!("Split prefix (Node48): old={:?}, new={:?}, remaining={:?}",
                    String::from_utf8_lossy(prefix),
                    String::from_utf8_lossy(&prefix[0..pos]),
                    String::from_utf8_lossy(&remaining)));

                    *prefix = prefix[0..pos].to_vec();
                } else {
                    log_info!("No split needed for Node48 - pos >= prefix length");
                }
            },
            Node::Node256 { prefix, .. } => {
                if pos < prefix.len() {
                    // Apply the same logic as Node4
                    let remaining = prefix.split_off(pos);

                    log_info!(&format!("Split prefix (Node256): old={:?}, new={:?}, remaining={:?}",
                    String::from_utf8_lossy(prefix),
                    String::from_utf8_lossy(&prefix[0..pos]),
                    String::from_utf8_lossy(&remaining)));

                    *prefix = prefix[0..pos].to_vec();
                } else {
                    log_info!("No split needed for Node256 - pos >= prefix length");
                }
            }
        }
    }

    /// Check if the node has a child for the given byte
    fn has_child(&self, byte: u8) -> bool {
        match self {
            Node::Node4 { keys, children, .. } => {
                keys.iter().position(|&k| k == byte).map_or(false, |i| children[i].is_some())
            }
            Node::Node16 { keys, children, .. } => {
                keys.iter().position(|&k| k == byte).map_or(false, |i| children[i].is_some())
            }
            Node::Node48 { indices, children, .. } => {
                indices[byte as usize].map_or(false, |i| children[i].is_some())
            }
            Node::Node256 { children, .. } => {
                children[byte as usize].is_some()
            }
        }
    }

    /// Get a mutable reference to the child for the given byte
    fn get_child_mut(&mut self, byte: u8) -> Option<&mut Box<Node>> {
        match self {
            Node::Node4 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    children.get_mut(pos).and_then(|c| c.as_mut())
                } else {
                    None
                }
            }
            Node::Node16 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    children.get_mut(pos).and_then(|c| c.as_mut())
                } else {
                    None
                }
            }
            Node::Node48 { indices, children, .. } => {
                if let Some(pos) = indices[byte as usize] {
                    children.get_mut(pos).and_then(|c| c.as_mut())
                } else {
                    None
                }
            }
            Node::Node256 { children, .. } => {
                children[byte as usize].as_mut()
            }
        }
    }

    /// Add a child node
    fn add_child(&mut self, byte: u8, child: Box<Node>) {
        match self {
            Node::Node4 { keys, children, .. } => {
                if keys.len() < 4 {
                    // Find insertion position (sorted)
                    let pos = keys.binary_search(&byte).unwrap_or_else(|e| e);
                    keys.insert(pos, byte);
                    children.insert(pos, Some(child));
                } else {
                    // Need to grow to Node16
                    *self = self.grow_to_node16();
                    self.add_child(byte, child);
                }
            }
            Node::Node16 { keys, children, .. } => {
                if keys.len() < 16 {
                    // Find insertion position (sorted)
                    let pos = keys.binary_search(&byte).unwrap_or_else(|e| e);
                    keys.insert(pos, byte);
                    children.insert(pos, Some(child));
                } else {
                    // Need to grow to Node48
                    *self = self.grow_to_node48();
                    self.add_child(byte, child);
                }
            }
            Node::Node48 { indices, children, .. } => {
                if children.len() < 48 {
                    // Add to the end
                    let pos = children.len();
                    indices[byte as usize] = Some(pos);
                    children.push(Some(child));
                } else {
                    // Need to grow to Node256
                    *self = self.grow_to_node256();
                    self.add_child(byte, child);
                }
            }
            Node::Node256 { children, .. } => {
                children[byte as usize] = Some(child);
            }
        }
    }

    /// Remove a child node
    fn remove_child(&mut self, byte: u8) -> bool {
        let removed = match self {
            Node::Node4 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    if children[pos].is_some() {
                        children[pos] = None;
                        // Don't remove the key for simplicity
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Node::Node16 { keys, children, .. } => {
                if let Some(pos) = keys.iter().position(|&k| k == byte) {
                    if children[pos].is_some() {
                        children[pos] = None;
                        // Don't remove the key for simplicity
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Node::Node48 { indices, children, .. } => {
                if let Some(pos) = indices[byte as usize] {
                    if children[pos].is_some() {
                        children[pos] = None;
                        // Keep the index for simplicity
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Node::Node256 { children, .. } => {
                if children[byte as usize].is_some() {
                    children[byte as usize] = None;
                    true
                } else {
                    false
                }
            }
        };

        if removed {
            // Check if we need to shrink the node
            self.shrink_if_needed();
        }

        removed
    }

    /// Get the number of children
    fn child_count(&self) -> usize {
        match self {
            Node::Node4 { children, .. } => children.iter().filter(|c| c.is_some()).count(),
            Node::Node16 { children, .. } => children.iter().filter(|c| c.is_some()).count(),
            Node::Node48 { children, .. } => children.iter().filter(|c| c.is_some()).count(),
            Node::Node256 { children, .. } => children.iter().filter(|c| c.is_some()).count(),
        }
    }

    /// Grow a Node4 to Node16
    fn grow_to_node16(&self) -> Self {
        match self {
            Node::Node4 { is_terminal, score, last_accessed, prefix, keys, children } => {
                let mut new_keys = Vec::with_capacity(16);
                let mut new_children = Vec::with_capacity(16);

                // Copy sorted keys and children
                for i in 0..keys.len() {
                    new_keys.push(keys[i]);
                    new_children.push(children[i].clone());
                }

                Node::Node16 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    keys: new_keys,
                    children: new_children,
                }
            }
            _ => panic!("Cannot grow non-Node4 to Node16"),
        }
    }

    /// Grow a Node16 to Node48
    fn grow_to_node48(&self) -> Self {
        match self {
            Node::Node16 { is_terminal, score, last_accessed, prefix, keys, children } => {
                let mut new_indices = [None; 256];
                let mut new_children = Vec::with_capacity(48);

                // Copy children and set up indices
                for i in 0..keys.len() {
                    let pos = new_children.len();
                    new_indices[keys[i] as usize] = Some(pos);
                    new_children.push(children[i].clone());
                }

                Node::Node48 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    indices: new_indices,
                    children: new_children,
                }
            }
            _ => panic!("Cannot grow non-Node16 to Node48"),
        }
    }

    /// Grow a Node48 to Node256
    fn grow_to_node256(&self) -> Self {
        match self {
            Node::Node48 { is_terminal, score, last_accessed, prefix, indices, children } => {
                let mut new_children: [Option<Box<Node>>; 256] = std::array::from_fn(|_| None);

                // Copy children
                for (byte, &idx_opt) in indices.iter().enumerate() {
                    if let Some(idx) = idx_opt {
                        if let Some(child) = &children[idx] {
                            new_children[byte] = Some(child.clone());
                        }
                    }
                }

                Node::Node256 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    children: new_children,
                }
            }
            _ => panic!("Cannot grow non-Node48 to Node256"),
        }
    }

    /// Check if the node should be shrunk after child removal
    fn should_shrink(&self) -> bool {
        match self {
            // Node4 is already the smallest type
            Node::Node4 { .. } => false,

            // Shrink Node16 if it has 3 or fewer children
            Node::Node16 { children, .. } => {
                children.iter().filter(|c| c.is_some()).count() <= 3
            },

            // Shrink Node48 if it has 10 or fewer children
            Node::Node48 { children, .. } => {
                children.iter().filter(|c| c.is_some()).count() <= 10
            },

            // Shrink Node256 if it has 40 or fewer children
            Node::Node256 { children, .. } => {
                children.iter().filter(|c| c.is_some()).count() <= 40
            }
        }
    }

    /// Shrink the node to a more appropriate size if needed
    fn shrink_if_needed(&mut self) {
        if !self.should_shrink() {
            return;
        }

        match self {
            Node::Node4 { .. } => {
                // Already the smallest type
            },
            Node::Node16 { .. } => {
                *self = self.shrink_to_node4();
            },
            Node::Node48 { .. } => {
                *self = self.shrink_to_node16();
            },
            Node::Node256 { .. } => {
                *self = self.shrink_to_node48();
            }
        }
    }

    /// Shrink a Node16 to Node4
    fn shrink_to_node4(&self) -> Self {
        match self {
            Node::Node16 { is_terminal, score, last_accessed, prefix, keys, children } => {
                let mut new_keys = Vec::with_capacity(4);
                let mut new_children = Vec::with_capacity(4);

                // Copy only the keys and children that actually exist
                for i in 0..keys.len() {
                    if children[i].is_some() {
                        new_keys.push(keys[i]);
                        new_children.push(children[i].clone());

                        if new_keys.len() >= 4 {
                            break;
                        }
                    }
                }

                Node::Node4 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    keys: new_keys,
                    children: new_children,
                }
            }
            _ => panic!("Cannot shrink non-Node16 to Node4"),
        }
    }

    /// Shrink a Node48 to Node16
    fn shrink_to_node16(&self) -> Self {
        match self {
            Node::Node48 { is_terminal, score, last_accessed, prefix, indices, children } => {
                let mut new_keys = Vec::with_capacity(16);
                let mut new_children = Vec::with_capacity(16);

                // Find and copy valid children
                for (byte, &idx_opt) in indices.iter().enumerate() {
                    if let Some(idx) = idx_opt {
                        if let Some(child) = &children[idx] {
                            new_keys.push(byte as u8);
                            new_children.push(Some(child.clone()));

                            if new_keys.len() >= 16 {
                                break;
                            }
                        }
                    }
                }

                Node::Node16 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    keys: new_keys,
                    children: new_children,
                }
            }
            _ => panic!("Cannot shrink non-Node48 to Node16"),
        }
    }

    /// Shrink a Node256 to Node48
    fn shrink_to_node48(&self) -> Self {
        match self {
            Node::Node256 { is_terminal, score, last_accessed, prefix, children } => {
                let mut new_indices = [None; 256];
                let mut new_children = Vec::with_capacity(48);

                // Find and copy valid children
                for (byte, child_opt) in children.iter().enumerate() {
                    if let Some(child) = child_opt {
                        let pos = new_children.len();
                        new_indices[byte] = Some(pos);
                        new_children.push(Some(child.clone()));

                        if new_children.len() >= 48 {
                            break;
                        }
                    }
                }

                Node::Node48 {
                    is_terminal: *is_terminal,
                    score: *score,
                    last_accessed: *last_accessed,
                    prefix: prefix.clone(),
                    indices: new_indices,
                    children: new_children,
                }
            }
            _ => panic!("Cannot shrink non-Node256 to Node48"),
        }
    }
}

#[cfg(test)]
mod tests_art_v2 {
    use super::*;
    use std::time::{Duration, Instant};
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

    /// Normalize paths without hardcoding specific patterns
    fn normalize_path(path: &str) -> String {
        // Handle platform-specific separators
        let mut normalized = path.replace('\\', "/");

        // Fix doubled slashes
        while normalized.contains("//") {
            normalized = normalized.replace("//", "/");
        }

        // Remove trailing separators
        let trimmed = normalized.trim_end_matches('/');

        // Detect and fix potentially malformed paths
        let components: Vec<&str> = trimmed.split('/').collect();
        let mut suspicious = false;

        // Check for suspicious long components
        for component in &components {
            if component.len() > 20 && !component.contains('.') {
                log_info!(&format!(
                "WARNING: Long path component ({} chars) may indicate missing separator: {}",
                component.len(), component));
                suspicious = true;
                break;
            }
        }

        // For suspicious paths, we'll try to infer separators
        if suspicious {
            // Use heuristics to detect common patterns that indicate missing separators
            // This is non-specific and works for any path structure
            let mut fixed = String::new();
            let mut last_slash = true;

            for (i, c) in trimmed.chars().enumerate() {
                // Add the character
                fixed.push(c);

                // Check for potential missing separators
                if !last_slash && c != '/' {
                    // If we see a transition from lowercase to uppercase, it might
                    // indicate a new component
                    if i > 0 && i < trimmed.len() - 1 {
                        let prev = trimmed.chars().nth(i - 1).unwrap();
                        if prev.is_lowercase() && c.is_uppercase() {
                            fixed.insert(fixed.len() - 1, '/');
                            last_slash = true;
                            continue;
                        }
                    }
                }

                last_slash = c == '/';
            }

            return fixed;
        }

        trimmed.to_string()
    }

    // Helper function to measure and log execution time
    fn measure_time<F, T>(name: &str, f: F) -> T
    where
        F: FnOnce() -> T
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        log_info!(&format!("Time for {}: {:?}", name, elapsed));
        result
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
        let mut trie = ART::new(5);

        assert_eq!(trie.len(), 0);
        assert!(trie.is_empty());

        let completions = trie.find_completions("anything");
        assert_eq!(completions.len(), 0);
        log_info!("Empty trie returns empty completions as expected");
    }

    #[test]
    fn test_complete_filenames() {
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
    fn test_large_scale_performance() {
        log_info!("Testing large-scale trie performance with real paths");
        let mut trie = ART::new(100);

        // Get all available test paths
        let paths = collect_test_paths(None);
        let path_count = paths.len();
        log_info!(&format!("Collected {} test paths", path_count));

        // Measure insertion time
        let insert_time = measure_time("inserting all paths", || {
            for (i, path) in paths.iter().enumerate() {
                trie.insert(path, 1.0 - (i as f32 * 0.0001));
            }
        });

        assert_eq!(trie.len(), path_count);

        // Extract some test prefixes from the actual data
        let prefixes_to_test: Vec<String> = if !paths.is_empty() {
            let mut prefixes = Vec::new();
            
            // Use parts of the first few paths as test prefixes
            for path in paths.iter().take(4) {
                if let Some(last_sep) = path.rfind(MAIN_SEPARATOR) {
                    if last_sep > 0 {
                        // Use the directory portion
                        prefixes.push(path[0..last_sep+1].to_string());
                        
                        // Also test with the start of the filename
                        if path.len() > last_sep + 2 {
                            prefixes.push(format!("{}{}", &path[0..last_sep+1], &path[last_sep+1..last_sep+2]));
                        }
                    }
                }
            }
            
            if prefixes.is_empty() {
                vec![
                    normalize_path("/t"),
                    normalize_path("/test"),
                    normalize_path("/home/"),
                    normalize_path("/usr/bin")
                ]
            } else {
                prefixes
            }
        } else {
            vec![
                normalize_path("/t"),
                normalize_path("/test"),
                normalize_path("/home/"),
                normalize_path("/usr/bin")
            ]
        };

        // Measure search time for each prefix
        for prefix in prefixes_to_test {
            let completions = measure_time(&format!("searching prefix '{}'", prefix), || {
                trie.find_completions(&prefix)
            });

            log_info!(&format!("Found {} completions for prefix '{}'",
                completions.len(), prefix));
        }

        // Measure removal time for a subset of paths
        let remove_time = measure_time("removing paths", || {
            for i in 0..paths.len().min(1000) {
                trie.remove(&paths[i]);
            }
        });

        assert_eq!(trie.len(), path_count - paths.len().min(1000));
        log_info!(&format!("Trie contains {} paths after removals", trie.len()));
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
    fn test_with_real_world_data() {
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
            
            for (prefix, count) in prefix_counts.into_iter().take(5) {
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
        for prefix in test_prefixes {
            let start = Instant::now();
            let completions = trie.find_completions(&prefix);
            let elapsed = start.elapsed();
            
            log_info!(&format!("Found {} completions for prefix '{}' in {:?}",
                      completions.len(), prefix, elapsed));
            
            if !completions.is_empty() {
                log_info!(&format!("First result: {} (score: {:.2})", 
                          completions[0].0, completions[0].1));

                // Verify that results actually match the prefix
                let valid_matches = completions.iter()
                    .filter(|(path, _)| path.starts_with(&prefix))
                    .count();

                log_info!(&format!("{} of {} results are valid prefix matches for '{}'",
                          valid_matches, completions.len(), prefix));

                assert!(valid_matches > 0, "No valid matches found for prefix '{}'", prefix);
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
}
