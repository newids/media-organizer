use std::path::{Path, PathBuf};
use std::time::SystemTime;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::task::JoinHandle;
use chrono::{DateTime, Utc};
use walkdir::WalkDir;

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
    #[error("Archive processing error: {0}")]
    ArchiveError(String),
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
    // Archive formats
    Zip,
    Tar,
    Gz,
    SevenZip,
    Rar,
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
            // Archives
            "zip" => Some(Self::Zip),
            "tar" => Some(Self::Tar),
            "gz" => Some(Self::Gz),
            "7z" => Some(Self::SevenZip),
            "rar" => Some(Self::Rar),
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

    /// Check if format is an archive type
    pub fn is_archive(&self) -> bool {
        matches!(self, Self::Zip | Self::Tar | Self::Gz | Self::SevenZip | Self::Rar)
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
        /// Text content preview
        content: String,
        /// Detected language for syntax highlighting
        language: Option<String>,
        /// Total number of lines in the file
        line_count: usize,
    },
    Archive {
        /// Archive contents listing
        contents: Vec<String>,
        /// Archive thumbnail/icon
        thumbnail: Vec<u8>,
    },
    /// Fallback content for unsupported file types
    Unsupported {
        /// File type/extension
        file_type: String,
        /// Reason why the file type is unsupported
        reason: String,
        /// Suggested action for the user
        suggested_action: Option<String>,
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
    pub max_concurrent_previews: Option<usize>,
    pub default_timeout: Option<std::time::Duration>,
    pub cache_ttl: Option<std::time::Duration>,
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
            max_concurrent_previews: Some(8),
            default_timeout: Some(std::time::Duration::from_secs(30)),
            cache_ttl: Some(std::time::Duration::from_secs(3600)), // 1 hour
        }
    }
}

/// Core trait for implementing preview providers with plugin architecture
/// Each provider specializes in handling specific file formats
#[async_trait]
pub trait PreviewProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "image", "video", "pdf")
    fn provider_id(&self) -> &'static str;
    
    /// Human-readable name for this provider
    fn provider_name(&self) -> &'static str;
    
    /// Check if this provider can handle the given format
    fn supports_format(&self, format: SupportedFormat) -> bool;
    
    /// Get supported file extensions for this provider
    fn supported_extensions(&self) -> Vec<&'static str>;
    
    /// Generate a complete preview for the given file
    /// Returns preview data including thumbnail, metadata, and content
    async fn generate_preview(
        &self,
        file_path: &Path,
        config: &PreviewConfig,
    ) -> Result<PreviewData, PreviewError>;
    
    /// Extract metadata from file without generating visual preview
    /// Optimized for quick file information display
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError>;
    
    /// Generate thumbnail only (optimized for performance)
    /// Returns raw image bytes in PNG format with caching support
    async fn generate_thumbnail(
        &self,
        file_path: &Path,
        size: (u32, u32),
    ) -> Result<Vec<u8>, PreviewError>;
    
    /// Check if this provider supports background processing
    /// True for resource-intensive operations (video, large images)
    fn supports_background_processing(&self) -> bool {
        false
    }
    
    /// Get the priority level for this provider (higher = preferred)
    /// Used when multiple providers support the same format
    fn priority(&self) -> u32 {
        100
    }
}

/// Legacy trait for backward compatibility - will be deprecated
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

/// Priority levels for preview generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreviewPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl Default for PreviewPriority {
    fn default() -> Self {
        PreviewPriority::Normal
    }
}

/// Background task handle for async thumbnail generation with enhanced features
#[derive(Debug)]
pub struct ThumbnailTask {
    pub file_path: PathBuf,
    pub handle: JoinHandle<Result<PreviewData, PreviewError>>,
    pub priority: PreviewPriority,
    pub created_at: std::time::Instant,
    pub timeout: Option<std::time::Duration>,
    pub abort_handle: tokio_util::sync::CancellationToken,
}

impl ThumbnailTask {
    pub fn new(
        file_path: PathBuf,
        handle: JoinHandle<Result<PreviewData, PreviewError>>,
        priority: PreviewPriority,
        timeout: Option<std::time::Duration>,
        abort_handle: tokio_util::sync::CancellationToken,
    ) -> Self {
        Self {
            file_path,
            handle,
            priority,
            created_at: std::time::Instant::now(),
            timeout,
            abort_handle,
        }
    }
    
    /// Check if the task is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_finished()
    }
    
    /// Check if the task has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout) = self.timeout {
            self.created_at.elapsed() > timeout
        } else {
            false
        }
    }
    
    /// Cancel the task
    pub fn cancel(&self) {
        self.abort_handle.cancel();
        self.handle.abort();
    }
    
    /// Get the result with timeout support
    pub async fn await_result_with_timeout(self) -> Result<PreviewData, PreviewError> {
        match self.timeout {
            Some(timeout) => {
                match tokio::time::timeout(timeout, self.handle).await {
                    Ok(Ok(result)) => result,
                    Ok(Err(join_error)) => Err(PreviewError::TaskError(join_error.to_string())),
                    Err(_timeout_error) => {
                        Err(PreviewError::TaskError("Preview generation timed out".to_string()))
                    }
                }
            }
            None => {
                match self.handle.await {
                    Ok(result) => result,
                    Err(join_error) => Err(PreviewError::TaskError(join_error.to_string())),
                }
            }
        }
    }
    
    /// Get the result if the task is complete (legacy method)
    pub async fn await_result(self) -> Result<PreviewData, PreviewError> {
        self.await_result_with_timeout().await
    }
    
    /// Get task age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

/// Task queue statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct TaskQueueStats {
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub cancelled_tasks: usize,
    pub total_processing_time: std::time::Duration,
    pub average_processing_time: std::time::Duration,
}

/// Task queue for managing background preview generation
#[derive(Debug)]
pub struct PreviewTaskQueue {
    active_tasks: std::collections::HashMap<String, ThumbnailTask>,
    max_concurrent: usize,
    stats: TaskQueueStats,
}

impl PreviewTaskQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            active_tasks: std::collections::HashMap::new(),
            max_concurrent,
            stats: TaskQueueStats::default(),
        }
    }
    
    /// Add a task to the queue
    pub fn add_task(&mut self, task_id: String, task: ThumbnailTask) -> Result<(), PreviewError> {
        if self.active_tasks.len() >= self.max_concurrent {
            return Err(PreviewError::TaskError("Task queue is full".to_string()));
        }
        
        self.active_tasks.insert(task_id, task);
        self.stats.active_tasks = self.active_tasks.len();
        Ok(())
    }
    
    /// Remove completed or failed tasks and update stats
    pub fn cleanup_finished_tasks(&mut self) {
        let mut completed = 0;
        let mut failed = 0;
        let mut cancelled = 0;
        
        self.active_tasks.retain(|_task_id, task| {
            if task.is_finished() || task.is_timed_out() {
                let processing_time = task.age();
                self.stats.total_processing_time += processing_time;
                
                if task.is_timed_out() {
                    cancelled += 1;
                } else {
                    // We can't easily determine success/failure without awaiting
                    // so we'll count as completed for now
                    completed += 1;
                }
                false // Remove from active tasks
            } else {
                true // Keep in active tasks
            }
        });
        
        self.stats.completed_tasks += completed;
        self.stats.failed_tasks += failed;
        self.stats.cancelled_tasks += cancelled;
        self.stats.active_tasks = self.active_tasks.len();
        
        // Update average processing time
        let total_finished = self.stats.completed_tasks + self.stats.failed_tasks + self.stats.cancelled_tasks;
        if total_finished > 0 {
            self.stats.average_processing_time = self.stats.total_processing_time / total_finished as u32;
        }
    }
    
    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<&ThumbnailTask> {
        self.active_tasks.get(task_id)
    }
    
    /// Cancel task by ID
    pub fn cancel_task(&self, task_id: &str) -> Result<(), PreviewError> {
        if let Some(task) = self.active_tasks.get(task_id) {
            task.cancel();
            Ok(())
        } else {
            Err(PreviewError::TaskError("Task not found".to_string()))
        }
    }
    
    /// Cancel all tasks
    pub fn cancel_all_tasks(&self) {
        for task in self.active_tasks.values() {
            task.cancel();
        }
    }
    
    /// Get queue statistics
    pub fn stats(&self) -> &TaskQueueStats {
        &self.stats
    }
    
    /// Check if queue has capacity for more tasks
    pub fn has_capacity(&self) -> bool {
        self.active_tasks.len() < self.max_concurrent
    }
}

/// Main preview service coordinating all preview providers
pub struct PreviewService {
    providers: Vec<Box<dyn PreviewProvider>>,
    legacy_handlers: Vec<Box<dyn PreviewHandler>>, // Backward compatibility
    config: PreviewConfig,
    cache_service: Option<crate::services::cache::CacheService>,
    task_queue: std::sync::Arc<std::sync::Mutex<PreviewTaskQueue>>,
}

impl PreviewService {
    /// Create new preview service with default configuration
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            legacy_handlers: Vec::new(),
            config: PreviewConfig::default(),
            cache_service: None,
            task_queue: std::sync::Arc::new(std::sync::Mutex::new(PreviewTaskQueue::new(8))), // Max 8 concurrent tasks
        }
    }
    
    /// Create preview service with custom configuration
    pub fn with_config(config: PreviewConfig) -> Self {
        let max_concurrent = config.max_concurrent_previews.unwrap_or(8);
        Self {
            providers: Vec::new(),
            legacy_handlers: Vec::new(),
            config,
            cache_service: None,
            task_queue: std::sync::Arc::new(std::sync::Mutex::new(PreviewTaskQueue::new(max_concurrent))),
        }
    }
    
    /// Add cache service for thumbnail caching
    pub fn with_cache(mut self, cache_service: crate::services::cache::CacheService) -> Self {
        self.cache_service = Some(cache_service);
        self
    }
    
    /// Register a preview provider (new plugin interface)
    pub fn register_provider(&mut self, provider: Box<dyn PreviewProvider>) {
        self.providers.push(provider);
    }
    
    /// Register all default preview providers
    pub fn with_default_providers(mut self) -> Self {
        // Register image provider
        let provider = crate::services::preview::ImagePreviewProvider::new();
        self.register_provider(Box::new(provider));
        
        // Register text provider
        if let Ok(provider) = crate::services::preview::TextPreviewProvider::new() {
            self.register_provider(Box::new(provider));
        } else {
            tracing::warn!("Failed to register TextPreviewProvider");
        }
        
        // Register video provider (feature gated)
        if let Ok(provider) = crate::services::preview::VideoPreviewProvider::new() {
            self.register_provider(Box::new(provider));
        } else {
            tracing::debug!("VideoPreviewProvider not available (feature not enabled or FFmpeg missing)");
        }
        
        // Register audio provider (feature gated)
        if let Ok(provider) = crate::services::preview::AudioPreviewProvider::new() {
            self.register_provider(Box::new(provider));
        } else {
            tracing::debug!("AudioPreviewProvider not available (feature not enabled)");
        }
        
        // Register PDF provider (feature gated)
        if let Ok(provider) = crate::services::preview::PdfPreviewProvider::new() {
            self.register_provider(Box::new(provider));
        } else {
            tracing::debug!("PdfPreviewProvider not available (feature not enabled)");
        }
        
        // Register archive provider
        if let Ok(provider) = crate::services::preview::ArchivePreviewProvider::new() {
            self.register_provider(Box::new(provider));
        } else {
            tracing::warn!("Failed to register ArchivePreviewProvider");
        }
        
        // Register fallback provider (lowest priority - always last)
        let fallback_provider = crate::services::preview::FallbackPreviewProvider::new();
        self.register_provider(Box::new(fallback_provider));
        
        self
    }
    
    /// Register only the fallback provider (lowest priority)
    pub fn with_fallback_provider(mut self) -> Self {
        let provider = crate::services::preview::FallbackPreviewProvider::new();
        self.register_provider(Box::new(provider));
        self
    }
    
    /// Register a legacy preview handler (backward compatibility)
    pub fn register_handler(&mut self, handler: Box<dyn PreviewHandler>) {
        self.legacy_handlers.push(handler);
    }
    
    /// Get all registered providers
    pub fn providers(&self) -> &[Box<dyn PreviewProvider>] {
        &self.providers
    }
    
    /// Find the best provider for a given format (highest priority)
    pub fn find_provider_for_format(&self, format: SupportedFormat) -> Option<&Box<dyn PreviewProvider>> {
        self.providers
            .iter()
            .filter(|provider| provider.supports_format(format))
            .max_by_key(|provider| provider.priority())
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
            // Check new providers first
            self.providers.iter().any(|provider| provider.supports_format(format)) ||
            // Fallback to legacy handlers
            self.legacy_handlers.iter().any(|handler| handler.supports_format(format))
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
        
        // Try to detect format - if unknown, we'll use the fallback provider
        let format_option = self.detect_format(path);
        
        // Check file size
        let metadata = std::fs::metadata(path)?;
        if metadata.len() > self.config.max_file_size {
            return Err(PreviewError::ReadError(format!("File too large: {} bytes", metadata.len())));
        }
        
        // Check cache first if available
        if let Some(cache) = &self.cache_service {
            if let Ok(Some(cached_thumbnail)) = cache.get_thumbnail_path(path).await {
                // Check if cached preview is still valid based on TTL
                if let Some(cache_ttl) = self.config.cache_ttl {
                    let age = Utc::now() - cached_thumbnail.created_at;
                    
                    if age.to_std().unwrap_or(std::time::Duration::MAX) <= cache_ttl {
                        // Cache is still valid, check if thumbnail file exists
                        if cached_thumbnail.thumbnail_path.exists() {
                            // Return cached preview data
                            tracing::debug!("Using cached thumbnail for {:?}: {:?}", path, cached_thumbnail.thumbnail_path);
                            
                            // Extract metadata for the cached preview
                            let file_metadata = self.extract_file_metadata(path).await?;
                            
                            return Ok(PreviewData {
                                file_path: path.to_path_buf(),
                                format: SupportedFormat::Jpeg, // TODO: Store actual format in cache
                                preview_content: PreviewContent::Image {
                                    thumbnail_data: Vec::new(), // TODO: Load thumbnail data
                                    original_format: "unknown".to_string(), // TODO: Store format in cache
                                },
                                thumbnail_path: Some(cached_thumbnail.thumbnail_path),
                                metadata: file_metadata,
                                generated_at: SystemTime::now(),
                            });
                        } else {
                            tracing::warn!("Cached thumbnail file missing, regenerating: {:?}", cached_thumbnail.thumbnail_path);
                        }
                    } else {
                        tracing::debug!("Cached thumbnail expired, regenerating: {:?}", path);
                    }
                } else {
                    // No TTL configured, assume cache is valid if file exists
                    if cached_thumbnail.thumbnail_path.exists() {
                        tracing::debug!("Using cached thumbnail (no TTL): {:?}", cached_thumbnail.thumbnail_path);
                        
                        let file_metadata = self.extract_file_metadata(path).await?;
                        
                        return Ok(PreviewData {
                            file_path: path.to_path_buf(),
                            format: SupportedFormat::Jpeg, // TODO: Store actual format in cache
                            preview_content: PreviewContent::Image {
                                thumbnail_data: Vec::new(), // TODO: Load thumbnail data
                                original_format: "unknown".to_string(), // TODO: Store format in cache
                            },
                            thumbnail_path: Some(cached_thumbnail.thumbnail_path),
                            metadata: file_metadata,
                            generated_at: SystemTime::now(),
                        });
                    }
                }
            }
        }
        
        // Generate preview using provider system with fallback support
        let preview_data = if let Some(format) = format_option {
            // Format detected - try format-specific providers first
            if let Some(provider) = self.find_provider_for_format(format) {
                provider.generate_preview(path, &self.config).await?
            } else if let Some(handler) = self.legacy_handlers.iter()
                .find(|handler| handler.supports_format(format)) {
                // Fallback to legacy handlers
                handler.generate_preview(path, &self.config).await?
            } else {
                // No provider for this format - use fallback provider
                tracing::debug!("No provider found for format {:?}, using fallback", format);
                self.generate_fallback_preview(path).await?
            }
        } else {
            // Format not detected - use fallback provider directly
            tracing::debug!("Unknown file format for {:?}, using fallback", path);
            self.generate_fallback_preview(path).await?
        };
        
        // Cache thumbnail if caching is enabled
        if self.config.cache_thumbnails {
            if let (Some(cache), Some(thumbnail_path)) = (&self.cache_service, &preview_data.thumbnail_path) {
                // Create cached thumbnail entry
                let cached_thumbnail = crate::services::cache::CachedThumbnail::new(
                    path.to_path_buf(),
                    thumbnail_path.clone()
                );
                match cache.store_thumbnail_path(&cached_thumbnail).await {
                    Ok(_) => {
                        tracing::debug!("Cached thumbnail for {:?}: {:?}", path, thumbnail_path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to cache thumbnail for {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(preview_data)
    }
    
    /// Generate preview in background (non-blocking) with default priority
    pub fn generate_preview_background<P: AsRef<Path>>(&self, file_path: P) -> ThumbnailTask {
        self.generate_preview_background_with_priority(file_path, PreviewPriority::Normal)
    }
    
    /// Generate preview in background with specified priority and timeout support
    pub fn generate_preview_background_with_priority<P: AsRef<Path>>(
        &self,
        file_path: P,
        priority: PreviewPriority,
    ) -> ThumbnailTask {
        let path = file_path.as_ref().to_path_buf();
        let path_clone = path.clone();
        let service = self.clone_for_background();
        let timeout = self.config.default_timeout;
        let abort_handle = tokio_util::sync::CancellationToken::new();
        let abort_clone = abort_handle.clone();
        
        let handle = tokio::spawn(async move {
            tokio::select! {
                result = service.generate_preview(&path_clone) => result,
                _ = abort_clone.cancelled() => {
                    Err(PreviewError::TaskError("Task was cancelled".to_string()))
                }
            }
        });
        
        ThumbnailTask::new(path, handle, priority, timeout, abort_handle)
    }
    
    /// Generate preview in background and add to managed task queue
    pub fn generate_preview_queued<P: AsRef<Path>>(
        &self,
        file_path: P,
        priority: PreviewPriority,
    ) -> Result<String, PreviewError> {
        let path = file_path.as_ref();
        let task_id = format!("{}_{}", 
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown"),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        
        let task = self.generate_preview_background_with_priority(path, priority);
        
        // Add to managed queue
        {
            let mut queue = self.task_queue.lock().unwrap();
            queue.cleanup_finished_tasks(); // Clean up before adding
            queue.add_task(task_id.clone(), task)?;
        }
        
        Ok(task_id)
    }
    
    /// Task Management Methods
    
    /// Get task queue statistics
    pub fn get_queue_stats(&self) -> TaskQueueStats {
        let queue = self.task_queue.lock().unwrap();
        queue.stats().clone()
    }
    
    /// Cancel a specific task by ID
    pub fn cancel_task(&self, task_id: &str) -> Result<(), PreviewError> {
        let queue = self.task_queue.lock().unwrap();
        queue.cancel_task(task_id)
    }
    
    /// Cancel all active tasks
    pub fn cancel_all_tasks(&self) {
        let queue = self.task_queue.lock().unwrap();
        queue.cancel_all_tasks();
    }
    
    /// Get task result by ID (non-blocking check)
    pub fn get_task_result(&self, task_id: &str) -> Option<bool> {
        let queue = self.task_queue.lock().unwrap();
        if let Some(task) = queue.get_task(task_id) {
            Some(task.is_finished())
        } else {
            None
        }
    }
    
    /// Check if task queue has capacity for more tasks
    pub fn has_queue_capacity(&self) -> bool {
        let queue = self.task_queue.lock().unwrap();
        queue.has_capacity()
    }
    
    /// Clean up finished tasks and update statistics
    pub fn cleanup_finished_tasks(&self) {
        let mut queue = self.task_queue.lock().unwrap();
        queue.cleanup_finished_tasks();
    }
    
    /// Generate fallback preview using the fallback provider
    async fn generate_fallback_preview(&self, path: &Path) -> Result<PreviewData, PreviewError> {
        // Find the fallback provider (lowest priority)
        if let Some(fallback_provider) = self.providers.iter()
            .find(|provider| provider.provider_id() == "fallback") {
            fallback_provider.generate_preview(path, &self.config).await
        } else {
            // If no fallback provider is registered, create a basic error response
            Err(PreviewError::UnsupportedFormat(
                format!("No fallback provider available for file: {:?}", path)
            ))
        }
    }
    
    /// Helper method to extract basic file metadata
    async fn extract_file_metadata(&self, path: &Path) -> Result<FileMetadata, PreviewError> {
        let metadata = std::fs::metadata(path)?;
        
        Ok(FileMetadata {
            file_size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            width: None, // Will be filled by format-specific providers
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
        })
    }
    
    /// Advanced Caching Methods
    
    /// Batch cache multiple previews efficiently
    pub async fn batch_generate_and_cache<P: AsRef<Path>>(
        &self,
        file_paths: &[P],
        priority: PreviewPriority,
    ) -> Result<Vec<String>, PreviewError> {
        let mut task_ids = Vec::new();
        
        for file_path in file_paths {
            if !self.has_queue_capacity() {
                self.cleanup_finished_tasks();
                if !self.has_queue_capacity() {
                    // If still no capacity, wait for some tasks to complete
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
            
            match self.generate_preview_queued(file_path, priority) {
                Ok(task_id) => task_ids.push(task_id),
                Err(e) => {
                    tracing::warn!("Failed to queue preview generation for {:?}: {}", file_path.as_ref(), e);
                }
            }
        }
        
        Ok(task_ids)
    }
    
    /// Clear expired cache entries based on TTL
    pub async fn cleanup_expired_cache(&self) -> Result<usize, PreviewError> {
        if let Some(cache) = &self.cache_service {
            if let Some(cache_ttl) = self.config.cache_ttl {
                // This would require additional methods in the cache service
                // For now, we'll return 0 as a placeholder
                tracing::debug!("Cache cleanup would remove entries older than {:?}", cache_ttl);
                return Ok(0);
            }
        }
        Ok(0)
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<(usize, usize, f64), PreviewError> {
        if let Some(cache) = &self.cache_service {
            // This would require additional methods in the cache service
            // For now, return placeholder values
            tracing::debug!("Cache statistics would be calculated here");
            return Ok((0, 0, 0.0)); // (total_entries, hit_count, hit_ratio)
        }
        Ok((0, 0, 0.0))
    }
    
    /// Warm cache by pre-generating previews for a directory
    pub async fn warm_cache_for_directory<P: AsRef<Path>>(
        &self,
        directory: P,
        recursive: bool,
        priority: PreviewPriority,
    ) -> Result<usize, PreviewError> {
        let dir_path = directory.as_ref();
        
        if !dir_path.is_dir() {
            return Err(PreviewError::FileNotFound(dir_path.to_path_buf()));
        }
        
        let mut file_paths = Vec::new();
        
        if recursive {
            // Use walkdir for recursive traversal
            for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if self.detect_format(entry.path()).is_some() {
                        file_paths.push(entry.path().to_path_buf());
                    }
                }
            }
        } else {
            // Only process direct children
            if let Ok(entries) = std::fs::read_dir(dir_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && self.detect_format(&path).is_some() {
                        file_paths.push(path);
                    }
                }
            }
        }
        
        // Batch generate previews
        let paths_refs: Vec<&PathBuf> = file_paths.iter().collect();
        let _task_ids = self.batch_generate_and_cache(&paths_refs, priority).await?;
        
        tracing::info!("Started cache warming for {} files in {:?}", file_paths.len(), dir_path);
        Ok(file_paths.len())
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
        
        // Find appropriate provider or handler
        if let Some(provider) = self.find_provider_for_format(format) {
            provider.extract_metadata(path).await
        } else if let Some(handler) = self.legacy_handlers.iter()
            .find(|handler| handler.supports_format(format)) {
            handler.extract_metadata(path).await
        } else {
            Err(PreviewError::UnsupportedFormat(format!("{:?}", format)))
        }
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
        
        // Find appropriate provider or handler
        if let Some(provider) = self.find_provider_for_format(format) {
            provider.generate_thumbnail(path, thumbnail_size).await
        } else if let Some(handler) = self.legacy_handlers.iter()
            .find(|handler| handler.supports_format(format)) {
            handler.generate_thumbnail(path, thumbnail_size).await
        } else {
            Err(PreviewError::UnsupportedFormat(format!("{:?}", format)))
        }
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
            // Archives
            Zip, Tar, Gz, SevenZip, Rar,
            // Text
            Text, Markdown, Json, Xml, Html, Css, Javascript, Rust, Python, Cpp, Java,
        ]
    }
    
    /// Clone service for background processing with shared task queue
    fn clone_for_background(&self) -> Self {
        // Create a lightweight clone for background tasks
        // Note: In production, providers would need proper cloning support
        Self {
            providers: Vec::new(), // TODO: Clone providers when they support it
            legacy_handlers: Vec::new(), // TODO: Clone handlers when they support it
            config: self.config.clone(),
            cache_service: self.cache_service.clone(),
            task_queue: self.task_queue.clone(), // Shared task queue
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
            max_concurrent_previews: self.max_concurrent_previews,
            default_timeout: self.default_timeout,
            cache_ttl: self.cache_ttl,
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