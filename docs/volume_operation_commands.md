# `get_system_volumes_information_as_json`

---
## Parameters
NONE

## Returns
- String - A JSON string representing the metadata. The structure is:
```json
[
  {
    "volume_name":"Macintosh HD",
    "mount_point":"/",
    "file_system":"apfs",
    "size":494384795648,
    "available_space":164262259391,
    "is_removable":false,
    "total_written_bytes":44234715136,
    "total_read_bytes":57412698112
  },
  {
    "volume_name":"Macintosh HD",
    "mount_point":"/System/Volumes/Data",
    "file_system":"apfs",
    "size":494384795648,
    "available_space":164262259391,
    "is_removable":false,
    "total_written_bytes":44234715136,
    "total_read_bytes":57412698112
  }
]
```