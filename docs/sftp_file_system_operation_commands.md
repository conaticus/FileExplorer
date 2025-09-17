# Tauri SFTP Commands Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content

- [Load Directory](#load_dir-endpoint)
- [Open File](#open_file_sftp-endpoint)
- [Create File](#create_file_sftp-endpoint)
- [Delete File](#delete_file_sftp-endpoint)
- [Rename File](#rename_file_sftp-endpoint)
- [Copy File](#copy_file_sftp-endpoint)
- [Move File](#move_file_sftp-endpoint)
- [Create Directory](#create_directory_sftp-endpoint)
- [Delete Directory](#delete_directory_sftp-endpoint)
- [Rename Directory](#rename_directory_sftp-endpoint)
- [Copy Directory](#copy_directory_sftp-endpoint)
- [Move Directory](#move_directory_sftp-endpoint)
- [Build Preview](#build_preview_sftp-endpoint)
- [Download and Open File](#download_and_open_sftp_file-endpoint)
- [Cleanup SFTP Temp Files](#cleanup_sftp_temp_files-endpoint)

---

# `load_dir` endpoint

Lists the contents of a directory on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `directory`: String - The directory path to list (use "." for current directory)

## Returns

- Ok(String) - JSON string containing the directory structure with files and subdirectories
- Err(String) - An error message if connection fails, authentication fails, or directory doesn't exist

## Example call

```typescript jsx
useEffect(() => {
  const loadDirectory = async () => {
    try {
      const result = await invoke("load_dir", {
        host: "localhost",
        port: 2222,
        username: "explorer",
        password: "explorer",
        directory: "."
      });
      const directoryData = JSON.parse(result);
      console.log("Directory contents:", directoryData);
    } catch (error) {
      console.error("Error loading directory:", error);
    }
  };

  loadDirectory();
}, []);
```

---

# `open_file_sftp` endpoint

Reads the contents of a file from the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `file_path`: String - The path to the file to read

## Returns

- Ok(String) - The contents of the file as a string
- Err(String) - An error message if connection fails, authentication fails, or file doesn't exist

## Example call

```typescript jsx
const readFile = async () => {
  try {
    const content = await invoke("open_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      file_path: "example.txt"
    });
    console.log("File content:", content);
  } catch (error) {
    console.error("Error reading file:", error);
  }
};
```

---

# `create_file_sftp` endpoint

Creates a new empty file on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `file_path`: String - The path where the new file should be created

## Returns

- Ok(String) - Success message with the file path
- Err(String) - An error message if connection fails, authentication fails, or file creation fails

## Example call

```typescript jsx
const createFile = async () => {
  try {
    const result = await invoke("create_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      file_path: "new_file.txt"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error creating file:", error);
  }
};
```

---

# `delete_file_sftp` endpoint

Deletes a file from the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `file_path`: String - The path to the file to delete

## Returns

- Ok(String) - Success message with the deleted file path
- Err(String) - An error message if connection fails, authentication fails, or file doesn't exist

## Example call

```typescript jsx
const deleteFile = async () => {
  try {
    const result = await invoke("delete_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      file_path: "file_to_delete.txt"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error deleting file:", error);
  }
};
```

---

# `rename_file_sftp` endpoint

Renames a file on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `old_path`: String - The current path of the file
- `new_path`: String - The new path/name for the file

## Returns

- Ok(String) - Success message with old and new paths
- Err(String) - An error message if connection fails, authentication fails, or file doesn't exist

## Example call

```typescript jsx
const renameFile = async () => {
  try {
    const result = await invoke("rename_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      old_path: "old_name.txt",
      new_path: "new_name.txt"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error renaming file:", error);
  }
};
```

---

# `copy_file_sftp` endpoint

Copies a file on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `source_path`: String - The path to the source file
- `destination_path`: String - The path where the file should be copied

## Returns

- Ok(String) - Success message with source and destination paths
- Err(String) - An error message if connection fails, authentication fails, or source file doesn't exist

## Example call

```typescript jsx
const copyFile = async () => {
  try {
    const result = await invoke("copy_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      source_path: "source.txt",
      destination_path: "copy.txt"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error copying file:", error);
  }
};
```

---

# `move_file_sftp` endpoint

Moves a file on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `source_path`: String - The current path of the file
- `destination_path`: String - The new path for the file

## Returns

- Ok(String) - Success message with source and destination paths
- Err(String) - An error message if connection fails, authentication fails, or source file doesn't exist

## Example call

```typescript jsx
const moveFile = async () => {
  try {
    const result = await invoke("move_file_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      source_path: "file.txt",
      destination_path: "moved/file.txt"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error moving file:", error);
  }
};
```

---

# `create_directory_sftp` endpoint

Creates a new directory on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `directory_path`: String - The path where the new directory should be created

## Returns

- Ok(String) - Success message with the directory path
- Err(String) - An error message if connection fails, authentication fails, or directory creation fails

## Example call

```typescript jsx
const createDirectory = async () => {
  try {
    const result = await invoke("create_directory_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      directory_path: "new_folder"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error creating directory:", error);
  }
};
```

---

# `delete_directory_sftp` endpoint

Deletes an empty directory from the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `directory_path`: String - The path to the directory to delete

## Returns

- Ok(String) - Success message with the deleted directory path
- Err(String) - An error message if connection fails, authentication fails, directory doesn't exist, or directory is not empty

## Example call

```typescript jsx
const deleteDirectory = async () => {
  try {
    const result = await invoke("delete_directory_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      directory_path: "folder_to_delete"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error deleting directory:", error);
  }
};
```

---

# `rename_directory_sftp` endpoint

Renames a directory on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `old_path`: String - The current path of the directory
- `new_path`: String - The new path/name for the directory

## Returns

- Ok(String) - Success message with old and new paths
- Err(String) - An error message if connection fails, authentication fails, or directory doesn't exist

## Example call

```typescript jsx
const renameDirectory = async () => {
  try {
    const result = await invoke("rename_directory_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      old_path: "old_folder",
      new_path: "new_folder"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error renaming directory:", error);
  }
};
```

---

# `copy_directory_sftp` endpoint

Recursively copies a directory and its contents on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `source_path`: String - The path to the source directory
- `destination_path`: String - The path where the directory should be copied

## Returns

- Ok(String) - Success message with source and destination paths
- Err(String) - An error message if connection fails, authentication fails, or source directory doesn't exist

## Example call

```typescript jsx
const copyDirectory = async () => {
  try {
    const result = await invoke("copy_directory_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      source_path: "source_folder",
      destination_path: "copied_folder"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error copying directory:", error);
  }
};
```

---

# `move_directory_sftp` endpoint

Moves a directory on the SFTP server.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `source_path`: String - The current path of the directory
- `destination_path`: String - The new path for the directory

## Returns

- Ok(String) - Success message with source and destination paths
- Err(String) - An error message if connection fails, authentication fails, or source directory doesn't exist

## Example call

```typescript jsx
const moveDirectory = async () => {
  try {
    const result = await invoke("move_directory_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      source_path: "folder",
      destination_path: "moved/folder"
    });
    console.log("Success:", result);
  } catch (error) {
    console.error("Error moving directory:", error);
  }
};
```

---

# `build_preview_sftp` endpoint

Generates a preview payload for a file or directory on the SFTP server, including type detection and metadata.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `file_path`: String - The path to the file or directory to preview

## Returns

- Ok(PreviewPayload) - A JSON object describing the preview (text, image, pdf, folder, or unknown)
- Err(String) - An error message if connection fails, authentication fails, or file/directory doesn't exist

## Example call

```typescript jsx
const preview = async () => {
  try {
    const result = await invoke("build_preview_sftp", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      file_path: "example.txt"
    });
    console.log("Preview payload:", result);
  } catch (error) {
    console.error("Error building preview:", error);
  }
};
```

---

# `download_and_open_sftp_file` endpoint

Downloads a file from the SFTP server to a temporary local directory and optionally opens it with the default application.

## Parameters

- `host`: String - The SFTP server hostname or IP address
- `port`: u16 - The SFTP server port (typically 22)
- `username`: String - The username for authentication
- `password`: String - The password for authentication
- `file_path`: String - The path to the file to download
- `open_file`: Option<bool> - Whether to open the file after downloading (default: true)

## Returns

- Ok(String) - The local path to the downloaded file, or a message indicating it was opened
- Err(String) - An error message if connection fails, authentication fails, or file doesn't exist

## Example call

```typescript jsx
const downloadAndOpen = async () => {
  try {
    const result = await invoke("download_and_open_sftp_file", {
      host: "localhost",
      port: 2222,
      username: "explorer",
      password: "explorer",
      file_path: "example.txt",
      open_file: true
    });
    console.log("Download result:", result);
  } catch (error) {
    console.error("Error downloading file:", error);
  }
};
```

---

# `cleanup_sftp_temp_files` endpoint

Removes temporary files downloaded from the SFTP server that are older than 24 hours from the local temp directory.

## Parameters

- None

## Returns

- Ok(String) - A message indicating how many files were cleaned
- Err(String) - An error message if the temp directory cannot be read or cleaned

## Example call

```typescript jsx
const cleanupTempFiles = async () => {
  try {
    const result = await invoke("cleanup_sftp_temp_files");
    console.log("Cleanup result:", result);
  } catch (error) {
    console.error("Error cleaning up temp files:", error);
  }
};
```

---

## Notes

- All SFTP commands require valid connection credentials (host, port, username, password)
- The default SFTP port is typically 22, but can vary depending on server configuration
- File and directory paths are relative to the user's home directory on the SFTP server
- For directory operations like copy, the operation is recursive and will include all subdirectories and files
- Delete directory only works on empty directories - use recursive deletion if needed
- All commands return descriptive error messages for troubleshooting connection and operation issues
