//! # Fast Fuzzy Path Matcher
//!
//! A high-performance fuzzy path matching engine using trigram indexing for efficient searches
//! through large collections of file paths. This implementation provides sub-linear search
//! performance even with hundreds of thousands of paths.
//!
//! ## Use Cases
//!
//! - **File Explorers**: Quickly find files and folders by partial name, even with typos
//! - **Command Palettes**: Implement fuzzy command matching like in VS Code or JetBrains IDEs
//! - **Autocompletion**: Power autocomplete for paths, filenames, or any textual data
//! - **Search Fields**: Backend for "search-as-you-type" interfaces with typo tolerance
//!
//! ## Performance Benchmarks
//!
//! Empirical measurements show sub-linear scaling with path count:
//!
//! | Paths    | Avg Search Time (µs) | Scaling Factor |
//! |----------|----------------------|----------------|
//! | 10       | 8.05                 | -              |
//! | 100      | 25.21                | 3.1×           |
//! | 1,000    | 192.05               | 7.6×           |
//! | 10,000   | 548.39               | 2.9×           |
//! | 170,456  | 3,431.88             | 6.3×           |
//!
//! With 10× more paths, search is typically only 3-7× slower, demonstrating **O(n^a)**
//! scaling where **a ≈ 0.5-0.7**.
//!
//! ## Comparison to Other Algorithms
//!
//! | Algorithm                  | Theoretical Complexity | Practical Scaling | Suitability       |
//! |----------------------------|------------------------|-------------------|-------------------|
//! | Levenshtein (brute force)  | O(N*M²)                | Linear/Quadratic  | Poor for large N  |
//! | Substring/Regex (scan)     | O(N*Q)                 | Linear            | Poor for large N  |
//! | Trie/Prefix Tree           | O(Q)                   | Sub-linear        | Good for prefixes |
//! | **Trigram Index (this)**   | **O(Q+S)**             | **Sub-linear**    | **Best for large N** |
//! | FZF/Sublime fuzzy scan     | O(N*Q)                 | Linear            | Good for small N  |
//!
//! Where:
//! - N = number of paths
//! - M = average string length
//! - Q = query length
//! - S = number of candidate paths (typically S << N)
//!
//! ## Features
//!
//! - Handles typos, transpositions, and character substitutions
//! - Case-insensitive matching with fast character mapping
//! - Boosts exact matches and filename matches over partial matches
//! - Length normalization to prevent bias toward longer paths
//! - Memory-efficient trigram storage with FxHashMap and SmallVec

use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::{smallvec, SmallVec};
use std::sync::Once;

type TrigramMap = FxHashMap<u32, SmallVec<[u32; 4]>>;

static CHAR_MAPPING_INIT: Once = Once::new();
static mut CHAR_MAPPING: [u8; 256] = [0; 256];

/// A fast fuzzy path matching engine that uses trigram indexing for efficient searches.
/// The PathMatcher enables rapid searching through large collections of file paths
/// with support for fuzzy matching, allowing for typos and variations in search queries.
///
/// # Time Complexity
/// Overall search complexity scales sub-linearly with the number of paths (O(n^a) where a ≈ 0.5-0.7),
/// significantly outperforming traditional algorithms like Levenshtein (O(N*M²)) or
/// simple substring matching (O(N*Q)).
pub struct PathMatcher {
    paths: Vec<String>,
    trigram_index: TrigramMap,
}

impl PathMatcher {
    /// Creates a new PathMatcher instance with empty path collection and trigram index.
    /// Initializes internal character mapping for fast case folding.
    ///
    /// # Returns
    /// * A new empty PathMatcher instance ready for indexing paths.
    ///
    /// # Example
    /// ```rust
    /// let matcher = PathMatcher::new();
    /// assert_eq!(matcher.search("test", 10).len(), 0); // Empty matcher returns no results
    /// ```
    ///
    /// # Time Complexity
    /// * O(1) - Constant time initialization
    pub fn new() -> Self {
        Self::init_char_mapping();

        PathMatcher {
            paths: Vec::new(),
            trigram_index: FxHashMap::with_capacity_and_hasher(4096, Default::default()),
        }
    }

    /// Initializes the static character mapping table for fast case-insensitive comparisons.
    /// This is called once during the first instantiation of a PathMatcher.
    ///
    /// The mapping table is used for efficient lowercase conversion without
    /// having to use the more expensive Unicode-aware to_lowercase() function.
    fn init_char_mapping() {
        CHAR_MAPPING_INIT.call_once(|| unsafe {
            for i in 0..256 {
                let c = i as u8 as char;
                let lower = c.to_lowercase().next().unwrap_or(c) as u8;
                CHAR_MAPPING[i] = lower;
            }
        });
    }

    /// Converts a single byte character to lowercase using the pre-computed mapping table.
    /// This is much faster than using the standard to_lowercase() function for ASCII characters.
    ///
    /// # Arguments
    /// * `c` - The byte to convert to lowercase.
    ///
    /// # Returns
    /// * The lowercase version of the input byte.
    ///
    /// # Example
    /// ```rust
    /// assert_eq!(PathMatcher::fast_lowercase(b'A'), b'a');
    /// assert_eq!(PathMatcher::fast_lowercase(b'z'), b'z');
    /// ```
    #[inline(always)]
    fn fast_lowercase(c: u8) -> u8 {
        Self::init_char_mapping();
        unsafe { CHAR_MAPPING[c as usize] }
    }

    /// Adds a path to the matcher, indexing it for fast retrieval during searches.
    /// Each path is broken down into trigrams (3-character sequences) that are
    /// indexed for efficient fuzzy matching.
    ///
    /// # Arguments
    /// * `path` - The file path string to add to the matcher.
    ///
    /// # Example
    /// ```rust
    /// let mut matcher = PathMatcher::new();
    /// matcher.add_path("/home/user/documents/report.pdf");
    /// let results = matcher.search("report", 10);
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].0, "/home/user/documents/report.pdf");
    /// ```
    ///
    /// # Time Complexity
    /// * O(L) where L is the length of the path
    /// * Overall index construction is O(N*L) for N paths with average length L
    pub fn add_path(&mut self, path: &str) {
        let path_index = self.paths.len() as u32;
        self.paths.push(path.to_string());
        self.extract_and_index_trigrams(path, path_index);
    }

    /// Removes a path from the matcher and updates all indices accordingly.
    /// This maintains the integrity of the trigram index by adjusting the indices
    /// of paths that come after the removed path.
    ///
    /// # Arguments
    /// * `path` - The path string to remove from the matcher.
    ///
    /// # Returns
    /// * `true` if the path was found and removed.
    /// * `false` if the path was not in the matcher.
    ///
    /// # Example
    /// ```rust
    /// let mut matcher = PathMatcher::new();
    /// matcher.add_path("/home/user/file.txt");
    /// assert_eq!(matcher.search("file", 10).len(), 1);
    ///
    /// let removed = matcher.remove_path("/home/user/file.txt");
    /// assert!(removed);
    /// assert_eq!(matcher.search("file", 10).len(), 0);
    /// ```
    ///
    /// # Time Complexity
    /// * O(T) where T is the number of trigrams in the index
    /// * Worst case O(N) where N is the total number of paths
    pub fn remove_path(&mut self, path: &str) -> bool {
        if let Some(path_idx) = self.paths.iter().position(|p| p == path) {
            let path_idx = path_idx as u32;
            self.paths.remove(path_idx as usize);

            for values in self.trigram_index.values_mut() {
                values.retain(|idx| *idx != path_idx);
                for idx in values.iter_mut() {
                    if *idx > path_idx {
                        *idx -= 1;
                    }
                }
            }

            self.trigram_index.retain(|_, values| !values.is_empty());
            true
        } else {
            false
        }
    }

    /// Extracts trigrams from a text string and indexes them for the given path.
    /// Trigrams are 3-character sequences that serve as the basis for fuzzy matching.
    /// The path is padded with spaces to ensure edge characters are properly indexed.
    ///
    /// # Arguments
    /// * `text` - The text string to extract trigrams from.
    /// * `path_idx` - The index of the path in the paths collection.
    ///
    /// # Implementation Details
    /// This method pads the text with spaces, converts all characters to lowercase,
    /// and generates a trigram for each consecutive 3-character sequence.
    ///
    /// # Time Complexity
    /// * O(L) where L is the length of the text
    #[inline]
    fn extract_and_index_trigrams(&mut self, text: &str, path_idx: u32) {
        let bytes = text.as_bytes();
        if bytes.len() < 3 {
            return;
        }

        let mut trigram_bytes = [0u8; 3];
        let mut padded = Vec::with_capacity(bytes.len() + 4);
        padded.push(b' ');
        padded.push(b' ');
        padded.extend_from_slice(bytes);
        padded.push(b' ');
        padded.push(b' ');

        let mut seen_trigrams =
            FxHashSet::with_capacity_and_hasher(padded.len(), Default::default());

        for i in 0..padded.len() - 2 {
            trigram_bytes[0] = Self::fast_lowercase(padded[i]);
            trigram_bytes[1] = Self::fast_lowercase(padded[i + 1]);
            trigram_bytes[2] = Self::fast_lowercase(padded[i + 2]);

            let trigram = Self::pack_trigram(trigram_bytes[0], trigram_bytes[1], trigram_bytes[2]);

            // Skip duplicate trigrams within the same path
            if !seen_trigrams.insert(trigram) {
                continue;
            }

            match self.trigram_index.entry(trigram) {
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    let v = e.get_mut();
                    if v.is_empty() || v[v.len() - 1] != path_idx {
                        v.push(path_idx);
                    }
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    let mut v = smallvec![];
                    v.push(path_idx);
                    e.insert(v);
                }
            }
        }
    }

    /// Packs three bytes into a single u32 value for efficient trigram storage.
    /// Each byte occupies 8 bits in the resulting u32, allowing for compact
    /// representation of trigrams in memory.
    ///
    /// # Arguments
    /// * `a` - The first byte (most significant).
    /// * `b` - The second byte (middle).
    /// * `c` - The third byte (least significant).
    ///
    /// # Returns
    /// * A u32 value containing all three bytes packed together.
    #[inline(always)]
    fn pack_trigram(a: u8, b: u8, c: u8) -> u32 {
        ((a as u32) << 16) | ((b as u32) << 8) | (c as u32)
    }

    /// Calculates a normalization factor based on path length using a sigmoid function.
    /// This helps prevent unfair advantages for very long paths that naturally contain more trigrams.
    ///
    /// # Arguments
    /// * `path_length` - The length of the path in characters
    ///
    /// # Returns
    /// * A normalization factor between 0.5 and 1.0
    ///
    /// # Implementation Details
    /// Uses a sigmoid function to create a smooth transition from no penalty (factor=1.0)
    /// for short paths to a maximum penalty (factor=0.5) for very long paths.
    #[inline]
    fn calculate_length_normalization(&self, path_length: usize) -> f32 {
        // Constants to control the sigmoid curve
        const MIDPOINT: f32 = 100.0; // Path length at which penalty is 0.75
        const STEEPNESS: f32 = 0.03; // Controls how quickly penalty increases with length
        const MIN_FACTOR: f32 = 0.5; // Maximum penalty (minimum factor)

        // No penalty for very short paths
        if path_length < 30 {
            return 1.0;
        }

        // Sigmoid function: 1 - MIN_FACTOR/(1 + e^(-STEEPNESS * (x - MIDPOINT)))
        let length_f32 = path_length as f32;
        let sigmoid =
            1.0 - (1.0 - MIN_FACTOR) / (1.0 + (-STEEPNESS * (length_f32 - MIDPOINT)).exp());

        sigmoid
    }

    /// Searches for paths matching the given query string, supporting fuzzy matching.
    /// This method performs a trigram-based search that can find matches even when
    /// the query contains typos or spelling variations.
    /// As optimization only score and rank up until a constant value, for faster fuzzy matching.
    /// Tune this value for improvements 1000 <= MAX_SCORING_CANDIDATES <= 5000.
    ///
    /// # Arguments
    /// * `query` - The search string to match against indexed paths.
    /// * `max_results` - The maximum number of results to return.
    ///
    /// # Returns
    /// * A vector of tuples containing matching paths and their relevance scores,
    ///   sorted by score in descending order (best matches first).
    ///
    /// # Example
    /// ```rust
    /// let mut matcher = PathMatcher::new();
    /// matcher.add_path("/home/user/documents/presentation.pptx");
    /// matcher.add_path("/home/user/images/photo.jpg");
    ///
    /// // Search with exact query
    /// let results = matcher.search("presentation", 10);
    /// assert!(!results.is_empty());
    ///
    /// // Search with misspelled query
    /// let fuzzy_results = matcher.search("presentaton", 10); // Missing 'i'
    /// assert!(!fuzzy_results.is_empty());
    /// ```
    ///
    /// # Time Complexity
    /// * Empirically scales as O(n^a) where a ≈ 0.5-0.7 (sub-linear)
    /// * Theoretical: O(Q + S) where:
    ///   - Q = length of query
    ///   - S = number of candidate paths sharing trigrams with query (typically S << N)
    /// * For 10× more paths, search is typically only 3-7× slower
    /// * Significantly faster than Levenshtein (O(N*M²)) or substring matching (O(N*Q))
    pub fn search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        const MAX_SCORING_CANDIDATES: usize = 2000;

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

        if total_hits == 0 {
            return self.fallback_search(query, max_results);
        }

        let mut candidates: Vec<(usize, u16)> = hit_counts
            .iter()
            .enumerate()
            .filter(|&(_idx, &count)| count > 0)
            .map(|(idx, &count)| (idx, count))
            .collect();

        // Sort candidates by hit count descending (most trigrams in common first)
        candidates.sort_unstable_by(|a, b| b.1.cmp(&a.1));

        // Take only the top N candidates to score (significantly reduces work and speed up search)
        let candidates_to_score = candidates
            .into_iter()
            .take(MAX_SCORING_CANDIDATES)
            .collect::<Vec<_>>();

        let mut results = Vec::with_capacity(max_results * 2);
        let query_first_char = query_lower.chars().next();
        let query_trigram_count = query_trigrams.len() as f32;

        for (path_idx, hits) in candidates_to_score {
            let path = &self.paths[path_idx];
            let hits = hits as f32;
            let path_lower = path.to_lowercase();

            let path_components: Vec<&str> = path.split('/').collect();
            let filename = path_components.last().unwrap_or(&"");
            let filename_lower = filename.to_lowercase();
            let mut score = hits / query_trigram_count;

            if filename_lower == query_lower {
                score += 0.5;
            } else if filename_lower.contains(&query_lower) {
                score += 0.3;
            } else if path_lower.contains(&query_lower) {
                score += 0.2;
            }

            if let Some(query_char) = query_first_char {
                if let Some(filename_char) = filename_lower.chars().next() {
                    if query_char == filename_char {
                        score += 0.15;
                    }
                }
            }

            if let Some(dot_pos) = query_lower.find('.') {
                let query_ext = &query_lower[dot_pos..];
                if path_lower.ends_with(query_ext) {
                    score += 0.1;
                }
            }

            if let Some(pos) = path_lower.find(&query_lower) {
                let pos_factor = 1.0 - (pos as f32 / path.len() as f32).min(0.9);
                score += pos_factor * 0.1;
            }

            // Apply path length normalization
            let length_factor = self.calculate_length_normalization(path.len());
            score *= length_factor;

            results.push((path.clone(), score));
        }

        results.sort_unstable_by(|a, b| {
            let cmp = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
            if cmp != std::cmp::Ordering::Equal {
                return cmp;
            }
            a.0.len().cmp(&b.0.len())
        });
        results.truncate(max_results);

        if results.is_empty() && query.len() >= 3 {
            return self.fallback_search(query, max_results);
        }

        results
    }

    /// Extracts trigrams from a query string for searching.
    /// Similar to extract_and_index_trigrams but optimized for search-time use.
    ///
    /// # Arguments
    /// * `query` - The query string to extract trigrams from.
    ///
    /// # Returns
    /// * A vector of u32 values representing the packed trigrams.
    ///
    /// # Implementation Details
    /// The query is padded with spaces and each consecutive 3-character sequence
    /// is converted to lowercase and packed into a u32 value.
    #[inline]
    fn extract_query_trigrams(&self, query: &str) -> Vec<u32> {
        let bytes = query.as_bytes();
        if bytes.len() < 3 {
            // Special case for very short queries
            return Vec::new();
        }

        let mut trigrams = Vec::with_capacity(bytes.len() + 2);
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

    /// Performs an optimized fallback search when the primary search method doesn't yield enough results.
    /// This method generates variations of the query and matches them against the trigram index
    /// to find matches even with significant typos or spelling variations.
    ///
    /// # Arguments
    /// * `query` - The original search query.
    /// * `max_results` - The maximum number of results to return.
    ///
    /// # Returns
    /// * A vector of tuples containing matching paths and their relevance scores.
    ///
    /// # Implementation Details
    /// The fallback search uses the following approach:
    ///
    /// - Generates efficient variations of the query (deletions, transpositions, substitutions)
    /// - Uses trigram matching against these variations for fast candidate identification
    /// - Employs bitmap-based tracking for high-performance path collection
    /// - Applies first-character matching bonuses to prioritize more relevant results
    /// - Applies path length normalization to prevent bias toward longer paths
    /// - Assigns scores based on the variation position (earlier variations get higher scores)
    ///
    /// # Time Complexity
    /// * O(V * (Q + S)) where:
    ///   - V = number of query variations generated (typically 2-3 times query length)
    ///   - Q = length of query
    ///   - S = number of candidate paths per variation
    /// * Still maintains sub-linear scaling relative to total paths N
    /// * Optimized to terminate early once sufficient results are found
    fn fallback_search(&self, query: &str, max_results: usize) -> Vec<(String, f32)> {
        let query_lower = query.to_lowercase();
        let variations = self.generate_efficient_variations(&query_lower);

        // === Step 1: Fast Variation-based Fallback ===
        let mut path_bitmap = vec![0u32; (self.paths.len() + 31) / 32];
        let mut variation_hits =
            FxHashMap::with_capacity_and_hasher(variations.len(), Default::default());
        let mut seen_paths =
            FxHashSet::with_capacity_and_hasher(max_results * 2, Default::default());
        let mut results = Vec::with_capacity(max_results * 2);

        for (variation_idx, variation) in variations.iter().enumerate() {
            let trigrams = self.extract_query_trigrams(variation);
            if trigrams.is_empty() {
                continue;
            }
            for word in &mut path_bitmap {
                *word = 0;
            }
            for &trigram in &trigrams {
                if let Some(path_indices) = self.trigram_index.get(&trigram) {
                    for &path_idx in path_indices {
                        let idx = path_idx as usize;
                        let word_idx = idx / 32;
                        let bit_pos = idx % 32;
                        path_bitmap[word_idx] |= 1 << bit_pos;
                        variation_hits
                            .entry(path_idx)
                            .or_insert_with(|| SmallVec::<[usize; 2]>::with_capacity(2))
                            .push(variation_idx);
                    }
                }
            }
            for word_idx in 0..path_bitmap.len() {
                let mut word = path_bitmap[word_idx];
                while word != 0 {
                    let bit_pos = word.trailing_zeros() as usize;
                    let path_idx = word_idx * 32 + bit_pos;
                    if path_idx < self.paths.len() && !seen_paths.contains(&path_idx) {
                        seen_paths.insert(path_idx);
                        let path = &self.paths[path_idx];
                        let filename = path.split('/').last().unwrap_or(path);
                        let filename_lower = filename.to_lowercase();
                        let variation_index = variation_idx as f32 / variations.len() as f32;
                        let mut score = 0.9 - (variation_index * 0.2);
                        // Bonus for matching first char
                        if !query_lower.is_empty() && !filename_lower.is_empty() {
                            if query_lower.chars().next().unwrap()
                                == filename_lower.chars().next().unwrap()
                            {
                                score += 0.3;
                            }
                        }
                        // Length normalization
                        let length_factor = self.calculate_length_normalization(path.len());
                        score *= length_factor;
                        results.push((path.clone(), score));
                    }
                    word &= !(1 << bit_pos);
                }
            }
            if results.len() >= max_results {
                break;
            }
        }

        // Sort and return top results
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(max_results);
        results
    }

    /// Generates efficient variations of a query string for fuzzy matching.
    /// Creates alternative spellings by applying character deletions, transpositions,
    /// and substitutions based on common typing errors.
    ///
    /// # Arguments
    /// * `query` - The original query string to generate variations for.
    ///
    /// # Returns
    /// * A vector of strings containing variations of the original query.
    ///
    /// # Implementation Details
    /// The number and type of variations generated depends on the query length:
    /// - Deletions: Remove one character at a time
    /// - Transpositions: Swap adjacent characters
    /// - Substitutions: Replace characters with common alternatives (only for short queries)
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
            for i in 0..len - 1 {
                let mut new_chars = chars.clone();
                new_chars.swap(i, i + 1);
                variations.push(new_chars.iter().collect());
            }
        }
        // 3. Only do substitutions for short queries (expensive)
        if len > 1 && len <= 5 {
            static SUBS: &[(char, char)] = &[
                ('a', 'e'),
                ('e', 'a'),
                ('i', 'y'),
                ('o', 'u'),
                ('s', 'z'),
                ('c', 'k'),
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
    use crate::{log_info, log_warn};
    use std::path::PathBuf;
    use std::time::Duration;
    use std::time::Instant;

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
        log_info!(&format!(
            "Benchmark '{}': {:?} per iteration",
            name, duration
        ));

        (result.unwrap(), duration)
    }

    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-fuzzy-search");
        if !path.exists() {
            log_warn!(&format!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            ));
            panic!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            );
        }
        path
    }

    // Helper function to collect real paths from the test data directory
    fn collect_test_paths(limit: Option<usize>) -> Vec<String> {
        let test_path = get_test_data_path();
        let mut paths = Vec::new();

        fn add_paths_recursively(
            dir: &std::path::Path,
            paths: &mut Vec<String>,
            limit: Option<usize>,
        ) {
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
            return (0..100)
                .map(|i| format!("/path/to/file{}.txt", i))
                .collect();
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
    fn test_remove_path() {
        let mut matcher = PathMatcher::new();

        matcher.add_path("/test/path1.txt");
        matcher.add_path("/test/path2.txt");
        matcher.add_path("/test/path3.txt");

        assert_eq!(matcher.paths.len(), 3);

        // Remove a path
        let removed = matcher.remove_path("/test/path2.txt");
        assert!(removed);

        // Check that the path was removed
        assert_eq!(matcher.paths.len(), 2);
        assert_eq!(matcher.paths[0], "/test/path1.txt");
        assert_eq!(matcher.paths[1], "/test/path3.txt");

        // Verify search still works
        let results = matcher.search("path", 10);
        assert_eq!(results.len(), 2);

        // Verify removing a non-existent path returns false
        let removed = matcher.remove_path("/test/nonexistent.txt");
        assert!(!removed);
        assert_eq!(matcher.paths.len(), 2);
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
        let misspelled_results = matcher.search("flie", 10); // 'file' misspelled as 'flie'
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
        let found_exact = results
            .iter()
            .any(|(path, _)| path == "/exact/match/rust-src/file.rs");
        let found_partial = results
            .iter()
            .any(|(path, _)| path == "/partial/rust/src/different.rs");
        let found_rust_src = results
            .iter()
            .any(|(path, _)| path == "/rust_src/something/else.txt");

        assert!(found_exact, "Should find the exact match path");
        assert!(found_partial, "Should find the partial match path");
        assert!(found_rust_src, "Should find the rust_src path");

        // The exact match should be ranked higher than unrelated paths
        let exact_idx = results
            .iter()
            .position(|(path, _)| path == "/exact/match/rust-src/file.rs")
            .unwrap();
        let unrelated_idx_opt = results
            .iter()
            .position(|(path, _)| path == "/unrelated/file.txt");

        if let Some(unrelated_idx) = unrelated_idx_opt {
            assert!(
                exact_idx < unrelated_idx,
                "Exact match should rank higher than unrelated path"
            );
        }

        // Verify scores are in descending order
        for i in 1..results.len() {
            assert!(
                results[i - 1].1 >= results[i].1,
                "Scores should be in descending order: {} >= {}",
                results[i - 1].1,
                results[i].1
            );
        }

        // Test with misspelled query
        let misspelled_results = matcher.search("rsut scr", 10); // 'rust src' misspelled

        // Check that despite misspelling we still get results
        assert!(
            !misspelled_results.is_empty(),
            "Misspelled search should return results"
        );

        // The relevant paths should be present despite misspelling
        let found_exact_misspelled = misspelled_results
            .iter()
            .any(|(path, _)| path == "/exact/match/rust-src/file.rs");
        let found_partial_misspelled = misspelled_results
            .iter()
            .any(|(path, _)| path == "/partial/rust/src/different.rs");
        let found_rust_src_misspelled = misspelled_results
            .iter()
            .any(|(path, _)| path == "/rust_src/something/else.txt");

        assert!(
            found_exact_misspelled || found_partial_misspelled || found_rust_src_misspelled,
            "Should find at least one of the relevant matches despite misspelling"
        );
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
        let misspelled_results = matcher.search("upprecaes", 10); // 'uppercase' misspelled
        assert!(!misspelled_results.is_empty());
        assert!(misspelled_results[0].0.to_lowercase().contains("upper"));

        let misspelled_results_2 = matcher.search("LWORECASE", 10); // 'LOWERCASE' misspelled
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

        let (results, elapsed) =
            run_benchmark("small dataset search", 10, || matcher.search("file", 10));

        assert!(!results.is_empty());
        log_info!(&format!(
            "Small dataset (100 items) search took: {:.2?}",
            elapsed
        ));
    }

    #[test]
    fn test_search_performance_medium() {
        let mut matcher = PathMatcher::new();

        // Add a medium number of paths
        for i in 0..1000 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        let (results, elapsed) =
            run_benchmark("medium dataset search", 10, || matcher.search("file", 10));

        assert!(!results.is_empty());
        log_info!(&format!(
            "Medium dataset (1,000 items) search took: {:.2?}",
            elapsed
        ));
    }

    #[test]
    fn test_search_performance_large() {
        let mut matcher = PathMatcher::new();

        // Add a large number of paths
        for i in 0..10000 {
            matcher.add_path(&format!("/path/to/file{}.txt", i));
        }

        let (results, elapsed) =
            run_benchmark("large dataset search", 5, || matcher.search("file", 10));

        assert!(!results.is_empty());
        log_info!(&format!(
            "Large dataset (10,000 items) search took: {:.2?}",
            elapsed
        ));
    }

    #[test]
    fn benchmark_indexing_speed() {
        let iterations = 5;

        // Get real test paths instead of synthetic ones
        let small_paths = collect_test_paths(Some(100));
        let medium_paths = collect_test_paths(Some(1000));
        let large_paths = collect_test_paths(Some(10000));

        log_info!(&format!(
            "Benchmarking with {} small paths, {} medium paths, and {} large paths",
            small_paths.len(),
            medium_paths.len(),
            large_paths.len()
        ));

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
        log_info!(&format!(
            "  Small ({} paths): {:?}",
            small_paths.len(),
            small_duration
        ));
        log_info!(&format!(
            "  Medium ({} paths): {:?}",
            medium_paths.len(),
            medium_duration
        ));
        log_info!(&format!(
            "  Large ({} paths): {:?}",
            large_paths.len(),
            large_duration
        ));

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

        log_info!(&format!(
            "Benchmarking query performance with {} real paths",
            test_paths.len()
        ));

        for path in &test_paths {
            matcher.add_path(path);
        }

        // Test queries of different lengths and complexity
        let queries = [
            "f",           // Single character
            "fi",          // Two characters
            "file",        // Common term
            "banana",      // Common term in real data
            "nonexistent", // No matches
            "flie",        // Misspelled
            "bannana",     // Misspelled real term
        ];

        log_info!(&format!("Query performance benchmark:"));

        for query in &queries {
            let (results, duration) = run_benchmark(&format!("query '{}'", query), 10, || {
                matcher.search(query, 10)
            });

            log_info!(&format!(
                "  Query '{}' took {:?} and found {} results",
                query,
                duration,
                results.len()
            ));
        }
    }

    #[test]
    fn benchmark_comparison_with_alternatives() {
        // Use real test data instead of synthetic paths
        let test_paths = collect_test_paths(Some(1000));
        let mut matcher = PathMatcher::new();

        log_info!(&format!(
            "Benchmarking search methods with {} real paths",
            test_paths.len()
        ));

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

        log_info!(&format!(
            "Using search term '{}' derived from real data",
            search_term
        ));

        // Benchmark our implementation
        let (our_results, our_duration) =
            run_benchmark("our fuzzy search", 20, || matcher.search(search_term, 10));

        // Benchmark simple substring search
        let (substr_results, substr_duration) = run_benchmark("substring search", 20, || {
            let query = search_term.to_lowercase();
            test_paths
                .iter()
                .filter(|path| path.to_lowercase().contains(&query))
                .map(|path| (path.clone(), 1.0))
                .take(10)
                .collect::<Vec<(String, f32)>>()
        });

        // Benchmark regex search
        let (regex_results, regex_duration) = run_benchmark("regex search", 20, || {
            let regex_pattern = format!("(?i){}", regex::escape(search_term));
            match regex::Regex::new(&regex_pattern) {
                Ok(re) => test_paths
                    .iter()
                    .filter(|path| re.is_match(path))
                    .map(|path| (path.clone(), 1.0))
                    .take(10)
                    .collect::<Vec<(String, f32)>>(),
                Err(_) => Vec::new(),
            }
        });

        log_info!(&format!("Search method comparison:"));
        log_info!(&format!(
            "  Our fuzzy search: {:?} with {} results",
            our_duration,
            our_results.len()
        ));
        log_info!(&format!(
            "  Substring search: {:?} with {} results",
            substr_duration,
            substr_results.len()
        ));
        log_info!(&format!(
            "  Regex search: {:?} with {} results",
            regex_duration,
            regex_results.len()
        ));

        let fuzzy_vs_substr = substr_duration.as_nanos() as f64 / our_duration.as_nanos() as f64;
        let fuzzy_vs_regex = regex_duration.as_nanos() as f64 / our_duration.as_nanos() as f64;

        log_info!(&format!(
            "  Our fuzzy search is {:.2}x {} than substring search",
            fuzzy_vs_substr.abs(),
            if fuzzy_vs_substr > 1.0 {
                "faster"
            } else {
                "slower"
            }
        ));
        log_info!(&format!(
            "  Our fuzzy search is {:.2}x {} than regex search",
            fuzzy_vs_regex.abs(),
            if fuzzy_vs_regex > 1.0 {
                "faster"
            } else {
                "slower"
            }
        ));
    }

    #[test]
    fn test_different_query_lengths() {
        // Use real test data
        let test_paths = collect_test_paths(Some(1000));
        let mut matcher = PathMatcher::new();

        log_info!(&format!(
            "Testing query lengths with {} real paths",
            test_paths.len()
        ));

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
                vec![
                    "f".to_string(),
                    "fi".to_string(),
                    "fil".to_string(),
                    "file".to_string(),
                    "file.".to_string(),
                    "file.t".to_string(),
                ]
            } else {
                terms
            }
        } else {
            vec![
                "f".to_string(),
                "fi".to_string(),
                "fil".to_string(),
                "file".to_string(),
                "file.".to_string(),
                "file.t".to_string(),
            ]
        };

        // Test different query lengths with real terms
        for query in &realistic_terms {
            let (results, elapsed) =
                run_benchmark(&format!("query length '{}'", query.len()), 5, || {
                    matcher.search(query, 10)
                });

            log_info!(&format!(
                "Query '{}' (length {}) took {:.2?} with {} results",
                query,
                query.len(),
                elapsed,
                results.len()
            ));

            assert!(!results.is_empty() || query.len() < 3);
        }

        // Test different query lengths with misspellings of real terms
        let misspelled_terms: Vec<String> = realistic_terms
            .iter()
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
            let (results, elapsed) =
                run_benchmark(&format!("misspelled query '{}'", query), 5, || {
                    matcher.search(query, 10)
                });

            log_info!(&format!(
                "Misspelled query '{}' (length {}) took {:.2?} with {} results",
                query,
                query.len(),
                elapsed,
                results.len()
            ));

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
            ("presentation", "persentaton"), // multiple errors
            ("beach", "beech"),              // single vowel error
            ("favorite", "favorit"),         // missing letter
            ("music", "musik"),              // phonetic error
            ("project", "progect"),          // single consonant error
            ("images", "imaegs"),            // transposed letters
            ("vacation", "vacasion"),        // phonetic substitution
            ("documents", "dokumentz"),      // multiple substitutions
            ("code", "kode"),                // spelling variation
        ];

        for (correct, misspelled) in &test_cases {
            // Search using the misspelled query
            let results = matcher.search(misspelled, 10);

            // Log the search results for debugging
            log_info!(&format!(
                "Search for misspelled '{}' (should match '{}') returned {} results",
                misspelled,
                correct,
                results.len()
            ));

            // Verify we have some results
            assert!(
                !results.is_empty(),
                "Misspelled query '{}' should find results",
                misspelled
            );

            // Check if the result contains the path with the correct spelling
            let expected_path = results
                .iter()
                .find(|(path, _)| path.to_lowercase().contains(&correct.to_lowercase()));

            assert!(
                expected_path.is_some(),
                "Misspelled query '{}' should have found a path containing '{}'",
                misspelled,
                correct
            );

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
                                if let Some(sub_path_str) =
                                    subentry.path().to_str().map(|s| s.to_string())
                                {
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

            log_info!(&format!(
                "Search for '{}' found {} results in {:.2?}",
                term,
                results.len(),
                elapsed
            ));

            // Print top 3 results (if any)
            for (i, (path, score)) in results.iter().take(3).enumerate() {
                log_info!(&format!(
                    "  Result #{}: {} (score: {:.4})",
                    i + 1,
                    path,
                    score
                ));
            }
        }

        // Test searching with some realistic terms including misspellings
        let search_terms = ["bananna", "txt", "mp3", "aple"]; // misspelled banana and apple

        for term in &search_terms {
            let start = Instant::now();
            let results = matcher.search(term, 20);
            let elapsed = start.elapsed();

            log_info!(&format!(
                "Search for misspelled '{}' found {} results in {:.2?}",
                term,
                results.len(),
                elapsed
            ));

            // Print top 3 results (if any)
            for (i, (path, score)) in results.iter().take(3).enumerate() {
                log_info!(&format!(
                    "  Result #{}: {} (score: {:.4})",
                    i + 1,
                    path,
                    score
                ));
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
            let substring_results: Vec<(String, f32)> = all_paths
                .iter()
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
    #[cfg(feature = "long-tests")]
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

        log_info!(&format!(
            "Indexed {} paths in {:.2?}",
            path_count, indexing_time
        ));

        // Test search performance with a variety of terms
        let query_terms = ["file", "banana", "txt", "mp3", "orange", "apple", "e"];

        for term in &query_terms {
            let search_start = Instant::now();
            let results = matcher.search(term, 50);
            let search_time = search_start.elapsed();

            log_info!(&format!(
                "Search for '{}' found {} results in {:.2?}",
                term,
                results.len(),
                search_time
            ));
        }
    }

    // Test to create test data directory if it doesn't exist
    #[test]
    #[cfg(feature = "generate-test-data")] // Only run when needed to generate test data
    fn create_test_data() {
        let base_path = PathBuf::from("./test-data-for-fuzzy-search");
        match crate::search_engine::test_generate_test_data::generate_test_data(base_path) {
            Ok(path) => log_info!(&format!("Test data generated successfully at {:?}", path)),
            Err(e) => panic!("Failed to generate test data: {}", e),
        }
    }

    #[cfg(feature = "long-tests")]
    #[test]
    fn benchmark_search_with_all_paths_path_matcher() {
        log_info!("Benchmarking PathMatcher with thousands of real-world paths");

        // 1. Collect all available paths
        let paths = collect_test_paths(None); // Get all available paths
        let path_count = paths.len();

        log_info!(&format!("Collected {} test paths", path_count));

        // Store all the original paths for verification
        let all_paths = paths.clone();

        // Helper function to generate guaranteed-to-match queries
        fn extract_guaranteed_queries(paths: &[String], limit: usize) -> Vec<String> {
            let mut queries = Vec::new();
            let mut seen_queries = std::collections::HashSet::new();

            // Helper function to add unique queries
            fn should_add_query(query: &str, seen: &mut std::collections::HashSet<String>) -> bool {
                let normalized = query.trim_end_matches('/').to_string();
                if !normalized.is_empty() && !seen.contains(&normalized) {
                    seen.insert(normalized);
                    return true;
                }
                false
            }

            if paths.is_empty() {
                return queries;
            }

            // a. Extract directory prefixes from actual paths
            for path in paths.iter().take(paths.len().min(100)) {
                let components: Vec<&str> = path.split(|c| c == '/' || c == '\\').collect();

                // Full path prefixes
                for i in 1..components.len() {
                    if queries.len() >= limit {
                        break;
                    }

                    let prefix = components[0..i].join("/");
                    if !prefix.is_empty() {
                        // Check and add the base prefix
                        if should_add_query(&prefix, &mut seen_queries) {
                            queries.push(prefix.clone());
                        }
                    }

                    if queries.len() >= limit {
                        break;
                    }
                }

                // b. Extract filename prefixes (for partial filename matches)
                if queries.len() < limit {
                    if let Some(last) = components.last() {
                        if !last.is_empty() && last.len() > 2 {
                            let first_chars = &last[..last.len().min(2)];
                            if !first_chars.is_empty() {
                                if should_add_query(first_chars, &mut seen_queries) {
                                    queries.push(first_chars.to_string());
                                }
                            }
                        }
                    }
                }
            }

            // c. Add specific test cases for fuzzy search patterns
            if queries.len() < limit {
                if paths
                    .iter()
                    .any(|p| p.contains("test-data-for-fuzzy-search"))
                {
                    // Add queries with various spelling patterns
                    let test_queries = [
                        "apple".to_string(),   // Common term in test data
                        "aple".to_string(),    // Misspelled
                        "bannana".to_string(), // Common with misspelling
                        "txt".to_string(),     // Common extension
                        "orangge".to_string(), // Common with misspelling
                    ];

                    for query in test_queries {
                        if queries.len() >= limit {
                            break;
                        }
                        if should_add_query(&query, &mut seen_queries) {
                            queries.push(query);
                        }
                    }

                    // Extract some specific filenames from test data
                    if queries.len() < limit {
                        for path in paths.iter() {
                            if queries.len() >= limit {
                                break;
                            }
                            if let Some(filename) = path.split('/').last() {
                                if filename.len() > 3 {
                                    let query = filename[..filename.len().min(4)].to_string();
                                    if should_add_query(&query, &mut seen_queries) {
                                        queries.push(query);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add basic queries if needed
            if queries.len() < 3 {
                let basic_queries = ["file".to_string(), "doc".to_string(), "img".to_string()];

                for query in basic_queries {
                    if should_add_query(&query, &mut seen_queries) {
                        queries.push(query);
                    }
                }
            }

            // Limit the number of queries
            if queries.len() > limit {
                queries.truncate(limit);
            }

            queries
        }

        // 2. Test with different batch sizes
        let batch_sizes = [10, 100, 1000, 10000, all_paths.len()];

        for &batch_size in &batch_sizes {
            // Reset for this batch size
            let subset_size = batch_size.min(all_paths.len());

            // Create a fresh engine with only the needed paths
            let mut subset_matcher = PathMatcher::new();
            let start_insert_subset = std::time::Instant::now();

            for i in 0..subset_size {
                subset_matcher.add_path(&all_paths[i]);
            }

            let subset_insert_time = start_insert_subset.elapsed();
            log_info!(&format!("\n=== BENCHMARK WITH {} PATHS ===", subset_size));
            log_info!(&format!(
                "Subset insertion time: {:?} ({:.2} paths/ms)",
                subset_insert_time,
                subset_size as f64 / subset_insert_time.as_millis().max(1) as f64
            ));

            // Generate test queries specifically for this subset
            let subset_paths = all_paths
                .iter()
                .take(subset_size)
                .cloned()
                .collect::<Vec<_>>();
            let subset_queries = extract_guaranteed_queries(&subset_paths, 15);

            log_info!(&format!(
                "Generated {} subset-specific queries",
                subset_queries.len()
            ));

            // Run a single warmup search to prime any caches
            subset_matcher.search("file", 10);

            // Run measurements on each test query
            let mut total_time = std::time::Duration::new(0, 0);
            let mut total_results = 0;
            let mut times = Vec::new();
            let mut fuzzy_counts = 0;

            for query in &subset_queries {
                // Measure search time
                let start = std::time::Instant::now();
                let completions = subset_matcher.search(query, 20);
                let elapsed = start.elapsed();

                total_time += elapsed;
                total_results += completions.len();
                times.push((query.clone(), elapsed, completions.len()));

                // Count fuzzy matches (any match not containing the exact query)
                let fuzzy_matches = completions
                    .iter()
                    .filter(|(path, _)| !path.to_lowercase().contains(&query.to_lowercase()))
                    .count();
                fuzzy_counts += fuzzy_matches;

                // Print top results for each search
                //log_info!(&format!("Results for '{}' (found {})", query, completions.len()));
                //for (i, (path, score)) in completions.iter().take(3).enumerate() {
                //    log_info!(&format!("    #{}: '{}' (score: {:.3})", i+1, path, score));
                //}
                //if completions.len() > 3 {
                //    log_info!(&format!("    ... and {} more results", completions.len() - 3));
                //}
            }

            // Calculate and report statistics
            let avg_time = if !subset_queries.is_empty() {
                total_time / subset_queries.len() as u32
            } else {
                std::time::Duration::new(0, 0)
            };

            let avg_results = if !subset_queries.is_empty() {
                total_results / subset_queries.len()
            } else {
                0
            };

            let avg_fuzzy = if !subset_queries.is_empty() {
                fuzzy_counts as f64 / subset_queries.len() as f64
            } else {
                0.0
            };

            log_info!(&format!("Ran {} searches", subset_queries.len()));
            log_info!(&format!("Average search time: {:?}", avg_time));
            log_info!(&format!("Average results per search: {}", avg_results));
            log_info!(&format!(
                "Average fuzzy matches per search: {:.1}",
                avg_fuzzy
            ));

            // Sort searches by time and log
            times.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by time, slowest first

            // Log the slowest searches
            log_info!("Slowest searches:");
            for (i, (query, time, count)) in times.iter().take(3).enumerate() {
                log_info!(&format!(
                    "  #{}: '{:40}' - {:?} ({} results)",
                    i + 1,
                    query,
                    time,
                    count
                ));
            }

            // Log the fastest searches
            log_info!("Fastest searches:");
            for (i, (query, time, count)) in times.iter().rev().take(3).enumerate() {
                log_info!(&format!(
                    "  #{}: '{:40}' - {:?} ({} results)",
                    i + 1,
                    query,
                    time,
                    count
                ));
            }

            // Test with different result counts
            let mut by_result_count = Vec::new();
            for &count in &[0, 1, 5, 10] {
                let matching: Vec<_> = times.iter().filter(|(_, _, c)| *c >= count).collect();

                if !matching.is_empty() {
                    let total = matching
                        .iter()
                        .fold(std::time::Duration::new(0, 0), |sum, (_, time, _)| {
                            sum + *time
                        });
                    let avg = total / matching.len() as u32;

                    by_result_count.push((count, avg, matching.len()));
                }
            }

            log_info!("Average search times by result count:");
            for (count, avg_time, num_searches) in by_result_count {
                log_info!(&format!(
                    "  ≥ {:3} results: {:?} (from {} searches)",
                    count, avg_time, num_searches
                ));
            }

            // Special test: Character edits for fuzzy matching
            if !subset_queries.is_empty() {
                let mut misspelled_queries = Vec::new();

                // Create misspelled versions of existing queries
                for query in subset_queries.iter().take(3) {
                    if query.len() >= 3 {
                        // Character deletion
                        let deletion = format!("{}{}", &query[..1], &query[2..]);
                        misspelled_queries.push(deletion);

                        // Character transposition (if possible)
                        if query.len() >= 4 {
                            let mut chars: Vec<char> = query.chars().collect();
                            chars.swap(1, 2);
                            misspelled_queries.push(chars.iter().collect::<String>());
                        }

                        // Character substitution
                        let substitution = if query.contains('a') {
                            query.replacen('a', "e", 1)
                        } else if query.contains('e') {
                            query.replacen('e', "a", 1)
                        } else {
                            format!("{}x{}", &query[..1], &query[2..])
                        };
                        misspelled_queries.push(substitution);
                    }
                }

                log_info!(&format!(
                    "Testing {} misspelled variations",
                    misspelled_queries.len()
                ));

                for misspelled in &misspelled_queries {
                    let start = std::time::Instant::now();
                    let results = subset_matcher.search(misspelled, 10);
                    let elapsed = start.elapsed();

                    log_info!(&format!(
                        "Misspelled '{}' found {} results in {:?}",
                        misspelled,
                        results.len(),
                        elapsed
                    ));

                    if !results.is_empty() {
                        log_info!(&format!(
                            "  Top result: {} (score: {:.3})",
                            results[0].0, results[0].1
                        ));
                    }
                }
            }
        }
    }
}
