use crate::models::VolumeInformation;
use sysinfo::Disks;

#[tauri::command]
pub fn get_system_volumes_information_as_json() -> String {
    let volume_information_vec = get_system_volumes_information();
    serde_json::to_string(&volume_information_vec).unwrap()
}

/// Gets information about all system volumes/disks
///
/// This function can be called both from Rust code and from the frontend via Tauri
#[tauri::command]
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

    // Create a new vector to store non-duplicate items
    let mut result = Vec::new();
    let mut skip_indices = std::collections::HashSet::new();

    // First pass: identify duplicates
    for i in 0..volume_information_vec.len() {
        if skip_indices.contains(&i) {
            continue;
        }

        for j in i + 1..volume_information_vec.len() {
            
            // Check if the two volumes have the same name
            if volume_information_vec[i].volume_name == volume_information_vec[j].volume_name {
                // Mark the one with longer mount_point to be skipped
                if volume_information_vec[i].mount_point.len() > volume_information_vec[j].mount_point.len() {
                    skip_indices.insert(i);
                } else {
                    skip_indices.insert(j);
                }
            }
            
            // Check if the two volumes have the same mount point
            if volume_information_vec[i].mount_point == volume_information_vec[j].mount_point {
                // Mark the one with longer volume_name to be skipped
                if volume_information_vec[i].volume_name.len() > volume_information_vec[j].volume_name.len() {
                    skip_indices.insert(i);
                } else {
                    skip_indices.insert(j);
                }
            }
        }
    }

    // Second pass: collect non-skipped items and remove boot volumes
    for (index, volume) in volume_information_vec.into_iter().enumerate() {
        if !skip_indices.contains(&index) {
            
            //filter boot volumes out on second pass
            if volume.mount_point == "C:\\" || volume.mount_point == "C:" || volume.mount_point.contains("boot") {
                continue;
            }
            result.push(volume);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_volumes() {
        let volumes = get_system_volumes_information();
        assert!(!volumes.is_empty(), "Should return at least one volume");

        let volumes_as_json = get_system_volumes_information_as_json();

        //printing the JSON string for debugging
        println!("Volumes as JSON: {}", volumes_as_json);

        for volume in &volumes {
            println!("{:?}", volume);
        }
    }
}
