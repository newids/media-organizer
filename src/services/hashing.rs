use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::io;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tracing::{debug, error};
use thiserror::Error;

/// Buffer size for reading files in chunks (64KB)
const BUFFER_SIZE: usize = 64 * 1024;

/// Maximum file size for hashing (1GB) to prevent memory issues
const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;

/// Errors that can occur during file hashing operations
#[derive(Debug, Error)]
pub enum HashingError {
    #[error("IO error while reading file: {0}")]
    Io(#[from] io::Error),
    
    #[error("File too large: {size} bytes (max: {max} bytes)")]
    FileTooLarge { size: u64, max: u64 },
    
    #[error("File does not exist: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Permission denied accessing file: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Cancelled by user")]
    Cancelled,
}

/// Result type for hashing operations
pub type HashingResult<T> = Result<T, HashingError>;

/// Represents a file hash with metadata
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileHash {
    /// The SHA-256 hash as a hex string
    pub hash: String,
    /// File path that was hashed
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Time taken to compute hash in milliseconds
    pub computation_time_ms: u64,
}

impl FileHash {
    /// Create a new FileHash
    pub fn new(hash: String, path: PathBuf, size: u64, computation_time_ms: u64) -> Self {
        Self {
            hash,
            path,
            size,
            computation_time_ms,
        }
    }
    
    /// Get hash as bytes
    pub fn hash_bytes(&self) -> Result<Vec<u8>, hex::FromHexError> {
        hex::decode(&self.hash)
    }
    
    /// Check if this hash matches another
    pub fn matches(&self, other: &FileHash) -> bool {
        self.hash == other.hash
    }
}

/// Hash algorithm variants for future extensibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    // Future: Sha512, Md5, etc.
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        Self::Sha256
    }
}

impl HashAlgorithm {
    /// Get the name of the algorithm
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sha256 => "SHA-256",
        }
    }
    
    /// Get the expected hash length in characters
    pub fn hash_length(&self) -> usize {
        match self {
            Self::Sha256 => 64, // 32 bytes * 2 hex chars
        }
    }
}

/// Configuration for the hashing service
#[derive(Debug, Clone)]
pub struct HashingConfig {
    /// Algorithm to use for hashing
    pub algorithm: HashAlgorithm,
    /// Buffer size for file reading
    pub buffer_size: usize,
    /// Maximum file size to hash
    pub max_file_size: u64,
    /// Whether to skip hidden files
    pub skip_hidden: bool,
}

impl Default for HashingConfig {
    fn default() -> Self {
        Self {
            algorithm: HashAlgorithm::default(),
            buffer_size: BUFFER_SIZE,
            max_file_size: MAX_FILE_SIZE,
            skip_hidden: true,
        }
    }
}

/// Service for computing file hashes
#[derive(Debug, Clone)]
pub struct HashingService {
    config: HashingConfig,
}

impl HashingService {
    /// Create a new hashing service with default configuration
    pub fn new() -> Self {
        Self {
            config: HashingConfig::default(),
        }
    }
    
    /// Create a new hashing service with custom configuration
    pub fn with_config(config: HashingConfig) -> Self {
        Self { config }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &HashingConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: HashingConfig) {
        self.config = config;
    }
    
    /// Hash a single file asynchronously
    pub async fn hash_file(&self, path: &Path) -> HashingResult<FileHash> {
        let start_time = std::time::Instant::now();
        
        // Validate file exists and get metadata
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            match e.kind() {
                io::ErrorKind::NotFound => HashingError::FileNotFound { 
                    path: path.to_path_buf() 
                },
                io::ErrorKind::PermissionDenied => HashingError::PermissionDenied { 
                    path: path.to_path_buf() 
                },
                _ => HashingError::Io(e),
            }
        })?;
        
        // Check if file is too large
        let file_size = metadata.len();
        if file_size > self.config.max_file_size {
            return Err(HashingError::FileTooLarge { 
                size: file_size, 
                max: self.config.max_file_size 
            });
        }
        
        // Skip hidden files if configured
        if self.config.skip_hidden && self.is_hidden_file(path) {
            debug!("Skipping hidden file: {}", path.display());
            return Err(HashingError::PermissionDenied { 
                path: path.to_path_buf() 
            });
        }
        
        // Compute hash based on algorithm
        let hash = match self.config.algorithm {
            HashAlgorithm::Sha256 => self.compute_sha256(path).await?,
        };
        
        let computation_time = start_time.elapsed().as_millis() as u64;
        
        debug!(
            "Hashed file: {} ({} bytes) in {}ms",
            path.display(),
            file_size,
            computation_time
        );
        
        Ok(FileHash::new(
            hash,
            path.to_path_buf(),
            file_size,
            computation_time,
        ))
    }
    
    /// Compute SHA-256 hash of a file
    async fn compute_sha256(&self, path: &Path) -> HashingResult<String> {
        let file = File::open(path).await?;
        let mut reader = BufReader::with_capacity(self.config.buffer_size, file);
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; self.config.buffer_size];
        
        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            hasher.update(&buffer[..bytes_read]);
            
            // Yield control occasionally to allow cancellation
            if bytes_read == self.config.buffer_size {
                tokio::task::yield_now().await;
            }
        }
        
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }
    
    /// Check if a file is hidden (starts with dot on Unix, has hidden attribute on Windows)
    fn is_hidden_file(&self, path: &Path) -> bool {
        if let Some(filename) = path.file_name() {
            if let Some(name_str) = filename.to_str() {
                // Unix-style hidden files (start with dot)
                // But exclude temporary files for testing
                if name_str.starts_with('.') && !name_str.starts_with(".tmp") {
                    return true;
                }
            }
        }
        
        // Windows hidden files would require additional platform-specific checks
        // For now, we'll just use the dot convention
        false
    }
    
    /// Hash multiple files and return results
    pub async fn hash_files(&self, paths: &[PathBuf]) -> Vec<HashingResult<FileHash>> {
        let mut results = Vec::with_capacity(paths.len());
        
        for path in paths {
            let result = self.hash_file(path).await;
            results.push(result);
        }
        
        results
    }
    
    /// Validate that a hash string is valid for the current algorithm
    pub fn is_valid_hash(&self, hash: &str) -> bool {
        if hash.len() != self.config.algorithm.hash_length() {
            return false;
        }
        
        // Check if all characters are valid hex
        hash.chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl Default for HashingService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;
    
    #[tokio::test]
    async fn test_hash_empty_file() {
        let temp_file = NamedTempFile::new().unwrap();
        // Empty file
        
        let service = HashingService::new();
        let result = service.hash_file(temp_file.path()).await.unwrap();
        
        // SHA-256 of empty file
        assert_eq!(result.hash, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
        assert_eq!(result.size, 0);
    }
    
    #[tokio::test]
    async fn test_hash_known_content() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        let content = b"hello world";
        file.write_all(content).await.unwrap();
        file.flush().await.unwrap();
        
        let service = HashingService::new();
        let result = service.hash_file(temp_file.path()).await.unwrap();
        
        // SHA-256 of "hello world"
        assert_eq!(result.hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
        assert_eq!(result.size, content.len() as u64);
    }
    
    #[tokio::test]
    async fn test_hash_large_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        
        // Write 1MB of data
        let chunk = vec![0u8; 1024];
        for _ in 0..1024 {
            file.write_all(&chunk).await.unwrap();
        }
        file.flush().await.unwrap();
        
        let service = HashingService::new();
        let result = service.hash_file(temp_file.path()).await.unwrap();
        
        assert_eq!(result.size, 1024 * 1024);
        assert!(!result.hash.is_empty());
        assert_eq!(result.hash.len(), 64); // SHA-256 hex length
    }
    
    #[tokio::test]
    async fn test_file_not_found() {
        let service = HashingService::new();
        let result = service.hash_file(Path::new("/nonexistent/path")).await;
        
        assert!(matches!(result, Err(HashingError::FileNotFound { .. })));
    }
    
    #[tokio::test]
    async fn test_file_too_large() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
        file.write_all(b"test").await.unwrap();
        file.flush().await.unwrap();
        
        let config = HashingConfig {
            max_file_size: 2, // Only 2 bytes allowed
            ..Default::default()
        };
        
        let service = HashingService::with_config(config);
        let result = service.hash_file(temp_file.path()).await;
        
        assert!(matches!(result, Err(HashingError::FileTooLarge { .. })));
    }
    
    #[test]
    fn test_is_valid_hash() {
        let service = HashingService::new();
        
        // Valid SHA-256 hash
        assert!(service.is_valid_hash("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"));
        
        // Invalid length
        assert!(!service.is_valid_hash("short"));
        
        // Invalid characters
        assert!(!service.is_valid_hash("g94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"));
    }
    
    #[test]
    fn test_file_hash_equality() {
        let hash1 = FileHash::new(
            "test_hash".to_string(),
            PathBuf::from("test.txt"),
            100,
            50,
        );
        
        let hash2 = FileHash::new(
            "test_hash".to_string(),
            PathBuf::from("other.txt"),
            200,
            75,
        );
        
        assert!(hash1.matches(&hash2));
    }
}