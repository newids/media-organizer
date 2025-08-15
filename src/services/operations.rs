use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::time::{SystemTime, Duration};
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use rand::Rng;

use super::file_system::{FileSystemService, FileSystemError};

/// Serialization module for SystemTime
mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration_since_epoch = time.duration_since(UNIX_EPOCH)
            .map_err(serde::ser::Error::custom)?;
        duration_since_epoch.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

/// Error severity levels for recovery classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - operation can be safely retried
    Low,
    /// Medium severity - operation may succeed with changes
    Medium,
    /// High severity - operation likely to fail repeatedly
    High,
    /// Critical severity - system-level issue, requires intervention
    Critical,
}

/// Error recovery classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// Retry immediately with same parameters
    RetryImmediate,
    /// Retry with exponential backoff delay
    RetryWithBackoff,
    /// Retry with modified parameters (e.g., smaller batch size)
    RetryWithModification,
    /// Skip this operation and continue with others
    Skip,
    /// Rollback and abort entire operation
    Abort,
    /// Requires manual intervention
    ManualIntervention,
}

/// Recovery suggestion for user feedback
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecoverySuggestion {
    /// Human-readable description of the problem
    pub description: String,
    /// Suggested action for the user
    pub suggestion: String,
    /// Whether this operation can be retried automatically
    pub can_retry: bool,
    /// Estimated time before retry (if applicable)
    pub retry_delay: Option<Duration>,
}

/// Errors that can occur during file operations
#[derive(Debug, Clone, Error)]
pub enum OperationError {
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    
    #[error("Command validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Command execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Undo operation failed: {0}")]
    UndoFailed(String),
    
    #[error("Operation already executed")]
    AlreadyExecuted,
    
    #[error("Cannot undo: operation not executed")]
    NotExecuted,
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Batch operation failed: {0}")]
    BatchFailed(String),
    
    #[error("Operation was cancelled")]
    Cancelled,
    
    #[error("Batch validation failed: {0}")]
    BatchValidationFailed(String),
    
    #[error("Rollback failed: {0}")]
    RollbackFailed(String),
    
    #[error("History error: {0}")]
    HistoryError(String),
    
    #[error("Cannot redo: no operations to redo")]
    NoRedoAvailable,
    
    #[error("Cannot undo: no operations to undo")]
    NoUndoAvailable,
    
    #[error("History limit exceeded: {0}")]
    HistoryLimitExceeded(usize),
    
    #[error("Progress tracking error: {0}")]
    ProgressError(String),
    
    #[error("Invalid progress callback")]
    InvalidCallback,
    
    // New error recovery related errors
    #[error("Transient error: {0}")]
    Transient(String),
    
    #[error("Resource unavailable: {0}")]
    ResourceUnavailable(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Insufficient disk space: {0}")]
    InsufficientSpace(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Recovery failed after {attempts} attempts: {message}")]
    RecoveryFailed { attempts: u32, message: String },
    
    #[error("Operation timeout after {duration:?}")]
    Timeout { duration: Duration },
}

/// Result type for operation commands
pub type OperationResult<T> = Result<T, OperationError>;

impl OperationError {
    /// Classify error for recovery planning
    pub fn classify(&self) -> (ErrorSeverity, RecoveryStrategy) {
        match self {
            // Transient errors - safe to retry
            Self::Transient(_) => (ErrorSeverity::Low, RecoveryStrategy::RetryWithBackoff),
            Self::Network(_) => (ErrorSeverity::Low, RecoveryStrategy::RetryWithBackoff),
            Self::Timeout { .. } => (ErrorSeverity::Low, RecoveryStrategy::RetryWithBackoff),
            Self::ResourceUnavailable(_) => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithBackoff),
            
            // File system errors - context dependent
            Self::FileSystem(fs_err) => Self::classify_filesystem_error(fs_err),
            
            // Permission and space issues - require intervention
            Self::PermissionDenied(_) => (ErrorSeverity::High, RecoveryStrategy::ManualIntervention),
            Self::InsufficientSpace(_) => (ErrorSeverity::High, RecoveryStrategy::ManualIntervention),
            
            // Validation errors - may be fixable
            Self::ValidationFailed(_) => (ErrorSeverity::Medium, RecoveryStrategy::Skip),
            Self::BatchValidationFailed(_) => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithModification),
            
            // Execution errors - depends on context
            Self::ExecutionFailed(_) => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithBackoff),
            Self::BatchFailed(_) => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithModification),
            
            // User actions - should not retry
            Self::Cancelled => (ErrorSeverity::Low, RecoveryStrategy::Abort),
            
            // System state errors - critical
            Self::AlreadyExecuted | Self::NotExecuted => (ErrorSeverity::Critical, RecoveryStrategy::Abort),
            
            // History and undo errors - operational issues
            Self::UndoFailed(_) => (ErrorSeverity::Medium, RecoveryStrategy::Skip),
            Self::RollbackFailed(_) => (ErrorSeverity::High, RecoveryStrategy::Abort),
            Self::NoUndoAvailable | Self::NoRedoAvailable => (ErrorSeverity::Low, RecoveryStrategy::Skip),
            
            // Recovery failures - critical
            Self::RecoveryFailed { .. } => (ErrorSeverity::Critical, RecoveryStrategy::Abort),
            
            // Other errors - default to medium severity
            _ => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithBackoff),
        }
    }
    
    /// Classify file system errors specifically
    fn classify_filesystem_error(fs_err: &FileSystemError) -> (ErrorSeverity, RecoveryStrategy) {
        use super::file_system::FileSystemError;
        
        match fs_err {
            FileSystemError::PathNotFound { .. } => (ErrorSeverity::Medium, RecoveryStrategy::Skip),
            FileSystemError::PermissionDenied { .. } => (ErrorSeverity::High, RecoveryStrategy::ManualIntervention),
            FileSystemError::FileAlreadyExists { .. } => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithModification),
            FileSystemError::InvalidPath { .. } => (ErrorSeverity::High, RecoveryStrategy::Skip),
            FileSystemError::Io(_) => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithBackoff),
            FileSystemError::DiskFull => (ErrorSeverity::High, RecoveryStrategy::ManualIntervention),
            FileSystemError::Cancelled => (ErrorSeverity::Low, RecoveryStrategy::Abort),
            FileSystemError::SymlinkLoop { .. } => (ErrorSeverity::High, RecoveryStrategy::Skip),
            FileSystemError::DirectoryNotEmpty { .. } => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithModification),
            FileSystemError::FileTooLarge { .. } => (ErrorSeverity::High, RecoveryStrategy::Skip),
            FileSystemError::NotSupported { .. } => (ErrorSeverity::High, RecoveryStrategy::Skip),
            FileSystemError::FileSystem { .. } => (ErrorSeverity::Medium, RecoveryStrategy::RetryWithBackoff),
        }
    }
    
    /// Get user-friendly recovery suggestion
    pub fn recovery_suggestion(&self) -> RecoverySuggestion {
        let (_severity, strategy) = self.classify();
        
        match (self, &strategy) {
            (Self::PermissionDenied(path), _) => RecoverySuggestion {
                description: format!("Access denied to '{}'", path),
                suggestion: "Check file permissions or run with administrator privileges".to_string(),
                can_retry: false,
                retry_delay: None,
            },
            
            (Self::InsufficientSpace(details), _) => RecoverySuggestion {
                description: format!("Not enough disk space: {}", details),
                suggestion: "Free up disk space and try again".to_string(),
                can_retry: true,
                retry_delay: None,
            },
            
            (Self::Network(details), &RecoveryStrategy::RetryWithBackoff) => RecoverySuggestion {
                description: format!("Network error: {}", details),
                suggestion: "Check network connection, will retry automatically".to_string(),
                can_retry: true,
                retry_delay: Some(Duration::from_secs(5)),
            },
            
            (Self::Transient(details), &RecoveryStrategy::RetryWithBackoff) => RecoverySuggestion {
                description: format!("Temporary error: {}", details),
                suggestion: "Will retry automatically in a moment".to_string(),
                can_retry: true,
                retry_delay: Some(Duration::from_secs(2)),
            },
            
            (Self::ResourceUnavailable(resource), &RecoveryStrategy::RetryWithBackoff) => RecoverySuggestion {
                description: format!("Resource temporarily unavailable: {}", resource),
                suggestion: "Will retry when resource becomes available".to_string(),
                can_retry: true,
                retry_delay: Some(Duration::from_secs(10)),
            },
            
            (Self::ValidationFailed(details), _) => RecoverySuggestion {
                description: format!("Invalid operation: {}", details),
                suggestion: "Check operation parameters and try again".to_string(),
                can_retry: false,
                retry_delay: None,
            },
            
            (Self::Timeout { duration }, &RecoveryStrategy::RetryWithBackoff) => RecoverySuggestion {
                description: format!("Operation timed out after {:?}", duration),
                suggestion: "Will retry with extended timeout".to_string(),
                can_retry: true,
                retry_delay: Some(Duration::from_secs(5)),
            },
            
            (Self::Cancelled, _) => RecoverySuggestion {
                description: "Operation was cancelled by user".to_string(),
                suggestion: "No action needed".to_string(),
                can_retry: false,
                retry_delay: None,
            },
            
            _ => RecoverySuggestion {
                description: format!("Operation failed: {}", self),
                suggestion: "Check the error details and try again".to_string(),
                can_retry: matches!(strategy, RecoveryStrategy::RetryWithBackoff | RecoveryStrategy::RetryImmediate),
                retry_delay: if matches!(strategy, RecoveryStrategy::RetryWithBackoff) {
                    Some(Duration::from_secs(3))
                } else {
                    None
                },
            },
        }
    }
    
    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        let (_, strategy) = self.classify();
        !matches!(strategy, RecoveryStrategy::Abort | RecoveryStrategy::ManualIntervention)
    }
    
    /// Get suggested retry delay based on error type
    pub fn retry_delay(&self) -> Option<Duration> {
        self.recovery_suggestion().retry_delay
    }
}

/// Configuration for retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add random jitter to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Calculate delay for given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay = self.initial_delay.as_millis() as f64 
            * self.backoff_multiplier.powi(attempt as i32);
        
        let delay_ms = base_delay.min(self.max_delay.as_millis() as f64);
        
        let final_delay = if self.jitter {
            // Add ±25% jitter
            let jitter_range = delay_ms * 0.25;
            let mut rng = rand::thread_rng();
            let jitter = (rng.gen::<f64>() - 0.5) * 2.0 * jitter_range;
            (delay_ms + jitter).max(0.0)
        } else {
            delay_ms
        };
        
        Duration::from_millis(final_delay as u64)
    }
}

/// Error recovery manager for handling retry logic and error classification
#[derive(Debug)]
pub struct ErrorRecoveryManager {
    config: RetryConfig,
    error_history: VecDeque<(SystemTime, OperationError)>,
    max_history_size: usize,
}

impl ErrorRecoveryManager {
    /// Create a new error recovery manager with default configuration
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
            error_history: VecDeque::new(),
            max_history_size: 100,
        }
    }
    
    /// Create a new error recovery manager with custom configuration
    pub fn with_config(config: RetryConfig) -> Self {
        Self {
            config,
            error_history: VecDeque::new(),
            max_history_size: 100,
        }
    }
    
    /// Record an error in the history with comprehensive logging
    pub fn record_error(&mut self, error: OperationError) {
        let timestamp = SystemTime::now();
        self.error_history.push_back((timestamp, error.clone()));
        
        // Maintain history size limit
        while self.error_history.len() > self.max_history_size {
            self.error_history.pop_front();
        }
        
        // Log error with structured context
        self.log_error_with_context(&error, timestamp);
    }
    
    /// Log error with comprehensive context information
    fn log_error_with_context(&self, error: &OperationError, timestamp: SystemTime) {
        let (severity, strategy) = error.classify();
        let suggestion = error.recovery_suggestion();
        
        // Create structured log entry
        tracing::error!(
            error = %error,
            error_type = ?error,
            severity = ?severity,
            recovery_strategy = ?strategy,
            can_retry = suggestion.can_retry,
            retry_delay_ms = suggestion.retry_delay.map(|d| d.as_millis() as u64),
            timestamp = ?timestamp,
            suggestion = %suggestion.suggestion,
            error_history_count = self.error_history.len(),
            "Error recorded in recovery manager"
        );
        
        // Log additional context based on error type
        match error {
            OperationError::FileSystem(fs_error) => {
                tracing::error!(
                    filesystem_error = %fs_error,
                    error_category = "filesystem",
                    "Filesystem operation failed"
                );
            },
            OperationError::ValidationFailed(details) => {
                tracing::error!(
                    validation_details = %details,
                    error_category = "validation",
                    "Operation validation failed"
                );
            },
            OperationError::BatchFailed(details) => {
                tracing::error!(
                    batch_details = %details,
                    error_category = "batch",
                    "Batch operation failed"
                );
            },
            OperationError::RecoveryFailed { attempts, message } => {
                tracing::error!(
                    recovery_attempts = attempts,
                    recovery_message = %message,
                    error_category = "recovery",
                    "Error recovery failed after multiple attempts"
                );
            },
            _ => {
                // Generic logging for other error types
                tracing::debug!(
                    error_category = "operation",
                    "Generic operation error recorded"
                );
            }
        }
    }
    
    /// Execute an operation with automatic retry logic
    pub async fn execute_with_retry<F, T>(&mut self, operation: F) -> OperationResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = OperationResult<T>> + Send>>,
    {
        let mut last_error = None;
        
        for attempt in 0..=self.config.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    self.record_error(error.clone());
                    last_error = Some(error.clone());
                    
                    // Check if error is recoverable
                    if !error.is_recoverable() {
                        tracing::warn!("Non-recoverable error, aborting: {}", error);
                        return Err(error);
                    }
                    
                    // Don't retry on last attempt
                    if attempt == self.config.max_attempts {
                        break;
                    }
                    
                    // Calculate delay and wait
                    let delay = self.config.delay_for_attempt(attempt);
                    tracing::info!(
                        "Operation failed (attempt {}/{}), retrying in {:?}: {}", 
                        attempt + 1, 
                        self.config.max_attempts + 1,
                        delay,
                        error
                    );
                    
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        // All retries exhausted
        let final_error = last_error.unwrap_or_else(|| {
            OperationError::RecoveryFailed {
                attempts: self.config.max_attempts,
                message: "Unknown error".to_string(),
            }
        });
        
        tracing::error!(
            "Operation failed after {} attempts: {}", 
            self.config.max_attempts + 1,
            final_error
        );
        
        Err(OperationError::RecoveryFailed {
            attempts: self.config.max_attempts + 1,
            message: final_error.to_string(),
        })
    }
    
    /// Get recent error statistics
    pub fn error_statistics(&self, since: SystemTime) -> ErrorStatistics {
        let recent_errors: Vec<_> = self.error_history
            .iter()
            .filter(|(time, _)| *time >= since)
            .collect();
        
        let total_errors = recent_errors.len();
        let mut by_severity = std::collections::HashMap::new();
        let mut recoverable_count = 0;
        
        for (_, error) in &recent_errors {
            let (severity, _) = error.classify();
            *by_severity.entry(severity).or_insert(0) += 1;
            
            if error.is_recoverable() {
                recoverable_count += 1;
            }
        }
        
        ErrorStatistics {
            total_errors,
            recoverable_errors: recoverable_count,
            non_recoverable_errors: total_errors - recoverable_count,
            by_severity,
            time_window: SystemTime::now().duration_since(since).unwrap_or_default(),
        }
    }
    
    /// Clear error history
    pub fn clear_history(&mut self) {
        self.error_history.clear();
    }
    
    /// Get configuration
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: RetryConfig) {
        self.config = config;
    }
}

impl Default for ErrorRecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about errors in a time window
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorStatistics {
    pub total_errors: usize,
    pub recoverable_errors: usize,
    pub non_recoverable_errors: usize,
    pub by_severity: std::collections::HashMap<ErrorSeverity, usize>,
    pub time_window: Duration,
}

/// Status of a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    /// Command created but not executed
    Pending,
    /// Command successfully executed
    Executed,
    /// Command execution failed
    Failed,
    /// Command was undone
    Undone,
}

/// Metadata about command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    pub id: Uuid,
    pub status: CommandStatus,
    pub created_at: SystemTime,
    pub executed_at: Option<SystemTime>,
    pub undone_at: Option<SystemTime>,
    pub error_message: Option<String>,
}

impl Default for CommandMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            status: CommandStatus::Pending,
            created_at: SystemTime::now(),
            executed_at: None,
            undone_at: None,
            error_message: None,
        }
    }
}

/// Progress tracking information for file operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressInfo {
    /// Current progress (completed items)
    pub current: u64,
    /// Total items to process
    pub total: u64,
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes to process
    pub total_bytes: u64,
    /// Current processing speed in bytes per second
    pub speed_bps: u64,
    /// Estimated time remaining in seconds
    pub eta_seconds: Option<u64>,
    /// Current operation description
    pub current_operation: String,
    /// Start time of the operation (as SystemTime for serialization)
    #[serde(with = "systemtime_serde")]
    pub started_at: SystemTime,
    /// Last update time (as SystemTime for serialization)
    #[serde(with = "systemtime_serde")]
    pub last_update: SystemTime,
}

impl ProgressInfo {
    /// Create new progress info
    pub fn new(total: u64, total_bytes: u64, operation: String) -> Self {
        let now = SystemTime::now();
        Self {
            current: 0,
            total,
            bytes_processed: 0,
            total_bytes,
            speed_bps: 0,
            eta_seconds: None,
            current_operation: operation,
            started_at: now,
            last_update: now,
        }
    }
    
    /// Update progress with current values
    pub fn update(&mut self, current: u64, bytes_processed: u64, current_operation: String) {
        let now = SystemTime::now();
        let elapsed = now.duration_since(self.started_at)
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f64();
        
        self.current = current;
        self.bytes_processed = bytes_processed;
        self.current_operation = current_operation;
        
        // Calculate speed (bytes per second)
        if elapsed > 0.0 {
            self.speed_bps = (self.bytes_processed as f64 / elapsed) as u64;
        }
        
        // Calculate ETA
        if self.speed_bps > 0 && self.bytes_processed < self.total_bytes {
            let remaining_bytes = self.total_bytes - self.bytes_processed;
            self.eta_seconds = Some(remaining_bytes / self.speed_bps);
        } else if self.current >= self.total {
            self.eta_seconds = Some(0);
        }
        
        self.last_update = now;
    }
    
    /// Get progress percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f64 / self.total as f64) * 100.0
        }
    }
    
    /// Get bytes progress percentage (0-100)
    pub fn bytes_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.bytes_processed as f64 / self.total_bytes as f64) * 100.0
        }
    }
    
    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        self.current >= self.total && self.bytes_processed >= self.total_bytes
    }
    
    /// Format speed as human-readable string
    pub fn format_speed(&self) -> String {
        format_bytes_per_second(self.speed_bps)
    }
    
    /// Format ETA as human-readable string
    pub fn format_eta(&self) -> String {
        match self.eta_seconds {
            Some(0) => "Complete".to_string(),
            Some(seconds) => format_duration(Duration::from_secs(seconds)),
            None => "Unknown".to_string(),
        }
    }
}

/// Cancellation token for graceful operation termination
#[derive(Debug, Clone)]
pub struct CancellationToken {
    /// Atomic flag indicating if cancellation was requested
    is_cancelled: Arc<AtomicBool>,
    /// Atomic counter for progress updates
    progress_counter: Arc<AtomicU64>,
}

impl CancellationToken {
    /// Create a new cancellation token
    pub fn new() -> Self {
        Self {
            is_cancelled: Arc::new(AtomicBool::new(false)),
            progress_counter: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Request cancellation of the operation
    pub fn cancel(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
    }
    
    /// Check if cancellation was requested
    pub fn is_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }
    
    /// Throw an error if cancellation was requested
    pub fn throw_if_cancelled(&self) -> OperationResult<()> {
        if self.is_cancelled() {
            Err(OperationError::Cancelled)
        } else {
            Ok(())
        }
    }
    
    /// Reset the cancellation token (for reuse)
    pub fn reset(&self) {
        self.is_cancelled.store(false, Ordering::SeqCst);
        self.progress_counter.store(0, Ordering::SeqCst);
    }
    
    /// Increment progress counter atomically
    pub fn increment_progress(&self) -> u64 {
        self.progress_counter.fetch_add(1, Ordering::SeqCst) + 1
    }
    
    /// Get current progress counter value
    pub fn progress_count(&self) -> u64 {
        self.progress_counter.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress callback function type
pub type ProgressCallback = Arc<dyn Fn(ProgressInfo) + Send + Sync>;

/// Progress tracker for file operations
pub struct ProgressTracker {
    /// Current progress information
    progress: ProgressInfo,
    /// Cancellation token
    cancellation_token: CancellationToken,
    /// Progress callback for UI updates
    callback: Option<ProgressCallback>,
    /// Whether to persist progress information
    persist_progress: bool,
}

impl std::fmt::Debug for ProgressTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgressTracker")
            .field("progress", &self.progress)
            .field("cancellation_token", &self.cancellation_token)
            .field("callback", &"<callback>")
            .field("persist_progress", &self.persist_progress)
            .finish()
    }
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(total: u64, total_bytes: u64, operation: String) -> Self {
        Self {
            progress: ProgressInfo::new(total, total_bytes, operation),
            cancellation_token: CancellationToken::new(),
            callback: None,
            persist_progress: false,
        }
    }
    
    /// Create a progress tracker with cancellation support
    pub fn with_cancellation(total: u64, total_bytes: u64, operation: String, cancellation_token: CancellationToken) -> Self {
        Self {
            progress: ProgressInfo::new(total, total_bytes, operation),
            cancellation_token,
            callback: None,
            persist_progress: false,
        }
    }
    
    /// Set progress callback for UI updates
    pub fn with_callback(mut self, callback: ProgressCallback) -> Self {
        self.callback = Some(callback);
        self
    }
    
    /// Enable progress persistence
    pub fn with_persistence(mut self, persist: bool) -> Self {
        self.persist_progress = persist;
        self
    }
    
    /// Update progress and trigger callback
    pub fn update(&mut self, current: u64, bytes_processed: u64, current_operation: String) -> OperationResult<()> {
        // Check for cancellation
        self.cancellation_token.throw_if_cancelled()?;
        
        // Update progress info
        self.progress.update(current, bytes_processed, current_operation);
        
        // Trigger callback if set
        if let Some(callback) = &self.callback {
            callback(self.progress.clone());
        }
        
        Ok(())
    }
    
    /// Increment progress by one item
    pub fn increment(&mut self, bytes_added: u64) -> OperationResult<()> {
        let new_current = self.progress.current + 1;
        let new_bytes = self.progress.bytes_processed + bytes_added;
        self.update(new_current, new_bytes, self.progress.current_operation.clone())
    }
    
    /// Mark operation as complete
    pub fn complete(&mut self) -> OperationResult<()> {
        self.update(
            self.progress.total,
            self.progress.total_bytes,
            "Complete".to_string(),
        )
    }
    
    /// Get current progress information
    pub fn progress(&self) -> &ProgressInfo {
        &self.progress
    }
    
    /// Get cancellation token
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }
    
    /// Check if operation was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancellation_token.is_cancelled()
    }
    
    /// Check if operation is complete
    pub fn is_complete(&self) -> bool {
        self.progress.is_complete()
    }
}

/// Helper function to format bytes per second as human-readable string
fn format_bytes_per_second(bps: u64) -> String {
    const UNITS: &[(&str, u64)] = &[
        ("B/s", 1),
        ("KB/s", 1024),
        ("MB/s", 1024 * 1024),
        ("GB/s", 1024 * 1024 * 1024),
    ];
    
    for &(unit, threshold) in UNITS.iter().rev() {
        if bps >= threshold {
            let value = bps as f64 / threshold as f64;
            return format!("{:.1} {}", value, unit);
        }
    }
    
    format!("{} B/s", bps)
}

/// Helper function to format duration as human-readable string
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Core Command trait for implementing the Command pattern
/// 
/// All file operations implement this trait to enable undo/redo functionality.
/// Commands store the necessary state to reverse their operations.
#[async_trait]
pub trait Command: Send + Sync + std::fmt::Debug {
    /// Execute the command
    /// 
    /// Returns Ok(()) if the command executed successfully.
    /// The command should store any necessary state for undo operations.
    async fn execute(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()>;
    
    /// Undo the command
    /// 
    /// Reverses the effects of execute(). Can only be called if execute() succeeded.
    /// Returns Ok(()) if the undo was successful.
    async fn undo(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()>;
    
    /// Validate the command before execution
    /// 
    /// Checks if the command can be executed (e.g., source files exist, 
    /// destination paths are valid, permissions are adequate).
    async fn validate(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<()>;
    
    /// Get command metadata
    fn metadata(&self) -> &CommandMetadata;
    
    /// Get mutable command metadata
    fn metadata_mut(&mut self) -> &mut CommandMetadata;
    
    /// Get a human-readable description of the command
    fn description(&self) -> String;
    
    /// Check if the command can be undone
    fn can_undo(&self) -> bool {
        matches!(self.metadata().status, CommandStatus::Executed)
    }
    
    /// Check if the command has been executed
    fn is_executed(&self) -> bool {
        matches!(self.metadata().status, CommandStatus::Executed)
    }
    
    /// Check if the command was undone
    fn is_undone(&self) -> bool {
        matches!(self.metadata().status, CommandStatus::Undone)
    }
    
    /// Execute the command with progress tracking
    /// 
    /// Default implementation calls execute() without progress tracking.
    /// Commands that support progress tracking should override this method.
    async fn execute_with_progress(
        &mut self, 
        fs: Arc<dyn FileSystemService>, 
        progress: Option<&mut ProgressTracker>
    ) -> OperationResult<()> {
        // Check for cancellation before starting
        if let Some(tracker) = progress {
            tracker.cancellation_token().throw_if_cancelled()?;
        }
        
        // Call the regular execute method
        self.execute(fs).await
    }
    
    /// Undo the command with progress tracking
    /// 
    /// Default implementation calls undo() without progress tracking.
    /// Commands that support progress tracking should override this method.
    async fn undo_with_progress(
        &mut self, 
        fs: Arc<dyn FileSystemService>, 
        progress: Option<&mut ProgressTracker>
    ) -> OperationResult<()> {
        // Check for cancellation before starting
        if let Some(tracker) = progress {
            tracker.cancellation_token().throw_if_cancelled()?;
        }
        
        // Call the regular undo method
        self.undo(fs).await
    }
    
    /// Estimate the operation size for progress tracking
    /// 
    /// Returns (item_count, total_bytes) for progress calculation.
    /// Default implementation returns (1, 0) indicating a single operation of unknown size.
    async fn estimate_work(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<(u64, u64)> {
        Ok((1, 0))
    }
    
    /// Check if this command supports progress tracking
    fn supports_progress(&self) -> bool {
        false
    }
}

/// Configuration for operation history management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Maximum number of operations to keep in history
    pub max_history_size: usize,
    /// Whether to persist history across application restarts
    pub persist_history: bool,
    /// Path to save history file (if persist_history is true)
    pub history_file_path: Option<PathBuf>,
    /// Memory limit for history in bytes (approximate)
    pub memory_limit_bytes: Option<usize>,
    /// Whether to compress old history entries
    pub compress_old_entries: bool,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            persist_history: true,
            history_file_path: None, // Will be set by service
            memory_limit_bytes: Some(100 * 1024 * 1024), // 100 MB
            compress_old_entries: true,
        }
    }
}

/// Represents a historical entry in the operation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: Uuid,
    pub command_id: Uuid,
    pub command_description: String,
    pub timestamp: SystemTime,
    pub command_size_bytes: usize,
    // We can't serialize the actual command due to trait objects,
    // but we store enough info to recreate simple operations if needed
}

impl HistoryEntry {
    pub fn new(command: &dyn Command) -> Self {
        Self {
            id: Uuid::new_v4(),
            command_id: command.metadata().id,
            command_description: command.description(),
            timestamp: SystemTime::now(),
            command_size_bytes: std::mem::size_of_val(command), // Approximate size
        }
    }
}

/// Operation history manager with undo/redo functionality
/// 
/// Manages a stack of executed commands that can be undone and redone.
/// Uses two stacks: undo_stack for executed commands, redo_stack for undone commands.
pub struct OperationHistory {
    /// Stack of commands that can be undone (most recent at end)
    undo_stack: VecDeque<Box<dyn Command>>,
    /// Stack of commands that can be redone (most recent at end)
    redo_stack: VecDeque<Box<dyn Command>>,
    /// Configuration for history management
    config: HistoryConfig,
    /// History entries for serialization and metadata
    history_entries: VecDeque<HistoryEntry>,
    /// Current memory usage estimate (bytes)
    current_memory_usage: usize,
    /// FileSystem service for operations
    fs_service: Arc<dyn FileSystemService>,
}

impl OperationHistory {
    /// Create a new operation history with default configuration
    pub fn new(fs_service: Arc<dyn FileSystemService>) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            config: HistoryConfig::default(),
            history_entries: VecDeque::new(),
            current_memory_usage: 0,
            fs_service,
        }
    }
    
    /// Create a new operation history with custom configuration
    pub fn with_config(fs_service: Arc<dyn FileSystemService>, config: HistoryConfig) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            config,
            history_entries: VecDeque::new(),
            current_memory_usage: 0,
            fs_service,
        }
    }
    
    /// Add a successfully executed command to the history
    /// This clears the redo stack since we're creating a new branch of history
    pub async fn add_executed_command(&mut self, command: Box<dyn Command>) -> OperationResult<()> {
        // Validate that the command is actually executed
        if !command.is_executed() {
            return Err(OperationError::HistoryError(
                "Cannot add non-executed command to history".to_string()
            ));
        }
        
        // Clear redo stack since we're creating a new branch
        self.redo_stack.clear();
        
        // Create history entry for metadata
        let entry = HistoryEntry::new(command.as_ref());
        self.history_entries.push_back(entry);
        
        // Update memory usage estimate
        let command_size = std::mem::size_of_val(command.as_ref());
        self.current_memory_usage += command_size;
        
        // Add to undo stack
        self.undo_stack.push_back(command);
        
        // Clean up if necessary
        self.cleanup_if_needed().await?;
        
        Ok(())
    }
    
    /// Undo the most recent operation
    pub async fn undo(&mut self) -> OperationResult<String> {
        if self.undo_stack.is_empty() {
            return Err(OperationError::NoUndoAvailable);
        }
        
        let mut command = self.undo_stack.pop_back()
            .ok_or(OperationError::NoUndoAvailable)?;
        
        // Attempt to undo the command
        let description = command.description();
        match command.undo(Arc::clone(&self.fs_service)).await {
            Ok(()) => {
                // Update command metadata
                command.metadata_mut().status = CommandStatus::Undone;
                command.metadata_mut().undone_at = Some(SystemTime::now());
                
                // Move to redo stack
                self.redo_stack.push_back(command);
                
                Ok(format!("Undone: {}", description))
            }
            Err(e) => {
                // Put command back on undo stack if undo failed
                self.undo_stack.push_back(command);
                Err(OperationError::UndoFailed(format!(
                    "Failed to undo operation '{}': {}", description, e
                )))
            }
        }
    }
    
    /// Redo the most recently undone operation
    pub async fn redo(&mut self) -> OperationResult<String> {
        if self.redo_stack.is_empty() {
            return Err(OperationError::NoRedoAvailable);
        }
        
        let mut command = self.redo_stack.pop_back()
            .ok_or(OperationError::NoRedoAvailable)?;
        
        // Attempt to re-execute the command
        let description = command.description();
        match command.execute(Arc::clone(&self.fs_service)).await {
            Ok(()) => {
                // Update command metadata
                command.metadata_mut().status = CommandStatus::Executed;
                command.metadata_mut().executed_at = Some(SystemTime::now());
                command.metadata_mut().undone_at = None;
                
                // Move back to undo stack
                self.undo_stack.push_back(command);
                
                Ok(format!("Redone: {}", description))
            }
            Err(e) => {
                // Put command back on redo stack if redo failed
                self.redo_stack.push_back(command);
                Err(OperationError::ExecutionFailed(format!(
                    "Failed to redo operation '{}': {}", description, e
                )))
            }
        }
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    /// Get the number of operations that can be undone
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }
    
    /// Get the number of operations that can be redone
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
    
    /// Get description of the next operation that would be undone
    pub fn next_undo_description(&self) -> Option<String> {
        self.undo_stack.back().map(|cmd| cmd.description())
    }
    
    /// Get description of the next operation that would be redone
    pub fn next_redo_description(&self) -> Option<String> {
        self.redo_stack.back().map(|cmd| cmd.description())
    }
    
    /// Clear all history (both undo and redo stacks)
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.history_entries.clear();
        self.current_memory_usage = 0;
    }
    
    /// Get current memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        self.current_memory_usage
    }
    
    /// Get current configuration
    pub fn config(&self) -> &HistoryConfig {
        &self.config
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: HistoryConfig) {
        self.config = config;
    }
    
    /// Clean up history if it exceeds configured limits
    async fn cleanup_if_needed(&mut self) -> OperationResult<()> {
        // Check memory limit
        if let Some(memory_limit) = self.config.memory_limit_bytes {
            if self.current_memory_usage > memory_limit {
                self.cleanup_by_memory().await?;
            }
        }
        
        // Check history size limit
        if self.undo_stack.len() > self.config.max_history_size {
            self.cleanup_by_size().await?;
        }
        
        Ok(())
    }
    
    /// Clean up oldest entries until under memory limit
    async fn cleanup_by_memory(&mut self) -> OperationResult<()> {
        let target_memory = self.config.memory_limit_bytes.unwrap_or(usize::MAX) * 9 / 10; // 90% of limit
        
        while self.current_memory_usage > target_memory && !self.undo_stack.is_empty() {
            if let Some(command) = self.undo_stack.pop_front() {
                let command_size = std::mem::size_of_val(command.as_ref());
                self.current_memory_usage = self.current_memory_usage.saturating_sub(command_size);
                
                // Also remove corresponding history entry
                if !self.history_entries.is_empty() {
                    self.history_entries.pop_front();
                }
                
                tracing::debug!("Removed oldest command from history to free memory: {}", command.description());
            }
        }
        
        Ok(())
    }
    
    /// Clean up oldest entries until under size limit
    async fn cleanup_by_size(&mut self) -> OperationResult<()> {
        let target_size = (self.config.max_history_size * 9) / 10; // 90% of limit
        
        while self.undo_stack.len() > target_size {
            if let Some(command) = self.undo_stack.pop_front() {
                let command_size = std::mem::size_of_val(command.as_ref());
                self.current_memory_usage = self.current_memory_usage.saturating_sub(command_size);
                
                // Also remove corresponding history entry
                if !self.history_entries.is_empty() {
                    self.history_entries.pop_front();
                }
                
                tracing::debug!("Removed oldest command from history due to size limit: {}", command.description());
            }
        }
        
        Ok(())
    }
    
    /// Get statistics about the current history
    pub fn get_stats(&self) -> HistoryStats {
        HistoryStats {
            undo_operations: self.undo_stack.len(),
            redo_operations: self.redo_stack.len(),
            total_operations: self.undo_stack.len() + self.redo_stack.len(),
            memory_usage_bytes: self.current_memory_usage,
            memory_limit_bytes: self.config.memory_limit_bytes,
            max_history_size: self.config.max_history_size,
            oldest_entry: self.history_entries.front().map(|e| e.timestamp),
            newest_entry: self.history_entries.back().map(|e| e.timestamp),
        }
    }
    
    /// Get list of recent operation descriptions (up to limit)
    pub fn get_recent_operations(&self, limit: usize) -> Vec<String> {
        self.undo_stack
            .iter()
            .rev() // Most recent first
            .take(limit)
            .map(|cmd| cmd.description())
            .collect()
    }
    
    /// Save history metadata to file (commands themselves cannot be serialized due to trait objects)
    pub async fn save_to_file(&self, path: &std::path::Path) -> OperationResult<()> {
        if !self.config.persist_history {
            return Err(OperationError::HistoryError(
                "History persistence is disabled".to_string()
            ));
        }
        
        let history_snapshot = HistorySnapshot {
            entries: self.history_entries.clone().into(),
            config: self.config.clone(),
            saved_at: SystemTime::now(),
            undo_count: self.undo_stack.len(),
            redo_count: self.redo_stack.len(),
        };
        
        let serialized = serde_json::to_string_pretty(&history_snapshot)
            .map_err(|e| OperationError::Serialization(format!("Failed to serialize history: {}", e)))?;
        
        tokio::fs::write(path, serialized)
            .await
            .map_err(|e| OperationError::HistoryError(format!("Failed to write history file: {}", e)))?;
        
        tracing::info!("Saved operation history to {}", path.display());
        Ok(())
    }
    
    /// Load history metadata from file (commands cannot be restored, only metadata)
    pub async fn load_from_file(&mut self, path: &std::path::Path) -> OperationResult<()> {
        if !self.config.persist_history {
            return Err(OperationError::HistoryError(
                "History persistence is disabled".to_string()
            ));
        }
        
        let contents = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| OperationError::HistoryError(format!("Failed to read history file: {}", e)))?;
        
        let history_snapshot: HistorySnapshot = serde_json::from_str(&contents)
            .map_err(|e| OperationError::Serialization(format!("Failed to deserialize history: {}", e)))?;
        
        // Restore metadata entries (note: actual commands cannot be restored)
        self.history_entries = VecDeque::from(history_snapshot.entries);
        self.config = history_snapshot.config;
        
        tracing::info!(
            "Loaded operation history metadata from {} ({} entries)", 
            path.display(), 
            self.history_entries.len()
        );
        tracing::warn!(
            "Note: Actual undo/redo functionality is not available after loading from file. \
            {} undo and {} redo operations were lost.", 
            history_snapshot.undo_count,
            history_snapshot.redo_count
        );
        
        Ok(())
    }
    
    /// Get default history file path based on configuration
    pub fn get_default_history_path(&self) -> std::path::PathBuf {
        if let Some(ref path) = self.config.history_file_path {
            path.clone()
        } else {
            // Default to user's data directory
            dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("media-organizer")
                .join("operation_history.json")
        }
    }
}

/// Serializable snapshot of operation history for persistence
#[derive(Debug, Serialize, Deserialize)]
pub struct HistorySnapshot {
    pub entries: Vec<HistoryEntry>,
    pub config: HistoryConfig,
    pub saved_at: SystemTime,
    pub undo_count: usize,
    pub redo_count: usize,
}

/// Statistics about operation history
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryStats {
    pub undo_operations: usize,
    pub redo_operations: usize,
    pub total_operations: usize,
    pub memory_usage_bytes: usize,
    pub memory_limit_bytes: Option<usize>,
    pub max_history_size: usize,
    pub oldest_entry: Option<SystemTime>,
    pub newest_entry: Option<SystemTime>,
}

/// Status of a batch operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Batch created but not started
    Pending,
    /// Batch validation in progress
    Validating,
    /// Batch execution in progress
    Executing,
    /// All commands in batch executed successfully
    Completed,
    /// Batch execution failed, rollback in progress
    RollingBack,
    /// Batch failed and was rolled back
    Failed,
    /// Batch was cancelled by user
    Cancelled,
}

/// Progress information for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total_commands: usize,
    pub completed_commands: usize,
    pub failed_commands: usize,
    pub current_command: Option<String>,
    pub status: BatchStatus,
    pub elapsed_time: Option<std::time::Duration>,
    pub estimated_remaining: Option<std::time::Duration>,
}

impl BatchProgress {
    pub fn new(total_commands: usize) -> Self {
        Self {
            total_commands,
            completed_commands: 0,
            failed_commands: 0,
            current_command: None,
            status: BatchStatus::Pending,
            elapsed_time: None,
            estimated_remaining: None,
        }
    }
    
    pub fn completion_percentage(&self) -> f32 {
        if self.total_commands == 0 {
            100.0
        } else {
            (self.completed_commands as f32 / self.total_commands as f32) * 100.0
        }
    }
}

/// Represents a batch of file operations that can be executed atomically
#[derive(Debug)]
pub struct BatchOperation {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub commands: Vec<Box<dyn Command>>,
    pub metadata: CommandMetadata,
    pub progress: BatchProgress,
    pub allow_partial_failure: bool,
    pub max_retries: u32,
    
    // State for rollback
    executed_commands: Vec<usize>, // Indices of successfully executed commands
    
    // Cancellation support
    cancel_token: Option<tokio_util::sync::CancellationToken>,
}

impl BatchOperation {
    /// Create a new batch operation
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            commands: Vec::new(),
            metadata: CommandMetadata::default(),
            progress: BatchProgress::new(0),
            allow_partial_failure: false,
            max_retries: 0,
            executed_commands: Vec::new(),
            cancel_token: Some(tokio_util::sync::CancellationToken::new()),
        }
    }
    
    /// Add a command to the batch
    pub fn add_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
        self.progress.total_commands = self.commands.len();
    }
    
    /// Add multiple commands to the batch
    pub fn add_commands(&mut self, commands: Vec<Box<dyn Command>>) {
        self.commands.extend(commands);
        self.progress.total_commands = self.commands.len();
    }
    
    /// Set whether the batch allows partial failure
    pub fn with_partial_failure(mut self, allow: bool) -> Self {
        self.allow_partial_failure = allow;
        self
    }
    
    /// Set maximum retry attempts for failed commands
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Get cancellation token for this batch
    pub fn cancellation_token(&self) -> Option<tokio_util::sync::CancellationToken> {
        self.cancel_token.as_ref().map(|token| token.clone())
    }
    
    /// Cancel the batch operation
    pub fn cancel(&self) {
        if let Some(token) = &self.cancel_token {
            token.cancel();
        }
    }
    
    /// Check if the batch has been cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.as_ref()
            .map(|token| token.is_cancelled())
            .unwrap_or(false)
    }
}

/// Message types for batch operation queue
#[derive(Debug)]
pub enum BatchMessage {
    Execute {
        batch: BatchOperation,
        response: oneshot::Sender<OperationResult<BatchOperation>>,
    },
    GetProgress {
        batch_id: Uuid,
        response: oneshot::Sender<Option<BatchProgress>>,
    },
    Cancel {
        batch_id: Uuid,
        response: oneshot::Sender<bool>,
    },
    Shutdown,
}

/// Batch operation processor that handles queued operations in the background
pub struct BatchProcessor {
    sender: mpsc::UnboundedSender<BatchMessage>,
    active_batches: Arc<tokio::sync::RwLock<HashMap<Uuid, BatchProgress>>>,
}

impl BatchProcessor {
    /// Create a new batch processor
    pub fn new(fs: Arc<dyn FileSystemService>) -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel();
        let active_batches = Arc::new(tokio::sync::RwLock::new(HashMap::new()));
        let active_batches_clone = active_batches.clone();
        
        // Spawn background task to process batch operations
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    BatchMessage::Execute { mut batch, response } => {
                        let batch_id = batch.id;
                        
                        // Add to active batches
                        active_batches_clone.write().await.insert(batch_id, batch.progress.clone());
                        
                        // Execute the batch
                        let result = Self::execute_batch(&mut batch, fs.clone(), active_batches_clone.clone()).await;
                        
                        // Remove from active batches
                        active_batches_clone.write().await.remove(&batch_id);
                        
                        let _ = response.send(result);
                    },
                    BatchMessage::GetProgress { batch_id, response } => {
                        let progress = active_batches_clone.read().await.get(&batch_id).cloned();
                        let _ = response.send(progress);
                    },
                    BatchMessage::Cancel { batch_id, response } => {
                        let mut batches = active_batches_clone.write().await;
                        let cancelled = if let Some(progress) = batches.get_mut(&batch_id) {
                            progress.status = BatchStatus::Cancelled;
                            true
                        } else {
                            false
                        };
                        let _ = response.send(cancelled);
                    },
                    BatchMessage::Shutdown => {
                        tracing::info!("Batch processor shutting down");
                        break;
                    }
                }
            }
        });
        
        Self {
            sender,
            active_batches,
        }
    }
    
    /// Execute a batch operation asynchronously
    pub async fn execute_batch_async(&self, batch: BatchOperation) -> OperationResult<BatchOperation> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.sender.send(BatchMessage::Execute {
            batch,
            response: response_tx,
        }).map_err(|_| OperationError::BatchFailed("Failed to queue batch operation".to_string()))?;
        
        response_rx.await
            .map_err(|_| OperationError::BatchFailed("Failed to receive batch result".to_string()))?
    }
    
    /// Get progress for a specific batch
    pub async fn get_progress(&self, batch_id: Uuid) -> Option<BatchProgress> {
        let (response_tx, response_rx) = oneshot::channel();
        
        if self.sender.send(BatchMessage::GetProgress {
            batch_id,
            response: response_tx,
        }).is_err() {
            return None;
        }
        
        response_rx.await.unwrap_or(None)
    }
    
    /// Cancel a specific batch operation
    pub async fn cancel_batch(&self, batch_id: Uuid) -> bool {
        let (response_tx, response_rx) = oneshot::channel();
        
        if self.sender.send(BatchMessage::Cancel {
            batch_id,
            response: response_tx,
        }).is_err() {
            return false;
        }
        
        response_rx.await.unwrap_or(false)
    }
    
    /// Shutdown the batch processor
    pub fn shutdown(&self) {
        let _ = self.sender.send(BatchMessage::Shutdown);
    }
}

impl BatchProcessor {
    /// Execute a batch operation with proper error handling and rollback
    async fn execute_batch(
        batch: &mut BatchOperation,
        fs: Arc<dyn FileSystemService>,
        active_batches: Arc<tokio::sync::RwLock<HashMap<Uuid, BatchProgress>>>,
    ) -> OperationResult<BatchOperation> {
        let start_time = SystemTime::now();
        batch.progress.status = BatchStatus::Validating;
        
        // Update progress
        Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
        
        // Check for cancellation
        if batch.is_cancelled() {
            batch.progress.status = BatchStatus::Cancelled;
            Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
            return Err(OperationError::Cancelled);
        }
        
        // Validate all commands first
        for (i, command) in batch.commands.iter().enumerate() {
            batch.progress.current_command = Some(format!("Validating {}", command.description()));
            Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
            
            if let Err(e) = command.validate(fs.clone()).await {
                batch.progress.status = BatchStatus::Failed;
                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                return Err(OperationError::BatchValidationFailed(
                    format!("Command {} failed validation: {}", i, e)
                ));
            }
            
            // Check for cancellation during validation
            if batch.is_cancelled() {
                batch.progress.status = BatchStatus::Cancelled;
                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                return Err(OperationError::Cancelled);
            }
        }
        
        // Start execution phase
        batch.progress.status = BatchStatus::Executing;
        batch.metadata.status = CommandStatus::Executed;
        batch.metadata.executed_at = Some(start_time);
        
        let mut execution_errors = Vec::new();
        
        // Execute commands in order  
        let total_commands = batch.commands.len(); // Store this before the loop
        for (i, command) in batch.commands.iter_mut().enumerate() {
            batch.progress.current_command = Some(command.description());
            Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
            
            // Check for cancellation before each command using cancel token
            if let Some(token) = &batch.cancel_token {
                if token.is_cancelled() {
                    batch.progress.status = BatchStatus::Cancelled;
                    Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                    
                    // Rollback executed commands
                    Self::rollback_executed_commands(batch, fs.clone()).await?;
                    return Err(OperationError::Cancelled);
                }
            }
            
            // Execute command with enhanced error handling
            let mut attempts = 0;
            let max_attempts = batch.max_retries.max(3); // Ensure at least some retries
            let mut success = false;
            let mut last_error = None;
            
            while attempts <= max_attempts && !success {
                match command.execute(fs.clone()).await {
                    Ok(_) => {
                        batch.executed_commands.push(i);
                        batch.progress.completed_commands += 1;
                        success = true;
                        
                        // Structured logging for successful command execution
                        tracing::debug!(
                            command_index = i,
                            command_desc = command.description(),
                            attempt = attempts + 1,
                            batch_id = %batch.id,
                            batch_name = %batch.name,
                            completed_commands = batch.progress.completed_commands,
                            total_commands = total_commands,
                            completion_percentage = batch.progress.completion_percentage(),
                            "Command executed successfully in batch operation"
                        );
                    },
                    Err(e) => {
                        attempts += 1;
                        last_error = Some(e.clone());
                        
                        // Check error classification for recovery strategy
                        let (severity, strategy) = e.classify();
                        let suggestion = e.recovery_suggestion();
                        
                        let error_msg = format!("Command {} failed (attempt {}/{}): {}", i, attempts, max_attempts + 1, e);
                        
                        // Structured logging for command failure
                        tracing::warn!(
                            command_index = i,
                            attempt = attempts,
                            max_attempts = max_attempts + 1,
                            command_desc = command.description(),
                            error = %e,
                            error_type = ?e,
                            severity = ?severity,
                            recovery_strategy = ?strategy,
                            batch_id = %batch.id,
                            batch_name = %batch.name,
                            "Command execution failed during batch operation"
                        );
                        
                        tracing::info!(
                            command_index = i,
                            suggestion = %suggestion.suggestion,
                            can_retry = suggestion.can_retry,
                            retry_delay_ms = suggestion.retry_delay.map(|d| d.as_millis() as u64),
                            batch_id = %batch.id,
                            "Recovery suggestion for failed command"
                        );
                        
                        if attempts > max_attempts {
                            // Max retries reached - apply recovery strategy
                            execution_errors.push((i, error_msg.clone()));
                            batch.progress.failed_commands += 1;
                            break;
                        }
                        
                        // Check if we should continue retrying based on strategy
                        if matches!(strategy, RecoveryStrategy::Abort | RecoveryStrategy::ManualIntervention) {
                            tracing::warn!("Non-recoverable error detected, stopping retries: {}", e);
                            execution_errors.push((i, error_msg.clone()));
                            batch.progress.failed_commands += 1;
                            break;
                        }
                        
                        // Calculate retry delay based on strategy and attempt
                        let delay = match strategy {
                            RecoveryStrategy::RetryImmediate => Duration::from_millis(10),
                            RecoveryStrategy::RetryWithBackoff => {
                                // Exponential backoff
                                let base_delay = suggestion.retry_delay.unwrap_or(Duration::from_secs(1));
                                Duration::from_millis(
                                    (base_delay.as_millis() as u64) * (2_u64.pow(attempts.min(6)))
                                )
                            },
                            _ => Duration::from_millis(100 * attempts as u64),
                        };
                        
                        // Structured logging for retry attempt
                        tracing::info!(
                            command_index = i,
                            retry_delay_ms = delay.as_millis() as u64,
                            retry_strategy = ?strategy,
                            next_attempt = attempts + 1,
                            max_attempts = max_attempts + 1,
                            batch_id = %batch.id,
                            "Retrying command execution with backoff delay"
                        );
                        
                        tokio::time::sleep(delay).await;
                    }
                }
            }
            
            // Handle final error state if not successful
            if !success {
                if let Some(e) = last_error {
                    let error_msg = format!("Command {} failed after recovery attempts: {}", i, e);
                    tracing::error!("{}", error_msg);
                    
                    // Check error severity and apply appropriate recovery strategy
                    let (severity, strategy) = e.classify();
                    let suggestion = e.recovery_suggestion();
                    
                    tracing::info!("Error recovery suggestion: {}", suggestion.suggestion);
                    
                    match strategy {
                        RecoveryStrategy::Abort | RecoveryStrategy::ManualIntervention => {
                            // Critical error - stop batch execution
                            if !batch.allow_partial_failure {
                                batch.progress.status = BatchStatus::RollingBack;
                                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                
                                if let Err(rollback_err) = Self::rollback_executed_commands(batch, fs.clone()).await {
                                    batch.progress.status = BatchStatus::Failed;
                                    Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                    return Err(OperationError::RollbackFailed(
                                        format!("Critical error and rollback failed: {} | Rollback error: {}", error_msg, rollback_err)
                                    ));
                                }
                                
                                batch.progress.status = BatchStatus::Failed;
                                batch.metadata.status = CommandStatus::Failed;
                                batch.metadata.error_message = Some(error_msg.clone());
                                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                
                                return Err(OperationError::BatchFailed(error_msg));
                            } else {
                                // Log critical error but continue with partial failure
                                tracing::warn!("Critical error in command {}, continuing due to partial failure mode: {}", i, e);
                            }
                        },
                        RecoveryStrategy::Skip => {
                            // Skip this command and continue
                            tracing::info!("Skipping command {} due to recoverable error: {}", i, e);
                        },
                        _ => {
                            // For other strategies, check if partial failure is allowed
                            if !batch.allow_partial_failure {
                                batch.progress.status = BatchStatus::RollingBack;
                                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                
                                if let Err(rollback_err) = Self::rollback_executed_commands(batch, fs.clone()).await {
                                    batch.progress.status = BatchStatus::Failed;
                                    Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                    return Err(OperationError::RollbackFailed(
                                        format!("Execution failed and rollback failed: {} | Rollback error: {}", error_msg, rollback_err)
                                    ));
                                }
                                
                                batch.progress.status = BatchStatus::Failed;
                                batch.metadata.status = CommandStatus::Failed;
                                batch.metadata.error_message = Some(error_msg.clone());
                                Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
                                
                                return Err(OperationError::BatchFailed(error_msg));
                            }
                        }
                    }
                }
            }
            
            Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
        }
        
        // Mark batch as completed
        batch.progress.status = BatchStatus::Completed;
        batch.progress.elapsed_time = start_time.elapsed().ok();
        batch.progress.current_command = None;
        Self::update_progress(batch.id, &batch.progress, active_batches.clone()).await;
        
        // Log execution summary
        if execution_errors.is_empty() {
            tracing::info!("Batch '{}' completed successfully with {} commands", 
                batch.name, batch.commands.len());
        } else {
            tracing::warn!("Batch '{}' completed with {} errors (partial failure allowed): {:?}", 
                batch.name, execution_errors.len(), execution_errors);
        }
        
        // Return owned BatchOperation (can't clone due to trait objects)
        // Instead, create a simple result status
        Ok(BatchOperation {
            id: batch.id,
            name: batch.name.clone(),
            description: batch.description.clone(),
            commands: Vec::new(), // Commands can't be cloned due to trait objects
            metadata: batch.metadata.clone(),
            progress: batch.progress.clone(),
            allow_partial_failure: batch.allow_partial_failure,
            max_retries: batch.max_retries,
            executed_commands: batch.executed_commands.clone(),
            cancel_token: None, // Reset cancellation token
        })
    }
    
    /// Rollback all executed commands in reverse order
    async fn rollback_executed_commands(
        batch: &mut BatchOperation,
        fs: Arc<dyn FileSystemService>,
    ) -> OperationResult<()> {
        let mut rollback_errors = Vec::new();
        
        // Rollback in reverse order
        for &command_index in batch.executed_commands.iter().rev() {
            if let Some(command) = batch.commands.get_mut(command_index) {
                if let Err(e) = command.undo(fs.clone()).await {
                    let error_msg = format!("Failed to rollback command {}: {}", command_index, e);
                    tracing::error!("{}", error_msg);
                    rollback_errors.push(error_msg);
                }
            }
        }
        
        if rollback_errors.is_empty() {
            tracing::info!("Successfully rolled back {} commands", batch.executed_commands.len());
            Ok(())
        } else {
            Err(OperationError::RollbackFailed(
                format!("Rollback completed with errors: {:?}", rollback_errors)
            ))
        }
    }
    
    /// Update progress in the active batches map
    async fn update_progress(
        batch_id: Uuid,
        progress: &BatchProgress,
        active_batches: Arc<tokio::sync::RwLock<HashMap<Uuid, BatchProgress>>>,
    ) {
        active_batches.write().await.insert(batch_id, progress.clone());
    }
}

/// Copy file command
/// 
/// Copies a file from source to destination. 
/// Undo operation removes the destination file if it was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyCommand {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub overwrite: bool,
    pub preserve_metadata: bool,
    
    // State for undo
    destination_existed_before: Option<bool>,
    original_destination_backup: Option<Vec<u8>>,
    
    metadata: CommandMetadata,
}

impl CopyCommand {
    /// Create a new copy command
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            overwrite: false,
            preserve_metadata: true,
            destination_existed_before: None,
            original_destination_backup: None,
            metadata: CommandMetadata::default(),
        }
    }
    
    /// Set whether to overwrite existing files
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
    
    /// Set whether to preserve file metadata (timestamps, permissions)
    pub fn with_preserve_metadata(mut self, preserve: bool) -> Self {
        self.preserve_metadata = preserve;
        self
    }
}

#[async_trait]
impl Command for CopyCommand {
    async fn execute(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if self.is_executed() {
            return Err(OperationError::AlreadyExecuted);
        }
        
        // Validate first
        self.validate(fs.clone()).await?;
        
        // Check if destination exists and backup if needed for undo
        self.destination_existed_before = Some(self.destination.exists());
        
        if *self.destination_existed_before.as_ref().unwrap() && !self.overwrite {
            return Err(OperationError::ValidationFailed(
                format!("Destination already exists: {}", self.destination.display())
            ));
        }
        
        // Backup original destination content if overwriting
        if *self.destination_existed_before.as_ref().unwrap() && self.overwrite {
            if let Ok(content) = tokio::fs::read(&self.destination).await {
                self.original_destination_backup = Some(content);
            }
        }
        
        // Execute the copy using file system service
        let operation = super::file_system::FileOperation {
            source: self.source.clone(),
            destination: self.destination.clone(),
            overwrite_mode: if self.overwrite {
                super::file_system::OverwriteMode::Overwrite
            } else {
                super::file_system::OverwriteMode::Fail
            },
            preserve_metadata: self.preserve_metadata,
        };
        
        fs.copy_file(operation).await.map_err(OperationError::FileSystem)?;
        
        // Update metadata
        self.metadata.status = CommandStatus::Executed;
        self.metadata.executed_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn undo(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if !self.can_undo() {
            return Err(OperationError::NotExecuted);
        }
        
        // Remove the copied file if destination didn't exist before
        if let Some(false) = self.destination_existed_before {
            fs.delete_file(&self.destination).await
                .map_err(|e| OperationError::UndoFailed(format!("Failed to remove copied file: {}", e)))?;
        }
        // Restore original file if we overwrote it
        else if let Some(backup) = &self.original_destination_backup {
            tokio::fs::write(&self.destination, backup).await
                .map_err(|e| OperationError::UndoFailed(format!("Failed to restore original file: {}", e)))?;
        }
        
        // Update metadata
        self.metadata.status = CommandStatus::Undone;
        self.metadata.undone_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn validate(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        // Check source exists
        if !self.source.exists() {
            return Err(OperationError::ValidationFailed(
                format!("Source file does not exist: {}", self.source.display())
            ));
        }
        
        // Check source is readable
        if !fs.check_read_permission(&self.source).await.unwrap_or(false) {
            return Err(OperationError::ValidationFailed(
                format!("No read permission for source: {}", self.source.display())
            ));
        }
        
        // Check destination directory exists
        if let Some(parent) = self.destination.parent() {
            if !parent.exists() {
                return Err(OperationError::ValidationFailed(
                    format!("Destination directory does not exist: {}", parent.display())
                ));
            }
            
            // Check write permission for destination directory
            if !fs.check_write_permission(parent).await.unwrap_or(false) {
                return Err(OperationError::ValidationFailed(
                    format!("No write permission for destination directory: {}", parent.display())
                ));
            }
        }
        
        // If destination exists and overwrite is false, fail validation
        if self.destination.exists() && !self.overwrite {
            return Err(OperationError::ValidationFailed(
                format!("Destination already exists and overwrite is disabled: {}", self.destination.display())
            ));
        }
        
        Ok(())
    }
    
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut CommandMetadata {
        &mut self.metadata
    }
    
    fn description(&self) -> String {
        format!("Copy {} to {}", 
            self.source.display(), 
            self.destination.display())
    }
}

/// Move file command
/// 
/// Moves a file from source to destination.
/// Undo operation moves the file back to the original location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveCommand {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub overwrite: bool,
    
    // State for undo
    destination_existed_before: Option<bool>,
    original_destination_backup: Option<Vec<u8>>,
    
    metadata: CommandMetadata,
}

impl MoveCommand {
    /// Create a new move command
    pub fn new(source: PathBuf, destination: PathBuf) -> Self {
        Self {
            source,
            destination,
            overwrite: false,
            destination_existed_before: None,
            original_destination_backup: None,
            metadata: CommandMetadata::default(),
        }
    }
    
    /// Set whether to overwrite existing files
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

#[async_trait]
impl Command for MoveCommand {
    async fn execute(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if self.is_executed() {
            return Err(OperationError::AlreadyExecuted);
        }
        
        // Validate first
        self.validate(fs.clone()).await?;
        
        // Check if destination exists and backup if needed
        self.destination_existed_before = Some(self.destination.exists());
        
        if *self.destination_existed_before.as_ref().unwrap() && self.overwrite {
            if let Ok(content) = tokio::fs::read(&self.destination).await {
                self.original_destination_backup = Some(content);
            }
        }
        
        // Execute the move using file system service
        let operation = super::file_system::FileOperation {
            source: self.source.clone(),
            destination: self.destination.clone(),
            overwrite_mode: if self.overwrite {
                super::file_system::OverwriteMode::Overwrite
            } else {
                super::file_system::OverwriteMode::Fail
            },
            preserve_metadata: true,
        };
        
        fs.move_file(operation).await.map_err(OperationError::FileSystem)?;
        
        // Update metadata
        self.metadata.status = CommandStatus::Executed;
        self.metadata.executed_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn undo(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if !self.can_undo() {
            return Err(OperationError::NotExecuted);
        }
        
        // Move the file back to its original location
        let operation = super::file_system::FileOperation {
            source: self.destination.clone(),
            destination: self.source.clone(),
            overwrite_mode: super::file_system::OverwriteMode::Overwrite,
            preserve_metadata: true,
        };
        
        fs.move_file(operation).await
            .map_err(|e| OperationError::UndoFailed(format!("Failed to move file back: {}", e)))?;
        
        // Restore original destination if it existed before
        if let Some(backup) = &self.original_destination_backup {
            tokio::fs::write(&self.destination, backup).await
                .map_err(|e| OperationError::UndoFailed(format!("Failed to restore original destination: {}", e)))?;
        }
        
        // Update metadata
        self.metadata.status = CommandStatus::Undone;
        self.metadata.undone_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn validate(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        // Check source exists
        if !self.source.exists() {
            return Err(OperationError::ValidationFailed(
                format!("Source file does not exist: {}", self.source.display())
            ));
        }
        
        // Check source permissions
        if !fs.check_read_permission(&self.source).await.unwrap_or(false) {
            return Err(OperationError::ValidationFailed(
                format!("No read permission for source: {}", self.source.display())
            ));
        }
        
        if let Some(source_parent) = self.source.parent() {
            if !fs.check_write_permission(source_parent).await.unwrap_or(false) {
                return Err(OperationError::ValidationFailed(
                    format!("No write permission for source directory: {}", source_parent.display())
                ));
            }
        }
        
        // Check destination directory
        if let Some(parent) = self.destination.parent() {
            if !parent.exists() {
                return Err(OperationError::ValidationFailed(
                    format!("Destination directory does not exist: {}", parent.display())
                ));
            }
            
            if !fs.check_write_permission(parent).await.unwrap_or(false) {
                return Err(OperationError::ValidationFailed(
                    format!("No write permission for destination directory: {}", parent.display())
                ));
            }
        }
        
        // Check destination overwrite
        if self.destination.exists() && !self.overwrite {
            return Err(OperationError::ValidationFailed(
                format!("Destination already exists and overwrite is disabled: {}", self.destination.display())
            ));
        }
        
        Ok(())
    }
    
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut CommandMetadata {
        &mut self.metadata
    }
    
    fn description(&self) -> String {
        format!("Move {} to {}", 
            self.source.display(), 
            self.destination.display())
    }
}

/// Delete file command
/// 
/// Deletes a file. Undo operation restores the file from backup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteCommand {
    pub path: PathBuf,
    
    // State for undo - store the entire file content
    file_backup: Option<Vec<u8>>,
    // Note: We skip std::fs::Metadata for serialization since it's not serializable
    #[serde(skip)]
    file_metadata_backup: Option<std::fs::Metadata>,
    
    metadata: CommandMetadata,
}

impl DeleteCommand {
    /// Create a new delete command
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            file_backup: None,
            file_metadata_backup: None,
            metadata: CommandMetadata::default(),
        }
    }
}

#[async_trait]
impl Command for DeleteCommand {
    async fn execute(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if self.is_executed() {
            return Err(OperationError::AlreadyExecuted);
        }
        
        // Validate first
        self.validate(fs.clone()).await?;
        
        // Backup file content and metadata for undo
        self.file_backup = Some(
            tokio::fs::read(&self.path).await
                .map_err(|e| OperationError::ExecutionFailed(format!("Failed to read file for backup: {}", e)))?
        );
        
        self.file_metadata_backup = std::fs::metadata(&self.path).ok();
        
        // Execute the delete
        fs.delete_file(&self.path).await.map_err(OperationError::FileSystem)?;
        
        // Update metadata
        self.metadata.status = CommandStatus::Executed;
        self.metadata.executed_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn undo(&mut self, _fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if !self.can_undo() {
            return Err(OperationError::NotExecuted);
        }
        
        // Restore file content
        if let Some(content) = &self.file_backup {
            tokio::fs::write(&self.path, content).await
                .map_err(|e| OperationError::UndoFailed(format!("Failed to restore file: {}", e)))?;
            
            // Try to restore original metadata
            if let Some(_original_metadata) = &self.file_metadata_backup {
                // Note: Restoring full metadata (permissions, timestamps) is complex
                // and platform-specific. For now, we just restore the file content.
                // In a production system, this would need platform-specific implementations.
            }
        } else {
            return Err(OperationError::UndoFailed("No backup available for restore".to_string()));
        }
        
        // Update metadata
        self.metadata.status = CommandStatus::Undone;
        self.metadata.undone_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn validate(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        // Check file exists
        if !self.path.exists() {
            return Err(OperationError::ValidationFailed(
                format!("File does not exist: {}", self.path.display())
            ));
        }
        
        // Check it's a file, not a directory
        if self.path.is_dir() {
            return Err(OperationError::ValidationFailed(
                format!("Path is a directory, not a file: {}", self.path.display())
            ));
        }
        
        // Check write permission for parent directory (needed to delete)
        if let Some(parent) = self.path.parent() {
            if !fs.check_write_permission(parent).await.unwrap_or(false) {
                return Err(OperationError::ValidationFailed(
                    format!("No write permission for parent directory: {}", parent.display())
                ));
            }
        }
        
        Ok(())
    }
    
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut CommandMetadata {
        &mut self.metadata
    }
    
    fn description(&self) -> String {
        format!("Delete {}", self.path.display())
    }
}

/// Rename file command
/// 
/// Renames a file. Undo operation renames it back to the original name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameCommand {
    pub old_path: PathBuf,
    pub new_name: String,
    
    // Computed new path
    pub new_path: PathBuf,
    
    // State for undo
    new_path_existed_before: Option<bool>,
    original_new_path_backup: Option<Vec<u8>>,
    
    metadata: CommandMetadata,
}

impl RenameCommand {
    /// Create a new rename command
    pub fn new(old_path: PathBuf, new_name: String) -> OperationResult<Self> {
        let new_path = if let Some(parent) = old_path.parent() {
            parent.join(&new_name)
        } else {
            PathBuf::from(&new_name)
        };
        
        Ok(Self {
            old_path,
            new_name,
            new_path,
            new_path_existed_before: None,
            original_new_path_backup: None,
            metadata: CommandMetadata::default(),
        })
    }
}

#[async_trait]
impl Command for RenameCommand {
    async fn execute(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if self.is_executed() {
            return Err(OperationError::AlreadyExecuted);
        }
        
        // Validate first
        self.validate(fs.clone()).await?;
        
        // Check if new path exists and backup if needed
        self.new_path_existed_before = Some(self.new_path.exists());
        
        if *self.new_path_existed_before.as_ref().unwrap() {
            if let Ok(content) = tokio::fs::read(&self.new_path).await {
                self.original_new_path_backup = Some(content);
            }
        }
        
        // Execute the rename using file system service
        fs.rename_file(&self.old_path, &self.new_name).await
            .map_err(OperationError::FileSystem)?;
        
        // Update metadata
        self.metadata.status = CommandStatus::Executed;
        self.metadata.executed_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn undo(&mut self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        if !self.can_undo() {
            return Err(OperationError::NotExecuted);
        }
        
        // Get the original name
        let original_name = self.old_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| OperationError::UndoFailed("Cannot get original filename".to_string()))?;
        
        // Rename back to original
        fs.rename_file(&self.new_path, original_name).await
            .map_err(|e| OperationError::UndoFailed(format!("Failed to rename back: {}", e)))?;
        
        // Restore original file at new path if it existed
        if let Some(backup) = &self.original_new_path_backup {
            tokio::fs::write(&self.new_path, backup).await
                .map_err(|e| OperationError::UndoFailed(format!("Failed to restore original file: {}", e)))?;
        }
        
        // Update metadata
        self.metadata.status = CommandStatus::Undone;
        self.metadata.undone_at = Some(SystemTime::now());
        
        Ok(())
    }
    
    async fn validate(&self, fs: Arc<dyn FileSystemService>) -> OperationResult<()> {
        // Check old path exists
        if !self.old_path.exists() {
            return Err(OperationError::ValidationFailed(
                format!("Source file does not exist: {}", self.old_path.display())
            ));
        }
        
        // Check parent directory write permission
        if let Some(parent) = self.old_path.parent() {
            if !fs.check_write_permission(parent).await.unwrap_or(false) {
                return Err(OperationError::ValidationFailed(
                    format!("No write permission for directory: {}", parent.display())
                ));
            }
        }
        
        // Check new name is valid
        if self.new_name.is_empty() {
            return Err(OperationError::ValidationFailed(
                "New name cannot be empty".to_string()
            ));
        }
        
        // Check new name doesn't contain invalid characters
        if self.new_name.contains('/') || self.new_name.contains('\\') {
            return Err(OperationError::ValidationFailed(
                "New name cannot contain path separators".to_string()
            ));
        }
        
        // Check if new path already exists
        if self.new_path.exists() {
            return Err(OperationError::ValidationFailed(
                format!("A file with the new name already exists: {}", self.new_path.display())
            ));
        }
        
        Ok(())
    }
    
    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }
    
    fn metadata_mut(&mut self) -> &mut CommandMetadata {
        &mut self.metadata
    }
    
    fn description(&self) -> String {
        format!("Rename {} to {}", 
            self.old_path.display(), 
            self.new_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;
    use crate::services::file_system::NativeFileSystemService;

    fn create_test_fs() -> Arc<dyn FileSystemService> {
        Arc::new(NativeFileSystemService::new())
    }

    #[tokio::test]
    async fn test_copy_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut command = CopyCommand::new(source_path.clone(), dest_path.clone());
        
        // Execute
        assert!(command.execute(fs.clone()).await.is_ok());
        assert!(dest_path.exists());
        assert_eq!(tokio::fs::read_to_string(&dest_path).await.unwrap(), "test content");
        assert!(command.is_executed());
        
        // Undo
        assert!(command.undo(fs).await.is_ok());
        assert!(!dest_path.exists());
        assert!(command.is_undone());
    }

    #[tokio::test]
    async fn test_move_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut command = MoveCommand::new(source_path.clone(), dest_path.clone());
        
        // Execute
        assert!(command.execute(fs.clone()).await.is_ok());
        assert!(!source_path.exists());
        assert!(dest_path.exists());
        assert_eq!(tokio::fs::read_to_string(&dest_path).await.unwrap(), "test content");
        
        // Undo
        assert!(command.undo(fs).await.is_ok());
        assert!(source_path.exists());
        assert!(!dest_path.exists());
        assert_eq!(tokio::fs::read_to_string(&source_path).await.unwrap(), "test content");
    }

    #[tokio::test]
    async fn test_delete_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create test file
        tokio::fs::write(&file_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut command = DeleteCommand::new(file_path.clone());
        
        // Execute
        assert!(command.execute(fs.clone()).await.is_ok());
        assert!(!file_path.exists());
        assert!(command.is_executed());
        
        // Undo
        assert!(command.undo(fs).await.is_ok());
        assert!(file_path.exists());
        assert_eq!(tokio::fs::read_to_string(&file_path).await.unwrap(), "test content");
        assert!(command.is_undone());
    }

    #[tokio::test]
    async fn test_rename_command_basic() {
        let temp_dir = TempDir::new().unwrap();
        let old_path = temp_dir.path().join("old.txt");
        let new_path = temp_dir.path().join("new.txt");
        
        // Create test file
        tokio::fs::write(&old_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut command = RenameCommand::new(old_path.clone(), "new.txt".to_string()).unwrap();
        
        // Execute
        assert!(command.execute(fs.clone()).await.is_ok());
        assert!(!old_path.exists());
        assert!(new_path.exists());
        assert_eq!(tokio::fs::read_to_string(&new_path).await.unwrap(), "test content");
        
        // Undo
        assert!(command.undo(fs).await.is_ok());
        assert!(old_path.exists());
        assert!(!new_path.exists());
        assert_eq!(tokio::fs::read_to_string(&old_path).await.unwrap(), "test content");
    }

    #[tokio::test]
    async fn test_command_validation() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        let fs = create_test_fs();
        
        // Test copy command validation with nonexistent source
        let command = CopyCommand::new(nonexistent_path.clone(), dest_path);
        assert!(command.validate(fs.clone()).await.is_err());
        
        // Test delete command validation with nonexistent file
        let command = DeleteCommand::new(nonexistent_path);
        assert!(command.validate(fs).await.is_err());
    }

    #[tokio::test]
    async fn test_command_status_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        tokio::fs::write(&source_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut command = CopyCommand::new(source_path, dest_path);
        
        // Initial status
        assert_eq!(command.metadata().status, CommandStatus::Pending);
        assert!(!command.is_executed());
        assert!(!command.can_undo());
        
        // After execution
        assert!(command.execute(fs.clone()).await.is_ok());
        assert_eq!(command.metadata().status, CommandStatus::Executed);
        assert!(command.is_executed());
        assert!(command.can_undo());
        assert!(command.metadata().executed_at.is_some());
        
        // After undo
        assert!(command.undo(fs).await.is_ok());
        assert_eq!(command.metadata().status, CommandStatus::Undone);
        assert!(command.is_undone());
        assert!(!command.can_undo());
        assert!(command.metadata().undone_at.is_some());
    }

    #[tokio::test]
    async fn test_batch_operation_success() {
        let temp_dir = TempDir::new().unwrap();
        let source_path1 = temp_dir.path().join("source1.txt");
        let dest_path1 = temp_dir.path().join("dest1.txt");
        let source_path2 = temp_dir.path().join("source2.txt");
        let dest_path2 = temp_dir.path().join("dest2.txt");
        
        // Create source files
        tokio::fs::write(&source_path1, "content1").await.unwrap();
        tokio::fs::write(&source_path2, "content2").await.unwrap();
        
        let fs = create_test_fs();
        let mut batch = BatchOperation::new("Test Batch".to_string(), "Test batch operation".to_string());
        
        // Add commands
        batch.add_command(Box::new(CopyCommand::new(source_path1.clone(), dest_path1.clone())));
        batch.add_command(Box::new(CopyCommand::new(source_path2.clone(), dest_path2.clone())));
        
        // Execute batch directly (not through processor for simpler testing)
        let start_time = SystemTime::now();
        batch.progress.status = BatchStatus::Validating;
        
        // Validate all commands
        for command in batch.commands.iter() {
            assert!(command.validate(fs.clone()).await.is_ok());
        }
        
        // Execute all commands
        batch.progress.status = BatchStatus::Executing;
        for (i, command) in batch.commands.iter_mut().enumerate() {
            assert!(command.execute(fs.clone()).await.is_ok());
            batch.executed_commands.push(i);
            batch.progress.completed_commands += 1;
        }
        
        // Mark as completed
        batch.progress.status = BatchStatus::Completed;
        batch.progress.elapsed_time = start_time.elapsed().ok();
        
        // Verify results
        assert_eq!(batch.progress.status, BatchStatus::Completed);
        assert_eq!(batch.progress.completed_commands, 2);
        assert_eq!(batch.progress.failed_commands, 0);
        assert!(dest_path1.exists());
        assert!(dest_path2.exists());
        assert_eq!(tokio::fs::read_to_string(&dest_path1).await.unwrap(), "content1");
        assert_eq!(tokio::fs::read_to_string(&dest_path2).await.unwrap(), "content2");
    }

    #[tokio::test]
    async fn test_batch_operation_rollback() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        let invalid_path = temp_dir.path().join("nonexistent/invalid.txt"); // Invalid destination
        
        // Create source file
        tokio::fs::write(&source_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut batch = BatchOperation::new("Failing Batch".to_string(), "Batch with failing command".to_string());
        
        // Add commands - first will succeed, second will fail
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), dest_path.clone())));
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), invalid_path.clone())));
        
        // Execute batch manually to test rollback
        batch.progress.status = BatchStatus::Executing;
        let mut execution_errors = Vec::new();
        
        // Execute first command (should succeed)
        if let Ok(_) = batch.commands[0].execute(fs.clone()).await {
            batch.executed_commands.push(0);
            batch.progress.completed_commands += 1;
        }
        
        // Verify first command succeeded
        assert!(dest_path.exists());
        
        // Execute second command (should fail)
        if let Err(e) = batch.commands[1].execute(fs.clone()).await {
            execution_errors.push((1, format!("Command failed: {}", e)));
            batch.progress.failed_commands += 1;
        }
        
        // Rollback since we don't allow partial failure
        for &command_index in batch.executed_commands.iter().rev() {
            batch.commands[command_index].undo(fs.clone()).await.unwrap();
        }
        
        batch.progress.status = BatchStatus::Failed;
        
        // Verify rollback worked
        assert!(!dest_path.exists()); // Should be rolled back
        assert!(execution_errors.len() == 1); // One command failed
        assert_eq!(batch.progress.status, BatchStatus::Failed);
    }

    #[tokio::test]
    async fn test_batch_processor_async_execution() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "async test").await.unwrap();
        
        let fs = create_test_fs();
        let processor = BatchProcessor::new(fs.clone());
        
        let mut batch = BatchOperation::new("Async Test".to_string(), "Test async execution".to_string());
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), dest_path.clone())));
        
        // Execute batch asynchronously
        let result = processor.execute_batch_async(batch).await;
        
        assert!(result.is_ok());
        let completed_batch = result.unwrap();
        assert_eq!(completed_batch.progress.status, BatchStatus::Completed);
        assert!(dest_path.exists());
        assert_eq!(tokio::fs::read_to_string(&dest_path).await.unwrap(), "async test");
        
        // Clean up processor
        processor.shutdown();
    }

    #[tokio::test]
    async fn test_batch_progress_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut files = Vec::new();
        
        // Create multiple source files
        for i in 0..5 {
            let path = temp_dir.path().join(format!("file{}.txt", i));
            tokio::fs::write(&path, format!("content{}", i)).await.unwrap();
            files.push(path);
        }
        
        let mut batch = BatchOperation::new("Progress Test".to_string(), "Test progress tracking".to_string());
        
        // Add multiple copy commands
        for (i, file) in files.iter().enumerate() {
            let dest = temp_dir.path().join(format!("dest{}.txt", i));
            batch.add_command(Box::new(CopyCommand::new(file.clone(), dest)));
        }
        
        assert_eq!(batch.progress.total_commands, 5);
        assert_eq!(batch.progress.completed_commands, 0);
        assert_eq!(batch.progress.completion_percentage(), 0.0);
        
        // Simulate progress
        batch.progress.completed_commands = 2;
        assert_eq!(batch.progress.completion_percentage(), 40.0);
        
        batch.progress.completed_commands = 5;
        assert_eq!(batch.progress.completion_percentage(), 100.0);
    }

    #[tokio::test]
    async fn test_batch_partial_failure_allowed() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path1 = temp_dir.path().join("dest1.txt");
        let invalid_path = temp_dir.path().join("nonexistent/invalid.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "partial failure test").await.unwrap();
        
        let fs = create_test_fs();
        let mut batch = BatchOperation::new("Partial Failure".to_string(), "Test partial failure".to_string())
            .with_partial_failure(true); // Allow partial failure
        
        // Add commands - one will succeed, one will fail
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), dest_path1.clone())));
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), invalid_path.clone())));
        
        // Manually execute to test partial failure behavior
        batch.progress.status = BatchStatus::Executing;
        
        // Execute first command (should succeed)
        if batch.commands[0].execute(fs.clone()).await.is_ok() {
            batch.executed_commands.push(0);
            batch.progress.completed_commands += 1;
        }
        
        // Execute second command (should fail but not rollback due to partial failure allowed)
        if batch.commands[1].execute(fs.clone()).await.is_err() {
            batch.progress.failed_commands += 1;
        }
        
        batch.progress.status = BatchStatus::Completed; // Still completed despite failure
        
        // Verify results
        assert!(dest_path1.exists()); // First command should still be executed
        assert!(!invalid_path.exists()); // Second command failed
        assert_eq!(batch.progress.completed_commands, 1);
        assert_eq!(batch.progress.failed_commands, 1);
        assert_eq!(batch.progress.status, BatchStatus::Completed);
    }

    #[test]
    fn test_batch_cancellation_token() {
        let batch = BatchOperation::new("Cancel Test".to_string(), "Test cancellation".to_string());
        
        // Initially not cancelled
        assert!(!batch.is_cancelled());
        
        // Cancel the batch
        batch.cancel();
        
        // Should now be cancelled
        assert!(batch.is_cancelled());
        
        // Get cancellation token
        if let Some(token) = batch.cancellation_token() {
            assert!(token.is_cancelled());
        } else {
            panic!("Cancellation token should be available");
        }
    }

    #[tokio::test]
    async fn test_batch_with_retries() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "retry test").await.unwrap();
        
        let mut batch = BatchOperation::new("Retry Test".to_string(), "Test retry behavior".to_string())
            .with_retries(2); // Allow 2 retries
        
        assert_eq!(batch.max_retries, 2);
        
        // Add a command
        batch.add_command(Box::new(CopyCommand::new(source_path.clone(), dest_path.clone())));
        
        // For this test, we'll just verify the retry configuration is set correctly
        // Actual retry behavior is tested in the BatchProcessor::execute_batch method
        let fs = create_test_fs();
        
        // Execute the command (should succeed on first try)
        assert!(batch.commands[0].execute(fs.clone()).await.is_ok());
        assert!(dest_path.exists());
    }

    // ===============================
    // OperationHistory Tests
    // ===============================

    #[tokio::test]
    async fn test_operation_history_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        // Create source file
        tokio::fs::write(&source_path, "test content").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Initially, no operations can be undone/redone
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
        
        // Create and execute a command
        let mut copy_cmd = CopyCommand::new(source_path.clone(), dest_path.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        
        // Add to history
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        // Now we should be able to undo
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);
        
        // Verify file was created
        assert!(dest_path.exists());
        
        // Undo the operation
        let undo_result = history.undo().await.unwrap();
        assert!(undo_result.contains("Undone"));
        
        // File should be removed (since it's a copy command, undo removes the destination)
        assert!(!dest_path.exists());
        
        // Now we should be able to redo but not undo
        assert!(!history.can_undo());
        assert!(history.can_redo());
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 1);
        
        // Redo the operation
        let redo_result = history.redo().await.unwrap();
        assert!(redo_result.contains("Redone"));
        
        // File should exist again
        assert!(dest_path.exists());
        
        // Back to original state
        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);
        assert_eq!(history.redo_count(), 0);
    }

    #[tokio::test]
    async fn test_operation_history_multiple_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");
        
        // Create source files
        tokio::fs::write(&file1, "content 1").await.unwrap();
        tokio::fs::write(&file2, "content 2").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Execute multiple operations
        let mut copy_cmd = CopyCommand::new(file1.clone(), file3.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        let mut rename_cmd = RenameCommand::new(file2.clone(), "renamed.txt".to_string()).unwrap();
        rename_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(rename_cmd)).await.unwrap();
        
        // Should have 2 operations to undo
        assert_eq!(history.undo_count(), 2);
        assert_eq!(history.redo_count(), 0);
        
        // Verify both operations worked
        assert!(file3.exists());
        assert!(!file2.exists());
        assert!(temp_dir.path().join("renamed.txt").exists());
        
        // Undo most recent (rename)
        history.undo().await.unwrap();
        assert!(file2.exists()); // Should be restored
        assert!(!temp_dir.path().join("renamed.txt").exists());
        
        // Undo the copy
        history.undo().await.unwrap();
        assert!(!file3.exists()); // Should be removed
        
        // Now we should have 2 redo operations available
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 2);
        
        // Redo copy
        history.redo().await.unwrap();
        assert!(file3.exists());
        
        // Redo rename
        history.redo().await.unwrap();
        assert!(!file2.exists());
        assert!(temp_dir.path().join("renamed.txt").exists());
    }

    #[tokio::test]
    async fn test_operation_history_new_operation_clears_redo() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");
        
        tokio::fs::write(&file1, "content").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Execute and add first operation
        let mut copy_cmd1 = CopyCommand::new(file1.clone(), file2.clone());
        copy_cmd1.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd1)).await.unwrap();
        
        // Undo it
        history.undo().await.unwrap();
        assert_eq!(history.redo_count(), 1);
        
        // Execute and add a new operation (should clear redo stack)
        let mut copy_cmd2 = CopyCommand::new(file1.clone(), file3.clone());
        copy_cmd2.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd2)).await.unwrap();
        
        // Redo stack should be cleared
        assert_eq!(history.redo_count(), 0);
        assert_eq!(history.undo_count(), 1);
        assert!(!file2.exists()); // First operation wasn't redone
        assert!(file3.exists()); // New operation was executed
    }

    #[tokio::test]
    async fn test_operation_history_cleanup_by_size() {
        let temp_dir = TempDir::new().unwrap();
        let fs = create_test_fs();
        
        // Create history with small limit
        let config = HistoryConfig {
            max_history_size: 3,
            ..Default::default()
        };
        let mut history = OperationHistory::with_config(fs.clone(), config);
        
        // Add 5 operations (exceeding limit of 3)
        for i in 1..=5 {
            let source = temp_dir.path().join(format!("source{}.txt", i));
            let dest = temp_dir.path().join(format!("dest{}.txt", i));
            tokio::fs::write(&source, format!("content {}", i)).await.unwrap();
            
            let mut copy_cmd = CopyCommand::new(source, dest);
            copy_cmd.execute(fs.clone()).await.unwrap();
            history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        }
        
        // Should be limited to 3 operations (90% of 3 = 2.7, so we keep 3)
        // Actually, cleanup triggers when we exceed the limit, so we should have 3 or fewer
        assert!(history.undo_count() <= 3);
        assert!(history.undo_count() >= 2); // But we should have at least 2 (90% cleanup target)
    }

    #[tokio::test]
    async fn test_operation_history_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Try to undo when nothing to undo
        let result = history.undo().await;
        assert!(matches!(result, Err(OperationError::NoUndoAvailable)));
        
        // Try to redo when nothing to redo
        let result = history.redo().await;
        assert!(matches!(result, Err(OperationError::NoRedoAvailable)));
        
        // Try to add non-executed command
        let copy_cmd = CopyCommand::new(
            temp_dir.path().join("nonexistent.txt"), 
            temp_dir.path().join("dest.txt")
        );
        // Don't execute it
        let result = history.add_executed_command(Box::new(copy_cmd)).await;
        assert!(matches!(result, Err(OperationError::HistoryError(_))));
    }

    #[tokio::test]
    async fn test_operation_history_stats() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        tokio::fs::write(&source_path, "test").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Initial stats
        let stats = history.get_stats();
        assert_eq!(stats.undo_operations, 0);
        assert_eq!(stats.redo_operations, 0);
        assert_eq!(stats.total_operations, 0);
        
        // Add an operation
        let mut copy_cmd = CopyCommand::new(source_path.clone(), dest_path.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        let stats = history.get_stats();
        assert_eq!(stats.undo_operations, 1);
        assert_eq!(stats.redo_operations, 0);
        assert_eq!(stats.total_operations, 1);
        assert!(stats.memory_usage_bytes > 0);
        
        // Undo it
        history.undo().await.unwrap();
        let stats = history.get_stats();
        assert_eq!(stats.undo_operations, 0);
        assert_eq!(stats.redo_operations, 1);
        assert_eq!(stats.total_operations, 1);
    }

    #[tokio::test]
    async fn test_operation_history_descriptions() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        tokio::fs::write(&source_path, "test").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Add operation
        let mut copy_cmd = CopyCommand::new(source_path.clone(), dest_path.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        // Check descriptions
        let undo_desc = history.next_undo_description();
        assert!(undo_desc.is_some());
        assert!(undo_desc.unwrap().contains("Copy"));
        
        let redo_desc = history.next_redo_description();
        assert!(redo_desc.is_none());
        
        // Undo and check again
        history.undo().await.unwrap();
        
        let undo_desc = history.next_undo_description();
        assert!(undo_desc.is_none());
        
        let redo_desc = history.next_redo_description();
        assert!(redo_desc.is_some());
        assert!(redo_desc.unwrap().contains("Copy"));
    }

    #[tokio::test]
    async fn test_operation_history_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        tokio::fs::write(&source_path, "test").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Add operation
        let mut copy_cmd = CopyCommand::new(source_path.clone(), dest_path.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        // Save to file
        history.save_to_file(&history_file).await.unwrap();
        assert!(history_file.exists());
        
        // Create new history and load
        let mut new_history = OperationHistory::new(fs.clone());
        new_history.load_from_file(&history_file).await.unwrap();
        
        // Should have the history entry but no actual commands (since they can't be serialized)
        assert_eq!(new_history.history_entries.len(), 1);
        assert_eq!(new_history.undo_count(), 0); // Commands not restored
        assert_eq!(new_history.redo_count(), 0);
    }

    #[tokio::test]
    async fn test_operation_history_persistence_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let history_file = temp_dir.path().join("history.json");
        
        let fs = create_test_fs();
        let config = HistoryConfig {
            persist_history: false,
            ..Default::default()
        };
        let history = OperationHistory::with_config(fs.clone(), config);
        
        // Save should fail when persistence is disabled
        let result = history.save_to_file(&history_file).await;
        assert!(matches!(result, Err(OperationError::HistoryError(_))));
        
        // Load should also fail
        let mut history = OperationHistory::with_config(fs.clone(), HistoryConfig {
            persist_history: false,
            ..Default::default()
        });
        let result = history.load_from_file(&history_file).await;
        assert!(matches!(result, Err(OperationError::HistoryError(_))));
    }

    #[tokio::test]
    async fn test_operation_history_clear() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        let dest_path = temp_dir.path().join("dest.txt");
        
        tokio::fs::write(&source_path, "test").await.unwrap();
        
        let fs = create_test_fs();
        let mut history = OperationHistory::new(fs.clone());
        
        // Add operation
        let mut copy_cmd = CopyCommand::new(source_path.clone(), dest_path.clone());
        copy_cmd.execute(fs.clone()).await.unwrap();
        history.add_executed_command(Box::new(copy_cmd)).await.unwrap();
        
        // Undo it to create redo entry
        history.undo().await.unwrap();
        
        assert!(history.undo_count() == 0);
        assert!(history.redo_count() == 1);
        assert!(history.memory_usage() > 0);
        
        // Clear everything
        history.clear();
        
        assert!(history.undo_count() == 0);
        assert!(history.redo_count() == 0);
        assert!(history.memory_usage() == 0);
        assert!(history.history_entries.is_empty());
    }

    #[tokio::test]
    async fn test_progress_info_basic() {
        let mut progress = ProgressInfo::new(100, 1000, "Test operation".to_string());
        
        // Test initial state
        assert_eq!(progress.current, 0);
        assert_eq!(progress.total, 100);
        assert_eq!(progress.bytes_processed, 0);
        assert_eq!(progress.total_bytes, 1000);
        assert_eq!(progress.speed_bps, 0);
        assert_eq!(progress.eta_seconds, None);
        assert_eq!(progress.current_operation, "Test operation");
        
        // Test percentages
        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.bytes_percentage(), 0.0);
        assert!(!progress.is_complete());
        
        // Update progress
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        progress.update(50, 500, "Halfway done".to_string());
        
        assert_eq!(progress.current, 50);
        assert_eq!(progress.bytes_processed, 500);
        assert_eq!(progress.current_operation, "Halfway done");
        assert_eq!(progress.percentage(), 50.0);
        assert_eq!(progress.bytes_percentage(), 50.0);
        assert!(!progress.is_complete());
        
        // Complete progress
        progress.update(100, 1000, "Complete".to_string());
        
        assert_eq!(progress.current, 100);
        assert_eq!(progress.bytes_processed, 1000);
        assert_eq!(progress.percentage(), 100.0);
        assert_eq!(progress.bytes_percentage(), 100.0);
        assert!(progress.is_complete());
    }

    #[tokio::test]
    async fn test_progress_info_speed_calculation() {
        let mut progress = ProgressInfo::new(10, 1000, "Speed test".to_string());
        
        // Sleep to ensure time difference
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        progress.update(5, 500, "Half done".to_string());
        
        // Speed should be calculated (bytes per second)
        assert!(progress.speed_bps > 0);
        
        let formatted_speed = progress.format_speed();
        assert!(formatted_speed.contains("B/s"));
        
        // ETA calculation depends on progress completion
        let formatted_eta = progress.format_eta();
        
        // ETA should be a valid format (contains "s", "Unknown", or "Complete")
        assert!(formatted_eta.contains("s") || formatted_eta == "Unknown" || formatted_eta == "Complete");
    }

    #[tokio::test]
    async fn test_cancellation_token() {
        let token = CancellationToken::new();
        
        // Initially not cancelled
        assert!(!token.is_cancelled());
        assert!(token.throw_if_cancelled().is_ok());
        
        // Test progress counter
        assert_eq!(token.progress_count(), 0);
        assert_eq!(token.increment_progress(), 1);
        assert_eq!(token.increment_progress(), 2);
        assert_eq!(token.progress_count(), 2);
        
        // Cancel the token
        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.throw_if_cancelled().is_err());
        
        // Reset the token
        token.reset();
        assert!(!token.is_cancelled());
        assert_eq!(token.progress_count(), 0);
        assert!(token.throw_if_cancelled().is_ok());
    }

    #[tokio::test]
    async fn test_progress_tracker_basic() {
        let mut tracker = ProgressTracker::new(10, 1000, "Basic test".to_string());
        
        // Test initial state
        assert!(!tracker.is_cancelled());
        assert!(!tracker.is_complete());
        assert_eq!(tracker.progress().current, 0);
        assert_eq!(tracker.progress().total, 10);
        
        // Test update
        assert!(tracker.update(5, 500, "Half done".to_string()).is_ok());
        assert_eq!(tracker.progress().current, 5);
        assert_eq!(tracker.progress().bytes_processed, 500);
        
        // Test increment
        assert!(tracker.increment(100).is_ok());
        assert_eq!(tracker.progress().current, 6);
        assert_eq!(tracker.progress().bytes_processed, 600);
        
        // Test completion
        assert!(tracker.complete().is_ok());
        assert!(tracker.is_complete());
        assert_eq!(tracker.progress().current, 10);
        assert_eq!(tracker.progress().bytes_processed, 1000);
    }

    #[tokio::test]
    async fn test_progress_tracker_with_cancellation() {
        let cancellation_token = CancellationToken::new();
        let mut tracker = ProgressTracker::with_cancellation(
            10, 1000, "Cancellation test".to_string(), cancellation_token.clone()
        );
        
        // Normal operation should work
        assert!(tracker.update(3, 300, "Working".to_string()).is_ok());
        
        // Cancel the operation
        cancellation_token.cancel();
        assert!(tracker.is_cancelled());
        
        // Further operations should fail
        assert!(tracker.update(5, 500, "Cancelled".to_string()).is_err());
        assert!(tracker.increment(100).is_err());
        assert!(tracker.complete().is_err());
    }

    #[tokio::test]
    async fn test_progress_tracker_with_callback() {
        use std::sync::{Arc, Mutex};
        
        let callback_data = Arc::new(Mutex::new(Vec::new()));
        let callback_data_clone = callback_data.clone();
        
        let callback: ProgressCallback = Arc::new(move |progress| {
            callback_data_clone.lock().unwrap().push((progress.current, progress.bytes_processed));
        });
        
        let mut tracker = ProgressTracker::new(5, 500, "Callback test".to_string())
            .with_callback(callback);
        
        // Update progress - callback should be triggered
        assert!(tracker.update(2, 200, "Progress".to_string()).is_ok());
        assert!(tracker.update(4, 400, "More progress".to_string()).is_ok());
        
        // Check callback was called
        let data = callback_data.lock().unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0], (2, 200));
        assert_eq!(data[1], (4, 400));
    }

    #[tokio::test]
    async fn test_progress_serialization() {
        let progress = ProgressInfo::new(100, 1000, "Serialization test".to_string());
        
        // Test serialization
        let serialized = serde_json::to_string(&progress).unwrap();
        assert!(serialized.contains("Serialization test"));
        
        // Test deserialization
        let deserialized: ProgressInfo = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.current, progress.current);
        assert_eq!(deserialized.total, progress.total);
        assert_eq!(deserialized.current_operation, progress.current_operation);
    }

    #[tokio::test]
    async fn test_format_functions() {
        // Test format_bytes_per_second
        assert_eq!(format_bytes_per_second(512), "512.0 B/s");
        assert_eq!(format_bytes_per_second(1536), "1.5 KB/s");
        assert_eq!(format_bytes_per_second(2097152), "2.0 MB/s");
        assert_eq!(format_bytes_per_second(3221225472), "3.0 GB/s");
        
        // Test format_duration
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m 1s");
    }

    #[tokio::test]
    async fn test_command_progress_integration() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        // Create file system service
        let fs_service: Arc<dyn FileSystemService> = Arc::new(NativeFileSystemService::new());
        
        // Create a test file
        let test_file = temp_path.join("test.txt");
        tokio::fs::write(&test_file, "test content").await.unwrap();
        
        // Create copy command
        let dest_file = temp_path.join("test_copy.txt");
        let mut copy_cmd = CopyCommand::new(test_file, dest_file);
        
        // Test execute with progress tracking
        let mut progress_tracker = ProgressTracker::new(1, 12, "Copying file".to_string());
        
        // Default implementation should work (calls execute without progress)
        assert!(copy_cmd.execute_with_progress(fs_service.clone(), Some(&mut progress_tracker)).await.is_ok());
        assert!(copy_cmd.is_executed());
        
        // Test undo with progress tracking
        assert!(copy_cmd.undo_with_progress(fs_service, Some(&mut progress_tracker)).await.is_ok());
        assert!(copy_cmd.is_undone());
    }

    #[tokio::test]
    async fn test_command_estimate_work() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        let fs_service: Arc<dyn FileSystemService> = Arc::new(NativeFileSystemService::new());
        
        // Create test file
        let test_file = temp_path.join("test.txt");
        tokio::fs::write(&test_file, "test content").await.unwrap();
        
        let dest_file = temp_path.join("test_copy.txt");
        let copy_cmd = CopyCommand::new(test_file, dest_file);
        
        // Test work estimation (default implementation returns (1, 0))
        let (items, bytes) = copy_cmd.estimate_work(fs_service).await.unwrap();
        assert_eq!(items, 1);
        assert_eq!(bytes, 0);
        
        // Test supports_progress (default is false)
        assert!(!copy_cmd.supports_progress());
    }

    #[tokio::test]
    async fn test_progress_tracker_persistence() {
        let mut tracker = ProgressTracker::new(100, 1000, "Persistence test".to_string())
            .with_persistence(true);
        
        assert!(tracker.persist_progress);
        
        // Update progress
        assert!(tracker.update(50, 500, "Half done".to_string()).is_ok());
        
        // Progress should be persisted (implementation would need actual persistence logic)
        let progress = tracker.progress();
        assert_eq!(progress.current, 50);
        assert_eq!(progress.bytes_processed, 500);
    }

    #[tokio::test]
    async fn test_progress_edge_cases() {
        // Test zero totals
        let mut progress = ProgressInfo::new(0, 0, "Zero test".to_string());
        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.bytes_percentage(), 0.0);
        
        progress.update(0, 0, "Still zero".to_string());
        assert_eq!(progress.percentage(), 0.0);
        assert_eq!(progress.bytes_percentage(), 0.0);
        
        // Test overshooting
        progress = ProgressInfo::new(10, 100, "Overshoot test".to_string());
        progress.update(15, 150, "Overshooting".to_string());
        assert_eq!(progress.percentage(), 150.0);
        assert_eq!(progress.bytes_percentage(), 150.0);
    }
}