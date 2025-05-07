use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::log_info;

type TrigramMap = HashMap<Trigram, HashSet<usize>>;

// Custom Trigram type with fast hashing
#[derive(Debug, Clone, Copy, Eq)]
struct Trigram(u32);

impl Trigram {
    fn new(a: char, b: char, c: char) -> Self {
        // Pack three characters into a single u32 for fast comparison
        let a_val = a as u32 & 0xFF;
        let b_val = b as u32 & 0xFF;
        let c_val = c as u32 & 0xFF;
        Trigram((a_val << 16) | (b_val << 8) | c_val)
    }
}

impl PartialEq for Trigram {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Trigram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(self.0);
    }
}

struct PathMatcher {
    paths: Vec<String>,
    trigram_index: TrigramMap,
}

impl PathMatcher {
    fn new() -> Self {
        PathMatcher {
            paths: Vec::new(),
            trigram_index: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        PathMatcher {
            paths: Vec::with_capacity(capacity),
            trigram_index: HashMap::with_capacity(capacity * 5), // Efficient pre-allocation
        }
    }

    fn add_path(&mut self, path: &str) {
        let path_index = self.paths.len();
        self.paths.push(path.to_string());

        // Generate trigrams and add to index
        let trigrams = Self::extract_trigrams(path);
        for trigram in trigrams {
            self.trigram_index
                .entry(trigram)
                .or_insert_with(HashSet::new)
                .insert(path_index);
        }
    }

    fn extract_trigrams(text: &str) -> Vec<Trigram> {
        // because we are using a trigram, we need at least 3 characters
        if text.len() < 3 {
            return Vec::new();
        }

        // Pad the string with spaces to handle edge cases (otherwise they would only appear in one trigram)
        let padded = format!("  {}  ", text.to_lowercase());
        let chars: Vec<char> = padded.chars().collect();

        let mut trigrams = Vec::with_capacity(chars.len() - 2);
        for i in 0..chars.len() - 2 {
            trigrams.push(Trigram::new(chars[i], chars[i + 1], chars[i + 2]));
        }
        trigrams
    }

    fn search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        // debugging todo
        let start_time = Instant::now();

        // Handle empty queries
        if query.is_empty() {
            return Vec::new();
        }

        let query_trigrams = Self::extract_trigrams(query);
        // If the query is too short, query_trigrams will be empty
        if query_trigrams.is_empty() {
            return Vec::new();
        }

        // Fast path: create a bitmap of matching paths for efficient filtering
        let mut path_matches = vec![0u32; (self.paths.len() + 31) / 32];
        let mut total_matches = 0;

        // Count matching trigrams for each path using bit operations
        for trigram in &query_trigrams {
            if let Some(path_indices) = self.trigram_index.get(trigram) {
                for &path_idx in path_indices {
                    let word_idx = path_idx / 32;
                    let bit_idx = path_idx % 32;
                    let old_word = path_matches[word_idx];
                    let new_word = old_word | (1 << bit_idx);
                    if old_word != new_word {
                        total_matches += 1;
                    }
                    path_matches[word_idx] = new_word;
                }
            }
        }

        // Early exit for no matches
        if total_matches == 0 {
            return self.fallback_search(query, max_results);
        }

        let mut results = Vec::with_capacity(total_matches.min(max_results * 2));
        for word_idx in 0..path_matches.len() {
            let mut word = path_matches[word_idx];
            while word != 0 {
                let bit_idx = word.trailing_zeros() as usize;
                let path_idx = word_idx * 32 + bit_idx;

                if path_idx < self.paths.len() {
                    let path = &self.paths[path_idx];

                    // Fast scoring using coefficient calculation
                    let path_trigrams = Self::extract_trigrams(path);
                    let common = Self::count_common_trigrams(&query_trigrams, &path_trigrams);

                    // Calculate Jaccard similarity coefficient (J(A, B) = |A ∩ B| / |A ∪ B|, for any sets A and B) && 0 <= score <= 1
                    let score = common as f32 / (query_trigrams.len() + path_trigrams.len() - common) as f32;

                    let path_lower = path.to_lowercase();
                    let query_lower = query.to_lowercase();

                    // Fast substring check (bonus for exact matches)
                    let mut bonus = if path_lower.contains(&query_lower) {
                        0.3
                    } else {
                        0.0
                    };

                    // Further bonus for file extension matches
                    if let Some(dot_pos) = query_lower.find('.') {
                        let query_ext = &query_lower[dot_pos..];
                        if path_lower.ends_with(query_ext) {
                            bonus += 0.15;
                        }
                    }

                    results.push((path.clone(), score + bonus));
                }

                // Clear the bit we just processed and continue
                word &= !(1 << bit_idx);
            }
        }

        // Sort by score (descending)
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top matches
        results.truncate(max_results);

        // for debugging todo
        let elapsed = start_time.elapsed();
        log_info!(&format!("Search for '{}' completed in {:.2?} with {} results",
                         query, elapsed, results.len()));

        if results.is_empty() {
            return self.fallback_search(query, max_results);
        }

        results
    }


    // Fast count of common elements between two sorted vectors
    fn count_common_trigrams(a: &[Trigram], b: &[Trigram]) -> usize {
        let mut count = 0;
        let mut i = 0;
        let mut j = 0;

        while i < a.len() && j < b.len() {
            if a[i].0 < b[j].0 {
                i += 1;
            } else if a[i].0 > b[j].0 {
                j += 1;
            } else {
                count += 1;
                i += 1;
                j += 1;
            }
        }

        count
    }

    // Optimized fallback for handling misspellings - only used when main search fails
    fn fallback_search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        // Only generate key variations to minimize processing time
        let variations = self.generate_key_variations(query);
        let mut all_results = Vec::with_capacity(max_results * 2);

        for variant in variations {
            let query_trigrams = Self::extract_trigrams(&variant);
            if query_trigrams.is_empty() {
                continue;
            }

            // Fast path for variant matching
            let mut candidates = HashMap::new();
            for trigram in &query_trigrams {
                if let Some(path_indices) = self.trigram_index.get(trigram) {
                    for &path_idx in path_indices {
                        *candidates.entry(path_idx).or_insert(0) += 1;
                    }
                }
            }

            // Score only the top candidates
            let mut variant_results = Vec::with_capacity(candidates.len().min(max_results));
            for (path_idx, matches) in candidates {
                let path = &self.paths[path_idx];

                // Simplified scoring for variants
                let score = matches as f32 / query_trigrams.len() as f32 * 0.9; // 90% of normal score
                variant_results.push((path.clone(), score));
            }

            // Only keep top results from this variant
            variant_results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            variant_results.truncate(max_results);

            all_results.extend(variant_results);
        }

        // Final sorting and deduplication
        all_results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        all_results.retain(|(path, _)| seen.insert(path.clone()));

        all_results.truncate(max_results);
        all_results
    }

    fn generate_key_variations(&self, query: &str) -> Vec<String> {
        if query.len() <= 1 {
            return vec![query.to_string()];
        }

        let mut variations = Vec::with_capacity(query.len() * 2);
        let chars: Vec<char> = query.chars().collect();

        // 1. Character deletions (extremely fast and catches many typos)
        for i in 0..chars.len() {
            let mut new_chars = chars.clone();
            new_chars.remove(i);
            variations.push(new_chars.iter().collect());
        }

        // 2. Adjacent character transpositions
        for i in 0..chars.len().saturating_sub(1) {
            let mut new_chars = chars.clone();
            new_chars.swap(i, i + 1);
            variations.push(new_chars.iter().collect());
        }

        // Only add more expensive variations for short queries
        if query.len() <= 5 {
            // 3. Character substitutions (only for common substitutions)
            for i in 0..chars.len() {
                match chars[i] {
                    'a' => variations.push(self.replace_char_at(&chars, i, 'e')),
                    'e' => variations.push(self.replace_char_at(&chars, i, 'a')),
                    'i' => variations.push(self.replace_char_at(&chars, i, 'y')),
                    'o' => variations.push(self.replace_char_at(&chars, i, 'u')),
                    's' => variations.push(self.replace_char_at(&chars, i, 'z')),
                    _ => {}
                }
            }
        }

        variations
    }

    // Helper to efficiently replace a character at a specific position
    fn replace_char_at(&self, chars: &[char], index: usize, replacement: char) -> String {
        let mut result = String::with_capacity(chars.len());
        for (i, &c) in chars.iter().enumerate() {
            if i == index {
                result.push(replacement);
            } else {
                result.push(c);
            }
        }
        result
    }
}

pub fn generate_test_data(base_path: PathBuf) -> Result<PathBuf, std::io::Error> {
    use std::fs::{create_dir_all, File};
    use rand::{thread_rng, Rng};
    use std::time::Instant;

    // Constants for the directory structure
    const FOLDERS_PER_LEVEL: usize = 20;
    const FILES_PER_FOLDER: usize = 20;
    const MAX_DEPTH: usize = 3;

    // Remove the directory if it already exists
    if base_path.exists() {
        println!("Removing existing test data at: {:?}", base_path);
        std::fs::remove_dir_all(&base_path)?;
    }

    // Create the base directory
    create_dir_all(&base_path)?;
    println!("Creating test data at: {:?}", base_path);

    let start_time = Instant::now();

    // Function to generate random strings based on a predefined set
    let generate_random_name = || -> String {
        let charset: Vec<&str> = "banana, apple, orange, grape, watermelon, kiwi, mango, peach, cherry, \
        strawberry, blueberry, raspberry, blackberry, lemon, lime, coconut, papaya, pineapple, tangerine, \
        car, truck, motorcycle, bicycle, bus, train, airplane, helicopter, boat, ship, submarine, scooter, van, \
        ambulance, taxi, firetruck, tractor, yacht, jetski, speedboat, racecar".split(",").collect::<Vec<_>>();

        let mut rng = thread_rng();

        let idx = rng.gen_range(0, charset.len());
        return charset[idx].to_string();
    };

    // Function to create file extensions
    let generate_extension = || -> &str {
        const EXTENSIONS: [&str; 20] = [
            "txt", "pdf", "doc", "jpg", "png", "mp3", "mp4", "html", "css", "js",
            "rs", "json", "xml", "md", "csv", "zip", "exe", "dll", "sh", "py"
        ];

        let mut rng = thread_rng();
        let idx = rng.gen_range(0, EXTENSIONS.len());
        EXTENSIONS[idx]
    };

    // Counter to track progress
    let entry_count = Arc::new(Mutex::new(0usize));

    // Recursive function to create the folder structure
    fn create_structure(
        path: &PathBuf,
        depth: usize,
        max_depth: usize,
        folders_per_level: usize,
        files_per_folder: usize,
        name_generator: &dyn Fn() -> String,
        ext_generator: &dyn Fn() -> &'static str,
        counter: &Arc<Mutex<usize>>,
    ) -> Result<(), std::io::Error> {
        // Create files in current folder
        for _ in 0..files_per_folder {
            let file_name = format!("{}.{}", name_generator(), ext_generator());
            let file_path = path.join(file_name);
            File::create(file_path)?;

            // Increment counter
            if let Ok(mut count) = counter.lock() {
                *count += 1;
                if *count % 1000 == 0 {
                    println!("Created {} entries so far...", *count);
                }
            }
        }

        // Stop creating subfolders if we've reached max depth
        if depth >= max_depth {
            return Ok(());
        }

        // Create subfolders and recurse
        for _ in 0..folders_per_level {
            let folder_name = name_generator();
            let folder_path = path.join(folder_name);
            create_dir_all(&folder_path)?;

            // Increment counter for folder
            if let Ok(mut count) = counter.lock() {
                *count += 1;
            }

            // Recurse into subfolder
            create_structure(
                &folder_path,
                depth + 1,
                max_depth,
                folders_per_level,
                files_per_folder,
                name_generator,
                ext_generator,
                counter,
            )?;
        }

        Ok(())
    }

    // Start the recursive creation
    create_structure(
        &base_path,
        0,
        MAX_DEPTH,
        FOLDERS_PER_LEVEL,
        FILES_PER_FOLDER,
        &generate_random_name,
        &generate_extension,
        &entry_count,
    )?;

    let total_count = *entry_count.lock().unwrap();
    let elapsed = start_time.elapsed();

    println!("Test data generation complete!");
    println!("Created {} total entries in {:?}", total_count, elapsed);
    println!("Path: {:?}", base_path);

    Ok(base_path)
}

#[cfg(test)]
mod tests_fast_fuzzy_v1 {
    use super::*;
    use std::time::Instant;
    use std::path::PathBuf;
    use std::env;

    #[test]
    fn test_trigram_creation() {
        let trigram = Trigram::new('a', 'b', 'c');
        let expected = ('a' as u32 & 0xFF) << 16 | ('b' as u32 & 0xFF) << 8 | ('c' as u32 & 0xFF);
        assert_eq!(trigram.0, expected);
    }

    #[test]
    fn test_trigram_equality() {
        let t1 = Trigram::new('a', 'b', 'c');
        let t2 = Trigram::new('a', 'b', 'c');
        let t3 = Trigram::new('x', 'y', 'z');

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_extract_trigrams() {
        // Test empty or short strings
        assert!(PathMatcher::extract_trigrams("").is_empty());
        assert!(PathMatcher::extract_trigrams("ab").is_empty());

        // Test normal string
        let trigrams = PathMatcher::extract_trigrams("abc");
        assert_eq!(trigrams.len(), 5); // "  abc  " -> 7 trigrams
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

        assert!(found_exact_misspelled, "Should find the exact match path despite misspelling");
        assert!(found_partial_misspelled || found_rust_src_misspelled,
               "Should find at least one of the partial matches despite misspelling");
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
    fn test_case_insensitivity() {
        let mut matcher = PathMatcher::new();

        matcher.add_path("/path/to/UPPERCASE.txt");
        matcher.add_path("/path/to/lowercase.txt");

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
        assert_eq!(misspelled_results[0].0, "/path/to/UPPERCASE.txt");

        let misspelled_results = matcher.search("LWORECASE", 10);  // 'LOWERCASE' misspelled
        assert!(!misspelled_results.is_empty());
        assert_eq!(misspelled_results[0].0, "/path/to/lowercase.txt");
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

        let start = Instant::now();
        let results = matcher.search("file", 10);
        let elapsed = start.elapsed();

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

        let start = Instant::now();
        let results = matcher.search("file", 10);
        let elapsed = start.elapsed();

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

        let start = Instant::now();
        let results = matcher.search("file", 10);
        let elapsed = start.elapsed();

        assert!(!results.is_empty());
        log_info!(&format!("Large dataset (10,000 items) search took: {:.2?}", elapsed));
    }

    #[test]
    fn test_trigram_extraction_performance() {
        let long_path = "/very/long/path/with/many/directories/and/a/long/filename/that/should/generate/many/trigrams.txt";

        let start = Instant::now();
        let trigrams = PathMatcher::extract_trigrams(long_path);
        let elapsed = start.elapsed();

        log_info!(&format!("Extracted {} trigrams from long path in {:.2?}", trigrams.len(), elapsed));
        assert!(!trigrams.is_empty());
    }

    #[test]
    fn test_different_query_lengths() {
        let mut matcher = PathMatcher::new();

        // Add some test paths
        for i in 0..1000 {
            matcher.add_path(&format!("/path/to/sample/directory/file{}.txt", i));
        }

        // Test different query lengths
        let queries = vec!["f", "fi", "fil", "file", "file.t", "file.txt"];

        for query in queries {
            let start = Instant::now();
            let results = matcher.search(query, 10);
            let elapsed = start.elapsed();

            log_info!(&format!("Query '{}' (length {}) took {:.2?} with {} results",
                             query, query.len(), elapsed, results.len()));

            assert!(!results.is_empty() || query.len() < 3);
        }

        // Test different query lengths with misspellings
        let queries = vec![
            "f", "fi", "fil", "flie", "fiel.t", "flie.txy"  // misspelled versions
        ];

        for query in queries {
            let start = Instant::now();
            let results = matcher.search(query, 10);
            let elapsed = start.elapsed();

            log_info!(&format!("Misspelled query '{}' (length {}) took {:.2?} with {} results",
                             query, query.len(), elapsed, results.len()));

            assert!(!results.is_empty() || query.len() < 3);
        }
    }

    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            panic!("Test data directory does not exist: {:?}. Run the 'create_test_data' test first.", path);
        }
        path
    }

    // New test using real-world data generation
    #[test]
    #[cfg(feature = "long-tests")]
    fn test_with_generated_real_world_data() {
        // Generate test data using the function from parent module
        let test_path = get_test_data_path();

        log_info!("Test data generated successfully");

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

    // Test for comparison with traditional string matching
    #[test]
    fn benchmark_fuzzy_vs_substring_matching() {
        // First generate and load test data
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
        use std::path::Path;
        // Get the test data directory or generate it if needed
        let test_path = get_test_data_path();

        let start_time = Instant::now();
        let mut matcher = PathMatcher::new();
        let mut path_count = 0;

        // Recursively add all paths from the test directory
        fn add_paths_from_dir(dir: &Path, matcher: &mut PathMatcher, count: &mut usize) {
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

    // Helper function to create a path matcher with test data
    fn create_test_matcher() -> PathMatcher {
        let mut matcher = PathMatcher::new();

        // Add a variety of paths for testing
        matcher.add_path("/home/user/documents/resume.pdf");
        matcher.add_path("/home/user/pictures/vacation/beach.jpg");
        matcher.add_path("/home/user/music/favorite_song.mp3");
        matcher.add_path("/home/user/code/projects/rust/main.rs");
        matcher.add_path("/home/user/downloads/presentation.pptx");
        matcher.add_path("/opt/applications/browser.exe");
        matcher.add_path("/var/log/system.log");
        matcher.add_path("/etc/config/settings.json");

        matcher
    }

    #[test]
    fn test_fuzzy_vs_exact_matching() {
        let matcher = create_test_matcher();

        // Test exact matches
        let exact_results = matcher.search("beach.jpg", 10);
        assert!(!exact_results.is_empty(), "Should find exact matches");

        // Test fuzzy matches with increasing fuzziness
        let slightly_fuzzy = matcher.search("beech.jpg", 10);  // one character different
        let more_fuzzy = matcher.search("beaach.jppg", 10);    // two characters different
        let very_fuzzy = matcher.search("beatch.jog", 10);     // multiple differences

        log_info!(&format!("Exact: {} results, Slightly fuzzy: {}, More fuzzy: {}, Very fuzzy: {}",
            exact_results.len(), slightly_fuzzy.len(), more_fuzzy.len(), very_fuzzy.len()));

        // At least the slightly fuzzy search should still find results
        assert!(!slightly_fuzzy.is_empty(), "Should find slightly fuzzy matches");

        // Verify the decrease in matching score as fuzziness increases
        if !exact_results.is_empty() && !slightly_fuzzy.is_empty() {
            let exact_score = exact_results[0].1;
            let fuzzy_score = slightly_fuzzy[0].1;
            assert!(exact_score >= fuzzy_score,
                "Exact match score ({:.4}) should be higher than fuzzy match score ({:.4})",
                exact_score, fuzzy_score);
        }
    }
}
