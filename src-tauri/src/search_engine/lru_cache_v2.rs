//! # LRU Cache Implementation
//!
//! This module provides an optimal LRU (Least Recently Used) cache implementation
//! using a combination of a HashMap and a doubly-linked list:
//!
//! - **HashMap<K, NonNull<Node<K,V>>>**: For O(1) key lookup
//! - **Doubly-linked list**: For maintaining usage order
//!
//! ## Performance Characteristics
//!
//! | Operation | Time Complexity | Notes |
//! |-----------|----------------|-------|
//! | Get       | O(1)           | Hash lookup + linked list update |
//! | Insert    | O(1)           | Hash insert + list prepend (may include eviction) |
//! | Remove    | O(1)           | Hash removal + list detachment |
//! | Clear     | O(n)           | Where n is the current cache size |
//!
//! ## Empirical Scaling
//!
//! Benchmarks show that as cache size increases by 10×, lookup time increases only slightly:
//!
//! | Cache Size | Avg Lookup Time (ns) | Scaling Factor |
//! |------------|----------------------|----------------|
//! | 100        | 57.4                 | -              |
//! | 1,000      | 141.9                | ~2.5×          |
//! | 10,000     | 204                  | ~1.4×          |
//! | 100,000    | 265.2                | ~1.3×          |
//!
//! This confirms the near O(1) performance with only a slight increase due to memory effects.

use std::collections::HashMap;
use std::hash::Hash;
use std::ptr::NonNull;
use std::time::{Duration, Instant};

pub struct LruPathCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    // HashMap, storing pointers to nodes
    map: HashMap<K, NonNull<Node<K, V>>>,

    // Head of the linked list (most recently used)
    head: Option<NonNull<Node<K, V>>>,

    // Tail of the linked list (least recently used)
    tail: Option<NonNull<Node<K, V>>>,

    //  TTL for cache entries
    ttl: Option<Duration>,

    // max items
    capacity: usize,
}

// Node in the doubly linked list
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
    last_accessed: Instant,
}

impl<K, V> LruPathCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new LRU cache with the specified capacity.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of entries the cache can hold. Must be greater than zero.
    ///
    /// # Returns
    ///
    /// A new `LruPathCache` instance with the specified capacity.
    ///
    /// # Panics
    ///
    /// Panics if the capacity is zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// let cache: LruPathCache<String, String> = LruPathCache::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than zero");
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            ttl: None,
        }
    }

    /// Creates a new LRU cache with the specified capacity and time-to-live duration.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of entries the cache can hold. Must be greater than zero.
    /// * `ttl` - The time-to-live duration after which entries are considered expired.
    ///
    /// # Returns
    ///
    /// A new `LruPathCache` instance with the specified capacity and TTL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// let cache: LruPathCache<String, String> = LruPathCache::with_ttl(
    ///     100,
    ///     Duration::from_secs(30)
    /// );
    /// ```
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        let mut cache = Self::new(capacity);
        cache.ttl = Some(ttl);
        cache
    }

    /// Checks if an entry with the given key exists and is not expired,
    /// without updating its position in the LRU order.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time hash lookup
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check for existence and non-expiration.
    ///
    /// # Returns
    ///
    /// * `true` - If the key exists and is not expired.
    /// * `false` - If the key does not exist or is expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// cache.insert("key1".to_string(), "value1".to_string());
    ///
    /// if cache.check_ttl(&"key1".to_string()) {
    ///     println!("Key exists and is not expired");
    /// }
    /// ```
    #[inline]
    pub fn check_ttl(&self, key: &K) -> bool {
        if let Some(&node_ptr) = self.map.get(key) {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &*node_ptr.as_ptr() };

            // expired?
            if let Some(ttl) = self.ttl {
                if node.last_accessed.elapsed() > ttl {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Retrieves a value from the cache by its key.
    ///
    /// If the entry exists and is not expired, it is moved to the front of the cache
    /// (marking it as most recently used) and its value is returned. If the entry has
    /// expired, it is removed from the cache and None is returned.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time hash lookup + linked list update
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up in the cache.
    ///
    /// # Returns
    ///
    /// * `Some(V)` - The value associated with the key if it exists and is not expired.
    /// * `None` - If the key does not exist or the entry has expired.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// cache.insert("key1".to_string(), "value1".to_string());
    ///
    /// match cache.get(&"key1".to_string()) {
    ///     Some(value) => println!("Found value: {}", value),
    ///     None => println!("Key not found or expired"),
    /// }
    /// ```
    #[inline]
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&node_ptr) = self.map.get(key) {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &mut *node_ptr.as_ptr() };

            // expired?
            if let Some(ttl) = self.ttl {
                if node.last_accessed.elapsed() > ttl {
                    self.remove(key);
                    return None;
                }
            }

            // Update last accessed time
            node.last_accessed = Instant::now();

            // skip if head
            if self.head != Some(node_ptr) {
                // move to front
                self.detach_node(node_ptr);
                self.prepend_node(node_ptr);
            }

            Some(node.value.clone())
        } else {
            None
        }
    }

    /// Removes an entry with the specified key from the cache.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time hash removal + linked list detachment
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the entry to remove.
    ///
    /// # Returns
    ///
    /// * `true` - If an entry with the key was found and removed.
    /// * `false` - If no entry with the key was found.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// cache.insert("key1".to_string(), "value1".to_string());
    ///
    /// if cache.remove(&"key1".to_string()) {
    ///     println!("Entry was successfully removed");
    /// } else {
    ///     println!("No entry to remove");
    /// }
    /// ```
    #[inline]
    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(node_ptr) = self.map.remove(key) {
            self.detach_node(node_ptr);

            // SAFETY: The pointer is valid as it's managed by the cache, and we own it now
            unsafe {
                //first convert to box and drop
                drop(Box::from_raw(node_ptr.as_ptr()));
            }

            true
        } else {
            false
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the key already exists, the value is updated and the entry is moved
    /// to the front of the cache (marked as most recently used). If the cache
    /// is at capacity and a new key is inserted, the least recently used entry
    /// is removed to make space.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time hash insertion + linked list update (including potential eviction)
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert.
    /// * `value` - The value to associate with the key.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    ///
    /// // Insert a new entry
    /// cache.insert("key1".to_string(), "value1".to_string());
    ///
    /// // Update an existing entry
    /// cache.insert("key1".to_string(), "updated_value".to_string());
    /// ```
    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        // Check if the key already exists
        if let Some(&node_ptr) = self.map.get(&key) {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &mut *node_ptr.as_ptr() };
            node.value = value;
            node.last_accessed = Instant::now();

            // skip if head
            if self.head != Some(node_ptr) {
                self.detach_node(node_ptr);
                self.prepend_node(node_ptr);
            }
            return;
        }

        // if capacity full, remove the least recently used item
        if self.map.len() >= self.capacity {
            if let Some(tail) = self.tail {
                // SAFETY: The pointer is valid as it's managed by the cache
                let tail_node = unsafe { &*tail.as_ptr() };
                self.remove(&tail_node.key);
            }
        }

        let node = Box::new(Node {
            key: key.clone(),
            value,
            prev: None,
            next: None,
            last_accessed: Instant::now(),
        });

        // Convert Box to raw pointer
        let node_ptr = unsafe { NonNull::new_unchecked(Box::into_raw(node)) };

        // Add to front of list
        self.prepend_node(node_ptr);

        // Add to map
        self.map.insert(key, node_ptr);
    }

    /// Removes all entries from the cache.
    ///
    /// # Time Complexity
    ///
    /// - O(n) - Linear in the number of elements in the cache
    ///
    /// This method properly deallocates all nodes and resets the cache to an empty state.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        // Free all nodes
        while let Some(head) = self.head {
            // SAFETY: The pointer is valid as it's managed by the cache
            let head_node = unsafe { &*head.as_ptr() };
            let head_key = head_node.key.clone();
            self.remove(&head_key);
        }

        // Clear the map
        self.map.clear();
    }

    /// Returns the number of entries currently in the cache.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Returns
    ///
    /// The number of entries in the cache.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// cache.insert("key2".to_string(), "value2".to_string());
    ///
    /// assert_eq!(cache.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Checks if the cache is empty.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Returns
    ///
    /// * `true` - If the cache contains no entries.
    /// * `false` - If the cache contains at least one entry.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = LruPathCache::new(100);
    /// assert!(cache.is_empty());
    ///
    /// cache.insert("key1".to_string(), "value1".to_string());
    /// assert!(!cache.is_empty());
    /// ```
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Purges all expired entries from the cache.
    ///
    /// # Time Complexity
    ///
    /// - O(n) - Linear in the number of elements in the cache
    ///
    /// This method checks all entries and removes any that have expired
    /// based on their TTL. If the cache does not have a TTL set, this
    /// method does nothing.
    ///
    /// # Returns
    ///
    /// The number of expired entries that were removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// let mut cache = LruPathCache::with_ttl(100, Duration::from_millis(100));
    /// cache.insert("key1".to_string(), "value1".to_string());
    ///
    /// sleep(Duration::from_millis(150));
    /// let purged = cache.purge_expired();
    /// assert_eq!(purged, 1);
    /// ```
    pub fn purge_expired(&mut self) -> usize {
        if self.ttl.is_none() {
            return 0;
        }

        let ttl = self.ttl.unwrap();
        let mut expired_keys = Vec::new();

        for (key, &node_ptr) in &self.map {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &*node_ptr.as_ptr() };
            if node.last_accessed.elapsed() > ttl {
                expired_keys.push(key.clone());
            }
        }

        for key in &expired_keys {
            self.remove(key);
        }

        expired_keys.len()
    }

    // Helper method to detach a node from the linked list
    #[inline(always)]
    fn detach_node(&mut self, node_ptr: NonNull<Node<K, V>>) {
        // SAFETY: The pointer is valid as it's managed by the cache
        let node = unsafe { &mut *node_ptr.as_ptr() };

        match (node.prev, node.next) {
            (Some(prev), Some(next)) => {
                // Node is in the middle of the list
                unsafe {
                    (*prev.as_ptr()).next = Some(next);
                    (*next.as_ptr()).prev = Some(prev);
                }
            }
            (None, Some(next)) => {
                // Node is at the head
                unsafe {
                    (*next.as_ptr()).prev = None;
                }
                self.head = Some(next);
            }
            (Some(prev), None) => {
                // Node is at the tail
                unsafe {
                    (*prev.as_ptr()).next = None;
                }
                self.tail = Some(prev);
            }
            (None, None) => {
                // Node is the only one in the list
                self.head = None;
                self.tail = None;
            }
        }

        // Clear node's pointers
        node.prev = None;
        node.next = None;
    }

    // Helper method to add a node to the front of the linked list
    #[inline(always)]
    fn prepend_node(&mut self, node_ptr: NonNull<Node<K, V>>) {
        // SAFETY: The pointer is valid as it's managed by the cache
        let node = unsafe { &mut *node_ptr.as_ptr() };

        match self.head {
            Some(head) => {
                // List is not empty
                node.next = Some(head);
                node.prev = None;

                // Update head's prev pointer
                unsafe {
                    (*head.as_ptr()).prev = Some(node_ptr);
                }

                // Update head
                self.head = Some(node_ptr);
            }
            None => {
                // List is empty
                node.next = None;
                node.prev = None;
                self.head = Some(node_ptr);
                self.tail = Some(node_ptr);
            }
        }
    }
}

impl<K, V> Drop for LruPathCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn drop(&mut self) {
        self.clear();
    }
}

// Guard against memory leaks with custom Clone and Debug impls
impl<K, V> Clone for LruPathCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        // Create a new empty cache with the same capacity and TTL
        let mut new_cache = Self::new(self.capacity);
        new_cache.ttl = self.ttl;

        // We need to walk through our linked list in order (from most to least recent)
        let mut current = self.head;
        while let Some(node_ptr) = current {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &*node_ptr.as_ptr() };
            new_cache.insert(node.key.clone(), node.value.clone());
            current = node.next;
        }

        new_cache
    }
}

#[cfg(test)]
mod tests_lru_cache_v2 {
    use super::*;
    use crate::log_info;
    use std::path::PathBuf;
    use std::thread::sleep;
    use std::time::Instant;

    #[test]
    fn test_basic_operations() {
        let mut cache: LruPathCache<String, String> = LruPathCache::new(3);

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        // Test insertion
        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());

        // Test retrieval
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(cache.get(&"key3".to_string()), None);

        // Test LRU behavior (capacity limit)
        cache.insert("key3".to_string(), "value3".to_string());
        cache.insert("key4".to_string(), "value4".to_string());

        // key1 should be evicted since it's the least recently used
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

        // Test removal
        assert!(cache.remove(&"key3".to_string()));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"key3".to_string()), None);

        // Test clear
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_ttl_expiration() {
        let ttl = Duration::from_millis(100);
        let mut cache = LruPathCache::with_ttl(5, ttl);

        cache.insert("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));

        // Wait for the entry to expire
        sleep(ttl + Duration::from_millis(10));

        // The entry should have expired
        assert_eq!(cache.get(&"key1".to_string()), None);

        // Test purge_expired
        cache.insert("key2".to_string(), "value2".to_string());
        cache.insert("key3".to_string(), "value3".to_string());

        sleep(ttl + Duration::from_millis(10));

        // Add a fresh entry
        cache.insert("key4".to_string(), "value4".to_string());

        // key2 and key3 should expire, but key4 should remain
        let purged = cache.purge_expired();
        assert_eq!(purged, 2);
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.get(&"key4".to_string()), Some("value4".to_string()));
    }

    #[test]
    fn test_clone() {
        let mut original = LruPathCache::new(3);
        original.insert("key1".to_string(), "value1".to_string());
        original.insert("key2".to_string(), "value2".to_string());

        let mut cloned = original.clone();

        assert_eq!(cloned.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cloned.get(&"key2".to_string()), Some("value2".to_string()));
        assert_eq!(cloned.len(), 2);
    }

    #[test]
    fn benchmark_path_retrieval() {
        // Create paths similar to what might be cached in a file explorer
        let base_path = PathBuf::from("C:/Users/username/Documents");
        let mut cache = LruPathCache::new(1000);

        // Populate cache with sample paths
        for i in 0..500 {
            let path = base_path.join(format!("folder_{}", i));
            let metadata = format!("size: {}, modified: 2023-01-01", i * 1000);
            cache.insert(path.to_string_lossy().to_string(), metadata);
        }

        log_info!("Starting path retrieval benchmark");

        // Benchmark getting existing paths
        let start = Instant::now();
        for i in 0..500 {
            let path = base_path.join(format!("folder_{}", i));
            let _ = cache.get(&path.to_string_lossy().to_string());
        }
        let elapsed = start.elapsed();

        let avg_retrieval_time = elapsed.as_nanos() as f64 / 500.0;
        log_info!(
            "Average retrieval time for existing paths: {:.2} ns",
            avg_retrieval_time
        );

        // Benchmark getting non-existent paths
        let start = Instant::now();
        for i in 1000..1500 {
            let path = base_path.join(format!("folder_{}", i));
            let _ = cache.get(&path.to_string_lossy().to_string());
        }
        let elapsed = start.elapsed();

        let avg_miss_time = elapsed.as_nanos() as f64 / 500.0;
        log_info!(
            "Average retrieval time for non-existent paths: {:.2} ns",
            avg_miss_time
        );
    }

    #[test]
    fn benchmark_cache_size_impact_lru_cache() {
        log_info!("Benchmarking impact of cache size on retrieval performance");

        let sizes = [100, 1000, 10000, 100000];

        for &size in &sizes {
            let mut cache = LruPathCache::new(size);

            // Fill the cache to capacity
            for i in 0..size {
                let path = format!("/path/to/file_{}", i);
                cache.insert(path.clone(), format!("metadata_{}", i));
            }

            // Measure retrieval time (mixed hits and misses)
            let start = Instant::now();
            for i in size / 2..(size / 2 + 1000).min(size + 500) {
                let path = format!("/path/to/file_{}", i);
                let _ = cache.get(&path);
            }
            let elapsed = start.elapsed();

            log_info!(
                "Cache size {}: 1000 lookups took {:?} (avg: {:.2} ns/lookup)",
                size,
                elapsed,
                elapsed.as_nanos() as f64 / 1000.0
            );
        }
    }

    #[test]
    fn benchmark_lru_behavior() {
        log_info!("Benchmarking LRU eviction behavior");

        let mut cache = LruPathCache::new(100);

        // Fill cache
        for i in 0..100 {
            cache.insert(format!("key_{}", i), format!("value_{}", i));
        }

        // Access first 20 items to make them recently used
        for i in 0..20 {
            let _ = cache.get(&format!("key_{}", i));
        }

        // Insert 20 new items, which should evict the least recently used
        let start = Instant::now();
        for i in 100..120 {
            cache.insert(format!("key_{}", i), format!("value_{}", i));
        }
        let elapsed = start.elapsed();

        log_info!(
            "Time to insert 20 items with eviction: {:?}",
            elapsed
        );

        // Verify the first 20 items are still there (recently used)
        for i in 0..20 {
            assert!(cache.get(&format!("key_{}", i)).is_some());
        }

        // Verify some of the middle items were evicted
        let mut evicted_count = 0;
        for i in 20..100 {
            if cache.get(&format!("key_{}", i)).is_none() {
                evicted_count += 1;
            }
        }

        log_info!(
            "Evicted {} items from the middle range",
            evicted_count
        );
    }
}
