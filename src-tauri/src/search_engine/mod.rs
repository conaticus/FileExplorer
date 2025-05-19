mod models;
mod fast_fuzzy_v2;
mod lru_cache_v2;
mod path_cache_wrapper;
pub mod autocomplete_engine;
mod art_v4;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::fs::read_dir;
use crate::search_engine::models::directory_se::DirectorySe;
use models::file_se::FileSe;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Entry {
    FILE(FileSe),
    DIRECTORY(DirectorySe),
}

// New parallel indexing function using Rayon
fn index_given_path_parallel(path: PathBuf) -> Vec<Entry> {
    let mut result: Vec<Entry> = Vec::new();

    // Return empty vector if we can't read the directory
    let read_dir_result = match read_dir(&path) {
        Ok(dir) => dir,
        Err(_) => return result,
    };

    // Collect entries first to enable parallel processing
    let entries: Vec<_> = read_dir_result.filter_map(Result::ok).collect();

    // Process entries in parallel
    let entries_result: Vec<Option<Entry>> = entries
        .par_iter()
        .map(|entry| {
            let metadata = match entry.metadata() {
                Ok(meta) => meta,
                Err(_) => return None,
            };

            if metadata.is_file() {
                match FileSe::from_dir_entry(entry) {
                    Ok(file) => Some(Entry::FILE(file)),
                    Err(_) => None,
                }
            } else if metadata.is_dir() {
                match DirectorySe::from_dir_entry(entry) {
                    Ok(dir) => Some(Entry::DIRECTORY(dir)),
                    Err(_) => None,
                }
            } else {
                None
            }
        })
        .collect();

    // Filter out None values and collect results
    result.extend(entries_result.into_iter().filter_map(|e| e));

    result
}

#[allow(dead_code)]
pub fn search_by_filename(search_term: &str, source: Vec<Entry>) -> Vec<Entry> {
    let mut results = Vec::new();
    let search_term = search_term.to_lowercase();

    // Create a search cache for improved performance on repeated searches
    let search_cache = Arc::new(Mutex::new(HashMap::new()));

    for entry in source {
        match &entry {
            Entry::FILE(file) => {
                if efficient_string_match(&file.name, &search_term, &search_cache) {
                    results.push(entry);
                }
            }
            Entry::DIRECTORY(dir) => {
                // Include directory itself if it matches
                if efficient_string_match(&dir.name, &search_term, &search_cache) {
                    results.push(entry.clone());
                }

                // Recursively search inside this directory
                let dir_path = PathBuf::from(&dir.path);
                let subdirectory_entries = index_given_path_parallel(dir_path);
                let subdirectory_matches = search_by_filename(&search_term, subdirectory_entries);
                results.extend(subdirectory_matches);
            }
        }
    }

    results
}


#[allow(dead_code)]
// New parallel search function
pub fn search_by_filename_parallel(search_term: &str, source: Vec<Entry>) -> Vec<Entry> {
    let search_term = search_term.to_lowercase();
    let search_cache = Arc::new(Mutex::new(HashMap::new()));

    let results: Vec<Entry> = source
        .par_iter()
        .flat_map(|entry| {
            let mut local_results = Vec::new();

            match entry {
                Entry::FILE(file) => {
                    if efficient_string_match(&file.name, &search_term, &search_cache) {
                        local_results.push(entry.clone());
                    }
                }
                Entry::DIRECTORY(dir) => {
                    // Include directory itself if it matches
                    if efficient_string_match(&dir.name, &search_term, &search_cache) {
                        local_results.push(entry.clone());
                    }

                    // Search subdirectories in parallel
                    let dir_path = PathBuf::from(&dir.path);
                    let subdirectory_entries = index_given_path_parallel(dir_path);
                    let subdirectory_matches =
                        search_by_filename_parallel(&search_term, subdirectory_entries);
                    local_results.extend(subdirectory_matches);
                }
            }

            local_results
        })
        .collect();

    results
}

// Improved string matching with caching
fn efficient_string_match(
    haystack: &str,
    needle: &str,
    cache: &Arc<Mutex<HashMap<String, bool>>>,
) -> bool {
    // Create a cache key
    let cache_key = format!("{}:{}", haystack, needle);

    // Check if result is in cache
    if let Ok(cache_ref) = cache.lock() {
        if let Some(&result) = cache_ref.get(&cache_key) {
            return result;
        }
    }

    // If not in cache, compute the result
    let haystack_lower = haystack.to_lowercase();
    let result = haystack_lower.contains(needle);

    // Store result in cache
    if let Ok(mut cache_ref) = cache.lock() {
        cache_ref.insert(cache_key, result);
    }

    result
}


#[allow(dead_code)]
/// Generates a test data directory structure with random folder and file names.
/// Creates 20 folders with 20 subfolders each up to a depth of 5, and 20 files in each folder.
pub fn generate_test_data(base_path: PathBuf) -> Result<PathBuf, std::io::Error> {
    use rand::{thread_rng, Rng};
    use std::fs::{create_dir_all, File};
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
        let charset: Vec<&str> = "banana,apple,orange,grape,watermelon,kiwi,mango,peach,cherry,\
        strawberry,blueberry,raspberry,blackberry,lemon,lime,coconut,papaya,pineapple,tangerine,\
        car,truck,motorcycle,bicycle,bus,train,airplane,helicopter,boat,ship,submarine,scooter,van,\
        ambulance,taxi,firetruck,tractor,yacht,jetski,speedboat,racecar".split(",").collect::<Vec<_>>();

        let mut rng = thread_rng();

        let idx = rng.gen_range(0, charset.len());
        return charset[idx].to_string();
    };

    // Function to create file extensions
    let generate_extension = || -> &str {
        const EXTENSIONS: [&str; 20] = [
            "txt", "pdf", "doc", "jpg", "png", "mp3", "mp4", "html", "css", "js", "rs", "json",
            "xml", "md", "csv", "zip", "exe", "dll", "sh", "py",
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

#[allow(dead_code)]
fn index_given_path(path: PathBuf) -> Vec<Entry> {
    let mut result: Vec<Entry> = Vec::new();
    
    // Return empty vector if we can't read the directory
    let read_dir_result = match read_dir(&path) {
        Ok(dir) => dir,
        Err(_) => return result,
    };
    
    // Process each entry in the directory
    for entry_result in read_dir_result {
        // Skip entries that we can't access
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        
        // Skip entries that we can't get metadata for
        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        
        if metadata.is_file() {
            // Process files
            match FileSe::from_dir_entry(&entry) {
                Ok(file) => result.push(Entry::FILE(file)),
                Err(_) => continue,
            }
        } else if metadata.is_dir() {
            // Process directories
            match DirectorySe::from_dir_entry(&entry) {
                Ok(dir) => result.push(Entry::DIRECTORY(dir)),
                Err(_) => continue,
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests_p {
    use super::*;

    // Helper function to get the test data path and verify it exists
    fn get_test_data_path() -> PathBuf {
        let path = PathBuf::from("./test-data-for-search-engine");
        if !path.exists() {
            panic!(
                "Test data directory does not exist: {:?}. Run the 'create_test_data' test first.",
                path
            );
        }
        path
    }

    #[test]
    #[ignore]
    fn start_indexing_home_dir_test() {
        let test_path = get_test_data_path();
        let entries = index_given_path_parallel(test_path);
        assert!(
            !entries.is_empty(),
            "Test directory should contain at least some entries"
        );

        //write the result to a file as json
        let json_result = serde_json::to_string_pretty(&entries).unwrap();
        let path = PathBuf::from("test_dir_index.json");
        std::fs::write(&path, json_result).expect("Unable to write file");
        println!("Test directory indexed and saved to {:?}", path);
    }

    #[test]
    #[ignore]
    fn search_performance_test() {
        // Index the test directory instead of home
        let test_path = get_test_data_path();
        let entries = index_given_path_parallel(test_path);
        assert!(
            !entries.is_empty(),
            "Test directory should contain at least some entries"
        );

        // Measure the time it takes to search for files with 'e' - sequential
        let start_time = std::time::Instant::now();
        let search_results = search_by_filename("e", entries.clone());
        let seq_duration = start_time.elapsed();

        // Measure the time it takes to search for files with 'e' - parallel
        let start_time = std::time::Instant::now();
        let parallel_search_results = search_by_filename_parallel("e", entries);
        let par_duration = start_time.elapsed();

        // Log the timing results
        println!(
            "Sequential search for files with 'e' took: {:?}",
            seq_duration
        );
        println!(
            "Parallel search for files with 'e' took: {:?}",
            par_duration
        );
        println!("Sequential found {} matching entries", search_results.len());
        println!(
            "Parallel found {} matching entries",
            parallel_search_results.len()
        );

        //write search results to a file as json
        let json_result = serde_json::to_string_pretty(&search_results).unwrap();
        let path = PathBuf::from("search_results.json");
        std::fs::write(&path, json_result).expect("Unable to write file");
        println!("Search results saved to {:?}", path);

        // Ensure we found at least some matches (very likely with letter 'e')
        assert!(
            !search_results.is_empty(),
            "Search should have found at least some entries with 'e'"
        );
    }

    #[test]
    #[ignore]
    fn parallel_vs_sequential_indexing_test() {
        let test_path = get_test_data_path();

        // Test sequential indexing
        let start_time = std::time::Instant::now();
        let seq_results = index_given_path(test_path.clone());
        let seq_duration = start_time.elapsed();

        // Test parallel indexing
        let start_time = std::time::Instant::now();
        let par_results = index_given_path_parallel(test_path);
        let par_duration = start_time.elapsed();

        println!("Sequential indexing took: {:?}", seq_duration);
        println!("Parallel indexing took: {:?}", par_duration);
        println!("Sequential found {} entries", seq_results.len());
        println!("Parallel found {} entries", par_results.len());
    }

    #[test]
    #[ignore]
    fn combined_parallel_indexing_and_searching_test() {
        // Measure parallel indexing performance
        let test_path = get_test_data_path();
        let start_time = std::time::Instant::now();
        let entries = index_given_path_parallel(test_path);
        let indexing_duration = start_time.elapsed();

        // Ensure we have entries to search through
        assert!(
            !entries.is_empty(),
            "Test directory should contain at least some entries"
        );
        println!("Parallel indexing took: {:?}", indexing_duration);

        // Common search terms to test
        let search_terms = ["doc", "image", "config", "e"];

        for term in search_terms {
            // Measure parallel search performance
            let search_start_time = std::time::Instant::now();
            let search_results = search_by_filename_parallel(term, entries.clone());
            let search_duration = search_start_time.elapsed();

            println!(
                "Parallel search for '{}' found {} matches in {:?}",
                term,
                search_results.len(),
                search_duration
            );
        }
    }

    #[test]
    #[ignore]
    fn sequential_vs_parallel_search_comparison_test() {
        // First index the test directory in parallel
        println!("Starting parallel indexing of test directory...");
        let test_path = get_test_data_path();
        let start_time = std::time::Instant::now();
        let entries = index_given_path_parallel(test_path);
        let indexing_duration = start_time.elapsed();

        println!(
            "Parallel indexing completed in {:?}, found {} entries",
            indexing_duration,
            entries.len()
        );
        assert!(
            !entries.is_empty(),
            "Test directory should contain at least some entries"
        );

        // Define a variety of search keywords with different specificity
        let search_keywords = ["banana", "blackberry", "peach"];

        println!(
            "\n{:<10} | {:<15} | {:<15} | {:<15} | {:<10}",
            "Keyword", "Sequential Time", "Parallel Time", "Speed Improvement", "Match Count"
        );
        println!("{:-<75}", "");

        // For each keyword, compare sequential vs parallel search
        for keyword in search_keywords {
            // Sequential search
            let seq_start = std::time::Instant::now();
            let seq_results = search_by_filename(keyword, entries.clone());
            let seq_duration = seq_start.elapsed();

            // Parallel search
            let _par_start = std::time::Instant::now();
            let par_results = search_by_filename_parallel(keyword, entries.clone());
            let par_duration = start_time.elapsed();

            // Calculate improvement ratio
            let improvement = if par_duration.as_micros() > 0 {
                seq_duration.as_micros() as f64 / par_duration.as_micros() as f64
            } else {
                f64::INFINITY
            };

            // Verify result counts match
            assert_eq!(
                seq_results.len(),
                par_results.len(),
                "Sequential and parallel search should return the same number of results"
            );

            // Print results in table format
            println!(
                "{:<10} | {:>15?} | {:>15?} | {:>15.2}x | {:>10}",
                keyword,
                seq_duration,
                par_duration,
                improvement,
                seq_results.len()
            );
        }

        // Save detailed results to a file for the most interesting keyword
        let detailed_keyword = search_keywords[0]; // First keyword
        let detailed_results = search_by_filename_parallel(detailed_keyword, entries);
        if !detailed_results.is_empty() {
            let json_result = serde_json::to_string_pretty(&detailed_results).unwrap();
            let path = PathBuf::from(format!("search_results_{}.json", detailed_keyword));
            std::fs::write(&path, json_result).expect("Unable to write file");
            println!(
                "\nDetailed search results for '{}' saved to {:?}",
                detailed_keyword, path
            );
        }
    }

    #[test]
    #[ignore]
    fn test_generate_test_data() {
        match generate_test_data(get_test_data_path()) {
            Ok(path) => {
                assert!(path.exists(), "Test data directory should exist");

                // Test sequential vs parallel indexing of the test data
                let start = std::time::Instant::now();
                let seq_entries = index_given_path(path.clone());
                let seq_duration = start.elapsed();

                let start = std::time::Instant::now();
                let par_entries = index_given_path_parallel(path.clone());
                let par_duration = start.elapsed();

                println!("Test data indexed:");
                println!(
                    "  - Sequential: {} entries in {:?}",
                    seq_entries.len(),
                    seq_duration
                );
                println!(
                    "  - Parallel: {} entries in {:?}",
                    par_entries.len(),
                    par_duration
                );

                assert!(!seq_entries.is_empty(), "Should have indexed some entries");
                assert_eq!(
                    seq_entries.len(),
                    par_entries.len(),
                    "Sequential and parallel indexing should find the same number of entries"
                );
            }
            Err(e) => {
                panic!("Failed to generate test data: {}", e);
            }
        }
    }

    //just create the test data
    #[test]
    #[cfg(feature = "generate-test-data")]
    fn create_test_data() {
        match generate_test_data(get_test_data_path()) {
            Ok(path) => println!("Test data created at: {:?}", path),
            Err(e) => panic!("Failed to generate test data: {}", e),
        }
    }
}
