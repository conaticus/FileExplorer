# Tauri Filesystem Commands Documentation

## Content
- [Open a File](#open_file-endpoint)
- [Create a File](#create_file-endpoint)
- [Open a Directory](#open_directory-endpoint)
- [Create a Directory](#create_directory-endpoint)
- [Rename a Dir or File](#rename-endpoint)
- [Move a Dir or File to trash](#move_to_trash-endpoint)


# `open_file` endpoint

---
## Parameters
- `file_path`: The path to the file to be opened. This should be a string representing the absolute path to the file.
## Returns
- Ok(String) - The content of a file as a string.
- Err(String) - An error message if the file cannot be opened or other errors occur.

## Example call
```typescript jsx
useEffect(() => {
    const fetchMetaData = async () => {
        try {
            const result = await invoke("open_file", { file_path: "/path/to/file" });
            console.log("Fetched MetaData:", result);
        } catch (error) {
            console.error("Error fetching metadata:", error);
        }
    };

    fetchMetaData();
}, []);
```
# `create_file` endpoint

---
## Parameters
- `folder_path_abs`: The absolute path to the folder where the file will be created.
- `file_name`: The name of the file to be created. This should be a string representing the name of the file.

## Returns
- Ok(): No content is returned. The function will create a file at the specified path.
- Err(String) - An error message if the file cannot be created or other errors occur.

# `open_directory` endpoint

---
- `path`: The path to the directory to be opened. This should be a string representing the absolute path to the directory.

## Returns
- Ok(String) - A JSON string representing the contents of the directory. The structure is:
```json
  {
    "directories": [
      {
        "name": "subdir",
        "path": "/path/to/subdir",
        "is_symlink": false,
        "access_rights_as_string": "rwxr-xr-x",
        "access_rights_as_number": 16877,
        "size_in_bytes": 38,
        "sub_file_count": 2,
        "sub_dir_count": 1,
        "created": "2023-04-13 19:34:14",
        "last_modified": "2023-04-13 19:34:14",
        "accessed": "2023-04-13 19:34:14"
      }
    ],
    "files": [
      {
        "name": "file1.txt",
        "path": "/path/to/file1.txt",
        "is_symlink": false,
        "access_rights_as_string": "rw-r--r--",
        "access_rights_as_number": 33188,
        "size_in_bytes": 15,
        "created": "2023-04-13 19:34:14",
        "last_modified": "2023-04-13 19:34:14",
        "accessed": "2023-04-13 19:34:14"
      }
    ]
  }
```

# `create_directory` endpoint

---
## Parameters

- `folder_path_abs`: The absolute path to the folder where the directory will be created.
- `directory_name`: The name of the directory to be created. This should be a string representing the name of the directory.

## Returns
- Ok(): No content is returned. The function will create a directory at the specified path.
- Err(String) - An error message if the directory cannot be created or other errors occur.

# `rename` endpoint

---
## Parameters
- `old_path`: The current path of the file or directory to be renamed. This should be a string representing the absolute path.
- `new_path`: The new path for the file or directory. This should be a string representing the new absolute path.

## Returns
- Ok(): No content is returned. The function will rename the file or directory at the specified path.
- Err(String) - An error message if the file or directory cannot be renamed or other errors occur.

# `move_to_trash` endpoint

---
## Parameters
- `path`: The path to the file or directory to be moved to the trash. This should be a string representing the absolute path.

## Returns
- Ok(): No content is returned. The function will move the file or directory to the trash.
- Err(String) - An error message if the file or directory cannot be moved to the trash or other errors occur.
