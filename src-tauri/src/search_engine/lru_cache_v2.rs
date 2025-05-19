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
    // Create a new LRU cache with the specified capacity
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

    // Create a new LRU cache with the specified capacity and time-to-live duration
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        let mut cache = Self::new(capacity);
        cache.ttl = Some(ttl);
        cache
    }

    // Get a value from the cache, returns None if not found or if expired
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&node_ptr) = self.map.get(key) {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &mut *node_ptr.as_ptr() };

            // Check if the entry has expired
            if let Some(ttl) = self.ttl {
                if node.last_accessed.elapsed() > ttl {
                    // Entry has expired, remove it
                    self.remove(key);
                    return None;
                }
            }

            // Update last accessed time
            node.last_accessed = Instant::now();

            // Optimization: Skip detach/prepend if already at head
            if self.head != Some(node_ptr) {
                // Move to front of list (most recently used)
                self.detach_node(node_ptr);
                self.prepend_node(node_ptr);
            }

            Some(node.value.clone())
        } else {
            None
        }
    }

    // Remove a key from the cache
    pub fn remove(&mut self, key: &K) -> bool {
        if let Some(node_ptr) = self.map.remove(key) {
            self.detach_node(node_ptr);

            // SAFETY: The pointer is valid as it's managed by the cache, and we own it now
            unsafe {
                // Convert back to Box and drop
                drop(Box::from_raw(node_ptr.as_ptr()));
            }

            true
        } else {
            false
        }
    }

    // Insert a value into cache
    pub fn insert(&mut self, key: K, value: V) {
        // Check if the key already exists
        if let Some(&node_ptr) = self.map.get(&key) {
            // SAFETY: The pointer is valid as it's managed by the cache
            let node = unsafe { &mut *node_ptr.as_ptr() };
            node.value = value;
            node.last_accessed = Instant::now();

            self.detach_node(node_ptr);
            self.prepend_node(node_ptr);
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

    // Clear the cache
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

    // Returns the number of items in the cache
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[allow(dead_code)] // used in wrapper so remove later
    // Returns true if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    // Remove all expired entries from the cache
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
    use std::time::Instant;
    use std::thread::sleep;
    use std::path::PathBuf;

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
        log_info!(&format!("Average retrieval time for existing paths: {:.2} ns", avg_retrieval_time));

        // Benchmark getting non-existent paths
        let start = Instant::now();
        for i in 1000..1500 {
            let path = base_path.join(format!("folder_{}", i));
            let _ = cache.get(&path.to_string_lossy().to_string());
        }
        let elapsed = start.elapsed();

        let avg_miss_time = elapsed.as_nanos() as f64 / 500.0;
        log_info!(&format!("Average retrieval time for non-existent paths: {:.2} ns", avg_miss_time));
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
            for i in size/2..(size/2 + 1000).min(size + 500) {
                let path = format!("/path/to/file_{}", i);
                let _ = cache.get(&path);
            }
            let elapsed = start.elapsed();

            log_info!(&format!("Cache size {}: 1000 lookups took {:?} (avg: {:.2} ns/lookup)",
                    size,
                    elapsed,
                    elapsed.as_nanos() as f64 / 1000.0));
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

        log_info!(&format!("Time to insert 20 items with eviction: {:?}", elapsed));

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

        log_info!(&format!("Evicted {} items from the middle range", evicted_count));
    }
}