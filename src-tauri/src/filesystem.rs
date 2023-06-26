use crate::{CachedPath, StateSafe};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::{read_dir, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use sysinfo::{Disk, DiskExt, System, SystemExt};
use tauri::State;
use walkdir::WalkDir;

const CACHE_FILE_PATH: &str = "./system_cache.json";

const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}

#[derive(Serialize)]
pub struct Volume {
    name: String,
    mountpoint: PathBuf,
    available_gb: u16,
    used_gb: u16,
    total_gb: u16,
    fs: VolumeFileSystem,
}

#[derive(Serialize)]
pub struct VolumeFileSystem {
    root: PathBuf,
    documents: PathBuf,
    downloads: PathBuf,
    pictures: PathBuf,
    videos: PathBuf,
    home: PathBuf,
    audio: PathBuf,
    desktop: PathBuf,
}

impl VolumeFileSystem {
    fn try_new() -> Result<Self, ()> {
        macro_rules! handle_err {
            ($func:expr) => {
                match $func {
                    Some(dir) => dir,
                    None => return Err(()),
                }
            };
        }

        let documents = handle_err!(dirs::document_dir());
        let downloads = handle_err!(dirs::download_dir());
        let pictures = handle_err!(dirs::picture_dir());
        let videos = handle_err!(dirs::video_dir());
        let home = handle_err!(dirs::home_dir());
        let audio = handle_err!(dirs::audio_dir());
        let desktop = handle_err!(dirs::desktop_dir());
        let root: PathBuf = {
            #[cfg(target_family = "unix")]
            {
                "/".into()
            }
            #[cfg(target_family = "windows")]
            {
                "C:".into()
            }
        };

        Ok(Self {
            root,
            documents,
            downloads,
            pictures,
            videos,
            home,
            audio,
            desktop,
        })
    }
}

impl Volume {
    fn from(disk: &Disk) -> Self {
        let used_bytes = disk.total_space() - disk.available_space();
        let available_gb = bytes_to_gb(disk.available_space());
        let used_gb = bytes_to_gb(used_bytes);
        let total_gb = bytes_to_gb(disk.total_space());

        let name = {
            let volume_name = disk.name().to_str().unwrap();
            match volume_name.is_empty() {
                true => "Local Volume",
                false => volume_name,
            }
            .to_string()
        };

        let mountpoint = disk.mount_point().to_path_buf();

        let fs = VolumeFileSystem::try_new().unwrap();

        Self {
            name,
            available_gb,
            used_gb,
            total_gb,
            fs,
            mountpoint,
        }
    }

    /// This traverses the provided volume and adds the file structure to the cache in memory.
    fn create_cache(&self, state_mux: &StateSafe) {
        let state = &mut state_mux.lock().unwrap();

        let volume = state
            .system_cache
            .entry(self.mountpoint.to_string_lossy().to_string())
            .or_insert_with(HashMap::new);

        let system_cache = Arc::new(Mutex::new(volume));

        WalkDir::new(self.mountpoint.clone())
            .into_iter()
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .for_each(|entry| {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_path = entry.path().to_string_lossy().to_string();

                let walkdir_filetype = entry.file_type();
                let file_type = if walkdir_filetype.is_dir() {
                    "directory"
                } else {
                    "file"
                }
                .to_string();

                let cache_guard = &mut system_cache.lock().unwrap();
                cache_guard
                    .entry(file_name)
                    .or_insert_with(Vec::new)
                    .push(CachedPath {
                        file_path,
                        file_type,
                    });
            });
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DirectoryChild {
    File(String, String), // Name of file, path to file
    Directory(String, String),
}

/// Gets the cache from the state (in memory), encodes and saves it to the cache file path.
/// This needs optimising.
pub fn save_system_cache(state_mux: &StateSafe) {
    let state = &mut state_mux.lock().unwrap();
    let serialized_cache = serde_json::to_string(&state.system_cache).unwrap();

    let mut file = fs::OpenOptions::new()
        .write(true)
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

/// Gets list of volumes and returns them.
/// If there is a cache stored on volume it is loaded.
/// If there is no cache stored on volume, one is created as well as stored in memory.
#[tauri::command]
pub fn get_volumes(state_mux: State<StateSafe>) -> Vec<Volume> {
    let mut volumes = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    let cache_exists = fs::metadata(CACHE_FILE_PATH).is_ok();
    if cache_exists {
        load_system_cache(&state_mux);
    } else {
        File::create(CACHE_FILE_PATH).unwrap();
    }

    for disk in sys.disks() {
        let volume = Volume::from(disk);

        if !cache_exists {
            volume.create_cache(&state_mux);
        }

        volumes.push(volume);
    }

    save_system_cache(&state_mux);

    volumes
}

/// Searches and returns the files in a given directory. This is not recursive.
#[tauri::command]
pub fn open_directory(path: String) -> Vec<DirectoryChild> {
    let mut dir_children = Vec::new();

    let Ok(directory) = read_dir(path) else {
        return dir_children;
    };

    for entry in directory {
        let entry = entry.unwrap();

        let file_name = entry.file_name().to_str().unwrap().to_string();
        let entry_is_file = entry.file_type().unwrap().is_file();
        let entry = entry.path().to_str().unwrap().to_string();

        if entry_is_file {
            dir_children.push(DirectoryChild::File(file_name, entry));
            continue;
        }

        dir_children.push(DirectoryChild::Directory(file_name, entry));
    }

    dir_children
}
