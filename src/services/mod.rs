pub mod file_system;
pub mod cache;
pub mod preview;
pub mod operations;

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