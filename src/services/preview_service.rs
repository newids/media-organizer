use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tracing::{debug, warn, info};

use super::file_system::{FileSystemService, FileSystemError, FileEntry, PreviewMetadata};
use super::preview_cache::{
    ThreadSafePreviewCache, PreviewCacheConfig, PreviewCacheKey, 
    CachedPreviewData, PreviewDataMetadata, PreviewCacheError, PreviewCacheStats
};
use super::progressive_loader::{
    ProgressiveLoader, ProgressiveLoaderConfig, ProgressiveLoaderError, 
    ProgressiveLoadHandle
};

/// Errors that can occur in the preview service
#[derive(Error, Debug)]
pub enum PreviewServiceError {
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    #[error("Cache error: {0}")]
    Cache(#[from] PreviewCacheError),
    #[error("Preview generation failed: {reason}")]
    GenerationFailed { reason: String },
    #[error("Unsupported file type: {path}")]
    UnsupportedFileType { path: String },
    #[error("Service not available: {reason}")]
    ServiceUnavailable { reason: String },
    #[error("Progressive loading error: {0}")]
    ProgressiveLoader(#[from] ProgressiveLoaderError),
}

/// Configuration for the integrated preview service
#[derive(Debug, Clone)]
pub struct PreviewServiceConfig {
    /// LRU cache configuration
    pub cache_config: PreviewCacheConfig,
    /// Whether to auto-generate previews for new files
    pub auto_generate_previews: bool,
    /// Maximum file size to process for previews
    pub max_file_size: u64,
    /// Background cleanup interval
    pub cleanup_interval: Duration,
    /// Whether to prefer cache over fresh extraction (for performance)
    pub prefer_cache: bool,
    /// Progressive loader configuration for large files
    pub progressive_config: ProgressiveLoaderConfig,
    /// Threshold for using progressive loading (in bytes)
    pub progressive_threshold: u64,
}

impl Default for PreviewServiceConfig {
    fn default() -> Self {
        Self {
            cache_config: PreviewCacheConfig::default(),
            auto_generate_previews: true,
            max_file_size: 100 * 1024 * 1024, // 100MB as per requirements
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            prefer_cache: true,
            progressive_config: ProgressiveLoaderConfig::default(),
            progressive_threshold: 10 * 1024 * 1024, // 10MB threshold for progressive loading
        }
    }
}

/// Integrated preview service that combines file system metadata extraction with LRU caching
pub struct PreviewService<F: FileSystemService> {
    file_system: Arc<F>,
    cache: ThreadSafePreviewCache,
    config: PreviewServiceConfig,
    last_cleanup: Arc<std::sync::Mutex<SystemTime>>,
    progressive_loader: ProgressiveLoader,
}

impl<F: FileSystemService> PreviewService<F> {
    /// Create a new preview service with the given file system service and configuration
    pub fn new(file_system: Arc<F>, config: PreviewServiceConfig) -> Self {
        let cache = ThreadSafePreviewCache::new(config.cache_config.clone());
        let progressive_loader = ProgressiveLoader::new(config.progressive_config.clone());
        
        Self {
            file_system,
            cache,
            progressive_loader,
            config,
            last_cleanup: Arc::new(std::sync::Mutex::new(SystemTime::now())),
        }
    }

    /// Get preview data for a file, using cache-first strategy
    pub async fn get_preview(&self, path: &Path) -> Result<Option<CachedPreviewData>, PreviewServiceError> {
        debug!("Getting preview for: {}", path.display());

        // Check if file needs preview metadata
        let file_entry = self.file_system.get_metadata(path).await?;
        if !file_entry.needs_preview_metadata() {
            debug!("File doesn't need preview metadata: {}", path.display());
            return Ok(None);
        }

        // Check absolute file size limit
        if file_entry.size > self.config.max_file_size {
            warn!("File too large for preview: {} ({} bytes)", path.display(), file_entry.size);
            return Err(PreviewServiceError::GenerationFailed {
                reason: format!("File too large: {} bytes", file_entry.size),
            });
        }

        // Create cache key
        let cache_key = PreviewCacheKey::new(path.to_path_buf(), file_entry.modified);
        
        // Try to get from cache first
        if let Some(cached_data) = self.cache.get(&cache_key) {
            debug!("Cache hit for: {}", path.display());
            return Ok(Some(cached_data));
        }

        debug!("Cache miss for: {}", path.display());

        // Decide whether to use progressive loading or standard extraction
        let preview_data = if file_entry.size >= self.config.progressive_threshold {
            info!("Using progressive loading for large file: {} ({} bytes)", path.display(), file_entry.size);
            self.extract_and_cache_preview_progressive(path, &cache_key).await?
        } else {
            debug!("Using standard extraction for file: {} ({} bytes)", path.display(), file_entry.size);
            self.extract_and_cache_preview(path, &cache_key).await?
        };

        // Perform background cleanup if needed
        self.maybe_cleanup().await;

        Ok(preview_data)
    }

    /// Get file metadata with preview data, using cache when available
    pub async fn get_metadata_with_preview(&self, path: &Path) -> Result<FileEntry, PreviewServiceError> {
        debug!("Getting metadata with preview for: {}", path.display());

        let mut file_entry = self.file_system.get_metadata(path).await?;
        
        // If the file needs preview metadata, try to get cached version first
        if file_entry.needs_preview_metadata() {
            let cache_key = PreviewCacheKey::new(path.to_path_buf(), file_entry.modified);
            
            // Check cache for preview data
            if let Some(cached_data) = self.cache.get(&cache_key) {
                debug!("Using cached preview metadata for: {}", path.display());
                // Convert cached preview data back to preview metadata
                file_entry.preview_metadata = Some(self.convert_cached_to_metadata(&cached_data));
                return Ok(file_entry);
            }
        }

        // Fall back to file system extraction
        debug!("Extracting fresh metadata for: {}", path.display());
        self.file_system.get_metadata_with_preview(path).await.map_err(Into::into)
    }

    /// Force refresh of preview data, bypassing cache
    pub async fn refresh_preview(&self, path: &Path) -> Result<Option<CachedPreviewData>, PreviewServiceError> {
        debug!("Force refreshing preview for: {}", path.display());

        let file_entry = self.file_system.get_metadata(path).await?;
        if !file_entry.needs_preview_metadata() {
            return Ok(None);
        }

        // Remove any existing cached data
        let cache_key = PreviewCacheKey::new(path.to_path_buf(), file_entry.modified);
        self.cache.remove(&cache_key);

        // Extract fresh preview
        self.extract_and_cache_preview(path, &cache_key).await
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> PreviewCacheStats {
        self.cache.stats()
    }

    /// Manually trigger cache cleanup
    pub async fn cleanup_cache(&self) {
        info!("Manual cache cleanup triggered");
        let cleaned = self.cache.cleanup_old();
        debug!("Cleaned {} old entries from cache", cleaned);
        
        // Update last cleanup time
        if let Ok(mut last_cleanup) = self.last_cleanup.lock() {
            *last_cleanup = SystemTime::now();
        }
    }

    /// Check if the service is healthy (cache not too full, cleanup working)
    pub fn is_healthy(&self) -> bool {
        let stats = self.cache.stats();
        let memory_usage_percent = (stats.memory_bytes as f64 / self.config.cache_config.max_memory_bytes as f64) * 100.0;
        let entry_usage_percent = (stats.entries as f64 / self.config.cache_config.max_entries as f64) * 100.0;
        
        // Consider healthy if under 95% of limits
        memory_usage_percent < 95.0 && entry_usage_percent < 95.0
    }

    /// Start progressive loading for a large file and return a handle for monitoring progress
    pub async fn start_progressive_preview(&self, path: &Path) -> Result<ProgressiveLoadHandle, PreviewServiceError> {
        debug!("Starting progressive preview for: {}", path.display());

        // Check if file needs preview metadata
        let file_entry = self.file_system.get_metadata(path).await?;
        if !file_entry.needs_preview_metadata() {
            return Err(PreviewServiceError::UnsupportedFileType {
                path: path.display().to_string(),
            });
        }

        // Check file size limits
        if file_entry.size > self.config.max_file_size {
            return Err(PreviewServiceError::GenerationFailed {
                reason: format!("File too large: {} bytes", file_entry.size),
            });
        }

        // Start progressive loading
        let handle = self.progressive_loader.load_file(path).await?;
        Ok(handle)
    }

    /// Get service health information
    pub fn health_info(&self) -> PreviewServiceHealth {
        let stats = self.cache.stats();
        let memory_usage_percent = (stats.memory_bytes as f64 / self.config.cache_config.max_memory_bytes as f64) * 100.0;
        let entry_usage_percent = (stats.entries as f64 / self.config.cache_config.max_entries as f64) * 100.0;
        
        let last_cleanup = self.last_cleanup.lock()
            .map(|time| *time)
            .unwrap_or(SystemTime::UNIX_EPOCH);
            
        PreviewServiceHealth {
            is_healthy: self.is_healthy(),
            memory_usage_percent,
            entry_usage_percent,
            cache_hit_rate: stats.hit_rate,
            last_cleanup_age: SystemTime::now().duration_since(last_cleanup).unwrap_or(Duration::ZERO),
            total_entries: stats.entries,
            total_memory_bytes: stats.memory_bytes,
        }
    }

    // Private helper methods

    /// Extract preview using progressive loading and cache the result
    async fn extract_and_cache_preview_progressive(
        &self,
        path: &Path,
        cache_key: &PreviewCacheKey,
    ) -> Result<Option<CachedPreviewData>, PreviewServiceError> {
        debug!("Extracting preview using progressive loading for: {}", path.display());

        // Start progressive loading
        let handle = self.progressive_loader.load_file(path).await?;
        
        // Wait for completion (in production, you might want to provide progress callbacks)
        let cached_data = handle.await_result().await?;

        // Store in cache
        match self.cache.put(cache_key.clone(), cached_data.clone()) {
            Ok(()) => {
                debug!("Cached progressive preview data for: {}", path.display());
                Ok(Some(cached_data))
            }
            Err(e) => {
                warn!("Failed to cache progressive preview data for {}: {}", path.display(), e);
                // Return the data even if caching failed
                Ok(Some(cached_data))
            }
        }
    }

    /// Extract preview metadata from file system and cache the result
    async fn extract_and_cache_preview(
        &self,
        path: &Path,
        cache_key: &PreviewCacheKey,
    ) -> Result<Option<CachedPreviewData>, PreviewServiceError> {
        debug!("Extracting preview metadata for: {}", path.display());

        // Extract metadata using file system service
        let metadata = match self.file_system.extract_preview_metadata(path).await {
            Ok(metadata) => metadata,
            Err(e) => {
                warn!("Failed to extract preview metadata for {}: {}", path.display(), e);
                return Err(PreviewServiceError::FileSystem(e));
            }
        };

        // Check if we got meaningful metadata
        if !metadata.has_preview_data() {
            debug!("No meaningful preview data extracted for: {}", path.display());
            return Ok(None);
        }

        // Convert to cached preview data
        let cached_data = self.convert_metadata_to_cached(&metadata, path).await?;

        // Store in cache
        match self.cache.put(cache_key.clone(), cached_data.clone()) {
            Ok(()) => {
                debug!("Cached preview data for: {}", path.display());
                Ok(Some(cached_data))
            }
            Err(e) => {
                warn!("Failed to cache preview data for {}: {}", path.display(), e);
                // Return the data even if caching failed
                Ok(Some(cached_data))
            }
        }
    }

    /// Convert PreviewMetadata to CachedPreviewData
    async fn convert_metadata_to_cached(
        &self,
        metadata: &PreviewMetadata,
        path: &Path,
    ) -> Result<CachedPreviewData, PreviewServiceError> {
        // For now, we'll create placeholder preview data
        // In a production system, this would generate actual thumbnails, waveforms, etc.
        let preview_data = self.generate_placeholder_preview_data(metadata, path).await?;
        
        let preview_metadata = PreviewDataMetadata {
            width: metadata.width,
            height: metadata.height,
            duration: metadata.duration,
            quality_level: 80, // Standard quality
        };

        let content_type = metadata.mime_type.clone().unwrap_or_else(|| {
            match path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase().as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png" => "image/png",
                "gif" => "image/gif",
                "mp4" => "video/mp4",
                "mp3" => "audio/mpeg",
                _ => "application/octet-stream",
            }.to_string()
        });

        let format = match content_type.split('/').next().unwrap_or("unknown") {
            "image" => "thumbnail",
            "video" => "video_thumbnail",
            "audio" => "waveform",
            _ => "preview",
        }.to_string();

        let file_size = self.file_system.get_file_size(path).await.unwrap_or(0);

        Ok(CachedPreviewData::new(
            preview_data,
            content_type,
            format,
            file_size,
            preview_metadata,
        ))
    }

    /// Generate placeholder preview data (in production, this would create actual previews)
    async fn generate_placeholder_preview_data(
        &self,
        metadata: &PreviewMetadata,
        path: &Path,
    ) -> Result<Vec<u8>, PreviewServiceError> {
        // This is a placeholder implementation
        // In production, you would:
        // - For images: Generate thumbnails using image processing libraries
        // - For videos: Extract frames and create thumbnails using FFmpeg
        // - For audio: Generate waveform visualizations
        // - For documents: Generate page previews

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let placeholder_content = match metadata.mime_type.as_deref() {
            Some(mime) if mime.starts_with("image/") => {
                format!("THUMBNAIL:{}", file_name).into_bytes()
            }
            Some(mime) if mime.starts_with("video/") => {
                format!("VIDEO_PREVIEW:{}", file_name).into_bytes()
            }
            Some(mime) if mime.starts_with("audio/") => {
                format!("WAVEFORM:{}", file_name).into_bytes()
            }
            _ => {
                format!("PREVIEW:{}", file_name).into_bytes()
            }
        };

        Ok(placeholder_content)
    }

    /// Convert CachedPreviewData back to PreviewMetadata
    fn convert_cached_to_metadata(&self, cached_data: &CachedPreviewData) -> PreviewMetadata {
        PreviewMetadata {
            width: cached_data.metadata.width,
            height: cached_data.metadata.height,
            duration: cached_data.metadata.duration,
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
            mime_type: Some(cached_data.content_type.clone()),
        }
    }

    /// Maybe perform background cleanup if enough time has passed
    async fn maybe_cleanup(&self) {
        let should_cleanup = if let Ok(last_cleanup) = self.last_cleanup.lock() {
            last_cleanup.elapsed().unwrap_or(Duration::ZERO) > self.config.cleanup_interval
        } else {
            false
        };

        if should_cleanup {
            self.cleanup_cache().await;
        }
    }
}

/// Health information for the preview service
#[derive(Debug, Clone)]
pub struct PreviewServiceHealth {
    pub is_healthy: bool,
    pub memory_usage_percent: f64,
    pub entry_usage_percent: f64,
    pub cache_hit_rate: f64,
    pub last_cleanup_age: Duration,
    pub total_entries: usize,
    pub total_memory_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::file_system::{NativeFileSystemService, FileSystemConfig};
    use crate::services::progressive_loader::LoadingStage;
    use tempfile::TempDir;
    use std::sync::Arc;

    async fn create_test_service() -> (PreviewService<NativeFileSystemService>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let fs_config = FileSystemConfig::default();
        let fs_service = Arc::new(NativeFileSystemService::with_config(fs_config));
        
        let mut preview_config = PreviewServiceConfig::default();
        preview_config.cache_config.max_entries = 10;
        preview_config.cache_config.max_memory_bytes = 1024 * 1024; // 1MB
        
        let service = PreviewService::new(fs_service, preview_config);
        (service, temp_dir)
    }

    #[tokio::test]
    async fn test_preview_service_cache_hit() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create test image file
        let image_path = temp_dir.path().join("test.jpg");
        std::fs::write(&image_path, "fake jpeg content").unwrap();
        
        // First call should miss cache and extract
        let preview1 = service.get_preview(&image_path).await.unwrap();
        assert!(preview1.is_some());
        
        // Second call should hit cache
        let preview2 = service.get_preview(&image_path).await.unwrap();
        assert!(preview2.is_some());
        
        // Verify cache stats
        let stats = service.cache_stats();
        assert!(stats.hits > 0);
        assert!(stats.hit_rate > 0.0);
    }

    #[tokio::test]
    async fn test_preview_service_unsupported_file() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create text file (doesn't need preview)
        let text_path = temp_dir.path().join("test.txt");
        std::fs::write(&text_path, "just text content").unwrap();
        
        let preview = service.get_preview(&text_path).await.unwrap();
        assert!(preview.is_none());
    }

    #[tokio::test]
    async fn test_preview_service_file_too_large() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create large image file
        let large_path = temp_dir.path().join("large.jpg");
        let large_content = vec![0u8; 200 * 1024 * 1024]; // 200MB
        std::fs::write(&large_path, large_content).unwrap();
        
        let result = service.get_preview(&large_path).await;
        assert!(matches!(result, Err(PreviewServiceError::GenerationFailed { .. })));
    }

    #[tokio::test]
    async fn test_preview_service_refresh() {
        let (service, temp_dir) = create_test_service().await;
        
        let image_path = temp_dir.path().join("test.jpg");
        std::fs::write(&image_path, "original content").unwrap();
        
        // Get initial preview
        let preview1 = service.get_preview(&image_path).await.unwrap();
        assert!(preview1.is_some());
        
        // Modify file
        std::fs::write(&image_path, "modified content").unwrap();
        
        // Refresh should get new preview
        let preview2 = service.refresh_preview(&image_path).await.unwrap();
        assert!(preview2.is_some());
    }

    #[tokio::test]
    async fn test_preview_service_health() {
        let (service, _temp_dir) = create_test_service().await;
        
        assert!(service.is_healthy());
        
        let health = service.health_info();
        assert!(health.is_healthy);
        assert!(health.memory_usage_percent < 100.0);
        assert!(health.entry_usage_percent < 100.0);
    }

    #[tokio::test]
    async fn test_progressive_loading_integration() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create a large test file (above progressive threshold)
        let large_path = temp_dir.path().join("large_video.mp4");
        let large_size = 15 * 1024 * 1024; // 15MB - above 10MB threshold
        let large_content = vec![0xFFu8; large_size];
        std::fs::write(&large_path, large_content).unwrap();
        
        // Should use progressive loading for this file
        let preview = service.get_preview(&large_path).await.unwrap();
        assert!(preview.is_some());
        
        let preview_data = preview.unwrap();
        assert!(!preview_data.data.is_empty());
        assert_eq!(preview_data.original_size, large_size as u64);
        assert_eq!(preview_data.format, "progressive_video_thumbnail");
        
        // Verify it was cached
        let stats = service.cache_stats();
        assert!(stats.entries > 0);
    }

    #[tokio::test]
    async fn test_progressive_loading_handle() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create a large test file
        let large_path = temp_dir.path().join("handle_test.mp4");
        let large_size = 20 * 1024 * 1024; // 20MB
        let large_content = vec![0xAAu8; large_size];
        std::fs::write(&large_path, large_content).unwrap();
        
        // Start progressive loading and get handle
        let mut handle = service.start_progressive_preview(&large_path).await.unwrap();
        
        // Monitor progress
        let mut progress_updates = 0;
        let mut final_progress = None;
        
        while let Some(progress) = handle.next_progress().await {
            progress_updates += 1;
            assert!(progress.percentage >= 0.0 && progress.percentage <= 100.0);
            final_progress = Some(progress.clone());
            
            if progress.stage == LoadingStage::Complete {
                break;
            }
        }
        
        // Should have received progress updates
        assert!(progress_updates > 0);
        
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
    async fn test_progressive_vs_standard_threshold() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create a small file (below progressive threshold)
        let small_path = temp_dir.path().join("small_image.jpg");
        let small_content = vec![0x55u8; 5 * 1024 * 1024]; // 5MB - below 10MB threshold
        std::fs::write(&small_path, small_content).unwrap();
        
        // Should use standard extraction
        let small_preview = service.get_preview(&small_path).await.unwrap();
        assert!(small_preview.is_some());
        let small_data = small_preview.unwrap();
        assert_eq!(small_data.format, "thumbnail"); // Standard format
        
        // Create a large file (above progressive threshold)
        let large_path = temp_dir.path().join("large_image.jpg");
        let large_content = vec![0x66u8; 15 * 1024 * 1024]; // 15MB - above 10MB threshold
        std::fs::write(&large_path, large_content).unwrap();
        
        // Should use progressive loading
        let large_preview = service.get_preview(&large_path).await.unwrap();
        assert!(large_preview.is_some());
        let large_data = large_preview.unwrap();
        assert_eq!(large_data.format, "progressive_thumbnail"); // Progressive format
    }

    #[tokio::test]
    async fn test_progressive_loading_cancellation() {
        let (service, temp_dir) = create_test_service().await;
        
        // Create a large test file
        let large_path = temp_dir.path().join("cancel_test.mp4");
        let large_content = vec![0x77u8; 25 * 1024 * 1024]; // 25MB
        std::fs::write(&large_path, large_content).unwrap();
        
        // Start progressive loading
        let handle = service.start_progressive_preview(&large_path).await.unwrap();
        
        // Cancel immediately
        handle.cancel();
        assert!(handle.is_cancelled());
        
        // Should get cancellation error
        let result = handle.await_result().await;
        assert!(matches!(result, Err(ProgressiveLoaderError::Cancelled)));
    }

    #[tokio::test]
    async fn test_progressive_loading_memory_management() {
        let (service, temp_dir) = create_test_service().await;
        
        // Test multiple large files to ensure memory management works
        let files = vec!["file1.mp4", "file2.avi", "file3.mkv"];
        let file_size = 15 * 1024 * 1024; // 15MB each
        
        for file_name in &files {
            let file_path = temp_dir.path().join(file_name);
            let content = vec![0x88u8; file_size];
            std::fs::write(&file_path, content).unwrap();
            
            // Load each file progressively
            let preview = service.get_preview(&file_path).await.unwrap();
            assert!(preview.is_some());
        }
        
        // Cache should manage memory correctly
        let stats = service.cache_stats();
        assert!(stats.memory_bytes <= service.config.cache_config.max_memory_bytes);
        
        // Service should remain healthy
        assert!(service.is_healthy());
    }

    #[tokio::test]
    async fn test_metadata_with_preview_cached() {
        let (service, temp_dir) = create_test_service().await;
        
        let image_path = temp_dir.path().join("test.jpg");
        std::fs::write(&image_path, "fake jpeg content").unwrap();
        
        // First call should extract and cache
        let entry1 = service.get_metadata_with_preview(&image_path).await.unwrap();
        assert!(entry1.has_preview_metadata());
        
        // Second call should use cached data
        let entry2 = service.get_metadata_with_preview(&image_path).await.unwrap();
        assert!(entry2.has_preview_metadata());
        
        // Cache should have entries
        let stats = service.cache_stats();
        assert!(stats.entries > 0);
    }
}