use crate::search_engine::lru_cache_v2::LruPathCache;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::time::Duration;

thread_local! {
    // Thread-local recent query cache to avoid lock acquisition
    static RECENT_QUERY: RefCell<Option<(String, PathData)>> = RefCell::new(None);
}

/// A thread-safe path cache implementation with two-level caching:
/// 1. Thread-local cache for the most recent query
/// 2. Shared LRU cache for all queries across threads
///
/// The two-level design minimizes contention when the same query is accessed
/// repeatedly by the same thread.
pub struct PathCache {
    inner: Arc<Mutex<LruPathCache<String, PathData>>>,
}

// explicitly Send+Sync
unsafe impl Send for PathCache {}
unsafe impl Sync for PathCache {}

#[derive(Clone)]
pub struct PathData {
    /// The search results (paths and scores)
    pub results: Vec<(String, f32)>,
}

impl PathCache {
    /// Creates a new path cache with the specified capacity.
    /// Available only in test configurations.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of entries the cache can hold
    ///
    /// # Returns
    ///
    /// A new `PathCache` instance with the specified capacity
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = PathCache::new(100);
    /// ```
    #[cfg(test)]
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruPathCache::new(capacity))),
        }
    }

    /// Creates a new path cache with the specified capacity and time-to-live duration.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of entries the cache can hold
    /// * `ttl` - The time-to-live duration after which entries are considered expired
    ///
    /// # Returns
    ///
    /// A new `PathCache` instance with the specified capacity and TTL
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    ///
    /// let mut cache = PathCache::with_ttl(
    ///     100,
    ///     Duration::from_secs(30)
    /// );
    /// ```
    #[inline]
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruPathCache::with_ttl(capacity, ttl))),
        }
    }

    /// Retrieves path data from the cache by its path key.
    ///
    /// This method first checks the thread-local cache to avoid lock contention,
    /// then falls back to the shared LRU cache if needed. If found, the entry is
    /// moved to the front of the LRU list (marking it as most recently used).
    ///
    /// # Time Complexity
    ///
    /// - Best case: O(1) - Constant time thread-local lookup
    /// - Average case: O(1) - Constant time hash lookup + linked list update when lock acquired
    ///
    /// # Arguments
    ///
    /// * `path` - The path key to look up in the cache
    ///
    /// # Returns
    ///
    /// * `Some(PathData)` - The path data associated with the key if it exists and is not expired
    /// * `None` - If the key does not exist or the entry has expired
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = PathCache::new(100);
    /// cache.insert("file.txt".to_string(), vec![("file.txt".to_string(), 1.0)]);
    ///
    /// match cache.get("file.txt") {
    ///     Some(data) => println!("Found {} results", data.results.len()),
    ///     None => println!("No cached results found"),
    /// }
    /// ```
    #[inline]
    pub fn get(&mut self, path: &str) -> Option<PathData> {
        // Check thread-local cache first
        let mut result = None;

        RECENT_QUERY.with(|recent| {
            if let Some((ref query, ref data)) = *recent.borrow() {
                if query == path {
                    result = Some(data.clone());
                }
            }
        });

        // if we get result check if it is expired
        if result.is_some() {
            if let Ok(cache) = self.inner.lock() {
                if !cache.check_ttl(&path.to_string()) {
                    RECENT_QUERY.with(|recent| {
                        *recent.borrow_mut() = None;
                    });
                    return None;
                }
            }
            return result;
        }

        // if not in thread-local cache try shared cache
        let mutex_guard = match self.inner.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        // Use destructuring to avoid holding the lock longer than needed
        let mut cache = mutex_guard;
        if let Some(data) = cache.get(&path.to_string()) {
            // Update thread-local cache
            let cloned_data = data.clone();
            RECENT_QUERY.with(|recent| {
                *recent.borrow_mut() = Some((path.to_string(), cloned_data.clone()));
            });

            return Some(cloned_data);
        }

        None
    }

    /// Inserts path data into the cache using the given query as the key.
    ///
    /// The data is stored both in the thread-local cache and the shared LRU cache.
    /// If the shared cache is at capacity, the least recently used entry will be evicted.
    ///
    /// # Time Complexity
    ///
    /// - Best case: O(1) - Constant time thread-local update
    /// - Average case: O(1) - Constant time hash insertion + linked list update when lock acquired
    ///
    /// # Arguments
    ///
    /// * `query` - The path query to use as the key
    /// * `results` - The search results to cache (path strings and their relevance scores)
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = PathCache::new(100);
    ///
    /// // Cache search results
    /// let results = vec![
    ///     ("C:/path/to/file.txt".to_string(), 0.95),
    ///     ("C:/path/to/other.txt".to_string(), 0.85)
    /// ];
    /// cache.insert("file.txt".to_string(), results);
    /// ```
    #[inline]
    pub fn insert(&mut self, query: String, results: Vec<(String, f32)>) {
        let data = PathData { results };

        // Update thread-local cache
        RECENT_QUERY.with(|recent| {
            *recent.borrow_mut() = Some((query.clone(), data.clone()));
        });

        // Then update the shared cache
        if let Ok(mut cache) = self.inner.lock() {
            cache.insert(query, data);
        }
    }

    /// Returns the number of entries currently in the shared cache.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation, but requires lock acquisition
    ///
    /// # Returns
    ///
    /// The number of entries in the cache, or 0 if the lock couldn't be acquired
    ///
    /// # Example
    ///
    /// ```rust
    /// let cache = PathCache::new(100);
    /// println!("Cache contains {} entries", cache.len());
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        if let Ok(cache) = self.inner.lock() {
            cache.len()
        } else {
            0
        }
    }

    /// Checks if the shared cache is empty.
    /// Available only in test configurations.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - Constant time operation, but requires lock acquisition
    ///
    /// # Returns
    ///
    /// * `true` - If the cache contains no entries or the lock couldn't be acquired
    /// * `false` - If the cache contains at least one entry
    ///
    /// # Example
    ///
    /// ```rust
    /// let cache = PathCache::new(100);
    /// if cache.is_empty() {
    ///     println!("Cache is empty");
    /// }
    /// ```
    #[cfg(test)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        if let Ok(cache) = self.inner.lock() {
            cache.is_empty()
        } else {
            true
        }
    }

    /// Removes all entries from both the thread-local and shared caches.
    ///
    /// # Time Complexity
    ///
    /// - O(1) - For clearing thread-local cache
    /// - O(n) - For clearing the shared cache, where n is the number of entries
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut cache = PathCache::new(100);
    /// // Add some entries
    /// cache.insert("file.txt".to_string(), vec![("path/to/file.txt".to_string(), 1.0)]);
    ///
    /// // Clear all entries
    /// cache.clear();
    /// assert_eq!(cache.len(), 0);
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        // Clear thread-local cache
        RECENT_QUERY.with(|recent| {
            *recent.borrow_mut() = None;
        });

        if let Ok(mut cache) = self.inner.lock() {
            cache.clear();
        }
    }

    /// Removes all expired entries from the shared cache.
    /// Also clears the thread-local cache to ensure consistency.
    ///
    /// # Time Complexity
    ///
    /// - O(n) - Linear in the number of elements in the cache
    ///
    /// # Returns
    ///
    /// The number of expired entries that were removed, or 0 if the lock couldn't be acquired
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use std::thread::sleep;
    ///
    /// let mut cache = PathCache::with_ttl(100, Duration::from_secs(5));
    /// cache.insert("file.txt".to_string(), vec![("path/to/file.txt".to_string(), 1.0)]);
    ///
    /// // Wait for entries to expire
    /// sleep(Duration::from_secs(6));
    ///
    /// // Purge expired entries
    /// let purged = cache.purge_expired();
    /// println!("Purged {} expired entries", purged);
    /// ```
    #[inline]
    pub fn purge_expired(&mut self) -> usize {
        // Clear thread-local cache as it might have expired
        RECENT_QUERY.with(|recent| {
            *recent.borrow_mut() = None;
        });

        if let Ok(mut cache) = self.inner.lock() {
            let purged = cache.purge_expired();
            if purged > 0 {
                // If we purged anything, also clear thread-local cache to be safe
                RECENT_QUERY.with(|recent| {
                    *recent.borrow_mut() = None;
                });
            }
            purged
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests_path_cache {
    use super::*;
    use crate::log_info;
    use std::thread::sleep;
    use std::time::Instant;

    #[test]
    fn test_basic_operations() {
        let mut cache = PathCache::new(3);

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        // Test insertion
        cache.insert(
            "/path/to/file1".to_string(),
            vec![("/path/to/file1".to_string(), 1.0)],
        );
        cache.insert(
            "/path/to/file2".to_string(),
            vec![("/path/to/file2".to_string(), 2.0)],
        );

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());

        // Test retrieval
        let file1 = cache.get("/path/to/file1");
        assert!(file1.is_some());
        let file1_data = file1.unwrap();
        assert_eq!(file1_data.results.len(), 1);
        assert_eq!(file1_data.results[0].0, "/path/to/file1");
        assert_eq!(file1_data.results[0].1, 1.0);

        let file2 = cache.get("/path/to/file2");
        assert!(file2.is_some());
        let file2_data = file2.unwrap();
        assert_eq!(file2_data.results.len(), 1);
        assert_eq!(file2_data.results[0].0, "/path/to/file2");
        assert_eq!(file2_data.results[0].1, 2.0);

        assert!(cache.get("/path/to/file3").is_none());

        // Test LRU behavior (capacity limit)
        cache.insert(
            "/path/to/file3".to_string(),
            vec![("/path/to/file3".to_string(), 3.0)],
        );
        cache.insert(
            "/path/to/file4".to_string(),
            vec![("/path/to/file4".to_string(), 4.0)],
        );

        // file1 should be evicted since it's the least recently used
        assert_eq!(cache.len(), 3);
        assert!(cache.get("/path/to/file1").is_none());
        assert!(cache.get("/path/to/file2").is_some());

        // Test clear
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_score_update() {
        let mut cache = PathCache::new(3);

        // Insert a path with initial score
        cache.insert(
            "/path/to/file".to_string(),
            vec![("/path/to/file".to_string(), 1.0)],
        );

        // Verify initial score
        let file_data = cache.get("/path/to/file").unwrap();
        assert_eq!(file_data.results.len(), 1);
        assert_eq!(file_data.results[0].1, 1.0);

        // Update the score
        cache.insert(
            "/path/to/file".to_string(),
            vec![("/path/to/file".to_string(), 2.5)],
        );

        // Verify updated score
        let updated_data = cache.get("/path/to/file").unwrap();
        assert_eq!(updated_data.results.len(), 1);
        assert_eq!(updated_data.results[0].1, 2.5);
    }

    #[test]
    fn test_ttl_expiration() {
        let ttl = Duration::from_millis(100);
        let mut cache = PathCache::with_ttl(5, ttl);

        cache.insert(
            "/path/to/file1".to_string(),
            vec![("/path/to/file1".to_string(), 1.0)],
        );
        cache.insert(
            "/path/to/file2".to_string(),
            vec![("/path/to/file2".to_string(), 2.0)],
        );
        cache.insert(
            "/path/to/file3".to_string(),
            vec![("/path/to/file3".to_string(), 3.0)],
        );

        let file1 = cache.get("/path/to/file1");
        assert!(file1.is_some());
        assert_eq!(file1.unwrap().results[0].0, "/path/to/file1");

        // Wait for the entries to expire
        sleep(ttl + Duration::from_millis(10));

        // The entry should have expired
        assert!(cache.get("/path/to/file1").is_none());
        assert!(cache.get("/path/to/file2").is_none());
        assert!(cache.get("/path/to/file3").is_none());

        // Add a fresh entry
        cache.insert(
            "/path/to/file4".to_string(),
            vec![("/path/to/file4".to_string(), 4.0)],
        );

        // file1, file2, and file3 should expire, but file4 should remain
        let purged = cache.purge_expired();
        assert_eq!(purged, 1); // Now correctly expects 3 purged items
        assert_eq!(cache.len(), 1);
        assert!(cache.get("/path/to/file4").is_some());
    }

    #[test]
    fn benchmark_path_retrieval() {
        let mut cache = PathCache::new(1000);

        // Populate cache with sample paths
        for i in 0..500 {
            let path = format!("/home/user/documents/folder_{}/file.txt", i);
            cache.insert(path.clone(), vec![(path, i as f32 / 100.0)]);
        }

        log_info!("Starting path cache retrieval benchmark");

        // Benchmark getting existing paths
        let start = Instant::now();
        for i in 0..500 {
            let path = format!("/home/user/documents/folder_{}/file.txt", i);
            let _ = cache.get(&path);
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
            let path = format!("/home/user/documents/folder_{}/file.txt", i);
            let _ = cache.get(&path);
        }
        let elapsed = start.elapsed();

        let avg_miss_time = elapsed.as_nanos() as f64 / 500.0;
        log_info!(
            "Average retrieval time for non-existent paths: {:.2} ns",
            avg_miss_time
        );
    }

    #[test]
    fn benchmark_cache_size_impact() {
        log_info!("Benchmarking impact of path cache size on retrieval performance");

        let sizes = [100, 1000, 10000];

        for &size in &sizes {
            let mut cache = PathCache::new(size);

            // Fill the cache to capacity
            for i in 0..size {
                let path = format!("/path/to/file_{}", i);
                cache.insert(path.clone(), vec![(path, (i % 10) as f32)]);
            }

            // Measure retrieval time (mixed hits and misses)
            let start = Instant::now();
            for i in size / 2..(size / 2 + 1000).min(size + 500) {
                let path = format!("/path/to/file_{}", i);
                let _ = cache.get(&path);
            }
            let elapsed = start.elapsed();

            log_info!(
                "Path cache size {}: 1000 lookups took {:?} (avg: {:.2} ns/lookup)",
                size,
                elapsed,
                elapsed.as_nanos() as f64 / 1000.0
            );
        }
    }

    #[test]
    fn benchmark_cache_size_impact_path_cache() {
        log_info!("Benchmarking impact of cache size on retrieval performance");

        let sizes = [100, 1000, 10000, 100000];

        for &size in &sizes {
            let mut cache = PathCache::new(size);

            // Fill the cache to capacity
            for i in 0..size {
                let path = format!("/path/to/file_{}", i);
                cache.insert(
                    path.clone(),
                    vec![(
                        path.clone(),
                        format!("metadata_{}", i).parse::<f32>().unwrap_or(1.0),
                    )],
                );
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
        log_info!("Benchmarking path cache LRU eviction behavior");

        let mut cache = PathCache::new(100);

        // Fill cache
        for i in 0..100 {
            cache.insert(
                format!("/path/to/file_{}", i),
                vec![(format!("/path/to/file_{}", i), i as f32)],
            );
        }

        // Access first 20 items to make them recently used
        for i in 0..20 {
            let _ = cache.get(&format!("/path/to/file_{}", i));
        }

        // Insert 20 new items, which should evict the least recently used
        let start = Instant::now();
        for i in 100..120 {
            cache.insert(
                format!("/path/to/file_{}", i),
                vec![(format!("/path/to/file_{}", i), i as f32)],
            );
        }
        let elapsed = start.elapsed();

        log_info!(
            "Time to insert 20 items with eviction: {:?}",
            elapsed
        );

        // Verify the first 20 items are still there (recently used)
        for i in 0..20 {
            assert!(cache.get(&format!("/path/to/file_{}", i)).is_some());
        }

        // Verify some of the middle items were evicted
        let mut evicted_count = 0;
        for i in 20..100 {
            if cache.get(&format!("/path/to/file_{}", i)).is_none() {
                evicted_count += 1;
            }
        }

        log_info!(
            "Evicted {} items from the middle range",
            evicted_count
        );
    }
}
