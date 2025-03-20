use std::{io};

use std::path::{Path, PathBuf};


pub fn with_permission_check<F>(path: &str, func: F) -> io::Result<()>
where
    F: FnOnce() -> io::Result<()>,
{
    // Perform permission check before executing the function
    if let Err(e) = restricted_file_access(path) {
        return Err(e);
    }
    // Call the actual function
    func()
}

fn restricted_file_access(path: &str) -> Result<(), io::Error> {
    match extract_path_until_target(path, "test-data-dir") {
        Some(_) => Ok(()),
        None => Err(io::Error::new(io::ErrorKind::PermissionDenied, "Access denied due to wrong directory for testing")),
    }
}

// Converts path to Unix-style with '/' separators
fn _path_to_unix_style(path: &Path) -> String {
    path.to_string_lossy()
        .replace("\\", "/") // Replace backslashes with forward slashes on Windows
        .to_string()
}

fn extract_path_until_target(path: &str, target: &str) -> Option<String> {
    let path = Path::new(path);

    let  components = path.components();
    let mut result = PathBuf::new();

    // Iterate through the components of the path
    for component in components {
        let component_str = component.as_os_str().to_string_lossy(); // Convert to a string
        // Check if the current component matches the target directory
        if component_str == target {
            break;
        }

        // Append the component to the result path
        result.push(component);
    }

    // If we have a valid result, convert it to a string and return it
    if !result.as_os_str().is_empty() {
        Some(result.to_string_lossy().to_string())
    } else {
        None
    }
}
