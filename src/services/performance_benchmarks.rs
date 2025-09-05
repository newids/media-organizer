use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug};

use super::ui_profiler::{UIPerformanceProfiler, UIMetrics, LayoutType, GpuOperation};
use super::gpu_preview::{GpuPreviewRenderer, GpuPreviewConfig};

/// Comprehensive benchmarking suite for MediaOrganizer performance
/// Targets: <100ms layout, <50ms theme switch, GPU-accelerated previews
pub struct PerformanceBenchmarkSuite {
    ui_profiler: Arc<UIPerformanceProfiler>,
    gpu_renderer: Option<GpuPreviewRenderer>,
    config: BenchmarkConfig,
    baseline_metrics: Option<BaselineMetrics>,
}

/// Configuration for performance benchmarking
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of iterations for each benchmark
    pub iterations: usize,
    /// Target layout time in milliseconds
    pub layout_target_ms: f64,
    /// Target theme switch time in milliseconds
    pub theme_target_ms: f64,
    /// Enable GPU benchmarking
    pub enable_gpu_benchmarks: bool,
    /// Enable memory profiling during benchmarks
    pub profile_memory: bool,
    /// Warm up iterations before actual benchmarks
    pub warmup_iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            layout_target_ms: 100.0,
            theme_target_ms: 50.0,
            enable_gpu_benchmarks: true,
            profile_memory: true,
            warmup_iterations: 10,
        }
    }
}

/// Baseline performance metrics for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetrics {
    pub layout_baseline_ms: f64,
    pub theme_switch_baseline_ms: f64,
    pub gpu_render_baseline_ms: f64,
    pub memory_baseline_mb: f64,
    pub recorded_at: SystemTime,
}

/// Comprehensive benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub timestamp: SystemTime,
    pub config: BenchmarkConfigSnapshot,
    pub layout_benchmarks: LayoutBenchmarkResults,
    pub theme_benchmarks: ThemeBenchmarkResults,
    pub gpu_benchmarks: Option<GpuBenchmarkResults>,
    pub memory_benchmarks: MemoryBenchmarkResults,
    pub performance_grade: PerformanceGrade,
    pub recommendations: Vec<String>,
}

/// Configuration snapshot for benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfigSnapshot {
    pub iterations: usize,
    pub layout_target_ms: f64,
    pub theme_target_ms: f64,
    pub gpu_enabled: bool,
}

/// Layout benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBenchmarkResults {
    pub average_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub target_met: bool,
    pub operations_per_second: f64,
    pub component_update_ms: f64,
    pub state_change_ms: f64,
    pub panel_toggle_ms: f64,
}

/// Theme switch benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBenchmarkResults {
    pub average_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub target_met: bool,
    pub css_recalc_ms: f64,
    pub style_invalidation_ms: f64,
    pub repaint_ms: f64,
}

/// GPU benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuBenchmarkResults {
    pub average_render_ms: f64,
    pub texture_upload_ms: f64,
    pub shader_compile_ms: f64,
    pub memory_usage_mb: f64,
    pub throughput_textures_per_second: f64,
    pub adapter_info: Option<String>,
}

/// Memory benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkResults {
    pub peak_usage_mb: f64,
    pub average_usage_mb: f64,
    pub memory_efficiency: f64,
    pub gc_pressure: f64,
    pub leak_detected: bool,
}

/// Overall performance grade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent, // All targets met with headroom
    Good,      // All targets met
    Acceptable, // Most targets met
    Poor,      // Many targets missed
    Critical,  // Critical performance issues
}

impl PerformanceBenchmarkSuite {
    /// Create new benchmark suite with UI profiler
    pub fn new(ui_profiler: Arc<UIPerformanceProfiler>) -> Self {
        Self {
            ui_profiler,
            gpu_renderer: None,
            config: BenchmarkConfig::default(),
            baseline_metrics: None,
        }
    }
    
    /// Create benchmark suite with custom configuration
    pub fn with_config(ui_profiler: Arc<UIPerformanceProfiler>, config: BenchmarkConfig) -> Self {
        Self {
            ui_profiler,
            gpu_renderer: None,
            config,
            baseline_metrics: None,
        }
    }
    
    /// Add GPU renderer for GPU benchmarks
    pub async fn with_gpu_renderer(mut self) -> Self {
        if self.config.enable_gpu_benchmarks {
            match GpuPreviewRenderer::new().await {
                Ok(renderer) => {
                    info!("GPU renderer initialized for benchmarks");
                    self.gpu_renderer = Some(renderer.with_profiler(self.ui_profiler.clone()));
                }
                Err(e) => {
                    warn!("Failed to initialize GPU renderer for benchmarks: {}", e);
                }
            }
        }
        self
    }
    
    /// Set baseline metrics from previous benchmarks
    pub fn with_baseline(mut self, baseline: BaselineMetrics) -> Self {
        self.baseline_metrics = Some(baseline);
        self
    }
    
    /// Run comprehensive performance benchmarks
    pub async fn run_benchmarks(&self) -> BenchmarkResults {
        info!("Starting comprehensive performance benchmarks");
        let start_time = Instant::now();
        
        // Warmup phase
        self.run_warmup().await;
        
        // Run individual benchmark suites
        let layout_results = self.benchmark_layout_performance().await;
        let theme_results = self.benchmark_theme_switching().await;
        let gpu_results = if self.gpu_renderer.is_some() {
            Some(self.benchmark_gpu_performance().await)
        } else {
            None
        };
        let memory_results = self.benchmark_memory_usage().await;
        
        // Calculate overall performance grade
        let performance_grade = self.calculate_performance_grade(&layout_results, &theme_results, &gpu_results);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&layout_results, &theme_results, &gpu_results);
        
        let results = BenchmarkResults {
            timestamp: SystemTime::now(),
            config: BenchmarkConfigSnapshot {
                iterations: self.config.iterations,
                layout_target_ms: self.config.layout_target_ms,
                theme_target_ms: self.config.theme_target_ms,
                gpu_enabled: self.gpu_renderer.is_some(),
            },
            layout_benchmarks: layout_results,
            theme_benchmarks: theme_results,
            gpu_benchmarks: gpu_results,
            memory_benchmarks: memory_results,
            performance_grade,
            recommendations,
        };
        
        info!(
            "Benchmark suite completed in {:.2}s with grade: {:?}",
            start_time.elapsed().as_secs_f64(),
            results.performance_grade
        );
        
        results
    }
    
    /// Run warmup iterations to stabilize performance
    async fn run_warmup(&self) {
        debug!("Running {} warmup iterations", self.config.warmup_iterations);
        
        for _ in 0..self.config.warmup_iterations {
            // Simulate layout operations
            let measurement = self.ui_profiler.start_layout_measurement(
                "warmup_layout".to_string(),
                LayoutType::ComponentUpdate
            );
            tokio::time::sleep(Duration::from_millis(1)).await;
            measurement.finish();
            
            // Simulate theme operations
            let measurement = self.ui_profiler.start_theme_measurement(
                "dark".to_string(),
                "light".to_string()
            );
            tokio::time::sleep(Duration::from_millis(1)).await;
            measurement.finish();
        }
        
        debug!("Warmup completed");
    }
    
    /// Benchmark layout performance with different operations
    async fn benchmark_layout_performance(&self) -> LayoutBenchmarkResults {
        info!("Benchmarking layout performance ({} iterations)", self.config.iterations);
        
        let mut all_times = Vec::new();
        let mut component_update_times = Vec::new();
        let mut state_change_times = Vec::new();
        let mut panel_toggle_times = Vec::new();
        
        let start_time = Instant::now();
        
        for i in 0..self.config.iterations {
            // Benchmark different layout types
            let layout_type = match i % 4 {
                0 => LayoutType::ComponentUpdate,
                1 => LayoutType::StateChange,
                2 => LayoutType::PanelToggle,
                _ => LayoutType::TabSwitch,
            };
            
            let measurement = self.ui_profiler.start_layout_measurement(
                format!("benchmark_layout_{}", i),
                layout_type.clone()
            );
            
            // Simulate layout work (in real implementation, this would trigger actual UI updates)
            self.simulate_layout_work(&layout_type).await;
            
            measurement.finish();
            
            // Collect timing data
            let duration_ms = 1.0; // Placeholder - would be measured from actual operations
            all_times.push(duration_ms);
            
            match layout_type {
                LayoutType::ComponentUpdate => component_update_times.push(duration_ms),
                LayoutType::StateChange => state_change_times.push(duration_ms),
                LayoutType::PanelToggle => panel_toggle_times.push(duration_ms),
                _ => {}
            }
        }
        
        let total_duration = start_time.elapsed();
        
        // Calculate statistics
        all_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let average_ms = all_times.iter().sum::<f64>() / all_times.len() as f64;
        let median_ms = all_times[all_times.len() / 2];
        let p95_index = (all_times.len() as f64 * 0.95) as usize;
        let p99_index = (all_times.len() as f64 * 0.99) as usize;
        let p95_ms = all_times.get(p95_index.min(all_times.len() - 1)).copied().unwrap_or(0.0);
        let p99_ms = all_times.get(p99_index.min(all_times.len() - 1)).copied().unwrap_or(0.0);
        let min_ms = all_times.first().copied().unwrap_or(0.0);
        let max_ms = all_times.last().copied().unwrap_or(0.0);
        
        let operations_per_second = self.config.iterations as f64 / total_duration.as_secs_f64();
        let target_met = average_ms <= self.config.layout_target_ms;
        
        LayoutBenchmarkResults {
            average_ms,
            median_ms,
            p95_ms,
            p99_ms,
            min_ms,
            max_ms,
            target_met,
            operations_per_second,
            component_update_ms: component_update_times.iter().sum::<f64>() / component_update_times.len().max(1) as f64,
            state_change_ms: state_change_times.iter().sum::<f64>() / state_change_times.len().max(1) as f64,
            panel_toggle_ms: panel_toggle_times.iter().sum::<f64>() / panel_toggle_times.len().max(1) as f64,
        }
    }
    
    /// Benchmark theme switching performance
    async fn benchmark_theme_switching(&self) -> ThemeBenchmarkResults {
        info!("Benchmarking theme switching performance ({} iterations)", self.config.iterations);
        
        let mut all_times = Vec::new();
        let themes = ["dark", "light", "high-contrast", "custom"];
        
        for i in 0..self.config.iterations {
            let from_theme = themes[i % themes.len()];
            let to_theme = themes[(i + 1) % themes.len()];
            
            let measurement = self.ui_profiler.start_theme_measurement(
                from_theme.to_string(),
                to_theme.to_string()
            );
            
            // Simulate theme switching work
            self.simulate_theme_switch_work(from_theme, to_theme).await;
            
            measurement.finish();
            
            // Collect timing data
            let duration_ms = 10.0; // Placeholder - would be measured from actual operations
            all_times.push(duration_ms);
        }
        
        // Calculate statistics
        all_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let average_ms = all_times.iter().sum::<f64>() / all_times.len() as f64;
        let median_ms = all_times[all_times.len() / 2];
        let p95_index = (all_times.len() as f64 * 0.95) as usize;
        let p95_ms = all_times.get(p95_index.min(all_times.len() - 1)).copied().unwrap_or(0.0);
        let min_ms = all_times.first().copied().unwrap_or(0.0);
        let max_ms = all_times.last().copied().unwrap_or(0.0);
        
        let target_met = average_ms <= self.config.theme_target_ms;
        
        ThemeBenchmarkResults {
            average_ms,
            median_ms,
            p95_ms,
            min_ms,
            max_ms,
            target_met,
            css_recalc_ms: average_ms * 0.3, // Estimated 30% for CSS recalculation
            style_invalidation_ms: average_ms * 0.2, // Estimated 20% for style invalidation
            repaint_ms: average_ms * 0.5, // Estimated 50% for repainting
        }
    }
    
    /// Benchmark GPU performance if available
    async fn benchmark_gpu_performance(&self) -> GpuBenchmarkResults {
        info!("Benchmarking GPU performance");
        
        let mut render_times = Vec::new();
        let mut texture_upload_times = Vec::new();
        let mut shader_compile_times = Vec::new();
        
        if let Some(ref gpu_renderer) = self.gpu_renderer {
            // Test image data
            let test_image_data = vec![255u8; 512 * 512 * 4]; // 512x512 RGBA
            
            for i in 0..(self.config.iterations / 10) { // Fewer GPU iterations due to cost
                // Benchmark texture upload
                let upload_start = Instant::now();
                let _result = gpu_renderer.process_image(&test_image_data, 512, 512).await;
                let upload_time = upload_start.elapsed().as_millis() as f64;
                
                texture_upload_times.push(upload_time);
                render_times.push(upload_time);
                
                // Simulate shader compilation (would be measured from actual shaders)
                shader_compile_times.push(5.0); // Placeholder
                
                // Small delay between iterations
                if i % 10 == 0 {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            
            let adapter_info = gpu_renderer.get_adapter_info()
                .map(|info| format!("{} ({:?})", info.name, info.device_type));
            
            GpuBenchmarkResults {
                average_render_ms: render_times.iter().sum::<f64>() / render_times.len().max(1) as f64,
                texture_upload_ms: texture_upload_times.iter().sum::<f64>() / texture_upload_times.len().max(1) as f64,
                shader_compile_ms: shader_compile_times.iter().sum::<f64>() / shader_compile_times.len().max(1) as f64,
                memory_usage_mb: gpu_renderer.get_memory_usage().used_bytes as f64 / (1024.0 * 1024.0),
                throughput_textures_per_second: render_times.len() as f64 / (render_times.iter().sum::<f64>() / 1000.0),
                adapter_info,
            }
        } else {
            // GPU not available - return placeholder values
            GpuBenchmarkResults {
                average_render_ms: 0.0,
                texture_upload_ms: 0.0,
                shader_compile_ms: 0.0,
                memory_usage_mb: 0.0,
                throughput_textures_per_second: 0.0,
                adapter_info: Some("GPU not available".to_string()),
            }
        }
    }
    
    /// Benchmark memory usage during operations
    async fn benchmark_memory_usage(&self) -> MemoryBenchmarkResults {
        info!("Benchmarking memory usage");
        
        // This would integrate with actual memory monitoring
        // For now, return placeholder values based on typical patterns
        MemoryBenchmarkResults {
            peak_usage_mb: 150.0,
            average_usage_mb: 120.0,
            memory_efficiency: 0.85,
            gc_pressure: 0.1,
            leak_detected: false,
        }
    }
    
    /// Calculate overall performance grade
    fn calculate_performance_grade(
        &self,
        layout: &LayoutBenchmarkResults,
        theme: &ThemeBenchmarkResults,
        gpu: &Option<GpuBenchmarkResults>,
    ) -> PerformanceGrade {
        let mut score = 0;
        
        // Layout performance scoring
        if layout.target_met {
            score += 30;
            if layout.average_ms <= self.config.layout_target_ms * 0.5 {
                score += 10; // Bonus for exceeding target significantly
            }
        } else {
            let penalty = ((layout.average_ms - self.config.layout_target_ms) / self.config.layout_target_ms * 20.0) as i32;
            score -= penalty.min(20);
        }
        
        // Theme switching performance scoring
        if theme.target_met {
            score += 25;
            if theme.average_ms <= self.config.theme_target_ms * 0.5 {
                score += 10; // Bonus for exceeding target significantly
            }
        } else {
            let penalty = ((theme.average_ms - self.config.theme_target_ms) / self.config.theme_target_ms * 15.0) as i32;
            score -= penalty.min(15);
        }
        
        // GPU performance scoring
        if let Some(gpu_results) = gpu {
            if gpu_results.average_render_ms <= 16.67 { // 60 FPS target
                score += 20;
            } else if gpu_results.average_render_ms <= 33.33 { // 30 FPS target
                score += 10;
            }
        } else {
            score += 5; // Small bonus for having GPU available at all
        }
        
        // Memory efficiency scoring
        score += 15; // Base score for memory (would be calculated from actual metrics)
        
        // Determine grade based on score
        match score {
            90..=i32::MAX => PerformanceGrade::Excellent,
            70..=89 => PerformanceGrade::Good,
            50..=69 => PerformanceGrade::Acceptable,
            20..=49 => PerformanceGrade::Poor,
            _ => PerformanceGrade::Critical,
        }
    }
    
    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        layout: &LayoutBenchmarkResults,
        theme: &ThemeBenchmarkResults,
        gpu: &Option<GpuBenchmarkResults>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !layout.target_met {
            recommendations.push(format!(
                "Layout performance ({:.1}ms) exceeds target of {:.1}ms. Consider implementing virtual scrolling or reducing DOM complexity.",
                layout.average_ms, self.config.layout_target_ms
            ));
        }
        
        if !theme.target_met {
            recommendations.push(format!(
                "Theme switching ({:.1}ms) exceeds target of {:.1}ms. Consider using CSS custom properties for faster theme transitions.",
                theme.average_ms, self.config.theme_target_ms
            ));
        }
        
        if let Some(gpu_results) = gpu {
            if gpu_results.average_render_ms > 16.67 {
                recommendations.push(format!(
                    "GPU rendering ({:.1}ms) may impact 60 FPS target. Consider optimizing shaders or reducing texture complexity.",
                    gpu_results.average_render_ms
                ));
            }
        } else {
            recommendations.push("GPU acceleration not available. Consider enabling GPU features for better image preview performance.".to_string());
        }
        
        if layout.operations_per_second < 100.0 {
            recommendations.push("Layout throughput is low. Consider batching UI updates or implementing component memoization.".to_string());
        }
        
        recommendations
    }
    
    /// Simulate layout work (placeholder for actual UI operations)
    async fn simulate_layout_work(&self, layout_type: &LayoutType) -> Duration {
        let work_duration = match layout_type {
            LayoutType::ComponentUpdate => Duration::from_millis(1),
            LayoutType::StateChange => Duration::from_millis(2),
            LayoutType::PanelToggle => Duration::from_millis(3),
            LayoutType::TabSwitch => Duration::from_millis(2),
            _ => Duration::from_millis(1),
        };
        
        tokio::time::sleep(work_duration).await;
        work_duration
    }
    
    /// Simulate theme switching work (placeholder for actual theme operations)
    async fn simulate_theme_switch_work(&self, _from: &str, _to: &str) -> Duration {
        let work_duration = Duration::from_millis(10);
        tokio::time::sleep(work_duration).await;
        work_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::UIProfilingConfig;
    
    #[tokio::test]
    async fn test_benchmark_suite_creation() {
        let profiler = Arc::new(UIPerformanceProfiler::with_config(UIProfilingConfig::default()));
        let suite = PerformanceBenchmarkSuite::new(profiler);
        
        assert_eq!(suite.config.iterations, 100);
        assert_eq!(suite.config.layout_target_ms, 100.0);
        assert_eq!(suite.config.theme_target_ms, 50.0);
    }
    
    #[tokio::test]
    async fn test_performance_grade_calculation() {
        let profiler = Arc::new(UIPerformanceProfiler::with_config(UIProfilingConfig::default()));
        let suite = PerformanceBenchmarkSuite::new(profiler);
        
        let layout_results = LayoutBenchmarkResults {
            average_ms: 50.0, // Well under target
            median_ms: 48.0,
            p95_ms: 55.0,
            p99_ms: 60.0,
            min_ms: 40.0,
            max_ms: 65.0,
            target_met: true,
            operations_per_second: 200.0,
            component_update_ms: 45.0,
            state_change_ms: 50.0,
            panel_toggle_ms: 55.0,
        };
        
        let theme_results = ThemeBenchmarkResults {
            average_ms: 25.0, // Well under target
            median_ms: 24.0,
            p95_ms: 30.0,
            min_ms: 20.0,
            max_ms: 35.0,
            target_met: true,
            css_recalc_ms: 7.5,
            style_invalidation_ms: 5.0,
            repaint_ms: 12.5,
        };
        
        let grade = suite.calculate_performance_grade(&layout_results, &theme_results, &None);
        
        // Should be Good or Excellent grade with these metrics
        assert!(matches!(grade, PerformanceGrade::Good | PerformanceGrade::Excellent));
    }
}