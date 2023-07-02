use std::fs;
use std::fs::read_dir;
use crate::errors::Error;
use crate::filesystem::volume::DirectoryChild;
use crate::filesystem::volume::DirectoryChild::File;

/// Opens a file at the given path. Returns a string if there was an error.
// NOTE(conaticus): I tried handling the errors nicely here but Tauri was mega cringe and wouldn't let me nest results in async functions, so used string error messages instead.
#[tauri::command]
pub async fn open_file(path: String) -> Result<(), Error> {
    let output_res = open::commands(path)[0].output();
    let output = match output_res {
        Ok(output) => output,
        Err(err) => {
            let err_msg = format!("Failed to get open command output: {}", err);
            return Err(Error::Custom(err_msg));
        }
    };

    if output.status.success() {
        return Ok(());
    }

    let err_msg = String::from_utf8(output.stderr).unwrap_or(String::from("Failed to open file and deserialize stderr."));
    Err(Error::Custom(err_msg))
}

/// Searches and returns the files in a given directory. This is not recursive.
#[tauri::command]
pub async fn open_directory(path: String) -> Result<Vec<DirectoryChild>, ()> {
    let Ok(directory) = read_dir(path) else {
        return Ok(Vec::new());
    };

    Ok(directory
        .map(|entry| {
            let entry = entry.unwrap();

            let file_name = entry.file_name().to_string_lossy().to_string();
            let entry_is_file = entry.file_type().unwrap().is_file();
            let entry = entry.path().to_string_lossy().to_string();

            if entry_is_file {
                return DirectoryChild::File(file_name, entry);
            }

            DirectoryChild::Directory(file_name, entry)
        })
        .collect())
}

#[tauri::command]
pub async fn create_file(path: String) -> Result<(), Error> {
    let res = fs::File::create(path);
    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(Error::Custom(err.to_string())),
    }
}