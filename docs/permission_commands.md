# Tauri Permission Commands Documentation

Error Structure as json can be found [here](./error_structure.md).

## Content

- [Request Full Disk Access](#request_full_disk_access-endpoint)
- [Check Directory Access](#check_directory_access-endpoint)

---

# `request_full_disk_access` endpoint

Requests the user to grant "Full Disk Access" permissions to the application (macOS only).

## Parameters

- None

## Returns

- Ok(()) - If the System Preferences window was successfully opened (macOS), or if not on macOS
- Err(String) - An error message if the System Preferences could not be opened

## Example call

```typescript jsx
const requestFullDiskAccess = async () => {
  try {
    await invoke("request_full_disk_access");
    console.log("Requested full disk access.");
  } catch (error) {
    console.error("Error requesting full disk access:", error);
  }
};
```

---

# `check_directory_access` endpoint

Checks if the application can access the contents of a given directory.

## Parameters

- `path`: String - The directory path to check

## Returns

- Ok(true) - If the directory is accessible
- Ok(false) - If the directory is not accessible
- Err(String) - An error message if the check fails

## Example call

```typescript jsx
const checkAccess = async (path) => {
  try {
    const hasAccess = await invoke("check_directory_access", { path });
    console.log("Directory access:", hasAccess);
  } catch (error) {
    console.error("Error checking directory access:", error);
  }
};
```

---

## Notes

- `request_full_disk_access` only performs an action on macOS. On other platforms, it is a no-op.
- `check_directory_access` returns a boolean indicating access, not the directory contents.

