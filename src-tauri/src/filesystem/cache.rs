use crate::{AppState, CachedPath, StateSafe, VolumeCache};
use std::{fs};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, MutexGuard};
use std::time::Duration;
use notify::{Event};
use notify::event::{CreateKind, ModifyKind, RenameMode};
use tokio::time;
use crate::filesystem::{DIRECTORY, FILE};

pub const CACHE_FILE_PATH: &str = "./system_cache.json";

/// Handles filesystem events, currently intended for cache invalidation.
pub struct FsEventHandler {
    state_mux: StateSafe,
    mountpoint: PathBuf,
}

impl FsEventHandler {
    pub fn new(state_mux: StateSafe, mountpoint: PathBuf) -> Self {
        Self { state_mux, mountpoint }
    }

    /// Gets the current volume from the cache
    fn get_from_cache<'a>(&self, state: &'a mut AppState) -> &'a mut VolumeCache {
        let mountpoint = self.mountpoint.to_string_lossy().to_string();

        state.system_cache.get_mut(&mountpoint)
            .unwrap_or_else(|| panic!("Failed to find mountpoint '{:?}' in cache.", self.mountpoint))
    }

    pub fn handle_create(&self, kind: CreateKind, path: &Path) {
        let state = &mut self.state_mux.lock().unwrap();
        let current_volume = self.get_from_cache(state);

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        let file_type = match kind {
            CreateKind::File => FILE,
            CreateKind::Folder => DIRECTORY,
            _ => return, // Other options are weird lol
        }.to_string();

        let file_path = path.to_string_lossy().to_string();
        current_volume.entry(filename).or_insert(vec![CachedPath{file_path, file_type}]);
    }

    pub fn handle_delete(&self, path: &Path) {
        let state = &mut self.state_mux.lock().unwrap();
        let current_volume = self.get_from_cache(state);

        let filename = path.file_name().unwrap().to_string_lossy().to_string();
        current_volume.remove(&filename);
    }

    /// Removes file from cache, when `handle_rename_to` is called a new file is added to the cache in place.
    pub fn handle_rename_from(&mut self, old_path: &Path) {
        let state = &mut self.state_mux.lock().unwrap();
        let current_volume = self.get_from_cache(state);

        let old_path_string=  old_path.to_string_lossy().to_string();
        let old_filename = old_path.file_name().unwrap().to_string_lossy().to_string();

        let empty_vec = &mut Vec::new();
        let cached_paths = current_volume.get_mut(&old_filename).unwrap_or(empty_vec);

        // If there is only one item in the cached paths, this means it can only be the renamed file and therefore it should be removed from the hashmap
        if cached_paths.len() <= 1 {
            current_volume.remove(&old_filename);
            return;
        }

        cached_paths.retain(|path| path.file_path != old_path_string);
    }

    /// Adds new file name & path to cache.
    pub fn handle_rename_to(&self, new_path: &Path) {
        let state = &mut self.state_mux.lock().unwrap();
        let current_volume = self.get_from_cache(state);

        let filename = new_path.file_name().unwrap().to_string_lossy().to_string();
        let file_type = if new_path.is_dir() { DIRECTORY } else { FILE };

        let path_string = new_path.to_string_lossy().to_string();
        current_volume.entry(filename).or_insert(vec![CachedPath{file_path: path_string, file_type: String::from(file_type)}]);
    }

    pub fn handle_event(&mut self, event: Event) {
        let paths = event.paths;

        match event.kind {
            notify::EventKind::Modify(modify_kind) => {
                if modify_kind == ModifyKind::Name(RenameMode::From) {
                    self.handle_rename_from(&paths[0]);
                } else if modify_kind == ModifyKind::Name(RenameMode::To) {
                    self.handle_rename_to(&paths[0]);
                }
            },
            notify::EventKind::Create(kind) => self.handle_create(kind, &paths[0]),
            notify::EventKind::Remove(_) => self.handle_delete(&paths[0]),
            _ => (),
        }
    }
}

/// Starts a constant interval loop where the cache is updated every ~30 seconds.
pub fn run_cache_interval(state_mux: &StateSafe) {
    let state_clone = Arc::clone(state_mux);

    tokio::spawn(async move { // We use tokio spawn because async closures with std spawn is unstable
        let mut interval = time::interval(Duration::from_secs(30));
        interval.tick().await; // Wait 30 seconds before doing first re-cache

        loop {
            interval.tick().await;

            let guard = &mut state_clone.lock().unwrap();
            save_to_cache(guard);
        }
    });
}

/// This takes in an Arc<Mutex<AppState>> and calls `save_to_cache` after locking it.
pub fn save_system_cache(state_mux: &StateSafe) {
    let state = &mut state_mux.lock().unwrap();
    save_to_cache(state);
}

/// Gets the cache from the state (in memory), encodes and saves it to the cache file path.
/// This needs optimising.
fn save_to_cache(state: &mut MutexGuard<AppState>) {
    let serialized_cache = serde_json::to_string(&state.system_cache).unwrap();

    let mut file = fs::OpenOptions::new()
        .write(true)
        .truncate(true) // I have literally no clue why but without truncate, the JSON gets messed up and a trailing character error occurs when deserializing the file contents.
        .open(CACHE_FILE_PATH)
        .unwrap();

    file.write_all(serialized_cache.as_bytes()).unwrap();
}

/// Reads and decodes the cache file and stores it in memory for quick access.
pub fn load_system_cache(state_mux: &StateSafe) {
    let state = &mut state_mux.lock().unwrap();
    let file_contents = fs::read_to_string(CACHE_FILE_PATH).unwrap();
    state.system_cache = serde_json::from_str(&file_contents).unwrap();
}
