pub mod file_system;
pub mod cache;
pub mod preview;
pub mod preview_cache;
pub mod progressive_loader;
pub mod preview_service;
// Temporarily disabled due to compilation errors
// pub mod validation_profiler;
// pub mod validation_runner;
// pub mod ui_profiler;
// pub mod gpu_preview;
// pub mod performance_benchmarks;
// pub mod memory_optimizer;
pub mod operations;
pub mod hashing;
pub mod background;
pub mod duplicate_detection;

// Re-export only actively used types to reduce unused import warnings
pub use file_system::{FileEntry, PreviewMetadata, ExifMetadata};
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
pub use preview_cache::{
    ThreadSafePreviewCache, PreviewCacheConfig,
    PreviewCacheKey, CachedPreviewData, PreviewCacheStats
};
pub use progressive_loader::{
    ProgressiveLoader, ProgressiveLoaderConfig, ProgressiveLoaderError,
    ProgressiveLoadHandle, LoadingProgress, LoadingStage
};
pub use preview_service::{
    PreviewService, PreviewServiceConfig, PreviewServiceError, PreviewServiceHealth
};
// Temporarily disabled due to compilation errors
/*
pub use validation_profiler::{
    ValidationProfiler, ValidationReport, ValidationConfig, ValidationError,
    MemoryValidationResult, CacheValidationResult, ProgressiveValidationResult,
    PerformanceBenchmarks, OptimizationRecommendation
};
pub use validation_runner::{
    ValidationRunner, ValidationSummary, ValidationStatus, KeyMetrics
};
pub use ui_profiler::{
    UIPerformanceProfiler, UIMetrics, LayoutPerformanceMetrics, ThemePerformanceMetrics,
    GpuPerformanceMetrics, UIProfilingConfig, LayoutType, GpuOperation,
    UIPerformanceReport, PerformanceRecommendation
};
pub use gpu_preview::{
    GpuPreviewRenderer, GpuPreviewConfig, GpuImageData, GpuTextureFormat,
    GpuAdapterInfo, GpuMemoryInfo, GpuPowerPreference, TextureFilter
};
pub use performance_benchmarks::{
    PerformanceBenchmarkSuite, BenchmarkConfig, BenchmarkResults, LayoutBenchmarkResults,
    ThemeBenchmarkResults, GpuBenchmarkResults, PerformanceGrade, BaselineMetrics
};
pub use memory_optimizer::{
    MemoryOptimizer, MemoryOptimizerConfig, MemoryUsageStats, MemoryOptimizationResult,
    MemoryBenchmarkResults, MemoryPerformanceGrade, benchmark_memory_optimization
};
*/