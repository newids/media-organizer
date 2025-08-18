pub mod file_system;
pub mod cache;
pub mod preview;
pub mod operations;
pub mod hashing;
pub mod background;
pub mod duplicate_detection;

// Re-export only actively used types to reduce unused import warnings
pub use file_system::{FileEntry};
pub use operations::{
    ProgressInfo, ErrorSeverity
};
pub use hashing::{
    HashingService, FileHash
};
pub use background::{
    BackgroundProcessor,
    ProgressInfo as BackgroundProgressInfo, ProgressCallback as BackgroundProgressCallback, 
    HashingTask
};
pub use duplicate_detection::{
    DuplicateDetector, DuplicateDetectionResults,
    ComparisonMethod, DuplicateGroup, PrimarySelectionStrategy,
    DuplicateDetectionConfig, DetectionProgress
};