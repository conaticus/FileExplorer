use serde::{Deserialize, Serialize};

/// Configuration for path ranking algorithm with adjustable weights.
///
/// This struct allows fine-tuning the relative importance of different
/// ranking factors like frequency, recency, directory context, and 
/// file extension preferences.
///
/// # Example
/// ```
/// let config = RankingConfig {
///     frequency_weight: 0.1,
///     max_frequency_boost: 0.6,
///     ..RankingConfig::default()
/// };
/// ```
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RankingConfig {
    /// Weight per usage count (frequency boost multiplier)
    pub frequency_weight: f32,
    /// Maximum cap for frequency boost
    pub max_frequency_boost: f32,
    /// Base weight for recency boost
    pub recency_weight: f32,
    /// Decay rate for recency (per second)
    pub recency_lambda: f32,
    /// Boost when path is in the exact current directory
    pub context_same_dir_boost: f32,
    /// Boost when path is in the parent of the current directory
    pub context_parent_dir_boost: f32,
    /// Multiplier for extension-based boost
    pub extension_boost: f32,
    /// Additional boost if query contains the extension
    pub extension_query_boost: f32,
    /// Boost for exact filename matches
    pub exact_match_boost: f32,
    /// Boost for filename prefix matches
    pub prefix_match_boost: f32,
    /// Boost for filename contains matches
    pub contains_match_boost: f32,
    /// Boost for directory matches
    pub directory_ranking_boost: f32,
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            frequency_weight: 0.05,
            max_frequency_boost: 0.5,
            recency_weight: 1.5,
            recency_lambda: 1.0 / 86400.0,
            context_same_dir_boost: 0.4,
            context_parent_dir_boost: 0.2,
            extension_boost: 2.0,
            extension_query_boost: 0.25,
            exact_match_boost: 1.0,
            prefix_match_boost: 0.3,
            contains_match_boost: 0.1,
            directory_ranking_boost: 0.2,
        }
    }
}