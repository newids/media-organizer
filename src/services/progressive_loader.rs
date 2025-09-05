use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::{mpsc, oneshot};
use thiserror::Error;
use tracing::{debug, info};

use super::file_system::{FileSystemError, PreviewMetadata};
use super::preview_cache::{CachedPreviewData, PreviewDataMetadata};

/// Errors that can occur during progressive loading
#[derive(Error, Debug)]
pub enum ProgressiveLoaderError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    #[error("Operation was cancelled")]
    Cancelled,
    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },
    #[error("Chunk processing failed: {reason}")]
    ChunkProcessingFailed { reason: String },
    #[error("Progress reporting failed: {reason}")]
    ProgressReportingFailed { reason: String },
    #[error("Invalid chunk configuration: {reason}")]
    InvalidChunkConfig { reason: String },
}

/// Configuration for progressive loading
#[derive(Debug, Clone)]
pub struct ProgressiveLoaderConfig {
    /// Size of each chunk to read (default: 8MB)
    pub chunk_size: usize,
    /// Maximum file size to process (default: 100MB)
    pub max_file_size: u64,
    /// Minimum time between progress updates
    pub progress_update_interval: Duration,
    /// Whether to preload metadata before chunked reading
    pub preload_metadata: bool,
    /// Whether to generate intermediate previews during loading
    pub generate_intermediate_previews: bool,
    /// Maximum number of chunks to buffer in memory
    pub max_buffered_chunks: usize,
}

impl Default for ProgressiveLoaderConfig {
    fn default() -> Self {
        Self {
            chunk_size: 8 * 1024 * 1024, // 8MB chunks
            max_file_size: 100 * 1024 * 1024, // 100MB max
            progress_update_interval: Duration::from_millis(100), // 100ms updates
            preload_metadata: true,
            generate_intermediate_previews: true,
            max_buffered_chunks: 4, // 32MB buffer max
        }
    }
}

/// Progress information for progressive loading
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    /// Total file size in bytes
    pub total_bytes: u64,
    /// Bytes processed so far
    pub processed_bytes: u64,
    /// Current chunk index being processed
    pub current_chunk: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Progress percentage (0.0 to 100.0)
    pub percentage: f64,
    /// Estimated time remaining (None if unknown)
    pub estimated_remaining: Option<Duration>,
    /// Current processing stage
    pub stage: LoadingStage,
    /// Optional intermediate preview data
    pub intermediate_preview: Option<CachedPreviewData>,
}

impl LoadingProgress {
    fn new(total_bytes: u64, total_chunks: usize) -> Self {
        Self {
            total_bytes,
            processed_bytes: 0,
            current_chunk: 0,
            total_chunks,
            percentage: 0.0,
            estimated_remaining: None,
            stage: LoadingStage::Initializing,
            intermediate_preview: None,
        }
    }

    fn update_progress(&mut self, processed_bytes: u64, current_chunk: usize, stage: LoadingStage) {
        self.processed_bytes = processed_bytes;
        self.current_chunk = current_chunk;
        self.stage = stage;
        self.percentage = if self.total_bytes > 0 {
            (processed_bytes as f64 / self.total_bytes as f64) * 100.0
        } else {
            0.0
        };
    }
}

/// Stages of the progressive loading process
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoadingStage {
    Initializing,
    ReadingMetadata,
    ProcessingChunk(usize),
    GeneratingPreview,
    Finalizing,
    Complete,
}

/// Handle for controlling and monitoring a progressive loading operation
pub struct ProgressiveLoadHandle {
    /// Channel for receiving progress updates
    pub progress_receiver: mpsc::Receiver<LoadingProgress>,
    /// Channel for receiving the final result
    pub result_receiver: oneshot::Receiver<Result<CachedPreviewData, ProgressiveLoaderError>>,
    /// Cancellation token
    pub cancellation_token: Arc<AtomicBool>,
    /// Loading metadata
    pub file_path: PathBuf,
    pub file_size: u64,
    pub started_at: SystemTime,
}

impl ProgressiveLoadHandle {
    /// Cancel the loading operation
    pub fn cancel(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }

    /// Check if the operation has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.load(Ordering::SeqCst)
    }

    /// Get the duration since loading started
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed().unwrap_or(Duration::ZERO)
    }

    /// Wait for the operation to complete and return the result
    pub async fn await_result(self) -> Result<CachedPreviewData, ProgressiveLoaderError> {
        match self.result_receiver.await {
            Ok(result) => result,
            Err(_) => Err(ProgressiveLoaderError::Cancelled),
        }
    }

    /// Get the next progress update (non-blocking)
    pub async fn next_progress(&mut self) -> Option<LoadingProgress> {
        self.progress_receiver.recv().await
    }
}

/// Core progressive loader for handling large files
pub struct ProgressiveLoader {
    config: ProgressiveLoaderConfig,
}

impl ProgressiveLoader {
    /// Create a new progressive loader with the given configuration
    pub fn new(config: ProgressiveLoaderConfig) -> Self {
        Self { config }
    }

    /// Create a new progressive loader with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ProgressiveLoaderConfig::default())
    }

    /// Start loading a file progressively
    pub async fn load_file(&self, file_path: &Path) -> Result<ProgressiveLoadHandle, ProgressiveLoaderError> {
        debug!("Starting progressive loading for: {}", file_path.display());

        // Validate file exists and get size
        let file_metadata = tokio::fs::metadata(file_path).await?;
        let file_size = file_metadata.len();

        // Check file size limit
        if file_size > self.config.max_file_size {
            return Err(ProgressiveLoaderError::FileTooLarge {
                size: file_size,
                max_size: self.config.max_file_size,
            });
        }

        // Calculate total chunks
        let total_chunks = if file_size == 0 {
            1
        } else {
            ((file_size as usize + self.config.chunk_size - 1) / self.config.chunk_size).max(1)
        };

        debug!("File size: {} bytes, chunk size: {} bytes, total chunks: {}", 
               file_size, self.config.chunk_size, total_chunks);

        // Create communication channels
        let (progress_sender, progress_receiver) = mpsc::channel(32);
        let (result_sender, result_receiver) = oneshot::channel();
        let cancellation_token = Arc::new(AtomicBool::new(false));

        // Create handle
        let handle = ProgressiveLoadHandle {
            progress_receiver,
            result_receiver,
            cancellation_token: cancellation_token.clone(),
            file_path: file_path.to_path_buf(),
            file_size,
            started_at: SystemTime::now(),
        };

        // Start the loading task
        let file_path = file_path.to_path_buf();
        let config = self.config.clone();
        tokio::spawn(async move {
            let result = Self::load_file_chunks(
                &file_path,
                file_size,
                total_chunks,
                &config,
                progress_sender,
                cancellation_token,
            ).await;

            // Send final result (ignore if receiver is dropped)
            let _ = result_sender.send(result);
        });

        Ok(handle)
    }

    /// Internal method to perform the actual chunked loading
    async fn load_file_chunks(
        file_path: &Path,
        file_size: u64,
        total_chunks: usize,
        config: &ProgressiveLoaderConfig,
        progress_sender: mpsc::Sender<LoadingProgress>,
        cancellation_token: Arc<AtomicBool>,
    ) -> Result<CachedPreviewData, ProgressiveLoaderError> {
        
        let mut progress = LoadingProgress::new(file_size, total_chunks);
        
        // Send initial progress
        progress.stage = LoadingStage::Initializing;
        let _ = progress_sender.send(progress.clone()).await;

        // Check for cancellation
        if cancellation_token.load(Ordering::SeqCst) {
            return Err(ProgressiveLoaderError::Cancelled);
        }

        // Open file for reading
        let mut file = File::open(file_path).await?;
        
        // Read metadata if enabled
        let preview_metadata = if config.preload_metadata {
            progress.stage = LoadingStage::ReadingMetadata;
            let _ = progress_sender.send(progress.clone()).await;
            
            Self::extract_file_metadata(file_path).await?
        } else {
            PreviewMetadata::default()
        };

        // Prepare for chunked reading
        let mut processed_bytes = 0u64;
        let mut accumulated_data = Vec::new();
        let mut last_progress_update = SystemTime::now();

        info!("Starting chunked reading of {} bytes in {} chunks", file_size, total_chunks);

        // Process file chunk by chunk
        for chunk_index in 0..total_chunks {
            // Check for cancellation before each chunk
            if cancellation_token.load(Ordering::SeqCst) {
                return Err(ProgressiveLoaderError::Cancelled);
            }

            // Calculate chunk size for this iteration
            let remaining_bytes = file_size - processed_bytes;
            let current_chunk_size = (config.chunk_size as u64).min(remaining_bytes) as usize;
            
            if current_chunk_size == 0 {
                break;
            }

            // Read chunk
            let mut chunk_buffer = vec![0u8; current_chunk_size];
            file.read_exact(&mut chunk_buffer).await?;
            
            processed_bytes += current_chunk_size as u64;

            // Update progress
            progress.update_progress(processed_bytes, chunk_index + 1, LoadingStage::ProcessingChunk(chunk_index));

            // Generate intermediate preview if enabled and at strategic points
            if config.generate_intermediate_previews && Self::should_generate_intermediate_preview(chunk_index, total_chunks) {
                progress.intermediate_preview = Self::generate_intermediate_preview(
                    &accumulated_data,
                    &chunk_buffer,
                    &preview_metadata,
                    file_path,
                    processed_bytes,
                    file_size,
                ).await.ok();
            }

            // Add chunk to accumulated data (with memory management)
            if accumulated_data.len() < config.max_buffered_chunks * config.chunk_size {
                accumulated_data.extend_from_slice(&chunk_buffer);
            }

            // Send progress update if enough time has passed
            if last_progress_update.elapsed().unwrap_or(Duration::ZERO) >= config.progress_update_interval {
                let _ = progress_sender.send(progress.clone()).await;
                last_progress_update = SystemTime::now();
            }

            debug!("Processed chunk {}/{} ({:.1}% complete)", 
                   chunk_index + 1, total_chunks, progress.percentage);
        }

        // Generate final preview
        progress.stage = LoadingStage::GeneratingPreview;
        let _ = progress_sender.send(progress.clone()).await;

        let final_preview = Self::generate_final_preview(
            &accumulated_data,
            &preview_metadata,
            file_path,
            file_size,
        ).await?;

        // Finalization
        progress.stage = LoadingStage::Finalizing;
        let _ = progress_sender.send(progress.clone()).await;

        // Complete
        progress.stage = LoadingStage::Complete;
        progress.percentage = 100.0;
        let _ = progress_sender.send(progress).await;

        info!("Progressive loading complete for: {}", file_path.display());
        Ok(final_preview)
    }

    /// Extract basic file metadata for preview generation
    async fn extract_file_metadata(file_path: &Path) -> Result<PreviewMetadata, ProgressiveLoaderError> {
        // This is a simplified metadata extraction
        // In production, you'd use proper media libraries for each file type
        
        let extension = file_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        let mime_type = match extension.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "mp4" => Some("video/mp4".to_string()),
            "avi" => Some("video/x-msvideo".to_string()),
            "mkv" => Some("video/x-matroska".to_string()),
            "mp3" => Some("audio/mpeg".to_string()),
            "wav" => Some("audio/wav".to_string()),
            "flac" => Some("audio/flac".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            _ => None,
        };

        Ok(PreviewMetadata {
            mime_type,
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
        })
    }

    /// Determine if an intermediate preview should be generated at this chunk
    fn should_generate_intermediate_preview(chunk_index: usize, total_chunks: usize) -> bool {
        // Generate previews at strategic points: 25%, 50%, 75% completion
        let progress_percent = ((chunk_index + 1) as f64 / total_chunks as f64) * 100.0;
        progress_percent >= 25.0 && (chunk_index + 1) % (total_chunks / 4).max(1) == 0
    }

    /// Generate an intermediate preview from partially loaded data
    async fn generate_intermediate_preview(
        _accumulated_data: &[u8],
        _current_chunk: &[u8],
        metadata: &PreviewMetadata,
        file_path: &Path,
        processed_bytes: u64,
        total_bytes: u64,
    ) -> Result<CachedPreviewData, ProgressiveLoaderError> {
        // This is a placeholder implementation
        // In production, you'd generate actual intermediate previews:
        // - For images: partial image with current data
        // - For videos: thumbnail from available frames
        // - For audio: partial waveform

        let progress_percent = (processed_bytes as f64 / total_bytes as f64) * 100.0;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let intermediate_data = format!(
            "INTERMEDIATE_PREVIEW:{}:{}%:{}", 
            file_name, 
            progress_percent as u32,
            processed_bytes
        ).into_bytes();

        let content_type = metadata.mime_type.clone().unwrap_or_else(|| "application/octet-stream".to_string());
        let format = "intermediate_preview".to_string();

        let preview_metadata = PreviewDataMetadata {
            width: metadata.width,
            height: metadata.height,
            duration: metadata.duration,
            quality_level: 60, // Lower quality for intermediate
        };

        Ok(CachedPreviewData::new(
            intermediate_data,
            content_type,
            format,
            total_bytes,
            preview_metadata,
        ))
    }

    /// Generate the final preview from all loaded data
    async fn generate_final_preview(
        data: &[u8],
        metadata: &PreviewMetadata,
        file_path: &Path,
        file_size: u64,
    ) -> Result<CachedPreviewData, ProgressiveLoaderError> {
        // This is a placeholder implementation
        // In production, you'd generate actual previews:
        // - For images: resize and create thumbnail
        // - For videos: extract representative frames
        // - For audio: generate complete waveform
        // - For documents: render preview pages

        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let preview_data = match metadata.mime_type.as_deref() {
            Some(mime) if mime.starts_with("image/") => {
                format!("PROGRESSIVE_IMAGE_THUMBNAIL:{}:{}", file_name, data.len()).into_bytes()
            }
            Some(mime) if mime.starts_with("video/") => {
                format!("PROGRESSIVE_VIDEO_THUMBNAIL:{}:{}", file_name, data.len()).into_bytes()
            }
            Some(mime) if mime.starts_with("audio/") => {
                format!("PROGRESSIVE_AUDIO_WAVEFORM:{}:{}", file_name, data.len()).into_bytes()
            }
            _ => {
                format!("PROGRESSIVE_PREVIEW:{}:{}", file_name, data.len()).into_bytes()
            }
        };

        let content_type = metadata.mime_type.clone().unwrap_or_else(|| "application/octet-stream".to_string());
        let format = match content_type.split('/').next().unwrap_or("unknown") {
            "image" => "progressive_thumbnail",
            "video" => "progressive_video_thumbnail", 
            "audio" => "progressive_waveform",
            _ => "progressive_preview",
        }.to_string();

        let preview_metadata = PreviewDataMetadata {
            width: metadata.width,
            height: metadata.height,
            duration: metadata.duration,
            quality_level: 85, // High quality for final preview
        };

        Ok(CachedPreviewData::new(
            preview_data,
            content_type,
            format,
            file_size,
            preview_metadata,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    async fn create_test_file(temp_dir: &TempDir, name: &str, size: usize) -> PathBuf {
        let file_path = temp_dir.path().join(name);
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        
        // Write test data
        let test_data = vec![0xA5u8; size]; // Pattern for testing
        file.write_all(&test_data).await.unwrap();
        file.flush().await.unwrap();
        
        file_path
    }

    #[tokio::test]
    async fn test_progressive_loader_small_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(&temp_dir, "small.jpg", 1024).await;
        
        let loader = ProgressiveLoader::with_defaults();
        let mut handle = loader.load_file(&file_path).await.unwrap();
        
        // Check that we can get progress updates
        let mut progress_count = 0;
        while let Some(progress) = handle.next_progress().await {
            progress_count += 1;
            assert!(progress.percentage >= 0.0 && progress.percentage <= 100.0);
            if progress.stage == LoadingStage::Complete {
                break;
            }
        }
        
        assert!(progress_count > 0);
        
        // Get final result
        let result = handle.await_result().await.unwrap();
        assert!(!result.data.is_empty());
        assert_eq!(result.original_size, 1024);
    }

    #[tokio::test]
    async fn test_progressive_loader_large_file() {
        let temp_dir = TempDir::new().unwrap();
        let large_size = 20 * 1024 * 1024; // 20MB
        let file_path = create_test_file(&temp_dir, "large.mp4", large_size).await;
        
        let mut config = ProgressiveLoaderConfig::default();
        config.chunk_size = 2 * 1024 * 1024; // 2MB chunks
        config.generate_intermediate_previews = true;
        
        let loader = ProgressiveLoader::new(config);
        let mut handle = loader.load_file(&file_path).await.unwrap();
        
        let mut intermediate_previews = 0;
        let mut final_progress = None;
        
        while let Some(progress) = handle.next_progress().await {
            if progress.intermediate_preview.is_some() {
                intermediate_previews += 1;
            }
            final_progress = Some(progress.clone());
            if progress.stage == LoadingStage::Complete {
                break;
            }
        }
        
        // Should have received intermediate previews
        assert!(intermediate_previews > 0);
        
        // Final progress should be 100%
        let final_progress = final_progress.unwrap();
        assert_eq!(final_progress.percentage, 100.0);
        assert_eq!(final_progress.stage, LoadingStage::Complete);
        
        // Get final result
        let result = handle.await_result().await.unwrap();
        assert!(!result.data.is_empty());
        assert_eq!(result.original_size, large_size as u64);
    }

    #[tokio::test]
    async fn test_progressive_loader_cancellation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(&temp_dir, "cancel_test.mp4", 10 * 1024 * 1024).await;
        
        let mut config = ProgressiveLoaderConfig::default();
        config.chunk_size = 1024 * 1024; // 1MB chunks for more granular cancellation
        
        let loader = ProgressiveLoader::new(config);
        let handle = loader.load_file(&file_path).await.unwrap();
        
        // Cancel immediately
        handle.cancel();
        
        // Should get cancellation error
        let result = handle.await_result().await;
        assert!(matches!(result, Err(ProgressiveLoaderError::Cancelled)));
    }

    #[tokio::test]
    async fn test_progressive_loader_file_too_large() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(&temp_dir, "huge.mp4", 1024).await;
        
        let mut config = ProgressiveLoaderConfig::default();
        config.max_file_size = 512; // Very small limit
        
        let loader = ProgressiveLoader::new(config);
        let result = loader.load_file(&file_path).await;
        
        assert!(matches!(result, Err(ProgressiveLoaderError::FileTooLarge { .. })));
    }

    #[tokio::test]
    async fn test_progressive_loader_stages() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file(&temp_dir, "stages_test.jpg", 5 * 1024 * 1024).await;
        
        let loader = ProgressiveLoader::with_defaults();
        let mut handle = loader.load_file(&file_path).await.unwrap();
        
        let mut stages_seen = std::collections::HashSet::new();
        
        while let Some(progress) = handle.next_progress().await {
            stages_seen.insert(progress.stage.clone());
            if progress.stage == LoadingStage::Complete {
                break;
            }
        }
        
        // Should have seen key stages
        assert!(stages_seen.contains(&LoadingStage::Initializing));
        assert!(stages_seen.contains(&LoadingStage::Complete));
        
        // Get final result
        let result = handle.await_result().await.unwrap();
        assert!(!result.data.is_empty());
    }
}