use crate::filesystem::models::VolumeInformation;
use sysinfo::Disks;
use tauri::command;

#[command]
pub fn get_system_volumes_information() -> Vec<VolumeInformation> {
    let mut volume_information_vec: Vec<VolumeInformation> = Vec::new();
    let disks = Disks::new_with_refreshed_list();

    for disk in &disks {
        volume_information_vec.push(VolumeInformation {
            volume_name: disk.name().to_string_lossy().into_owned(), // Convert OsStr to String
            mount_point: disk.mount_point().to_string_lossy().into_owned(), // Convert mount point
            file_system: disk
                .file_system()
                .to_str()
                .expect("Error during parsing the given string from file_system")
                .to_owned(), // Convert file system
            size: disk.total_space(),
            available_space: disk.available_space(),
            is_removable: disk.is_removable(),
            total_written_bytes: disk.usage().total_written_bytes,
            total_read_bytes: disk.usage().total_read_bytes,
        });
    }
    volume_information_vec
}

#[test]
fn get() {
    let volumes = get_system_volumes_information();
    for volume in &volumes {
        println!("{:?}", volume);
    }
}
