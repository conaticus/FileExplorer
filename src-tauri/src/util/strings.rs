use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

pub fn ostr_to_string(os_string: &OsStr) -> String {
    os_string.to_string_lossy().to_string()
}
pub fn os_to_string(os_string: OsString) -> String { os_string.to_string_lossy().to_string() }
pub fn pathbuf_to_string(path: PathBuf) -> String { path.to_string_lossy().to_string() }
pub fn bytes_to_gb(bytes: u64) -> u16 { (bytes / (1e+9 as u64)) as u16 }

pub fn get_letter_from_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.chars().nth(0).unwrap().to_string()
}
