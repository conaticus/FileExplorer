use std::time::Duration;
use crate::search_engine::lru_cache_v2::LruPathCache;

pub struct PathCache {
    inner: LruPathCache<String, PathData>,
}

#[derive(Clone)]
pub struct PathData {
    /// The full path string
    pub path: String,

    /// The path's score for ranking in autocompletion (higher is better)
    pub score: f32,
}

impl PathCache {
    #[cfg(test)]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: LruPathCache::new(capacity),
        }
    }

    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        Self {
            inner: LruPathCache::with_ttl(capacity, ttl),
        }
    }

    pub fn get(&mut self, path: &str) -> Option<PathData> {
        self.inner.get(&path.to_string())
    }

    pub fn insert(&mut self, path: String, score: f32) {
        let data = PathData {
            path: path.clone(),
            score,
        };
        self.inner.insert(path, data);
    }

    pub fn remove(&mut self, path: &str) -> bool {
        self.inner.remove(&path.to_string())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn purge_expired(&mut self) -> usize {
        self.inner.purge_expired()
    }
}

#[cfg(test)]
mod tests_path_cache {
    use super::*;
    use crate::log_info;
    use std::time::Instant;
    use std::thread::sleep;

    #[test]
    fn test_basic_operations() {
        let mut cache = PathCache::new(3);

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        // Test insertion
        cache.insert("/path/to/file1".to_string(), 1.0);
        cache.insert("/path/to/file2".to_string(), 2.0);

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());

        // Test retrieval
        let file1 = cache.get("/path/to/file1");
        assert!(file1.is_some());
        let file1_data = file1.unwrap();
        assert_eq!(file1_data.path, "/path/to/file1");
        assert_eq!(file1_data.score, 1.0);

        let file2 = cache.get("/path/to/file2");
        assert!(file2.is_some());
        let file2_data = file2.unwrap();
        assert_eq!(file2_data.path, "/path/to/file2");
        assert_eq!(file2_data.score, 2.0);

        assert!(cache.get("/path/to/file3").is_none());

        // Test LRU behavior (capacity limit)
        cache.insert("/path/to/file3".to_string(), 3.0);
        cache.insert("/path/to/file4".to_string(), 4.0);

        // file1 should be evicted since it's the least recently used
        assert_eq!(cache.len(), 3);
        assert!(cache.get("/path/to/file1").is_none());
        assert!(cache.get("/path/to/file2").is_some());

        // Test removal
        assert!(cache.remove("/path/to/file3"));
        assert_eq!(cache.len(), 2);
        assert!(cache.get("/path/to/file3").is_none());

        // Test clear
        cache.clear();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_score_update() {
        let mut cache = PathCache::new(3);

        // Insert a path with initial score
        cache.insert("/path/to/file".to_string(), 1.0);

        // Verify initial score
        let file_data = cache.get("/path/to/file").unwrap();
        assert_eq!(file_data.score, 1.0);

        // Update the score
        cache.insert("/path/to/file".to_string(), 2.5);

        // Verify updated score
        let updated_data = cache.get("/path/to/file").unwrap();
        assert_eq!(updated_data.score, 2.5);
    }

    #[test]
    fn test_ttl_expiration() {
        let ttl = Duration::from_millis(100);
        let mut cache = PathCache::with_ttl(5, ttl);

        cache.insert("/path/to/file1".to_string(), 1.0);
        let file1 = cache.get("/path/to/file1");
        assert!(file1.is_some());
        assert_eq!(file1.unwrap().path, "/path/to/file1");

        // Wait for the entry to expire
        sleep(ttl + Duration::from_millis(10));

        // The entry should have expired
        assert!(cache.get("/path/to/file1").is_none());

        // Test purge_expired
        cache.insert("/path/to/file2".to_string(), 2.0);
        cache.insert("/path/to/file3".to_string(), 3.0);

        sleep(ttl + Duration::from_millis(10));

        // Add a fresh entry
        cache.insert("/path/to/file4".to_string(), 4.0);

        // file2 and file3 should expire, but file4 should remain
        let purged = cache.purge_expired();
        assert_eq!(purged, 2);
        assert_eq!(cache.len(), 1);
        assert!(cache.get("/path/to/file4").is_some());
    }

    #[test]
    fn benchmark_path_retrieval() {
        let mut cache = PathCache::new(1000);

        // Populate cache with sample paths
        for i in 0..500 {
            let path = format!("/home/user/documents/folder_{}/file.txt", i);
            cache.insert(path, i as f32 / 100.0);
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
        log_info!(&format!("Average retrieval time for existing paths: {:.2} ns", avg_retrieval_time));

        // Benchmark getting non-existent paths
        let start = Instant::now();
        for i in 1000..1500 {
            let path = format!("/home/user/documents/folder_{}/file.txt", i);
            let _ = cache.get(&path);
        }
        let elapsed = start.elapsed();

        let avg_miss_time = elapsed.as_nanos() as f64 / 500.0;
        log_info!(&format!("Average retrieval time for non-existent paths: {:.2} ns", avg_miss_time));
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
                cache.insert(path, (i % 10) as f32);
            }

            // Measure retrieval time (mixed hits and misses)
            let start = Instant::now();
            for i in size/2..(size/2 + 1000).min(size + 500) {
                let path = format!("/path/to/file_{}", i);
                let _ = cache.get(&path);
            }
            let elapsed = start.elapsed();

            log_info!(&format!("Path cache size {}: 1000 lookups took {:?} (avg: {:.2} ns/lookup)",
                    size,
                    elapsed,
                    elapsed.as_nanos() as f64 / 1000.0));
        }
    }

    #[test]
    fn benchmark_lru_behavior() {
        log_info!("Benchmarking path cache LRU eviction behavior");

        let mut cache = PathCache::new(100);

        // Fill cache
        for i in 0..100 {
            cache.insert(format!("/path/to/file_{}", i), i as f32);
        }

        // Access first 20 items to make them recently used
        for i in 0..20 {
            let _ = cache.get(&format!("/path/to/file_{}", i));
        }

        // Insert 20 new items, which should evict the least recently used
        let start = Instant::now();
        for i in 100..120 {
            cache.insert(format!("/path/to/file_{}", i), i as f32);
        }
        let elapsed = start.elapsed();

        log_info!(&format!("Time to insert 20 items with eviction: {:?}", elapsed));

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

        log_info!(&format!("Evicted {} items from the middle range", evicted_count));
    }
}
