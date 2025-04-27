# Tauri Hash Commands Documentation

## Content
- [Generate Hash and Return String](#gen_hash_and_return_string-endpoint)
- [Generate Hash and Save to File](#gen_hash_and_save_to_file-endpoint)
- [Compare File with Hash](#compare_file_or_dir_with_hash-endpoint)

# `gen_hash_and_return_string` endpoint

---
## Parameters
- `path`: The path to the file to generate a hash for. This should be a string representing the absolute path to the file.

## Returns
- Ok(String) - The generated hash value as a string.
- Err(String) - An error message if the hash cannot be generated or other errors occur.

## Example call
```typescript jsx
useEffect(() => {
    const generateHash = async () => {
        try {
            const hash = await invoke("gen_hash_and_return_string", { path: "/path/to/file" });
            console.log("Generated hash:", hash);
        } catch (error) {
            console.error("Error generating hash:", error);
        }
    };

    generateHash();
}, []);
```

# `gen_hash_and_save_to_file` endpoint

---
## Parameters
- `source_path`: The path to the file to generate a hash for. This should be a string representing the absolute path to the file.
- `output_path`: The path where the hash should be saved. This should be a string representing the absolute path to the output file.

## Returns
- Ok(String) - The generated hash value as a string. The hash will also be saved to the specified output file.
- Err(String) - An error message if the hash cannot be generated or saved, or other errors occur.

## Example call
```typescript jsx
useEffect(() => {
    const generateAndSaveHash = async () => {
        try {
            const hash = await invoke("gen_hash_and_save_to_file", { 
                source_path: "/path/to/source/file",
                output_path: "/path/to/output/hash.txt"
            });
            console.log("Generated and saved hash:", hash);
        } catch (error) {
            console.error("Error generating/saving hash:", error);
        }
    };

    generateAndSaveHash();
}, []);
```

# `compare_file_or_dir_with_hash` endpoint

---
## Parameters
- `path`: The path to the file to check. This should be a string representing the absolute path to the file.
- `hash_to_compare`: The hash value to compare against. This should be a string representing the expected hash.

## Returns
- Ok(bool) - A boolean indicating whether the generated hash matches the provided hash (true for match, false for mismatch).
- Err(String) - An error message if the comparison cannot be performed or other errors occur.

## Example call
```typescript jsx
useEffect(() => {
    const compareHash = async () => {
        try {
            const matches = await invoke("compare_file_or_dir_with_hash", { 
                path: "/path/to/file",
                hash_to_compare: "expected_hash_value"
            });
            console.log("Hash comparison result:", matches);
        } catch (error) {
            console.error("Error comparing hash:", error);
        }
    };

    compareHash();
}, []);
```

## Notes
- All hash operations use the default hash method configured in the application settings (MD5, SHA256, SHA384, SHA512, or CRC32).
- Hash comparisons are case-insensitive.
- Impl copy to clipboard in frontend
