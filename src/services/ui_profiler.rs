use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, debug};

/// UI Performance profiler for measuring layout, theme switching, and GPU rendering performance
/// Targets: <100ms layout, <50ms theme switch, GPU-accelerated previews
pub struct UIPerformanceProfiler {
    metrics: Arc<Mutex<UIMetrics>>,
    config: UIProfilingConfig,
    layout_benchmarks: Arc<Mutex<VecDeque<LayoutBenchmark>>>,
    theme_benchmarks: Arc<Mutex<VecDeque<ThemeBenchmark>>>,
    gpu_benchmarks: Arc<Mutex<VecDeque<GpuBenchmark>>>,
    session_start: Instant,
}

/// UI performance metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIMetrics {
    pub layout_performance: LayoutPerformanceMetrics,
    pub theme_performance: ThemePerformanceMetrics,
    pub gpu_performance: GpuPerformanceMetrics,
    pub frame_rate_metrics: FrameRateMetrics,
    pub memory_usage_ui: MemoryUsageMetrics,
    pub interaction_latency: InteractionLatencyMetrics,
}

/// Layout rendering performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPerformanceMetrics {
    pub average_layout_time_ms: f64,
    pub max_layout_time_ms: f64,
    pub layout_target_met: bool, // <100ms target
    pub layout_samples: usize,
    pub p95_layout_time_ms: f64,
    pub p99_layout_time_ms: f64,
    pub layout_operations_per_second: f64,
}

/// Theme switching performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePerformanceMetrics {
    pub average_theme_switch_ms: f64,
    pub max_theme_switch_ms: f64,
    pub theme_target_met: bool, // <50ms target
    pub theme_samples: usize,
    pub p95_theme_switch_ms: f64,
    pub css_recalculation_ms: f64,
    pub style_invalidation_ms: f64,
}

/// GPU-accelerated rendering performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuPerformanceMetrics {
    pub average_gpu_render_ms: f64,
    pub max_gpu_render_ms: f64,
    pub gpu_memory_usage_mb: f64,
    pub gpu_utilization_percent: f64,
    pub texture_upload_time_ms: f64,
    pub shader_compilation_ms: f64,
    pub command_buffer_time_ms: f64,
    pub gpu_samples: usize,
}

/// Frame rate and rendering smoothness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameRateMetrics {
    pub average_fps: f64,
    pub min_fps: f64,
    pub frame_drops: usize,
    pub vsync_misses: usize,
    pub frame_time_variance_ms: f64,
    pub target_60fps_met: bool,
}

/// UI memory usage specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    pub dom_nodes: usize,
    pub css_rules: usize,
    pub render_tree_size_mb: f64,
    pub gpu_texture_memory_mb: f64,
    pub ui_component_count: usize,
}

/// User interaction latency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionLatencyMetrics {
    pub click_response_time_ms: f64,
    pub keyboard_response_time_ms: f64,
    pub scroll_latency_ms: f64,
    pub drag_response_time_ms: f64,
    pub hover_feedback_time_ms: f64,
}

/// Individual layout benchmark record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBenchmark {
    pub timestamp: SystemTime,
    pub operation: String,
    pub duration_ms: f64,
    pub component_count: usize,
    pub dom_nodes_affected: usize,
    pub layout_type: LayoutType,
}

/// Individual theme switch benchmark record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeBenchmark {
    pub timestamp: SystemTime,
    pub from_theme: String,
    pub to_theme: String,
    pub total_duration_ms: f64,
    pub css_recalc_ms: f64,
    pub style_invalidation_ms: f64,
    pub repaint_ms: f64,
    pub affected_elements: usize,
}

/// Individual GPU rendering benchmark record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuBenchmark {
    pub timestamp: SystemTime,
    pub operation: GpuOperation,
    pub duration_ms: f64,
    pub memory_used_mb: f64,
    pub texture_size: Option<(u32, u32)>,
    pub shader_program: Option<String>,
    pub command_count: usize,
}

/// Types of layout operations being measured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    InitialLoad,
    ComponentUpdate,
    StateChange,
    WindowResize,
    TabSwitch,
    PanelToggle,
    SidebarToggle,
}

/// Types of GPU operations being measured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuOperation {
    TextureUpload,
    ShaderCompilation,
    RenderPass,
    ComputeShader,
    BufferUpdate,
    CommandSubmission,
}

/// Configuration for UI profiling
#[derive(Debug, Clone)]
pub struct UIProfilingConfig {
    /// Maximum number of benchmark samples to keep in memory
    pub max_samples: usize,
    /// Target layout time in milliseconds
    pub layout_target_ms: f64,
    /// Target theme switch time in milliseconds
    pub theme_target_ms: f64,
    /// Target frame rate (FPS)
    pub target_fps: f64,
    /// Whether to enable GPU profiling (requires GPU feature)
    pub enable_gpu_profiling: bool,
    /// Whether to profile memory usage during operations
    pub profile_memory: bool,
    /// Whether to log performance warnings
    pub log_warnings: bool,
}

impl Default for UIProfilingConfig {
    fn default() -> Self {
        Self {
            max_samples: 1000,
            layout_target_ms: 100.0,
            theme_target_ms: 50.0,
            target_fps: 60.0,
            enable_gpu_profiling: true,
            profile_memory: true,
            log_warnings: true,
        }
    }
}

impl Default for UIMetrics {
    fn default() -> Self {
        Self {
            layout_performance: LayoutPerformanceMetrics::default(),
            theme_performance: ThemePerformanceMetrics::default(),
            gpu_performance: GpuPerformanceMetrics::default(),
            frame_rate_metrics: FrameRateMetrics::default(),
            memory_usage_ui: MemoryUsageMetrics::default(),
            interaction_latency: InteractionLatencyMetrics::default(),
        }
    }
}

impl Default for LayoutPerformanceMetrics {
    fn default() -> Self {
        Self {
            average_layout_time_ms: 0.0,
            max_layout_time_ms: 0.0,
            layout_target_met: true,
            layout_samples: 0,
            p95_layout_time_ms: 0.0,
            p99_layout_time_ms: 0.0,
            layout_operations_per_second: 0.0,
        }
    }
}

impl Default for ThemePerformanceMetrics {
    fn default() -> Self {
        Self {
            average_theme_switch_ms: 0.0,
            max_theme_switch_ms: 0.0,
            theme_target_met: true,
            theme_samples: 0,
            p95_theme_switch_ms: 0.0,
            css_recalculation_ms: 0.0,
            style_invalidation_ms: 0.0,
        }
    }
}

impl Default for GpuPerformanceMetrics {
    fn default() -> Self {
        Self {
            average_gpu_render_ms: 0.0,
            max_gpu_render_ms: 0.0,
            gpu_memory_usage_mb: 0.0,
            gpu_utilization_percent: 0.0,
            texture_upload_time_ms: 0.0,
            shader_compilation_ms: 0.0,
            command_buffer_time_ms: 0.0,
            gpu_samples: 0,
        }
    }
}

impl Default for FrameRateMetrics {
    fn default() -> Self {
        Self {
            average_fps: 0.0,
            min_fps: 60.0,
            frame_drops: 0,
            vsync_misses: 0,
            frame_time_variance_ms: 0.0,
            target_60fps_met: true,
        }
    }
}

impl Default for MemoryUsageMetrics {
    fn default() -> Self {
        Self {
            dom_nodes: 0,
            css_rules: 0,
            render_tree_size_mb: 0.0,
            gpu_texture_memory_mb: 0.0,
            ui_component_count: 0,
        }
    }
}

impl Default for InteractionLatencyMetrics {
    fn default() -> Self {
        Self {
            click_response_time_ms: 0.0,
            keyboard_response_time_ms: 0.0,
            scroll_latency_ms: 0.0,
            drag_response_time_ms: 0.0,
            hover_feedback_time_ms: 0.0,
        }
    }
}

impl UIPerformanceProfiler {
    /// Create new UI performance profiler with default configuration
    pub fn new() -> Self {
        Self::with_config(UIProfilingConfig::default())
    }
    
    /// Create UI performance profiler with custom configuration
    pub fn with_config(config: UIProfilingConfig) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(UIMetrics::default())),
            config,
            layout_benchmarks: Arc::new(Mutex::new(VecDeque::new())),
            theme_benchmarks: Arc::new(Mutex::new(VecDeque::new())),
            gpu_benchmarks: Arc::new(Mutex::new(VecDeque::new())),
            session_start: Instant::now(),
        }
    }
    
    /// Start measuring a layout operation
    pub fn start_layout_measurement(&self, operation: String, layout_type: LayoutType) -> LayoutMeasurement {
        LayoutMeasurement {
            profiler: self,
            operation,
            layout_type,
            start_time: Instant::now(),
            component_count: self.estimate_component_count(),
        }
    }
    
    /// Start measuring a theme switch operation
    pub fn start_theme_measurement(&self, from_theme: String, to_theme: String) -> ThemeMeasurement {
        ThemeMeasurement {
            profiler: self,
            from_theme,
            to_theme,
            start_time: Instant::now(),
            css_recalc_start: None,
            style_invalidation_start: None,
        }
    }
    
    /// Start measuring a GPU operation
    pub fn start_gpu_measurement(&self, operation: GpuOperation) -> GpuMeasurement {
        GpuMeasurement {
            profiler: self,
            operation,
            start_time: Instant::now(),
            memory_before: self.get_gpu_memory_usage(),
        }
    }
    
    /// Record a completed layout benchmark
    fn record_layout_benchmark(&self, benchmark: LayoutBenchmark) {
        // Add to rolling window
        {
            let mut benchmarks = self.layout_benchmarks.lock().unwrap();
            benchmarks.push_back(benchmark.clone());
            
            // Keep only the most recent samples
            while benchmarks.len() > self.config.max_samples {
                benchmarks.pop_front();
            }
        }
        
        // Update metrics
        self.update_layout_metrics();
        
        // Log warnings if performance targets are missed
        if self.config.log_warnings && benchmark.duration_ms > self.config.layout_target_ms {
            warn!(
                "Layout operation '{}' took {:.2}ms, exceeding target of {:.2}ms", 
                benchmark.operation, 
                benchmark.duration_ms, 
                self.config.layout_target_ms
            );
        }
    }
    
    /// Record a completed theme switch benchmark
    fn record_theme_benchmark(&self, benchmark: ThemeBenchmark) {
        // Add to rolling window
        {
            let mut benchmarks = self.theme_benchmarks.lock().unwrap();
            benchmarks.push_back(benchmark.clone());
            
            // Keep only the most recent samples
            while benchmarks.len() > self.config.max_samples {
                benchmarks.pop_front();
            }
        }
        
        // Update metrics
        self.update_theme_metrics();
        
        // Log warnings if performance targets are missed
        if self.config.log_warnings && benchmark.total_duration_ms > self.config.theme_target_ms {
            warn!(
                "Theme switch from '{}' to '{}' took {:.2}ms, exceeding target of {:.2}ms", 
                benchmark.from_theme, 
                benchmark.to_theme,
                benchmark.total_duration_ms, 
                self.config.theme_target_ms
            );
        }
    }
    
    /// Record a completed GPU benchmark
    fn record_gpu_benchmark(&self, benchmark: GpuBenchmark) {
        // Add to rolling window
        {
            let mut benchmarks = self.gpu_benchmarks.lock().unwrap();
            benchmarks.push_back(benchmark.clone());
            
            // Keep only the most recent samples
            while benchmarks.len() > self.config.max_samples {
                benchmarks.pop_front();
            }
        }
        
        // Update metrics
        self.update_gpu_metrics();
        
        debug!(
            "GPU operation {:?} completed in {:.2}ms using {:.2}MB memory",
            benchmark.operation,
            benchmark.duration_ms,
            benchmark.memory_used_mb
        );
    }
    
    /// Update layout performance metrics from recent benchmarks
    fn update_layout_metrics(&self) {
        let benchmarks = self.layout_benchmarks.lock().unwrap();
        if benchmarks.is_empty() {
            return;
        }
        
        let durations: Vec<f64> = benchmarks.iter().map(|b| b.duration_ms).collect();
        let sum: f64 = durations.iter().sum();
        let count = durations.len();
        
        let mut sorted_durations = durations.clone();
        sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let average = sum / count as f64;
        let max = sorted_durations.last().copied().unwrap_or(0.0);
        let p95_index = ((count as f64 * 0.95) as usize).min(count - 1);
        let p99_index = ((count as f64 * 0.99) as usize).min(count - 1);
        let p95 = sorted_durations.get(p95_index).copied().unwrap_or(0.0);
        let p99 = sorted_durations.get(p99_index).copied().unwrap_or(0.0);
        
        let session_duration = self.session_start.elapsed().as_secs_f64();
        let ops_per_second = if session_duration > 0.0 { count as f64 / session_duration } else { 0.0 };
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.layout_performance = LayoutPerformanceMetrics {
            average_layout_time_ms: average,
            max_layout_time_ms: max,
            layout_target_met: average <= self.config.layout_target_ms,
            layout_samples: count,
            p95_layout_time_ms: p95,
            p99_layout_time_ms: p99,
            layout_operations_per_second: ops_per_second,
        };
    }
    
    /// Update theme performance metrics from recent benchmarks  
    fn update_theme_metrics(&self) {
        let benchmarks = self.theme_benchmarks.lock().unwrap();
        if benchmarks.is_empty() {
            return;
        }
        
        let durations: Vec<f64> = benchmarks.iter().map(|b| b.total_duration_ms).collect();
        let css_times: Vec<f64> = benchmarks.iter().map(|b| b.css_recalc_ms).collect();
        let style_times: Vec<f64> = benchmarks.iter().map(|b| b.style_invalidation_ms).collect();
        
        let sum: f64 = durations.iter().sum();
        let count = durations.len();
        
        let mut sorted_durations = durations.clone();
        sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let average = sum / count as f64;
        let max = sorted_durations.last().copied().unwrap_or(0.0);
        let p95_index = ((count as f64 * 0.95) as usize).min(count - 1);
        let p95 = sorted_durations.get(p95_index).copied().unwrap_or(0.0);
        
        let avg_css_recalc: f64 = css_times.iter().sum::<f64>() / count as f64;
        let avg_style_invalidation: f64 = style_times.iter().sum::<f64>() / count as f64;
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.theme_performance = ThemePerformanceMetrics {
            average_theme_switch_ms: average,
            max_theme_switch_ms: max,
            theme_target_met: average <= self.config.theme_target_ms,
            theme_samples: count,
            p95_theme_switch_ms: p95,
            css_recalculation_ms: avg_css_recalc,
            style_invalidation_ms: avg_style_invalidation,
        };
    }
    
    /// Update GPU performance metrics from recent benchmarks
    fn update_gpu_metrics(&self) {
        let benchmarks = self.gpu_benchmarks.lock().unwrap();
        if benchmarks.is_empty() {
            return;
        }
        
        let durations: Vec<f64> = benchmarks.iter().map(|b| b.duration_ms).collect();
        let memory_usage: Vec<f64> = benchmarks.iter().map(|b| b.memory_used_mb).collect();
        
        let sum: f64 = durations.iter().sum();
        let count = durations.len();
        
        let average_duration = sum / count as f64;
        let max_duration = durations.iter().fold(0.0f64, |a, &b| a.max(b));
        let average_memory: f64 = memory_usage.iter().sum::<f64>() / count as f64;
        
        // Calculate averages for specific operation types
        let texture_times: Vec<f64> = benchmarks.iter()
            .filter(|b| matches!(b.operation, GpuOperation::TextureUpload))
            .map(|b| b.duration_ms)
            .collect();
        let shader_times: Vec<f64> = benchmarks.iter()
            .filter(|b| matches!(b.operation, GpuOperation::ShaderCompilation))
            .map(|b| b.duration_ms)
            .collect();
        let command_times: Vec<f64> = benchmarks.iter()
            .filter(|b| matches!(b.operation, GpuOperation::CommandSubmission))
            .map(|b| b.duration_ms)
            .collect();
        
        let avg_texture_upload = if !texture_times.is_empty() {
            texture_times.iter().sum::<f64>() / texture_times.len() as f64
        } else { 0.0 };
        
        let avg_shader_compilation = if !shader_times.is_empty() {
            shader_times.iter().sum::<f64>() / shader_times.len() as f64
        } else { 0.0 };
        
        let avg_command_buffer = if !command_times.is_empty() {
            command_times.iter().sum::<f64>() / command_times.len() as f64
        } else { 0.0 };
        
        let mut metrics = self.metrics.lock().unwrap();
        metrics.gpu_performance = GpuPerformanceMetrics {
            average_gpu_render_ms: average_duration,
            max_gpu_render_ms: max_duration,
            gpu_memory_usage_mb: average_memory,
            gpu_utilization_percent: self.get_gpu_utilization(),
            texture_upload_time_ms: avg_texture_upload,
            shader_compilation_ms: avg_shader_compilation,
            command_buffer_time_ms: avg_command_buffer,
            gpu_samples: count,
        };
    }
    
    /// Get current UI performance metrics snapshot
    pub fn get_metrics(&self) -> UIMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    /// Get current GPU memory usage (placeholder implementation)
    fn get_gpu_memory_usage(&self) -> f64 {
        // This would require actual GPU API integration
        // For now, return a placeholder value
        0.0
    }
    
    /// Get current GPU utilization percentage (placeholder implementation)
    fn get_gpu_utilization(&self) -> f64 {
        // This would require actual GPU monitoring
        // For now, return a placeholder value
        0.0
    }
    
    /// Estimate current UI component count
    fn estimate_component_count(&self) -> usize {
        // This would integrate with Dioxus internals to count active components
        // For now, return a placeholder value
        100
    }
    
    /// Generate comprehensive performance report
    pub fn generate_performance_report(&self) -> UIPerformanceReport {
        let metrics = self.get_metrics();
        let layout_benchmarks = self.layout_benchmarks.lock().unwrap().clone().into();
        let theme_benchmarks = self.theme_benchmarks.lock().unwrap().clone().into();
        let gpu_benchmarks = self.gpu_benchmarks.lock().unwrap().clone().into();
        
        UIPerformanceReport {
            timestamp: SystemTime::now(),
            session_duration: self.session_start.elapsed(),
            metrics,
            layout_benchmarks,
            theme_benchmarks,
            gpu_benchmarks,
            recommendations: self.generate_recommendations(),
        }
    }
    
    /// Generate performance optimization recommendations
    fn generate_recommendations(&self) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        let metrics = self.get_metrics();
        
        // Layout performance recommendations
        if !metrics.layout_performance.layout_target_met {
            recommendations.push(PerformanceRecommendation {
                category: "Layout Performance".to_string(),
                severity: RecommendationSeverity::High,
                description: format!(
                    "Average layout time {:.1}ms exceeds target of {:.1}ms",
                    metrics.layout_performance.average_layout_time_ms,
                    self.config.layout_target_ms
                ),
                solution: "Consider virtualization for large lists, reduce DOM complexity, or optimize component re-renders".to_string(),
                estimated_impact: "10-30% layout performance improvement".to_string(),
            });
        }
        
        // Theme switching recommendations
        if !metrics.theme_performance.theme_target_met {
            recommendations.push(PerformanceRecommendation {
                category: "Theme Performance".to_string(),
                severity: RecommendationSeverity::Medium,
                description: format!(
                    "Average theme switch time {:.1}ms exceeds target of {:.1}ms",
                    metrics.theme_performance.average_theme_switch_ms,
                    self.config.theme_target_ms
                ),
                solution: "Optimize CSS transitions, reduce style recalculations, or implement CSS custom properties".to_string(),
                estimated_impact: "20-40% theme switching improvement".to_string(),
            });
        }
        
        // GPU performance recommendations
        if metrics.gpu_performance.average_gpu_render_ms > 16.67 { // 60 FPS target
            recommendations.push(PerformanceRecommendation {
                category: "GPU Performance".to_string(),
                severity: RecommendationSeverity::High,
                description: format!(
                    "GPU render time {:.1}ms may impact 60 FPS target",
                    metrics.gpu_performance.average_gpu_render_ms
                ),
                solution: "Optimize shaders, reduce texture sizes, or implement GPU command batching".to_string(),
                estimated_impact: "Maintain 60 FPS rendering".to_string(),
            });
        }
        
        recommendations
    }
}

/// Layout measurement handle for tracking layout operations
pub struct LayoutMeasurement<'a> {
    profiler: &'a UIPerformanceProfiler,
    operation: String,
    layout_type: LayoutType,
    start_time: Instant,
    component_count: usize,
}

impl<'a> LayoutMeasurement<'a> {
    /// Complete the layout measurement
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        let benchmark = LayoutBenchmark {
            timestamp: SystemTime::now(),
            operation: self.operation,
            duration_ms: duration.as_millis() as f64,
            component_count: self.component_count,
            dom_nodes_affected: self.estimate_dom_nodes_affected(),
            layout_type: self.layout_type,
        };
        
        self.profiler.record_layout_benchmark(benchmark);
    }
    
    /// Estimate DOM nodes affected by this layout operation
    fn estimate_dom_nodes_affected(&self) -> usize {
        // This would ideally integrate with the actual DOM mutation observer
        // For now, estimate based on component count
        self.component_count * 3 // rough estimate of nodes per component
    }
}

/// Theme switch measurement handle for tracking theme operations
pub struct ThemeMeasurement<'a> {
    profiler: &'a UIPerformanceProfiler,
    from_theme: String,
    to_theme: String,
    start_time: Instant,
    css_recalc_start: Option<Instant>,
    style_invalidation_start: Option<Instant>,
}

impl<'a> ThemeMeasurement<'a> {
    /// Mark the start of CSS recalculation phase
    pub fn mark_css_recalc_start(&mut self) {
        self.css_recalc_start = Some(Instant::now());
    }
    
    /// Mark the start of style invalidation phase
    pub fn mark_style_invalidation_start(&mut self) {
        self.style_invalidation_start = Some(Instant::now());
    }
    
    /// Complete the theme switch measurement
    pub fn finish(self) {
        let total_duration = self.start_time.elapsed();
        
        // Calculate phase durations (simplified - would need browser integration for accuracy)
        let css_recalc_duration = self.css_recalc_start
            .map(|start| start.elapsed().as_millis() as f64)
            .unwrap_or(total_duration.as_millis() as f64 * 0.3); // 30% estimate
        
        let style_invalidation_duration = self.style_invalidation_start
            .map(|start| start.elapsed().as_millis() as f64)
            .unwrap_or(total_duration.as_millis() as f64 * 0.2); // 20% estimate
        
        let repaint_duration = total_duration.as_millis() as f64 - css_recalc_duration - style_invalidation_duration;
        
        let benchmark = ThemeBenchmark {
            timestamp: SystemTime::now(),
            from_theme: self.from_theme,
            to_theme: self.to_theme,
            total_duration_ms: total_duration.as_millis() as f64,
            css_recalc_ms: css_recalc_duration,
            style_invalidation_ms: style_invalidation_duration,
            repaint_ms: repaint_duration.max(0.0),
            affected_elements: self.estimate_affected_elements(),
        };
        
        self.profiler.record_theme_benchmark(benchmark);
    }
    
    /// Estimate elements affected by theme change
    fn estimate_affected_elements(&self) -> usize {
        // This would ideally query the actual DOM
        // For now, estimate based on typical UI complexity
        500 // rough estimate for a complete theme change
    }
}

/// GPU operation measurement handle for tracking GPU operations
pub struct GpuMeasurement<'a> {
    profiler: &'a UIPerformanceProfiler,
    operation: GpuOperation,
    start_time: Instant,
    memory_before: f64,
}

impl<'a> GpuMeasurement<'a> {
    /// Complete the GPU measurement
    pub fn finish(self) {
        let duration = self.start_time.elapsed();
        let memory_after = self.profiler.get_gpu_memory_usage();
        let memory_used = (memory_after - self.memory_before).max(0.0);
        
        let benchmark = GpuBenchmark {
            timestamp: SystemTime::now(),
            operation: self.operation,
            duration_ms: duration.as_millis() as f64,
            memory_used_mb: memory_used,
            texture_size: None, // Would be set by specific GPU operations
            shader_program: None, // Would be set by shader operations
            command_count: 1, // Would be counted by actual GPU operations
        };
        
        self.profiler.record_gpu_benchmark(benchmark);
    }
}

/// Comprehensive UI performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPerformanceReport {
    pub timestamp: SystemTime,
    pub session_duration: Duration,
    pub metrics: UIMetrics,
    pub layout_benchmarks: Vec<LayoutBenchmark>,
    pub theme_benchmarks: Vec<ThemeBenchmark>,
    pub gpu_benchmarks: Vec<GpuBenchmark>,
    pub recommendations: Vec<PerformanceRecommendation>,
}

/// Performance optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: String,
    pub severity: RecommendationSeverity,
    pub description: String,
    pub solution: String,
    pub estimated_impact: String,
}

/// Severity levels for performance recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_profiler_creation() {
        let profiler = UIPerformanceProfiler::new();
        let metrics = profiler.get_metrics();
        
        assert_eq!(metrics.layout_performance.layout_samples, 0);
        assert_eq!(metrics.theme_performance.theme_samples, 0);
        assert_eq!(metrics.gpu_performance.gpu_samples, 0);
    }
    
    #[test]
    fn test_layout_measurement() {
        let profiler = UIPerformanceProfiler::new();
        
        let measurement = profiler.start_layout_measurement(
            "test_layout".to_string(),
            LayoutType::ComponentUpdate
        );
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        measurement.finish();
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.layout_performance.layout_samples, 1);
        assert!(metrics.layout_performance.average_layout_time_ms > 0.0);
    }
    
    #[test]
    fn test_theme_measurement() {
        let profiler = UIPerformanceProfiler::new();
        
        let measurement = profiler.start_theme_measurement(
            "dark".to_string(),
            "light".to_string()
        );
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        measurement.finish();
        
        let metrics = profiler.get_metrics();
        assert_eq!(metrics.theme_performance.theme_samples, 1);
        assert!(metrics.theme_performance.average_theme_switch_ms > 0.0);
    }
}