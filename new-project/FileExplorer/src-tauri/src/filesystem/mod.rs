pub mod cache;
pub mod explorer;
mod fs_utils;
pub mod volume;

pub const DIRECTORY: &str = "directory";
pub const FILE: &str = "file";

pub const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}
