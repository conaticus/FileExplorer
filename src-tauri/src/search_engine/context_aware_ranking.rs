use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

pub struct RankingFactors {
    pub recency_weight: f64,
    pub frequency_weight: f64,
    pub proximity_weight: f64,
    pub extension_weights: Vec<(String, f64)>,
    pub max_recency_score: f64,
}

impl Default for RankingFactors {
    fn default() -> Self {
        Self {
            recency_weight: 0.3,
            frequency_weight: 0.4,
            proximity_weight: 0.3,
            extension_weights: vec![
                // Higher weights for common code files
                ("rs".to_string(), 1.2),
                ("go".to_string(), 1.1),
                ("py".to_string(), 1.1),
                ("js".to_string(), 1.0),
                ("ts".to_string(), 1.0),
                ("c".to_string(), 1.0),
                ("cpp".to_string(), 1.0),
                ("h".to_string(), 1.0),
                ("hpp".to_string(), 1.0),
                // Config files are often important
                ("toml".to_string(), 0.9),
                ("json".to_string(), 0.9),
                ("yaml".to_string(), 0.9),
                ("yml".to_string(), 0.9),
                // Documentation files
                ("md".to_string(), 0.8),
                ("txt".to_string(), 0.7),
                // Other files
                ("pdf".to_string(), 0.6),
                ("doc".to_string(), 0.6),
                ("docx".to_string(), 0.6),
            ],
            max_recency_score: 1.0,
        }
    }
}

pub struct ContextAwareRanker {
    pub current_directory: PathBuf,
    pub factors: RankingFactors,
    pub recency_data: Arc<RwLock<HashMap<PathBuf, SystemTime>>>,
    pub frequency_data: Arc<RwLock<HashMap<PathBuf, u32>>>,
}

impl ContextAwareRanker {
    pub fn new(current_directory: PathBuf) -> Self {
        Self {
            current_directory,
            factors: RankingFactors::default(),
            recency_data: Arc::new(RwLock::new(HashMap::new())),
            frequency_data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn update_current_directory(&mut self, dir: PathBuf) {
        self.current_directory = dir;
    }

    pub fn record_access(&self, path: &Path) {
        self.update_recency(path);
        self.increment_frequency(path);
    }

    fn update_recency(&self, path: &Path) {
        if let Ok(mut recency_data) = self.recency_data.write() {
            recency_data.insert(path.to_path_buf(), SystemTime::now());
        }
    }

    fn increment_frequency(&self, path: &Path) {
        if let Ok(mut frequency_data) = self.frequency_data.write() {
            let count = frequency_data.entry(path.to_path_buf()).or_insert(0);
            *count += 1;
        }
    }

    pub fn rank_results(&self, paths: Vec<PathBuf>) -> Vec<PathBuf> {
        let recency_data = if let Ok(data) = self.recency_data.read() {
            data.clone()
        } else {
            HashMap::new()
        };

        let frequency_data = if let Ok(data) = self.frequency_data.read() {
            data.clone()
        } else {
            HashMap::new()
        };

        let mut scored_paths: Vec<(PathBuf, f64)> = paths.into_iter()
            .map(|path| {
                let recency_score = self.calculate_recency_score(
                    recency_data.get(&path).cloned()
                );

                let frequency_score = self.calculate_frequency_score(
                    frequency_data.get(&path).cloned().unwrap_or(0)
                );

                let proximity_score = self.calculate_proximity_score(&path);

                let extension_score = self.calculate_extension_score(&path);

                let total_score =
                    (recency_score * self.factors.recency_weight) +
                        (frequency_score * self.factors.frequency_weight) +
                        (proximity_score * self.factors.proximity_weight) +
                        extension_score;

                (path, total_score)
            })
            .collect();

        // Sort by score (descending)
        scored_paths.sort_by(|(_, score1), (_, score2)| {
            score2.partial_cmp(score1).unwrap_or(Ordering::Equal)
        });

        // Return just the paths in sorted order
        scored_paths.into_iter().map(|(path, _)| path).collect()
    }

    pub fn calculate_recency_score(&self, last_access: Option<SystemTime>) -> f64 {
        match last_access {
            Some(time) => {
                // Calculate recency based on time elapsed since last access
                let now = SystemTime::now();
                let elapsed = now.duration_since(time).unwrap_or(Duration::from_secs(0));
                let hours = elapsed.as_secs() as f64 / 3600.0;

                // Exponential decay function: score = e^(-λt)
                // Where λ determines how quickly the score decays
                // and t is time in hours
                let decay_factor = 0.05; // Lower = slower decay
                let score = (-decay_factor * hours).exp();

                // Clamp between 0 and max_recency_score
                score.min(self.factors.max_recency_score).max(0.0)
            },
            None => 0.0
        }
    }

    pub fn calculate_frequency_score(&self, frequency: u32) -> f64 {
        // Log function to prevent extremely frequent items from dominating
        // Add 1 to avoid log(0)
        // Ensure even zero frequency has a small baseline score
        if frequency == 0 {
            return 0.1; // Small baseline score for zero frequency
        }
        (frequency as f64 + 1.0).ln() / 5.0
    }

    pub fn calculate_proximity_score(&self, path: &Path) -> f64 {
        // Calculate proximity to current directory
        if path.starts_with(&self.current_directory) {
            // Check if it's directly in the current directory or in a subdirectory
            let components_after_current = path.strip_prefix(&self.current_directory)
                .map(|p| p.components().count())
                .unwrap_or(0);

            if components_after_current <= 1 {
                // File is directly in the current directory
                1.0
            } else {
                // File is in a subdirectory of the current directory
                // Score decreases with depth
                0.9 / components_after_current as f64
            }
        } else if let Some(parent) = self.current_directory.parent() {
            if path.starts_with(parent) {
                // File is in a sibling directory
                0.8
            } else {
                // Calculate based on directory tree distance
                let current_depth = self.current_directory.components().count();
                let path_depth = path.components().count();

                // Find common prefix
                let common_prefix_len = self.current_directory.ancestors()
                    .filter(|ancestor| path.starts_with(ancestor))
                    .count();

                // Calculate directory distance
                let distance = (current_depth + path_depth - 2 * common_prefix_len) as f64;

                // Score decreases with distance
                1.0 / (1.0 + distance)
            }
        } else {
            0.1 // Fallback for root directory
        }
    }

    pub fn calculate_extension_score(&self, path: &Path) -> f64 {
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();

            // Find matching extension weight
            for (ext, weight) in &self.factors.extension_weights {
                if ext_str == *ext {
                    // Use exact subtraction to avoid floating-point precision issues
                    return (weight * 10.0 - 10.0) / 10.0; // Normalize to be a bonus/penalty with exact precision
                }
            }
        }

        0.0 // No extension or no matching weight
    }
}

#[cfg(test)]
mod tests_context_aware_ranking {
    use super::*;
    use crate::log_info;

    fn setup_test_ranker() -> ContextAwareRanker {
        let current_dir = PathBuf::from("/test/directory");

        // Create ranker with custom factors for testing
        let mut ranker = ContextAwareRanker::new(current_dir);

        // Customize factors for testing
        ranker.factors = RankingFactors {
            recency_weight: 0.4,
            frequency_weight: 0.3,
            proximity_weight: 0.3,
            extension_weights: vec![
                ("rs".to_string(), 1.5),  // Higher weight for Rust files
                ("txt".to_string(), 0.8), // Lower weight for text files
            ],
            max_recency_score: 1.0,
        };

        ranker
    }

    fn setup_with_history() -> ContextAwareRanker {
        let ranker = setup_test_ranker();

        // Add recency data
        {
            let mut recency_data = ranker.recency_data.write().unwrap();
            let now = SystemTime::now();

            // File accessed very recently
            recency_data.insert(
                PathBuf::from("/test/directory/recent_file.rs"),
                now
            );

            // File accessed 1 hour ago
            recency_data.insert(
                PathBuf::from("/test/directory/older_file.rs"),
                now - Duration::from_secs(3600)
            );

            // File accessed 1 day ago
            recency_data.insert(
                PathBuf::from("/test/directory/old_file.rs"),
                now - Duration::from_secs(86400)
            );
        }

        // Add frequency data
        {
            let mut frequency_data = ranker.frequency_data.write().unwrap();

            // Frequently accessed file
            frequency_data.insert(PathBuf::from("/test/directory/frequent_file.rs"), 10);

            // Moderately accessed file
            frequency_data.insert(PathBuf::from("/test/directory/moderate_file.rs"), 5);

            // Rarely accessed file
            frequency_data.insert(PathBuf::from("/test/directory/rare_file.rs"), 1);
        }

        ranker
    }

    #[test]
    fn test_recency_score_calculation() {
        let ranker = setup_test_ranker();
        let now = SystemTime::now();

        // Test recency scores at different time points
        let recent_score = ranker.calculate_recency_score(Some(now));
        let hour_ago_score = ranker.calculate_recency_score(Some(now - Duration::from_secs(3600)));
        let day_ago_score = ranker.calculate_recency_score(Some(now - Duration::from_secs(86400)));
        let none_score = ranker.calculate_recency_score(None);

        log_info!(format!("Recent score: {}", recent_score).as_str());
        log_info!(format!("Hour ago score: {}", hour_ago_score).as_str());
        log_info!(format!("Day ago score: {}", day_ago_score).as_str());

        assert!(recent_score > 0.9, "Recently accessed files should have high scores");
        assert!(hour_ago_score < recent_score, "Older files should have lower scores");
        assert!(day_ago_score < hour_ago_score, "Much older files should have even lower scores");
        assert_eq!(none_score, 0.0, "Non-accessed files should have zero recency score");
    }

    #[test]
    fn test_frequency_score_calculation() {
        let ranker = setup_test_ranker();

        // Test various frequency levels
        let high_freq_score = ranker.calculate_frequency_score(10);
        let med_freq_score = ranker.calculate_frequency_score(5);
        let low_freq_score = ranker.calculate_frequency_score(1);
        let zero_freq_score = ranker.calculate_frequency_score(0);

        log_info!(format!("High frequency score: {}", high_freq_score).as_str());
        log_info!(format!("Medium frequency score: {}", med_freq_score).as_str());
        log_info!(format!("Low frequency score: {}", low_freq_score).as_str());

        assert!(high_freq_score > med_freq_score, "Higher frequency should result in higher score");
        assert!(med_freq_score > low_freq_score, "Medium frequency should score higher than low frequency");
        assert!(low_freq_score > zero_freq_score, "Even low frequency should score higher than zero");
        assert_eq!(zero_freq_score, 0.1, "Zero frequency should have a small baseline score");
    }

    #[test]
    fn test_proximity_score_calculation() {
        let ranker = setup_test_ranker();

        // Test files at different locations relative to current directory
        let current_dir_file = PathBuf::from("/test/directory/file.rs");
        let subdirectory_file = PathBuf::from("/test/directory/subdir/file.rs");
        let parent_dir_file = PathBuf::from("/test/file.rs");
        let distant_file = PathBuf::from("/other/path/file.rs");

        let current_dir_score = ranker.calculate_proximity_score(&current_dir_file);
        let subdir_score = ranker.calculate_proximity_score(&subdirectory_file);
        let parent_score = ranker.calculate_proximity_score(&parent_dir_file);
        let distant_score = ranker.calculate_proximity_score(&distant_file);

        log_info!(format!("Current directory score: {}", current_dir_score).as_str());
        log_info!(format!("Subdirectory score: {}", subdir_score).as_str());
        log_info!(format!("Parent directory score: {}", parent_score).as_str());
        log_info!(format!("Distant directory score: {}", distant_score).as_str());

        assert_eq!(current_dir_score, 1.0, "Files in current directory should have maximum proximity score");
        assert!(subdir_score < current_dir_score, "Files in subdirectories should have lower score than current directory");
        assert!(parent_score < current_dir_score, "Files in parent directory should have lower score than current directory");
        assert!(distant_score < parent_score, "Files in distant directories should have the lowest scores");
    }

    #[test]
    fn test_extension_score_calculation() {
        let ranker = setup_test_ranker();

        // Test files with different extensions
        let rust_file = PathBuf::from("/test/file.rs");
        let text_file = PathBuf::from("/test/file.txt");
        let unknown_file = PathBuf::from("/test/file.xyz");

        let rust_score = ranker.calculate_extension_score(&rust_file);
        let text_score = ranker.calculate_extension_score(&text_file);
        let unknown_score = ranker.calculate_extension_score(&unknown_file);

        log_info!(format!("Rust file extension score: {}", rust_score).as_str());
        log_info!(format!("Text file extension score: {}", text_score).as_str());
        log_info!(format!("Unknown file extension score: {}", unknown_score).as_str());

        assert_eq!(rust_score, 0.5, "Rust files should get 0.5 bonus (1.5-1.0)");
        assert_eq!(text_score, -0.2, "Text files should get -0.2 penalty (0.8-1.0)");
        assert_eq!(unknown_score, 0.0, "Unknown extensions should get neutral score");
    }

    #[test]
    fn test_record_access() {
        let ranker = setup_test_ranker();
        let test_path = PathBuf::from("/test/record_test.rs");

        // Verify initial state
        {
            let recency_data = ranker.recency_data.read().unwrap();
            let frequency_data = ranker.frequency_data.read().unwrap();

            assert!(!recency_data.contains_key(&test_path), "Path should not have recency data initially");
            assert!(!frequency_data.contains_key(&test_path), "Path should not have frequency data initially");
        }

        // Record an access
        ranker.record_access(&test_path);

        // Verify updated state
        {
            let recency_data = ranker.recency_data.read().unwrap();
            let frequency_data = ranker.frequency_data.read().unwrap();

            assert!(recency_data.contains_key(&test_path), "Path should have recency data after access");
            assert!(frequency_data.contains_key(&test_path), "Path should have frequency data after access");
            assert_eq!(*frequency_data.get(&test_path).unwrap(), 1, "Frequency should be 1 after first access");
        }

        // Record another access
        ranker.record_access(&test_path);

        // Verify updated frequency
        {
            let frequency_data = ranker.frequency_data.read().unwrap();
            assert_eq!(*frequency_data.get(&test_path).unwrap(), 2, "Frequency should be 2 after second access");
        }

        log_info!(format!("Successfully recorded and verified {} accesses to {}",
                         2, test_path.display()).as_str());
    }

    #[test]
    fn test_rank_results() {
        let ranker = setup_with_history();

        // Create test paths
        let paths = vec![
            PathBuf::from("/test/directory/rare_file.rs"),
            PathBuf::from("/test/directory/recent_file.rs"),
            PathBuf::from("/test/directory/moderate_file.rs"),
            PathBuf::from("/other/path/distant_file.txt"),
            PathBuf::from("/test/directory/frequent_file.rs"),
        ];

        // Rank the results
        let ranked_paths = ranker.rank_results(paths);

        // Log the ranked results
        for (i, path) in ranked_paths.iter().enumerate() {
            log_info!(format!("Rank {}: {}", i + 1, path.display()).as_str());
        }

        // The recent and frequent Rust files should be ranked higher
        assert_eq!(ranked_paths[0], PathBuf::from("/test/directory/recent_file.rs"),
                   "Recent file should be ranked highest due to recency weight");
        assert_eq!(ranked_paths[1], PathBuf::from("/test/directory/frequent_file.rs"),
                   "Frequent file should be ranked second due to frequency weight");

        // The distant text file should be ranked lower
        assert_eq!(ranked_paths.last().unwrap(), &PathBuf::from("/other/path/distant_file.txt"),
                   "Distant txt file should be ranked lowest due to distance and extension");
    }

    #[test]
    fn test_update_current_directory() {
        let mut ranker = setup_test_ranker();
        let initial_dir = ranker.current_directory.clone();
        let new_dir = PathBuf::from("/test/new_directory");

        // Update current directory
        ranker.update_current_directory(new_dir.clone());

        log_info!(format!("Directory updated from {} to {}",
                         initial_dir.display(), ranker.current_directory.display()).as_str());

        assert_eq!(ranker.current_directory, new_dir, "Current directory should be updated");

        // Check that proximity scoring changes with new directory
        let old_file = PathBuf::from("/test/directory/file.rs");
        let new_file = PathBuf::from("/test/new_directory/file.rs");

        let old_file_score = ranker.calculate_proximity_score(&old_file);
        let new_file_score = ranker.calculate_proximity_score(&new_file);

        log_info!(format!("Old file proximity score: {}", old_file_score).as_str());
        log_info!(format!("New file proximity score: {}", new_file_score).as_str());

        assert!(new_file_score > old_file_score,
                "File in new current directory should have higher proximity score than file in old directory");
    }

    #[test]
    fn test_get_recent_paths() {
        let ranker = setup_with_history();

        // Get recent paths
        let recent_paths = ranker.get_recent_paths(2);

        assert_eq!(recent_paths.len(), 2, "Should return exactly 2 recent paths");
        assert_eq!(recent_paths[0], PathBuf::from("/test/directory/recent_file.rs"),
                   "Most recent path should be first");

        log_info!(format!("Retrieved {} recent paths: {} and {}",
                         recent_paths.len(),
                         recent_paths[0].display(),
                         recent_paths[1].display()).as_str());
    }

    #[test]
    fn test_get_frequent_paths() {
        let ranker = setup_with_history();

        // Get frequent paths
        let frequent_paths = ranker.get_frequent_paths(2);

        assert_eq!(frequent_paths.len(), 2, "Should return exactly 2 frequent paths");
        assert_eq!(frequent_paths[0], PathBuf::from("/test/directory/frequent_file.rs"),
                   "Most frequent path should be first");

        log_info!(format!("Retrieved {} frequent paths: {} and {}",
                         frequent_paths.len(),
                         frequent_paths[0].display(),
                         frequent_paths[1].display()).as_str());
    }

    #[test]
    fn test_calculate_score() {
        let ranker = setup_with_history();

        let paths = vec![
            PathBuf::from("/test/directory/recent_file.rs"),
            PathBuf::from("/test/directory/frequent_file.rs"),
            PathBuf::from("/other/path/distant_file.txt"),
        ];

        for path in &paths {
            let score = ranker.calculate_score(path);
            log_info!(format!("Total score for {}: {}", path.display(), score).as_str());

            // Calculate individual components manually to verify
            let recency_score = ranker.calculate_recency_score(ranker.get_last_access(path));
            let frequency_score = ranker.calculate_frequency_score(ranker.get_frequency(path));
            let proximity_score = ranker.calculate_proximity_score(path);
            let extension_score = ranker.calculate_extension_score(path);

            log_info!(format!("Score components for {}: recency={}, frequency={}, proximity={}, extension={}",
                             path.display(), recency_score, frequency_score, proximity_score, extension_score).as_str());

            // Verify the total score calculation
            let expected_score =
                (recency_score * ranker.factors.recency_weight) +
                    (frequency_score * ranker.factors.frequency_weight) +
                    (proximity_score * ranker.factors.proximity_weight) +
                    extension_score;

            assert!((score - expected_score).abs() < f64::EPSILON,
                    "Total score calculation should match the sum of weighted components");
        }
    }
}