use std::sync::Arc;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use md5::{Md5, Digest};
use sha2::{Sha256, Sha512};
use crc32fast::Hasher;

use crate::state::SettingsState;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum HashError {
    SettingsLockError,
    InvalidChecksumMethod,
    FileOperationError,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ChecksumMethod {
    MD5,
    SHA256,
    SHA384,
    SHA512,
    SHA512_256,
    CRC32,
}

impl FromStr for ChecksumMethod {
    type Err = HashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "MD5" => Ok(ChecksumMethod::MD5),
            "SHA256" => Ok(ChecksumMethod::SHA256),
            "SHA512" => Ok(ChecksumMethod::SHA512),
            "SHA512_256" => Ok(ChecksumMethod::SHA512_256),
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
        }
    }
}

async fn get_checksum_method(state: Arc<Mutex<SettingsState>>) -> Result<ChecksumMethod, HashError> {
    let settings = state.lock().map_err(|_| HashError::SettingsLockError)?;
    let checksum = &settings.default_checksum_hash;
    ChecksumMethod::from_str(checksum)
}

fn calculate_md5(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
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
        ChecksumMethod::SHA512 => calculate_sha512(data),
        ChecksumMethod::SHA512_256 => calculate_sha512_256(data),
        ChecksumMethod::CRC32 => calculate_crc32(data),
    };
    Ok(result)
}

#[tauri::command]
pub async fn gen_hash_and_copy_to_clipboard(state: Arc<Mutex<SettingsState>>) -> Result<String, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    // Assuming you'll read file data into a buffer
    let data = Vec::new(); // Replace with actual file reading
    calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn gen_hash_and_save_to_file(state: Arc<Mutex<SettingsState>>) -> Result<String, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = Vec::new(); // Replace with actual file reading
    calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn compare_file_or_dir_with_hash(state: Arc<Mutex<SettingsState>>) -> Result<bool, String> {
    let checksum_method = get_checksum_method(state).await.map_err(|e| e.to_string())?;
    let data = Vec::new(); // Replace with actual file reading
    let calculated_hash = calculate_hash(checksum_method, &data)
        .await
        .map_err(|e| e.to_string())?;

    // Replace with actual hash comparison
    Ok(true) // Placeholder return
}

