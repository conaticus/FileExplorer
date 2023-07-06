use serde::{Deserialize, Serialize};

pub mod explorer;
pub mod cache;
pub mod volume;
mod fs_utils;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum FileType{
    Directory,
    File
}

pub const fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}