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

    /// Optional metadata about the path
    //pub metadata: Option<PathMetadata>, // maybe useful but idk?
}

impl PathCache {
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
            //metadata,
        };
        self.inner.insert(path, data);
    }

    pub fn remove(&mut self, path: &str) -> bool {
        self.inner.remove(&path.to_string())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

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