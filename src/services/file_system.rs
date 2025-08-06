use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },
    
    #[error("Operation not supported: {operation}")]
    NotSupported { operation: String },
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
    pub size: u64,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub is_directory: bool,
    pub is_hidden: bool,
    pub permissions: FilePermissions,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Directory,
    Image(ImageFormat),
    Video(VideoFormat),
    Audio(AudioFormat),
    Document(DocumentFormat),
    Text(TextFormat),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg, Png, Gif, WebP, Tiff, Bmp, Svg
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    Mp4, Avi, Mov, Wmv, Mkv, WebM
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormat {
    Mp3, Wav, Flac, Aac, Ogg
}

#[derive(Debug, Clone, PartialEq)]
pub enum DocumentFormat {
    Pdf, Docx, Doc, Xlsx, Xls, Pptx, Ppt, Txt, Md
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextFormat {
    Plain, Markdown, Json, Xml, Html, Css, JavaScript, Rust, Python
}

#[derive(Debug, Clone)]
pub struct FilePermissions {
    pub readable: bool,
    pub writable: bool,
    pub executable: bool,
}

impl Default for FilePermissions {
    fn default() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
        }
    }
}

impl FileType {
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) => match ext.to_lowercase().as_str() {
                // Images
                "jpg" | "jpeg" => FileType::Image(ImageFormat::Jpeg),
                "png" => FileType::Image(ImageFormat::Png),
                "gif" => FileType::Image(ImageFormat::Gif),
                "webp" => FileType::Image(ImageFormat::WebP),
                "tiff" | "tif" => FileType::Image(ImageFormat::Tiff),
                "bmp" => FileType::Image(ImageFormat::Bmp),
                "svg" => FileType::Image(ImageFormat::Svg),
                
                // Videos
                "mp4" => FileType::Video(VideoFormat::Mp4),
                "avi" => FileType::Video(VideoFormat::Avi),
                "mov" => FileType::Video(VideoFormat::Mov),
                "wmv" => FileType::Video(VideoFormat::Wmv),
                "mkv" => FileType::Video(VideoFormat::Mkv),
                "webm" => FileType::Video(VideoFormat::WebM),
                
                // Audio
                "mp3" => FileType::Audio(AudioFormat::Mp3),
                "wav" => FileType::Audio(AudioFormat::Wav),
                "flac" => FileType::Audio(AudioFormat::Flac),
                "aac" => FileType::Audio(AudioFormat::Aac),
                "ogg" => FileType::Audio(AudioFormat::Ogg),
                
                // Documents
                "pdf" => FileType::Document(DocumentFormat::Pdf),
                "docx" => FileType::Document(DocumentFormat::Docx),
                "doc" => FileType::Document(DocumentFormat::Doc),
                "xlsx" => FileType::Document(DocumentFormat::Xlsx),
                "xls" => FileType::Document(DocumentFormat::Xls),
                "pptx" => FileType::Document(DocumentFormat::Pptx),
                "ppt" => FileType::Document(DocumentFormat::Ppt),
                "txt" => FileType::Document(DocumentFormat::Txt),
                "md" => FileType::Document(DocumentFormat::Md),
                
                // Text files
                "json" => FileType::Text(TextFormat::Json),
                "xml" => FileType::Text(TextFormat::Xml),
                "html" | "htm" => FileType::Text(TextFormat::Html),
                "css" => FileType::Text(TextFormat::Css),
                "js" => FileType::Text(TextFormat::JavaScript),
                "rs" => FileType::Text(TextFormat::Rust),
                "py" => FileType::Text(TextFormat::Python),
                
                // Other
                _ => FileType::Other(ext.to_string()),
            },
            None => {
                if path.is_dir() {
                    FileType::Directory
                } else {
                    FileType::Other("unknown".to_string())
                }
            }
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            FileType::Directory => "📁",
            FileType::Image(_) => "🖼️",
            FileType::Video(_) => "🎬",
            FileType::Audio(_) => "🎵",
            FileType::Document(_) => "📄",
            FileType::Text(_) => "📝",
            FileType::Other(_) => "📄",
        }
    }
}

#[async_trait::async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn get_metadata(&self, path: &Path) -> Result<FileEntry, FileSystemError>;
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError>;
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError>;
    async fn get_home_directory(&self) -> Result<PathBuf, FileSystemError>;
    async fn get_desktop_directory(&self) -> Result<PathBuf, FileSystemError>;
    async fn get_documents_directory(&self) -> Result<PathBuf, FileSystemError>;
}

pub struct NativeFileSystemService;

impl NativeFileSystemService {
    pub fn new() -> Self {
        Self
    }
    
    fn create_file_entry(path: PathBuf, metadata: &std::fs::Metadata) -> FileEntry {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let file_type = if metadata.is_dir() {
            FileType::Directory
        } else {
            FileType::from_path(&path)
        };
        
        let is_hidden = name.starts_with('.');
        
        FileEntry {
            path: path.clone(),
            name,
            file_type,
            size: metadata.len(),
            modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            created: metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
            is_directory: metadata.is_dir(),
            is_hidden,
            permissions: get_permissions(metadata),
        }
    }
}

#[async_trait::async_trait]
impl FileSystemService for NativeFileSystemService {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            if !path.is_dir() {
                return Err(FileSystemError::InvalidPath { path });
            }
            
            let mut entries = Vec::new();
            
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let entry_path = entry.path();
                let metadata = entry.metadata()?;
                
                let file_entry = Self::create_file_entry(entry_path, &metadata);
                entries.push(file_entry);
            }
            
            // Sort entries: directories first, then by name
            entries.sort_by(|a, b| {
                match (a.is_directory, b.is_directory) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            });
            
            Ok(entries)
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn get_metadata(&self, path: &Path) -> Result<FileEntry, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            Ok(Self::create_file_entry(path, &metadata))
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&path)?;
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError> {
        let sources = sources.to_vec();
        let dest = dest.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !dest.exists() {
                std::fs::create_dir_all(&dest)?;
            }
            
            for source in sources {
                if !source.exists() {
                    return Err(FileSystemError::PathNotFound { path: source });
                }
                
                let file_name = source.file_name()
                    .ok_or_else(|| FileSystemError::InvalidPath { path: source.clone() })?;
                let dest_path = dest.join(file_name);
                
                if source.is_dir() {
                    copy_dir_recursively(&source, &dest_path)?;
                } else {
                    std::fs::copy(&source, &dest_path)?;
                }
            }
            
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError> {
        let sources = sources.to_vec();
        let dest = dest.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !dest.exists() {
                std::fs::create_dir_all(&dest)?;
            }
            
            for source in sources {
                if !source.exists() {
                    return Err(FileSystemError::PathNotFound { path: source });
                }
                
                let file_name = source.file_name()
                    .ok_or_else(|| FileSystemError::InvalidPath { path: source.clone() })?;
                let dest_path = dest.join(file_name);
                
                std::fs::rename(&source, &dest_path)?;
            }
            
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError> {
        let paths = paths.to_vec();
        
        tokio::task::spawn_blocking(move || {
            for path in paths {
                if !path.exists() {
                    return Err(FileSystemError::PathNotFound { path });
                }
                
                if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                } else {
                    std::fs::remove_file(&path)?;
                }
            }
            
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?
    }
    
    async fn get_home_directory(&self) -> Result<PathBuf, FileSystemError> {
        dirs::home_dir()
            .ok_or_else(|| FileSystemError::NotSupported { 
                operation: "get home directory".to_string() 
            })
    }
    
    async fn get_desktop_directory(&self) -> Result<PathBuf, FileSystemError> {
        dirs::desktop_dir()
            .ok_or_else(|| FileSystemError::NotSupported { 
                operation: "get desktop directory".to_string() 
            })
    }
    
    async fn get_documents_directory(&self) -> Result<PathBuf, FileSystemError> {
        dirs::document_dir()
            .ok_or_else(|| FileSystemError::NotSupported { 
                operation: "get documents directory".to_string() 
            })
    }
}

// Helper function to recursively copy directories
fn copy_dir_recursively(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(src)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let dest_path = dest.join(relative_path);
        
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    
    Ok(())
}

// Platform-specific permission handling
#[cfg(unix)]
fn get_permissions(metadata: &std::fs::Metadata) -> FilePermissions {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    
    FilePermissions {
        readable: mode & 0o400 != 0,
        writable: mode & 0o200 != 0,
        executable: mode & 0o100 != 0,
    }
}

#[cfg(windows)]
fn get_permissions(metadata: &std::fs::Metadata) -> FilePermissions {
    FilePermissions {
        readable: !metadata.permissions().readonly(),
        writable: !metadata.permissions().readonly(),
        executable: false, // Windows doesn't have simple executable bit
    }
}

#[cfg(not(any(unix, windows)))]
fn get_permissions(_metadata: &std::fs::Metadata) -> FilePermissions {
    FilePermissions::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_list_directory() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create test files
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        let test_dir = temp_dir.path().join("subdir");
        std::fs::create_dir(&test_dir).unwrap();
        
        let entries = service.list_directory(temp_dir.path()).await.unwrap();
        
        assert_eq!(entries.len(), 2);
        
        // Directories should come first
        assert!(entries[0].is_directory);
        assert_eq!(entries[0].name, "subdir");
        
        assert!(!entries[1].is_directory);
        assert_eq!(entries[1].name, "test.txt");
    }
    
    #[tokio::test]
    async fn test_file_type_detection() {
        assert_eq!(FileType::from_path(Path::new("test.jpg")), FileType::Image(ImageFormat::Jpeg));
        assert_eq!(FileType::from_path(Path::new("test.mp4")), FileType::Video(VideoFormat::Mp4));
        assert_eq!(FileType::from_path(Path::new("test.mp3")), FileType::Audio(AudioFormat::Mp3));
        assert_eq!(FileType::from_path(Path::new("test.pdf")), FileType::Document(DocumentFormat::Pdf));
    }
    
    #[tokio::test]
    async fn test_copy_files() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source file
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, "test content").unwrap();
        
        // Create destination directory
        let dest_dir = temp_dir.path().join("dest");
        std::fs::create_dir(&dest_dir).unwrap();
        
        // Copy file
        service.copy_files(&[source_file.clone()], &dest_dir).await.unwrap();
        
        // Verify copy
        let copied_file = dest_dir.join("source.txt");
        assert!(copied_file.exists());
        
        let content = std::fs::read_to_string(&copied_file).unwrap();
        assert_eq!(content, "test content");
        
        // Verify original still exists
        assert!(source_file.exists());
    }
}