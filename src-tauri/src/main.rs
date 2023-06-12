// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

struct Drive {
    name: String,
    used_capacity: i32,
    total_capacity: i32,
}

enum DirectoryChild {
    File(String), // String is the path to the file
    Directory(String)
}

// #[tauri::command]
// fn get_drives() -> Vec<Drive> {
// }

// #[tauri::command]
// fn open_drive() {}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_drives])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
