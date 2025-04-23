use std::sync::{Arc, Mutex};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use md5::{Digest as Md5Digest, Md5 as Md5Hasher};
use sha2::{Sha256, Sha384, Sha512, Digest as Sha2Digest};
use crc32fast::Hasher;
use tokio::fs::File;
use tokio::fs;
use tokio::io::AsyncReadExt;
use std::path::{Path, PathBuf};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use tauri::State;
use crate::state::SettingsState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HashError {
    SettingsLockError,
    InvalidChecksumMethod,
    FileOperationError,
    ClipboardError,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum ChecksumMethod {
    MD5,
    SHA256,
    SHA384,
    SHA512,
    CRC32,
}

impl FromStr for ChecksumMethod {
    type Err = HashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MD5" => Ok(ChecksumMethod::MD5),
            "SHA256" => Ok(ChecksumMethod::SHA256),
            "SHA384" => Ok(ChecksumMethod::SHA384),
            "SHA512" => Ok(ChecksumMethod::SHA512),
            "CRC32" => Ok(ChecksumMethod::CRC32),
            _ => Err(HashError::InvalidChecksumMethod),
        }
    }
}

impl ToString for HashError {
    fn to_string(&self) -> String {
        match self {
            HashError::SettingsLockError => "Failed to access settings".to_string(),
            HashError::InvalidChecksumMethod => "Invalid checksum method".to_string(),
            HashError::FileOperationError => "File operation failed".to_string(),
            HashError::ClipboardError => "Failed to copy to clipboard".to_string(),
        }
    }
}

async fn get_checksum_method(state: Arc<Mutex<SettingsState>>) -> Result<ChecksumMethod, HashError> {
    let settings_state = state.lock().unwrap();
    let inner_settings = settings_state.0.lock().map_err(|_| HashError::SettingsLockError)?;
    Ok(inner_settings.default_checksum_hash.clone())
}

fn calculate_md5(data: &[u8]) -> String {
    let mut hasher = Md5Hasher::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn calculate_sha384(data: &[u8]) -> String {
    let mut hasher = Sha384::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn calculate_sha512(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn calculate_crc32(data: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data);
    let checksum = hasher.finalize();
    format!("{:08x}", checksum)
}

async fn calculate_hash(method: ChecksumMethod, data: &[u8]) -> Result<String, HashError> {
    let result = match method {
        ChecksumMethod::MD5 => calculate_md5(data),
        ChecksumMethod::SHA256 => calculate_sha256(data),
        ChecksumMethod::SHA384 => calculate_sha384(data),
        ChecksumMethod::SHA512 => calculate_sha512(data),
        ChecksumMethod::CRC32 => calculate_crc32(data),
    };
    Ok(result)
}

async fn get_files_in_dir(path: &Path) -> Result<Vec<PathBuf>, HashError> {
    let mut files = Vec::new();
    let mut dirs = vec![path.to_path_buf()];

    while let Some(dir) = dirs.pop() {
        let mut entries = fs::read_dir(&dir)
            .await
            .map_err(|_| HashError::FileOperationError)?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            } else {
                files.push(path);
            }
        }
    }

    files.sort();
    Ok(files)
}

fn combine_hashes(hashes: &[String]) -> String {
    if hashes.len() == 1 {
        return hashes[0].clone();
    }

    let combined = hashes.join("");
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn read_file(path: &Path) -> Result<Vec<u8>, HashError> {
    if path.is_dir() {
        let files = get_files_in_dir(path).await?;

        if files.is_empty() {
            return Ok(Vec::new());
        }

        let mut hashes = Vec::new();
        for file_path in files {
            let data = fs::read(&file_path)
                .await
                .map_err(|_| HashError::FileOperationError)?;
            let mut hasher = Sha256::new();
            hasher.update(&data);
            hashes.push(format!("{:x}", hasher.finalize()));
        }

        let combined_hash = combine_hashes(&hashes);
        Ok(combined_hash.into_bytes())
    } else {
        let mut file = File::open(path)
            .await
            .map_err(|_| HashError::FileOperationError)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .await
            .map_err(|_| HashError::FileOperationError)?;
        Ok(buffer)
    }
}

// Simplified clipboard trait
trait ClipboardOperations {
    async fn set_contents(&mut self, content: String) -> Result<(), HashError>;
}

struct RealClipboard;

impl ClipboardOperations for RealClipboard {
    async fn set_contents(&mut self, content: String) -> Result<(), HashError> {
        tokio::task::spawn_blocking(move || {
            ClipboardContext::new()
                .map_err(|_| HashError::ClipboardError)?
                .set_contents(content)
                .map_err(|_| HashError::ClipboardError)
        })
        .await
        .map_err(|_| HashError::ClipboardError)?
    }
}

// Simplified clipboard function that always uses real clipboard
async fn copy_to_clipboard(hash: &str) -> Result<(), HashError> {
    RealClipboard.set_contents(hash.to_string()).await
}

// Update the implementation function to use async clipboard
pub async fn gen_hash_and_copy_to_clipboard_impl(
    path: String,
    state: Arc<Mutex<SettingsState>>
) -> Result<String, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = read_file(Path::new(&path))
        .await
        .map_err(|e| e.to_string())?;
    let hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    copy_to_clipboard(&hash)
        .await
        .map_err(|e| e.to_string())?;

    Ok(hash)
}

/// Generates a hash for the given file and copies it to the clipboard.
/// The hash algorithm used is determined by the application settings (MD5, SHA256, SHA384, SHA512, or CRC32).
///
/// # Arguments
/// * `path` - A string representing the absolute path to the file to generate a hash for.
/// * `state` - The application's settings state containing the default hash algorithm.
///
/// # Returns
/// * `Ok(String)` - The generated hash value as a string. The hash is also copied to the clipboard.
/// * `Err(String)` - An error message if the hash cannot be generated or copied to clipboard.
///
/// # Example
/// ```rust
/// let result = gen_hash_and_copy_to_clipboard("/path/to/file", state).await;
/// match result {
///     Ok(hash) => println!("Generated hash: {}", hash),
///     Err(err) => println!("Error generating hash: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn gen_hash_and_copy_to_clipboard(
    path: String,
    state: State<'_, Arc<Mutex<SettingsState>>>
) -> Result<String, String> {
    gen_hash_and_copy_to_clipboard_impl(path, state.inner().clone()).await
}

pub async fn gen_hash_and_save_to_file_impl(
    source_path: String,
    output_path: String,
    state: Arc<Mutex<SettingsState>>
) -> Result<String, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = read_file(Path::new(&source_path))
        .await
        .map_err(|e| e.to_string())?;
    let hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    tokio::fs::write(output_path, hash.as_bytes())
        .await
        .map_err(|_| "Failed to write hash to file".to_string())?;

    Ok(hash)
}

/// Generates a hash for the given file and saves it to a specified output file.
/// The hash algorithm used is determined by the application settings.
///
/// # Arguments
/// * `source_path` - A string representing the absolute path to the file to generate a hash for.
/// * `output_path` - A string representing the absolute path where the hash will be saved.
/// * `state` - The application's settings state containing the default hash algorithm.
///
/// # Returns
/// * `Ok(String)` - The generated hash value as a string. The hash is also saved to the output file.
/// * `Err(String)` - An error message if the hash cannot be generated or saved.
///
/// # Example
/// ```rust
/// let result = gen_hash_and_save_to_file("/path/to/source", "/path/to/output", state).await;
/// match result {
///     Ok(hash) => println!("Generated and saved hash: {}", hash),
///     Err(err) => println!("Error generating/saving hash: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn gen_hash_and_save_to_file(
    source_path: String,
    output_path: String,
    state: State<'_, Arc<Mutex<SettingsState>>>
) -> Result<String, String> {
    gen_hash_and_save_to_file_impl(source_path, output_path, state.inner().clone()).await
}

pub async fn compare_file_or_dir_with_hash_impl(
    path: String,
    hash_to_compare: String,
    state: Arc<Mutex<SettingsState>>
) -> Result<bool, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = read_file(Path::new(&path))
        .await
        .map_err(|e| e.to_string())?;
    let calculated_hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    Ok(calculated_hash.eq_ignore_ascii_case(&hash_to_compare))
}

/// Compares a file's generated hash with a provided hash value.
/// The hash algorithm used is determined by the application settings.
///
/// # Arguments
/// * `path` - A string representing the absolute path to the file to check.
/// * `hash_to_compare` - A string representing the expected hash value to compare against.
/// * `state` - The application's settings state containing the default hash algorithm.
///
/// # Returns
/// * `Ok(bool)` - True if the generated hash matches the provided hash, false otherwise.
/// * `Err(String)` - An error message if the hash comparison cannot be performed.
///
/// # Example
/// ```rust
/// let result = compare_file_or_dir_with_hash("/path/to/file", "expected_hash", state).await;
/// match result {
///     Ok(matches) => println!("Hash comparison result: {}", matches),
///     Err(err) => println!("Error comparing hash: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn compare_file_or_dir_with_hash(
    path: String,
    hash_to_compare: String,
    state: State<'_, Arc<Mutex<SettingsState>>>
) -> Result<bool, String> {
    compare_file_or_dir_with_hash_impl(path, hash_to_compare, state.inner().clone()).await
}

#[cfg(test)]
mod tests_hash_commands {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use std::sync::Arc;
    use serde_json::json;
    use crate::state::SettingsState;

    // Mock clipboard for testing
    struct MockClipboard {
        content: Arc<Mutex<String>>,
    }

    impl MockClipboard {
        fn new() -> Self {
            MockClipboard {
                content: Arc::new(Mutex::new(String::new())),
            }
        }
    }

    impl ClipboardOperations for MockClipboard {
        async fn set_contents(&mut self, content: String) -> Result<(), HashError> {
            let mut guard = self.content.lock().unwrap();
            *guard = content;
            Ok(())
        }
    }

    // Test helper that uses mock clipboard
    async fn test_hash_with_mock_clipboard(
        file_path: &str,
        state: Arc<Mutex<SettingsState>>,
        clipboard: Arc<Mutex<String>>
    ) -> Result<String, String> {
        let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
        let data = read_file(Path::new(file_path))
            .await
            .map_err(|e| e.to_string())?;
        let hash = calculate_hash(checksum_method, &data)
            .await
            .map_err(|e| e.to_string())?;

        let mut guard = clipboard.lock().unwrap();
        *guard = hash.clone();

        Ok(hash)
    }

    // Testing: Helper function to create a test SettingsState
    fn create_test_settings_state() -> Arc<Mutex<SettingsState>> {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        // Create a settings state with a temporary file path
        Arc::new(Mutex::new(SettingsState::new_with_path(path)))
    }

    fn create_test_state(method: ChecksumMethod) -> Arc<Mutex<SettingsState>> {
        let state = create_test_settings_state();
        let state_guard = state.lock().unwrap();
        state_guard.update_setting_field("default_checksum_hash", json!(method)).unwrap();
        state.clone()
    }

    fn run_async_test<F: std::future::Future>(f: F) -> F::Output {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        rt.block_on(f)
    }

    #[test]
    fn test_save_hash_to_file() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("test_hash.txt");
            let hash_file_path = temp_dir.path().join("hash.txt");
            let test_content = b"Hello, world!";

            let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
            file.write_all(test_content).expect("Failed to write test content");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let state: Arc<Mutex<SettingsState>> = mock_state.clone();

            let result = gen_hash_and_save_to_file_impl(
                test_file_path.to_str().unwrap().to_string(),
                hash_file_path.to_str().unwrap().to_string(),
                state
            ).await;

            assert!(result.is_ok(), "Hash save failed");
            assert!(hash_file_path.exists(), "Hash file was not created");

            let hash_content = std::fs::read_to_string(hash_file_path).expect("Failed to read hash file");
            assert_eq!(hash_content, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
        })
    }

    #[test]
    fn test_compare_file_hash() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("test_hash.txt");
            let test_content = b"Hello, world!";

            let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
            file.write_all(test_content).expect("Failed to write test content");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let state: Arc<Mutex<SettingsState>> = mock_state.clone();

            let correct_hash = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
            let wrong_hash = "wronghashvalue";

            let result_correct = compare_file_or_dir_with_hash_impl(
                test_file_path.to_str().unwrap().to_string(),
                correct_hash.to_string(),
                state.clone()
            ).await;

            assert!(result_correct.is_ok(), "Hash comparison failed");
            assert!(result_correct.unwrap(), "Hash should match");

            let result_wrong = compare_file_or_dir_with_hash_impl(
                test_file_path.to_str().unwrap().to_string(),
                wrong_hash.to_string(),
                state
            ).await;

            assert!(result_wrong.is_ok(), "Hash comparison failed");
            assert!(!result_wrong.unwrap(), "Hash should not match");
        })
    }

    #[test]
    fn test_all_hash_methods() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("test_hash.txt");
            let test_content = b"Hello, world!";

            let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
            file.write_all(test_content).expect("Failed to write test content");

            let expected_hashes = vec![
                (ChecksumMethod::MD5, "6cd3556deb0da54bca060b4c39479839"),
                (ChecksumMethod::SHA256, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"),
                (ChecksumMethod::SHA384, "55bc556b0d2fe0fce582ba5fe07baafff035653638c7ac0d5494c2a64c0bea1cc57331c7c12a45cdbca7f4c34a089eeb"),
                (ChecksumMethod::SHA512, "c1527cd893c124773d811911970c8fe6e857d6df5dc9226bd8a160614c0cd963a4ddea2b94bb7d36021ef9d865d5cea294a82dd49a0bb269f51f6e7a57f79421"),
                (ChecksumMethod::CRC32, "ebe6c6e6"),
            ];

            for (method, expected_hash) in expected_hashes {
                let mock_state = create_test_state(method.clone());
                let clipboard_content = Arc::new(Mutex::new(String::new()));

                let result = test_hash_with_mock_clipboard(
                    test_file_path.to_str().unwrap(),
                    mock_state,
                    clipboard_content.clone()
                ).await;

                assert!(result.is_ok(), "Hash generation failed for {:?}", method);
                let hash = result.unwrap();
                assert_eq!(hash, expected_hash, "Hash mismatch for {:?}", method);

                let clipboard_value = clipboard_content.lock().unwrap();
                assert_eq!(*clipboard_value, expected_hash, "Clipboard content mismatch for {:?}", method);
            }
        })
    }

    #[test]
    fn test_hash_directory() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");

            let file1_path = temp_dir.path().join("file1.txt");
            let file2_path = temp_dir.path().join("file2.txt");
            let subdir_path = temp_dir.path().join("subdir");
            let file3_path = subdir_path.join("file3.txt");

            fs::create_dir_all(&subdir_path).await.expect("Failed to create subdirectory");
            fs::write(&file1_path, b"content1").await.expect("Failed to write file1");
            fs::write(&file2_path, b"content2").await.expect("Failed to write file2");
            fs::write(&file3_path, b"content3").await.expect("Failed to write file3");

            let mock_state = create_test_state(ChecksumMethod::SHA256);

            let single_dir = tempdir().expect("Failed to create single file directory");
            let single_file = single_dir.path().join("single.txt");
            fs::write(&single_file, b"content").await.expect("Failed to write single file");

            let single_result = gen_hash_and_copy_to_clipboard_impl(
                single_dir.path().to_str().unwrap().to_string(),
                mock_state.clone()
            ).await;
            assert!(single_result.is_ok());

            let result = gen_hash_and_copy_to_clipboard_impl(
                temp_dir.path().to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());

            let empty_dir = tempdir().expect("Failed to create empty directory");
            let empty_result = gen_hash_and_copy_to_clipboard_impl(
                empty_dir.path().to_str().unwrap().to_string(),
                create_test_state(ChecksumMethod::SHA256)
            ).await;

            assert!(empty_result.is_ok());
            assert_eq!(empty_result.unwrap(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        });
    }

    #[test]
    fn test_hash_directory_no_files() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let state: Arc<Mutex<SettingsState>> = mock_state.clone();

            let result = gen_hash_and_copy_to_clipboard_impl(
                temp_dir.path().to_str().unwrap().to_string(),
                state.clone()
            ).await;

            assert!(result.is_ok(), "Directory hash generation should succeed");
            assert_eq!(result.unwrap(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        })
    }

    #[test]
    fn test_non_existent_file() {
        run_async_test(async {
            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let non_existent_path = "not_a_real_file.txt";

            let result = gen_hash_and_copy_to_clipboard_impl(
                non_existent_path.to_string(),
                mock_state
            ).await;

            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), HashError::FileOperationError.to_string());
        })
    }

    #[test]
    fn test_empty_file() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("empty.txt");
            std::fs::File::create(&test_file_path).expect("Failed to create empty file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        })
    }

    #[test]
    fn test_invalid_hash_comparison() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("test.txt");
            std::fs::write(&test_file_path, b"test data").expect("Failed to write test file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);

            let result = compare_file_or_dir_with_hash_impl(
                test_file_path.to_str().unwrap().to_string(),
                "invalid_hash".to_string(),
                mock_state.clone()
            ).await;

            assert!(result.is_ok());
            assert!(!result.unwrap());
        })
    }

    #[test]
    fn test_special_chars_in_path() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("test with spaces & special chars!@#$.txt");
            std::fs::write(&test_file_path, b"test data").expect("Failed to write test file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
        })
    }

    #[test]
    fn test_binary_file_hash() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("binary.bin");
            let binary_data: Vec<u8> = (0..255).collect();
            std::fs::write(&test_file_path, &binary_data).expect("Failed to write binary file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_hash_method_switching() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("switch_test.txt");
            std::fs::write(&test_file_path, b"test data").expect("Failed to write test file");

            let state = create_test_settings_state();
            let file_path = test_file_path.to_str().unwrap().to_string();

            let methods = vec![
                ChecksumMethod::MD5,
                ChecksumMethod::SHA256,
                ChecksumMethod::SHA384,
                ChecksumMethod::SHA512,
                ChecksumMethod::CRC32,
            ];

            for method in methods {
                let state_guard = state.lock().unwrap();
                state_guard.update_setting_field("default_checksum_hash", json!(method.clone())).unwrap();
                drop(state_guard);

                let result = gen_hash_and_copy_to_clipboard_impl(
                    file_path.clone(),
                    state.clone()
                ).await;

                assert!(result.is_ok(), "Hash generation failed for {:?}", method);
            }
        });
    }

    #[test]
    fn test_unicode_content() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("unicode.txt");
            let unicode_content = "Hello, 世界! こんにちは! 🌍 👋";
            std::fs::write(&test_file_path, unicode_content).expect("Failed to write unicode file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_zero_byte_boundaries() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("zero_bytes.bin");
            let data = vec![0, 255, 0, 255, 0];
            std::fs::write(&test_file_path, &data).expect("Failed to write file");

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
        });
    }

    #[ignore = "Ignore for now"]
    #[test]
    fn test_sha256_large_known_vector() {
        run_async_test(async {
            let temp_dir = tempdir().expect("Failed to create temporary directory");
            let test_file_path = temp_dir.path().join("large_vector.txt");
            let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");

            let base_pattern = "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno";
            let repeat_count = 16_777_216;

            for _ in 0..repeat_count {
                file.write_all(base_pattern.as_bytes()).expect("Failed to write chunk");
            }

            let mock_state = create_test_state(ChecksumMethod::SHA256);
            let result = gen_hash_and_copy_to_clipboard_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state
            ).await;

            assert!(result.is_ok());
            assert_eq!(
                result.unwrap(),
                "50e72a0e26442fe2552dc3938ac58658228c0cbfb1d2ca872ae435266fcd055e"
            );
        });
    }
}
