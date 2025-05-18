use std::cmp;
use std::fmt::Debug;
use std::mem;
use crate::log_warn;

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

// Use a small-size optimization for prefixes
const INLINE_PREFIX_SIZE: usize = 8;

// A byte is used to navigate between node types and operations
type KeyType = u8;

// Small-size optimization for prefixes
#[derive(Clone)]
enum PrefixStorage {
    Inline {
        data: [KeyType; INLINE_PREFIX_SIZE],
        len: usize,
    },
    Heap(Vec<KeyType>),
}

impl PrefixStorage {
    fn new() -> Self {
        PrefixStorage::Inline {
            data: [0; INLINE_PREFIX_SIZE],
            len: 0,
        }
    }

    fn from_slice(slice: &[KeyType]) -> Self {
        if slice.len() <= INLINE_PREFIX_SIZE {
            let mut data = [0; INLINE_PREFIX_SIZE];
            data[..slice.len()].copy_from_slice(slice);
            PrefixStorage::Inline {
                data,
                len: slice.len(),
            }
        } else {
            PrefixStorage::Heap(slice.to_vec())
        }
    }

    fn len(&self) -> usize {
        match self {
            PrefixStorage::Inline { len, .. } => *len,
            PrefixStorage::Heap(vec) => vec.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, index: usize) -> Option<KeyType> {
        match self {
            PrefixStorage::Inline { data, len } => {
                if index < *len {
                    Some(data[index])
                } else {
                    None
                }
            }
            PrefixStorage::Heap(vec) => vec.get(index).copied(),
        }
    }

    fn as_slice(&self) -> &[KeyType] {
        match self {
            PrefixStorage::Inline { data, len } => &data[..*len],
            PrefixStorage::Heap(vec) => vec.as_slice(),
        }
    }

    fn take_first(&mut self) -> Option<KeyType> {
        if self.is_empty() {
            return None;
        }

        match self {
            PrefixStorage::Inline { data, len } => {
                let first = data[0];
                *len -= 1;
                if *len > 0 {
                    data.copy_within(1.., 0);
                }
                Some(first)
            }
            PrefixStorage::Heap(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
        }
    }

    fn truncate_front(&mut self, count: usize) {
        match self {
            PrefixStorage::Inline { data, len } => {
                if count >= *len {
                    *len = 0;
                } else {
                    data.copy_within(count.., 0);
                    *len -= count;
                }
            }
            PrefixStorage::Heap(vec) => {
                if count >= vec.len() {
                    vec.clear();
                } else {
                    vec.drain(0..count);
                }
            }
        }
    }
}

// ------------------ ARTNode Enum and Implementations ---------------

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

    fn get_prefix(&self) -> &PrefixStorage {
        match self {
            ARTNode::Node4(n) => &n.prefix,
            ARTNode::Node16(n) => &n.prefix,
            ARTNode::Node48(n) => &n.prefix,
            ARTNode::Node256(n) => &n.prefix,
        }
    }

    fn get_prefix_mut(&mut self) -> &mut PrefixStorage {
        match self {
            ARTNode::Node4(n) => &mut n.prefix,
            ARTNode::Node16(n) => &mut n.prefix,
            ARTNode::Node48(n) => &mut n.prefix,
            ARTNode::Node256(n) => &mut n.prefix,
        }
    }

    // Optimized prefix match that returns match length and whether it's exact
    fn check_prefix(&self, key: &[KeyType], depth: usize) -> (usize, bool) {
        let prefix = self.get_prefix();

        if prefix.is_empty() {
            return (0, true);
        }

        // Calculate how many characters we can compare
        let prefix_len = prefix.len();
        let max_cmp = cmp::min(prefix_len, key.len() - depth);

        // Fast path if the key is shorter than our prefix
        if max_cmp < prefix_len {
            return (max_cmp, false);
        }

        // Compare prefix bytes - using SIMD optimization when available
        let prefix_slice = prefix.as_slice();
        let mut i = 0;

        // Manual unrolling for better performance
        while i + 4 <= max_cmp {
            let mut match_failed = false;
            if prefix_slice[i] != key[depth + i] {
                return (i, false);
            }
            if prefix_slice[i+1] != key[depth + i+1] {
                return (i+1, false);
            }
            if prefix_slice[i+2] != key[depth + i+2] {
                return (i+2, false);
            }
            if prefix_slice[i+3] != key[depth + i+3] {
                return (i+3, false);
            }
            i += 4;
        }

        // Handle remaining characters
        while i < max_cmp {
            if prefix_slice[i] != key[depth + i] {
                return (i, false);
            }
            i += 1;
        }

        (max_cmp, max_cmp == prefix_len)
    }

    // Optimized split_prefix method
    fn split_prefix(&mut self, mismatch_pos: usize) {
        // Fast path for no split needed
        if mismatch_pos == 0 {
            return;
        }

        // Get the current prefix
        let old_prefix = self.get_prefix().as_slice().to_vec();

        // Update this node's prefix to just the matched portion
        let mut new_prefix = PrefixStorage::from_slice(&old_prefix[..mismatch_pos]);
        mem::swap(self.get_prefix_mut(), &mut new_prefix);

        // Create a new node for the rest of the prefix and the children
        let mut new_node = ARTNode::new_node4();

        // If there's remaining prefix, add it to the new node
        if mismatch_pos < old_prefix.len() {
            *new_node.get_prefix_mut() = PrefixStorage::from_slice(&old_prefix[mismatch_pos+1..]);
        }

        // Move terminal status and score to the new node
        new_node.set_terminal(self.is_terminal());
        new_node.set_score(self.get_score());
        self.set_terminal(false);
        self.set_score(None);

        // Get the split character (the character at the mismatch position)
        let split_char = old_prefix[mismatch_pos];

        // Move all children from this node to the new node
        self.move_children_to(&mut new_node);

        // Add the new node as a child under the split character
        self.add_child(split_char, Some(Box::new(new_node)));
    }

    // Helper to move all children from self to another node
    fn move_children_to(&mut self, target: &mut ARTNode) {
        match (self, target) {
            (ARTNode::Node4(n), ARTNode::Node4(target_n)) => {
                // Direct swap of children arrays and metadata
                mem::swap(&mut n.children, &mut target_n.children);
                mem::swap(&mut n.keys, &mut target_n.keys);
                mem::swap(&mut n.size, &mut target_n.size);
            },
            (ARTNode::Node16(n), ARTNode::Node4(target_n)) => {
                // Transfer children to the target node (must be one by one)
                for i in 0..n.size {
                    if n.children[i].is_some() {
                        let child = mem::replace(&mut n.children[i], None);
                        target_n.add_child(n.keys[i], child);
                    }
                }
                n.size = 0;
            },
            (ARTNode::Node48(n), ARTNode::Node4(target_n)) => {
                // Transfer children to the target node
                for i in 0..256 {
                    if let Some(idx) = n.child_index[i] {
                        if n.children[idx as usize].is_some() {
                            let child = mem::replace(&mut n.children[idx as usize], None);
                            target_n.add_child(i as u8, child);
                        }
                    }
                }
                n.child_index = [None; 256];
                n.size = 0;
            },
            (ARTNode::Node256(n), ARTNode::Node4(target_n)) => {
                // Transfer children to the target node
                for i in 0..256 {
                    if n.children[i].is_some() {
                        let child = mem::replace(&mut n.children[i], None);
                        target_n.add_child(i as u8, child);
                    }
                }
                n.size = 0;
            },
            // Currently not handled: larger target node types
            _ => {}
        }
    }

    // Add a child with node growth if needed
    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        // Try to add the child first
        let add_result = match self {
            ARTNode::Node4(n) => n.add_child(key, child.clone()),
            ARTNode::Node16(n) => n.add_child(key, child.clone()),
            ARTNode::Node48(n) => n.add_child(key, child.clone()),
            ARTNode::Node256(n) => n.add_child(key, child.clone()),
        };

        // If the node is full and we need to grow
        if !add_result {
            // Check if we need to grow
            let need_grow = match self {
                ARTNode::Node4(n) => n.size >= NODE4_MAX,
                ARTNode::Node16(n) => n.size >= NODE16_MAX,
                ARTNode::Node48(n) => n.size >= NODE48_MAX,
                ARTNode::Node256(_) => false, // Node256 never needs to grow
            };

            if need_grow {
                // Grow to the next size
                let mut grown_node = self.grow();
                // Try to add to the grown node
                let result = grown_node.add_child(key, child);
                // Replace self with the grown node
                *self = grown_node;
                result
            } else {
                false
            }
        } else {
            true
        }
    }

    // Find a child by key with inlining for performance
    #[inline(always)]
    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        match self {
            ARTNode::Node4(n) => n.find_child(key),
            ARTNode::Node16(n) => n.find_child(key),
            ARTNode::Node48(n) => n.find_child(key),
            ARTNode::Node256(n) => n.find_child(key),
        }
    }

    // Find a child by key (mutable variant) with inlining
    #[inline(always)]
    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        match self {
            ARTNode::Node4(n) => n.find_child_mut(key),
            ARTNode::Node16(n) => n.find_child_mut(key),
            ARTNode::Node48(n) => n.find_child_mut(key),
            ARTNode::Node256(n) => n.find_child_mut(key),
        }
    }

    // Remove a child by key
    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        let removed = match self {
            ARTNode::Node4(n) => n.remove_child(key),
            ARTNode::Node16(n) => {
                let removed = n.remove_child(key);
                // Only shrink if we actually removed something
                if removed.is_some() && n.size <= NODE4_MAX {
                    *self = self.shrink();
                }
                removed
            },
            ARTNode::Node48(n) => {
                let removed = n.remove_child(key);
                if removed.is_some() && n.size <= NODE16_MAX {
                    *self = self.shrink();
                }
                removed
            },
            ARTNode::Node256(n) => {
                let removed = n.remove_child(key);
                if removed.is_some() && n.size <= NODE48_MAX {
                    *self = self.shrink();
                }
                removed
            },
        };
        removed
    }

    // Iterate over all children
    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        match self {
            ARTNode::Node4(n) => n.iter_children(),
            ARTNode::Node16(n) => n.iter_children(),
            ARTNode::Node48(n) => n.iter_children(),
            ARTNode::Node256(n) => n.iter_children(),
        }
    }

    // Number of children
    fn num_children(&self) -> usize {
        match self {
            ARTNode::Node4(n) => n.size,
            ARTNode::Node16(n) => n.size,
            ARTNode::Node48(n) => n.size,
            ARTNode::Node256(n) => n.size,
        }
    }

    // Grow to a larger node type
    fn grow(&self) -> Self {
        match self {
            ARTNode::Node4(n) => {
                // Node4 -> Node16
                let mut n16 = Node16::new();
                n16.prefix = n.prefix.clone();
                n16.is_terminal = n.is_terminal;
                n16.score = n.score;

                // Copy all children
                for i in 0..n.size {
                    if let Some(child) = &n.children[i] {
                        n16.keys[i] = n.keys[i];
                        n16.children[i] = Some(Box::new(*child.clone()));
                    }
                }
                n16.size = n.size;

                ARTNode::Node16(n16)
            },
            ARTNode::Node16(n) => {
                // Node16 -> Node48
                let mut n48 = Node48::new();
                n48.prefix = n.prefix.clone();
                n48.is_terminal = n.is_terminal;
                n48.score = n.score;

                // Copy all children
                for i in 0..n.size {
                    if let Some(child) = &n.children[i] {
                        let key = n.keys[i] as usize;
                        n48.children[i] = Some(Box::new(*child.clone()));
                        n48.child_index[key] = Some(i as u8);
                    }
                }
                n48.size = n.size;

                ARTNode::Node48(n48)
            },
            ARTNode::Node48(n) => {
                // Node48 -> Node256
                let mut n256 = Node256::new();
                n256.prefix = n.prefix.clone();
                n256.is_terminal = n.is_terminal;
                n256.score = n.score;

                // Copy all children
                for i in 0..256 {
                    if let Some(idx) = n.child_index[i] {
                        if let Some(child) = &n.children[idx as usize] {
                            n256.children[i] = Some(Box::new(*child.clone()));
                        }
                    }
                }
                n256.size = n.size;

                ARTNode::Node256(n256)
            },
            ARTNode::Node256(_) => {
                // Node256 is already the largest type
                self.clone()
            },
        }
    }

    // Shrink to a smaller node type
    fn shrink(&self) -> Self {
        match self {
            ARTNode::Node16(n) => {
                // Node16 -> Node4
                let mut n4 = Node4::new();
                n4.prefix = n.prefix.clone();
                n4.is_terminal = n.is_terminal;
                n4.score = n.score;

                // Copy up to 4 children
                let mut count = 0;
                for i in 0..n.size {
                    if count >= NODE4_MAX {
                        break;
                    }

                    if let Some(child) = &n.children[i] {
                        n4.keys[count] = n.keys[i];
                        n4.children[count] = Some(Box::new(*child.clone()));
                        count += 1;
                    }
                }
                n4.size = count;

                ARTNode::Node4(n4)
            },
            ARTNode::Node48(n) => {
                // Node48 -> Node16
                let mut n16 = Node16::new();
                n16.prefix = n.prefix.clone();
                n16.is_terminal = n.is_terminal;
                n16.score = n.score;

                // Collect all children
                let mut count = 0;
                for i in 0..256 {
                    if count >= NODE16_MAX {
                        break;
                    }

                    if let Some(idx) = n.child_index[i] {
                        if let Some(child) = &n.children[idx as usize] {
                            n16.keys[count] = i as KeyType;
                            n16.children[count] = Some(Box::new(*child.clone()));
                            count += 1;
                        }
                    }
                }
                n16.size = count;

                ARTNode::Node16(n16)
            },
            ARTNode::Node256(n) => {
                // Node256 -> Node48
                let mut n48 = Node48::new();
                n48.prefix = n.prefix.clone();
                n48.is_terminal = n.is_terminal;
                n48.score = n.score;

                // Collect up to 48 children
                let mut count = 0;
                for i in 0..256 {
                    if count >= NODE48_MAX {
                        break;
                    }

                    if let Some(child) = &n.children[i] {
                        n48.children[count] = Some(Box::new(*child.clone()));
                        n48.child_index[i] = Some(count as u8);
                        count += 1;
                    }
                }
                n48.size = count;

                ARTNode::Node48(n48)
            },
            _ => self.clone(), // Other node types aren't shrunk
        }
    }
}

// In Rust we must explicitly implement Clone for ARTNode
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
    prefix: PrefixStorage,
    is_terminal: bool,
    score: Option<f32>,
    keys: [KeyType; NODE4_MAX],
    children: [Option<Box<ARTNode>>; NODE4_MAX],
    size: usize,
}

impl Node4 {
    fn new() -> Self {
        Node4 {
            prefix: PrefixStorage::new(),
            is_terminal: false,
            score: None,
            keys: [0; NODE4_MAX],
            children: [None, None, None, None],
            size: 0,
        }
    }

    // Add a child with optimized implementation
    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        // Fast path for replacement
        for i in 0..self.size {
            if self.keys[i] == key {
                self.children[i] = child;
                return true;
            }
        }

        // Check if there's space
        if self.size >= NODE4_MAX {
            return false;
        }

        // Insertion sort to maintain order
        let mut i = self.size;
        // Use reverse scan for better likely cache behavior
        while i > 0 && self.keys[i - 1] > key {
            self.keys[i] = self.keys[i - 1];
            self.children[i] = self.children[i - 1].take();
            i -= 1;
        }

        // Insert the new child
        self.keys[i] = key;
        self.children[i] = child;
        self.size += 1;
        true
    }

    // Find child with linear search (optimized for small arrays)
    #[inline(always)]
    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        // Unrolled linear search for better performance
        if self.size > 0 && self.keys[0] == key {
            return self.children[0].as_ref();
        }
        if self.size > 1 && self.keys[1] == key {
            return self.children[1].as_ref();
        }
        if self.size > 2 && self.keys[2] == key {
            return self.children[2].as_ref();
        }
        if self.size > 3 && self.keys[3] == key {
            return self.children[3].as_ref();
        }
        None
    }

    // Mutable child lookup
    #[inline(always)]
    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        // Unrolled linear search
        if self.size > 0 && self.keys[0] == key {
            return Some(&mut self.children[0]);
        }
        if self.size > 1 && self.keys[1] == key {
            return Some(&mut self.children[1]);
        }
        if self.size > 2 && self.keys[2] == key {
            return Some(&mut self.children[2]);
        }
        if self.size > 3 && self.keys[3] == key {
            return Some(&mut self.children[3]);
        }
        None
    }

    // Remove a child
    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        // Find the child
        let mut idx = None;
        for i in 0..self.size {
            if self.keys[i] == key {
                idx = Some(i);
                break;
            }
        }

        // If found, remove it
        if let Some(i) = idx {
            let removed = mem::replace(&mut self.children[i], None);

            // Shift remaining children
            for j in i..self.size-1 {
                self.keys[j] = self.keys[j+1];
                self.children[j] = self.children[j+1].take();
            }

            self.size -= 1;
            removed
        } else {
            None
        }
    }

    // Iterate over children
    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            if let Some(child) = &self.children[i] {
                result.push((self.keys[i], child));
            }
        }
        result
    }
}

// Node16: Stores up to 16 children with binary search
#[derive(Clone)]
struct Node16 {
    prefix: PrefixStorage,
    is_terminal: bool,
    score: Option<f32>,
    keys: [KeyType; NODE16_MAX],
    children: [Option<Box<ARTNode>>; NODE16_MAX],
    size: usize,
}

impl Node16 {
    fn new() -> Self {
        // In Rust we must initialize fixed-size arrays
        Node16 {
            prefix: PrefixStorage::new(),
            is_terminal: false,
            score: None,
            keys: [0; NODE16_MAX],
            children: unsafe {
                // SAFETY: We're initializing all elements to None
                let mut arr: [Option<Box<ARTNode>>; NODE16_MAX] = mem::zeroed();
                for i in 0..NODE16_MAX {
                    arr[i] = None;
                }
                arr
            },
            size: 0,
        }
    }

    // Add a child implementation
    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        // Fast path for key replacement
        for i in 0..self.size {
            if self.keys[i] == key {
                self.children[i] = child;
                return true;
            }
        }

        // Check capacity
        if self.size >= NODE16_MAX {
            return false;
        }

        // Binary search would be more efficient for finding insertion position
        // but for simplicity we'll use insertion sort since the array is already sorted
        let mut i = self.size;
        while i > 0 && self.keys[i - 1] > key {
            self.keys[i] = self.keys[i - 1];
            self.children[i] = self.children[i - 1].take();
            i -= 1;
        }

        self.keys[i] = key;
        self.children[i] = child;
        self.size += 1;
        true
    }

    // Find a child using binary search
    #[inline(always)]
    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        // Binary search
        let mut l = 0;
        let mut r = self.size;

        while l < r {
            let m = l + (r - l) / 2;
            if self.keys[m] < key {
                l = m + 1;
            } else {
                r = m;
            }
        }

        if l < self.size && self.keys[l] == key {
            self.children[l].as_ref()
        } else {
            None
        }
    }

    // Find a child by key (mutable variant)
    #[inline(always)]
    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        // Binary search
        let mut l = 0;
        let mut r = self.size;

        while l < r {
            let m = l + (r - l) / 2;
            if self.keys[m] < key {
                l = m + 1;
            } else {
                r = m;
            }
        }

        if l < self.size && self.keys[l] == key {
            Some(&mut self.children[l])
        } else {
            None
        }
    }

    // Remove a child by key
    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        // Binary search to find the child
        let mut l = 0;
        let mut r = self.size;

        while l < r {
            let m = l + (r - l) / 2;
            if self.keys[m] < key {
                l = m + 1;
            } else {
                r = m;
            }
        }

        if l < self.size && self.keys[l] == key {
            let removed = mem::replace(&mut self.children[l], None);

            // Shift all elements after the removed one
            for j in l..self.size-1 {
                self.keys[j] = self.keys[j+1];
                self.children[j] = self.children[j+1].take();
            }

            self.size -= 1;
            removed
        } else {
            None
        }
    }

    // Iterate over all children
    fn iter_children(&self) -> Vec<(KeyType, &Box<ARTNode>)> {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            if let Some(child) = &self.children[i] {
                result.push((self.keys[i], child));
            }
        }
        result
    }
}

// Node48: Uses a direct index array for fast access
#[derive(Clone)]
struct Node48 {
    prefix: PrefixStorage,
    is_terminal: bool,
    score: Option<f32>,
    // Index array: Points from byte value to position in children array
    child_index: [Option<u8>; 256],
    // Only non-null entries are stored in the children array
    children: [Option<Box<ARTNode>>; NODE48_MAX],
    size: usize,
}

impl Node48 {
    fn new() -> Self {
        // Initialize child_index with None
        let child_index = [None; 256];

        Node48 {
            prefix: PrefixStorage::new(),
            is_terminal: false,
            score: None,
            child_index,
            children: unsafe {
                // SAFETY: We're initializing all elements to None
                let mut arr: [Option<Box<ARTNode>>; NODE48_MAX] = mem::zeroed();
                for i in 0..NODE48_MAX {
                    arr[i] = None;
                }
                arr
            },
            size: 0,
        }
    }

    // Add child implementation - optimized for O(1) access
    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        let key_idx = key as usize;

        // Check if key already exists
        if let Some(idx) = self.child_index[key_idx] {
            // Replace existing child
            self.children[idx as usize] = child;
            return true;
        }

        // Check capacity
        if self.size >= NODE48_MAX {
            return false;
        }

        // Add new child to next available slot
        self.children[self.size] = child;
        self.child_index[key_idx] = Some(self.size as u8);
        self.size += 1;
        true
    }

    // Find a child by key - O(1) operation
    #[inline(always)]
    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        let key_idx = key as usize;
        // Direct array lookup - very fast
        if let Some(idx) = self.child_index[key_idx] {
            self.children[idx as usize].as_ref()
        } else {
            None
        }
    }

    // Find a child by key (mutable variant) - O(1) operation
    #[inline(always)]
    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        let key_idx = key as usize;
        // Direct array lookup
        if let Some(idx) = self.child_index[key_idx] {
            Some(&mut self.children[idx as usize])
        } else {
            None
        }
    }

    // Remove a child by key
    fn remove_child(&mut self, key: KeyType) -> Option<Box<ARTNode>> {
        let key_idx = key as usize;

        // Try to find the child
        if let Some(idx) = self.child_index[key_idx] {
            let idx = idx as usize;
            let removed = mem::replace(&mut self.children[idx], None);

            // Remove index mapping
            self.child_index[key_idx] = None;

            // If the removed child wasn't the last one, move the last child to its position
            if idx < self.size - 1 && self.size > 1 {
                // Find the key for the last child
                for (k, &child_idx) in self.child_index.iter().enumerate() {
                    if let Some(ci) = child_idx {
                        if ci as usize == self.size - 1 {
                            // Move last child to the freed position
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

    // Iterate over all children
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

// Node256: Direct access for all possible byte values
#[derive(Clone)]
struct Node256 {
    prefix: PrefixStorage,
    is_terminal: bool,
    score: Option<f32>,
    // Uses the byte value directly as index (O(1) access)
    children: [Option<Box<ARTNode>>; NODE256_MAX],
    size: usize,
}

impl Node256 {
    fn new() -> Self {
        Node256 {
            prefix: PrefixStorage::new(),
            is_terminal: false,
            score: None,
            children: unsafe {
                // SAFETY: We're initializing all elements to None
                let mut arr: [Option<Box<ARTNode>>; NODE256_MAX] = mem::zeroed();
                for i in 0..NODE256_MAX {
                    arr[i] = None;
                }
                arr
            },
            size: 0,
        }
    }

    // Add a child - direct array access
    fn add_child(&mut self, key: KeyType, child: Option<Box<ARTNode>>) -> bool {
        let key_idx = key as usize;
        let is_new = self.children[key_idx].is_none();

        self.children[key_idx] = child;

        if is_new {
            self.size += 1;
        }

        true // Node256 can always add a child
    }

    // Find a child - direct array access
    #[inline(always)]
    fn find_child(&self, key: KeyType) -> Option<&Box<ARTNode>> {
        self.children[key as usize].as_ref()
    }

    // Find a child (mutable) - direct array access
    #[inline(always)]
    fn find_child_mut(&mut self, key: KeyType) -> Option<&mut Option<Box<ARTNode>>> {
        Some(&mut self.children[key as usize])
    }

    // Remove a child
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

    // Iterate over all children
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

// ------------------ ART Implementation ------------------

impl ART {
    pub fn new(max_results: usize) -> Self {
        ART {
            root: None,
            path_count: 0,
            max_results,
        }
    }

    // Fast path normalization using pre-calculated sizes
    fn normalize_path(&self, path: &str) -> String {
        if path.is_empty() {
            return String::new();
        }

        // Step 1: Handle escaped spaces - pre-allocate to avoid reallocations
        let space_fixed = if path.contains("\\ ") {
            path.replace("\\ ", " ")
        } else {
            path.to_string()
        };

        // Step 2: Handle platform-specific separators
        let slash_fixed = if space_fixed.contains('\\') {
            space_fixed.replace('\\', "/")
        } else {
            space_fixed
        };

        // Step 3: Fix doubled slashes
        let mut normalized = slash_fixed;
        if normalized.contains("//") {
            while normalized.contains("//") {
                normalized = normalized.replace("//", "/");
            }
        }

        // Step 4: Handle trailing slashes
        let trimmed = if normalized == "/" {
            "/".to_string()
        } else {
            normalized.trim_end_matches('/').to_string()
        };

        // Step 5: Clean up spaces that should be separators
        if trimmed.contains(' ') {
            let components: Vec<&str> = trimmed.split(' ').collect();
            if components.len() > 1 &&
                components[0].contains('/') &&
                !components.iter().skip(1).any(|&c| c.contains('/')) {
                return components.join("/");
            }
        }

        trimmed
    }

    // Optimized insert method with path existence check
    pub fn insert(&mut self, path: &str, score: f32) -> bool {
        let normalized = self.normalize_path(path);
        let path_bytes = normalized.as_bytes();

        // Create root if it doesn't exist
        if self.root.is_none() {
            self.root = Some(Box::new(ARTNode::new_node4()));
            // No need to check for existence in this case
            let (changed, _new_path, new_root) = Self::insert_recursive(
                self.root.take(),
                path_bytes,
                0,
                score,
                &mut self.path_count
            );
            self.root = new_root;
            return changed;
        }

        // Direct insert with path counting
        let (changed, _new_path, new_root) = Self::insert_recursive(
            self.root.take(),
            path_bytes,
            0,
            score,
            &mut self.path_count
        );
        self.root = new_root;
        changed
    }

    // Recursive insert helper method - improved to inline path count updates
    fn insert_recursive(
        mut node: Option<Box<ARTNode>>,
        key: &[u8],
        depth: usize,
        score: f32,
        path_count: &mut usize
    ) -> (bool, bool, Option<Box<ARTNode>>) {
        if node.is_none() {
            node = Some(Box::new(ARTNode::new_node4()));
        }

        let mut node_ref = node.unwrap();

        // Fast path: end of key
        if depth == key.len() {
            let mut changed = false;
            let mut new_path = false;

            // If this node is not already terminal, it's a new path
            if !node_ref.is_terminal() {
                node_ref.set_terminal(true);
                new_path = true;
                *path_count += 1; // Directly update path count
                changed = true;
            }

            // Update score if different
            if node_ref.get_score() != Some(score) {
                node_ref.set_score(Some(score));
                changed = true;
            }

            return (changed, new_path, Some(node_ref));
        }

        // Check prefix match
        let (match_len, exact_match) = node_ref.check_prefix(key, depth);

        if !exact_match {
            // Prefix doesn't match - split the node
            node_ref.split_prefix(match_len);
        }

        // After the prefix - position in the key
        let next_depth = depth + match_len;

        // Another fast path: end of key after prefix
        if next_depth == key.len() {
            let mut changed = false;
            let mut new_path = false;

            if !node_ref.is_terminal() {
                node_ref.set_terminal(true);
                new_path = true;
                *path_count += 1; // Directly update path count
                changed = true;
            }

            if node_ref.get_score() != Some(score) {
                node_ref.set_score(Some(score));
                changed = true;
            }

            return (changed, new_path, Some(node_ref));
        }

        // Get next character in the path
        let c = key[next_depth];

        // Check if we need to create a new child
        let need_new_child = node_ref.find_child_mut(c).is_none();

        if need_new_child {
            // Create a new empty child
            node_ref.add_child(c, None);
        }

        // Continue with the child node
        if let Some(child) = node_ref.find_child_mut(c) {
            let taken_child = child.take();
            let (changed, _new_path, new_child) = Self::insert_recursive(
                taken_child,
                key,
                next_depth + 1,
                score,
                path_count // Pass path_count directly
            );
            *child = new_child;
            return (changed, false, Some(node_ref));
        }

        // Should never reach here
        (false, false, Some(node_ref))
    }

    // Find completions using non-recursive traversal for speed
    pub fn find_completions(&self, prefix: &str) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        if self.root.is_none() {
            return results;
        }

        let normalized = self.normalize_path(prefix);
        let prefix_bytes = normalized.as_bytes();

        // Find the node that matches the prefix
        let mut current = self.root.as_ref().unwrap();
        let mut node = current.as_ref();
        let mut depth = 0;

        // Navigate to the prefix node
        while depth < prefix_bytes.len() {
            // Check prefix match
            let (match_len, exact_match) = node.check_prefix(prefix_bytes, depth);

            if !exact_match {
                // Prefix doesn't match - no results
                return results;
            }

            depth += match_len;

            if depth == prefix_bytes.len() {
                // We've reached the end of the prefix
                break;
            }

            // Get next character
            let c = prefix_bytes[depth];

            // Find matching child
            match node.find_child(c) {
                Some(child) => {
                    node = child.as_ref();
                    depth += 1;
                },
                None => return results, // No matching child
            }
        }

        // Collect all completions from the prefix node
        self.collect_results(node, &normalized, &mut results);

        // Sort results by score
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }

    // Optimized results collection using Vec with capacity
    fn collect_results(&self, node: &ARTNode, prefix: &str, results: &mut Vec<(String, f32)>) {
        // Stack-based traversal to avoid recursion
        let mut stack = Vec::with_capacity(64); // Pre-allocate for likely depth
        stack.push((node, prefix.to_string()));

        while let Some((current, current_prefix)) = stack.pop() {
            // If this node is terminal, add it to results
            if current.is_terminal() {
                if let Some(score) = current.get_score() {
                    results.push((current_prefix.clone(), score));
                }
            }

            // Process all children
            for (key, child) in current.iter_children() {
                let mut new_prefix = current_prefix.clone();
                new_prefix.push(key as char);

                // If the child has a compressed prefix, add it
                let prefix_storage = child.get_prefix();
                if !prefix_storage.is_empty() {
                    for i in 0..prefix_storage.len() {
                        if let Some(c) = prefix_storage.get(i) {
                            new_prefix.push(c as char);
                        }
                    }
                }

                // Add the child to the stack
                stack.push((child.as_ref(), new_prefix));
            }
        }
    }

    // Optimized implementation of collect_all_paths for component search
    fn collect_all_paths(&self, node: &ARTNode, results: &mut Vec<(String, f32)>) {
        // Use a stack-based approach to avoid recursion
        let mut stack = Vec::with_capacity(64);
        stack.push((node, String::new()));

        while let Some((current, current_path)) = stack.pop() {
            // If this node is terminal, add the current path to results
            if current.is_terminal() {
                if let Some(score) = current.get_score() {
                    results.push((current_path.clone(), score));
                }
            }

            // Process each child
            for (key, child) in current.iter_children() {
                let mut new_path = current_path.clone();

                // Add the key character
                new_path.push(key as char);

                // If the child has a compressed prefix, add it
                let prefix_storage = child.get_prefix();
                if !prefix_storage.is_empty() {
                    for i in 0..prefix_storage.len() {
                        if let Some(c) = prefix_storage.get(i) {
                            new_path.push(c as char);
                        }
                    }
                }

                // Add to stack for processing
                stack.push((child.as_ref(), new_path));
            }
        }
    }

    // Optimized remove method
    pub fn remove(&mut self, path: &str) -> bool {
        if self.root.is_none() {
            return false;
        }

        let normalized = self.normalize_path(path);
        let path_bytes = normalized.as_bytes();

        // Perform recursive removal
        let mut root = self.root.take();
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

    // Recursive removal helper method
    fn remove_recursive(
        node: Option<Box<ARTNode>>,
        key: &[u8],
        depth: usize
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
            return (true, should_remove, if should_remove { None } else { Some(node_ref) });
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

            return (removed, should_remove_this,
                    if should_remove_this { None } else { Some(node_ref) });
        }

        // Child not found
        (false, false, Some(node_ref))
    }

    // Optimized search with component matching
    pub fn search(&self, query: &str, current_dir: Option<&str>, allow_partial_components: bool) -> Vec<(String, f32)> {
        let mut results = Vec::new();

        if query.is_empty() {
            return results;
        }

        // Pre-allocate an estimated capacity
        results.reserve(self.max_results * 2);

        // Case 1: Direct prefix search
        let direct_matches = self.find_completions(query);
        results.extend(direct_matches);

        // Case 2: Search in current directory context
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

        // Case 3: Partial component search - only if needed
        if allow_partial_components && results.len() < self.max_results {
            self.find_component_matches(query, current_dir, &mut results);
        }

        // Sort and deduplicate results
        self.sort_and_deduplicate_results(&mut results);

        // Limit results
        if results.len() > self.max_results {
            results.truncate(self.max_results);
        }

        results
    }

    // Optimized component matching
    fn find_component_matches(&self, query: &str, current_dir: Option<&str>, results: &mut Vec<(String, f32)>) {
        if self.root.is_none() {
            return;
        }

        let normalized_query = self.normalize_path(query);

        if normalized_query.is_empty() {
            return;
        }

        // If we already have enough results, don't do expensive component matching
        if results.len() >= self.max_results {
            return;
        }

        let normalized_dir = current_dir.map(|dir| self.normalize_path(dir));

        // Only collect paths if we need to - this is expensive!
        let mut all_paths = Vec::with_capacity(self.path_count.min(1000));
        if let Some(root) = &self.root {
            self.collect_all_paths(root.as_ref(), &mut all_paths);
        }

        // Fast query match lookup
        let query_bytes = normalized_query.as_bytes();

        for (path, score) in all_paths {
            // Check directory context first to avoid unnecessary processing
            if let Some(ref dir) = normalized_dir {
                if !path.starts_with(dir) && !path.starts_with(&format!("{}/", dir)) {
                    continue;
                }
            }

            // Split path into components using a preallocated buffer
            let mut components = Vec::with_capacity(10); // Most paths have fewer than 10 components
            let mut start = 0;

            for (i, &b) in path.as_bytes().iter().enumerate() {
                if b == b'/' {
                    if i > start {
                        components.push(&path[start..i]);
                    }
                    start = i + 1;
                }
            }

            // Add the last component
            if start < path.len() {
                components.push(&path[start..]);
            }

            // Check if any component contains the query
            for component in &components {
                // Fast case-sensitive check for query in component
                if component.contains(&normalized_query) {
                    // Adjust score based on match type
                    let adjusted_score = if component.starts_with(&normalized_query) {
                        score * 0.95 // Small penalty for prefix match
                    } else {
                        score * 0.9  // Larger penalty for substring match
                    };

                    results.push((path.clone(), adjusted_score));
                    break; // Only count each path once
                }
            }
        }
    }

    // Optimized sort and deduplicate
    fn sort_and_deduplicate_results(&self, results: &mut Vec<(String, f32)>) {
        if results.is_empty() {
            return;
        }

        // Sort by score (highest first)
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate (keeping highest score for each path)
        let mut seen_paths = std::collections::HashSet::with_capacity(results.len());
        results.retain(|(path, _)| seen_paths.insert(path.clone()));
    }

    // Helper methods

    pub fn len(&self) -> usize {
        self.path_count
    }

    pub fn is_empty(&self) -> bool {
        self.path_count == 0
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.path_count = 0;
    }

    // Fast path lookup
    pub fn contains(&self, path: &str) -> bool {
        if self.root.is_none() {
            return false;
        }

        let normalized = self.normalize_path(path);
        let path_bytes = normalized.as_bytes();

        let mut current = self.root.as_ref().unwrap().as_ref();
        let mut depth = 0;

        while depth < path_bytes.len() {
            // Check prefix match
            let (match_len, exact_match) = current.check_prefix(path_bytes, depth);

            if !exact_match {
                return false;
            }

            depth += match_len;

            if depth == path_bytes.len() {
                // Reached the end of the path
                return current.is_terminal();
            }

            // Get next character
            let c = path_bytes[depth];

            // Find matching child
            match current.find_child(c) {
                Some(child) => {
                    current = child.as_ref();
                    depth += 1;
                },
                None => return false,
            }
        }

        current.is_terminal()
    }
}

#[cfg(test)]
mod tests_art_v4 {
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

        log_info!(&format!("Inserted {} paths in {:?} ({:.2} paths/ms)",
            paths.len(), elapsed, paths.len() as f64 / elapsed.as_millis().max(1) as f64));

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
    fn benchmark_prefix_search_with_all_paths_art_v4() {
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
