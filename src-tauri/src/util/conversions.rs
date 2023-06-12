use std::ffi::OsStr;
use std::path::Path;

pub fn os_to_string(os_string: &OsStr) -> String {
    os_string.to_string_lossy().to_string()
}

pub fn path_to_string(os_string: &Path) -> String {
    let s = os_string.to_string_lossy();
    s[..s.len() - 2].to_string()
}

pub fn bytes_to_gb(bytes: u64) -> u16 {
    (bytes / (1e+9 as u64)) as u16
}
