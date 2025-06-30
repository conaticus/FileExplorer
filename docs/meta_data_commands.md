# `get_meta_data_as_json`

Error Structure as json can be found [here](./error_structure.md).

- All possible values which are given in the 
  - `current_running_os` filed -> https://doc.rust-lang.org/std/env/consts/constant.OS.html
  - `current_cpu_architecture` filed -> https://doc.rust-lang.org/std/env/consts/constant.ARCH.html
  - `user_home_dir` field -> when there is a dir there is a path given but when no dir can be found
    then it is empty so `""`
---
## Parameters
NONE

## Returns
- String - A JSON string representing the metadata. The structure is:

Every Size is given in bytes so it is universal.

```json
{
  "version": "test-version",
  "abs_file_path_buf": "/var/folders/2w/pshnh3fn1xz05ws6n3kvmf4c0000gn/T/.tmpjJuuxg/meta_data.json",
  "current_running_os": "macos",
  "current_cpu_architecture": "",
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
