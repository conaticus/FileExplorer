pub mod basic_file_operations;
pub mod cache;
pub mod file_operations;
pub mod fs_entry_options;
mod fs_utils;
pub mod infos;
pub mod volume;
pub mod volume_operations;
mod fs_dir_loader;
pub mod models;

pub const DIRECTORY: &str = "directory";
pub const FILE: &str = "file";

pub const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}
