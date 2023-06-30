use crate::filesystem::volume::DirectoryChild;
use crate::StateSafe;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::Path;
use std::time::Instant;
use tauri::State;

const MINIMUM_SCORE: i16 = 20;

#[derive(Default)]
struct SearchEngine {
    matcher: SkimMatcherV2,

    /// the mountpoint for the volume the user is in
    mountpoint: Option<String>,

    /// search query entry
    query: String,

    /// searches for files that have a certain extension
    extension: Option<String>,

    /// includes files in search
    is_file: bool,

    /// includes directories in search
    is_dir: bool,

    /// stores search results
    results: Vec<DirectoryChild>,

    /// stores match scores where 1000 means exact match and 0 means no match
    fuzzy_scores: Vec<i16>,

    /// ???
    search_directory: String,
}

impl SearchEngine {
    fn search_query(&mut self, state_mux: State<StateSafe>) -> Option<Vec<DirectoryChild>> {
        let start_time = Instant::now();
        let state = state_mux.lock().unwrap();
        let query = self.query.to_lowercase();

        let mountpoint = &self.mountpoint.clone()?;
        let system_cache = state.system_cache.get(mountpoint).unwrap();
        for (filename, paths) in system_cache {
            for path in paths {
                let file_type = &path.file_type;
                let file_path = &path.file_path;

                if !file_path.starts_with(&self.search_directory) {
                    continue;
                }

                if file_type == "file" {
                    self.check_file(filename, &file_path);

                    continue;
                }

                if self.is_dir {
                    // Gives a filename a fuzzy matcher score
                    // score is 1000 if there is an exact match
                    let score = if *filename == query {
                        1000
                    } else {
                        self.matcher
                            .fuzzy_match(filename, self.query.as_str())
                            .unwrap_or(0) as i16
                    };

                    if score < MINIMUM_SCORE {
                        continue;
                    }

                    self.results.push(DirectoryChild::Directory(
                        filename.to_string(),
                        file_path.to_string(),
                    ));
                    self.fuzzy_scores.push(score);
                }
            }
        }

        let end_time = Instant::now();
        println!("Elapsed time: {:?}", end_time - start_time);

        // Sort by best match first.
        let mut tuples: Vec<(usize, _)> = self.fuzzy_scores.iter().enumerate().collect();
        tuples.sort_by(|a, b| b.1.cmp(a.1));

        Some(tuples
            .into_iter()
            .map(|(index, _)| self.results[index].clone())
            .collect())
    }

    fn check_file(&mut self, filename: &String, filepath: &String) {
        if !self.is_file {
            return;
        }

        // checks for file extension match
        if let Some(ref extension) = self.extension {
            if !filename.ends_with(extension.as_str()) {
                return;
            }
        }

        let filename_path = Path::new(filename);
        let cleaned_filename = filename_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");

        let score = if *filename == self.query {
            1000
        } else {
            self.matcher
                .fuzzy_match(cleaned_filename, self.query.as_str())
                .unwrap_or(0) as i16
        };

        if score < MINIMUM_SCORE {
            return;
        }

        self.results.push(DirectoryChild::File(
            filename.to_string(),
            filepath.to_string(),
        ));

        self.fuzzy_scores.push(score);
    }
}

/// Reads the cache and does a fuzzy search for a directory.
/// Takes into account the filters provided.
/// Returns the results ONLY when the entire volume is searched
#[tauri::command]
pub fn search_directory(
    state_mux: State<StateSafe>,
    query: String,
    mount_pnt: String,
    extension: String,
    accept_files: bool,
    accept_directories: bool,
) -> Vec<DirectoryChild> {
    let mut engine = SearchEngine::default();
    engine.query = query;
    engine.mountpoint = Some(mount_pnt);
    engine.extension = Some(extension);
    engine.is_dir = accept_directories;
    engine.is_file = accept_files;
    engine.search_query(state_mux).unwrap()
}
