# Documentation for Endpoints

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

The metadata contains information about the system and sate of the application. Fields are following:

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
