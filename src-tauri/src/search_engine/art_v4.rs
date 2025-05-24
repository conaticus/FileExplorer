use smallvec::SmallVec;
use std::cmp;
use std::mem;
use crate::{log_error, log_info};

pub struct ART {
    root: Option<Box<ARTNode>>,
    path_count: usize,
    max_results: usize,
}

// Constants for different node types
const NODE4_MAX: usize = 4;
const NODE16_MAX: usize = 16;
const NODE48_MAX: usize = 48;
const NODE256_MAX: usize = 256;
type KeyType = u8;

type Prefix = SmallVec<[KeyType; 8]>;

enum ARTNode {
    Node4(Node4),
    Node16(Node16),
    Node48(Node48),
    Node256(Node256),
}

impl ARTNode {
    fn new_node4() -> Self {
        ARTNode::Node4(Node4::new())
    }

    // Common properties for all node types
    fn is_terminal(&self) -> bool {
        match self {
            ARTNode::Node4(n) => n.is_terminal,
            ARTNode::Node16(n) => n.is_terminal,
            ARTNode::Node48(n) => n.is_terminal,
            ARTNode::Node256(n) => n.is_terminal,
        }
    }

    fn set_terminal(&mut self, value: bool) {
        match self {
            ARTNode::Node4(n) => n.is_terminal = value,
            ARTNode::Node16(n) => n.is_terminal = value,
            ARTNode::Node48(n) => n.is_terminal = value,
            ARTNode::Node256(n) => n.is_terminal = value,
        }
    }

    fn get_score(&self) -> Option<f32> {
        match self {
            ARTNode::Node4(n) => n.score,
            ARTNode::Node16(n) => n.score,
            ARTNode::Node48(n) => n.score,
            ARTNode::Node256(n) => n.score,
        }
    }

    fn set_score(&mut self, score: Option<f32>) {
        match self {
            ARTNode::Node4(n) => n.score = score,
            ARTNode::Node16(n) => n.score = score,
            ARTNode::Node48(n) => n.score = score,
            ARTNode::Node256(n) => n.score = score,
        }
    }

    fn get_prefix(&self) -> &[KeyType] {
        match self {
            ARTNode::Node4(n) => &n.prefix,
            ARTNode::Node16(n) => &n.prefix,
            ARTNode::Node48(n) => &n.prefix,
            ARTNode::Node256(n) => &n.prefix,
        }
    }

    fn get_prefix_mut(&mut self) -> &mut Prefix {
        match self {
            ARTNode::Node4(n) => &mut n.prefix,
            ARTNode::Node16(n) => &mut n.prefix,
            ARTNode::Node48(n) => &mut n.prefix,
            ARTNode::Node256(n) => &mut n.prefix,
        }
    }

    // Check for prefix match and return length of match
    fn check_prefix(&self, key: &[KeyType], depth: usize) -> (usize, bool) {
        let prefix = self.get_prefix();

        if prefix.is_empty() {
            return (0, true);
        }

        let max_len = cmp::min(prefix.len(), key.len() - depth);
        let mut i = 0;

        // Compare prefix bytes
        while i < max_len && prefix[i] == key[depth + i] {
            i += 1;
        }

        (i, i == prefix.len())
    }

    fn split_prefix(&mut self, mismatch_pos: usize) {
        let old_prefix = self.get_prefix().to_vec();

        if mismatch_pos == 0 {
            return;
        }

        // The common prefix stays in this node
        let mut common_prefix: SmallVec<[KeyType; 8]> =
            old_prefix[..mismatch_pos].iter().copied().collect();
        mem::swap(self.get_prefix_mut(), &mut common_prefix);

        // The rest of the prefix (after mismatch_pos) goes to the new node
        let mut new_node = ARTNode::new_node4();

        // If there is a remaining prefix, assign it to the new node
        if mismatch_pos < old_prefix.len() {
            *new_node.get_prefix_mut() = old_prefix[mismatch_pos..].iter().copied().collect();
        }

        // Move terminal status and score to the new node
        new_node.set_terminal(self.is_terminal());
        new_node.set_score(self.get_score());
        self.set_terminal(false);
        self.set_score(None);

        // Move all children from current node to new node
        match self {
            ARTNode::Node4(n) => match &mut new_node {
                ARTNode::Node4(new_n) => {
                    mem::swap(&mut n.children, &mut new_n.children);
                    mem::swap(&mut n.keys, &mut new_n.keys);
                }
                _ => unreachable!(),
            },
            ARTNode::Node16(n) => {
                if let ARTNode::Node4(new_n) = &mut new_node {
                    for i in 0..n.keys.len() {
                        if n.children[i].is_some() {
                            let child = mem::replace(&mut n.children[i], None);
                            new_n.add_child(n.keys[i], child);
                        }
                    }
                }
            }
            ARTNode::Node48(n) => {
                if let ARTNode::Node4(new_n) = &mut new_node {
                    for i in 0..256 {
                        if let Some(idx) = n.child_index[i] {
                            if n.children[idx as usize].is_some() {
                                let child = mem::replace(&mut n.children[idx as usize], None);
                                new_n.add_child(i as u8, child);
                            }
                        }
                    }
                    n.child_index = [None; 256];
                    n.size = 0;
                }
            }
            ARTNode::Node256(n) => {
                if let ARTNode::Node4(new_n) = &mut new_node {
                    for i in 0..256 {
                        if n.children[i].is_some() {
                            let child = mem::replace(&mut n.children[i], None);
                            new_n.add_child(i as u8, child);
                        }
                    }
                    n.size = 0;
                }
            }
        }

        // The first byte of the new node's prefix is the key for the child
        let split_char = if !new_node.get_prefix().is_empty() {
            new_node.get_prefix()[0]
        } else {
            // should not happen
            0
        };

        // Remove the first byte from the new node's prefix (since it's now the key)
        if !new_node.get_prefix().is_empty() {
            let mut prefix: SmallVec<[KeyType; 8]> =
                new_node.get_prefix().iter().copied().collect();
            prefix.remove(0);
            *new_node.get_prefix_mut() = prefix;
        }

        // Remove all children from self (already moved above)
        // Add the new node as a child under the split character
        self.add_child(split_char, Some(Box::new(new_node)));
    }

    // Add a child or replace it if already exists, with node growth
    fn add_child(&mut self, key: KeyType, mut child: Option<Box<ARTNode>>) -> bool {
        let mut grown = false;
        let added = match self {
            ARTNode::Node4(n) => {
                let added = n.add_child(key, child.take());
                if !added && n.keys.len() >= NODE4_MAX {
                    // Grow to Node16
                    let grown_node = self.grow();
                    grown = true;
                    *self = grown_node;
                    let added = self.add_child(key, child);
                    added
                } else {
                    added
                }
            }
            ARTNode::Node16(n) => {
                let added = n.add_child(key, child.take());
                if !added && n.keys.len() >= NODE16_MAX {
                    // Grow to Node48
                    let grown_node = self.grow();
                    grown = true;
                    *self = grown_node;
                    let added = self.add_child(key, child);
                    added
                } else {
                    added
                }
            }
            ARTNode::Node48(n) => {
                let added = n.add_child(key, child.take());
                if !added && n.size >= NODE48_MAX {
                    // Grow to Node256
                    let grown_node = self.grow();
                    grown = true;
                    *self = grown_node;
                    let added = self.add_child(key, child);
                    added
                } else {
                    added
                }
            }
            ARTNode::Node256(n) => n.add_child(key, child),
        };
        added || grown
    }

    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        match self {
            ARTNode::Node4(n) => n.find_child(key),
            ARTNode::Node16(n) => n.find_child(key),
            ARTNode::Node48(n) => n.find_child(key),
            ARTNode::Node256(n) => n.find_child(key),
        }
    }

    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        match self {
            ARTNode::Node4(n) => n.find_child_mut(key),
            ARTNode::Node16(n) => n.find_child_mut(key),
            ARTNode::Node48(n) => n.find_child_mut(key),
            ARTNode::Node256(n) => n.find_child_mut(key),
        }
    }

    // Remove a child by key, with node shrinking
    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        let removed = match self {
            ARTNode::Node4(n) => n.remove_child(key),
            ARTNode::Node16(n) => {
                let removed = n.remove_child(key);
                if n.keys.len() < NODE4_MAX / 2 {
                    // Shrink to Node4
                    let shrunk = self.shrink();
                    *self = shrunk;
                }
                removed
            }
            ARTNode::Node48(n) => {
                let removed = n.remove_child(key);
                if n.size < NODE16_MAX / 2 {
                    // Shrink to Node16
                    let shrunk = self.shrink();
                    *self = shrunk;
                }
                removed
            }
            ARTNode::Node256(n) => {
                let removed = n.remove_child(key);
                if n.size < NODE48_MAX / 2 {
                    // Shrink to Node48
                    let shrunk = self.shrink();
                    *self = shrunk;
                }
                removed
            }
        };
        removed
    }

    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        match self {
            ARTNode::Node4(n) => n.iter_children(),
            ARTNode::Node16(n) => n.iter_children(),
            ARTNode::Node48(n) => n.iter_children(),
            ARTNode::Node256(n) => n.iter_children(),
        }
    }

    fn num_children(&self) -> usize {
        match self {
            ARTNode::Node4(n) => n.keys.len(),
            ARTNode::Node16(n) => n.keys.len(),
            ARTNode::Node48(n) => n.size,
            ARTNode::Node256(n) => n.size,
        }
    }

    // Grow to a larger node type
    fn grow(&mut self) -> Self {
        match self {
            ARTNode::Node4(n) => {
                let mut n16 = Node16::new();
                n16.prefix = std::mem::take(&mut n.prefix);
                n16.is_terminal = n.is_terminal;
                n16.score = n.score;
                for i in 0..n.keys.len() {
                    n16.keys.push(n.keys[i]);
                    n16.children.push(n.children[i].take());
                }
                ARTNode::Node16(n16)
            }
            ARTNode::Node16(n) => {
                let mut n48 = Node48::new();
                n48.prefix = std::mem::take(&mut n.prefix);
                n48.is_terminal = n.is_terminal;
                n48.score = n.score;
                let mut child_count = 0;
                for i in 0..n.keys.len() {
                    if let Some(child) = n.children[i].take() {
                        let key = n.keys[i] as usize;
                        n48.children[child_count] = Some(child);
                        n48.child_index[key] = Some(child_count as u8);
                        child_count += 1;
                    }
                }
                n48.size = child_count;
                ARTNode::Node48(n48)
            }
            ARTNode::Node48(n) => {
                let mut n256 = Node256::new();
                n256.prefix = std::mem::take(&mut n.prefix);
                n256.is_terminal = n.is_terminal;
                n256.score = n.score;
                for i in 0..256 {
                    if let Some(idx) = n.child_index[i] {
                        if let Some(child) = n.children[idx as usize].take() {
                            n256.children[i] = Some(child);
                        }
                    }
                }
                n256.size = n.size;
                ARTNode::Node256(n256)
            }
            ARTNode::Node256(_) => {
                log_error!("Node256 cannot be grown further");
                panic!("Node256 cannot be grown further");
            }
        }
    }

    // Shrink to a smaller node type
    fn shrink(&mut self) -> Self {
        match self {
            ARTNode::Node16(n) => {
                let mut n4 = Node4::new();
                n4.prefix = std::mem::take(&mut n.prefix);
                n4.is_terminal = n.is_terminal;
                n4.score = n.score;
                for i in 0..n.keys.len().min(NODE4_MAX) {
                    n4.keys.push(n.keys[i]);
                    n4.children.push(n.children[i].take());
                }
                ARTNode::Node4(n4)
            }
            ARTNode::Node48(n) => {
                let mut n16 = Node16::new();
                n16.prefix = std::mem::take(&mut n.prefix);
                n16.is_terminal = n.is_terminal;
                n16.score = n.score;
                let mut count = 0;
                for i in 0..256 {
                    if count >= NODE16_MAX {
                        break;
                    }
                    if let Some(idx) = n.child_index[i] {
                        if let Some(child) = n.children[idx as usize].take() {
                            n16.keys.push(i as KeyType);
                            n16.children.push(Some(child));
                            count += 1;
                        }
                    }
                }
                ARTNode::Node16(n16)
            }
            ARTNode::Node256(n) => {
                let mut n48 = Node48::new();
                n48.prefix = std::mem::take(&mut n.prefix);
                n48.is_terminal = n.is_terminal;
                n48.score = n.score;
                let mut count = 0;
                for i in 0..256 {
                    if count >= NODE48_MAX {
                        break;
                    }
                    if let Some(child) = n.children[i].take() {
                        n48.children[count] = Some(child);
                        n48.child_index[i] = Some(count as u8);
                        count += 1;
                    }
                }
                n48.size = count;
                ARTNode::Node48(n48)
            }
            _ => {
                log_error!("Cannot shrink node smaller than Node4");
                panic!("Cannot shrink node smaller than Node4");
            }
        }
    }
}

impl Clone for ARTNode {
    fn clone(&self) -> Self {
        match self {
            ARTNode::Node4(n) => ARTNode::Node4(n.clone()),
            ARTNode::Node16(n) => ARTNode::Node16(n.clone()),
            ARTNode::Node48(n) => ARTNode::Node48(n.clone()),
            ARTNode::Node256(n) => ARTNode::Node256(n.clone()),
        }
    }
}

// ------------------ Specific Node Implementations ------------------

// Node4: Stores up to 4 children in a small array
#[derive(Clone)]
struct Node4 {
    prefix: Prefix,
    is_terminal: bool,
    score: Option<f32>,
    keys: SmallVec<[KeyType; NODE4_MAX]>,
    children: SmallVec<[Option<Box<ARTNode>>; NODE4_MAX]>,
}

struct Node16 {
    prefix: Prefix,
    is_terminal: bool,
    score: Option<f32>,
    keys: SmallVec<[KeyType; NODE16_MAX]>,
    children: SmallVec<[Option<Box<ARTNode>>; NODE16_MAX]>,
}

// Only Node48 and Node256 have a size field
struct Node48 {
    prefix: Prefix,
    is_terminal: bool,
    score: Option<f32>,
    child_index: [Option<u8>; 256],
    children: Box<[Option<Box<ARTNode>>]>, // 48 slots
    size: usize,
}

struct Node256 {
    prefix: Prefix,
    is_terminal: bool,
    score: Option<f32>,
    children: Box<[Option<Box<ARTNode>>]>, // 256 slots
    size: usize,
}

// --- Node4/Node16 implementations ---
impl Node4 {
    fn new() -> Self {
        Node4 {
            prefix: SmallVec::new(),
            is_terminal: false,
            score: None,
            keys: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                self.children[i] = child;
                return true;
            }
        }

        if self.keys.len() >= NODE4_MAX {
            return false;
        }

        let mut i = self.keys.len();
        while i > 0 && self.keys[i - 1] > key {
            i -= 1;
        }

        self.keys.insert(i, key);
        self.children.insert(i, child);
        true
    }

    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                return self.children[i].as_ref();
            }
        }
        None
    }

    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                return Some(&mut self.children[i]);
            }
        }
        None
    }

    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                let removed = self.children.remove(i);
                self.keys.remove(i);
                return removed;
            }
        }
        None
    }

    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.keys.len());
        for i in 0..self.keys.len() {
            if let Some(child) = &self.children[i] {
                result.push((self.keys[i], child));
            }
        }
        result
    }
}

impl Node16 {
    fn new() -> Self {
        Node16 {
            prefix: SmallVec::new(),
            is_terminal: false,
            score: None,
            keys: SmallVec::new(),
            children: SmallVec::new(),
        }
    }

    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                self.children[i] = child;
                return true;
            }
        }

        if self.keys.len() >= NODE16_MAX {
            return false;
        }

        let mut i = self.keys.len();
        while i > 0 && self.keys[i - 1] > key {
            i -= 1;
        }

        self.keys.insert(i, key);
        self.children.insert(i, child);
        true
    }

    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                return self.children[i].as_ref();
            }
        }
        None
    }

    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                return Some(&mut self.children[i]);
            }
        }
        None
    }

    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        for i in 0..self.keys.len() {
            if self.keys[i] == key {
                let removed = self.children.remove(i);
                self.keys.remove(i);
                return removed;
            }
        }
        None
    }

    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.keys.len());
        for i in 0..self.keys.len() {
            if let Some(child) = &self.children[i] {
                result.push((self.keys[i], child));
            }
        }
        result
    }
}

impl Node48 {
    fn new() -> Self {
        Node48 {
            prefix: SmallVec::new(),
            is_terminal: false,
            score: None,
            child_index: [None; 256],
            children: vec![None; NODE48_MAX].into_boxed_slice(),
            size: 0,
        }
    }

    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        let key_idx = key as usize;

        if let Some(idx) = self.child_index[key_idx] {
            self.children[idx as usize] = child;
            return true;
        }

        if self.size >= NODE48_MAX {
            return false;
        }

        self.children[self.size] = child;
        self.child_index[key_idx] = Some(self.size as u8);
        self.size += 1;
        true
    }

    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        let key_idx = key as usize;
        if let Some(idx) = self.child_index[key_idx] {
            self.children[idx as usize].as_ref()
        } else {
            None
        }
    }

    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        let key_idx = key as usize;
        if let Some(idx) = self.child_index[key_idx] {
            Some(&mut self.children[idx as usize])
        } else {
            None
        }
    }

    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        let key_idx = key as usize;

        if let Some(idx) = self.child_index[key_idx] {
            let idx = idx as usize;
            let removed = mem::replace(&mut self.children[idx], None);

            self.child_index[key_idx] = None;

            if idx < self.size - 1 && self.size > 1 {
                for (k, &child_idx) in self.child_index.iter().enumerate() {
                    if let Some(ci) = child_idx {
                        if ci as usize == self.size - 1 {
                            self.children[idx] = self.children[self.size - 1].take();
                            self.child_index[k] = Some(idx as u8);
                            break;
                        }
                    }
                }
            }

            self.size -= 1;
            removed
        } else {
            None
        }
    }

    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..256 {
            if let Some(idx) = self.child_index[i] {
                if let Some(child) = &self.children[idx as usize] {
                    result.push((i as KeyType, child));
                }
            }
        }
        result
    }
}

impl Node256 {
    fn new() -> Self {
        Node256 {
            prefix: SmallVec::new(),
            is_terminal: false,
            score: None,
            children: vec![None; NODE256_MAX].into_boxed_slice(),
            size: 0,
        }
    }

    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        let key_idx = key as usize;
        let is_new = self.children[key_idx].is_none();

        self.children[key_idx] = child;

        if is_new {
            self.size += 1;
        }

        true
    }

    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        self.children[key as usize].as_ref()
    }

    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        Some(&mut self.children[key as usize])
    }

    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        let key_idx = key as usize;

        if self.children[key_idx].is_some() {
            let removed = mem::replace(&mut self.children[key_idx], None);
            self.size -= 1;
            removed
        } else {
            None
        }
    }

    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..256 {
            if let Some(child) = &self.children[i] {
                result.push((i as KeyType, child));
            }
        }
        result
    }
}
impl Clone for Node16 {
    fn clone(&self) -> Self {
        Node16 {
            prefix: self.prefix.clone(),
            is_terminal: self.is_terminal,
            score: self.score,
            keys: self.keys.clone(),
            children: self
                .children
                .iter()
                .map(|c| c.as_ref().map(|n| Box::new((**n).clone())))
                .collect(),
        }
    }
}
impl Clone for Node48 {
    fn clone(&self) -> Self {
        Node48 {
            prefix: self.prefix.clone(),
            is_terminal: self.is_terminal,
            score: self.score,
            child_index: self.child_index,
            children: self
                .children
                .iter()
                .map(|c| c.as_ref().map(|n| Box::new((**n).clone())))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            size: self.size,
        }
    }
}
impl Clone for Node256 {
    fn clone(&self) -> Self {
        Node256 {
            prefix: self.prefix.clone(),
            is_terminal: self.is_terminal,
            score: self.score,
            children: self
                .children
                .iter()
                .map(|c| c.as_ref().map(|n| Box::new((**n).clone())))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            size: self.size,
        }
    }
}

// ------------------ ART Implementation ------------------

impl ART {
    /// Creates a new Adaptive Radix Trie (ART) with specified maximum results limit.
    /// This trie is optimized for efficiently storing and searching file paths.
    ///
    /// # Arguments
    /// * `max_results` - The maximum number of results to return from search operations.
    ///
    /// # Returns
    /// * A new empty ART instance.
    ///
    /// # Example
    /// ```rust
    /// let trie = ART::new(100); // Create a new ART with max 100 results
    /// assert_eq!(trie.len(), 0);
    /// assert!(trie.is_empty());
    /// ```
    pub fn new(max_results: usize) -> Self {
        ART {
            root: None,
            path_count: 0,
            max_results,
        }
    }

    /// Normalizes a file path to ensure consistent representation in the trie.
    /// This function standardizes separators, removes redundant whitespace,
    /// and handles platform-specific path characteristics.
    ///
    /// # Arguments
    /// * `path` - A string slice containing the path to normalize.
    ///
    /// # Returns
    /// * A normalized String representation of the path.
    ///
    /// # Example
    /// ```rust
    /// let trie = ART::new(10);
    /// let normalized = trie.normalize_path("C:\\Users\\Documents\\ file.txt");
    /// assert_eq!(normalized, "C:/Users/Documents/file.txt");
    /// ```
    fn normalize_path(&self, path: &str) -> String {
        let mut result = String::with_capacity(path.len());
        let mut saw_slash = false;
        let mut started = false;

        let mut chars = path.chars().peekable();

        // Skip leading whitespace (including Unicode whitespace)
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&first) = chars.peek() {
            if first == '/' || first == '\\' {
                result.push('/');
                saw_slash = true;
                started = true;
                chars.next();
            }
        }

        for c in chars {
            match c {
                '/' | '\\' => {
                    if !saw_slash && started {
                        result.push('/');
                        saw_slash = true;
                    }
                }
                _ => {
                    result.push(c);
                    saw_slash = false;
                    started = true;
                }
            }
        }

        // Remove trailing slash (unless result is exactly "/")
        let len = result.len();
        if len > 1 && result.ends_with('/') {
            result.truncate(len - 1);
        }

        result
    }

    pub fn debug_print(&self) {
        if let Some(root) = &self.root {
            log_info!(&format!("ART ({} paths):", self.path_count));
            Self::debug_print_node(root.as_ref(), 0);
        } else {
            log_info!("ART is empty");
        }
    }

    fn debug_print_node(node: &ARTNode, indent: usize) {
        let pad = "  ".repeat(indent);
        // Node type
        let (node_type, prefix, is_term, score) = match node {
            ARTNode::Node4(n)   => ("Node4", &n.prefix[..], n.is_terminal, n.score),
            ARTNode::Node16(n)  => ("Node16", &n.prefix[..], n.is_terminal, n.score),
            ARTNode::Node48(n)  => ("Node48", &n.prefix[..], n.is_terminal, n.score),
            ARTNode::Node256(n) => ("Node256", &n.prefix[..], n.is_terminal, n.score),
        };
        // Decode prefix bytes as UTF-8 lossily
        let prefix_str = String::from_utf8_lossy(prefix);
        // Header line
        if is_term {
            log_info!(&format!(
                "{}{} [{}] (terminal, score={:?})",
                pad, node_type, prefix_str, score
            ));
        } else {
            log_info!(&format!("{}{} [{}]", pad, node_type, prefix_str));
        }
        // Recurse into children
        for (key, child) in node.iter_children() {
            let key_char = if key.is_ascii_graphic() {
                key as char
            } else {
                '.'  // non-printable
            };
                log_info!(&format!("{}  ├─ key={:?} ('{}') →", pad, key, key_char));
            Self::debug_print_node(child, indent + 2);
        }
    }

    /// Inserts a path into the trie with an associated score for ranking.
    /// Normalizes the path before insertion to ensure consistency.
    ///
    /// # Arguments
    /// * `path` - A string slice containing the path to insert.
    /// * `score` - A floating-point score to associate with this path (higher is better).
    ///
    /// # Returns
    /// * `true` if the path was inserted or its score was updated.
    /// * `false` if no change was made.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// assert!(trie.insert("/home/user/documents/file.txt", 1.0));
    /// assert_eq!(trie.len(), 1);
    /// ```
    pub fn insert(&mut self, path: &str, score: f32) -> bool {
        let normalized = self.normalize_path(path);
        let path_bytes = normalized.as_bytes();

        if self.root.is_none() {
            self.root = Some(Box::new(ARTNode::new_node4()));
        }

        let root = self.root.take();
        let (changed, new_path, new_root) = Self::insert_recursive(root, path_bytes, 0, score);
        self.root = new_root;

        if new_path {
            self.path_count += 1;
        }

        changed
    }

    /// Recursively inserts a path into the trie, navigating and modifying nodes as needed.
    /// This internal helper method is used by the public insert method.
    ///
    /// # Arguments
    /// * `node` - The current node in the traversal.
    /// * `key` - The byte representation of the path being inserted.
    /// * `depth` - The current depth in the key.
    /// * `score` - The score to associate with the path.
    ///
    /// # Returns
    /// * A tuple containing:
    ///   - Whether the insertion changed the trie
    ///   - Whether this is a new path
    ///   - The new node after insertion
    fn insert_recursive(
        mut node: Option<Box<ARTNode>>,
        key: &[u8],
        depth: usize,
        score: f32,
    ) -> (bool, bool, Option<Box<ARTNode>>) {
        if node.is_none() {
            node = Some(Box::new(ARTNode::new_node4()));
        }

        let mut node_ref = node.unwrap();

        // Check if we've reached the end of the path
        if depth == key.len() {
            let mut changed = false;
            let mut new_path = false;

            // If the node wasn't terminal yet
            if !node_ref.is_terminal() {
                node_ref.set_terminal(true);
                new_path = true;
                changed = true;
            }

            // If the score is different
            if node_ref.get_score() != Some(score) {
                node_ref.set_score(Some(score));
                changed = true;
            }

            return (changed, new_path, Some(node_ref));
        }

        // Check prefix match
        let (match_len, exact_match) = node_ref.check_prefix(key, depth);

        if !exact_match {
            // Prefix doesn't match - we need to split the node
            node_ref.split_prefix(match_len);
        }

        // After the prefix - position in the key
        let next_depth = depth + match_len;

        if next_depth == key.len() {
            // We've reached the end of the path - mark as terminal
            let mut changed = false;
            let mut new_path = false;

            if !node_ref.is_terminal() {
                node_ref.set_terminal(true);
                new_path = true;
                changed = true;
            }

            if node_ref.get_score() != Some(score) {
                node_ref.set_score(Some(score));
                changed = true;
            }

            return (changed, new_path, Some(node_ref));
        }

        let c = key[next_depth];
        if node_ref.find_child_mut(c).is_none() {
            node_ref.add_child(c, None);
        }

        // Process the child (need to handle the case where node might grow)
        if let Some(child) = node_ref.find_child_mut(c) {
            let taken_child = child.take();
            let (changed, new_path_in_child, new_child) =
                Self::insert_recursive(taken_child, key, next_depth + 1, score);
            *child = new_child;

            return (changed, new_path_in_child, Some(node_ref));
        }

        // Should never reach here
        (false, false, Some(node_ref))
    }

    /// Finds the node that matches a given prefix.
    /// This internal method traverses the trie to find the node that corresponds
    /// to the end of the specified prefix.
    ///
    /// # Arguments
    /// * `prefix` - The byte representation of the prefix to find.
    ///
    /// # Returns
    /// * `Some((&ARTNode, usize))` - A reference to the matching node and the depth
    ///   reached in the prefix.
    /// * `None` - If no node matches the prefix.
    fn find_node_for_prefix(&self, prefix: &[u8]) -> Option<(&ARTNode, usize)> {
        if self.root.is_none() {
            return None;
        }

        if prefix.is_empty() {
            return self.root.as_ref().map(|n| (n.as_ref(), 0));
        }

        let mut current = self.root.as_ref()?;
        let mut depth = 0;

        // Navigate through the tree to find the prefix
        while depth < prefix.len() {
            // Check prefix match
            let (match_len, exact_match) = current.check_prefix(prefix, depth);

            if !exact_match {
                // Prefix doesn't fully match - no match
                return None;
            }

            depth += match_len;

            if depth == prefix.len() {
                // We've traversed the complete prefix
                return Some((current, depth));
            }

            // Next character in the prefix
            let c = prefix[depth];

            // Look for matching child
            match current.find_child(c) {
                Some(child) => {
                    current = child;
                    depth += 1;
                }
                None => return None, // No matching child
            }
        }

        Some((current, depth))
    }

    /// Collects all paths stored below a given node in the trie.
    /// Uses an iterative approach to prevent stack overflow for deep tries.
    ///
    /// # Arguments
    /// * `node` - The node from which to start collection.
    /// * `results` - A mutable reference to a vector where results will be stored.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// trie.insert("/home/user/file1.txt", 1.0);
    /// trie.insert("/home/user/file2.txt", 0.9);
    ///
    /// let mut results = Vec::new();
    /// if let Some(root) = &trie.root {
    ///     trie.collect_all_paths(root.as_ref(), &mut results);
    /// }
    /// assert_eq!(results.len(), 2);
    /// ```
    fn collect_all_paths(&self, node: &ARTNode, results: &mut Vec<(String, f32)>) {
        let mut stack = Vec::new();
        stack.push((node, String::new(), false)); // (node, current_path, processed_children)

        while let Some((current_node, current_path, processed)) = stack.pop() {
            if !processed {
                // First visit - process this node
                let mut node_path = current_path.clone();

                // Add this node's prefix to the path
                let node_prefix = current_node.get_prefix();
                if !node_prefix.is_empty() {
                    node_path.push_str(&String::from_utf8_lossy(node_prefix));
                }

                // If terminal, add to results
                if current_node.is_terminal() {
                    if let Some(score) = current_node.get_score() {
                        results.push((node_path.clone(), score));
                    }
                }

                // Push this node back to process children later
                stack.push((current_node, node_path.clone(), true));

                // Then process all children (in reverse order because we're using a stack)
                let children: Vec<_> = current_node.iter_children().into_iter().collect();
                for (key, child) in children.into_iter().rev() {
                    let mut child_path = node_path.clone();
                    child_path.push(key as char);

                    // Push this child to process it
                    stack.push((child, child_path, false));
                }
            }
            // If already processed, nothing to do - we've handled everything on the first visit
        }
    }

    /// Finds path components that match a query string, optimized for performance.
    /// This is particularly useful for finding files by partial names regardless of their
    /// location in the directory structure.
    ///
    /// # Arguments
    /// * `query` - The query string to match against path components.
    /// * `current_dir` - Optional current directory to restrict search scope.
    /// * `results` - A mutable reference to a vector where results will be stored.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// trie.insert("/home/user/documents/report.pdf", 1.0);
    ///
    /// let mut results = Vec::new();
    /// trie.find_component_matches_optimized("report", Some("/home/user"), &mut results);
    /// assert_eq!(results.len(), 1);
    /// ```
    fn find_component_matches_optimized(
        &self,
        query: &str,
        current_dir: Option<&str>,
        results: &mut Vec<(String, f32)>,
    ) {
        if self.root.is_none() || query.is_empty() {
            return;
        }

        let normalized_query = self.normalize_path(query);
        let normalized_dir = current_dir.map(|dir| self.normalize_path(dir));

        // Use a depth-limited search for component matching
        if let Some(root) = &self.root {
            // Using a more straightforward collection approach for search completeness
            let mut all_paths = Vec::new();
            self.collect_all_paths(root.as_ref(), &mut all_paths);

            // Process all collected paths
            for (path, score) in all_paths {
                // Check directory context if applicable
                if let Some(ref dir) = normalized_dir {
                    if !path.starts_with(dir) && !path.starts_with(&format!("{}/", dir)) {
                        // Skip paths outside our context
                        continue;
                    }
                }

                // Check if any component matches the query
                let components: Vec<&str> = path.split('/').collect();
                let mut found_match = false;

                for component in &components {
                    // Check for both prefix and substring matches
                    if component.starts_with(&normalized_query) {
                        // Direct prefix match
                        results.push((path.clone(), score * 0.95));
                        found_match = true;
                        break;
                    } else if component.contains(&normalized_query) {
                        // Substring match (this is crucial for matching "doc" in "documents")
                        results.push((path.clone(), score * 0.9));
                        found_match = true;
                        break;
                    }
                }

                // If no component matched but the whole path contains the query
                if !found_match && path.contains(&normalized_query) {
                    results.push((path.clone(), score * 0.85));
                }
            }
        }
    }

    /// Finds all paths that start with a given prefix.
    /// This is the primary method for quickly retrieving paths matching a partial input.
    ///
    /// # Arguments
    /// * `prefix` - A string slice containing the prefix to search for.
    ///
    /// # Returns
    /// * A vector of tuples containing matching paths and their scores, sorted by score.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// trie.insert("/home/user/file1.txt", 1.0);
    /// trie.insert("/home/user/file2.txt", 0.9);
    /// trie.insert("/home/admin/file3.txt", 0.8);
    ///
    /// let results = trie.find_completions("/home/user");
    /// assert_eq!(results.len(), 2);
    /// ```
    pub fn find_completions(&self, prefix: &str) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        if self.root.is_none() {
            return results;
        }

        let normalized = self.normalize_path(prefix);

        // Special case for empty or root queries
        if normalized.is_empty() || normalized == "." || normalized == "./" {
            // Instead of collecting all paths, we'll traverse the trie directly
            if let Some(root) = &self.root {
                // Use direct trie traversal with a maximum limit
                self.collect_results_with_limit(
                    root.as_ref(),
                    &normalized,
                    &mut results,
                    self.max_results,
                );

                // Direct traversal guarantees unique results - skip deduplication
                self.sort_and_deduplicate_results(&mut results, true);

                if results.len() > self.max_results {
                    results.truncate(self.max_results);
                }

                return results;
            }
        }

        // Standard case: search the trie
        let prefix_bytes = normalized.as_bytes();

        // First try exact prefix match (most efficient)
        if let Some((node, _depth)) = self.find_node_for_prefix(prefix_bytes) {
            // Collect results directly from this node
            self.collect_results_with_limit(node, &normalized, &mut results, self.max_results);

            // If we found enough results, return them without further processing
            if results.len() >= self.max_results / 2 {
                // Direct prefix match guarantees unique results
                self.sort_and_deduplicate_results(&mut results, true);
                results.truncate(self.max_results);
                return results;
            }
        }

        // Just accept the low results, because of fuzzy fallback search
        self.sort_and_deduplicate_results(&mut results, true);

        // Limit results
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }

    /// Collects results from a node with a specified limit.
    /// Uses breadth-first traversal to efficiently collect paths,
    /// Due to bad performance with deep searching
    ///
    /// # Arguments
    /// * `node` - The node from which to start collection.
    /// * `prefix` - The prefix string that led to this node.
    /// * `results` - A mutable reference to a vector where results will be stored.
    /// * `limit` - The maximum number of results to collect.
    fn collect_results_with_limit(
        &self,
        node: &ARTNode,
        prefix: &str,
        results: &mut Vec<(String, f32)>,
        limit: usize,
    ) {
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        queue.push_back((node, prefix.to_string()));

        while let Some((current_node, current_path)) = queue.pop_front() {
            // Add terminal nodes
            if current_node.is_terminal() {
                if let Some(score) = current_node.get_score() {
                    let mut node_path = current_path.clone();
                    let node_prefix = current_node.get_prefix();
                    if !node_prefix.is_empty() {
                        node_path.push_str(&String::from_utf8_lossy(node_prefix));
                    }
                    results.push((node_path, score));
                    if results.len() >= limit {
                        break;
                    }
                }
            }

            // Enqueue children (breadth-first)
            for (key, child) in current_node.iter_children() {
                let mut child_path = current_path.clone();
                child_path.push(key as char);
                let child_prefix = child.get_prefix();
                if !child_prefix.is_empty() {
                    child_path.push_str(&String::from_utf8_lossy(child_prefix));
                }
                queue.push_back((child, child_path));
            }
        }
    }

    /// Removes a path from the trie.
    /// Normalizes the path before removal to ensure consistency.
    ///
    /// # Arguments
    /// * `path` - A string slice containing the path to remove.
    ///
    /// # Returns
    /// * `true` if the path was found and removed.
    /// * `false` if the path was not found.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// trie.insert("/home/user/documents/file.txt", 1.0);
    /// assert_eq!(trie.len(), 1);
    ///
    /// assert!(trie.remove("/home/user/documents/file.txt"));
    /// assert_eq!(trie.len(), 0);
    /// ```
    pub fn remove(&mut self, path: &str) -> bool {
        if self.root.is_none() {
            return false;
        }

        let normalized = self.normalize_path(path);
        let path_bytes = normalized.as_bytes();

        // Perform recursive removal
        let root = self.root.take();
        let (removed, should_remove, new_root) = Self::remove_recursive(root, path_bytes, 0);

        if should_remove {
            self.root = None;
        } else {
            self.root = new_root;
        }

        if removed {
            self.path_count -= 1;
        }

        removed
    }

    /// Recursively removes a path from the trie, navigating and cleaning up nodes as needed.
    /// This internal helper method is used by the public remove method.
    ///
    /// # Arguments
    /// * `node` - The current node in the traversal.
    /// * `key` - The byte representation of the path being removed.
    /// * `depth` - The current depth in the key.
    ///
    /// # Returns
    /// * A tuple containing:
    ///   - Whether the path was removed
    ///   - Whether this node should be removed
    ///   - The new node after removal
    fn remove_recursive(
        node: Option<Box<ARTNode>>,
        key: &[u8],
        depth: usize,
    ) -> (bool, bool, Option<Box<ARTNode>>) {
        if node.is_none() {
            return (false, false, None);
        }

        let mut node_ref = node.unwrap();

        // Check prefix match
        let (match_len, exact_match) = node_ref.check_prefix(key, depth);

        if !exact_match {
            // Prefix doesn't match - path not found
            return (false, false, Some(node_ref));
        }

        // After the prefix
        let next_depth = depth + match_len;

        if next_depth == key.len() {
            // We've reached the end of the path
            if !node_ref.is_terminal() {
                // Node exists but is not terminal
                return (false, false, Some(node_ref));
            }

            // Mark as non-terminal
            node_ref.set_terminal(false);
            node_ref.set_score(None);

            // Check if the node should be removed
            let should_remove = node_ref.num_children() == 0;
            return (
                true,
                should_remove,
                if should_remove { None } else { Some(node_ref) },
            );
        }

        // Not at the end of the path - continue recursively
        let c = key[next_depth];

        if let Some(child) = node_ref.find_child_mut(c) {
            let taken_child = child.take();
            let (removed, should_remove_child, new_child) =
                Self::remove_recursive(taken_child, key, next_depth + 1);

            if should_remove_child {
                // Child should be removed
                node_ref.remove_child(c);
            } else {
                // Restore the child with potentially updated state
                *child = new_child;
            }

            // This node should be removed if:
            // 1. It's not terminal
            // 2. It has no children
            let should_remove_this = !node_ref.is_terminal() && node_ref.num_children() == 0;

            return (
                removed,
                should_remove_this,
                if should_remove_this {
                    None
                } else {
                    Some(node_ref)
                },
            );
        }

        // Child not found
        (false, false, Some(node_ref))
    }

    // Additional helper methods for search, length, is_empty, etc.

    pub fn len(&self) -> usize {
        self.path_count
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.path_count == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.path_count = 0;
    }

    /// Sorts and optionally deduplicates a collection of search results.
    /// Results are sorted by score in descending order (highest first).
    ///
    /// # Arguments
    /// * `results` - A mutable reference to a vector of (path, score) tuples.
    /// * `skip_dedup` - Whether to skip deduplication (set to true when results are known to be unique).
    fn sort_and_deduplicate_results(&self, results: &mut Vec<(String, f32)>, skip_dedup: bool) {
        if results.is_empty() {
            return;
        }

        // Sort by score in descending order (highest scores first)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(cmp::Ordering::Equal));

        // Skip deduplication if specified (when we know results are already unique)
        if !skip_dedup {
            // Remove duplicates (keep first occurrence which will be the highest score)
            let mut seen_paths = std::collections::HashSet::new();
            results.retain(|(path, _)| seen_paths.insert(path.clone()));
        }
    }

    /// Searches for paths matching a query string, with optional context directory and component matching.
    /// This is the main search algorithm for the ART implementation.
    ///
    /// # Arguments
    /// * `query` - The query string to search for.
    /// * `current_dir` - Optional current directory to restrict search scope.
    /// * `allow_partial_components` - Whether to match partial components within paths.
    ///
    /// # Returns
    /// * A vector of tuples containing matching paths and their scores, sorted by score.
    ///
    /// # Example
    /// ```rust
    /// let mut trie = ART::new(10);
    /// trie.insert("/home/user/documents/report.pdf", 1.0);
    /// trie.insert("/home/admin/images/photo.jpg", 0.9);
    ///
    /// // Search in context of user directory
    /// let results = trie.search("doc", Some("/home/user"), true);
    /// assert_eq!(results.len(), 1);
    ///
    /// // Search everywhere with component matching
    /// let results = trie.search("rep", None, true);
    /// assert_eq!(results.len(), 1);
    /// ```
    pub fn search(
        &self,
        _query: &str,
        current_dir: Option<&str>,
        allow_partial_components: bool,
    ) -> Vec<(String, f32)> {
        let mut results = Vec::new();
        let query = &self.normalize_path(_query);

        if query.is_empty() {
            return results;
        }

        // If we have a current directory, search only in that context
        if let Some(dir) = current_dir {
            let normalized_dir = self.normalize_path(dir);

            // Format the search path correctly
            let combined_path = if normalized_dir.ends_with('/') {
                format!("{}{}", normalized_dir, query)
            } else {
                format!("{}/{}", normalized_dir, query)
            };

            let context_matches = self.find_completions(&combined_path);
            results.extend(context_matches);

            // If we want partial component matching, use the updated method
            if allow_partial_components {
                self.find_component_matches_optimized(query, Some(dir), &mut results);

                // filter results to ensure they all start with the current directory
                let dir_prefix = if normalized_dir.ends_with('/') {
                    normalized_dir.clone()
                } else {
                    format!("{}/", normalized_dir)
                };

                results.retain(|(path, _)| {
                    path.starts_with(&normalized_dir) || path.starts_with(&dir_prefix)
                });

                // deduplication because of different searches
                self.sort_and_deduplicate_results(&mut results, false);
            } else {
                // no additional deduplication needed
            }
        } else {
            // no current directory, so search everywhere
            let direct_matches = self.find_completions(query);
            results.extend(direct_matches);

            if allow_partial_components {
                self.find_component_matches_optimized(query, None, &mut results);

                self.sort_and_deduplicate_results(&mut results, false);
            }
        }

        // Limit results
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }
}

#[cfg(test)]
mod tests_art_v4 {
    use super::*;
    use crate::{log_info, log_warn};
    use std::path::{Path, PathBuf, MAIN_SEPARATOR};
    #[cfg(feature = "long-tests")]
    use std::time::Duration;
    use std::time::Instant;

    // Helper function to get test data directory
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            log_warn!(&format!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            ));
            panic!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            );
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
            return (0..100)
                .map(|i| {
                    format!(
                        "{}path{}to{}file{}.txt",
                        MAIN_SEPARATOR, MAIN_SEPARATOR, MAIN_SEPARATOR, i
                    )
                })
                .collect();
        }

        paths
    }

    fn normalize_path(path: &str) -> String {
        let mut result = String::with_capacity(path.len());
        let mut saw_slash = false;
        let mut started = false;

        let mut chars = path.chars().peekable();

        // Skip leading whitespace (including Unicode whitespace)
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
            } else {
                break;
            }
        }

        if let Some(&first) = chars.peek() {
            if first == '/' || first == '\\' {
                result.push('/');
                saw_slash = true;
                started = true;
                chars.next();
            }
        }

        for c in chars {
            match c {
                '/' | '\\' => {
                    if !saw_slash && started {
                        result.push('/');
                        saw_slash = true;
                    }
                }
                _ => {
                    result.push(c);
                    saw_slash = false;
                    started = true;
                }
            }
        }

        // Remove trailing slash (unless result is exactly "/")
        let len = result.len();
        if len > 1 && result.ends_with('/') {
            result.truncate(len - 1);
        }

        result
    }

    // Basic functionality tests
    #[test]
    fn test_basic_insert_and_find() {
        log_info!("Starting basic insert and find test");
        let mut trie = ART::new(10);

        // Use platform-agnostic paths by joining components
        let docs_path = Path::new("C:")
            .join("Users")
            .join("Documents")
            .to_string_lossy()
            .to_string();
        let downloads_path = Path::new("C:")
            .join("Users")
            .join("Downloads")
            .to_string_lossy()
            .to_string();
        let pictures_path = Path::new("C:")
            .join("Users")
            .join("Pictures")
            .to_string_lossy()
            .to_string();

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
        let prefix = Path::new("C:").join("Users").to_string_lossy().to_string();
        let completions = trie.find_completions(&prefix);
        assert_eq!(completions.len(), 3);
        log_info!(&format!(
            "Found {} completions for '{}'",
            completions.len(),
            prefix
        ));

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
            "./test-data-for-fuzzy-search/apple.pdf",
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
            let filename = &path[last_slash + 1..];
            assert!(
                filename.starts_with('a'),
                "Filename should start with 'a': {}",
                filename
            );
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
        log_info!(&format!(
            "Normalized for insert: '{}'",
            normalized_for_insert
        ));
        log_info!(&format!(
            "Normalized bytes: {:?}",
            normalized_for_insert.as_bytes()
        ));

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
        log_info!(&format!(
            "Backslash path bytes: {:?}",
            backslash_path.as_bytes()
        ));

        let normalized_bs = trie.normalize_path(backslash_path);
        log_info!(&format!("Normalized backslash path: '{}'", normalized_bs));
        log_info!(&format!(
            "Normalized backslash bytes: {:?}",
            normalized_bs.as_bytes()
        ));
    }

    #[test]
    fn test_empty_prefix_split_and_merge() {
        let mut trie = ART::new(10);

        // Insert paths that only differ at the first char
        trie.insert("a/foo", 1.0);
        trie.insert("b/bar", 2.0);

        trie.debug_print();

        // Insert a path that is a prefix of another
        trie.insert("a", 3.0);

        trie.debug_print();

        fn check_no_chain_of_empty_prefix(node: &ARTNode, is_root: bool, parent_empty: bool, path: String) {
            let prefix = node.get_prefix();
            let this_empty = prefix.is_empty();
            if !is_root && parent_empty && this_empty {
                log_info!(&format!("Chain detected at path: {:?}", path));
                panic!("Chain of empty prefixes detected!");
            }
            let path_desc = format!("{}{:#?}/", path, String::from_utf8_lossy(prefix));
            for (_, child) in node.iter_children() {
                check_no_chain_of_empty_prefix(child, false, this_empty, path_desc.clone());
            }
        }

        // and in the test:
        if let Some(ref root) = trie.root {
            check_no_chain_of_empty_prefix(root, true, false, String::new());
        }
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
        assert_eq!(
            still_find1.len(),
            1,
            "Should still find first path"
        );
        assert_eq!(
            still_find1[0].0, path1,
            "First path should still match exactly"
        );

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
        assert_eq!(
            found1_again.len(),
            1,
            "Should still find path1 after second insertion"
        );
        assert_eq!(found1_again[0].0, path1, "Should still match exact path1");

        let found2 = trie.find_completions(path2);
        assert_eq!(found2.len(), 1, "Should find path2");
        assert_eq!(found2[0].0, path2, "Should match exact path2");

        // Check prefix search - should find both
        let prefix_results = trie.find_completions("a/b/file");
        assert_eq!(
            prefix_results.len(),
            2,
            "Prefix search should find both files"
        );
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
        log_info!(&format!(
            "Before removal: found {} completions for '{}'",
            before_completions.len(),
            path1
        ));
        log_info!(&format!(
            "is_in_trie: {}",
            trie.find_completions(path1).len() > 0
        ));
        assert_eq!(
            before_completions.len(),
            1,
            "Path1 should be found before removal"
        );

        // If needed, verify the exact string (for debugging)
        if !before_completions.is_empty() {
            let found_path = &before_completions[0].0;
            log_info!(&format!(
                "Found path: '{}', Expected: '{}'",
                found_path, path1
            ));
            log_info!(&format!("Path bytes: {:?}", found_path.as_bytes()));
            log_info!(&format!("Expected bytes: {:?}", path1.as_bytes()));
        }

        // Remove path1
        let removed = trie.remove(path1);
        assert!(removed, "Path1 should be successfully removed");
        assert_eq!(trie.len(), 2, "Should have 2 paths after removal");

        // Verify path1 is gone
        let after_completions = trie.find_completions(path1);
        assert_eq!(
            after_completions.len(),
            0,
            "Path1 should be gone after removal"
        );

        // Check that we still find path2 with a common prefix search
        let user_prefix = "home/user/";
        let user_paths = trie.find_completions(user_prefix);
        assert_eq!(
            user_paths.len(),
            1,
            "Should find only 1 user path after removal"
        );
        assert_eq!(
            user_paths[0].0, path2,
            "The remaining user path should be path2"
        );
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
            assert_eq!(
                completions.len(),
                expected_count,
                "Failed for prefix: {}",
                prefix
            );
            log_info!(&format!(
                "Prefix '{}' returned {} completions",
                prefix,
                completions.len()
            ));
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

        log_info!(&format!(
            "Completions correctly sorted by score: {:.1} > {:.1} > {:.1}",
            completions[0].1, completions[1].1, completions[2].1
        ));
    }

    // Performance tests with real-world data
    #[test]
    fn test_insertion_performance_art_v4() {
        log_info!("Testing insertion performance with real paths");
        let mut trie = ART::new(100);

        // Get real-world paths from test data
        let paths = collect_test_paths(Some(500));
        log_info!(&format!("Collected {} test paths", paths.len()));

        // Only insert unique, normalized paths and count them
        let mut unique_normalized = std::collections::HashSet::new();
        for path in &paths {
            let norm = trie.normalize_path(path);
            unique_normalized.insert(norm);
        }

        // Measure time to insert all paths (including duplicates)
        let start = Instant::now();
        for (i, path) in paths.iter().enumerate() {
            trie.insert(path, 1.0 - (i as f32 * 0.001));
        }
        let elapsed = start.elapsed();

        log_info!(&format!(
            "Inserted {} paths in {:?} ({:.2} paths/ms)",
            paths.len(),
            elapsed,
            paths.len() as f64 / elapsed.as_millis().max(1) as f64
        ));

        assert_eq!(trie.len(), unique_normalized.len());
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
                    prefixes.push(path[0..last_sep + 1].to_string());
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
                normalize_path("/home"),
            ]
        };

        for prefix in test_prefixes {
            let start = Instant::now();
            let completions = trie.find_completions(&prefix);
            let elapsed = start.elapsed();

            log_info!(&format!(
                "Found {} completions for '{}' in {:?}",
                completions.len(),
                prefix,
                elapsed
            ));

            if completions.len() > 0 {
                log_info!(&format!(
                    "First completion: {} (score: {:.1})",
                    completions[0].0, completions[0].1
                ));
            }
        }
    }

    #[test]
    fn test_specific_path_cases() {
        let mut trie = ART::new(10);

        // Test the specific cases from your logs
        let base_path = "./test-data-for-fuzzy-search";
        let files = vec!["/airplane.mp4", "/ambulance", "/apple.pdf"];

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

            assert!(
                found,
                "Path {} should be found in completions",
                expected_path
            );
            log_info!(&format!("Found expected path {}: {}", i, expected_path));
        }

        // Test partially matching path
        let partial_path = format!("{}/a", base_path);
        let partial_completions = trie.find_completions(&partial_path);

        assert!(
            partial_completions.len() >= 2,
            "Should find at least airplane.mp4 and apple.pdf"
        );

        // Verify no character splitting
        for (path, _) in &partial_completions {
            // Check no character was incorrectly split
            assert!(
                !path.contains("/i/rplane"),
                "No character splitting in airplane"
            );
            assert!(
                !path.contains("/m/bulance"),
                "No character splitting in ambulance"
            );
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

        log_info!(&format!(
            "Removed 90 paths, trie now contains {} paths",
            trie.len()
        ));

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

    // Fixed debug_test to prevent stack overflow
    #[test]
    fn debug_test() {
        let mut trie = ART::new(10);

        // Use shorter paths to avoid stack issues
        let path = "a/b/f1.txt";
        let path2 = "a/b/f2.txt";
        let path3 = "a/b/d";

        // Insert paths
        trie.insert(path, 1.0);
        trie.insert(path2, 1.0);
        trie.insert(path3, 1.0);

        // Find a path
        let found = trie.find_completions(path);
        assert_eq!(found.len(), 1, "Should find the exact path");

        // Remove a path and check it's gone
        trie.remove(path);
        assert_eq!(
            trie.find_completions(path).len(),
            0,
            "Path should be removed"
        );

        // Verify remaining paths
        assert_eq!(
            trie.find_completions(path2).len(),
            1,
            "Path2 should still exist"
        );
        assert_eq!(
            trie.find_completions(path3).len(),
            1,
            "Path3 should still exist"
        );
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
        assert_eq!(
            results3.len(),
            1,
            "Should only find documents in home/other"
        );
        assert_eq!(results3[0].0, "home/other/documents/report.pdf");

        // Test 4: Partial component matching without directory context
        let results4 = trie.search("doc", None, true);
        assert_eq!(
            results4.len(),
            2,
            "Should find all paths with 'doc' component"
        );

        // Test 5: Search for component that's not in the path
        let results5 = trie.search("missing", Some("home/user"), true);
        assert_eq!(
            results5.len(),
            0,
            "Should find no results for non-existent component"
        );
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
                    let component = &prefix[last_sep_pos + 1..];
                    if component.len() >= 2 {
                        partial_prefixes.push(format!(
                            "{}{}",
                            &prefix[..=last_sep_pos],
                            &component[..component.len().min(2)]
                        ));
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

            log_info!(&format!(
                "Found {} completions for prefix '{}' in {:?}",
                completions.len(),
                original_prefix,
                elapsed
            ));

            if !completions.is_empty() {
                log_info!(&format!(
                    "First result: {} (score: {:.2})",
                    completions[0].0, completions[0].1
                ));

                // Verify that results actually match the normalized prefix
                let valid_matches = completions
                    .iter()
                    .filter(|(path, _)| path.starts_with(&normalized_prefix))
                    .count();

                log_info!(&format!(
                    "{} of {} results are valid prefix matches for '{}' (normalized: '{}')",
                    valid_matches,
                    completions.len(),
                    original_prefix,
                    normalized_prefix
                ));

                assert!(
                    valid_matches > 0,
                    "No valid matches found for prefix '{}' (normalized: '{}')",
                    original_prefix,
                    normalized_prefix
                );
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
    fn benchmark_prefix_search_with_all_paths_art_v4() {
        log_info!("Benchmarking prefix search with thousands of real-world paths");

        // 1. Collect all available paths
        let paths = collect_test_paths(None); // Get all available paths
        let path_count = paths.len();

        log_info!(&format!("Collected {} test paths", path_count));

        // Store all the original paths for verification
        let all_paths = paths.clone();

        // 2. Create ART and insert all paths - add verification
        let start_insert = Instant::now();
        let mut trie = ART::new(100);

        // Track unique normalized paths for accurate verification
        let mut unique_normalized_paths = std::collections::HashSet::new();
        let temp_art = ART::new(1); // Temporary ART for normalization

        for (i, path) in all_paths.iter().enumerate() {
            // Use varying scores based on position
            let score = 1.0 - (i as f32 * 0.0001).min(0.99);

            // Track unique normalized paths before insertion
            let normalized = temp_art.normalize_path(path);
            unique_normalized_paths.insert(normalized);

            trie.insert(path, score);

            // Verify insertion every 10000 paths
            if i % 10000 == 0 && i > 0 {
                log_info!(&format!("Inserted {} paths, verifying...", i));

                // Calculate expected unique count up to this point
                let expected_unique_count = i + 1; // Maximum possible - actual will be lower due to duplicates

                // Check the count is reasonable (allowing for duplicates)
                assert!(
                    trie.len() <= expected_unique_count,
                    "Trie should have at most {} paths, but has {}",
                    expected_unique_count,
                    trie.len()
                );
            }
        }

        let insert_time = start_insert.elapsed();
        log_info!(&format!(
            "Inserted {} paths in {:?} ({:.2} paths/ms)",
            all_paths.len(),
            insert_time,
            all_paths.len() as f64 / insert_time.as_millis().max(1) as f64
        ));

        // Verify the final count matches expectation (accounting for duplicates)
        log_info!(&format!(
            "Expected unique paths: {}, Actual in trie: {}",
            unique_normalized_paths.len(),
            trie.len()
        ));

        // Create a function to generate a diverse set of queries that will have matches
        fn extract_guaranteed_queries(paths: &[String], limit: usize) -> Vec<String> {
            let mut queries = Vec::new();
            let mut seen_queries = std::collections::HashSet::new();

            // Helper function instead of closure to avoid borrowing issues
            fn should_add_query(query: &str, seen: &mut std::collections::HashSet<String>) -> bool {
                let normalized = query.trim_end_matches('/').to_string();
                if !normalized.is_empty() && !seen.contains(&normalized) {
                    seen.insert(normalized);
                    return true;
                }
                false
            }

            if paths.is_empty() {
                return queries;
            }

            // a. Extract directory prefixes from actual paths
            for path in paths.iter().take(paths.len().min(100)) {
                let components: Vec<&str> = path.split(|c| c == '/' || c == '\\').collect();

                // Full path prefixes
                for i in 1..components.len() {
                    if queries.len() >= limit {
                        break;
                    }

                    let prefix = components[0..i].join("/");
                    if !prefix.is_empty() {
                        // Check and add the base prefix
                        if should_add_query(&prefix, &mut seen_queries) {
                            queries.push(prefix.clone());
                        }

                        // Check and add with trailing slash
                        let prefix_slash = format!("{}/", prefix);
                        if should_add_query(&prefix_slash, &mut seen_queries) {
                            queries.push(prefix_slash);
                        }
                    }

                    if queries.len() >= limit {
                        break;
                    }
                }

                // b. Extract filename prefixes (for partial filename matches)
                if queries.len() < limit {
                    if let Some(last) = components.last() {
                        if !last.is_empty() && last.len() > 2 {
                            let first_chars = &last[..last.len().min(2)];
                            if !first_chars.is_empty() {
                                // Add to parent directory
                                if components.len() > 1 {
                                    let parent = components[0..components.len() - 1].join("/");
                                    let partial = format!("{}/{}", parent, first_chars);
                                    if should_add_query(&partial, &mut seen_queries) {
                                        queries.push(partial);
                                    }
                                } else {
                                    if should_add_query(first_chars, &mut seen_queries) {
                                        queries.push(first_chars.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // c. Add specific test cases for backslash and space handling
            if queries.len() < limit {
                if paths
                    .iter()
                    .any(|p| p.contains("test-data-for-fuzzy-search"))
                {
                    // Add queries with various path formats targeting the test data
                    let test_queries = [
                        "./test-data-for-fuzzy-search".to_string(),
                        "./test-data-for-fuzzy-search/".to_string(),
                        "./test-data-for-fuzzy-search\\".to_string(),
                        "./t".to_string(),
                        ".".to_string(),
                    ];

                    for query in test_queries {
                        if queries.len() >= limit {
                            break;
                        }
                        if should_add_query(&query, &mut seen_queries) {
                            queries.push(query);
                        }
                    }

                    // Extract some specific directories from test data
                    if queries.len() < limit {
                        for path in paths.iter() {
                            if queries.len() >= limit {
                                break;
                            }
                            if path.contains("test-data-for-fuzzy-search") {
                                if let Some(suffix) =
                                    path.strip_prefix("./test-data-for-fuzzy-search/")
                                {
                                    if let Some(first_dir_end) = suffix.find('/') {
                                        if first_dir_end > 0 {
                                            let dir_name = &suffix[..first_dir_end];

                                            let query1 = format!(
                                                "./test-data-for-fuzzy-search/{}",
                                                dir_name
                                            );
                                            if should_add_query(&query1, &mut seen_queries) {
                                                queries.push(query1);
                                            }

                                            if queries.len() >= limit {
                                                break;
                                            }

                                            // Add with backslash for test variety
                                            let query2 = format!(
                                                "./test-data-for-fuzzy-search\\{}",
                                                dir_name
                                            );
                                            if should_add_query(&query2, &mut seen_queries) {
                                                queries.push(query2);
                                            }

                                            // Removed the backslash+space test case to avoid spaces in paths
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // If we still don't have enough queries, add some basic ones
            if queries.len() < 3 {
                let basic_queries = ["./".to_string(), "/".to_string(), ".".to_string()];

                for query in basic_queries {
                    if should_add_query(&query, &mut seen_queries) {
                        queries.push(query);
                    }
                }
            }

            // Only keep a reasonable number of queries
            if queries.len() > limit {
                queries.truncate(limit);
            }

            queries
        }

        // Use our function to generate guaranteed-to-match queries
        let test_queries = extract_guaranteed_queries(&all_paths, 15);

        log_info!(&format!(
            "Generated {} guaranteed-to-match queries",
            test_queries.len()
        ));

        // Pre-test queries to verify they match something
        for query in &test_queries {
            let results = trie.search(query, None, false);
            if results.is_empty() {
                log_info!(&format!(
                    "Warning: Query '{}' didn't match any paths",
                    query
                ));
            }
        }

        // 4. Benchmark searches with different batch sizes, with separate tries.
        // Ensure complete independence between different batch size tests
        let batch_sizes = [10, 100, 1000, 10000, all_paths.len()];

        for &batch_size in &batch_sizes {
            // Reset measurements for this batch size
            let subset_size = batch_size.min(all_paths.len());

            // Create a fresh trie with only the needed paths
            let mut subset_trie = ART::new(100);
            let start_insert_subset = Instant::now();

            for i in 0..subset_size {
                subset_trie.insert(&all_paths[i], 1.0 - (i as f32 * 0.0001));
            }

            let subset_insert_time = start_insert_subset.elapsed();
            log_info!(&format!("\n=== BENCHMARK WITH {} PATHS ===", subset_size));
            log_info!(&format!(
                "Subset insertion time: {:?} ({:.2} paths/ms)",
                subset_insert_time,
                subset_size as f64 / subset_insert_time.as_millis().max(1) as f64
            ));

            // Generate test queries specifically for this subset
            let subset_paths = all_paths
                .iter()
                .take(subset_size)
                .cloned()
                .collect::<Vec<_>>();
            let subset_queries = extract_guaranteed_queries(&subset_paths, 15);

            log_info!(&format!(
                "Generated {} subset-specific queries",
                subset_queries.len()
            ));

            // Run a single warmup search to prime any caches
            subset_trie.search("./", None, false);

            // Run measurements on each test query
            let mut total_time = Duration::new(0, 0);
            let mut total_results = 0;
            let mut times = Vec::new();

            for query in &subset_queries {
                // Measure the search performance
                let start = Instant::now();
                let completions = subset_trie.search(&normalize_path(query), None, false);
                let elapsed = start.elapsed();

                total_time += elapsed;
                total_results += completions.len();
                times.push((query.clone(), elapsed, completions.len()));

                // Print top 3 results for each search
                //log_info!(&format!("Top results for '{}' (found {})", normalize_path(query), completions.len()));
                //for (i, (path, score)) in completions.iter().take(3).enumerate() {
                //    log_info!(&format!("    #{}: '{}' (score: {:.3})", i+1, path, score));
                //}
                //if completions.len() > 3 {
                //    log_info!(&format!("    ... and {} more results", completions.len() - 3));
                //}
            }

            // 5. Report statistics
            times.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by time, slowest first

            let avg_time = if !subset_queries.is_empty() {
                total_time / subset_queries.len() as u32
            } else {
                Duration::new(0, 0)
            };

            let avg_results = if !subset_queries.is_empty() {
                total_results / subset_queries.len()
            } else {
                0
            };

            log_info!(&format!("Ran {} prefix searches", subset_queries.len()));
            log_info!(&format!("Average search time: {:?}", avg_time));
            log_info!(&format!("Average results per search: {}", avg_results));

            // Log the slowest searches
            log_info!("Slowest searches:");
            for (i, (query, time, count)) in times.iter().take(3).enumerate() {
                log_info!(&format!(
                    "  #{}: '{:40}' - {:?} ({} results)",
                    i + 1,
                    normalize_path(query),
                    time,
                    count
                ));
            }

            // Log the fastest searches
            log_info!("Fastest searches:");
            for (i, (query, time, count)) in times.iter().rev().take(3).enumerate() {
                log_info!(&format!(
                    "  #{}: '{:40}' - {:?} ({} results)",
                    i + 1,
                    normalize_path(query),
                    time,
                    count
                ));
            }

            // Log search times for different result counts
            let mut by_result_count = Vec::new();
            for &count in &[0, 1, 10, 100] {
                let matching: Vec<_> = times.iter().filter(|(_, _, c)| *c >= count).collect();

                if !matching.is_empty() {
                    let total = matching
                        .iter()
                        .fold(Duration::new(0, 0), |sum, (_, time, _)| sum + *time);
                    let avg = total / matching.len() as u32;

                    by_result_count.push((count, avg, matching.len()));
                }
            }

            log_info!("Average search times by result count:");
            for (count, avg_time, num_searches) in by_result_count {
                log_info!(&format!(
                    "  ≥ {:3} results: {:?} (from {} searches)",
                    count, avg_time, num_searches
                ));
            }
        }
    }

    #[test]
    fn test_preserve_space_searches() {
        let mut trie = ART::new(10);

        // Create paths with backslash+space sequences that match benchmark problematic searches
        let paths = vec![
            "./test-data-for-fuzzy-search/ coconut/file1.txt",
            "./test-data-for-fuzzy-search/ blueberry/file2.txt",
            "./test-data-for-fuzzy-search/ truck/banana/ raspberry/file3.txt",
            "./test-data-for-fuzzy-search/ tangerine/file4.txt",
        ];

        // Insert all paths
        for path in &paths {
            trie.insert(path, 1.0);

            // Verify insertion worked
            let found = trie.find_completions(path);
            assert_eq!(
                found.len(),
                1,
                "Path should be found after insertion: {}",
                path
            );
        }

        // Test searches with escaped spaces
        let searches = vec![
            "./test-data-for-fuzzy-search\\ coconut",
            "./test-data-for-fuzzy-search\\ blueberry",
            "./test-data-for-fuzzy-search\\ truck\\banana\\ raspberry",
            "./test-data-for-fuzzy-search\\ tangerine",
        ];

        for (i, search) in searches.iter().enumerate() {
            let results = trie.find_completions(search);
            assert!(
                !results.is_empty(),
                "Search '{}' should find at least one result",
                search
            );

            // The corresponding path should be found
            let expected_path = &paths[i];
            let found = results.iter().any(|(p, _)| p.starts_with(expected_path));
            assert!(
                found,
                "Path '{}' should be found for search '{}'",
                expected_path, search
            );
        }
    }

    #[test]
    fn test_extended_normalization() {
        let art = ART::new(10);

        // 1. Simple ASCII path
        assert_eq!(
            art.normalize_path("foo/bar/baz.txt"),
            "foo/bar/baz.txt"
        );

        // 2. Mixed slashes, should be normalized
        assert_eq!(
            art.normalize_path("foo\\bar/baz\\qux.txt"),
            "foo/bar/baz/qux.txt"
        );

        // 3. Leading slash and duplicate slashes
        assert_eq!(
            art.normalize_path("//foo///bar//baz//"),
            "/foo/bar/baz"
        );

        // 4. Spaces inside components
        assert_eq!(
            art.normalize_path("dir with spaces/file name.txt"),
            "dir with spaces/file name.txt"
        );

        // 5. Spaces at the start and end (should be preserved if inside components)
        assert_eq!(
            art.normalize_path(" /foo/ bar /baz "),
            "/foo/ bar /baz "
        );

        // 6. Unicode: Chinese, emoji, diacritics
        assert_eq!(
            art.normalize_path("用户/桌面/🚀 rocket/naïve.txt"),
            "用户/桌面/🚀 rocket/naïve.txt"
        );

        // 7. Combination: leading backslash, spaces, Unicode, duplicate slashes
        assert_eq!(
            art.normalize_path("\\用户//桌面/ 🚀  rocket//naïve.txt "),
            "/用户/桌面/ 🚀  rocket/naïve.txt "
        );

        // 8. Only slashes (should be "/")
        assert_eq!(
            art.normalize_path("//////"),
            "/"
        );

        // 9. Rooted path with component with space and unicode
        assert_eq!(
            art.normalize_path("/a/ b 🚗 /c"),
            "/a/ b 🚗 /c"
        );

        // 10. Windows absolute path with mixed slashes and unicode
        assert_eq!(
            art.normalize_path("C:\\用户\\桌面\\My File 🚲.txt"),
            "C:/用户/桌面/My File 🚲.txt"
        );

        // 11. Trailing slash, not root (should remove trailing)
        assert_eq!(
            art.normalize_path("/foo/bar/"),
            "/foo/bar"
        );
    }
    #[test]
    fn test_normalization() {
        let mut trie = ART::new(10);

        // Test paths with different separators
        let paths = vec![
            "./test-data-for-fuzzy-search/ airplane.mp4",
            "./test-data-for-fuzzy-search\\ambulance",
            "./test-data-for-fuzzy-search\\ apple.pdf",
        ];

        // Insert all paths
        for path in &paths {
            trie.insert(path, 1.0);

            // Verify insertion worked
            let found = trie.find_completions(path);
            assert_eq!(
                found.len(),
                1,
                "Path should be found after insertion: {}",
                path
            );
        }

        // Test normalization
        for path in &paths {
            let normalized = trie.normalize_path(path);
            assert_eq!(
                normalized,
                normalize_path(path),
                "Normalization failed for path: {}",
                path
            );
        }
    }
}
