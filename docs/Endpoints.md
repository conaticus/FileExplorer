# Documentation for Endpoints

This document provides an overview of the available endpoints in the application. Each endpoint is
described with its purpose, example usage, and response format.
All endpoints are designed to be invoked using the `invoke` function, which is part of the Tauri
API.
The `invoke` function allows you to call Rust commands from your JavaScript code. The endpoints are
designed to be used in a React application, but the concepts can be applied to other frameworks as
well.

The React code snippets may be wrong or incomplete, but they should give you a good idea of how to
use the endpoints.

# List of Endpoints

- [Get Metadata](#1-get_meta_data-endpoint) (Endpoint to get the metadata of the application)
- [Get all entries for a Directory](#2-get_entries_for_directory-endpoint) (Endpoint to get a complete
  json file for all the entries for a directory)
- [Set/get a Path for an action manually](#3-set_selected_path_for_action-and-get_selected_path_for_action-endpoint)
  (Endpoint to set the selected path for an action)

## 1. `get_meta_data` endpoint

```typescript jsx
useEffect(() => {
    const fetchMetaData = async () => {
        try {
            const result = await invoke("get_meta_data");
            console.log("Fetched MetaData:", result);
        } catch (error) {
            console.error("Error fetching metadata:", error);
        }
    };

    fetchMetaData();
}, []);
```

### Example Response

```json
{
  "version": "1.0.0",
  "abs_file_path_buf": "/path/to/file",
  "all_volumes_with_information": [
    {
      "volume_name": "Volume1",
      "mount_point": "/mnt/volume1",
      "file_system": "ext4",
      "size": 1000000000,
      "available_space": 500000000,
      "is_removable": false,
      "total_written_bytes": 10000000,
      "total_read_bytes": 5000000
    }
  ]
}
```

### Rust background

The metadata contains information about the system and sate of the application. Fields are
following:

- `version`: The version of the application.
- `abs_file_path_buf`: The absolute file path of the file
- `all_volumes_with_information`: A list of all volumes with information.
    - Volume information is:
  ```rust
  pub struct VolumeInformation {
    pub volume_name: String,
    pub mount_point: String,
    pub file_system: String,
    pub size: u64,
    pub available_space: u64,
    pub is_removable: bool,
    pub total_written_bytes: u64,
    pub total_read_bytes: u64,
  }
  ```

### The Rust code

```rust
pub struct MetaData {
    version: String,
    abs_file_path_buf: PathBuf,
    all_volumes_with_information: Vec<VolumeInformation>,
}
```

## 2. `get_entries_for_directory` endpoint

```typescript jsx
useEffect(() => {
    const fetchEntries = async () => {
        try {
            const result = await invoke("get_entries_for_directory", {path: "/path/to/directory"});
            console.log("Fetched Entries:", result);
        } catch (error) {
            console.error("Error fetching entries:", error);
        }
    };

    fetchEntries();
}, []);
```

### Example Response

The example response can be found in [fs_dir_loader](fs_dir_loader.json) file.

## 3. `set_selected_path_for_action` and `get_selected_path_for_action` endpoint

This endpoint is used to set the selected path for an action.
**It is not used when you want to copy, move, or delete files. It is only used when you want to select
a file from the frontend and want to do something other that the provided functions. For deleting,
creating etc. are other endpoints provided.**

```typescript jsx
useEffect(() => {
    const setPath = async () => {
        try {
            const result = await invoke("set_selected_path_for_action", {path: "/path/to/directory"});
            console.log("Set Path Result:", result);
        } catch (error) {
            console.error("Error setting path:", error);
        }
    };

    setPath();
}, []);
```
```typescript
useEffect(() => {
    const getPath = async () => {
        try {
            const result = await invoke("get_selected_path_for_action");
            console.log("Get Path Result:", result);
        } catch (error) {
            console.error("Error getting path:", error);
        }
    };

    getPath();
}, []);
```