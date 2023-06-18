use std::path::Path;
use std::time::Instant;
use fuzzy_matcher::FuzzyMatcher;
use crate::file_explorer::DirectoryChild;
use fuzzy_matcher::skim::SkimMatcherV2;
use rayon::prelude::*;
use tauri::State;
use crate::StateSafe;

#[tauri::command]
pub fn search_directory(state_mux: State<'_, StateSafe>, query: String, search_directory: String, extension: String, accept_files: bool, accept_directories: bool) -> Vec<DirectoryChild> {
    let start_time = Instant::now();

    let mut results: Vec<_> = Vec::new();
    let mut fuzzy_scores: Vec<_> = Vec::new();
    let matcher = SkimMatcherV2::default();

    let state = state_mux.lock().unwrap();
    let letter = search_directory.chars().nth(0).unwrap().to_string();
    let query = query.to_lowercase();

    let disk_cache = state.drive_cache.get(letter.as_str()).unwrap();
    for (file_name, paths) in disk_cache {
        for path in paths {
            let file_type = &path.file_type;
            let file_path = &path.file_path;

            if file_type == "file" {
                if !accept_files { continue; }

                if extension.len() > 0 && !file_name.ends_with(extension.as_str()) { continue; }

                let filename_path = Path::new(file_name);
                let cleaned_filename = filename_path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("");

                let mut score;

                if cleaned_filename != &query {
                    score = matcher.fuzzy_match(cleaned_filename, &query).unwrap_or(0);
                    if score < 20 { continue; }
                } else {
                    score = 1000;
                }

                results.push(DirectoryChild::File(file_name.to_string(), file_path.to_string()));
                fuzzy_scores.push(score);
            } else {
                if !accept_directories { continue }

                let score = matcher.fuzzy_match(&file_name, &query).unwrap_or(0);
                if score < 20 { continue; }

                results.push(DirectoryChild::Directory(file_name.to_string(), file_path.to_string()));
                fuzzy_scores.push(score);
            }
        }
    }

    let end_time = Instant::now();
    println!("Elapsed time: {:?}", end_time - start_time);

    let mut tuples: Vec<(usize, _)> = fuzzy_scores.iter().enumerate().collect();
    tuples.sort_by(|a, b| b.1.cmp(a.1));

    tuples.into_iter().map(|(index, _)| results[index].clone()).collect()
}