use crate::state::SettingsState;
use crc32fast::Hasher;
use md5::{Digest as Md5Digest, Md5 as Md5Hasher};
use serde::{Deserialize, Serialize};
use sha2::{Digest as Sha2Digest, Sha256, Sha384, Sha512};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::State;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

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

async fn get_checksum_method(
    state: Arc<Mutex<SettingsState>>,
) -> Result<ChecksumMethod, HashError> {
    let settings_state = state.lock().unwrap();
    let inner_settings = settings_state
        .0
        .lock()
        .map_err(|_| HashError::SettingsLockError)?;
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

async fn read_file(path: &Path) -> Result<Vec<u8>, HashError> {
    if !path.exists() && path.is_dir() {
        return Err(HashError::FileOperationError);
    }
    let mut file = File::open(path)
        .await
        .map_err(|_| HashError::FileOperationError)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| HashError::FileOperationError)?;
    Ok(buffer)
}

pub async fn gen_hash_and_return_string_impl(
    path: String,
    state: Arc<Mutex<SettingsState>>,
) -> Result<String, String> {
    let checksum_method = get_checksum_method(state)
        .await
        .map_err(|e| e.to_string())?;
    let data = read_file(Path::new(&path))
        .await
        .map_err(|e| e.to_string())?;
    let hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    Ok(hash)
}

/// Generates a hash for the given file and returns it as a string.
/// The hash algorithm used is determined by the application settings (MD5, SHA256, SHA384, SHA512, or CRC32).
///
/// # Arguments
/// * `path` - A string representing the absolute path to the file to generate a hash for.
/// * `state` - The application's settings state containing the default hash algorithm.
///
/// # Returns
/// * `Ok(String)` - The generated hash value as a string.
/// * `Err(String)` - An error message if the hash cannot be generated.
///
/// # Example
/// ```rust
/// let result = gen_hash_and_return_string("/path/to/file", state).await;
/// match result {
///     Ok(hash) => println!("Generated hash: {}", hash),
///     Err(err) => println!("Error generating hash: {}", err),
/// }
/// ```
#[tauri::command]
pub async fn gen_hash_and_return_string(
    path: String,
    state: State<'_, Arc<Mutex<SettingsState>>>,
) -> Result<String, String> {
    gen_hash_and_return_string_impl(path, state.inner().clone()).await
}

pub async fn gen_hash_and_save_to_file_impl(
    source_path: String,
    output_path: String,
    state: Arc<Mutex<SettingsState>>,
) -> Result<String, String> {
    let checksum_method = get_checksum_method(state)
        .await
        .map_err(|e| e.to_string())?;
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
    state: State<'_, Arc<Mutex<SettingsState>>>,
) -> Result<String, String> {
    gen_hash_and_save_to_file_impl(source_path, output_path, state.inner().clone()).await
}

pub async fn compare_file_or_dir_with_hash_impl(
    path: String,
    hash_to_compare: String,
    state: Arc<Mutex<SettingsState>>,
) -> Result<bool, String> {
    let checksum_method = get_checksum_method(state)
        .await
        .map_err(|e| e.to_string())?;
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
    state: State<'_, Arc<Mutex<SettingsState>>>,
) -> Result<bool, String> {
    compare_file_or_dir_with_hash_impl(path, hash_to_compare, state.inner().clone()).await
}

#[cfg(test)]
mod tests_hash_commands {
    use super::*;
    use crate::state::SettingsState;
    use serde_json::json;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::tempdir;

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
        state_guard
            .update_setting_field("default_checksum_hash", json!(method))
            .unwrap();
        state.clone()
    }

    #[tokio::test]
    async fn test_save_hash_to_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let hash_file_path = temp_dir.path().join("hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content)
            .expect("Failed to write test content");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let state: Arc<Mutex<SettingsState>> = mock_state.clone();

        let result = gen_hash_and_save_to_file_impl(
            test_file_path.to_str().unwrap().to_string(),
            hash_file_path.to_str().unwrap().to_string(),
            state,
        )
        .await;

        assert!(result.is_ok(), "Hash save failed");
        assert!(hash_file_path.exists(), "Hash file was not created");

        let hash_content =
            std::fs::read_to_string(hash_file_path).expect("Failed to read hash file");
        assert_eq!(
            hash_content,
            "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        );
    }

    #[tokio::test]
    async fn test_compare_file_hash() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content)
            .expect("Failed to write test content");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let state: Arc<Mutex<SettingsState>> = mock_state.clone();

        let correct_hash = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let wrong_hash = "wronghashvalue";

        let result_correct = compare_file_or_dir_with_hash_impl(
            test_file_path.to_str().unwrap().to_string(),
            correct_hash.to_string(),
            state.clone(),
        )
        .await;

        assert!(result_correct.is_ok(), "Hash comparison failed");
        assert!(result_correct.unwrap(), "Hash should match");

        let result_wrong = compare_file_or_dir_with_hash_impl(
            test_file_path.to_str().unwrap().to_string(),
            wrong_hash.to_string(),
            state,
        )
        .await;

        assert!(result_wrong.is_ok(), "Hash comparison failed");
        assert!(!result_wrong.unwrap(), "Hash should not match");
    }

    #[tokio::test]
    async fn test_all_hash_methods() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content)
            .expect("Failed to write test content");

        let expected_hashes = vec![
                (ChecksumMethod::MD5, "6cd3556deb0da54bca060b4c39479839"),
                (ChecksumMethod::SHA256, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"),
                (ChecksumMethod::SHA384, "55bc556b0d2fe0fce582ba5fe07baafff035653638c7ac0d5494c2a64c0bea1cc57331c7c12a45cdbca7f4c34a089eeb"),
                (ChecksumMethod::SHA512, "c1527cd893c124773d811911970c8fe6e857d6df5dc9226bd8a160614c0cd963a4ddea2b94bb7d36021ef9d865d5cea294a82dd49a0bb269f51f6e7a57f79421"),
                (ChecksumMethod::CRC32, "ebe6c6e6"),
            ];

        for (method, expected_hash) in expected_hashes {
            let mock_state = create_test_state(method.clone());

            let result = gen_hash_and_return_string_impl(
                test_file_path.to_str().unwrap().to_string(),
                mock_state,
            )
            .await;

            assert!(result.is_ok(), "Hash generation failed for {:?}", method);
            let hash = result.unwrap();
            assert_eq!(hash, expected_hash, "Hash mismatch for {:?}", method);
        }
    }

    #[tokio::test]
    async fn test_non_existent_file() {
        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let non_existent_path = "not_a_real_file.txt";

        let result =
            gen_hash_and_return_string_impl(non_existent_path.to_string(), mock_state).await;

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            HashError::FileOperationError.to_string()
        );
    }

    #[tokio::test]
    async fn test_empty_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("empty.txt");
        std::fs::File::create(&test_file_path).expect("Failed to create empty file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[tokio::test]
    async fn test_invalid_hash_comparison() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test.txt");
        std::fs::write(&test_file_path, b"test data").expect("Failed to write test file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);

        let result = compare_file_or_dir_with_hash_impl(
            test_file_path.to_str().unwrap().to_string(),
            "invalid_hash".to_string(),
            mock_state.clone(),
        )
        .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_special_chars_in_path() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir
            .path()
            .join("test with spaces & special chars!@#$.txt");
        std::fs::write(&test_file_path, b"test data").expect("Failed to write test file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_binary_file_hash() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("binary.bin");
        let binary_data: Vec<u8> = (0..255).collect();
        std::fs::write(&test_file_path, &binary_data).expect("Failed to write binary file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_hash_method_switching() {
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
            state_guard
                .update_setting_field("default_checksum_hash", json!(method.clone()))
                .unwrap();
            drop(state_guard);

            let result =
                gen_hash_and_return_string_impl(file_path.clone(), state.clone()).await;

            assert!(result.is_ok(), "Hash generation failed for {:?}", method);
        }
    }

    #[tokio::test]
    async fn test_unicode_content() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("unicode.txt");
        let unicode_content = "Hello, 世界! こんにちは! 🌍 👋";
        std::fs::write(&test_file_path, unicode_content).expect("Failed to write unicode file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_zero_byte_boundaries() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("zero_bytes.bin");
        let data = vec![0, 255, 0, 255, 0];
        std::fs::write(&test_file_path, &data).expect("Failed to write file");

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
    }

    #[cfg(feature = "long-tests")]
    #[tokio::test]
    async fn test_sha256_large_known_vector() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("large_vector.txt");
        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");

        let base_pattern = "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmno";
        let repeat_count = 16_777_216;

        for _ in 0..repeat_count {
            file.write_all(base_pattern.as_bytes())
                .expect("Failed to write chunk");
        }

        let mock_state = create_test_state(ChecksumMethod::SHA256);
        let result = gen_hash_and_return_string_impl(
            test_file_path.to_str().unwrap().to_string(),
            mock_state,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "50e72a0e26442fe2552dc3938ac58658228c0cbfb1d2ca872ae435266fcd055e"
        );
    }
}
