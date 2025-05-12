use std::collections::{HashMap, HashSet};
use std::sync::Once;

type TrigramMap = HashMap<u32, Vec<u32>>;

static CHAR_MAPPING_INIT: Once = Once::new();
static mut CHAR_MAPPING: [u8; 256] = [0; 256];

pub struct PathMatcher {
    paths: Vec<String>,
    trigram_index: TrigramMap,
    // Cache for parsed paths - avoids repeated allocations
    #[cfg(feature = "path_cache")]
    path_component_cache: HashMap<usize, Vec<String>>,
}

impl PathMatcher {
    pub fn new() -> Self {
        Self::init_char_mapping();

        PathMatcher {
            paths: Vec::new(),
            trigram_index: HashMap::with_capacity(4096),
            #[cfg(feature = "path_cache")]
            path_component_cache: HashMap::new(),
        }
    }

    // Fast case folding using a lookup table
    fn init_char_mapping() {
        CHAR_MAPPING_INIT.call_once(|| {
            unsafe {
                for i in 0..256 {
                    let c = i as u8 as char;
                    let lower = c.to_lowercase().next().unwrap_or(c) as u8;
                    CHAR_MAPPING[i] = lower;
                }
            }
        });
    }

    #[inline(always)]
    fn fast_lowercase(c: u8) -> u8 {
        // Ensure mapping is initialized before using it
        Self::init_char_mapping();
        unsafe { CHAR_MAPPING[c as usize] }
    }

    pub fn add_path(&mut self, path: &str) {
        let path_index = self.paths.len() as u32;
        self.paths.push(path.to_string());

        // Extract trigrams directly from bytes without intermediate allocations
        self.extract_and_index_trigrams(path, path_index);
    }

    // Optimized trigram extraction working directly on bytes
    #[inline]
    fn extract_and_index_trigrams(&mut self, text: &str, path_idx: u32) {
        let bytes = text.as_bytes();
        if bytes.len() < 3 {
            return;
        }

        // Pre-allocate workspace to avoid repeated allocations
        let mut trigram_bytes = [0u8; 3];

        // Use a preallocated buffer for the padded version
        let mut padded = Vec::with_capacity(bytes.len() + 4);
        //generate padded path
        padded.push(b' ');
        padded.push(b' ');
        padded.extend_from_slice(bytes);
        padded.push(b' ');
        padded.push(b' ');

        for i in 0..padded.len() - 2 {
            // Fast lowercase conversion using lookup table
            trigram_bytes[0] = Self::fast_lowercase(padded[i]);
            trigram_bytes[1] = Self::fast_lowercase(padded[i + 1]);
            trigram_bytes[2] = Self::fast_lowercase(padded[i + 2]);

            // Pack into a single u32 for maximum performance
            let trigram = Self::pack_trigram(trigram_bytes[0], trigram_bytes[1], trigram_bytes[2]);

            // Use entry API with Vec instead of HashSet for better cache locality
            match self.trigram_index.entry(trigram) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    let v = e.get_mut();
                    // Only add if not already present (deduplication)
                    if v.is_empty() || v[v.len() - 1] != path_idx {
                        v.push(path_idx);
                    }
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    let mut v = Vec::with_capacity(4); // Most trigrams appear in few paths
                    v.push(path_idx);
                    e.insert(v);
                }
            }
        }
    }

    #[inline(always)]
    fn pack_trigram(a: u8, b: u8, c: u8) -> u32 {
        ((a as u32) << 16) | ((b as u32) << 8) | (c as u32)
    }

    pub fn search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let query_trigrams = self.extract_query_trigrams(&query_lower);
        if query_trigrams.is_empty() && query.len() >= 3 {
            return Vec::new();
        }

        // 32-bit words can track 32 paths each
        let bitmap_size = (self.paths.len() + 31) / 32;
        let mut path_bitmap = vec![0u32; bitmap_size];
        let mut hit_counts = vec![0u16; self.paths.len()];
        let mut total_hits = 0;

        // For each trigram, mark matching paths in bitmap
        for &trigram in &query_trigrams {
            if let Some(path_indices) = self.trigram_index.get(&trigram) {
                for &path_idx in path_indices {
                    let idx = path_idx as usize;
                    if idx < hit_counts.len() {
                        // Increment hit count
                        hit_counts[idx] += 1;

                        // Set bitmap bit if first hit
                        if hit_counts[idx] == 1 {
                            total_hits += 1;
                        }

                        // Mark in bitmap using fast bit ops
                        let word_idx = idx / 32;
                        let bit_pos = idx % 32;
                        path_bitmap[word_idx] |= 1 << bit_pos;
                    }
                }
            }
        }

        // early termination
        if total_hits == 0 {
            return self.fallback_search(query, max_results);
        }

        let mut results = Vec::with_capacity(total_hits.min(max_results * 2));

        // Track the first letter of the query for prioritization
        let query_first_char = query_lower.chars().next();

        let query_lower = query.to_lowercase();
        let query_trigram_count = query_trigrams.len() as f32;

        for word_idx in 0..path_bitmap.len() {
            let mut word = path_bitmap[word_idx];

            if word==0 {
                continue;
            }

            while word != 0 {
                let bit_idx = word.trailing_zeros() as usize;
                let path_idx = word_idx * 32 + bit_idx;

                if path_idx < self.paths.len() {
                    let path = &self.paths[path_idx];
                    let hits = hit_counts[path_idx] as f32;

                    let path_lower = path.to_lowercase();

                    // Check for exact filename matches first
                    let path_components: Vec<&str> = path.split('/').collect();
                    let filename = path_components.last().unwrap_or(&"");

                    // Critical: Match the actual query term with proper case sensitivity
                    // If this file contains the actual search term in its name, give it priority
                    let filename_lower = filename.to_lowercase();

                    let mut score = hits / query_trigram_count;

                    if filename_lower == query_lower {
                        score += 0.5; // Exact match bonus
                    }
                    else if filename_lower.contains(&query_lower) {
                        score += 0.3; // substring match
                    }
                    // Contains anywhere in path
                    else if path_lower.contains(&query_lower) {
                        score += 0.2; // General path substring match
                    }

                    // This ensures "LWORECASE" prioritizes files starting with 'l'
                    if let Some(query_char) = query_first_char {
                        if let Some(filename_char) = filename_lower.chars().next() {
                            // If the first letter matches, give a significant boost
                            if query_char == filename_char {
                                score += 0.15; // bonus for first letter match
                            }
                        }
                    }

                    // Further bonus for file extension matches
                    if let Some(dot_pos) = query_lower.find('.') {
                        let query_ext = &query_lower[dot_pos..];
                        if path.to_lowercase().ends_with(query_ext) {
                            score += 0.1;
                        }
                    }



                    // Apply position bias - paths with match near start get bonus
                    if let Some(pos) = path_lower.find(&query_lower) {
                        // Position bonus decreases as position increases
                        let pos_factor = 1.0 - (pos as f32 / path.len() as f32).min(0.9);
                        score += pos_factor * 0.1;
                    }

                    results.push((path.clone(), score));
                }

                // Clear the bit we just processed and continue
                word &= !(1 << bit_idx);
            }
        }

        // Sort results by score
        results.sort_unstable_by(|a, b| {
            // Primary sort by score (descending)
            let cmp = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }

            // Secondary sort: path length (ascending)
            a.0.len().cmp(&b.0.len())
        });

        // Return top matches
        results.truncate(max_results);

        if results.is_empty() && query.len() >= 3 {
            return self.fallback_search(query, max_results);
        }

        results
    }

    // Extract trigrams from query
    #[inline]
    fn extract_query_trigrams(&self, query: &str) -> Vec<u32> {
        let bytes = query.as_bytes();
        if bytes.len() < 3 {
            // Special case for very short queries
            return Vec::new();
        }

        let mut trigrams = Vec::with_capacity(bytes.len() + 2);

        // Use a preallocated buffer for the padded version
        let mut padded = Vec::with_capacity(bytes.len() + 4);
        padded.push(b' ');
        padded.push(b' ');
        padded.extend_from_slice(bytes);
        padded.push(b' ');
        padded.push(b' ');

        for i in 0..padded.len() - 2 {
            let a = Self::fast_lowercase(padded[i]);
            let b = Self::fast_lowercase(padded[i + 1]);
            let c = Self::fast_lowercase(padded[i + 2]);

            trigrams.push(Self::pack_trigram(a, b, c));
        }

        trigrams
    }

    fn fallback_search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        let query_lower = query.to_lowercase();
        // Generate minimal set of variations
        let variations = self.generate_efficient_variations(&query_lower);

        // Reuse a bitmap across all variations to minimize allocations
        let bitmap_size = (self.paths.len() + 31) / 32;
        let mut path_bitmap = vec![0u32; bitmap_size];
        let mut variation_hits = HashMap::with_capacity(variations.len());

        let mut seen_paths = HashSet::with_capacity(max_results * 2);
        let mut results = Vec::with_capacity(max_results * 2);

        // Process each variation
        for (variation_idx, variation) in variations.iter().enumerate() {
            let trigrams = self.extract_query_trigrams(variation);
            if trigrams.is_empty() {
                continue;
            }

            // Clear bitmap for this variation
            for word in &mut path_bitmap {
                *word = 0;
            }

            // Mark paths containing these trigrams
            for &trigram in &trigrams {
                if let Some(path_indices) = self.trigram_index.get(&trigram) {
                    for &path_idx in path_indices {
                        let idx = path_idx as usize;
                        let word_idx = idx / 32;
                        let bit_pos = idx % 32;
                        path_bitmap[word_idx] |= 1 << bit_pos;

                        // Track which variation matched this path
                        variation_hits.entry(path_idx).or_insert_with(|| Vec::with_capacity(2))
                            .push(variation_idx);
                    }
                }
            }
            // Extract matches from bitmap
            for word_idx in 0..path_bitmap.len() {
                let mut word = path_bitmap[word_idx];
                while word != 0 {
                    let bit_pos = word.trailing_zeros() as usize;
                    let path_idx = word_idx * 32 + bit_pos;

                    if path_idx < self.paths.len() && !seen_paths.contains(&path_idx) {
                        seen_paths.insert(path_idx);
                        let path = &self.paths[path_idx];

                        // Extract filename for case-insensitive comparison
                        let filename = path.split('/').last().unwrap_or(path);
                        let filename_lower = filename.to_lowercase();

                        // Simplified scoring for variations
                        let variation_index = variation_idx as f32 / variations.len() as f32;
                        let mut score = 0.9 - (variation_index * 0.2); // Earlier variations score higher

                        // This is crucial for differentiating "LWORECASE" → "lowercase" vs "UPPERCASE"
                        if !query_lower.is_empty() && !filename_lower.is_empty() {
                            let query_first_char = query_lower.chars().next().unwrap();
                            let filename_first_char = filename_lower.chars().next().unwrap();

                            if query_first_char == filename_first_char {
                                // Significantly boost score when first letter matches
                                score += 0.3;
                            }
                        }


                        results.push((path.clone(), score));
                    }

                    word &= !(1 << bit_pos);
                }
            }

            // Early exit if we have enough results
            if results.len() >= max_results * 2 {
                break;
            }
        }

        // Sort and return top results
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        results
    }

    #[inline]
    fn generate_efficient_variations(&self, query: &str) -> Vec<String> {
        let len = query.len();
        let chars: Vec<char> = query.chars().collect();

        // Capacity calculation based on likely number of variations
        let mut variations = Vec::with_capacity(len * 2);

        // Only add the most effective variations

        // 1. Deletions (critical for catching extra characters)
        if len > 1 {
            for i in 0..len {
                let mut new_query = String::with_capacity(len - 1);
                for j in 0..len {
                    if j != i {
                        new_query.push(chars[j]);
                    }
                }
                variations.push(new_query);
            }
        }

        // 2. Adjacent transpositions (critical for catching swapped characters)
        if len > 2 {
            for i in 0..len-1 {
                let mut new_chars = chars.clone();
                new_chars.swap(i, i+1);
                variations.push(new_chars.iter().collect());
            }
        }
        // 3. Only do substitutions for short queries (expensive)
        if len > 1 && len <= 5 {
            // Common substitution patterns
            static SUBS: &[(char, char)] = &[
                ('a', 'e'), ('e', 'a'),
                ('i', 'y'), ('o', 'u'),
                ('s', 'z'), ('c', 'k'),
            ];

            for i in 0..len {
                let c = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                for &(from, to) in SUBS {
                    if c == from {
                        let mut new_chars = chars.clone();
                        new_chars[i] = to;
                        variations.push(new_chars.iter().collect());
                        break;
                    }
                }
            }
        }

        variations
    }
}

#[cfg(test)]
mod tests_fast_fuzzy_v2 {
    use super::*;
    use std::time::Instant;
    use std::path::PathBuf;
    use std::time::Duration;
    use crate::{log_info, log_warn};

    // Helper function for benchmarking
    fn run_benchmark<F, R>(name: &str, iterations: usize, f: F) -> (R, Duration)
    where
        F: Fn() -> R,
    {
        // Warmup
        for _ in 0..3 {
            let _ = f();
        }

        let start = Instant::now();
        let mut result = None;

        for _i in 0..iterations {
            result = Some(f());
        }

        let duration = start.elapsed() / iterations as u32;
        log_info!(&format!("Benchmark '{}': {:?} per iteration", name, duration));

        (result.unwrap(), duration)
    }

    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            log_warn!(&format!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path));
            panic!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path);
        }
        path
    }

    // Helper function to collect real paths from the test data directory
    fn collect_test_paths(limit: Option<usize>) -> Vec<String> {
        let test_path = get_test_data_path();
        let mut paths = Vec::new();

        fn add_paths_recursively(dir: &std::path::Path, paths: &mut Vec<String>, limit: Option<usize>) {
            if let Some(max) = limit {
                if paths.len() >= max {
                    return;
                }
            }

            if let Some(walker) = std::fs::read_dir(dir).ok() {
                for entry in walker.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(path_str) = path.to_str() {
                        paths.push(path_str.to_string());

                        if let Some(max) = limit {
                            if paths.len() >= max {
                                return;
                            }
                        }
                    }

                    if path.is_dir() {
                        add_paths_recursively(&path, paths, limit);
                    }
                }
            }
        }

        add_paths_recursively(&test_path, &mut paths, limit);

        // If test data doesn't contain enough paths or doesn't exist,
        // fall back to synthetic data with a warning
        if paths.is_empty() {
            log_warn!("No test data found, using synthetic data instead");
            return (0..100).map(|i| format!("/path/to/file{}.txt", i)).collect();
        }

        paths
    }

    #[test]
    fn test_fast_lowercase() {
        assert_eq!(PathMatcher::fast_lowercase(b'A'), b'a');
        assert_eq!(PathMatcher::fast_lowercase(b'Z'), b'z');
        assert_eq!(PathMatcher::fast_lowercase(b'a'), b'a');
        assert_eq!(PathMatcher::fast_lowercase(b'1'), b'1');
    }

    #[test]
    fn test_pack_trigram() {
        let trigram = PathMatcher::pack_trigram(b'a', b'b', b'c');
        let expected = (b'a' as u32) << 16 | (b'b' as u32) << 8 | (b'c' as u32);
        assert_eq!(trigram, expected);
    }

    #[test]
    fn test_extract_query_trigrams() {
        let matcher = PathMatcher::new();

        // Test empty or short strings
        assert!(matcher.extract_query_trigrams("").is_empty());
        assert!(matcher.extract_query_trigrams("ab").is_empty());

        // Test normal string
        let trigrams = matcher.extract_query_trigrams("abc");
        assert_eq!(trigrams.len(), 5); // "  abc  " -> 5 trigrams
    }

    #[test]
    fn test_add_path() {
        let mut matcher = PathMatcher::new();
        assert_eq!(matcher.paths.len(), 0);

        matcher.add_path("/test/path.txt");
        assert_eq!(matcher.paths.len(), 1);
        assert_eq!(matcher.paths[0], "/test/path.txt");
        assert!(!matcher.trigram_index.is_empty());
    }

    #[test]
    fn test_basic_search() {
        let mut matcher = PathMatcher::new();

        matcher.add_path("/home/user/file.txt");
        matcher.add_path("/var/log/system.log");
        matcher.add_path("/home/user/documents/notes.md");

        // Search for something that should match the first path
        let results = matcher.search("file", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/home/user/file.txt");

        // Test with misspelled query
        let misspelled_results = matcher.search("flie", 10);  // 'file' misspelled as 'flie'
        assert!(!misspelled_results.is_empty());
        assert_eq!(misspelled_results[0].0, "/home/user/file.txt");
    }

    #[test]
    fn test_search_ranking() {
        let mut matcher = PathMatcher::new();

        // Add paths with varying similarity to our query
        matcher.add_path("/exact/match/rust-src/file.rs");
        matcher.add_path("/partial/rust/src/different.rs");
        matcher.add_path("/unrelated/file.txt");
        matcher.add_path("/rust_src/something/else.txt");

        let results = matcher.search("rust src", 10);

        // Check that we have results
        assert!(!results.is_empty(), "Search should return results");

        // Check that the most relevant paths are present in the results
        let found_exact = results.iter().any(|(path, _)| path == "/exact/match/rust-src/file.rs");
        let found_partial = results.iter().any(|(path, _)| path == "/partial/rust/src/different.rs");
        let found_rust_src = results.iter().any(|(path, _)| path == "/rust_src/something/else.txt");

        assert!(found_exact, "Should find the exact match path");
        assert!(found_partial, "Should find the partial match path");
        assert!(found_rust_src, "Should find the rust_src path");

        // The exact match should be ranked higher than unrelated paths
        let exact_idx = results.iter().position(|(path, _)| path == "/exact/match/rust-src/file.rs").unwrap();
        let unrelated_idx_opt = results.iter().position(|(path, _)| path == "/unrelated/file.txt");

        if let Some(unrelated_idx) = unrelated_idx_opt {
            assert!(exact_idx < unrelated_idx, "Exact match should rank higher than unrelated path");
        }

        // Verify scores are in descending order
        for i in 1..results.len() {
            assert!(results[i-1].1 >= results[i].1,
                "Scores should be in descending order: {} >= {}", results[i-1].1, results[i].1);
        }

        // Test with misspelled query
        let misspelled_results = matcher.search("rsut scr", 10);  // 'rust src' misspelled

        // Check that despite misspelling we still get results
        assert!(!misspelled_results.is_empty(), "Misspelled search should return results");

        // The relevant paths should be present despite misspelling
        let found_exact_misspelled = misspelled_results.iter().any(|(path, _)| path == "/exact/match/rust-src/file.rs");
        let found_partial_misspelled = misspelled_results.iter().any(|(path, _)| path == "/partial/rust/src/different.rs");
        let found_rust_src_misspelled = misspelled_results.iter().any(|(path, _)| path == "/rust_src/something/else.txt");

        assert!(found_exact_misspelled || found_partial_misspelled || found_rust_src_misspelled,
               "Should find at least one of the relevant matches despite misspelling");
    }

    #[test]
    fn test_search_limit() {
        let mut matcher = PathMatcher::new();

        // Add many paths
        for i in 0..100 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        // Test that max_results is respected
        let results = matcher.search("file", 5);
        assert_eq!(results.len(), 5);

        let results = matcher.search("file", 10);
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_case_insensitivity_v2() {
        let mut matcher = PathMatcher::new();

        matcher.add_path("/path/to/UPPERCASE.txt");
        matcher.add_path("/path/to/lowercase.txt");
        matcher.add_path("/path/to/something_else.txt");

        // Search should be case insensitive
        let results = matcher.search("uppercase", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/path/to/UPPERCASE.txt");

        let results = matcher.search("LOWERCASE", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/path/to/lowercase.txt");

        // Test with misspelled queries
        let misspelled_results = matcher.search("upprecaes", 10);  // 'uppercase' misspelled
        assert!(!misspelled_results.is_empty());
        assert!(misspelled_results[0].0.to_lowercase().contains("upper"));

        let misspelled_results_2 = matcher.search("LWORECASE", 10);  // 'LOWERCASE' misspelled
        assert!(!misspelled_results_2.is_empty());
        assert!(misspelled_results_2[0].0.to_lowercase().contains("lower"));
    }

    #[test]
    fn test_substring_bonus() {
        let mut matcher = PathMatcher::new();

        matcher.add_path("/exact-substring/file.txt");
        matcher.add_path("/file/with/exact/separated.txt");

        let results = matcher.search("exact substring", 10);

        // First result should be the exact substring match due to bonus
        assert!(!results.is_empty());
        assert_eq!(results[0].0, "/exact-substring/file.txt");
    }

    #[test]
    fn test_empty_query() {
        let mut matcher = PathMatcher::new();
        matcher.add_path("/some/path.txt");

        let results = matcher.search("", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_performance_small() {
        let mut matcher = PathMatcher::new();

        // Add a small number of paths
        for i in 0..100 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        let (results, elapsed) = run_benchmark("small dataset search", 10, || {
            matcher.search("file", 10)
        });

        assert!(!results.is_empty());
        log_info!(&format!("Small dataset (100 items) search took: {:.2?}", elapsed));
    }

    #[test]
    fn test_search_performance_medium() {
        let mut matcher = PathMatcher::new();

        // Add a medium number of paths
        for i in 0..1000 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        let (results, elapsed) = run_benchmark("medium dataset search", 10, || {
            matcher.search("file", 10)
        });

        assert!(!results.is_empty());
        log_info!(&format!("Medium dataset (1,000 items) search took: {:.2?}", elapsed));
    }

    #[test]
    fn test_search_performance_large() {
        let mut matcher = PathMatcher::new();

        // Add a large number of paths
        for i in 0..10000 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        let (results, elapsed) = run_benchmark("large dataset search", 5, || {
            matcher.search("file", 10)
        });

        assert!(!results.is_empty());
        log_info!(&format!("Large dataset (10,000 items) search took: {:.2?}", elapsed));
    }

    #[test]
    fn benchmark_indexing_speed() {
        let iterations = 5;

        // Get real test paths instead of synthetic ones
        let small_paths = collect_test_paths(Some(100));
        let medium_paths = collect_test_paths(Some(1000));
        let large_paths = collect_test_paths(Some(10000));

        log_info!(&format!("Benchmarking with {} small paths, {} medium paths, and {} large paths",
            small_paths.len(), medium_paths.len(), large_paths.len()));

        // Benchmark small dataset
        let (_, small_duration) = run_benchmark("small dataset indexing", iterations, || {
            let mut matcher = PathMatcher::new();
            for path in &small_paths {
                matcher.add_path(path);
            }
            matcher
        });

        // Benchmark medium dataset
        let (_, medium_duration) = run_benchmark("medium dataset indexing", iterations, || {
            let mut matcher = PathMatcher::new();
            for path in &medium_paths {
                matcher.add_path(path);
            }
            matcher
        });

        // Benchmark large dataset
        let (_, large_duration) = run_benchmark("large dataset indexing", iterations, || {
            let mut matcher = PathMatcher::new();
            for path in &large_paths {
                matcher.add_path(path);
            }
            matcher
        });

        log_info!(&format!("Indexing performance comparison:"));
        log_info!(&format!("  Small ({} paths): {:?}", small_paths.len(), small_duration));
        log_info!(&format!("  Medium ({} paths): {:?}", medium_paths.len(), medium_duration));
        log_info!(&format!("  Large ({} paths): {:?}", large_paths.len(), large_duration));

        // Calculate paths per second
        let small_paths_per_sec = small_paths.len() as f64 / small_duration.as_secs_f64();
        let medium_paths_per_sec = medium_paths.len() as f64 / medium_duration.as_secs_f64();
        let large_paths_per_sec = large_paths.len() as f64 / large_duration.as_secs_f64();

        log_info!(&format!("Indexing throughput:"));
        log_info!(&format!("  Small: {:.2} paths/sec", small_paths_per_sec));
        log_info!(&format!("  Medium: {:.2} paths/sec", medium_paths_per_sec));
        log_info!(&format!("  Large: {:.2} paths/sec", large_paths_per_sec));
    }

    #[test]
    fn benchmark_query_performance() {
        // Create matcher with real test data instead of synthetic paths
        let mut matcher = PathMatcher::new();
        let test_paths = collect_test_paths(Some(10000));

        log_info!(&format!("Benchmarking query performance with {} real paths", test_paths.len()));

        for path in &test_paths {
            matcher.add_path(path);
        }

        // Test queries of different lengths and complexity
        let queries = [
            "f",           // Single character
            "fi",          // Two characters
            "file",        // Common term
            "banana",    // Common term in real data
            "nonexistent", // No matches
            "flie",        // Misspelled
            "bannana",    // Misspelled real term
        ];

        log_info!(&format!("Query performance benchmark:"));

        for query in &queries {
            let (results, duration) = run_benchmark(&format!("query '{}'", query), 10, || {
                matcher.search(query, 10)
            });

            log_info!(&format!("  Query '{}' took {:?} and found {} results",
                      query, duration, results.len()));
        }
    }

    #[test]
    fn benchmark_comparison_with_alternatives() {
        // Use real test data instead of synthetic paths
        let test_paths = collect_test_paths(Some(1000));
        let mut matcher = PathMatcher::new();

        log_info!(&format!("Benchmarking search methods with {} real paths", test_paths.len()));

        for path in &test_paths {
            matcher.add_path(path);
        }

        // Extract a search term that will likely exist in the real dataset
        let search_term = if !test_paths.is_empty() {
            let sample_path = &test_paths[test_paths.len() / 2];
            let components: Vec<&str> = sample_path.split(std::path::MAIN_SEPARATOR).collect();
            if let Some(filename) = components.last() {
                if filename.len() >= 4 {
                    &filename[0..4]
                } else {
                    "file"
                }
            } else {
                "file"
            }
        } else {
            "file"
        };

        log_info!(&format!("Using search term '{}' derived from real data", search_term));

        // Benchmark our implementation
        let (our_results, our_duration) = run_benchmark("our fuzzy search", 20, || {
            matcher.search(search_term, 10)
        });

        // Benchmark simple substring search
        let (substr_results, substr_duration) = run_benchmark("substring search", 20, || {
            let query = search_term.to_lowercase();
            test_paths.iter()
                .filter(|path| path.to_lowercase().contains(&query))
                .map(|path| (path.clone(), 1.0))
                .take(10)
                .collect::<Vec<(String, f32)>>()
        });

        // Benchmark regex search
        let (regex_results, regex_duration) = run_benchmark("regex search", 20, || {
            let regex_pattern = format!("(?i){}", regex::escape(search_term));
            match regex::Regex::new(&regex_pattern) {
                Ok(re) => test_paths.iter()
                    .filter(|path| re.is_match(path))
                    .map(|path| (path.clone(), 1.0))
                    .take(10)
                    .collect::<Vec<(String, f32)>>(),
                Err(_) => Vec::new(),
            }
        });

        log_info!(&format!("Search method comparison:"));
        log_info!(&format!("  Our fuzzy search: {:?} with {} results", our_duration, our_results.len()));
        log_info!(&format!("  Substring search: {:?} with {} results", substr_duration, substr_results.len()));
        log_info!(&format!("  Regex search: {:?} with {} results", regex_duration, regex_results.len()));

        let fuzzy_vs_substr = substr_duration.as_nanos() as f64 / our_duration.as_nanos() as f64;
        let fuzzy_vs_regex = regex_duration.as_nanos() as f64 / our_duration.as_nanos() as f64;

        log_info!(&format!("  Our fuzzy search is {:.2}x {} than substring search",
                  fuzzy_vs_substr.abs(), if fuzzy_vs_substr > 1.0 { "faster" } else { "slower" }));
        log_info!(&format!("  Our fuzzy search is {:.2}x {} than regex search",
                  fuzzy_vs_regex.abs(), if fuzzy_vs_regex > 1.0 { "faster" } else { "slower" }));
    }

    #[test]
    fn test_different_query_lengths() {
        // Use real test data
        let test_paths = collect_test_paths(Some(1000));
        let mut matcher = PathMatcher::new();

        log_info!(&format!("Testing query lengths with {} real paths", test_paths.len()));

        for path in &test_paths {
            matcher.add_path(path);
        }

        // Extract realistic search terms from the test data
        let realistic_terms: Vec<String> = if !test_paths.is_empty() {
            let mut terms = Vec::new();
            for path in test_paths.iter().take(5) {
                if let Some(filename) = path.split('/').last().or_else(|| path.split('\\').last()) {
                    if filename.len() >= 3 {
                        terms.push(filename[0..3].to_string());
                    }
                }
            }
            if terms.is_empty() {
                vec!["f".to_string(), "fi".to_string(), "fil".to_string(),
                     "file".to_string(), "file.".to_string(), "file.t".to_string()]
            } else {
                terms
            }
        } else {
            vec!["f".to_string(), "fi".to_string(), "fil".to_string(),
                 "file".to_string(), "file.".to_string(), "file.t".to_string()]
        };

        // Test different query lengths with real terms
        for query in &realistic_terms {
            let (results, elapsed) = run_benchmark(&format!("query length '{}'", query.len()), 5, || {
                matcher.search(query, 10)
            });

            log_info!(&format!("Query '{}' (length {}) took {:.2?} with {} results",
                     query, query.len(), elapsed, results.len()));

            assert!(!results.is_empty() || query.len() < 3);
        }

        // Test different query lengths with misspellings of real terms
        let misspelled_terms: Vec<String> = realistic_terms.iter()
            .filter(|term| term.len() >= 3)
            .map(|term| {
                let chars: Vec<char> = term.chars().collect();
                if chars.len() >= 2 {
                    // Swap two adjacent characters for a simple misspelling
                    let mut misspelled = chars.clone();
                    misspelled.swap(0, 1);
                    misspelled.iter().collect()
                } else {
                    term.clone()
                }
            })
            .collect();

        for query in &misspelled_terms {
            let (results, elapsed) = run_benchmark(&format!("misspelled query '{}'", query), 5, || {
                matcher.search(query, 10)
            });

            log_info!(&format!("Misspelled query '{}' (length {}) took {:.2?} with {} results",
                     query, query.len(), elapsed, results.len()));

            assert!(!results.is_empty() || query.len() < 3);
        }
    }

    #[test]
    fn test_variation_generation() {
        let matcher = PathMatcher::new();

        // Test short query
        let variations = matcher.generate_efficient_variations("cat");
        assert!(!variations.is_empty());
        assert!(variations.contains(&"at".to_string())); // Deletion
        assert!(variations.contains(&"act".to_string())); // Transposition

        // Test longer query
        let variations = matcher.generate_efficient_variations("document");
        assert!(!variations.is_empty());

        // Test very short query
        let variations = matcher.generate_efficient_variations("a");
        assert!(variations.is_empty(), "No variations for single character");
    }

    #[test]
    fn test_misspelled_queries() {
        let mut matcher = PathMatcher::new();

        // Add some test paths
        matcher.add_path("/documents/presentation.pptx");
        matcher.add_path("/images/vacation/beach.jpg");
        matcher.add_path("/music/favorite_song.mp3");
        matcher.add_path("/code/project/main.rs");

        // Test various misspellings with different severity
        let test_cases = [
            // (correct, misspelled)
            ("presentation", "persentaton"),  // multiple errors
            ("beach", "beech"),               // single vowel error
            ("favorite", "favorit"),          // missing letter
            ("music", "musik"),               // phonetic error
            ("project", "progect"),           // single consonant error
            ("images", "imaegs"),             // transposed letters
            ("vacation", "vacasion"),         // phonetic substitution
            ("documents", "dokumentz"),       // multiple substitutions
            ("code", "kode")                  // spelling variation
        ];

        for (correct, misspelled) in &test_cases {
            // Search using the misspelled query
            let results = matcher.search(misspelled, 10);

            // Log the search results for debugging
            log_info!(&format!("Search for misspelled '{}' (should match '{}') returned {} results",
                misspelled, correct, results.len()));

            // Verify we have some results
            assert!(!results.is_empty(), "Misspelled query '{}' should find results", misspelled);

            // Check if the result contains the path with the correct spelling
            let expected_path = results.iter()
                .find(|(path, _)| path.to_lowercase().contains(&correct.to_lowercase()));

            assert!(expected_path.is_some(),
                "Misspelled query '{}' should have found a path containing '{}'",
                misspelled, correct);

            // Log the score to help with tuning
            if let Some((path, score)) = expected_path {
                log_info!(&format!("  Found '{}' with score {:.4}", path, score));
            }
        }
    }

    // Test using real-world data generation
    #[test]
    #[cfg(feature = "long-tests")]
    fn test_with_generated_real_world_data() {
        // Get the test data path
        let test_path = get_test_data_path();

        log_info!(&format!("Loading test data from: {:?}", test_path));

        // Now build our PathMatcher with the generated data
        let mut matcher = PathMatcher::new();
        let mut path_count = 0;

        // Walk the directory and add all paths to the matcher
        if let Some(walker) = std::fs::read_dir(&test_path).ok() {
            for entry in walker.filter_map(|e| e.ok()) {
                if let Some(path_str) = entry.path().to_str().map(|s| s.to_string()) {
                    matcher.add_path(&path_str);
                    path_count += 1;

                    // Also process subdirectories
                    if entry.path().is_dir() {
                        if let Some(subwalker) = std::fs::read_dir(entry.path()).ok() {
                            for subentry in subwalker.filter_map(|e| e.ok()) {
                                if let Some(sub_path_str) = subentry.path().to_str().map(|s| s.to_string()) {
                                    matcher.add_path(&sub_path_str);
                                    path_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        log_info!(&format!("Added {} paths to PathMatcher", path_count));
        assert!(path_count > 0, "Should have indexed at least some paths");

        // Test searching with some realistic terms
        let search_terms = ["banana", "txt", "mp3", "apple"];

        for term in &search_terms {
            let start = Instant::now();
            let results = matcher.search(term, 20);
            let elapsed = start.elapsed();

            log_info!(&format!("Search for '{}' found {} results in {:.2?}",
                term, results.len(), elapsed));

            // Print top 3 results (if any)
            for (i, (path, score)) in results.iter().take(3).enumerate() {
                log_info!(&format!("  Result #{}: {} (score: {:.4})", i+1, path, score));
            }
        }

        // Test searching with some realistic terms including misspellings
        let search_terms = ["bananna", "txt", "mp3", "aple"];  // misspelled banana and apple

        for term in &search_terms {
            let start = Instant::now();
            let results = matcher.search(term, 20);
            let elapsed = start.elapsed();

            log_info!(&format!("Search for misspelled '{}' found {} results in {:.2?}",
                term, results.len(), elapsed));

            // Print top 3 results (if any)
            for (i, (path, score)) in results.iter().take(3).enumerate() {
                log_info!(&format!("  Result #{}: {} (score: {:.4})", i+1, path, score));
            }
        }
    }

    // Benchmark comparing fuzzy vs substring matching
    #[test]
    fn benchmark_fuzzy_vs_substring_matching() {
        // Get test data
        let test_path = get_test_data_path();

        // Build PathMatcher
        let mut matcher = PathMatcher::new();
        let mut all_paths = Vec::new();

        // Walk the directory and collect paths
        if let Some(walker) = std::fs::read_dir(&test_path).ok() {
            for entry in walker.filter_map(|e| e.ok()) {
                if let Some(path_str) = entry.path().to_str().map(|s| s.to_string()) {
                    matcher.add_path(&path_str);
                    all_paths.push(path_str);
                }
            }
        }

        log_info!(&format!("Loaded {} paths for benchmark", all_paths.len()));
        assert!(!all_paths.is_empty(), "Should have loaded some paths");

        // Test terms to search for
        let search_terms = ["apple", "banana", "txt", "mp3", "orange"];

        for term in &search_terms {
            // Benchmark fuzzy search
            let fuzzy_start = Instant::now();
            let fuzzy_results = matcher.search(term, 20);
            let fuzzy_elapsed = fuzzy_start.elapsed();

            // Benchmark simple substring matching
            let substring_start = Instant::now();
            let substring_results: Vec<(String, f32)> = all_paths.iter()
                .filter(|path| path.to_lowercase().contains(&term.to_lowercase()))
                .map(|path| (path.clone(), 1.0))
                .take(20)
                .collect();
            let substring_elapsed = substring_start.elapsed();

            log_info!(&format!(
                "Search for '{}': Fuzzy found {} results in {:.2?}, Substring found {} results in {:.2?}",
                term, fuzzy_results.len(), fuzzy_elapsed, substring_results.len(), substring_elapsed
            ));
        }
    }

    // Test performance on larger dataset
    #[test]
    fn test_large_dataset_performance() {
        // Get the test data directory
        let test_path = get_test_data_path();

        let start_time = Instant::now();
        let mut matcher = PathMatcher::new();
        let mut path_count = 0;

        // Recursively add all paths from the test directory
        fn add_paths_from_dir(dir: &std::path::Path, matcher: &mut PathMatcher, count: &mut usize) {
            if let Some(walker) = std::fs::read_dir(dir).ok() {
                for entry in walker.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if let Some(path_str) = path.to_str() {
                        matcher.add_path(path_str);
                        *count += 1;
                    }

                    if path.is_dir() {
                        add_paths_from_dir(&path, matcher, count);
                    }
                }
            }
        }

        add_paths_from_dir(&test_path, &mut matcher, &mut path_count);
        let indexing_time = start_time.elapsed();

        log_info!(&format!("Indexed {} paths in {:.2?}", path_count, indexing_time));

        // Test search performance with a variety of terms
        let query_terms = ["file", "banana", "txt", "mp3", "orange", "apple", "e"];

        for term in &query_terms {
            let search_start = Instant::now();
            let results = matcher.search(term, 50);
            let search_time = search_start.elapsed();

            log_info!(&format!("Search for '{}' found {} results in {:.2?}",
                term, results.len(), search_time));
        }
    }

    // Test to create test data directory if it doesn't exist
    #[test]
    #[ignore] // Only run when needed to generate test data
    fn create_test_data() {

        let base_path = PathBuf::from("./test-data-for-fuzzy-search");
        match super::super::generate_test_data(base_path) {
            Ok(path) => log_info!(&format!("Test data generated successfully at {:?}", path)),
            Err(e) => panic!("Failed to generate test data: {}", e),
        }
    }
}
