use serde::{Deserialize, Serialize};

/// Configuration options for the search engine.
///
/// Defines adjustable parameters that control search engine behavior,
/// including result limits, file type preferences, and indexing constraints.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchEngineConfig {
    pub search_engine_enabled: bool,
    pub max_results: usize,
    pub preferred_extensions: Vec<String>,
    pub indexing_depth: Option<usize>, // None means unlimited depth
    pub excluded_patterns: Vec<String>,
    pub cache_size: usize,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            search_engine_enabled: true,
            max_results: 20,
            preferred_extensions: vec![
                "txt".to_string(),
                "pdf".to_string(),
                "docx".to_string(),
                "xlsx".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "js".to_string(),
                "html".to_string(),
                "css".to_string(),
                "json".to_string(),
                "png".to_string(),
                "jpg".to_string(),
            ],
            indexing_depth: None,
            excluded_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ],
            cache_size: 1000,

        }
    }
}