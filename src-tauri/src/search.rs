use crate::filesystem::volume::DirectoryChild;
use crate::StateSafe;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::path::Path;
use std::time::Instant;
use tauri::State;

const MINIMUM_SCORE: i16 = 20;

type FilterCallback =
    fn(query: &SearchEngine, filename: &String, filepath: &crate::CachedPath) -> bool;

#[derive(Default)]
struct SearchEngine {
    matcher: SkimMatcherV2,
    /// stores filter functions used to determine what files match query
    query_filters: Vec<FilterCallback>,

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

    /// the directory from which the search is being made
    search_directory: String,
}

impl SearchEngine {
    /// TODO consider changing to a builder pattern to reduce boilerplate
    fn new(
        query: String,
        mountpoint: Option<String>,
        extension: Option<String>,
        is_dir: bool,
        is_file: bool,
    ) -> Self {
        // search engine initialization
        let mut engine = SearchEngine::default();
        engine.matcher = engine.matcher.smart_case();
        engine.query = query.to_lowercase();
        engine.mountpoint = mountpoint;
        engine.extension = extension;
        engine.is_dir = is_dir;
        engine.is_file = is_file;

        // adds filters to engine
        // note: make sure you add a field for switching on and off your filter
        engine
            // checks for file extension match
            .add_filter(|query, filename, _| match query.extension {
                Some(ref extension) => {
                    if filename.ends_with(extension.as_str()) {
                        true
                    } else {
                        false
                    }
                }

                // ignores file extensions if not set
                None => true,
            });

        engine
    }

    fn add_filter(&mut self, filter: FilterCallback) -> &mut Self {
        self.query_filters.push(filter);
        self
    }

    // returns true if path passes every enabled filter
    fn passes_filters(&mut self, filename: &String, filepath: &crate::CachedPath) -> bool {
        for filter in &self.query_filters {
            if !filter(&self, filename, filepath) {
                return false;
            }
        }

        true
    }

    fn search_query(&mut self, state_mux: State<StateSafe>) -> Option<Vec<DirectoryChild>> {
        let start_time = Instant::now();
        let state = state_mux.lock().unwrap();

        let mountpoint = &self.mountpoint.clone()?;
        let system_cache = state.system_cache.get(mountpoint).unwrap();
        for (filename, paths) in system_cache {
            for path in paths {
                let file_type = &path.file_type;
                let file_path = &path.file_path;

                if !file_path.starts_with(&self.search_directory) {
                    continue;
                }

                if !self.passes_filters(filename, path) {
                    continue;
                }

                if file_type == "file" {
                    let filename_path = Path::new(filename);
                    let cleaned_filename = filename_path
                        .file_stem()
                        .and_then(|stem| stem.to_str())
                        .unwrap_or("");

                    self.score_file(&cleaned_filename.to_string(), &file_path);
                    continue;
                }

                if self.is_dir {
                    self.score_file(filename, &file_path);
                }
            }
        }

        let end_time = Instant::now();
        println!("Elapsed time: {:?}", end_time - start_time);

        // Sort by best match first.
        let mut tuples: Vec<(usize, _)> = self.fuzzy_scores.iter().enumerate().collect();
        tuples.sort_by(|a, b| b.1.cmp(a.1));

        Some(
            tuples
                .into_iter()
                .map(|(index, _)| self.results[index].clone())
                .collect(),
        )
    }

    /// Gives a filename a fuzzy matcher score
    /// score is 1000 if there is an exact match
    fn score_file(&mut self, filename: &String, filepath: &String) {
        let score = if *filename == self.query {
            1000
        } else {
            self.matcher
                .fuzzy_match(filename, self.query.as_str())
                .unwrap_or(0) as i16
        };

        if score < MINIMUM_SCORE {
            return;
        }

        self.results.push(DirectoryChild::Directory(
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
    let mut engine = SearchEngine::new(
        query,
        Some(mount_pnt),
        Some(extension),
        accept_directories,
        accept_files,
    );
    engine.search_query(state_mux).unwrap()
}
