use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct LRUCache<K, V> {
    cache: HashMap<K, (V, usize)>, // Value and position
    position_map: Vec<K>,          // Ordered keys by recent usage
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V: Clone> LRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: HashMap::new(),
            position_map: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if self.cache.contains_key(key) {
            // Move to end (most recently used)
            self.move_to_end(key);
            let (value, _) = self.cache.get(key).unwrap();
            Some(value.clone())
        } else {
            None
        }
    }

    fn move_to_end(&mut self, key: &K) {
        // Find and remove key from position_map
        if let Some(pos) = self.position_map.iter().position(|k| k == key) {
            self.position_map.remove(pos);
        }
        // Add to end (most recent)
        self.position_map.push(key.clone());
        // Update position in cache
        if let Some((_value, pos)) = self.cache.get_mut(key) {
            *pos = self.position_map.len() - 1;
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        // Remove the key if it already exists
        self.remove(&key);

        // Evict least recently used item if at capacity
        if self.position_map.len() >= self.capacity && !self.position_map.is_empty() {
            let oldest_key = self.position_map.remove(0);
            self.cache.remove(&oldest_key);
        }

        // Insert new key-value pair
        self.position_map.push(key.clone());
        self.cache.insert(key, (value, self.position_map.len() - 1));
    }

    pub fn update(&mut self, key: &K, value: V) -> bool {
        if self.cache.contains_key(key) {
            if let Some((val, _pos)) = self.cache.get_mut(key) {
                *val = value;
                self.move_to_end(key);
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&mut self, key: &K) -> bool {
        if let Some((_, pos)) = self.cache.remove(key) {
            if pos < self.position_map.len() {
                self.position_map.remove(pos);
                // Update positions for all keys after the removed one
                for (_, p) in self.cache.values_mut() {
                    if *p > pos {
                        *p -= 1;
                    }
                }
            }
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.position_map.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// Thread-safe LRU cache wrapper
pub struct ThreadSafeLRUCache<K, V> {
    cache: Arc<RwLock<LRUCache<K, V>>>,
}

impl<K: Hash + Eq + Clone, V: Clone> ThreadSafeLRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LRUCache::<K, V>::new(capacity))),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        if let Ok(mut cache) = self.cache.write() {
            cache.get(key)
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, value: V) -> Result<(), String> {
        match self.cache.write() {
            Ok(mut cache) => {
                cache.insert(key, value);
                Ok(())
            },
            Err(_) => Err("Failed to acquire write lock on LRU cache".to_string()),
        }
    }

    pub fn update(&self, key: &K, value: V) -> Result<bool, String> {
        match self.cache.write() {
            Ok(mut cache) => Ok(cache.update(key, value)),
            Err(_) => Err("Failed to acquire write lock on LRU cache".to_string()),
        }
    }

    pub fn remove(&self, key: &K) -> Result<bool, String> {
        match self.cache.write() {
            Ok(mut cache) => Ok(cache.remove(key)),
            Err(_) => Err("Failed to acquire write lock on LRU cache".to_string()),
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        match self.cache.write() {
            Ok(mut cache) => {
                cache.clear();
                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock on LRU cache".to_string()),
        }
    }

    pub fn len(&self) -> Result<usize, String> {
        match self.cache.read() {
            Ok(cache) => Ok(cache.len()),
            Err(_) => Err("Failed to acquire read lock on LRU cache".to_string()),
        }
    }

    pub fn is_empty(&self) -> Result<bool, String> {
        match self.cache.read() {
            Ok(cache) => Ok(cache.is_empty()),
            Err(_) => Err("Failed to acquire read lock on LRU cache".to_string()),
        }
    }
}

// Specialized LRU cache for autocomplete results
pub struct AutocompleteLRUCache {
    cache: ThreadSafeLRUCache<String, Vec<PathBuf>>,
}

impl AutocompleteLRUCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: ThreadSafeLRUCache::new(capacity),
        }
    }

    pub fn get_suggestions(&self, query: &str) -> Option<Vec<PathBuf>> {
        self.cache.get(&query.to_lowercase())
    }

    pub fn cache_suggestions(&self, query: &str, results: Vec<PathBuf>) -> Result<(), String> {
        self.cache.insert(query.to_lowercase(), results)
    }

    pub fn invalidate(&self, prefix: &str) -> Result<(), String> {
        // This is a simplistic implementation that clears the entire cache
        // when a path changes. A more sophisticated implementation would only
        // invalidate entries affected by the change.
        if prefix.is_empty() {
            self.cache.clear()
        } else {
            // For a more sophisticated approach, you could iterate through
            // all cached queries and remove those that contain the prefix
            Ok(())
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
        }
    }

    pub fn clear(&self) -> Result<(), String> {
        self.cache.clear()
    }

    pub fn len(&self) -> Result<usize, String> {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests_lru_cache {
    use std::sync::Barrier;
    use std::thread;
    use crate::{log_info, log_error};
    use super::*;

    // Basic LRUCache tests
    #[test]
    fn test_lru_cache_basic_operations() {
        log_info!("Starting basic LRU cache operations test");

        let mut cache = LRUCache::<String, i32>::new(3);

        // Test insertion
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"key1".to_string()), Some(1));
        assert_eq!(cache.get(&"key2".to_string()), Some(2));
        assert_eq!(cache.get(&"key3".to_string()), Some(3));

        log_info!("Basic insertion and retrieval tests passed");

        // Test LRU eviction
        cache.insert("key4".to_string(), 4);
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&"key1".to_string()), None); // key1 should be evicted
        assert_eq!(cache.get(&"key2".to_string()), Some(2));
        assert_eq!(cache.get(&"key3".to_string()), Some(3));
        assert_eq!(cache.get(&"key4".to_string()), Some(4));

        log_info!("LRU eviction test passed");

        // Test update
        assert!(cache.update(&"key3".to_string(), 33));
        assert_eq!(cache.get(&"key3".to_string()), Some(33));

        // Test non-existent key update
        assert!(!cache.update(&"key5".to_string(), 5));

        log_info!("Update operation tests passed");

        // Test remove
        assert!(cache.remove(&"key3".to_string()));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&"key3".to_string()), None);

        // Test removing non-existent key
        assert!(!cache.remove(&"key1".to_string()));

        log_info!("Remove operation tests passed");

        // Test clear
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        log_info!("Clear operation test passed");
    }

    #[test]
    fn test_lru_cache_order_update() {
        log_info!("Starting LRU cache order update test");

        let mut cache = LRUCache::<String, i32>::new(3);

        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3);

        // Access key1, making it most recently used
        assert_eq!(cache.get(&"key1".to_string()), Some(1));

        // Add key4, key2 should be evicted since key1 was recently used
        cache.insert("key4".to_string(), 4);

        assert_eq!(cache.get(&"key1".to_string()), Some(1));
        assert_eq!(cache.get(&"key2".to_string()), None); // key2 should be evicted
        assert_eq!(cache.get(&"key3".to_string()), Some(3));
        assert_eq!(cache.get(&"key4".to_string()), Some(4));

        log_info!("LRU cache order update test passed");
    }

    // ThreadSafeLRUCache tests
    #[test]
    fn test_thread_safe_lru_cache() {
        log_info!("Starting thread-safe LRU cache test");

        let cache = ThreadSafeLRUCache::<String, i32>::new(3);

        // Test basic operations
        cache.insert("key1".to_string(), 1).unwrap();
        cache.insert("key2".to_string(), 2).unwrap();
        cache.insert("key3".to_string(), 3).unwrap();

        assert_eq!(cache.len().unwrap(), 3);
        assert_eq!(cache.get(&"key1".to_string()), Some(1));

        // Test LRU eviction - the least recently used item (key2) should be evicted
        // since key1 was just accessed via the get() method
        cache.insert("key4".to_string(), 4).unwrap();
        assert_eq!(cache.len().unwrap(), 3);
        assert_eq!(cache.get(&"key1".to_string()), Some(1));
        assert_eq!(cache.get(&"key2".to_string()), None); // key2 should be evicted
        assert_eq!(cache.get(&"key3".to_string()), Some(3));
        assert_eq!(cache.get(&"key4".to_string()), Some(4));

        // Test update
        assert!(cache.update(&"key3".to_string(), 33).unwrap());
        assert_eq!(cache.get(&"key3".to_string()), Some(33));

        // Test remove
        assert!(cache.remove(&"key3".to_string()).unwrap());
        assert_eq!(cache.len().unwrap(), 2);

        // Test clear
        cache.clear().unwrap();
        assert_eq!(cache.len().unwrap(), 0);
        assert!(cache.is_empty().unwrap());

        log_info!("Thread-safe LRU cache basic test passed");
    }

    #[test]
    fn test_thread_safe_lru_cache_concurrency() {
        log_info!("Starting thread-safe LRU cache concurrency test");

        let cache = Arc::new(ThreadSafeLRUCache::<String, i32>::new(100));
        let threads_count = 10;
        let operations_per_thread = 100;

        let barrier = Arc::new(Barrier::new(threads_count));
        let mut handles = vec![];

        for i in 0..threads_count {
            let cache_clone = Arc::clone(&cache);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                // Wait for all threads to be ready
                barrier_clone.wait();

                let base = i * operations_per_thread;

                for j in 0..operations_per_thread {
                    let key = format!("key{}", base + j);
                    let value = (base + j) as i32;

                    // Mix of operations
                    match j % 4 {
                        0 => {
                            // Insert
                            if let Err(e) = cache_clone.insert(key.clone(), value) {
                                log_error!(format!("Thread {} failed to insert {}: {}", i, key, e).as_str());
                            }
                        },
                        1 => {
                            // Get
                            let result = cache_clone.get(&key);
                            if result.is_none() {
                                // This is not necessarily an error, as other threads may have caused eviction
                                log_info!(format!("Thread {} couldn't find {}", i, key).as_str());
                            }
                        },
                        2 => {
                            // Update
                            if let Err(e) = cache_clone.update(&key, value * 2) {
                                log_error!(format!("Thread {} failed to update {}: {}", i, key, e).as_str());
                            }
                        },
                        3 => {
                            // Remove
                            if let Err(e) = cache_clone.remove(&key) {
                                log_error!(format!("Thread {} failed to remove {}: {}", i, key, e).as_str());
                            }
                        },
                        _ => unreachable!(),
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify the cache is in a consistent state
        let len = cache.len().unwrap();
        log_info!(format!("Final cache size: {}", len).as_str());
        assert!(len <= 100); // Cache should not exceed its capacity

        log_info!("Thread-safe LRU cache concurrency test passed");
    }

    // AutocompleteLRUCache tests
    #[test]
    fn test_autocomplete_lru_cache() {
        log_info!("Starting autocomplete LRU cache test");

        let cache = AutocompleteLRUCache::new(3);

        // Test caching suggestions
        let results1 = vec![
            PathBuf::from("/path/to/file1.rs"),
            PathBuf::from("/path/to/file2.rs"),
        ];
        cache.cache_suggestions("rust", results1.clone()).unwrap();

        let results2 = vec![
            PathBuf::from("/path/to/file3.py"),
            PathBuf::from("/path/to/file4.py"),
        ];
        cache.cache_suggestions("python", results2.clone()).unwrap();

        let results3 = vec![
            PathBuf::from("/path/to/file5.js"),
            PathBuf::from("/path/to/file6.js"),
        ];
        cache.cache_suggestions("javascript", results3.clone()).unwrap();

        // Test retrieving suggestions
        assert_eq!(cache.get_suggestions("rust"), Some(results1.clone()));
        assert_eq!(cache.get_suggestions("python"), Some(results2.clone()));
        assert_eq!(cache.get_suggestions("javascript"), Some(results3.clone()));

        // Test LRU behavior by adding a fourth entry
        let results4 = vec![
            PathBuf::from("/path/to/file7.go"),
            PathBuf::from("/path/to/file8.go"),
        ];
        cache.cache_suggestions("golang", results4.clone()).unwrap();

        // The least recently used "rust" should be evicted
        assert_eq!(cache.get_suggestions("rust"), None);
        assert_eq!(cache.get_suggestions("python"), Some(results2.clone()));
        assert_eq!(cache.get_suggestions("javascript"), Some(results3));
        assert_eq!(cache.get_suggestions("golang"), Some(results4));

        // Test case insensitivity
        assert_eq!(cache.get_suggestions("PYTHON"), Some(results2));

        // Test invalidation
        cache.invalidate("py").unwrap();

        // Test clear
        cache.clear().unwrap();
        assert_eq!(cache.len().unwrap(), 0);

        log_info!("Autocomplete LRU cache test passed");
    }
}

#[cfg(test)]
mod benchmarks_lru_cache {
    use std::sync::Barrier;
    use std::thread;
    use std::time::Instant;
    use super::*;
    use crate::log_info;

    #[cfg(feature = "bench")]
    #[test]
    fn bench_lru_cache_insert() {
        let mut cache = LRUCache::<String, i32>::new(1000);
        let iterations = 10000;

        let start = Instant::now();
        for i in 0..iterations {
            let key = format!("key{}", i);
            cache.insert(key, i);
        }
        let duration = start.elapsed();

        log_info!(format!("LRU cache insertion of {} items took: {:?}", iterations, duration).as_str());
        log_info!(format!("Average insertion time: {:?} per item", duration / iterations as u32).as_str());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn bench_lru_cache_get_hit() {
        let mut cache = LRUCache::<String, i32>::new(1000);

        // Fill cache
        for i in 0..900 {
            let key = format!("key{}", i);
            cache.insert(key, i);
        }

        // Add the key we'll look for repeatedly
        let test_key = "test_key".to_string();
        cache.insert(test_key.clone(), 1000);

        let iterations = 10000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = cache.get(&test_key);
        }
        let duration = start.elapsed();

        log_info!(format!("LRU cache {} gets (hits) took: {:?}", iterations, duration).as_str());
        log_info!(format!("Average get (hit) time: {:?} per item", duration / iterations as u32).as_str());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn bench_lru_cache_get_miss() {
        let mut cache = LRUCache::<String, i32>::new(1000);

        // Fill cache with different keys
        for i in 0..900 {
            let key = format!("key{}", i);
            cache.insert(key, i);
        }

        // Use a key that's not in the cache
        let missing_key = "missing_key".to_string();

        let iterations = 10000;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = cache.get(&missing_key);
        }
        let duration = start.elapsed();

        log_info!(format!("LRU cache {} gets (misses) took: {:?}", iterations, duration).as_str());
        log_info!(format!("Average get (miss) time: {:?} per item", duration / iterations as u32).as_str());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn bench_lru_cache_update() {
        let mut cache = LRUCache::<String, i32>::new(1000);

        // Fill cache
        for i in 0..900 {
            let key = format!("key{}", i);
            cache.insert(key, i);
        }

        // Update a specific key repeatedly
        let test_key = "key500".to_string();
        let iterations = 10000;

        let start = Instant::now();
        for val in 0..iterations {
            cache.update(&test_key, val);
        }
        let duration = start.elapsed();

        log_info!(format!("LRU cache {} updates took: {:?}", iterations, duration).as_str());
        log_info!(format!("Average update time: {:?} per item", duration / iterations as u32).as_str());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn bench_thread_safe_lru_cache_concurrent_access() {
        let threads_count = 4;
        let cache = Arc::new(ThreadSafeLRUCache::<String, i32>::new(1000));
        let iterations = 1000;

        // Fill cache
        for i in 0..900 {
            let key = format!("key{}", i);
            cache.insert(key, i).unwrap();
        }

        let start = Instant::now();

        let barrier = Arc::new(Barrier::new(threads_count + 1)); // +1 for the main thread
        let mut handles = vec![];

        for i in 0..threads_count {
            let cache_clone = Arc::clone(&cache);
            let barrier_clone = Arc::clone(&barrier);

            let handle = thread::spawn(move || {
                barrier_clone.wait();

                for _ in 0..iterations {
                    // Each thread performs a different operation
                    match i % 4 {
                        0 => {
                            // Insert
                            let key = format!("thread_key{}", i);
                            let _ = cache_clone.insert(key, i as i32);
                        },
                        1 => {
                            // Get
                            let key = format!("key{}", i * 10);
                            let _ = cache_clone.get(&key);
                        },
                        2 => {
                            // Update
                            let key = format!("key{}", i * 5);
                            let _ = cache_clone.update(&key, i as i32 * 2);
                        },
                        3 => {
                            // Remove
                            let key = format!("key{}", i * 7);
                            let _ = cache_clone.remove(&key);
                        },
                        _ => unreachable!(),
                    }
                }
            });

            handles.push(handle);
        }

        // Start all threads simultaneously
        barrier.wait();

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let duration = start.elapsed();
        let total_operations = threads_count * iterations;

        log_info!(format!("Thread-safe LRU cache {} concurrent operations took: {:?}",
            total_operations, duration).as_str());
        log_info!(format!("Average concurrent operation time: {:?} per operation",
            duration / total_operations as u32).as_str());
    }

    #[test]
    #[cfg(feature = "bench")]
    fn bench_autocomplete_lru_cache() {
        let cache = AutocompleteLRUCache::new(1000);
        let iterations = 1000;

        // Fill with some sample data
        for i in 0..10 {
            let query = format!("query{}", i);
            let results = vec![
                PathBuf::from(format!("/path/to/file{}.rs", i * 2)),
                PathBuf::from(format!("/path/to/file{}.rs", i * 2 + 1)),
            ];
            cache.cache_suggestions(&query, results).unwrap();
        }

        let start = Instant::now();
        for i in 0..iterations {
            let query = format!("query{}", i % 10);
            let _ = cache.get_suggestions(&query);
        }
        let duration = start.elapsed();

        log_info!(format!("Autocomplete LRU cache {} gets took: {:?}", iterations, duration).as_str());
        log_info!(format!("Average get suggestion time: {:?} per item", duration / iterations as u32).as_str());
    }
}
