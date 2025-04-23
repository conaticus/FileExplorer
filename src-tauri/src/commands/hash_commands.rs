use std::sync::Arc;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use md5::{Digest as Md5Digest, Md5 as Md5Hasher};
use sha2::{Sha256, Sha384, Sha512, Digest as Sha2Digest};
use crc32fast::Hasher;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use std::path::Path;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

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

async fn get_checksum_method(state: tauri::State<'_, Arc<Mutex<SettingsState>>>) -> Result<ChecksumMethod, HashError> {
    let settings_state = state.lock().await;
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

async fn read_file(path: &Path) -> Result<Vec<u8>, HashError> {
    let mut file = File::open(path)
        .await
        .map_err(|_| HashError::FileOperationError)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| HashError::FileOperationError)?;
    Ok(buffer)
}

#[tauri::command]
pub async fn gen_hash_and_copy_to_clipboard(
    path: String,
    state: tauri::State<'_, Arc<Mutex<SettingsState>>>
) -> Result<String, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = read_file(Path::new(&path))
        .await
        .map_err(|e| e.to_string())?;
    let hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    // Copy hash to clipboard
    let mut ctx = ClipboardContext::new()
        .map_err(|_| HashError::ClipboardError.to_string())?;
    ctx.set_contents(hash.clone())
        .map_err(|_| HashError::ClipboardError.to_string())?;

    Ok(hash)
}

#[tauri::command]
pub async fn gen_hash_and_save_to_file(
    source_path: String,
    output_path: String,
    state: tauri::State<'_, Arc<Mutex<SettingsState>>>
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

#[tauri::command]
pub async fn compare_file_or_dir_with_hash(
    path: String,
    hash_to_compare: String,
    state: tauri::State<'_, Arc<Mutex<SettingsState>>>
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

#[cfg(test)]
mod tests_hash_commands {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;
    use std::sync::Arc;
    use serde_json::json;
    use tokio::sync::Mutex;
    use crate::commands::settings_commands::update_settings_field_impl;
    use crate::state::{Settings, SettingsState};

    fn create_test_settings_state() -> Arc<Mutex<SettingsState>> {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        Arc::new(Mutex::new(SettingsState::new_with_path(path)))
    }

    async fn create_test_state(method: ChecksumMethod) -> Arc<Mutex<SettingsState>> {
        let state = create_test_settings_state();
        let state_clone = state.clone();
        let state_guard = state.lock().await;
        state_guard.update_setting_field("default_checksum_hash", json!(method)).unwrap();
        drop(state_guard);
        state_clone
    }

    #[tokio::test]
    async fn test_hash_file_md5() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::MD5).await;

        let result = gen_hash_and_copy_to_clipboard(
            test_file_path.to_str().unwrap().to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result.is_ok(), "Hash generation failed");
        assert_eq!(result.unwrap(), "6cd3556deb0da54bca060b4c39479839");
    }

    #[tokio::test]
    async fn test_hash_file_sha256() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::SHA256).await;

        let result = gen_hash_and_copy_to_clipboard(
            test_file_path.to_str().unwrap().to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result.is_ok(), "Hash generation failed");
        assert_eq!(result.unwrap(), "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
    }

    #[tokio::test]
    async fn test_save_hash_to_file() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let hash_file_path = temp_dir.path().join("hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::SHA256).await;

        let result = gen_hash_and_save_to_file(
            test_file_path.to_str().unwrap().to_string(),
            hash_file_path.to_str().unwrap().to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result.is_ok(), "Hash save failed");
        assert!(hash_file_path.exists(), "Hash file was not created");

        let hash_content = std::fs::read_to_string(hash_file_path).expect("Failed to read hash file");
        assert_eq!(hash_content, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");
    }

    #[tokio::test]
    async fn test_compare_file_hash() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::SHA256).await;

        let correct_hash = "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3";
        let wrong_hash = "wronghashvalue";

        let result_correct = compare_file_or_dir_with_hash(
            test_file_path.to_str().unwrap().to_string(),
            correct_hash.to_string(),
            tauri::State::new(state.clone())
        ).await;

        assert!(result_correct.is_ok(), "Hash comparison failed");
        assert!(result_correct.unwrap(), "Hash should match");

        let result_wrong = compare_file_or_dir_with_hash(
            test_file_path.to_str().unwrap().to_string(),
            wrong_hash.to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result_wrong.is_ok(), "Hash comparison failed");
        assert!(!result_wrong.unwrap(), "Hash should not match");
    }

    #[tokio::test]
    async fn test_all_hash_methods() {
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
            let state = create_test_state(method.clone()).await;

            let result = gen_hash_and_copy_to_clipboard(
                test_file_path.to_str().unwrap().to_string(),
                tauri::State::new(state)
            ).await;

            assert!(result.is_ok(), "Hash generation failed for {:?}", method);
            assert_eq!(result.unwrap(), expected_hash, "Hash mismatch for {:?}", method);
        }
    }

    #[tokio::test]
    async fn test_hash_file_md5_and_clipboard() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::MD5).await;

        let result = gen_hash_and_copy_to_clipboard(
            test_file_path.to_str().unwrap().to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result.is_ok(), "Hash generation failed");
        let hash = result.unwrap();
        assert_eq!(hash, "6cd3556deb0da54bca060b4c39479839");

        // Verify clipboard contents
        let mut ctx = ClipboardContext::new().expect("Failed to create clipboard context");
        let clipboard_content = ctx.get_contents().expect("Failed to get clipboard contents");
        assert_eq!(clipboard_content, hash, "Clipboard content should match generated hash");
    }

    #[tokio::test]
    async fn test_hash_file_sha256_and_clipboard() {
        let temp_dir = tempdir().expect("Failed to create temporary directory");
        let test_file_path = temp_dir.path().join("test_hash.txt");
        let test_content = b"Hello, world!";

        let mut file = std::fs::File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(test_content).expect("Failed to write test content");

        let state = create_test_state(ChecksumMethod::SHA256).await;

        let result = gen_hash_and_copy_to_clipboard(
            test_file_path.to_str().unwrap().to_string(),
            tauri::State::new(state)
        ).await;

        assert!(result.is_ok(), "Hash generation failed");
        let hash = result.unwrap();
        assert_eq!(hash, "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3");

        // Verify clipboard contents
        let mut ctx = ClipboardContext::new().expect("Failed to create clipboard context");
        let clipboard_content = ctx.get_contents().expect("Failed to get clipboard contents");
        assert_eq!(clipboard_content, hash, "Clipboard content should match generated hash");
    }
}
