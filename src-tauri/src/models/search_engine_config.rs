use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::models::ranking_config::RankingConfig;

/// Configuration options for the search engine.
///
/// Defines adjustable parameters that control search engine behavior,
/// including result limits, file type preferences, and indexing constraints.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SearchEngineConfig {
    pub search_engine_enabled: bool,
    pub max_results: usize,
    pub preferred_extensions: Vec<String>,
    pub excluded_patterns: Option<Vec<String>>,
    pub cache_size: usize,
    pub ranking_config: RankingConfig,
    pub prefer_directories: bool,
    pub cache_ttl: Option<Duration>,
    // To be implemented
    pub collect_usage_stats: bool,
    pub indexing_logging_enabled: bool,
    pub search_logging_enabled: bool,
    pub search_timeout_ms: Option<u64>,
    pub result_score_threshold: Option<f32>,
    pub min_query_length: Option<usize>,
    pub max_indexed_files: Option<usize>,
    pub max_index_depth: Option<usize>,
    pub index_hidden_files: bool,
    pub follow_symlinks: bool,
    pub fuzzy_trigram_threshold: Option<f32>,
    pub fuzzy_search_enabled: bool,
    pub case_sensitive_search: bool,
    pub group_results_by_directory: bool,
    pub persistent_index_path: Option<String>,
    pub index_compression_enabled: bool,
    pub indexing_priority: Option<u8>,
    pub default_search_operator: Option<String>,
    pub enable_wildcard_search: bool,
    pub indexing_batch_size: Option<usize>,
    pub retry_failed_indexing: bool,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            search_engine_enabled: true,
            max_results: 20,
            ranking_config: RankingConfig::default(),
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
            excluded_patterns: Some(vec![
                ".git".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
            ]),
            cache_size: 1000,
            
            cache_ttl: Duration::from_secs(300).into(), // 5 minutes
            prefer_directories: false,
            collect_usage_stats: true,
            indexing_logging_enabled: false,
            search_logging_enabled: false,
            search_timeout_ms: Some(5000), // 5 seconds
            result_score_threshold: Some(0.1),
            min_query_length: None,           
            max_indexed_files: None, 
            max_index_depth: None,            
            index_hidden_files: false, 
            follow_symlinks: false,  
            fuzzy_trigram_threshold: Some(0.5),
            fuzzy_search_enabled: true,
            case_sensitive_search: false,
            group_results_by_directory: true,
            persistent_index_path: None,
            index_compression_enabled: true,
            indexing_priority: Some(1),
            default_search_operator: Some("AND".to_string()), 
            enable_wildcard_search: false,
            indexing_batch_size: Some(100), 
            retry_failed_indexing: true, 
        }
    }
}