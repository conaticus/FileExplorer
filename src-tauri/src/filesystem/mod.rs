pub mod cache;
mod fs_utils;
pub mod volume;
pub mod file_operations;
pub mod file_system_operations;
pub mod fs_entry_options;
pub mod basic_file_operations;

pub const DIRECTORY: &str = "directory";
pub const FILE: &str = "file";

pub const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}
