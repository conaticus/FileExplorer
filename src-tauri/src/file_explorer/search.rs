use std::ops::Deref;
use fuzzy_matcher::clangd::fuzzy_match;
use fuzzy_matcher::FuzzyMatcher;
use crate::file_explorer::DirectoryChild;
use walkdir::WalkDir;
use crate::util::conversions::ostr_to_string;
use fuzzy_matcher::skim::SkimMatcherV2;

#[tauri::command]
pub fn search_directory(query: String, search_directory: String, extension: String, accept_files: bool, accept_directories: bool) -> Vec<DirectoryChild> {
    let mut results = Vec::new();
    let mut fuzzy_scores = Vec::new();
    let matcher = SkimMatcherV2::default();

    for entry in WalkDir::new(search_directory).into_iter().flatten() {
        let file_name = entry.file_name().to_string_lossy();

        if entry.file_type().is_file() {
            if !accept_files { continue }

            if extension.len() > 0 && !file_name.ends_with(extension.as_str()) { continue }


            // Remove extension from file name.
            let cleaned_filename = entry.path().file_stem().and_then(|stem| stem.to_str()).unwrap_or("");
            let score = matcher.fuzzy_match(cleaned_filename, &query).unwrap_or(0);
            if score < 20 { continue }

            fuzzy_scores.push(score);
            results.push(DirectoryChild::File(file_name.to_string()));

            continue
        } else if !accept_directories { continue }

        let score = matcher.fuzzy_match(&file_name, &query).unwrap_or(0);
        if score < 40 { continue }

        results.push(DirectoryChild::Directory(file_name.to_string()));
        fuzzy_scores.push(score);
    }

    let mut tuples: Vec<(usize, _)> = fuzzy_scores.iter().enumerate().collect();
    tuples.sort_by(|a, b| b.1.cmp(a.1));
    results = tuples.into_iter().map(|(index, _)| results[index].clone()).collect();

    results
}