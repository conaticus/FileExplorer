# `get_meta_data_as_json`

---
## Parameters
NONE

## Returns
- String - A JSON string representing the metadata. The structure is:
```json
{
  "version": "test-version",
  "abs_file_path_buf": "/var/folders/2w/pshnh3fn1xz05ws6n3kvmf4c0000gn/T/.tmpjJuuxg/meta_data.json",
  "all_volumes_with_information": [
    {
      "volume_name": "Macintosh HD",
      "mount_point": "/",
      "file_system": "apfs",
      "size": 494384795648,
      "available_space": 164522551999,
      "is_removable": false,
      "total_written_bytes": 38754762752,
      "total_read_bytes": 53392805888
    },
    {
      "volume_name": "Macintosh HD",
      "mount_point": "/System/Volumes/Data",
      "file_system": "apfs",
      "size": 494384795648,
      "available_space": 164522551999,
      "is_removable": false,
      "total_written_bytes": 38754762752,
      "total_read_bytes": 53392805888
    }
  ]
}
```