//! File reading utilities with memory mapping support.
//!
//! Provides efficient file reading for both small and large files,
//! with automatic detection of when to use memory mapping.

// Memory mapping requires unsafe but is well-documented and safe for read-only access
#![allow(unsafe_code)]

use crate::error::{IoError, Result};
use memmap2::Mmap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Threshold for using memory mapping (1MB).
const MMAP_THRESHOLD: u64 = 1024 * 1024;

/// Maximum file size to read into memory (1GB).
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;

/// File reader with support for memory mapping.
///
/// Automatically chooses the best reading strategy based on file size:
/// - Small files (< 1MB): Read directly into memory
/// - Large files (>= 1MB): Use memory mapping
///
/// # Examples
///
/// ```no_run
/// use rlm_rs::io::FileReader;
///
/// let reader = FileReader::open("large_file.txt").unwrap();
/// let content = reader.read_to_string().unwrap();
/// ```
pub struct FileReader {
    /// File handle.
    file: File,
    /// File size in bytes.
    size: u64,
    /// File path for error messages.
    path: String,
}

impl FileReader {
    /// Opens a file for reading.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file doesn't exist or can't be opened.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();

        if !path_ref.exists() {
            return Err(IoError::FileNotFound { path: path_str }.into());
        }

        let file = File::open(path_ref).map_err(|e| IoError::ReadFailed {
            path: path_str.clone(),
            reason: e.to_string(),
        })?;

        let metadata = file.metadata().map_err(|e| IoError::ReadFailed {
            path: path_str.clone(),
            reason: e.to_string(),
        })?;

        let size = metadata.len();

        if size > MAX_FILE_SIZE {
            return Err(IoError::ReadFailed {
                path: path_str,
                reason: format!("file too large: {size} bytes (max: {MAX_FILE_SIZE} bytes)"),
            }
            .into());
        }

        Ok(Self {
            file,
            size,
            path: path_str,
        })
    }

    /// Returns the file size in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns the file path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Reads the file content as a string.
    ///
    /// Uses memory mapping for large files.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or content is not valid UTF-8.
    pub fn read_to_string(&self) -> Result<String> {
        if self.size >= MMAP_THRESHOLD {
            self.read_mmap()
        } else {
            self.read_direct()
        }
    }

    /// Reads the file content as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    pub fn read_to_bytes(&self) -> Result<Vec<u8>> {
        if self.size >= MMAP_THRESHOLD {
            self.read_mmap_bytes()
        } else {
            self.read_direct_bytes()
        }
    }

    /// Reads using memory mapping.
    fn read_mmap(&self) -> Result<String> {
        let bytes = self.read_mmap_bytes()?;
        String::from_utf8(bytes).map_err(|e| {
            IoError::ReadFailed {
                path: self.path.clone(),
                reason: format!("invalid UTF-8: {e}"),
            }
            .into()
        })
    }

    /// Reads bytes using memory mapping.
    fn read_mmap_bytes(&self) -> Result<Vec<u8>> {
        // Safety: We're only reading from the file, which is safe
        let mmap = unsafe {
            Mmap::map(&self.file).map_err(|e| IoError::MmapFailed {
                path: self.path.clone(),
                reason: e.to_string(),
            })?
        };

        Ok(mmap.to_vec())
    }

    /// Reads directly into memory.
    fn read_direct(&self) -> Result<String> {
        let bytes = self.read_direct_bytes()?;
        String::from_utf8(bytes).map_err(|e| {
            IoError::ReadFailed {
                path: self.path.clone(),
                reason: format!("invalid UTF-8: {e}"),
            }
            .into()
        })
    }

    /// Reads bytes directly into memory.
    #[allow(clippy::cast_possible_truncation)]
    fn read_direct_bytes(&self) -> Result<Vec<u8>> {
        let mut file = &self.file;
        let mut buffer = Vec::with_capacity(self.size as usize);
        file.read_to_end(&mut buffer)
            .map_err(|e| IoError::ReadFailed {
                path: self.path.clone(),
                reason: e.to_string(),
            })?;
        Ok(buffer)
    }

    /// Creates a memory-mapped view of the file.
    ///
    /// Useful when you need to access the file content multiple times
    /// without copying.
    ///
    /// # Errors
    ///
    /// Returns an error if memory mapping fails.
    pub fn mmap(&self) -> Result<Mmap> {
        // Safety: We're only reading from the file
        unsafe {
            Mmap::map(&self.file).map_err(|e| {
                IoError::MmapFailed {
                    path: self.path.clone(),
                    reason: e.to_string(),
                }
                .into()
            })
        }
    }
}

/// Reads a file to string, automatically choosing the best method.
///
/// # Arguments
///
/// * `path` - Path to the file.
///
/// # Errors
///
/// Returns an error if the file cannot be read or is not valid UTF-8.
///
/// # Examples
///
/// ```no_run
/// use rlm_rs::io::read_file;
///
/// let content = read_file("example.txt").unwrap();
/// ```
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    FileReader::open(path)?.read_to_string()
}

/// Reads a file using memory mapping.
///
/// This is useful for very large files that shouldn't be fully loaded
/// into memory.
///
/// # Arguments
///
/// * `path` - Path to the file.
///
/// # Errors
///
/// Returns an error if the file cannot be opened or memory mapping fails.
pub fn read_file_mmap<P: AsRef<Path>>(path: P) -> Result<Mmap> {
    FileReader::open(path)?.mmap()
}

/// Writes content to a file, creating parent directories if needed.
///
/// # Arguments
///
/// * `path` - Path to the file.
/// * `content` - Content to write.
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_string_lossy().to_string();

    // Create parent directories
    if let Some(parent) = path_ref.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| IoError::DirectoryFailed {
                path: parent.to_string_lossy().to_string(),
                reason: e.to_string(),
            })?;
        }
    }

    std::fs::write(path_ref, content).map_err(|e| IoError::WriteFailed {
        path: path_str,
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Writes chunks to individual files in a directory.
///
/// # Arguments
///
/// * `out_dir` - Directory to write chunks to.
/// * `chunks` - Iterator of (index, content) pairs.
/// * `prefix` - Filename prefix (e.g., "chunk").
///
/// # Returns
///
/// Vector of paths to the written files.
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn write_chunks<'a, P, I>(out_dir: P, chunks: I, prefix: &str) -> Result<Vec<String>>
where
    P: AsRef<Path>,
    I: Iterator<Item = (usize, &'a str)>,
{
    let out_path = out_dir.as_ref();
    let out_str = out_path.to_string_lossy().to_string();

    // Create output directory
    if !out_path.exists() {
        std::fs::create_dir_all(out_path).map_err(|e| IoError::DirectoryFailed {
            path: out_str.clone(),
            reason: e.to_string(),
        })?;
    }

    let mut paths = Vec::new();

    for (index, content) in chunks {
        let filename = format!("{prefix}_{index:04}.txt");
        let file_path = out_path.join(&filename);
        let file_str = file_path.to_string_lossy().to_string();

        std::fs::write(&file_path, content).map_err(|e| IoError::WriteFailed {
            path: file_str.clone(),
            reason: e.to_string(),
        })?;

        paths.push(file_str);
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("small.txt");
        std::fs::write(&file_path, "Hello, world!").unwrap();

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "Hello, world!");
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file("/nonexistent/path/file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_reader_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello").unwrap();

        let reader = FileReader::open(&file_path).unwrap();
        assert_eq!(reader.size(), 5);
    }

    #[test]
    fn test_write_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir/output.txt");

        write_file(&file_path, "Test content").unwrap();

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "Test content");
    }

    #[test]
    fn test_write_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let out_dir = temp_dir.path().join("chunks");

        let chunks = vec![(0, "First chunk"), (1, "Second chunk")];
        let paths = write_chunks(&out_dir, chunks.into_iter(), "chunk").unwrap();

        assert_eq!(paths.len(), 2);

        let content0 = std::fs::read_to_string(&paths[0]).unwrap();
        let content1 = std::fs::read_to_string(&paths[1]).unwrap();
        assert_eq!(content0, "First chunk");
        assert_eq!(content1, "Second chunk");
    }

    #[test]
    fn test_read_utf8_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.txt");
        std::fs::write(&file_path, "Hello, 世界! 🌍").unwrap();

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "Hello, 世界! 🌍");
    }
}
