pub mod file_system;
pub mod cache;
pub mod preview;
pub mod operations;
pub mod hashing;
pub mod background;
pub mod duplicate_detection;

pub use file_system::{FileSystemService, NativeFileSystemService, FileEntry};
pub use cache::{
    CacheService, CacheConfig, CacheError, CachedFileMetadata, CachedThumbnail, CacheStats,
    CacheCleanupResult, MetadataCacheStats, ThumbnailCacheStats, DatabaseStats, CacheMetrics,
    CacheMaintenanceConfig, CacheMaintenanceResult
};
pub use preview::{
    PreviewService, PreviewHandler, PreviewData, PreviewConfig, PreviewError,
    SupportedFormat, FileMetadata, ExifData, PreviewContent, VideoThumbnail,
    ThumbnailTask, ImagePreviewHandler, VideoPreviewHandler, AudioPreviewHandler, PdfPreviewHandler, TextPreviewHandler,
    ThumbnailService, ThumbnailPriority, ThumbnailJobStatus, ThumbnailJobConfig, ThumbnailJob, ThumbnailServiceStats,
    MetadataDisplay, BasicInfoSection, TechnicalInfoSection, ContentInfoSection, ExifInfoSection, TimestampInfoSection
};
pub use operations::{
    Command, OperationError, OperationResult, CommandStatus, CommandMetadata,
    CopyCommand, MoveCommand, DeleteCommand, RenameCommand,
    BatchOperation, BatchStatus, BatchProgress, BatchProcessor, BatchMessage,
    OperationHistory, HistoryConfig, HistoryEntry, HistorySnapshot, HistoryStats,
    ProgressInfo, ProgressTracker, CancellationToken, ProgressCallback,
    ErrorSeverity, RecoveryStrategy, RecoverySuggestion, RetryConfig,
    ErrorRecoveryManager, ErrorStatistics
};
pub use hashing::{
    HashingService, HashingConfig, HashingError, HashingResult, FileHash, HashAlgorithm
};
pub use background::{
    BackgroundProcessor, BackgroundError, BackgroundResult, TaskStatus, 
    ProgressInfo as BackgroundProgressInfo, ProgressCallback as BackgroundProgressCallback, 
    HashingTask, HashingTaskResult
};
pub use duplicate_detection::{
    DuplicateDetector, DuplicateDetectionError, DuplicateDetectionResult, DuplicateDetectionResults,
    ComparisonMethod, DuplicateFile, DuplicateGroup, PrimarySelectionStrategy, FileSortCriteria,
    DuplicateDetectionConfig, DetectionProgress, DetectionPhase, DetectionProgressCallback
};