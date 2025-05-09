use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct FuzzySearchIndex {
    // Maps n-grams to file paths containing them
    n_gram_index: HashMap<String, HashSet<PathBuf>>,
    // Maximum edit distance for fuzzy matching
    max_distance: usize,
}

impl FuzzySearchIndex {
    pub fn new(max_distance: usize) -> Self {
        Self {
            n_gram_index: HashMap::new(),
            max_distance,
        }
    }

    pub fn index_path(&mut self, path: &Path) {
        if let Some(filename) = path.file_name() {
            let filename = filename.to_string_lossy().to_string().to_lowercase();

            // Generate n-grams (2-grams)
            for i in 0..filename.len().saturating_sub(1) {
                let n_gram = &filename[i..i+2];
                self.n_gram_index
                    .entry(n_gram.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(path.to_path_buf());
            }

            // Also index the first character separately for better matching
            if !filename.is_empty() {
                let first_char = &filename[0..1];
                self.n_gram_index
                    .entry(first_char.to_string())
                    .or_insert_with(HashSet::new)
                    .insert(path.to_path_buf());
            }
        }
    }

    pub fn remove_path(&mut self, path: &Path) {
        // First, collect n-grams from the path to be removed
        let mut n_grams_to_update = Vec::new();

        if let Some(filename) = path.file_name() {
            let filename = filename.to_string_lossy().to_string().to_lowercase();

            // Same n-gram generation as in index_path
            for i in 0..filename.len().saturating_sub(1) {
                let n_gram = &filename[i..i+2];
                n_grams_to_update.push(n_gram.to_string());
            }

            // Also handle the first character
            if !filename.is_empty() {
                let first_char = &filename[0..1];
                n_grams_to_update.push(first_char.to_string());
            }
        }

        // Remove this path from all relevant n-gram entries
        for n_gram in n_grams_to_update {
            if let Some(paths) = self.n_gram_index.get_mut(&n_gram) {
                paths.remove(path);

                // If the set is now empty, we'll clean it up later
            }
        }

        // Clean up empty n-gram entries
        self.n_gram_index.retain(|_, paths| !paths.is_empty());
    }

    // Check if a path exists in the fuzzy index
    pub fn contains_path(&self, path: &Path) -> bool {
        // Check if the path exists in any of the n-gram collections
        for (_, paths) in &self.n_gram_index {
            if paths.contains(path) {
                return true;
            }
        }

        // Path not found in any n-gram index
        false
    }

    pub fn find_matches(&self, query: &str, limit: usize) -> Vec<PathBuf> {
        let query = query.to_lowercase();

        // Get candidate paths from n-grams
        let candidates = self.get_candidates(&query);

        // Filter candidates by edit distance and collect with distance
        let mut matches: Vec<_> = candidates.into_iter()
            .filter_map(|path| {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy().to_lowercase();

                    // Calculate edit distance between the filename and query
                    let distance = levenshtein_distance(&filename, &query);

                    // Include if:
                    // 1. Within max edit distance
                    // 2. The filename contains the query (substring match)
                    // 3. The query contains the filename (to catch shorter names)
                    // 4. The edit distance between query and filename without extension is within limit
                    if distance <= self.max_distance ||
                       filename.contains(&query) ||
                       query.contains(&filename) {
                        Some((path, distance))
                    } else {
                        // Try matching without extension
                        if let Some(stem) = path.file_stem() {
                            let stem_str = stem.to_string_lossy().to_lowercase();
                            let stem_distance = levenshtein_distance(&stem_str, &query);
                            if stem_distance <= self.max_distance {
                                return Some((path, stem_distance));
                            }
                        }
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort by edit distance (lowest first)
        matches.sort_by_key(|(_, distance)| *distance);

        // Take top results
        let result = matches.into_iter()
            .take(limit)
            .map(|(path, _)| path)
            .collect();

        result
    }

    fn get_candidates(&self, query: &str) -> HashSet<PathBuf> {
        let mut candidates = HashSet::new();

        // Handle empty query
        if query.is_empty() {
            return candidates;
        }

        // For very short queries, use the first character
        if query.len() == 1 {
            if let Some(paths) = self.n_gram_index.get(query) {
                candidates.extend(paths.iter().cloned());
            }
            return candidates;
        }

        // For optimization, don't process all n-grams for long queries
        // Only consider a subset of n-grams for performance
        let max_ngrams_to_check = if query.len() > 5 { 3 } else { query.len() - 1 };

        // Extract n-grams from query for longer queries (but limit processing for performance)
        for i in 0..std::cmp::min(query.len().saturating_sub(1), max_ngrams_to_check) {
            let n_gram = &query[i..i+2];
            if let Some(paths) = self.n_gram_index.get(n_gram) {
                candidates.extend(paths.iter().cloned());
            }
        }

        // Always check the beginning of the string as it's usually most important
        if query.len() >= 2 {
            let start_ngram = &query[0..2];
            if let Some(paths) = self.n_gram_index.get(start_ngram) {
                candidates.extend(paths.iter().cloned());
            }
        }

        // Add candidates using first character as a fallback
        if candidates.is_empty() && !query.is_empty() {
            let first_char = &query[0..1];
            if let Some(paths) = self.n_gram_index.get(first_char) {
                candidates.extend(paths.iter().cloned());
            }
        }

        // If still no candidates and query is long enough, try with substring matching
        // but only for reasonable query sizes to maintain performance
        if candidates.is_empty() && query.len() >= 3 && query.len() <= 10 {
            for (n_gram, paths) in &self.n_gram_index {
                if n_gram.len() >= 2 && query.contains(n_gram) {
                    candidates.extend(paths.iter().cloned());

                    // Limit the number of paths we process to maintain performance
                    if candidates.len() > 500 {
                        break;
                    }
                }
            }
        }

        // For the case where we want files with similar names (like "tile" for "file"),
        // ensure we include more potential candidates by checking first letter and nearby characters
        if query.len() >= 2 {
            // Include files that start with the same letter (might be within edit distance)
            let first_char = &query[0..1];
            if let Some(paths) = self.n_gram_index.get(first_char) {
                candidates.extend(paths.iter().cloned());
            }

            // For short queries, it might be worth checking files that match other letters
            // in the query to account for transpositions and substitutions
            if query.len() <= 5 {
                for i in 1..query.len() {
                    let char_at_i = &query[i..i+1];
                    if let Some(paths) = self.n_gram_index.get(char_at_i) {
                        candidates.extend(paths.iter().cloned());
                    }
                }
            }
        }

        // IMPORTANT FIX: Special case for common edit operations like substitution at the beginning
        // Check for paths that have n-grams similar to query but with character changes
        // This ensures we catch cases like "tile" when searching for "file"
        if query.len() >= 2 {
            for (n_gram, paths) in &self.n_gram_index {
                if n_gram.len() >= 2 {
                    // Check for close matches (common first or second letter)
                    if query.len() >= 2 && n_gram.len() >= 2 {
                        if query[1..2] == n_gram[1..2] ||
                           (query.len() >= 3 && n_gram.len() >= 3 && query[2..3] == n_gram[2..3]) {
                            candidates.extend(paths.iter().cloned());
                        }
                    }
                }
            }
        }

        candidates
    }
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    // Fast path for empty strings and exact matches
    if s1 == s2 { return 0; }
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    if len1 == 0 { return len2; }
    if len2 == 0 { return len1; }

    // Early termination if strings differ too much in length
    // This can significantly improve performance
    let length_diff = if len1 > len2 { len1 - len2 } else { len2 - len1 };
    if length_diff > 3 { // If length difference exceeds typical edit distance threshold
        return length_diff; // Return the length difference as a conservative estimate
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Use a single vector to save memory allocation overhead
    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row: Vec<usize> = vec![0; len2 + 1];

    for i in 1..=len1 {
        curr_row[0] = i;

        for j in 1..=len2 {
            let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };

            curr_row[j] = std::cmp::min(
                curr_row[j-1] + 1,              // insertion
                std::cmp::min(
                    prev_row[j] + 1,            // deletion
                    prev_row[j-1] + cost        // substitution
                )
            );
        }

        // Swap rows for the next iteration
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    // The result is in prev_row after the final swap
    prev_row[len2]
}

#[cfg(test)]
mod tests_fuzzy {
    use super::*;
    use std::time::{Duration, Instant};
    use crate::log_info;

    // Unit tests
    #[test]
    fn test_empty_index() {
        log_info!("Running test_empty_index");
        let index = FuzzySearchIndex::new(2);
        let results = index.find_matches("test", 10);
        assert!(results.is_empty(), "Empty index should return no results");
    }

    #[test]
    fn test_exact_match() {
        log_info!("Running test_exact_match");
        let mut index = FuzzySearchIndex::new(2);
        let path = PathBuf::from("/test/file.txt");
        index.index_path(&path);

        let results = index.find_matches("file.txt", 10);
        assert_eq!(results.len(), 1, "Should find exactly one match");
        assert_eq!(results[0], path, "Should find the exact path");
    }

    #[test]
    fn test_fuzzy_match() {
        log_info!("Running test_fuzzy_match");
        let mut index = FuzzySearchIndex::new(2);
        let path = PathBuf::from("/test/document.txt");
        index.index_path(&path);

        // Typo: "documnet" instead of "document"
        let results = index.find_matches("documnet.txt", 10);
        assert_eq!(results.len(), 1, "Should find one fuzzy match");
        assert_eq!(results[0], path, "Should find the path despite the typo");
    }

    #[test]
    fn test_multiple_matches() {
        log_info!("Running test_multiple_matches");
        let mut index = FuzzySearchIndex::new(2);
        let path1 = PathBuf::from("/test/file1.txt");
        let path2 = PathBuf::from("/test/file2.txt");
        let path3 = PathBuf::from("/test/tile.txt");

        index.index_path(&path1);
        index.index_path(&path2);
        index.index_path(&path3);

        // Should match both file1.txt, file2.txt, and tile.txt with edit distance <= 2
        let results = index.find_matches("file", 10);
        assert_eq!(results.len(), 3, "Should find all three fuzzy matches");
    }

    #[test]
    fn test_remove_path() {
        log_info!("Running test_remove_path");
        let mut index = FuzzySearchIndex::new(2);
        let path = PathBuf::from("/test/document.txt");
        index.index_path(&path);

        // First verify the path is indexed
        let results = index.find_matches("document", 10);
        assert_eq!(results.len(), 1, "Path should be found before removal");

        // Now remove the path
        index.remove_path(&path);

        // Verify it's no longer found
        let results_after = index.find_matches("document", 10);
        assert!(results_after.is_empty(), "Path should not be found after removal");
    }

    #[test]
    fn test_max_distance_limit() {
        log_info!("Running test_max_distance_limit");
        // Create index with max distance 1
        let mut index_strict = FuzzySearchIndex::new(1);
        // Create index with max distance 3
        let mut index_lenient = FuzzySearchIndex::new(3);

        let path = PathBuf::from("/test/document.txt");
        index_strict.index_path(&path);
        index_lenient.index_path(&path);

        // Query with 2 edits (beyond strict limit, within lenient limit)
        // "document" -> "ducumant"
        let strict_results = index_strict.find_matches("ducumant.txt", 10);
        let lenient_results = index_lenient.find_matches("ducumant.txt", 10);

        assert!(strict_results.is_empty(), "Strict index should not find matches beyond distance 1");
        assert_eq!(lenient_results.len(), 1, "Lenient index should find the match");
    }

    #[test]
    fn test_limit_results() {
        log_info!("Running test_limit_results");
        let mut index = FuzzySearchIndex::new(2);

        // Add 5 similar paths
        for i in 1..=5 {
            let path = PathBuf::from(format!("/test/file{}.txt", i));
            index.index_path(&path);
        }

        // Retrieve with limit 3
        let limited_results = index.find_matches("file", 3);
        assert_eq!(limited_results.len(), 3, "Should respect the result limit");

        // Retrieve all
        let all_results = index.find_matches("file", 10);
        assert_eq!(all_results.len(), 5, "Should find all matches with sufficient limit");
    }

    #[test]
    fn test_levenshtein_distance() {
        log_info!("Running test_levenshtein_distance");
        // Test exact match
        assert_eq!(levenshtein_distance("file", "file"), 0);

        // Test single operations
        assert_eq!(levenshtein_distance("file", "files"), 1); // Insert 's'
        assert_eq!(levenshtein_distance("files", "file"), 1); // Delete 's'
        assert_eq!(levenshtein_distance("file", "bile"), 1);  // Substitute 'f' -> 'b'

        // Test multiple operations
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);

        // Test empty strings
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("file", ""), 4);
        assert_eq!(levenshtein_distance("", "file"), 4);
    }

    // Benchmarks using simple timing
    #[test]
    fn benchmark_index_large_dataset() {
        log_info!("Running benchmark_index_large_dataset");
        let mut index = FuzzySearchIndex::new(2);
        let start = Instant::now();

        // Generate and index 10,000 paths
        for i in 0..10_000 {
            let path = PathBuf::from(format!("/test/benchmark/file{}.txt", i));
            index.index_path(&path);
        }

        let duration = start.elapsed();
        log_info!(format!("Indexed 10,000 paths in {:?}", duration).as_str());
        assert!(duration < Duration::from_secs(1), "Indexing should be fast");
    }

    #[test]
    fn benchmark_fuzzy_search() {
        log_info!("Running benchmark_fuzzy_search");
        let mut index = FuzzySearchIndex::new(2);

        // Create a moderately large dataset with varied names
        for i in 0..5_000 {
            let path = PathBuf::from(format!("/test/benchmark/file{}.txt", i));
            index.index_path(&path);

            let path = PathBuf::from(format!("/test/benchmark/document{}.pdf", i));
            index.index_path(&path);

            let path = PathBuf::from(format!("/test/benchmark/image{}.jpg", i));
            index.index_path(&path);
        }

        // Benchmark search with common typos
        let queries = [
            "flie", "documant", "imag", "dokument", "phile"
        ];

        for query in &queries {
            let start = Instant::now();
            let results = index.find_matches(query, 20);
            let duration = start.elapsed();

            log_info!(format!("Search for '{}' found {} results in {:?}", query, results.len(), duration).as_str());
            assert!(duration < Duration::from_millis(100), "Search should be fast");
        }
    }

    #[test]
    fn benchmark_levenshtein() {
        log_info!("Running benchmark_levenshtein");
        // Benchmark different string lengths
        let test_cases = [
            ("a", "b"),                            // Very short (1)
            ("file", "files"),                     // Short (5)
            ("document", "documnet"),              // Medium (8)
            ("application.config", "aplication.konfig"), // Long (17)
            ("this_is_a_very_long_filename.txt", "this_is_a_vrey_long_filename.txt"), // Very long (30)
        ];

        for (s1, s2) in &test_cases {
            let start = Instant::now();

            // Run the calculation multiple times for more accurate measurement
            for _ in 0..1000 {
                let _ = levenshtein_distance(s1, s2);
            }

            let duration = start.elapsed();
            let avg_duration = duration.as_nanos() as f64 / 1000.0;

            log_info!(format!("Levenshtein '{}' vs '{}' (len {}, {}): avg {:?}ns per calc",
                     s1, s2, s1.len(), s2.len(), avg_duration).as_str());
        }
    }

    #[test]
    fn benchmark_candidates_generation() {
        log_info!("Running benchmark_candidates_generation");
        let mut index = FuzzySearchIndex::new(2);

        // Create dataset with specific n-gram patterns
        for i in 0..1000 {
            let path = PathBuf::from(format!("/test/benchmark/file{}.txt", i));
            index.index_path(&path);
        }

        // Add some paths with very common n-grams
        for i in 0..1000 {
            let path = PathBuf::from(format!("/test/benchmark/aaaaa{}.txt", i));
            index.index_path(&path);
        }

        // Benchmark candidate generation with different queries
        let queries = ["file", "aaaaa", "xyz"]; // Common, very common, and non-existent n-grams

        for query in &queries {
            let start = Instant::now();
            let candidates = index.get_candidates(query);
            let duration = start.elapsed();

            log_info!(format!("Generated {} candidates for '{}' in {:?}",
                     candidates.len(), query, duration).as_str());
        }
    }
}

