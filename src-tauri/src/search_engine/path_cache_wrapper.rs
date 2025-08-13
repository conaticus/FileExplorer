use crate::search_engine::lru_cache_v2::LruPathCache;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct CachedSearchResults {
    pub results: Vec<(String, f32)>,
}

pub struct PathCache {
    inner: Arc<RwLock<LruPathCache<String, PathData>>>,
}

// explicitly Send+Sync
unsafe impl Send for PathCache {}
unsafe impl Sync for PathCache {}

#[derive(Clone)]
pub struct PathData {
    pub results: Vec<(String, f32)>,
}

impl PathCache {
    #[cfg(test)]
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LruPathCache::new(capacity))),
        }
    }

    #[inline]
    pub fn with_ttl(capacity: usize, ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(LruPathCache::with_ttl(capacity, ttl))),
        }
    }

    #[inline]
    pub fn get(&mut self, path: &str) -> Option<PathData> {
        self.inner.write().get(&path.to_string())
    }

    #[inline]
    pub fn insert(&mut self, query: String, results: Vec<(String, f32)>) {
        let data = PathData { results };
        self.put_data(query, data);
    }
    
    #[inline]  
    pub fn put(&mut self, query: String, data: crate::search_engine::path_cache_wrapper::CachedSearchResults) {
        let path_data = PathData { results: data.results };
        self.put_data(query, path_data);
    }
    
    #[inline]
    fn put_data(&mut self, query: String, data: PathData) {
        self.inner.write().put(query, data);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    #[cfg(test)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.write().clear();
    }

    #[inline]
    pub fn purge_expired(&mut self) -> usize {
        self.inner.write().purge_expired()
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

        // Wait for the entries to expire
        sleep(ttl + Duration::from_millis(10));

        // The entries should have expired, but we won't call get() as that might remove them.
        // Instead, we'll rely on purge_expired to do the cleanup and report the count.

        // Add a fresh entry
        cache.insert(
            "/path/to/file4".to_string(),
            vec![("/path/to/file4".to_string(), 4.0)],
        );

        // file1, file2, and file3 should expire, but file4 should remain
        let purged = cache.purge_expired();
        assert_eq!(purged, 3);
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

        log_info!("Time to insert 20 items with eviction: {:?}", elapsed);

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

        log_info!("Evicted {} items from the middle range", evicted_count);
    }
}