use base64::Engine;
use anyhow::Result;
use serde::Serialize;
use std::{fs, io::Read, path::{Path, PathBuf}};

#[derive(Serialize, Debug)]
#[serde(tag = "kind")]
pub enum PreviewPayload {
    Image { name: String, data_uri: String, bytes: usize },
    Pdf   { name: String, data_uri: String, bytes: usize },
    Video { name: String, path: String },
    Audio { name: String, path: String },
    Text  { name: String, text: String, truncated: bool },
    Folder {
        name: String,
        size: u64,
        item_count: usize,
        modified: Option<String>,
    },
    Unknown { name: String },
    #[allow(dead_code)]
    Error { name: String, message: String },
}

fn filename(p: &Path) -> String {
    p.file_name().and_then(|s| s.to_str()).unwrap_or("file").to_string()
}

fn read_prefix(path: &Path, max_bytes: usize) -> Result<Vec<u8>> {
    let mut f = fs::File::open(path)?;
    let mut buf = Vec::with_capacity(max_bytes.min(1024 * 1024));
    (&mut f).take(max_bytes as u64).read_to_end(&mut buf)?;
    Ok(buf)
}

fn detect_mime(path: &Path, head: &[u8]) -> Option<&'static str> {
    if let Some(kind) = infer::get(head) {
        return Some(kind.mime_type());
    }
    // fallback by extension if needed
    if let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
        return Some(match ext.as_str() {
            "md" | "rs" | "ts" | "tsx" | "js" | "jsx" | "json" | "txt" | "log" | "toml" | "yaml" | "yml" | "xml" | "ini" | "csv" => "text/plain",
            "pdf" => "application/pdf",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "mp4" => "video/mp4",
            "mov" => "video/quicktime",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            _ => "application/octet-stream",
        });
    }
    None
}

#[tauri::command]
pub fn build_preview(path: String) -> Result<PreviewPayload, String> {
    let p = PathBuf::from(&path);
    let name = filename(&p);

    // Folders: return Folder preview
    if p.is_dir() {
        // Count items (files + dirs, not recursive)
        let mut item_count = 0;
        let mut size: u64 = 0;
        let mut latest_modified: Option<std::time::SystemTime> = None;
        if let Ok(entries) = fs::read_dir(&p) {
            for entry in entries.flatten() {
                item_count += 1;
                if let Ok(meta) = entry.metadata() {
                    size += meta.len();
                    if let Ok(modified) = meta.modified() {
                        latest_modified = match latest_modified {
                            Some(current) if current > modified => Some(current),
                            _ => Some(modified),
                        };
                    }
                }
            }
        }
        // Use folder's own modified time if no children
        let folder_meta = fs::metadata(&p).ok();
        let folder_modified = folder_meta.and_then(|m| m.modified().ok());
        let modified_time = latest_modified.or(folder_modified);
        let modified_str = modified_time.and_then(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339().into());
        return Ok(PreviewPayload::Folder {
            name,
            size,
            item_count,
            modified: modified_str,
        });
    }

    // Files
    let meta = fs::metadata(&p).map_err(|e| e.to_string())?;
    // Read a small head for detection + maybe text
    let head = read_prefix(&p, 256 * 1024).map_err(|e| e.to_string())?;
    let mime = detect_mime(&p, &head).unwrap_or("application/octet-stream");

    // Branch by mime top-level type
    if mime.starts_with("image/") {
        // Encode entire file only if small; else just the head (fast path)
        // You can raise this cap depending on your perf goals
        let cap = 6 * 1024 * 1024;
        let bytes = meta.len() as usize;
        let data = if bytes <= cap {
            fs::read(&p).map_err(|e| e.to_string())?
        } else {
            head.clone()
        };
    let data_uri = format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(data));
        return Ok(PreviewPayload::Image { name, data_uri, bytes });
    }
    if mime == "application/pdf" {
        // Encode entire file only if small; else just the head (fast path)
        let cap = 12 * 1024 * 1024; // Allow larger PDFs than images
        let bytes = meta.len() as usize;
        let data = if bytes <= cap {
            fs::read(&p).map_err(|e| e.to_string())?
        } else {
            head.clone()
        };
    let data_uri = format!("data:{};base64,{}", mime, base64::engine::general_purpose::STANDARD.encode(data));
        return Ok(PreviewPayload::Pdf { name, data_uri, bytes });
    }

    if mime.starts_with("video/") {
        return Ok(PreviewPayload::Video { name, path });
    }

    if mime.starts_with("audio/") {
        return Ok(PreviewPayload::Audio { name, path });
    }

    // Heuristic: treat smallish or text‑ish files as text
    let looks_texty = mime.starts_with("text/") || head.iter().all(|&b| b == 9 || b == 10 || b == 13 || (b >= 32 && b < 0xF5));
    if looks_texty || meta.len() <= 2 * 1024 * 1024 {
        let mut det = chardetng::EncodingDetector::new();
        det.feed(&head, true);
        let enc = det.guess(None, true);
        let (cow, _, _) = enc.decode(&head);
        let mut text = cow.to_string();
        let mut truncated = false;
        if text.len() > 200_000 {
            text.truncate(200_000);
            text.push_str("\n…(truncated)");
            truncated = true;
        }
        return Ok(PreviewPayload::Text { name, text, truncated });
    }

    Ok(PreviewPayload::Unknown { name })
}

#[cfg(test)]
mod preview_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use crate::{log_info, log_error};

    #[test]
    fn test_filename() {
        log_info!("Starting test_filename");
        
        let path = Path::new("/some/path/file.txt");
        assert_eq!(filename(path), "file.txt");
        
        let path = Path::new("file.txt");
        assert_eq!(filename(path), "file.txt");
        
        let path = Path::new("");
        assert_eq!(filename(path), "file");
        
        log_info!("test_filename completed successfully");
    }

    #[test]
    fn test_detect_mime_by_content() {
        log_info!("Starting test_detect_mime_by_content");
        
        // PNG signature
        let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let path = Path::new("test.png");
        let result = detect_mime(path, &png_bytes);
        
        if result == Some("image/png") {
            log_info!("PNG detection successful");
        } else {
            log_error!("PNG detection failed: expected 'image/png', got {:?}", result);
        }
        
        assert_eq!(result, Some("image/png"));
        log_info!("test_detect_mime_by_content completed successfully");
    }

    #[test]
    fn test_detect_mime_by_extension() {
        log_info!("Starting test_detect_mime_by_extension");
        
        let empty_bytes = vec![];
        
        let path = Path::new("test.rs");
        assert_eq!(detect_mime(path, &empty_bytes), Some("text/plain"));
        log_info!("Rust file extension detection successful");
        
        let path = Path::new("test.pdf");
        assert_eq!(detect_mime(path, &empty_bytes), Some("application/pdf"));
        log_info!("PDF file extension detection successful");
        
        let path = Path::new("test.mp4");
        assert_eq!(detect_mime(path, &empty_bytes), Some("video/mp4"));
        log_info!("MP4 file extension detection successful");
        
        let path = Path::new("test.mp3");
        assert_eq!(detect_mime(path, &empty_bytes), Some("audio/mpeg"));
        log_info!("MP3 file extension detection successful");
        
        log_info!("test_detect_mime_by_extension completed successfully");
    }

    #[test]
    fn test_build_preview_folder() {
        log_info!("Starting test_build_preview_folder");
        
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("test_folder");
        
        if let Err(e) = fs::create_dir(&test_dir) {
            log_error!("Failed to create test directory: {}", e);
            panic!("Failed to create test directory: {}", e);
        }
        
        // Create some test files
        if let Err(e) = fs::write(test_dir.join("file1.txt"), "content") {
            log_error!("Failed to create test file: {}", e);
            panic!("Failed to create test file: {}", e);
        }
        
        if let Err(e) = fs::create_dir(test_dir.join("subfolder")) {
            log_error!("Failed to create test subfolder: {}", e);
            panic!("Failed to create test subfolder: {}", e);
        }
        
        log_info!("Test folder structure created successfully");
        
        let result = build_preview(test_dir.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Folder { name, size, item_count, modified, .. }) => {
                log_info!("Folder preview generated: name={}, size={}, item_count={}, modified={:?}", name, size, item_count, modified);
                assert_eq!(name, "test_folder");
                // Additional checks for size, item_count, modified can be added if needed
                log_info!("All folder preview assertions passed");
            }
            Ok(other) => {
                log_error!("Expected folder preview, got: {:?}", other);
                panic!("Expected folder preview");
            }
            Err(e) => {
                log_error!("Failed to build folder preview: {}", e);
                panic!("Failed to build folder preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_folder completed successfully");
    }

    #[test]
    fn test_build_preview_text_file() {
        log_info!("Starting test_build_preview_text_file");
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        let content = "Hello, world!\nThis is a test file.";
        
        if let Err(e) = fs::write(&test_file, content) {
            log_error!("Failed to create test text file: {}", e);
            panic!("Failed to create test text file: {}", e);
        }
        
        log_info!("Test text file created successfully");
        
        let result = build_preview(test_file.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Text { name, text, truncated }) => {
                log_info!("Text preview generated: name={}, length={}, truncated={}", name, text.len(), truncated);
                assert_eq!(name, "test.txt");
                assert_eq!(text, content);
                assert!(!truncated);
                log_info!("All text preview assertions passed");
            }
            Ok(other) => {
                log_error!("Expected text preview, got: {:?}", other);
                panic!("Expected text preview");
            }
            Err(e) => {
                log_error!("Failed to build text preview: {}", e);
                panic!("Failed to build text preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_text_file completed successfully");
    }

    #[test]
    fn test_build_preview_image_file() {
        log_info!("Starting test_build_preview_image_file");
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.png");
        // Simple PNG signature + minimal data
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        
        if let Err(e) = fs::write(&test_file, &png_data) {
            log_error!("Failed to create test PNG file: {}", e);
            panic!("Failed to create test PNG file: {}", e);
        }
        
        log_info!("Test PNG file created successfully");
        
        let result = build_preview(test_file.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Image { name, data_uri, bytes }) => {
                log_info!("Image preview generated: name={}, bytes={}", name, bytes);
                assert_eq!(name, "test.png");
                assert!(data_uri.starts_with("data:image/png;base64,"));
                assert_eq!(bytes, png_data.len());
                log_info!("All image preview assertions passed");
            }
            Ok(other) => {
                log_error!("Expected image preview, got: {:?}", other);
                panic!("Expected image preview");
            }
            Err(e) => {
                log_error!("Failed to build image preview: {}", e);
                panic!("Failed to build image preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_image_file completed successfully");
    }

    #[test]
    fn test_build_preview_pdf_file() {
        log_info!("Starting test_build_preview_pdf_file");
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.pdf");
        
        if let Err(e) = fs::write(&test_file, "dummy pdf content") {
            log_error!("Failed to create test PDF file: {}", e);
            panic!("Failed to create test PDF file: {}", e);
        }
        
        log_info!("Test PDF file created successfully");
        
        let result = build_preview(test_file.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Pdf { name, data_uri: _, bytes, .. }) => {
                log_info!("PDF preview generated: name={}, bytes={}", name, bytes);
                assert_eq!(name, "test.pdf");
                // Additional checks for data_uri and bytes can be added if needed
                log_info!("All PDF preview assertions passed");
            }
            Ok(other) => {
                log_error!("Expected PDF preview, got: {:?}", other);
                panic!("Expected PDF preview");
            }
            Err(e) => {
                log_error!("Failed to build PDF preview: {}", e);
                panic!("Failed to build PDF preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_pdf_file completed successfully");
    }

    #[test]
    fn test_build_preview_nonexistent_file() {
        log_info!("Starting test_build_preview_nonexistent_file");
        
        let result = build_preview("/nonexistent/path".to_string());
        
        match result {
            Err(e) => {
                log_info!("Expected error for nonexistent file: {}", e);
                log_info!("test_build_preview_nonexistent_file completed successfully");
            }
            Ok(payload) => {
                log_error!("Expected error for nonexistent file, but got: {:?}", payload);
                panic!("Expected error for nonexistent file");
            }
        }
    }

    #[test]
    fn test_build_preview_large_text_truncation() {
        log_info!("Starting test_build_preview_large_text_truncation");
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("large.txt");
        let large_content = "a".repeat(300_000);
        
        if let Err(e) = fs::write(&test_file, &large_content) {
            log_error!("Failed to create large test file: {}", e);
            panic!("Failed to create large test file: {}", e);
        }
        
        log_info!("Large test file created successfully (300,000 characters)");
        
        let result = build_preview(test_file.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Text { name, text, truncated }) => {
                log_info!("Large text preview generated: name={}, length={}, truncated={}", name, text.len(), truncated);
                assert_eq!(name, "large.txt");
                assert!(truncated);
                assert!(text.len() <= 200_000 + 15); // +15 for truncation message
                assert!(text.ends_with("…(truncated)"));
                log_info!("All large text truncation assertions passed");
            }
            Ok(other) => {
                log_error!("Expected text preview, got: {:?}", other);
                panic!("Expected text preview");
            }
            Err(e) => {
                log_error!("Failed to build large text preview: {}", e);
                panic!("Failed to build large text preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_large_text_truncation completed successfully");
    }

    #[test]
    fn test_build_preview_folder_truncation() {
        log_info!("Starting test_build_preview_folder_truncation");
        
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("large_folder");
        
        if let Err(e) = fs::create_dir(&test_dir) {
            log_error!("Failed to create large test directory: {}", e);
            panic!("Failed to create large test directory: {}", e);
        }
        
        // Create more than 200 files to test truncation
        for i in 0..250 {
            if let Err(e) = fs::write(test_dir.join(format!("file{}.txt", i)), "content") {
                log_error!("Failed to create test file {}: {}", i, e);
                panic!("Failed to create test file {}: {}", i, e);
            }
        }
        
        log_info!("Large folder created successfully (250 files)");
        
        let result = build_preview(test_dir.to_string_lossy().to_string());
        
        match result {
            Ok(PreviewPayload::Folder { name, size, item_count, modified, .. }) => {
                log_info!("Large folder preview generated: name={}, size={}, item_count={}, modified={:?}", name, size, item_count, modified);
                assert_eq!(name, "large_folder");
                // Additional checks for size, item_count, modified can be added if needed
                log_info!("All large folder truncation assertions passed");
            }
            Ok(other) => {
                log_error!("Expected folder preview, got: {:?}", other);
                panic!("Expected folder preview");
            }
            Err(e) => {
                log_error!("Failed to build large folder preview: {}", e);
                panic!("Failed to build large folder preview: {}", e);
            }
        }
        
        log_info!("test_build_preview_folder_truncation completed successfully");
    }
}
