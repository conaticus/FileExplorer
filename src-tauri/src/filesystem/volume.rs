use crate::filesystem::cache::{
    load_system_cache, run_cache_interval, save_system_cache, FsEventHandler, CACHE_FILE_PATH,
};
use crate::filesystem::{bytes_to_gb, DIRECTORY, FILE};
use crate::{CachedPath, StateSafe};
use notify::{RecursiveMode, Watcher};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use sysinfo::{Disk, DiskExt, System, SystemExt};
use tauri::State;
use tokio::task::block_in_place;
use walkdir::WalkDir;

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
                    DIRECTORY
                } else {
                    FILE
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

    fn watch_changes(&self, state_mux: &StateSafe) {
        let mut fs_event_manager = FsEventHandler::new(state_mux.clone(), self.mountpoint.clone());

        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(event) => fs_event_manager.handle_event(event),
            Err(e) => panic!("Failed to handle event: {:?}", e),
        })
        .unwrap();

        let path = self.mountpoint.clone();

        thread::spawn(move || {
            watcher.watch(&path, RecursiveMode::Recursive).unwrap();

            block_in_place(|| loop {
                thread::park();
            })
        });
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DirectoryChild {
    File(String, String), // Name of file, path to file
    Directory(String, String),
}

/// Gets list of volumes and returns them.
/// If there is a cache stored on volume it is loaded.
/// If there is no cache stored on volume, one is created as well as stored in memory.
#[tauri::command]
pub fn get_volumes(state_mux: State<StateSafe>) -> Vec<Volume> {
    let mut volumes = Vec::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    let mut cache_exists = fs::metadata(&CACHE_FILE_PATH[..]).is_ok();
    if cache_exists {
        cache_exists = load_system_cache(&state_mux);
    } else {
        File::create(&CACHE_FILE_PATH[..]).unwrap();
    }

    for disk in sys.disks() {
        let volume = Volume::from(disk);

        if !cache_exists {
            volume.create_cache(&state_mux);
        }

        volume.watch_changes(&state_mux);
        volumes.push(volume);
    }

    save_system_cache(&state_mux);
    run_cache_interval(&state_mux);

    volumes
}
