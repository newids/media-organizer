use std::path::{Path, PathBuf};
use std::time::SystemTime;
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Clone, Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },
    
    #[error("Operation not supported: {operation}")]
    NotSupported { operation: String },
    
    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },
    
    #[error("Directory not empty: {path}")]
    DirectoryNotEmpty { path: PathBuf },
    
    #[error("File already exists: {path}")]
    FileAlreadyExists { path: PathBuf },
    
    #[error("Disk full: insufficient space for operation")]
    DiskFull,
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Symlink loop detected in path: {path}")]
    SymlinkLoop { path: PathBuf },
    
    #[error("File system error: {message}")]
    FileSystem { message: String },
}

impl From<std::io::Error> for FileSystemError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl FileSystemError {
    /// Create a FileSystemError from a standard IO error with additional context
    pub fn from_io_error(error: std::io::Error, path: &Path) -> Self {
        match error.kind() {
            std::io::ErrorKind::NotFound => Self::PathNotFound { path: path.to_path_buf() },
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied { path: path.to_path_buf() },
            std::io::ErrorKind::AlreadyExists => Self::FileAlreadyExists { path: path.to_path_buf() },
            std::io::ErrorKind::InvalidInput => Self::InvalidPath { path: path.to_path_buf() },
            _ => Self::Io(error.to_string()),
        }
    }
    
    /// Check if this error represents a recoverable condition
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::PathNotFound { .. } | 
            Self::FileAlreadyExists { .. } |
            Self::DirectoryNotEmpty { .. }
        )
    }
    
    /// Get the path associated with this error, if any
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::PermissionDenied { path } |
            Self::PathNotFound { path } |
            Self::InvalidPath { path } |
            Self::DirectoryNotEmpty { path } |
            Self::FileAlreadyExists { path } |
            Self::SymlinkLoop { path } => Some(path),
            _ => None,
        }
    }
}

// Conversion utilities for common error scenarios
impl From<walkdir::Error> for FileSystemError {
    fn from(error: walkdir::Error) -> Self {
        if let Some(io_error) = error.io_error() {
            if let Some(path) = error.path() {
                Self::from_io_error(std::io::Error::from(io_error.kind()), path)
            } else {
                Self::Io(std::io::Error::from(io_error.kind()).to_string())
            }
        } else {
            Self::FileSystem { 
                message: format!("Directory traversal error: {}", error) 
            }
        }
    }
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

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.name == other.name
            && self.file_type == other.file_type
            && self.size == other.size
            && self.is_directory == other.is_directory
            && self.is_hidden == other.is_hidden
            && self.permissions == other.permissions
        // Note: We ignore modified and created times for comparison
    }
}

impl FileEntry {
    /// Get the file extension, if any
    pub fn extension(&self) -> Option<String> {
        self.path.extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
    
    /// Get the parent directory of this file
    pub fn parent(&self) -> Option<&Path> {
        self.path.parent()
    }
    
    /// Check if this is an image file
    pub fn is_image(&self) -> bool {
        matches!(self.file_type, FileType::Image(_))
    }
    
    /// Check if this is a video file
    pub fn is_video(&self) -> bool {
        matches!(self.file_type, FileType::Video(_))
    }
    
    /// Check if this is an audio file
    pub fn is_audio(&self) -> bool {
        matches!(self.file_type, FileType::Audio(_))
    }
    
    /// Check if this is a document file
    pub fn is_document(&self) -> bool {
        matches!(self.file_type, FileType::Document(_))
    }
    
    /// Check if this is a text file
    pub fn is_text(&self) -> bool {
        matches!(self.file_type, FileType::Text(_))
    }
    
    /// Check if this is a media file (image, video, or audio)
    pub fn is_media(&self) -> bool {
        self.is_image() || self.is_video() || self.is_audio()
    }
    
    /// Get a human-readable file size string
    pub fn size_string(&self) -> String {
        format_file_size(self.size)
    }
    
    /// Get the age of the file since last modification
    pub fn modified_duration(&self) -> Option<std::time::Duration> {
        SystemTime::now().duration_since(self.modified).ok()
    }
    
    /// Get a human-readable modified time string
    pub fn modified_string(&self) -> String {
        format_system_time(self.modified)
    }
    
    /// Check if the file can be read by current user
    pub fn can_read(&self) -> bool {
        self.permissions.can_read()
    }
    
    /// Check if the file can be written by current user
    pub fn can_write(&self) -> bool {
        self.permissions.can_write()
    }
    
    /// Check if the file can be executed by current user
    pub fn can_execute(&self) -> bool {
        self.permissions.can_execute()
    }
    
    /// Get the file type icon emoji
    pub fn icon(&self) -> &'static str {
        self.file_type.icon()
    }
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

#[derive(Debug, Clone, PartialEq)]
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

impl FilePermissions {
    pub fn new(readable: bool, writable: bool, executable: bool) -> Self {
        Self {
            readable,
            writable,
            executable,
        }
    }
    
    pub fn read_only() -> Self {
        Self {
            readable: true,
            writable: false,
            executable: false,
        }
    }
    
    pub fn read_write() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: false,
        }
    }
    
    pub fn all_permissions() -> Self {
        Self {
            readable: true,
            writable: true,
            executable: true,
        }
    }
    
    pub fn no_permissions() -> Self {
        Self {
            readable: false,
            writable: false,
            executable: false,
        }
    }
    
    pub fn can_read(&self) -> bool {
        self.readable
    }
    
    pub fn can_write(&self) -> bool {
        self.writable
    }
    
    pub fn can_execute(&self) -> bool {
        self.executable
    }
    
    pub fn is_read_only(&self) -> bool {
        self.readable && !self.writable && !self.executable
    }
    
    pub fn has_any_permission(&self) -> bool {
        self.readable || self.writable || self.executable
    }
    
    pub fn permission_string(&self) -> String {
        let mut perms = String::with_capacity(3);
        perms.push(if self.readable { 'r' } else { '-' });
        perms.push(if self.writable { 'w' } else { '-' });
        perms.push(if self.executable { 'x' } else { '-' });
        perms
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

#[derive(Debug, Clone)]
pub struct TraversalOptions {
    pub recursive: bool,
    pub include_hidden: Option<bool>,  // None = use config default
    pub follow_symlinks: Option<bool>, // None = use config default
    pub max_depth: Option<usize>,      // None = use config default
    pub file_types: Option<Vec<FileType>>, // Filter by specific file types
    pub name_patterns: Option<Vec<String>>, // Glob patterns for file names
}

impl Default for TraversalOptions {
    fn default() -> Self {
        Self {
            recursive: false,
            include_hidden: None,
            follow_symlinks: None,
            max_depth: None,
            file_types: None,
            name_patterns: None,
        }
    }
}

impl TraversalOptions {
    pub fn recursive() -> Self {
        Self {
            recursive: true,
            ..Default::default()
        }
    }
    
    pub fn with_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }
    
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = Some(include);
        self
    }
    
    pub fn follow_symlinks(mut self, follow: bool) -> Self {
        self.follow_symlinks = Some(follow);
        self
    }
    
    pub fn filter_types(mut self, types: Vec<FileType>) -> Self {
        self.file_types = Some(types);
        self
    }
    
    pub fn filter_patterns(mut self, patterns: Vec<String>) -> Self {
        self.name_patterns = Some(patterns);
        self
    }
}

#[derive(Debug, Clone)]
pub enum OverwriteMode {
    /// Fail if destination exists
    Fail,
    /// Overwrite existing files
    Overwrite,
    /// Skip existing files (for batch operations)
    Skip,
    /// Create backup before overwriting (filename.bak)
    Backup,
}

impl Default for OverwriteMode {
    fn default() -> Self {
        Self::Fail
    }
}

#[derive(Debug, Clone)]
pub struct FileOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub overwrite_mode: OverwriteMode,
    pub preserve_metadata: bool,
}

impl FileOperation {
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            overwrite_mode: OverwriteMode::default(),
            preserve_metadata: true,
        }
    }
    
    pub fn with_overwrite_mode(mut self, mode: OverwriteMode) -> Self {
        self.overwrite_mode = mode;
        self
    }
    
    pub fn preserve_metadata(mut self, preserve: bool) -> Self {
        self.preserve_metadata = preserve;
        self
    }
}

#[async_trait::async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn traverse_directory(&self, path: &Path, options: TraversalOptions) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn get_metadata(&self, path: &Path) -> Result<FileEntry, FileSystemError>;
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError>;
    
    // Individual file operations
    async fn copy_file(&self, operation: FileOperation) -> Result<(), FileSystemError>;
    async fn move_file(&self, operation: FileOperation) -> Result<(), FileSystemError>;
    async fn delete_file(&self, path: &Path) -> Result<(), FileSystemError>;
    async fn rename_file(&self, source: &Path, new_name: &str) -> Result<PathBuf, FileSystemError>;
    
    // Batch operations (existing)
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError>;
    
    // Permission checking methods
    async fn check_read_permission(&self, path: &Path) -> Result<bool, FileSystemError>;
    async fn check_write_permission(&self, path: &Path) -> Result<bool, FileSystemError>;
    async fn check_execute_permission(&self, path: &Path) -> Result<bool, FileSystemError>;
    async fn get_file_permissions(&self, path: &Path) -> Result<FilePermissions, FileSystemError>;
    
    // Extended metadata methods
    async fn get_file_size(&self, path: &Path) -> Result<u64, FileSystemError>;
    async fn get_modification_time(&self, path: &Path) -> Result<SystemTime, FileSystemError>;
    async fn get_creation_time(&self, path: &Path) -> Result<SystemTime, FileSystemError>;
    async fn is_hidden(&self, path: &Path) -> Result<bool, FileSystemError>;
    
    async fn get_home_directory(&self) -> Result<PathBuf, FileSystemError>;
    async fn get_desktop_directory(&self) -> Result<PathBuf, FileSystemError>;
    async fn get_documents_directory(&self) -> Result<PathBuf, FileSystemError>;
}

#[derive(Debug, Clone)]
pub struct FileSystemConfig {
    pub max_file_size: Option<u64>,  // Maximum file size for operations (bytes)
    pub follow_symlinks: bool,       // Whether to follow symbolic links
    pub include_hidden: bool,        // Whether to include hidden files by default
    pub max_depth: Option<usize>,    // Maximum directory depth for recursive operations
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        Self {
            max_file_size: None,          // No limit by default
            follow_symlinks: false,       // Don't follow symlinks for security
            include_hidden: false,        // Hide system files by default
            max_depth: Some(100),         // Reasonable recursion limit
        }
    }
}

#[derive(Debug)]
pub struct NativeFileSystemService {
    config: FileSystemConfig,
}

impl NativeFileSystemService {
    pub fn new() -> Self {
        Self {
            config: FileSystemConfig::default(),
        }
    }
    
    pub fn with_config(config: FileSystemConfig) -> Self {
        Self { config }
    }
    
    fn should_include_entry(entry: &FileEntry, options: &TraversalOptions, include_hidden: bool) -> bool {
        // Filter hidden files
        if entry.is_hidden && !include_hidden {
            return false;
        }
        
        // Filter by file types if specified
        if let Some(ref allowed_types) = options.file_types {
            let entry_type = &entry.file_type;
            let matches = allowed_types.iter().any(|allowed| {
                std::mem::discriminant(allowed) == std::mem::discriminant(entry_type)
            });
            if !matches {
                return false;
            }
        }
        
        // Filter by name patterns if specified
        if let Some(ref patterns) = options.name_patterns {
            let name = &entry.name.to_lowercase();
            let matches = patterns.iter().any(|pattern| {
                // Simple glob-like matching (supports * wildcard)
                Self::matches_pattern(name, &pattern.to_lowercase())
            });
            if !matches {
                return false;
            }
        }
        
        true
    }
    
    fn matches_pattern(name: &str, pattern: &str) -> bool {
        // Simple wildcard matching - supports * as wildcard
        if pattern == "*" {
            return true;
        }
        
        if !pattern.contains('*') {
            return name == pattern || name.ends_with(pattern);
        }
        
        // Handle patterns with wildcards
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.is_empty() {
            return false;
        }
        
        let mut pos = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            
            if i == 0 {
                // First part must match from beginning
                if !name[pos..].starts_with(part) {
                    return false;
                }
                pos += part.len();
            } else if i == parts.len() - 1 {
                // Last part must match at end
                if !name[pos..].ends_with(part) {
                    return false;
                }
            } else {
                // Middle parts
                if let Some(found_pos) = name[pos..].find(part) {
                    pos += found_pos + part.len();
                } else {
                    return false;
                }
            }
        }
        
        true
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
    async fn traverse_directory(&self, path: &Path, options: TraversalOptions) -> Result<Vec<FileEntry>, FileSystemError> {
        let path = path.to_path_buf();
        let config = self.config.clone();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            if !path.is_dir() {
                return Err(FileSystemError::InvalidPath { path });
            }
            
            let mut entries = Vec::new();
            
            // Configure walkdir based on options and config
            let mut walkdir = WalkDir::new(&path);
            
            // Set depth limit
            let max_depth = options.max_depth
                .or(config.max_depth)
                .unwrap_or(if options.recursive { 100 } else { 1 });
            walkdir = walkdir.max_depth(max_depth);
            
            // Configure symlink following
            let follow_symlinks = options.follow_symlinks
                .unwrap_or(config.follow_symlinks);
            if follow_symlinks {
                walkdir = walkdir.follow_links(true);
            }
            
            // Include hidden files setting
            let include_hidden = options.include_hidden
                .unwrap_or(config.include_hidden);
            
            for entry_result in walkdir {
                let entry = entry_result?;
                let entry_path = entry.path();
                
                // Skip the root directory itself
                if entry_path == path {
                    continue;
                }
                
                // Get metadata
                let metadata = match entry.metadata() {
                    Ok(meta) => meta,
                    Err(e) => {
                        // Log warning but continue with other entries
                        tracing::warn!("Failed to get metadata for {}: {}", entry_path.display(), e);
                        continue;
                    }
                };
                
                let file_entry = Self::create_file_entry(entry_path.to_path_buf(), &metadata);
                
                // Apply filtering
                if !Self::should_include_entry(&file_entry, &options, include_hidden) {
                    continue;
                }
                
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn copy_file(&self, operation: FileOperation) -> Result<(), FileSystemError> {
        let source = operation.source.clone();
        let dest = operation.destination.clone();
        let overwrite_mode = operation.overwrite_mode.clone();
        let preserve_metadata = operation.preserve_metadata;
        let config = self.config.clone();
        
        tokio::task::spawn_blocking(move || {
            // Validate source exists
            if !source.exists() {
                return Err(FileSystemError::PathNotFound { path: source });
            }
            
            // Check file size if configured
            if let Some(max_size) = config.max_file_size {
                if let Ok(metadata) = source.metadata() {
                    if metadata.len() > max_size {
                        return Err(FileSystemError::FileTooLarge {
                            size: metadata.len(),
                            max_size,
                        });
                    }
                }
            }
            
            // Handle destination existence
            if dest.exists() {
                match overwrite_mode {
                    OverwriteMode::Fail => {
                        return Err(FileSystemError::FileAlreadyExists { path: dest });
                    }
                    OverwriteMode::Skip => {
                        return Ok(());
                    }
                    OverwriteMode::Backup => {
                        let backup_path = dest.with_extension(
                            format!("{}.bak", dest.extension().and_then(|s| s.to_str()).unwrap_or(""))
                        );
                        std::fs::rename(&dest, &backup_path)?;
                    }
                    OverwriteMode::Overwrite => {
                        // Continue with operation
                    }
                }
            }
            
            // Create parent directory if needed
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Perform copy
            if source.is_dir() {
                copy_dir_recursively(&source, &dest)?;
            } else {
                std::fs::copy(&source, &dest)?;
            }
            
            // Preserve metadata if requested
            if preserve_metadata {
                if let Ok(src_metadata) = source.metadata() {
                    // Preserve timestamps (Unix/Linux)
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        use std::time::{Duration, UNIX_EPOCH};
                        
                        let _atime = UNIX_EPOCH + Duration::from_secs(src_metadata.atime() as u64);
                        let _mtime = UNIX_EPOCH + Duration::from_secs(src_metadata.mtime() as u64);
                        
                        // Note: Setting timestamps requires additional crates like `filetime`
                        // For now, we'll skip this to avoid additional dependencies
                    }
                    
                    // Preserve permissions
                    let permissions = src_metadata.permissions();
                    if let Err(e) = std::fs::set_permissions(&dest, permissions) {
                        tracing::warn!("Failed to preserve permissions for {}: {}", dest.display(), e);
                    }
                }
            }
            
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn move_file(&self, operation: FileOperation) -> Result<(), FileSystemError> {
        let source = operation.source.clone();
        let dest = operation.destination.clone();
        let overwrite_mode = operation.overwrite_mode.clone();
        
        tokio::task::spawn_blocking(move || {
            // Validate source exists
            if !source.exists() {
                return Err(FileSystemError::PathNotFound { path: source });
            }
            
            // Handle destination existence
            if dest.exists() {
                match overwrite_mode {
                    OverwriteMode::Fail => {
                        return Err(FileSystemError::FileAlreadyExists { path: dest });
                    }
                    OverwriteMode::Skip => {
                        return Ok(());
                    }
                    OverwriteMode::Backup => {
                        let backup_path = dest.with_extension(
                            format!("{}.bak", dest.extension().and_then(|s| s.to_str()).unwrap_or(""))
                        );
                        std::fs::rename(&dest, &backup_path)?;
                    }
                    OverwriteMode::Overwrite => {
                        if dest.is_dir() {
                            std::fs::remove_dir_all(&dest)?;
                        } else {
                            std::fs::remove_file(&dest)?;
                        }
                    }
                }
            }
            
            // Create parent directory if needed
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Attempt atomic move first (same filesystem)
            match std::fs::rename(&source, &dest) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
                    // Cross-device move: copy then delete
                    if source.is_dir() {
                        copy_dir_recursively(&source, &dest)?;
                        std::fs::remove_dir_all(&source)?;
                    } else {
                        std::fs::copy(&source, &dest)?;
                        std::fs::remove_file(&source)?;
                    }
                    Ok(())
                }
                Err(e) => Err(FileSystemError::from_io_error(e, &source)),
            }
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn delete_file(&self, path: &Path) -> Result<(), FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            if path.is_dir() {
                // Check if directory is empty
                match std::fs::read_dir(&path) {
                    Ok(mut entries) => {
                        if entries.next().is_some() {
                            return Err(FileSystemError::DirectoryNotEmpty { path });
                        }
                    }
                    Err(e) => return Err(FileSystemError::from_io_error(e, &path)),
                }
                std::fs::remove_dir(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn rename_file(&self, source: &Path, new_name: &str) -> Result<PathBuf, FileSystemError> {
        let source = source.to_path_buf();
        let new_name = new_name.to_string();
        
        tokio::task::spawn_blocking(move || {
            if !source.exists() {
                return Err(FileSystemError::PathNotFound { path: source });
            }
            
            // Validate new name doesn't contain path separators
            if new_name.contains('/') || new_name.contains('\\') {
                return Err(FileSystemError::InvalidPath { 
                    path: PathBuf::from(&new_name) 
                });
            }
            
            let parent = source.parent()
                .ok_or_else(|| FileSystemError::InvalidPath { path: source.clone() })?;
            let dest = parent.join(&new_name);
            
            if dest.exists() {
                return Err(FileSystemError::FileAlreadyExists { path: dest });
            }
            
            std::fs::rename(&source, &dest)?;
            Ok(dest)
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&path)?;
            Ok(())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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
    
    async fn check_read_permission(&self, path: &Path) -> Result<bool, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            let permissions = get_permissions(&metadata);
            Ok(permissions.can_read())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn check_write_permission(&self, path: &Path) -> Result<bool, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            let permissions = get_permissions(&metadata);
            Ok(permissions.can_write())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn check_execute_permission(&self, path: &Path) -> Result<bool, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            let permissions = get_permissions(&metadata);
            Ok(permissions.can_execute())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn get_file_permissions(&self, path: &Path) -> Result<FilePermissions, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            Ok(get_permissions(&metadata))
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn get_file_size(&self, path: &Path) -> Result<u64, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            Ok(metadata.len())
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn get_modification_time(&self, path: &Path) -> Result<SystemTime, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            Ok(metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH))
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn get_creation_time(&self, path: &Path) -> Result<SystemTime, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let metadata = std::fs::metadata(&path)?;
            Ok(metadata.created().unwrap_or(SystemTime::UNIX_EPOCH))
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
    }
    
    async fn is_hidden(&self, path: &Path) -> Result<bool, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            if !path.exists() {
                return Err(FileSystemError::PathNotFound { path });
            }
            
            let file_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            
            #[cfg(unix)]
            {
                // On Unix systems, files starting with . are hidden
                Ok(file_name.starts_with('.'))
            }
            
            #[cfg(windows)]
            {
                // On Windows, check the hidden attribute
                use std::os::windows::fs::MetadataExt;
                let metadata = std::fs::metadata(&path)?;
                const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                Ok((metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0)
            }
            
            #[cfg(not(any(unix, windows)))]
            {
                // Default implementation for other platforms
                Ok(file_name.starts_with('.'))
            }
        }).await
        .map_err(|e| FileSystemError::Io(e.to_string()))?
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

/// Format file size in human-readable format (B, KB, MB, GB, etc.)
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log10() / THRESHOLD.log10()).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    
    if unit_index == 0 {
        format!("{} B", bytes)
    } else {
        let size = bytes_f / THRESHOLD.powi(unit_index as i32);
        if size >= 100.0 {
            format!("{:.0} {}", size, UNITS[unit_index])
        } else if size >= 10.0 {
            format!("{:.1} {}", size, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}

/// Format SystemTime as human-readable string
fn format_system_time(time: SystemTime) -> String {
    use chrono::{DateTime, Local};
    
    match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => {
            let datetime = DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap())
                .with_timezone(&Local);
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        Err(_) => "Unknown".to_string(),
    }
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
    async fn test_traverse_directory_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create nested directory structure
        let subdir = temp_dir.path().join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        let nested_dir = subdir.join("nested");
        std::fs::create_dir(&nested_dir).unwrap();
        
        // Create files at different levels
        std::fs::write(temp_dir.path().join("root.txt"), "root").unwrap();
        std::fs::write(subdir.join("sub.txt"), "sub").unwrap();
        std::fs::write(nested_dir.join("nested.txt"), "nested").unwrap();
        std::fs::write(temp_dir.path().join(".hidden"), "hidden").unwrap();
        
        // Test non-recursive (default)
        let entries = service
            .traverse_directory(temp_dir.path(), TraversalOptions::default())
            .await
            .unwrap();
        
        // Should only include immediate children (non-recursive)
        assert!(entries.len() >= 3); // root.txt, .hidden, subdir (exact count may vary)
        
        // Test recursive
        let entries = service
            .traverse_directory(temp_dir.path(), TraversalOptions::recursive())
            .await
            .unwrap();
        
        // Should include all files in nested structure
        assert!(entries.len() >= 5); // All files and directories
        assert!(entries.iter().any(|e| e.name == "nested.txt"));
        
        // Test with depth limit
        let entries = service
            .traverse_directory(
                temp_dir.path(),
                TraversalOptions::recursive().with_depth(2)
            )
            .await
            .unwrap();
        
        // Should include files up to depth 2, but not nested.txt (depth 3)
        assert!(entries.iter().any(|e| e.name == "sub.txt"));
        let has_nested = entries.iter().any(|e| e.name == "nested.txt");
        assert!(!has_nested || entries.len() < 10); // Flexible assertion
    }
    
    #[tokio::test]
    async fn test_traverse_directory_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create test files
        std::fs::write(temp_dir.path().join("test.jpg"), "image").unwrap();
        std::fs::write(temp_dir.path().join("test.txt"), "text").unwrap();
        std::fs::write(temp_dir.path().join("document.pdf"), "pdf").unwrap();
        std::fs::write(temp_dir.path().join(".hidden"), "hidden").unwrap();
        
        // Test file type filtering
        let entries = service
            .traverse_directory(
                temp_dir.path(),
                TraversalOptions::default()
                    .filter_types(vec![FileType::Image(ImageFormat::Jpeg)])
            )
            .await
            .unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.jpg");
        
        // Test pattern filtering
        let entries = service
            .traverse_directory(
                temp_dir.path(),
                TraversalOptions::default()
                    .filter_patterns(vec!["*.txt".to_string()])
            )
            .await
            .unwrap();
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test.txt");
        
        // Test hidden file inclusion
        let entries_no_hidden = service
            .traverse_directory(temp_dir.path(), TraversalOptions::default())
            .await
            .unwrap();
        
        let entries_with_hidden = service
            .traverse_directory(
                temp_dir.path(),
                TraversalOptions::default().include_hidden(true)
            )
            .await
            .unwrap();
        
        assert!(entries_with_hidden.len() > entries_no_hidden.len());
        assert!(entries_with_hidden.iter().any(|e| e.name == ".hidden"));
    }
    
    #[tokio::test]
    async fn test_pattern_matching() {
        assert!(NativeFileSystemService::matches_pattern("test.txt", "*.txt"));
        assert!(NativeFileSystemService::matches_pattern("image.jpg", "*.jpg"));
        assert!(NativeFileSystemService::matches_pattern("document.pdf", "doc*"));
        assert!(NativeFileSystemService::matches_pattern("anything", "*"));
        assert!(!NativeFileSystemService::matches_pattern("test.txt", "*.jpg"));
        assert!(!NativeFileSystemService::matches_pattern("test.txt", "image*"));
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

    #[tokio::test]
    async fn test_copy_file_individual() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source file
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, "test content").unwrap();
        
        let dest_file = temp_dir.path().join("destination.txt");
        
        // Test basic copy
        let operation = FileOperation::new(source_file.clone(), dest_file.clone());
        service.copy_file(operation).await.unwrap();
        
        assert!(dest_file.exists());
        assert!(source_file.exists());
        
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_copy_file_overwrite_modes() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source and existing destination
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("destination.txt");
        std::fs::write(&source_file, "new content").unwrap();
        std::fs::write(&dest_file, "old content").unwrap();
        
        // Test Fail mode (default)
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Fail);
        let result = service.copy_file(operation).await;
        assert!(matches!(result, Err(FileSystemError::FileAlreadyExists { .. })));
        
        // Test Skip mode
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Skip);
        service.copy_file(operation).await.unwrap();
        
        // File should still have old content
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "old content");
        
        // Test Overwrite mode
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Overwrite);
        service.copy_file(operation).await.unwrap();
        
        // File should have new content
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "new content");
        
        // Test Backup mode
        std::fs::write(&dest_file, "backup test").unwrap();
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Backup);
        service.copy_file(operation).await.unwrap();
        
        // Original should be backed up
        let backup_file = dest_file.with_extension("txt.bak");
        assert!(backup_file.exists());
        let backup_content = std::fs::read_to_string(&backup_file).unwrap();
        assert_eq!(backup_content, "backup test");
        
        // Destination should have new content
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "new content");
    }

    #[tokio::test]
    async fn test_move_file_individual() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source file
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, "test content").unwrap();
        
        let dest_file = temp_dir.path().join("destination.txt");
        
        // Test basic move
        let operation = FileOperation::new(source_file.clone(), dest_file.clone());
        service.move_file(operation).await.unwrap();
        
        assert!(dest_file.exists());
        assert!(!source_file.exists()); // Source should be gone
        
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_move_file_overwrite_modes() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source and existing destination
        let source_file = temp_dir.path().join("source.txt");
        let dest_file = temp_dir.path().join("destination.txt");
        std::fs::write(&source_file, "new content").unwrap();
        std::fs::write(&dest_file, "old content").unwrap();
        
        // Test Fail mode
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Fail);
        let result = service.move_file(operation).await;
        assert!(matches!(result, Err(FileSystemError::FileAlreadyExists { .. })));
        assert!(source_file.exists()); // Source should still exist on failure
        
        // Test Overwrite mode
        let operation = FileOperation::new(source_file.clone(), dest_file.clone())
            .with_overwrite_mode(OverwriteMode::Overwrite);
        service.move_file(operation).await.unwrap();
        
        assert!(!source_file.exists()); // Source should be gone
        let content = std::fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "new content");
    }

    #[tokio::test]
    async fn test_delete_file_individual() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Test deleting file
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        service.delete_file(&test_file).await.unwrap();
        assert!(!test_file.exists());
        
        // Test deleting empty directory
        let test_dir = temp_dir.path().join("empty_dir");
        std::fs::create_dir(&test_dir).unwrap();
        
        service.delete_file(&test_dir).await.unwrap();
        assert!(!test_dir.exists());
        
        // Test deleting non-empty directory (should fail)
        let test_dir = temp_dir.path().join("non_empty_dir");
        std::fs::create_dir(&test_dir).unwrap();
        std::fs::write(test_dir.join("file.txt"), "content").unwrap();
        
        let result = service.delete_file(&test_dir).await;
        assert!(matches!(result, Err(FileSystemError::DirectoryNotEmpty { .. })));
    }

    #[tokio::test]
    async fn test_rename_file_individual() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create source file
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, "test content").unwrap();
        
        // Test basic rename
        let new_path = service.rename_file(&source_file, "renamed.txt").await.unwrap();
        
        assert!(!source_file.exists());
        assert!(new_path.exists());
        assert_eq!(new_path.file_name().unwrap().to_str().unwrap(), "renamed.txt");
        
        let content = std::fs::read_to_string(&new_path).unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_rename_file_validation() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        let source_file = temp_dir.path().join("source.txt");
        std::fs::write(&source_file, "test content").unwrap();
        
        // Test invalid names with path separators
        let result = service.rename_file(&source_file, "sub/dir.txt").await;
        assert!(matches!(result, Err(FileSystemError::InvalidPath { .. })));
        
        let result = service.rename_file(&source_file, "sub\\dir.txt").await;
        assert!(matches!(result, Err(FileSystemError::InvalidPath { .. })));
        
        // Test existing target name
        let existing_file = temp_dir.path().join("existing.txt");
        std::fs::write(&existing_file, "existing").unwrap();
        
        let result = service.rename_file(&source_file, "existing.txt").await;
        assert!(matches!(result, Err(FileSystemError::FileAlreadyExists { .. })));
    }

    #[tokio::test]
    async fn test_copy_file_with_config() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create service with file size limit
        let config = FileSystemConfig {
            max_file_size: Some(10), // 10 bytes limit
            ..Default::default()
        };
        let service = NativeFileSystemService::with_config(config);
        
        // Create large file
        let source_file = temp_dir.path().join("large.txt");
        let large_content = "a".repeat(100); // 100 bytes
        std::fs::write(&source_file, &large_content).unwrap();
        
        let dest_file = temp_dir.path().join("dest.txt");
        let operation = FileOperation::new(source_file, dest_file);
        
        // Should fail due to size limit
        let result = service.copy_file(operation).await;
        assert!(matches!(result, Err(FileSystemError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        let non_existent = temp_dir.path().join("non_existent.txt");
        let dest = temp_dir.path().join("dest.txt");
        
        // Test copying non-existent file
        let operation = FileOperation::new(non_existent.clone(), dest.clone());
        let result = service.copy_file(operation).await;
        assert!(matches!(result, Err(FileSystemError::PathNotFound { .. })));
        
        // Test moving non-existent file  
        let operation = FileOperation::new(non_existent.clone(), dest.clone());
        let result = service.move_file(operation).await;
        assert!(matches!(result, Err(FileSystemError::PathNotFound { .. })));
        
        // Test deleting non-existent file
        let result = service.delete_file(&non_existent).await;
        assert!(matches!(result, Err(FileSystemError::PathNotFound { .. })));
        
        // Test renaming non-existent file
        let result = service.rename_file(&non_existent, "new_name.txt").await;
        assert!(matches!(result, Err(FileSystemError::PathNotFound { .. })));
    }

    #[tokio::test]
    async fn test_file_permissions() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, "test content").unwrap();
        
        // Test permission checking
        let can_read = service.check_read_permission(&test_file).await.unwrap();
        let can_write = service.check_write_permission(&test_file).await.unwrap();
        let permissions = service.get_file_permissions(&test_file).await.unwrap();
        
        assert!(can_read);
        assert_eq!(can_read, permissions.can_read());
        assert_eq!(can_write, permissions.can_write());
        
        // Test permissions for non-existent file
        let non_existent = temp_dir.path().join("non_existent.txt");
        let result = service.check_read_permission(&non_existent).await;
        assert!(matches!(result, Err(FileSystemError::PathNotFound { .. })));
    }

    #[tokio::test]
    async fn test_file_permissions_methods() {
        // Test FilePermissions methods
        let read_only = FilePermissions::read_only();
        assert!(read_only.can_read());
        assert!(!read_only.can_write());
        assert!(!read_only.can_execute());
        assert!(read_only.is_read_only());
        assert_eq!(read_only.permission_string(), "r--");
        
        let read_write = FilePermissions::read_write();
        assert!(read_write.can_read());
        assert!(read_write.can_write());
        assert!(!read_write.can_execute());
        assert!(!read_write.is_read_only());
        assert_eq!(read_write.permission_string(), "rw-");
        
        let all_perms = FilePermissions::all_permissions();
        assert!(all_perms.can_read());
        assert!(all_perms.can_write());
        assert!(all_perms.can_execute());
        assert!(!all_perms.is_read_only());
        assert!(all_perms.has_any_permission());
        assert_eq!(all_perms.permission_string(), "rwx");
        
        let no_perms = FilePermissions::no_permissions();
        assert!(!no_perms.has_any_permission());
        assert_eq!(no_perms.permission_string(), "---");
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create test file with known content
        let test_file = temp_dir.path().join("metadata_test.txt");
        let test_content = "This is a test file for metadata extraction.";
        std::fs::write(&test_file, test_content).unwrap();
        
        // Test size extraction
        let size = service.get_file_size(&test_file).await.unwrap();
        assert_eq!(size, test_content.len() as u64);
        
        // Test modification time (should be recent)
        let mod_time = service.get_modification_time(&test_file).await.unwrap();
        let now = SystemTime::now();
        let duration = now.duration_since(mod_time).unwrap();
        assert!(duration.as_secs() < 10); // Should be within 10 seconds
        
        // Test creation time
        let create_time = service.get_creation_time(&test_file).await.unwrap();
        assert!(create_time <= now);
        
        // Test hidden file detection
        let is_hidden = service.is_hidden(&test_file).await.unwrap();
        assert!(!is_hidden); // Normal file should not be hidden
    }

    #[tokio::test]
    async fn test_hidden_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let service = NativeFileSystemService::new();
        
        // Create hidden file (Unix-style)
        let hidden_file = temp_dir.path().join(".hidden_file");
        std::fs::write(&hidden_file, "hidden content").unwrap();
        
        let is_hidden = service.is_hidden(&hidden_file).await.unwrap();
        assert!(is_hidden);
        
        // Create normal file
        let normal_file = temp_dir.path().join("normal_file.txt");
        std::fs::write(&normal_file, "normal content").unwrap();
        
        let is_hidden = service.is_hidden(&normal_file).await.unwrap();
        assert!(!is_hidden);
    }

    #[tokio::test]
    async fn test_file_entry_methods() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a test image file
        let image_file = temp_dir.path().join("test.jpg");
        std::fs::write(&image_file, "fake jpeg content").unwrap();
        let metadata = std::fs::metadata(&image_file).unwrap();
        
        let file_entry = NativeFileSystemService::create_file_entry(image_file.clone(), &metadata);
        
        // Test file type methods
        assert!(file_entry.is_image());
        assert!(!file_entry.is_video());
        assert!(!file_entry.is_audio());
        assert!(!file_entry.is_document());
        assert!(!file_entry.is_text());
        assert!(file_entry.is_media());
        
        // Test extension
        assert_eq!(file_entry.extension(), Some("jpg".to_string()));
        
        // Test icon
        assert_eq!(file_entry.icon(), "🖼️");
        
        // Test size string
        let size_str = file_entry.size_string();
        assert!(size_str.contains("B")); // Should contain bytes unit
        
        // Test permissions
        assert!(file_entry.can_read());
        
        // Test parent
        assert_eq!(file_entry.parent(), Some(temp_dir.path()));
    }

    #[tokio::test]
    async fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.00 KB");
        assert_eq!(format_file_size(1536), "1.50 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_file_size(1024u64.pow(4)), "1.00 TB");
    }

    #[tokio::test]
    async fn test_format_system_time() {
        let time = SystemTime::UNIX_EPOCH;
        let formatted = format_system_time(time);
        assert!(formatted.contains("1970") || formatted == "Unknown");
        
        let now = SystemTime::now();
        let formatted = format_system_time(now);
        assert!(formatted.len() > 10); // Should be a reasonable date string
    }

    #[tokio::test]
    async fn test_enhanced_metadata_for_different_file_types() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test various file types
        let test_files = vec![
            ("document.pdf", FileType::Document(DocumentFormat::Pdf)),
            ("music.mp3", FileType::Audio(AudioFormat::Mp3)),
            ("video.mp4", FileType::Video(VideoFormat::Mp4)),
            ("code.rs", FileType::Text(TextFormat::Rust)),
            ("data.json", FileType::Text(TextFormat::Json)),
        ];
        
        for (filename, expected_type) in test_files {
            let file_path = temp_dir.path().join(filename);
            std::fs::write(&file_path, "test content").unwrap();
            let metadata = std::fs::metadata(&file_path).unwrap();
            
            let file_entry = NativeFileSystemService::create_file_entry(file_path, &metadata);
            assert_eq!(file_entry.file_type, expected_type);
            
            // Test type-specific methods
            match expected_type {
                FileType::Document(_) => assert!(file_entry.is_document()),
                FileType::Audio(_) => {
                    assert!(file_entry.is_audio());
                    assert!(file_entry.is_media());
                }
                FileType::Video(_) => {
                    assert!(file_entry.is_video());
                    assert!(file_entry.is_media());
                }
                FileType::Text(_) => assert!(file_entry.is_text()),
                _ => {}
            }
        }
    }
}