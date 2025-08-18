use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use thiserror::Error;
use uuid::Uuid;

use crate::services::{HashingService, FileHash};

/// Errors that can occur during background processing
#[derive(Debug, Error)]
pub enum BackgroundError {
    #[error("Task was cancelled")]
    Cancelled,
    
    #[error("Task not found: {id}")]
    TaskNotFound { id: Uuid },
    
    #[error("Task already running: {id}")]
    TaskAlreadyRunning { id: Uuid },
    
    #[error("Channel communication error")]
    ChannelError,
    
    #[error("File processing error: {0}")]
    ProcessingError(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for background operations
pub type BackgroundResult<T> = Result<T, BackgroundError>;

/// Status of a background task
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Progress information for a background task
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Unique task identifier
    pub task_id: Uuid,
    /// Current task status
    pub status: TaskStatus,
    /// Number of files processed
    pub files_processed: usize,
    /// Total number of files to process
    pub total_files: usize,
    /// Number of bytes processed
    pub bytes_processed: u64,
    /// Total bytes to process
    pub total_bytes: u64,
    /// Current file being processed
    pub current_file: Option<PathBuf>,
    /// Time when task started
    pub start_time: Instant,
    /// Estimated time remaining in seconds
    pub estimated_remaining_secs: Option<u64>,
    /// Processing rate in files per second
    pub files_per_second: f64,
    /// Processing rate in bytes per second
    pub bytes_per_second: f64,
    /// Any error that occurred
    pub error: Option<String>,
}

impl ProgressInfo {
    /// Create new progress info for a task
    pub fn new(task_id: Uuid, total_files: usize, total_bytes: u64) -> Self {
        Self {
            task_id,
            status: TaskStatus::Pending,
            files_processed: 0,
            total_files,
            bytes_processed: 0,
            total_bytes,
            current_file: None,
            start_time: Instant::now(),
            estimated_remaining_secs: None,
            files_per_second: 0.0,
            bytes_per_second: 0.0,
            error: None,
        }
    }
    
    /// Get completion percentage (0.0 to 1.0)
    pub fn completion_percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            self.files_processed as f64 / self.total_files as f64
        }
    }
    
    /// Get bytes completion percentage (0.0 to 1.0)
    pub fn bytes_completion_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            self.bytes_processed as f64 / self.total_bytes as f64
        }
    }
    
    /// Update progress with new file processing
    pub fn update_progress(&mut self, file_path: PathBuf, file_size: u64) {
        self.files_processed += 1;
        self.bytes_processed += file_size;
        self.current_file = Some(file_path);
        
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        if elapsed_secs > 0.0 {
            self.files_per_second = self.files_processed as f64 / elapsed_secs;
            self.bytes_per_second = self.bytes_processed as f64 / elapsed_secs;
            
            // Estimate remaining time based on files processed
            if self.files_processed > 0 && self.total_files > self.files_processed {
                let remaining_files = self.total_files - self.files_processed;
                let estimated_secs = remaining_files as f64 / self.files_per_second;
                self.estimated_remaining_secs = Some(estimated_secs as u64);
            }
        }
    }
    
    /// Mark task as completed
    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.estimated_remaining_secs = Some(0);
        self.current_file = None;
    }
    
    /// Mark task as failed with error
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed;
        self.error = Some(error);
        self.current_file = None;
    }
    
    /// Mark task as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.current_file = None;
    }
}

/// Progress callback function type
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// Background task for file hashing
pub struct HashingTask {
    /// Unique task identifier
    pub id: Uuid,
    /// Files to hash
    pub files: Vec<PathBuf>,
    /// Total size of all files
    pub total_size: u64,
    /// Progress callback
    pub progress_callback: ProgressCallback,
    /// Cancellation token
    pub cancellation_token: CancellationToken,
}

impl std::fmt::Debug for HashingTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HashingTask")
            .field("id", &self.id)
            .field("files", &self.files)
            .field("total_size", &self.total_size)
            .field("cancellation_token", &self.cancellation_token)
            .finish()
    }
}

impl HashingTask {
    /// Create a new hashing task
    pub fn new(
        files: Vec<PathBuf>,
        total_size: u64,
        progress_callback: ProgressCallback,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            files,
            total_size,
            progress_callback,
            cancellation_token: CancellationToken::new(),
        }
    }
    
    /// Get cancellation token for this task
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }
}

/// Results from a completed hashing task
#[derive(Debug, Clone)]
pub struct HashingTaskResult {
    /// Task ID
    pub task_id: Uuid,
    /// Successfully hashed files
    pub successful_hashes: Vec<FileHash>,
    /// Files that failed to hash with error messages
    pub failed_files: Vec<(PathBuf, String)>,
    /// Final progress information
    pub final_progress: ProgressInfo,
}

/// Background processor for file operations
#[derive(Debug)]
pub struct BackgroundProcessor {
    /// Hashing service
    hashing_service: Arc<HashingService>,
    /// Currently running tasks
    running_tasks: Arc<RwLock<std::collections::HashMap<Uuid, CancellationToken>>>,
    /// Task results
    completed_tasks: Arc<Mutex<std::collections::HashMap<Uuid, HashingTaskResult>>>,
}

impl BackgroundProcessor {
    /// Create a new background processor
    pub fn new(hashing_service: HashingService) -> Self {
        Self {
            hashing_service: Arc::new(hashing_service),
            running_tasks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            completed_tasks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    /// Start a hashing task in the background
    pub async fn start_hashing_task(&self, task: HashingTask) -> BackgroundResult<Uuid> {
        let task_id = task.id;
        
        // Check if task is already running
        {
            let running_tasks = self.running_tasks.read().await;
            if running_tasks.contains_key(&task_id) {
                return Err(BackgroundError::TaskAlreadyRunning { id: task_id });
            }
        }
        
        // Add to running tasks
        {
            let mut running_tasks = self.running_tasks.write().await;
            running_tasks.insert(task_id, task.cancellation_token.clone());
        }
        
        let hashing_service = self.hashing_service.clone();
        let running_tasks = self.running_tasks.clone();
        let completed_tasks = self.completed_tasks.clone();
        
        // Spawn the background task
        tokio::spawn(async move {
            let result = Self::execute_hashing_task(hashing_service, task).await;
            
            // Remove from running tasks
            {
                let mut running_tasks = running_tasks.write().await;
                running_tasks.remove(&task_id);
            }
            
            // Store result
            if let Ok(task_result) = result {
                let mut completed_tasks = completed_tasks.lock().await;
                completed_tasks.insert(task_id, task_result);
            }
        });
        
        info!("Started background hashing task: {}", task_id);
        Ok(task_id)
    }
    
    /// Execute a hashing task
    async fn execute_hashing_task(
        hashing_service: Arc<HashingService>,
        task: HashingTask,
    ) -> BackgroundResult<HashingTaskResult> {
        let mut progress = ProgressInfo::new(task.id, task.files.len(), task.total_size);
        progress.status = TaskStatus::Running;
        
        // Initial progress callback
        (task.progress_callback)(progress.clone());
        
        let mut successful_hashes = Vec::new();
        let mut failed_files = Vec::new();
        
        for file_path in &task.files {
            // Check for cancellation
            if task.cancellation_token.is_cancelled() {
                progress.mark_cancelled();
                (task.progress_callback)(progress.clone());
                return Err(BackgroundError::Cancelled);
            }
            
            // Get file size for progress tracking
            let file_size = match tokio::fs::metadata(file_path).await {
                Ok(metadata) => metadata.len(),
                Err(e) => {
                    warn!("Failed to get metadata for {}: {}", file_path.display(), e);
                    failed_files.push((file_path.clone(), e.to_string()));
                    continue;
                }
            };
            
            // Hash the file
            match hashing_service.hash_file(file_path).await {
                Ok(file_hash) => {
                    debug!("Successfully hashed: {}", file_path.display());
                    successful_hashes.push(file_hash);
                }
                Err(e) => {
                    warn!("Failed to hash {}: {}", file_path.display(), e);
                    failed_files.push((file_path.clone(), e.to_string()));
                }
            }
            
            // Update progress
            progress.update_progress(file_path.clone(), file_size);
            (task.progress_callback)(progress.clone());
            
            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }
        
        // Mark as completed
        progress.mark_completed();
        (task.progress_callback)(progress.clone());
        
        info!(
            "Completed hashing task {}: {} successful, {} failed",
            task.id,
            successful_hashes.len(),
            failed_files.len()
        );
        
        Ok(HashingTaskResult {
            task_id: task.id,
            successful_hashes,
            failed_files,
            final_progress: progress,
        })
    }
    
    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: Uuid) -> BackgroundResult<()> {
        let running_tasks = self.running_tasks.read().await;
        
        if let Some(cancellation_token) = running_tasks.get(&task_id) {
            cancellation_token.cancel();
            info!("Cancelled background task: {}", task_id);
            Ok(())
        } else {
            Err(BackgroundError::TaskNotFound { id: task_id })
        }
    }
    
    /// Check if a task is running
    pub async fn is_task_running(&self, task_id: Uuid) -> bool {
        let running_tasks = self.running_tasks.read().await;
        running_tasks.contains_key(&task_id)
    }
    
    /// Get the result of a completed task
    pub async fn get_task_result(&self, task_id: Uuid) -> Option<HashingTaskResult> {
        let completed_tasks = self.completed_tasks.lock().await;
        completed_tasks.get(&task_id).cloned()
    }
    
    /// Get all running task IDs
    pub async fn get_running_tasks(&self) -> Vec<Uuid> {
        let running_tasks = self.running_tasks.read().await;
        running_tasks.keys().cloned().collect()
    }
    
    /// Cancel all running tasks
    pub async fn cancel_all_tasks(&self) {
        let running_tasks = self.running_tasks.read().await;
        
        for (task_id, cancellation_token) in running_tasks.iter() {
            cancellation_token.cancel();
            info!("Cancelled background task: {}", task_id);
        }
    }
    
    /// Clear completed task results
    pub async fn clear_completed_tasks(&self) {
        let mut completed_tasks = self.completed_tasks.lock().await;
        completed_tasks.clear();
        info!("Cleared all completed task results");
    }
}

impl Default for BackgroundProcessor {
    fn default() -> Self {
        Self::new(HashingService::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_progress_info_creation() {
        let task_id = Uuid::new_v4();
        let progress = ProgressInfo::new(task_id, 100, 1024 * 1024);
        
        assert_eq!(progress.task_id, task_id);
        assert_eq!(progress.total_files, 100);
        assert_eq!(progress.total_bytes, 1024 * 1024);
        assert_eq!(progress.files_processed, 0);
        assert_eq!(progress.completion_percentage(), 0.0);
    }
    
    #[tokio::test]
    async fn test_progress_info_updates() {
        let task_id = Uuid::new_v4();
        let mut progress = ProgressInfo::new(task_id, 10, 1024);
        
        // Update progress
        progress.update_progress(PathBuf::from("test1.txt"), 100);
        assert_eq!(progress.files_processed, 1);
        assert_eq!(progress.bytes_processed, 100);
        assert_eq!(progress.completion_percentage(), 0.1);
        
        // Add more progress
        progress.update_progress(PathBuf::from("test2.txt"), 200);
        assert_eq!(progress.files_processed, 2);
        assert_eq!(progress.bytes_processed, 300);
        assert_eq!(progress.completion_percentage(), 0.2);
    }
    
    #[tokio::test]
    async fn test_hashing_task_creation() {
        let files = vec![PathBuf::from("test1.txt"), PathBuf::from("test2.txt")];
        let callback = Arc::new(|_progress: ProgressInfo| {});
        
        let task = HashingTask::new(files.clone(), 1024, callback);
        
        assert_eq!(task.files, files);
        assert_eq!(task.total_size, 1024);
        assert!(!task.cancellation_token.is_cancelled());
    }
    
    #[tokio::test]
    async fn test_background_processor_creation() {
        let processor = BackgroundProcessor::default();
        
        assert!(processor.get_running_tasks().await.is_empty());
    }
    
    #[tokio::test]
    async fn test_simple_hashing_task() {
        // Create test files
        let temp_file1 = NamedTempFile::new().unwrap();
        let temp_file2 = NamedTempFile::new().unwrap();
        
        // Write test content
        {
            let mut file1 = tokio::fs::File::create(temp_file1.path()).await.unwrap();
            file1.write_all(b"hello").await.unwrap();
            file1.flush().await.unwrap();
            
            let mut file2 = tokio::fs::File::create(temp_file2.path()).await.unwrap();
            file2.write_all(b"world").await.unwrap();
            file2.flush().await.unwrap();
        }
        
        let files = vec![
            temp_file1.path().to_path_buf(),
            temp_file2.path().to_path_buf(),
        ];
        
        // Track progress updates
        let progress_count = Arc::new(AtomicUsize::new(0));
        let progress_count_clone = progress_count.clone();
        
        let callback = Arc::new(move |progress: ProgressInfo| {
            progress_count_clone.fetch_add(1, Ordering::SeqCst);
            println!("Progress: {}/{} files", progress.files_processed, progress.total_files);
        });
        
        let task = HashingTask::new(files, 10, callback);
        let task_id = task.id;
        
        let processor = BackgroundProcessor::default();
        
        // Start the task
        let result_id = processor.start_hashing_task(task).await.unwrap();
        assert_eq!(result_id, task_id);
        
        // Wait for task to complete
        for _ in 0..50 {
            if !processor.is_task_running(task_id).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Verify task completed
        assert!(!processor.is_task_running(task_id).await);
        
        // Check for results
        let task_result = processor.get_task_result(task_id).await;
        assert!(task_result.is_some());
        
        let result = task_result.unwrap();
        assert_eq!(result.successful_hashes.len(), 2);
        assert!(result.failed_files.is_empty());
        assert_eq!(result.final_progress.status, TaskStatus::Completed);
        
        // Verify we got progress updates
        assert!(progress_count.load(Ordering::SeqCst) > 0);
    }
    
    #[tokio::test]
    async fn test_task_cancellation() {
        // Create test files
        let temp_file = NamedTempFile::new().unwrap();
        {
            let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
            file.write_all(b"test content").await.unwrap();
            file.flush().await.unwrap();
        }
        
        let files = vec![temp_file.path().to_path_buf()];
        
        let callback = Arc::new(|_progress: ProgressInfo| {});
        let task = HashingTask::new(files, 12, callback);
        let task_id = task.id;
        
        let processor = BackgroundProcessor::default();
        
        // Start the task
        processor.start_hashing_task(task).await.unwrap();
        
        // Cancel immediately
        processor.cancel_task(task_id).await.unwrap();
        
        // Wait a bit to ensure cancellation is processed
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Task should no longer be running
        assert!(!processor.is_task_running(task_id).await);
    }
    
    #[test]
    fn test_task_status_transitions() {
        let mut progress = ProgressInfo::new(Uuid::new_v4(), 1, 100);
        
        assert_eq!(progress.status, TaskStatus::Pending);
        
        progress.status = TaskStatus::Running;
        assert_eq!(progress.status, TaskStatus::Running);
        
        progress.mark_completed();
        assert_eq!(progress.status, TaskStatus::Completed);
        
        progress.mark_failed("Test error".to_string());
        assert_eq!(progress.status, TaskStatus::Failed);
        assert_eq!(progress.error, Some("Test error".to_string()));
        
        progress.mark_cancelled();
        assert_eq!(progress.status, TaskStatus::Cancelled);
    }
}