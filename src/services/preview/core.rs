use std::path::{Path, PathBuf};
use std::time::SystemTime;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;

/// Comprehensive preview service for multi-format file support
/// Supports images, videos, audio, PDFs, and text files with metadata extraction

#[derive(Debug, Error)]
pub enum PreviewError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("Failed to read file: {0}")]
    ReadError(String),
    #[error("Image processing error: {0}")]
    ImageError(String),
    #[error("Video processing error: {0}")]
    VideoError(String),
    #[error("Audio processing error: {0}")]
    AudioError(String),
    #[error("PDF processing error: {0}")]
    PdfError(String),
    #[error("Text processing error: {0}")]
    TextError(String),
    #[error("Metadata extraction error: {0}")]
    MetadataError(String),
    #[error("Thumbnail generation error: {0}")]
    ThumbnailError(String),
    #[error("Background task error: {0}")]
    TaskError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Supported file formats for preview generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupportedFormat {
    // Image formats
    Jpeg,
    Png,
    Gif,
    WebP,
    Tiff,
    Bmp,
    Svg,
    // Video formats
    Mp4,
    Avi,
    Mkv,
    Mov,
    Wmv,
    Flv,
    WebM,
    // Audio formats
    Mp3,
    Wav,
    Flac,
    Aac,
    Ogg,
    M4a,
    // Document formats
    Pdf,
    // Text formats
    Text,
    Markdown,
    Json,
    Xml,
    Html,
    Css,
    Javascript,
    Rust,
    Python,
    Cpp,
    Java,
}

impl SupportedFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            // Images
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "png" => Some(Self::Png),
            "gif" => Some(Self::Gif),
            "webp" => Some(Self::WebP),
            "tiff" | "tif" => Some(Self::Tiff),
            "bmp" => Some(Self::Bmp),
            "svg" => Some(Self::Svg),
            // Videos
            "mp4" => Some(Self::Mp4),
            "avi" => Some(Self::Avi),
            "mkv" => Some(Self::Mkv),
            "mov" => Some(Self::Mov),
            "wmv" => Some(Self::Wmv),
            "flv" => Some(Self::Flv),
            "webm" => Some(Self::WebM),
            // Audio
            "mp3" => Some(Self::Mp3),
            "wav" => Some(Self::Wav),
            "flac" => Some(Self::Flac),
            "aac" => Some(Self::Aac),
            "ogg" => Some(Self::Ogg),
            "m4a" => Some(Self::M4a),
            // Documents
            "pdf" => Some(Self::Pdf),
            // Text
            "txt" => Some(Self::Text),
            "md" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "xml" => Some(Self::Xml),
            "html" | "htm" => Some(Self::Html),
            "css" => Some(Self::Css),
            "js" => Some(Self::Javascript),
            "rs" => Some(Self::Rust),
            "py" => Some(Self::Python),
            "cpp" | "cc" | "cxx" => Some(Self::Cpp),
            "java" => Some(Self::Java),
            _ => None,
        }
    }

    /// Check if format is an image type
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Jpeg | Self::Png | Self::Gif | Self::WebP | Self::Tiff | Self::Bmp | Self::Svg)
    }

    /// Check if format is a video type
    pub fn is_video(&self) -> bool {
        matches!(self, Self::Mp4 | Self::Avi | Self::Mkv | Self::Mov | Self::Wmv | Self::Flv | Self::WebM)
    }

    /// Check if format is an audio type
    pub fn is_audio(&self) -> bool {
        matches!(self, Self::Mp3 | Self::Wav | Self::Flac | Self::Aac | Self::Ogg | Self::M4a)
    }

    /// Check if format is a document type
    pub fn is_document(&self) -> bool {
        matches!(self, Self::Pdf)
    }

    /// Check if format is a text type
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text | Self::Markdown | Self::Json | Self::Xml | Self::Html | Self::Css | Self::Javascript | Self::Rust | Self::Python | Self::Cpp | Self::Java)
    }
}

/// Preview data containing generated preview and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewData {
    pub file_path: PathBuf,
    pub format: SupportedFormat,
    pub thumbnail_path: Option<PathBuf>,
    pub metadata: FileMetadata,
    pub preview_content: PreviewContent,
    pub generated_at: SystemTime,
}

/// File metadata extracted during preview generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f64>, // seconds
    pub bit_rate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub codec: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<u32>,
    pub page_count: Option<u32>,
    pub color_space: Option<String>,
    pub compression: Option<String>,
    pub exif_data: Option<ExifData>,
}

impl FileMetadata {
    pub fn new() -> Self {
        Self {
            file_size: 0,
            created: None,
            modified: None,
            width: None,
            height: None,
            duration: None,
            bit_rate: None,
            sample_rate: None,
            codec: None,
            title: None,
            artist: None,
            album: None,
            year: None,
            page_count: None,
            color_space: None,
            compression: None,
            exif_data: None,
        }
    }
}

/// EXIF data for images
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifData {
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub focal_length: Option<f32>,
    pub aperture: Option<f32>,
    pub shutter_speed: Option<String>,
    pub iso: Option<u32>,
    pub flash: Option<bool>,
    pub date_taken: Option<SystemTime>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub orientation: Option<u32>,
}

/// Preview content specific to file type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewContent {
    Image {
        /// Base64 encoded thumbnail image
        thumbnail_data: Vec<u8>,
        /// Image format info
        original_format: String,
    },
    Video {
        /// Frame thumbnails at different timestamps
        thumbnails: Vec<VideoThumbnail>,
        /// Video stream information
        streams: Vec<String>,
    },
    Audio {
        /// Waveform data points for visualization
        waveform_data: Vec<f32>,
        /// Audio sample for preview
        sample_data: Option<Vec<u8>>,
    },
    Document {
        /// First page as image
        first_page_image: Vec<u8>,
        /// Document outline/table of contents
        outline: Vec<String>,
    },
    Text {
        /// First few lines of text content
        preview_text: String,
        /// Detected language for syntax highlighting
        language: Option<String>,
        /// Text encoding
        encoding: String,
    },
}

/// Video thumbnail at specific timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoThumbnail {
    pub timestamp: f64, // seconds
    pub thumbnail_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Configuration for preview generation
#[derive(Debug)]
pub struct PreviewConfig {
    pub thumbnail_size: (u32, u32),
    pub max_preview_text_length: usize,
    pub video_thumbnail_count: usize,
    pub audio_waveform_samples: usize,
    pub background_processing: bool,
    pub cache_thumbnails: bool,
    pub max_file_size: u64, // bytes
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            thumbnail_size: (256, 256),
            max_preview_text_length: 1000,
            video_thumbnail_count: 5,
            audio_waveform_samples: 1000,
            background_processing: true,
            cache_thumbnails: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Main trait for preview handlers
#[async_trait]
pub trait PreviewHandler: Send + Sync {
    /// Check if this handler supports the given format
    fn supports_format(&self, format: SupportedFormat) -> bool;
    
    /// Generate preview for the given file
    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError>;
    
    /// Extract metadata without generating full preview
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError>;
    
    /// Generate thumbnail only (faster than full preview)
    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError>;
}

/// Background task handle for async thumbnail generation
pub struct ThumbnailTask {
    pub file_path: PathBuf,
    pub handle: JoinHandle<Result<PreviewData, PreviewError>>,
}

impl ThumbnailTask {
    pub fn new(file_path: PathBuf, handle: JoinHandle<Result<PreviewData, PreviewError>>) -> Self {
        Self { file_path, handle }
    }
    
    /// Check if the task is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
    
    /// Get the result if the task is complete
    pub async fn await_result(self) -> Result<PreviewData, PreviewError> {
        match self.handle.await {
            Ok(result) => result,
            Err(join_error) => Err(PreviewError::TaskError(join_error.to_string())),
        }
    }
}

/// Main preview service coordinating all preview handlers
pub struct PreviewService {
    handlers: Vec<Box<dyn PreviewHandler>>,
    config: PreviewConfig,
    cache_service: Option<crate::services::cache::CacheService>,
}

impl PreviewService {
    /// Create new preview service with default configuration
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            config: PreviewConfig::default(),
            cache_service: None,
        }
    }
    
    /// Create preview service with custom configuration
    pub fn with_config(config: PreviewConfig) -> Self {
        Self {
            handlers: Vec::new(),
            config,
            cache_service: None,
        }
    }
    
    /// Add cache service for thumbnail caching
    pub fn with_cache(mut self, cache_service: crate::services::cache::CacheService) -> Self {
        self.cache_service = Some(cache_service);
        self
    }
    
    /// Register a preview handler
    pub fn register_handler(&mut self, handler: Box<dyn PreviewHandler>) {
        self.handlers.push(handler);
    }
    
    /// Detect file format from path
    pub fn detect_format<P: AsRef<Path>>(&self, file_path: P) -> Option<SupportedFormat> {
        file_path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(SupportedFormat::from_extension)
    }
    
    /// Check if file format is supported
    pub fn is_supported<P: AsRef<Path>>(&self, file_path: P) -> bool {
        if let Some(format) = self.detect_format(file_path) {
            self.handlers.iter().any(|handler| handler.supports_format(format))
        } else {
            false
        }
    }
    
    /// Generate preview for a file
    pub async fn generate_preview<P: AsRef<Path>>(&self, file_path: P) -> Result<PreviewData, PreviewError> {
        let path = file_path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Err(PreviewError::FileNotFound(path.to_path_buf()));
        }
        
        // Detect format
        let format = self.detect_format(path)
            .ok_or_else(|| PreviewError::UnsupportedFormat(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;
        
        // Find appropriate handler
        let handler = self.handlers.iter()
            .find(|handler| handler.supports_format(format))
            .ok_or_else(|| PreviewError::UnsupportedFormat(format!("{:?}", format)))?;
        
        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > self.config.max_file_size {
            return Err(PreviewError::ReadError(format!("File too large: {} bytes", metadata.len())));
        }
        
        // Check cache first if available
        if let Some(cache) = &self.cache_service {
            if let Ok(Some(cached_thumbnail)) = cache.get_thumbnail_path(path).await {
                // TODO: Check if cached preview is still valid
                // For now, generate fresh preview
                tracing::debug!("Found cached thumbnail for {:?}: {:?}", path, cached_thumbnail.thumbnail_path);
            }
        }
        
        // Generate preview
        let preview_data = handler.generate_preview(path, &self.config).await?;
        
        // Cache thumbnail if caching is enabled
        if self.config.cache_thumbnails {
            if let (Some(_cache), Some(_thumbnail_path)) = (&self.cache_service, &preview_data.thumbnail_path) {
                // TODO: Implement thumbnail path caching once we have the correct API
                // The current cache API expects CachedThumbnail struct, not separate path params
                tracing::debug!("Thumbnail caching not yet implemented");
            }
        }
        
        Ok(preview_data)
    }
    
    /// Generate preview in background (non-blocking)
    pub fn generate_preview_background<P: AsRef<Path>>(&self, file_path: P) -> ThumbnailTask {
        let path = file_path.as_ref().to_path_buf();
        let path_clone = path.clone();
        let service = self.clone_for_background();
        
        let handle = tokio::spawn(async move {
            service.generate_preview(&path_clone).await
        });
        
        ThumbnailTask::new(path, handle)
    }
    
    /// Extract metadata only (faster than full preview)
    pub async fn extract_metadata<P: AsRef<Path>>(&self, file_path: P) -> Result<FileMetadata, PreviewError> {
        let path = file_path.as_ref();
        
        // Check if file exists
        if !path.exists() {
            return Err(PreviewError::FileNotFound(path.to_path_buf()));
        }
        
        // Detect format
        let format = self.detect_format(path)
            .ok_or_else(|| PreviewError::UnsupportedFormat(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;
        
        // Find appropriate handler
        let handler = self.handlers.iter()
            .find(|handler| handler.supports_format(format))
            .ok_or_else(|| PreviewError::UnsupportedFormat(format!("{:?}", format)))?;
        
        handler.extract_metadata(path).await
    }
    
    /// Generate thumbnail only
    pub async fn generate_thumbnail<P: AsRef<Path>>(&self, file_path: P, size: Option<(u32, u32)>) -> Result<Vec<u8>, PreviewError> {
        let path = file_path.as_ref();
        let thumbnail_size = size.unwrap_or(self.config.thumbnail_size);
        
        // Check if file exists
        if !path.exists() {
            return Err(PreviewError::FileNotFound(path.to_path_buf()));
        }
        
        // Detect format
        let format = self.detect_format(path)
            .ok_or_else(|| PreviewError::UnsupportedFormat(
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            ))?;
        
        // Find appropriate handler
        let handler = self.handlers.iter()
            .find(|handler| handler.supports_format(format))
            .ok_or_else(|| PreviewError::UnsupportedFormat(format!("{:?}", format)))?;
        
        handler.generate_thumbnail(path, thumbnail_size).await
    }
    
    /// Get list of all supported formats
    pub fn supported_formats(&self) -> Vec<SupportedFormat> {
        use SupportedFormat::*;
        vec![
            // Images
            Jpeg, Png, Gif, WebP, Tiff, Bmp, Svg,
            // Videos  
            Mp4, Avi, Mkv, Mov, Wmv, Flv, WebM,
            // Audio
            Mp3, Wav, Flac, Aac, Ogg, M4a,
            // Documents
            Pdf,
            // Text
            Text, Markdown, Json, Xml, Html, Css, Javascript, Rust, Python, Cpp, Java,
        ]
    }
    
    /// Clone service for background processing
    fn clone_for_background(&self) -> Self {
        // Note: This is a simplified clone for demo purposes
        // In a real implementation, handlers would need to be cloneable
        Self {
            handlers: Vec::new(), // Simplified for now
            config: self.config.clone(),
            cache_service: self.cache_service.clone(),
        }
    }
}

impl Default for PreviewService {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for configurations
impl Clone for PreviewConfig {
    fn clone(&self) -> Self {
        Self {
            thumbnail_size: self.thumbnail_size,
            max_preview_text_length: self.max_preview_text_length,
            video_thumbnail_count: self.video_thumbnail_count,
            audio_waveform_samples: self.audio_waveform_samples,
            background_processing: self.background_processing,
            cache_thumbnails: self.cache_thumbnails,
            max_file_size: self.max_file_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_format_detection() {
        assert_eq!(SupportedFormat::from_extension("jpg"), Some(SupportedFormat::Jpeg));
        assert_eq!(SupportedFormat::from_extension("png"), Some(SupportedFormat::Png));
        assert_eq!(SupportedFormat::from_extension("mp4"), Some(SupportedFormat::Mp4));
        assert_eq!(SupportedFormat::from_extension("pdf"), Some(SupportedFormat::Pdf));
        assert_eq!(SupportedFormat::from_extension("txt"), Some(SupportedFormat::Text));
        assert_eq!(SupportedFormat::from_extension("unknown"), None);
    }
    
    #[test]
    fn test_format_categories() {
        assert!(SupportedFormat::Jpeg.is_image());
        assert!(SupportedFormat::Mp4.is_video());
        assert!(SupportedFormat::Mp3.is_audio());
        assert!(SupportedFormat::Pdf.is_document());
        assert!(SupportedFormat::Text.is_text());
    }
    
    #[tokio::test]
    async fn test_preview_service_creation() {
        let service = PreviewService::new();
        assert_eq!(service.handlers.len(), 0);
        assert_eq!(service.config.thumbnail_size, (256, 256));
    }
    
    #[tokio::test]
    async fn test_format_detection_service() {
        let service = PreviewService::new();
        
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.jpg");
        fs::write(&image_path, b"fake image data").unwrap();
        
        assert_eq!(service.detect_format(&image_path), Some(SupportedFormat::Jpeg));
        assert!(!service.is_supported(&image_path)); // No handlers registered yet
    }
    
    #[test]
    fn test_metadata_creation() {
        let metadata = FileMetadata::new();
        assert_eq!(metadata.file_size, 0);
        assert!(metadata.created.is_none());
        assert!(metadata.width.is_none());
    }
}