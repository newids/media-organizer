use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, Duration};
use tokio::sync::{RwLock, mpsc, oneshot, Semaphore};
use tokio::time::{sleep, Instant};
use uuid::Uuid;
use tracing::{info, warn, debug};

use crate::services::preview::{
    PreviewHandler, PreviewError, PreviewConfig, SupportedFormat,
    ImagePreviewHandler, VideoPreviewHandler, AudioPreviewHandler, 
    PdfPreviewHandler, TextPreviewHandler
};
use crate::services::cache::CacheService;

/// Priority levels for thumbnail generation jobs
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThumbnailPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Status of a thumbnail generation job
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThumbnailJobStatus {
    Queued,
    Processing,
    Completed,
    Failed(String),
    Cancelled,
}

/// Configuration for thumbnail generation jobs
#[derive(Debug, Clone)]
pub struct ThumbnailJobConfig {
    pub size: (u32, u32),
    pub priority: ThumbnailPriority,
    pub timeout: Duration,
    pub max_retries: u32,
    pub cache_result: bool,
}

impl Default for ThumbnailJobConfig {
    fn default() -> Self {
        Self {
            size: (256, 256),
            priority: ThumbnailPriority::Normal,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            cache_result: true,
        }
    }
}

/// A thumbnail generation job
#[derive(Debug)]
pub struct ThumbnailJob {
    pub id: Uuid,
    pub file_path: PathBuf,
    pub config: ThumbnailJobConfig,
    pub created_at: SystemTime,
    pub status: ThumbnailJobStatus,
    pub retry_count: u32,
    pub result_sender: Option<oneshot::Sender<Result<Vec<u8>, PreviewError>>>,
}

impl ThumbnailJob {
    pub fn new(file_path: PathBuf, config: ThumbnailJobConfig) -> (Self, oneshot::Receiver<Result<Vec<u8>, PreviewError>>) {
        let (sender, receiver) = oneshot::channel();
        
        let job = Self {
            id: Uuid::new_v4(),
            file_path,
            config,
            created_at: SystemTime::now(),
            status: ThumbnailJobStatus::Queued,
            retry_count: 0,
            result_sender: Some(sender),
        };
        
        (job, receiver)
    }
}

/// Statistics for the thumbnail service
#[derive(Debug, Clone)]
pub struct ThumbnailServiceStats {
    pub total_jobs_processed: u64,
    pub successful_generations: u64,
    pub failed_generations: u64,
    pub cancelled_jobs: u64,
    pub average_processing_time: Duration,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_jobs: usize,
    pub queued_jobs: usize,
}

/// Main thumbnail generation service with background processing
pub struct ThumbnailService {
    /// Preview handlers for different file formats
    handlers: Arc<Vec<Box<dyn PreviewHandler + Send + Sync>>>,
    /// Job queue with priority ordering
    job_queue: Arc<RwLock<VecDeque<ThumbnailJob>>>,
    /// Currently active jobs
    active_jobs: Arc<RwLock<HashMap<Uuid, ThumbnailJob>>>,
    /// Service statistics
    stats: Arc<RwLock<ThumbnailServiceStats>>,
    /// Cache service for thumbnail persistence
    cache_service: Option<Arc<CacheService>>,
    /// Preview configuration
    config: PreviewConfig,
    /// Semaphore to limit concurrent thumbnail generation
    processing_semaphore: Arc<Semaphore>,
    /// Channel for shutdown signaling
    shutdown_tx: Option<mpsc::UnboundedSender<()>>,
    /// Service configuration
    max_concurrent_jobs: usize,
    max_queue_size: usize,
}

impl ThumbnailService {
    /// Create a new thumbnail service
    pub fn new(config: PreviewConfig) -> Self {
        let handlers: Vec<Box<dyn PreviewHandler + Send + Sync>> = vec![
            Box::new(ImagePreviewHandler::default()),
            Box::new(VideoPreviewHandler::new().expect("Failed to create VideoPreviewHandler")),
            Box::new(AudioPreviewHandler::new().expect("Failed to create AudioPreviewHandler")),
            Box::new(PdfPreviewHandler::new().expect("Failed to create PdfPreviewHandler")),
            Box::new(TextPreviewHandler::new().expect("Failed to create TextPreviewHandler")),
        ];

        let max_concurrent_jobs = num_cpus::get().max(2).min(8);

        Self {
            handlers: Arc::new(handlers),
            job_queue: Arc::new(RwLock::new(VecDeque::new())),
            active_jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ThumbnailServiceStats {
                total_jobs_processed: 0,
                successful_generations: 0,
                failed_generations: 0,
                cancelled_jobs: 0,
                average_processing_time: Duration::from_millis(0),
                cache_hits: 0,
                cache_misses: 0,
                active_jobs: 0,
                queued_jobs: 0,
            })),
            cache_service: None,
            config,
            processing_semaphore: Arc::new(Semaphore::new(max_concurrent_jobs)),
            shutdown_tx: None,
            max_concurrent_jobs,
            max_queue_size: 1000,
        }
    }

    /// Create thumbnail service with cache integration
    pub fn with_cache(mut self, cache_service: Arc<CacheService>) -> Self {
        self.cache_service = Some(cache_service);
        self
    }

    /// Start the background processing loop
    pub async fn start(&mut self) -> Result<(), PreviewError> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel();
        self.shutdown_tx = Some(shutdown_tx);

        let job_queue = Arc::clone(&self.job_queue);
        let active_jobs = Arc::clone(&self.active_jobs);
        let stats = Arc::clone(&self.stats);
        let handlers = Arc::clone(&self.handlers);
        let cache_service = self.cache_service.clone();
        let config = self.config.clone();
        let processing_semaphore = Arc::clone(&self.processing_semaphore);

        tokio::spawn(async move {
            info!("Starting thumbnail service background processor");
            
            let mut processing_interval = tokio::time::interval(Duration::from_millis(100));
            
            loop {
                tokio::select! {
                    _ = processing_interval.tick() => {
                        Self::process_queue(
                            &job_queue,
                            &active_jobs,
                            &stats,
                            &handlers,
                            &cache_service,
                            &config,
                            &processing_semaphore,
                        ).await;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Shutting down thumbnail service background processor");
                        break;
                    }
                }
            }
        });

        info!("Thumbnail service started with {} max concurrent jobs", self.max_concurrent_jobs);
        Ok(())
    }

    /// Stop the background processing
    pub async fn stop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
            
            // Wait for active jobs to complete with timeout
            let timeout = Duration::from_secs(10);
            let start = Instant::now();
            
            while start.elapsed() < timeout {
                let active_count = self.active_jobs.read().await.len();
                if active_count == 0 {
                    break;
                }
                sleep(Duration::from_millis(100)).await;
            }
            
            info!("Thumbnail service stopped");
        }
    }

    /// Queue a thumbnail generation job
    pub async fn generate_thumbnail_async(
        &self, 
        file_path: impl AsRef<Path>,
        config: ThumbnailJobConfig,
    ) -> Result<oneshot::Receiver<Result<Vec<u8>, PreviewError>>, PreviewError> {
        let file_path = file_path.as_ref().to_path_buf();
        
        // Check if file exists
        if !file_path.exists() {
            return Err(PreviewError::FileNotFound(file_path));
        }

        // Check queue size limit
        let queue_size = self.job_queue.read().await.len();
        if queue_size >= self.max_queue_size {
            return Err(PreviewError::TaskError("Thumbnail queue is full".to_string()));
        }

        // Check cache first if enabled
        if config.cache_result {
            if let Some(cache) = &self.cache_service {
                if let Ok(Some(_cached)) = cache.get_thumbnail_path(&file_path).await {
                    // TODO: Return cached thumbnail data when cache API is updated
                    debug!("Found cached thumbnail for {:?}", file_path);
                }
            }
        }

        // Create and queue the job
        let (job, receiver) = ThumbnailJob::new(file_path.clone(), config);
        let job_id = job.id;
        
        {
            let mut queue = self.job_queue.write().await;
            
            // Insert job maintaining priority order
            let insert_pos = queue.iter().position(|existing_job| {
                existing_job.config.priority < job.config.priority
            }).unwrap_or(queue.len());
            
            queue.insert(insert_pos, job);
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.queued_jobs = self.job_queue.read().await.len();
        }

        debug!("Queued thumbnail job {} for {:?}", job_id, file_path);
        Ok(receiver)
    }

    /// Process the job queue
    async fn process_queue(
        job_queue: &Arc<RwLock<VecDeque<ThumbnailJob>>>,
        active_jobs: &Arc<RwLock<HashMap<Uuid, ThumbnailJob>>>,
        stats: &Arc<RwLock<ThumbnailServiceStats>>,
        handlers: &Arc<Vec<Box<dyn PreviewHandler + Send + Sync>>>,
        cache_service: &Option<Arc<CacheService>>,
        config: &PreviewConfig,
        processing_semaphore: &Arc<Semaphore>,
    ) {
        // Try to acquire semaphore permit for concurrent job limiting  
        let permit = match Arc::clone(processing_semaphore).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return, // All processing slots are busy
        };

        // Get next job from queue
        let job = {
            let mut queue = job_queue.write().await;
            queue.pop_front()
        };

        let Some(mut job) = job else {
            return; // No jobs in queue
        };

        let job_id = job.id;
        let file_path = job.file_path.clone();
        let job_config = job.config.clone();

        // Move job to active jobs
        job.status = ThumbnailJobStatus::Processing;
        {
            let mut active = active_jobs.write().await;
            active.insert(job_id, job);
        }

        // Update stats
        {
            let mut stats = stats.write().await;
            stats.active_jobs = active_jobs.read().await.len();
            stats.queued_jobs = job_queue.read().await.len();
        }

        // Spawn processing task
        let active_jobs_clone = Arc::clone(active_jobs);
        let stats_clone = Arc::clone(stats);
        let handlers_clone = Arc::clone(handlers);
        let cache_service_clone = cache_service.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            let _permit = permit; // Keep permit until task completes
            let start_time = Instant::now();

            debug!("Processing thumbnail job {} for {:?}", job_id, file_path);

            let result = Self::generate_thumbnail_internal(
                &file_path,
                &job_config,
                &handlers_clone,
                &cache_service_clone,
                &config_clone,
            ).await;

            let processing_time = start_time.elapsed();
            let result_success = result.is_ok();

            // Update job status and send result
            {
                let mut active = active_jobs_clone.write().await;
                if let Some(mut job) = active.remove(&job_id) {
                    match &result {
                        Ok(_) => {
                            job.status = ThumbnailJobStatus::Completed;
                            debug!("Completed thumbnail job {} in {:?}", job_id, processing_time);
                        }
                        Err(e) => {
                            job.status = ThumbnailJobStatus::Failed(e.to_string());
                            warn!("Failed thumbnail job {}: {}", job_id, e);
                        }
                    }

                    // Send result if receiver is still waiting
                    if let Some(sender) = job.result_sender.take() {
                        let _ = sender.send(result);
                    }
                }
            }

            // Update stats
            {
                let mut stats = stats_clone.write().await;
                stats.total_jobs_processed += 1;
                if result_success {
                    stats.successful_generations += 1;
                } else {
                    stats.failed_generations += 1;
                }
                
                // Update average processing time
                let total_time = stats.average_processing_time * (stats.total_jobs_processed - 1) as u32 + processing_time;
                stats.average_processing_time = total_time / stats.total_jobs_processed as u32;
                
                stats.active_jobs = active_jobs_clone.read().await.len();
            }
        });
    }

    /// Internal thumbnail generation logic
    async fn generate_thumbnail_internal(
        file_path: &Path,
        config: &ThumbnailJobConfig,
        handlers: &Vec<Box<dyn PreviewHandler + Send + Sync>>,
        cache_service: &Option<Arc<CacheService>>,
        _preview_config: &PreviewConfig,
    ) -> Result<Vec<u8>, PreviewError> {
        // Detect file format
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        ).ok_or_else(|| PreviewError::UnsupportedFormat("Unknown format".to_string()))?;

        // Find appropriate handler
        let handler = handlers.iter()
            .find(|h| h.supports_format(format))
            .ok_or_else(|| PreviewError::UnsupportedFormat(format!("{:?}", format)))?;

        // Generate thumbnail with timeout
        let thumbnail_result = tokio::time::timeout(
            config.timeout,
            handler.generate_thumbnail(file_path, config.size)
        ).await;

        match thumbnail_result {
            Ok(Ok(thumbnail_data)) => {
                // Cache the result if enabled
                if config.cache_result {
                    if let Some(_cache) = cache_service {
                        // TODO: Store thumbnail in cache when API is available
                        debug!("Thumbnail caching not yet fully implemented");
                    }
                }
                Ok(thumbnail_data)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(PreviewError::TaskError("Thumbnail generation timed out".to_string())),
        }
    }

    /// Get current service statistics
    pub async fn get_stats(&self) -> ThumbnailServiceStats {
        self.stats.read().await.clone()
    }

    /// Cancel a specific job by ID
    pub async fn cancel_job(&self, job_id: Uuid) -> bool {
        // Check active jobs first
        {
            let mut active = self.active_jobs.write().await;
            if let Some(mut job) = active.remove(&job_id) {
                job.status = ThumbnailJobStatus::Cancelled;
                if let Some(sender) = job.result_sender.take() {
                    let _ = sender.send(Err(PreviewError::TaskError("Job cancelled".to_string())));
                }
                
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.cancelled_jobs += 1;
                }
                
                return true;
            }
        }

        // Check queued jobs
        {
            let mut queue = self.job_queue.write().await;
            if let Some(pos) = queue.iter().position(|job| job.id == job_id) {
                if let Some(mut job) = queue.remove(pos) {
                    job.status = ThumbnailJobStatus::Cancelled;
                    if let Some(sender) = job.result_sender.take() {
                        let _ = sender.send(Err(PreviewError::TaskError("Job cancelled".to_string())));
                    }
                    
                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.cancelled_jobs += 1;
                    }
                    
                    return true;
                }
            }
        }

        false
    }

    /// Clear all queued jobs
    pub async fn clear_queue(&self) -> usize {
        let mut queue = self.job_queue.write().await;
        let cleared_count = queue.len();
        
        // Cancel all queued jobs
        for mut job in queue.drain(..) {
            job.status = ThumbnailJobStatus::Cancelled;
            if let Some(sender) = job.result_sender.take() {
                let _ = sender.send(Err(PreviewError::TaskError("Queue cleared".to_string())));
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.cancelled_jobs += cleared_count as u64;
            stats.queued_jobs = 0;
        }

        cleared_count
    }

    /// Check if the service supports a given file format
    pub fn supports_format(&self, format: SupportedFormat) -> bool {
        self.handlers.iter().any(|handler| handler.supports_format(format))
    }

    /// Get the number of jobs in queue
    pub async fn queue_size(&self) -> usize {
        self.job_queue.read().await.len()
    }

    /// Get the number of active jobs
    pub async fn active_jobs_count(&self) -> usize {
        self.active_jobs.read().await.len()
    }
}

impl Drop for ThumbnailService {
    fn drop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_thumbnail_service_creation() {
        let config = PreviewConfig::default();
        let service = ThumbnailService::new(config);
        assert_eq!(service.max_concurrent_jobs, num_cpus::get().max(2).min(8));
        assert!(service.cache_service.is_none());
    }

    #[tokio::test]
    async fn test_job_creation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.jpg");
        fs::write(&file_path, b"fake image data").unwrap();
        
        let config = ThumbnailJobConfig::default();
        let (job, _receiver) = ThumbnailJob::new(file_path.clone(), config.clone());
        
        assert_eq!(job.file_path, file_path);
        assert_eq!(job.config.size, (256, 256));
        assert_eq!(job.config.priority, ThumbnailPriority::Normal);
        assert_eq!(job.status, ThumbnailJobStatus::Queued);
        assert_eq!(job.retry_count, 0);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        assert!(ThumbnailPriority::Urgent > ThumbnailPriority::High);
        assert!(ThumbnailPriority::High > ThumbnailPriority::Normal);
        assert!(ThumbnailPriority::Normal > ThumbnailPriority::Low);
    }

    #[tokio::test]
    async fn test_job_queue_operations() {
        let config = PreviewConfig::default();
        let service = ThumbnailService::new(config);
        
        assert_eq!(service.queue_size().await, 0);
        assert_eq!(service.active_jobs_count().await, 0);
    }

    #[tokio::test]
    async fn test_format_support() {
        let config = PreviewConfig::default();
        let service = ThumbnailService::new(config);
        
        assert!(service.supports_format(SupportedFormat::Jpeg));
        assert!(service.supports_format(SupportedFormat::Mp4));
        assert!(service.supports_format(SupportedFormat::Mp3));
        assert!(service.supports_format(SupportedFormat::Pdf));
        assert!(service.supports_format(SupportedFormat::Text));
    }

    #[tokio::test]
    async fn test_stats_initialization() {
        let config = PreviewConfig::default();
        let service = ThumbnailService::new(config);
        let stats = service.get_stats().await;
        
        assert_eq!(stats.total_jobs_processed, 0);
        assert_eq!(stats.successful_generations, 0);
        assert_eq!(stats.failed_generations, 0);
        assert_eq!(stats.cancelled_jobs, 0);
        assert_eq!(stats.active_jobs, 0);
        assert_eq!(stats.queued_jobs, 0);
    }

    #[tokio::test]
    async fn test_nonexistent_file() {
        let config = PreviewConfig::default();
        let service = ThumbnailService::new(config);
        let job_config = ThumbnailJobConfig::default();
        
        let result = service.generate_thumbnail_async("/nonexistent/file.jpg", job_config).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PreviewError::FileNotFound(_) => {}, // Expected
            other => panic!("Expected FileNotFound, got {:?}", other),
        }
    }
}