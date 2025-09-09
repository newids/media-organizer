//! MediaOrganizer Library
//! 
//! Cross-platform media/file management application built with Dioxus.


pub mod models;
pub mod performance;
pub mod services;
pub mod state;
pub mod theme;
pub mod ui;
pub mod utils;

// Re-export commonly used types for convenience
pub use state::{LayoutState, Theme, ActivityBarPosition, SidebarPosition, PanelPosition};

// Re-export performance profiling components - temporarily disabled
// pub use services::performance_benchmarks::{PerformanceBenchmarkSuite, BenchmarkConfig, BenchmarkResults};
// pub use services::ui_profiler::{UIPerformanceProfiler, UIProfilingConfig};
// #[cfg(feature = "gpu-acceleration")]
// pub use services::gpu_preview::{GpuPreviewRenderer, GpuPreviewConfig};
pub use state::performance::{PerformanceProfiler, PerformanceReport, init_profiler, with_profiler};
pub use performance::rendering_optimizations::{RenderingOptimizationSuite, ThemeOptimizer, DragOptimizer, VirtualScrollOptimizer, DOMBatchOptimizer, RenderingProfiler};

// Temporarily disabled - performance profiling infrastructure
/*
/// Initialize performance profiling infrastructure for the MediaOrganizer
pub async fn init_performance_systems() -> Result<Arc<UIPerformanceProfiler>, Box<dyn std::error::Error>> {
    // Initialize global state performance profiler
    init_profiler();
    
    // Create UI performance profiler
    let ui_profiler = Arc::new(UIPerformanceProfiler::with_config(UIProfilingConfig::default()));
    
    tracing::info!("Performance profiling systems initialized");
    
    Ok(ui_profiler)
}

/// Create and configure a comprehensive benchmark suite
pub async fn create_benchmark_suite() -> Result<PerformanceBenchmarkSuite, Box<dyn std::error::Error>> {
    let ui_profiler = init_performance_systems().await?;
    
    let config = BenchmarkConfig {
        iterations: 50, // Reduced for faster testing
        layout_target_ms: 100.0,
        theme_target_ms: 50.0,
        enable_gpu_benchmarks: cfg!(feature = "gpu-acceleration"),
        profile_memory: true,
        warmup_iterations: 5,
    };
    
    let suite = PerformanceBenchmarkSuite::with_config(ui_profiler, config);
    
    // Try to initialize GPU renderer if available
    #[cfg(feature = "gpu-acceleration")]
    let suite = suite.with_gpu_renderer().await;
    
    Ok(suite)
}

/// Run comprehensive performance benchmarks
pub async fn run_performance_benchmarks() -> Result<BenchmarkResults, Box<dyn std::error::Error>> {
    tracing::info!("Starting MediaOrganizer performance benchmarking");
    
    let suite = create_benchmark_suite().await?;
    let results = suite.run_benchmarks().await;
    
    tracing::info!(
        "Benchmark completed with grade: {:?}, Layout: {:.1}ms, Theme: {:.1}ms", 
        results.performance_grade,
        results.layout_benchmarks.average_ms,
        results.theme_benchmarks.average_ms
    );
    
    Ok(results)
}
*/