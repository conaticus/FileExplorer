mod models;

use std::path::PathBuf;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::read_dir;
use models::file_se::FileSe;
use crate::search_engine::models::directory_se::DirectorySe;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Entry {
    FILE(FileSe),
    DIRECTORY(DirectorySe),
}

pub fn start_indexing_home_dir() -> Vec<Entry> {
    let home_dir = home_dir().unwrap_or_default();
    index_given_path_parallel(home_dir)
}

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
    let entries_result: Vec<Option<Entry>> = entries.par_iter()
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
            },
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

// New parallel search function
pub fn search_by_filename_parallel(search_term: &str, source: Vec<Entry>) -> Vec<Entry> {
    let search_term = search_term.to_lowercase();
    let search_cache = Arc::new(Mutex::new(HashMap::new()));
    
    let results: Vec<Entry> = source.par_iter()
        .flat_map(|entry| {
            let mut local_results = Vec::new();
            
            match entry {
                Entry::FILE(file) => {
                    if efficient_string_match(&file.name, &search_term, &search_cache) {
                        local_results.push(entry.clone());
                    }
                },
                Entry::DIRECTORY(dir) => {
                    // Include directory itself if it matches
                    if efficient_string_match(&dir.name, &search_term, &search_cache) {
                        local_results.push(entry.clone());
                    }
                    
                    // Search subdirectories in parallel
                    let dir_path = PathBuf::from(&dir.path);
                    let subdirectory_entries = index_given_path_parallel(dir_path);
                    let subdirectory_matches = search_by_filename_parallel(&search_term, subdirectory_entries);
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
    cache: &Arc<Mutex<HashMap<String, bool>>>
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn start_indexing_home_dir_test() {
        let entries = start_indexing_home_dir();
        assert!(!entries.is_empty(), "Home directory should contain at least some entries");
        
        let result = start_indexing_home_dir();
        
        //write the result to a file as json 
        let json_result = serde_json::to_string_pretty(&result).unwrap();
        let path = PathBuf::from("home_dir_index.json");
        std::fs::write(&path, json_result).expect("Unable to write file");
        println!("Home directory indexed and saved to {:?}", path);
    }
    
    #[test]
    fn search_performance_test() {
        // First get all entries from the home directory
        let entries = start_indexing_home_dir();
        assert!(!entries.is_empty(), "Home directory should contain at least some entries");
        
        // Measure the time it takes to search for files with 'e' - sequential
        let start_time = std::time::Instant::now();
        let search_results = search_by_filename("e", entries.clone());
        let seq_duration = start_time.elapsed();
        
        // Measure the time it takes to search for files with 'e' - parallel
        let start_time = std::time::Instant::now();
        let parallel_search_results = search_by_filename_parallel("e", entries);
        let par_duration = start_time.elapsed();
        
        // Log the timing results
        println!("Sequential search for files with 'e' took: {:?}", seq_duration);
        println!("Parallel search for files with 'e' took: {:?}", par_duration);
        println!("Sequential found {} matching entries", search_results.len());
        println!("Parallel found {} matching entries", parallel_search_results.len());
        
        //write search results to a file as json
        let json_result = serde_json::to_string_pretty(&search_results).unwrap();
        let path = PathBuf::from("search_results.json");
        std::fs::write(&path, json_result).expect("Unable to write file");
        println!("Search results saved to {:?}", path);
        
        // Ensure we found at least some matches (very likely with letter 'e')
        assert!(!search_results.is_empty(), "Search should have found at least some entries with 'e'");
    }
    
    #[test]
    fn parallel_vs_sequential_indexing_test() {
        let home_dir = home_dir().unwrap_or_default();
        
        // Test sequential indexing
        let start_time = std::time::Instant::now();
        let seq_results = index_given_path(home_dir.clone());
        let seq_duration = start_time.elapsed();
        
        // Test parallel indexing
        let start_time = std::time::Instant::now();
        let par_results = index_given_path_parallel(home_dir);
        let par_duration = start_time.elapsed();
        
        println!("Sequential indexing took: {:?}", seq_duration);
        println!("Parallel indexing took: {:?}", par_duration);
        println!("Sequential found {} entries", seq_results.len());
        println!("Parallel found {} entries", par_results.len());
    }
    
    #[test]
    fn combined_parallel_indexing_and_searching_test() {
        // Measure parallel indexing performance
        let start_time = std::time::Instant::now();
        let entries = start_indexing_home_dir();
        let indexing_duration = start_time.elapsed();
        
        // Ensure we have entries to search through
        assert!(!entries.is_empty(), "Home directory should contain at least some entries");
        println!("Parallel indexing took: {:?}", indexing_duration);
        
        // Common search terms to test
        let search_terms = ["doc", "image", "config", "e"];
        
        for term in search_terms {
            // Measure parallel search performance
            let search_start_time = std::time::Instant::now();
            let search_results = search_by_filename_parallel(term, entries.clone());
            let search_duration = search_start_time.elapsed();
            
            println!("Parallel search for '{}' found {} matches in {:?}", 
                term, search_results.len(), search_duration);
        }
    }
}
