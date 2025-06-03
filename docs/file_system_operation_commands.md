# Tauri Filesystem Commands Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content

- [Open a File](#open_file-endpoint)
- [Create a File](#create_file-endpoint)
- [Open a Directory](#open_directory-endpoint)
- [Create a Directory](#create_directory-endpoint)
- [Rename a Dir or File](#rename-endpoint)
- [Move a Dir or File to trash](#move_to_trash-endpoint)
- [Zip a Dir or File](#zip-endpoint)
- [Unzip a Dir or File](#unzip-endpoint)

# `open_file` endpoint CURRENTLY NOT ACTIVE

---

## Parameters

- `file_path`: The path to the file to be opened. This should be a string representing the absolute
  path to the file.

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

# `open_in_default_app`

---

## Parameters

- `path`: The path to the file to be opened. This should be a string representing the absolute
  path to the file.

## Returns

- Ok(): No content is returned. The function simply opens the file in the default application.
- Err(): An error message with what went wrong.

# `create_file` endpoint

---

- `file_path`: The path to the file to be opened. This should be a string representing the absolute
  path to the file.

## Parameters

- `folder_path_abs`: The absolute path to the folder where the file will be created.
- `file_name`: The name of the file to be created. This should be a string representing the name of
  the file.

## Returns

- Ok(): No content is returned. The function will create a file at the specified path.
- Err(String) - An error message if the file cannot be created or other errors occur.

# `open_directory` endpoint

---

- `path`: The path to the directory to be opened. This should be a string representing the absolute
  path to the directory.

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
- `directory_name`: The name of the directory to be created. This should be a string representing
  the name of the directory.

## Returns

- Ok(): No content is returned. The function will create a directory at the specified path.
- Err(String) - An error message if the directory cannot be created or other errors occur.

# `rename` endpoint

---

## Parameters

- `old_path`: The current path of the file or directory to be renamed. This should be a string
  representing the absolute path.
- `new_path`: The new path for the file or directory. This should be a string representing the new
  absolute path.

## Returns

- Ok(): No content is returned. The function will rename the file or directory at the specified
  path.
- Err(String) - An error message if the file or directory cannot be renamed or other errors occur.

# `move_to_trash` endpoint

---

## Parameters

- `path`: The path to the file or directory to be moved to the trash. This should be a string
  representing the absolute path.

## Returns

- Ok(): No content is returned. The function will move the file or directory to the trash.
- Err(String) - An error message if the file or directory cannot be moved to the trash or other
  errors occur.

# `zip` endpoint

---

## Parameters

- `source_path(s)`: An array of paths to files and/or directories to be zipped. Each path should be
  a string representing the absolute path.
- `destination_path`: An optional destination path for the zip file. Required when zipping multiple
  files/directories. When not provided for a single source, creates a zip with the same name as the
  source.

## Returns

- Ok(): No content is returned. The function will create a zip file at the specified or default
  location.
- Err(String) - An error message if the zip operation fails.

## Description

Creates a zip archive from one or more files/directories. For a single source with no destination
specified, creates a zip file at the same location with the same name. When zipping multiple sources
or when specifying a destination, creates the zip at the specified location. All directory contents
including subdirectories are included in the zip.

## Example call

```typescript jsx
useEffect(() => {
  const zipFiles = async () => {
    try {
      // Single file with auto destination
      await invoke("zip", {
        source_paths: ["/path/to/file"],
        destination_path: null,
      });

      // Multiple files with specified destination
      await invoke("zip", {
        source_paths: ["/path/to/file1", "/path/to/dir1"],
        destination_path: "/path/to/archive.zip",
      });
    } catch (error) {
      console.error("Error creating zip:", error);
    }
  };

  zipFiles();
}, []);
```

# `unzip` endpoint

---

## Parameters

- `zip_path(s)`: An array of paths to zip files to be extracted. Each path should be a string
  representing the absolute path.
- `destination_path`: An optional destination directory for extraction. Required when extracting
  multiple zips. When not provided for a single zip, extracts to a directory with the same name as
  the zip file (without .zip extension).

## Returns

- Ok(): No content is returned. The function will extract all zip files to the specified or default
  location.
- Err(String) - An error message if any extraction fails.

## Description

Extracts one or more zip files. For a single zip without a specified destination, creates a
directory with the same name as the zip file (without .zip extension) and extracts contents there.
When extracting multiple zips or specifying a destination, creates subdirectories for each zip under
the destination path using the zip filenames. Preserves the internal directory structure of the zip
files.

## Example call

```typescript jsx
useEffect(() => {
  const unzip = async () => {
    try {
      // Single zip with auto destination
      await invoke("unzip", {
        zip_paths: ["/path/to/archive.zip"],
        destination_path: null,
      });

      // Multiple zips with specified destination
      await invoke("unzip", {
        zip_paths: ["/path/to/archive1.zip", "/path/to/archive2.zip"],
        destination_path: "/path/to/extract",
      });
    } catch (error) {
      console.error("Error extracting zips:", error);
    }
  };

  unzip();
}, []);
```
