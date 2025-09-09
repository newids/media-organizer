use std::time::{Duration, Instant};
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Performance measurement and optimization utilities for layout state management
/// Tracks update timing, signal efficiency, and provides optimization recommendations

const PERFORMANCE_SAMPLE_SIZE: usize = 100;
const TARGET_UPDATE_TIME_MS: u64 = 100;
const WARNING_UPDATE_TIME_MS: u64 = 80;
const CRITICAL_UPDATE_TIME_MS: u64 = 150;

/// Performance metrics collector for layout state operations
#[derive(Debug, Clone)]
pub struct PerformanceProfiler {
    /// Recent update timings for trend analysis
    update_timings: VecDeque<UpdateTiming>,
    /// Signal read/write operation counts
    signal_operations: SignalMetrics,
    /// Persistence operation metrics
    persistence_metrics: PersistenceMetrics,
    /// Batch operation efficiency tracking
    batch_metrics: BatchMetrics,
    /// Performance session start time
    session_start: Instant,
}

/// Individual update timing measurement
#[derive(Debug, Clone)]
pub struct UpdateTiming {
    /// Type of operation performed
    pub operation: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Timestamp when operation occurred
    pub timestamp: Instant,
    /// Number of affected UI components (estimated)
    pub affected_components: usize,
    /// Whether the operation triggered persistence
    pub triggered_persistence: bool,
}

/// Signal usage patterns and efficiency metrics
#[derive(Debug, Clone, Default)]
pub struct SignalMetrics {
    /// Total number of signal reads
    pub total_reads: u64,
    /// Total number of signal writes
    pub total_writes: u64,
    /// Number of unnecessary re-renders detected
    pub unnecessary_rerenders: u64,
    /// Peak concurrent signal operations
    pub peak_concurrent_ops: u64,
    /// Signal subscription count
    pub active_subscriptions: u64,
}

/// Persistence operation performance tracking
#[derive(Debug, Clone, Default)]
pub struct PersistenceMetrics {
    /// Average time for debounced saves
    pub avg_save_time: Duration,
    /// Number of saves triggered
    pub save_count: u64,
    /// Number of saves that were debounced (skipped)
    pub debounced_saves: u64,
    /// Largest save payload size
    pub max_payload_size: usize,
    /// Failed save attempts
    pub failed_saves: u64,
}

/// Batch operation efficiency metrics
#[derive(Debug, Clone, Default)]
pub struct BatchMetrics {
    /// Average batch size
    pub avg_batch_size: f64,
    /// Largest batch processed
    pub max_batch_size: usize,
    /// Number of single-operation "batches" (inefficient)
    pub single_op_batches: u64,
    /// Time saved through batching
    pub batching_time_saved: Duration,
}

/// Performance recommendations and optimization suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Overall performance grade (A-F)
    pub grade: char,
    /// Average update time across all operations
    pub avg_update_time: Duration,
    /// 95th percentile update time
    pub p95_update_time: Duration,
    /// Operations exceeding target time
    pub slow_operations: Vec<String>,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
    /// Signal efficiency score (0-100)
    pub signal_efficiency: u8,
    /// Persistence efficiency score (0-100)
    pub persistence_efficiency: u8,
    /// Memory usage estimation
    pub estimated_memory_usage: usize,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            update_timings: VecDeque::with_capacity(PERFORMANCE_SAMPLE_SIZE),
            signal_operations: SignalMetrics::default(),
            persistence_metrics: PersistenceMetrics::default(),
            batch_metrics: BatchMetrics::default(),
            session_start: Instant::now(),
        }
    }

    /// Start timing a layout operation
    pub fn start_operation(&mut self, operation: &str) -> OperationTimer {
        debug!("Starting performance measurement for operation: {}", operation);
        OperationTimer::new(operation.to_string())
    }

    /// Record completion of a layout operation
    pub fn record_operation(
        &mut self,
        timer: OperationTimer,
        affected_components: usize,
        triggered_persistence: bool,
    ) {
        let timing = timer.finish(affected_components, triggered_persistence);
        
        // Log slow operations immediately
        if timing.duration.as_millis() > WARNING_UPDATE_TIME_MS as u128 {
            warn!(
                "Slow layout operation '{}' took {}ms (target: <{}ms)",
                timing.operation,
                timing.duration.as_millis(),
                TARGET_UPDATE_TIME_MS
            );
        }

        // Store timing data
        if self.update_timings.len() >= PERFORMANCE_SAMPLE_SIZE {
            self.update_timings.pop_front();
        }
        self.update_timings.push_back(timing);
    }

    /// Record signal operation metrics
    pub fn record_signal_read(&mut self) {
        self.signal_operations.total_reads += 1;
    }

    pub fn record_signal_write(&mut self) {
        self.signal_operations.total_writes += 1;
    }

    pub fn record_unnecessary_rerender(&mut self) {
        self.signal_operations.unnecessary_rerenders += 1;
    }

    /// Record persistence operation metrics
    pub fn record_save_operation(&mut self, duration: Duration, payload_size: usize, success: bool) {
        if success {
            self.persistence_metrics.save_count += 1;
            // Update rolling average
            let current_avg_ms = self.persistence_metrics.avg_save_time.as_millis() as f64;
            let new_duration_ms = duration.as_millis() as f64;
            let count = self.persistence_metrics.save_count as f64;
            let new_avg_ms = (current_avg_ms * (count - 1.0) + new_duration_ms) / count;
            self.persistence_metrics.avg_save_time = Duration::from_millis(new_avg_ms as u64);
            
            if payload_size > self.persistence_metrics.max_payload_size {
                self.persistence_metrics.max_payload_size = payload_size;
            }
        } else {
            self.persistence_metrics.failed_saves += 1;
        }
    }

    pub fn record_debounced_save(&mut self) {
        self.persistence_metrics.debounced_saves += 1;
    }

    /// Record batch operation metrics
    pub fn record_batch_operation(&mut self, batch_size: usize, time_saved: Duration) {
        if batch_size == 1 {
            self.batch_metrics.single_op_batches += 1;
        }

        // Update average batch size
        let current_avg = self.batch_metrics.avg_batch_size;
        let total_batches = self.get_total_batches();
        self.batch_metrics.avg_batch_size = 
            (current_avg * (total_batches - 1) as f64 + batch_size as f64) / total_batches as f64;

        if batch_size > self.batch_metrics.max_batch_size {
            self.batch_metrics.max_batch_size = batch_size;
        }

        self.batch_metrics.batching_time_saved += time_saved;
    }

    /// Generate comprehensive performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let timings: Vec<&UpdateTiming> = self.update_timings.iter().collect();
        
        if timings.is_empty() {
            return PerformanceReport {
                grade: 'C',
                avg_update_time: Duration::ZERO,
                p95_update_time: Duration::ZERO,
                slow_operations: vec![],
                recommendations: vec!["No performance data available yet.".to_string()],
                signal_efficiency: 50,
                persistence_efficiency: 50,
                estimated_memory_usage: 0,
            };
        }

        // Calculate timing statistics
        let mut durations: Vec<Duration> = timings.iter().map(|t| t.duration).collect();
        durations.sort();

        let avg_duration = Duration::from_nanos(
            (durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128) as u64
        );

        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p95_duration = durations.get(p95_index).copied().unwrap_or(Duration::ZERO);

        // Identify slow operations
        let slow_operations: Vec<String> = timings.iter()
            .filter(|t| t.duration.as_millis() > TARGET_UPDATE_TIME_MS as u128)
            .map(|t| t.operation.clone())
            .collect();

        // Calculate efficiency scores
        let signal_efficiency = self.calculate_signal_efficiency();
        let persistence_efficiency = self.calculate_persistence_efficiency();

        // Determine grade based on performance
        let grade = self.calculate_performance_grade(avg_duration, &slow_operations, signal_efficiency);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            avg_duration,
            &slow_operations,
            signal_efficiency,
            persistence_efficiency,
        );

        // Estimate memory usage
        let estimated_memory_usage = self.estimate_memory_usage();

        PerformanceReport {
            grade,
            avg_update_time: avg_duration,
            p95_update_time: p95_duration,
            slow_operations,
            recommendations,
            signal_efficiency,
            persistence_efficiency,
            estimated_memory_usage,
        }
    }

    /// Get real-time performance status
    pub fn get_current_status(&self) -> PerformanceStatus {
        let recent_timings: Vec<&UpdateTiming> = self.update_timings
            .iter()
            .rev()
            .take(10)
            .collect();

        if recent_timings.is_empty() {
            return PerformanceStatus::Unknown;
        }

        let avg_recent = Duration::from_nanos(
            (recent_timings.iter().map(|t| t.duration.as_nanos()).sum::<u128>() / recent_timings.len() as u128) as u64
        );

        match avg_recent.as_millis() as u64 {
            0..=50 => PerformanceStatus::Excellent,
            51..=TARGET_UPDATE_TIME_MS => PerformanceStatus::Good,
            101..=CRITICAL_UPDATE_TIME_MS => PerformanceStatus::Warning,
            _ => PerformanceStatus::Critical,
        }
    }

    /// Check if performance optimization is needed
    pub fn needs_optimization(&self) -> bool {
        matches!(self.get_current_status(), PerformanceStatus::Warning | PerformanceStatus::Critical)
    }

    /// Get the most problematic operations
    pub fn get_bottlenecks(&self) -> Vec<String> {
        let mut operation_times: std::collections::HashMap<String, Vec<Duration>> = 
            std::collections::HashMap::new();

        for timing in &self.update_timings {
            operation_times.entry(timing.operation.clone())
                .or_insert_with(Vec::new)
                .push(timing.duration);
        }

        let mut bottlenecks: Vec<(String, Duration)> = operation_times
            .into_iter()
            .map(|(op, durations)| {
                let avg = Duration::from_nanos(
                    (durations.iter().map(|d| d.as_nanos()).sum::<u128>() / durations.len() as u128) as u64
                );
                (op, avg)
            })
            .filter(|(_, avg)| avg.as_millis() as u64 > TARGET_UPDATE_TIME_MS)
            .collect();

        bottlenecks.sort_by_key(|(_, avg)| avg.as_millis());
        bottlenecks.reverse();

        bottlenecks.into_iter().map(|(op, _)| op).take(5).collect()
    }

    // Private helper methods

    fn calculate_signal_efficiency(&self) -> u8 {
        let total_ops = self.signal_operations.total_reads + self.signal_operations.total_writes;
        if total_ops == 0 {
            return 100;
        }

        let inefficiency_ratio = self.signal_operations.unnecessary_rerenders as f64 / total_ops as f64;
        ((1.0 - inefficiency_ratio) * 100.0).clamp(0.0, 100.0) as u8
    }

    fn calculate_persistence_efficiency(&self) -> u8 {
        let total_saves = self.persistence_metrics.save_count + self.persistence_metrics.debounced_saves;
        if total_saves == 0 {
            return 100;
        }

        let debounce_ratio = self.persistence_metrics.debounced_saves as f64 / total_saves as f64;
        let failure_ratio = self.persistence_metrics.failed_saves as f64 / self.persistence_metrics.save_count.max(1) as f64;
        
        let efficiency = (debounce_ratio * 0.7 + (1.0 - failure_ratio) * 0.3) * 100.0;
        efficiency.clamp(0.0, 100.0) as u8
    }

    fn calculate_performance_grade(&self, avg_duration: Duration, slow_operations: &[String], signal_efficiency: u8) -> char {
        let time_score = match avg_duration.as_millis() {
            0..=30 => 4,      // A
            31..=60 => 3,     // B
            61..=100 => 2,    // C
            101..=150 => 1,   // D
            _ => 0,           // F
        };

        let slow_penalty = slow_operations.len().min(2);
        let efficiency_bonus = if signal_efficiency > 80 { 1 } else { 0 };

        let final_score = (time_score as i32 + efficiency_bonus as i32).saturating_sub(slow_penalty as i32);

        match final_score {
            4.. => 'A',
            3 => 'B',
            2 => 'C',
            1 => 'D',
            _ => 'F',
        }
    }

    fn generate_recommendations(&self, avg_duration: Duration, slow_operations: &[String], signal_efficiency: u8, persistence_efficiency: u8) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Performance recommendations
        if avg_duration.as_millis() > TARGET_UPDATE_TIME_MS as u128 {
            recommendations.push("Consider batching layout updates to reduce overhead.".to_string());
            recommendations.push("Profile individual operations to identify bottlenecks.".to_string());
        }

        // Slow operation recommendations
        if !slow_operations.is_empty() {
            recommendations.push(format!("Optimize these slow operations: {}", slow_operations.join(", ")));
        }

        // Signal efficiency recommendations
        if signal_efficiency < 70 {
            recommendations.push("Reduce unnecessary re-renders by optimizing signal usage.".to_string());
            recommendations.push("Consider using more granular signals for better performance.".to_string());
        }

        // Persistence efficiency recommendations
        if persistence_efficiency < 70 {
            recommendations.push("Increase debounce delay to reduce persistence overhead.".to_string());
            recommendations.push("Review save failure patterns and improve error handling.".to_string());
        }

        // Batch operation recommendations
        if self.batch_metrics.single_op_batches > 10 {
            recommendations.push("Increase use of batch operations instead of individual updates.".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is good! Continue monitoring for any regressions.".to_string());
        }

        recommendations
    }

    fn estimate_memory_usage(&self) -> usize {
        // Rough estimation based on data structures
        let timings_size = self.update_timings.len() * std::mem::size_of::<UpdateTiming>();
        let metrics_size = std::mem::size_of::<SignalMetrics>() + 
                          std::mem::size_of::<PersistenceMetrics>() + 
                          std::mem::size_of::<BatchMetrics>();
        
        timings_size + metrics_size + 1024 // Base overhead
    }

    fn get_total_batches(&self) -> usize {
        // Estimate total batches from metrics
        (self.batch_metrics.single_op_batches + 
         (self.update_timings.len() / 3).max(1) as u64).try_into().unwrap_or(1)
    }
}

/// Operation timer for measuring individual operation performance
pub struct OperationTimer {
    operation: String,
    start_time: Instant,
}

impl OperationTimer {
    fn new(operation: String) -> Self {
        Self {
            operation,
            start_time: Instant::now(),
        }
    }

    fn finish(self, affected_components: usize, triggered_persistence: bool) -> UpdateTiming {
        let duration = self.start_time.elapsed();
        UpdateTiming {
            operation: self.operation,
            duration,
            timestamp: self.start_time,
            affected_components,
            triggered_persistence,
        }
    }
}

/// Current performance status classification
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceStatus {
    Excellent,  // < 50ms
    Good,       // 50-100ms
    Warning,    // 100-150ms  
    Critical,   // > 150ms
    Unknown,    // No data
}

impl PerformanceStatus {
    pub fn color(&self) -> &'static str {
        match self {
            PerformanceStatus::Excellent => "green",
            PerformanceStatus::Good => "blue", 
            PerformanceStatus::Warning => "yellow",
            PerformanceStatus::Critical => "red",
            PerformanceStatus::Unknown => "gray",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PerformanceStatus::Excellent => "Excellent performance",
            PerformanceStatus::Good => "Good performance",
            PerformanceStatus::Warning => "Performance needs attention",
            PerformanceStatus::Critical => "Critical performance issues",
            PerformanceStatus::Unknown => "Performance data unavailable",
        }
    }
}

use std::sync::{Mutex, OnceLock};

/// Global performance profiler instance using thread-safe OnceLock
static GLOBAL_PROFILER: OnceLock<Mutex<PerformanceProfiler>> = OnceLock::new();

/// Initialize the global performance profiler
pub fn init_profiler() {
    GLOBAL_PROFILER.get_or_init(|| {
        info!("Performance profiler initialized");
        Mutex::new(PerformanceProfiler::new())
    });
}

/// Get a reference to the global profiler
pub fn with_profiler<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PerformanceProfiler) -> R,
{
    GLOBAL_PROFILER
        .get()
        .and_then(|profiler| profiler.lock().ok())
        .map(|mut profiler| f(&mut *profiler))
}

/// Convenience macro for measuring operation performance
#[macro_export]
macro_rules! measure_performance {
    ($operation:expr, $components:expr, $persistence:expr, $code:block) => {{
        use $crate::state::performance::{with_profiler};
        
        let timer = with_profiler(|profiler| {
            profiler.start_operation($operation)
        });
        
        let result = $code;
        
        if let Some(timer) = timer {
            with_profiler(|profiler| {
                profiler.record_operation(timer, $components, $persistence);
            });
        }
        
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_profiler_creation() {
        let profiler = PerformanceProfiler::new();
        assert_eq!(profiler.update_timings.len(), 0);
        assert_eq!(profiler.signal_operations.total_reads, 0);
    }

    #[test]
    fn test_operation_timing() {
        let mut profiler = PerformanceProfiler::new();
        let timer = profiler.start_operation("test_operation");
        
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        profiler.record_operation(timer, 5, true);
        
        assert_eq!(profiler.update_timings.len(), 1);
        let timing = &profiler.update_timings[0];
        assert_eq!(timing.operation, "test_operation");
        assert!(timing.duration.as_millis() >= 10);
        assert_eq!(timing.affected_components, 5);
        assert!(timing.triggered_persistence);
    }

    #[test]
    fn test_signal_metrics() {
        let mut profiler = PerformanceProfiler::new();
        
        profiler.record_signal_read();
        profiler.record_signal_read();
        profiler.record_signal_write();
        profiler.record_unnecessary_rerender();
        
        assert_eq!(profiler.signal_operations.total_reads, 2);
        assert_eq!(profiler.signal_operations.total_writes, 1);
        assert_eq!(profiler.signal_operations.unnecessary_rerenders, 1);
    }

    #[test]
    fn test_persistence_metrics() {
        let mut profiler = PerformanceProfiler::new();
        
        profiler.record_save_operation(Duration::from_millis(50), 1024, true);
        profiler.record_save_operation(Duration::from_millis(30), 512, true);
        profiler.record_debounced_save();
        
        assert_eq!(profiler.persistence_metrics.save_count, 2);
        assert_eq!(profiler.persistence_metrics.debounced_saves, 1);
        assert_eq!(profiler.persistence_metrics.max_payload_size, 1024);
        assert_eq!(profiler.persistence_metrics.avg_save_time.as_millis(), 40);
    }

    #[test]
    fn test_performance_status() {
        let mut profiler = PerformanceProfiler::new();
        
        // Test with no data
        assert_eq!(profiler.get_current_status(), PerformanceStatus::Unknown);
        
        // Add fast operation
        let timer = profiler.start_operation("fast_op");
        profiler.record_operation(timer, 1, false);
        
        assert_eq!(profiler.get_current_status(), PerformanceStatus::Excellent);
    }

    #[test]
    fn test_performance_report_generation() {
        let mut profiler = PerformanceProfiler::new();
        
        // Add some sample data
        for i in 0..5 {
            let timer = profiler.start_operation(&format!("operation_{}", i));
            thread::sleep(Duration::from_millis(i * 10));
            profiler.record_operation(timer, i as usize, i % 2 == 0);
        }
        
        let report = profiler.generate_report();
        
        assert!(!matches!(report.grade, 'F'));
        assert!(report.avg_update_time > Duration::ZERO);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_bottleneck_identification() {
        let mut profiler = PerformanceProfiler::new();
        
        // Add slow operation multiple times
        for _ in 0..3 {
            let timer = profiler.start_operation("slow_operation");
            thread::sleep(Duration::from_millis(110)); // Exceeds target
            profiler.record_operation(timer, 10, true);
        }
        
        // Add fast operation
        let timer = profiler.start_operation("fast_operation");
        profiler.record_operation(timer, 1, false);
        
        let bottlenecks = profiler.get_bottlenecks();
        assert!(bottlenecks.contains(&"slow_operation".to_string()));
        assert!(!bottlenecks.contains(&"fast_operation".to_string()));
    }

    #[test]
    fn test_efficiency_calculations() {
        let mut profiler = PerformanceProfiler::new();
        
        // Perfect efficiency case
        profiler.signal_operations.total_reads = 100;
        profiler.signal_operations.total_writes = 50;
        profiler.signal_operations.unnecessary_rerenders = 0;
        
        assert_eq!(profiler.calculate_signal_efficiency(), 100);
        
        // Add some inefficiency
        profiler.signal_operations.unnecessary_rerenders = 15; // 10% inefficiency
        assert!(profiler.calculate_signal_efficiency() >= 85);
    }

    #[test]
    fn test_global_profiler() {
        init_profiler();
        
        let result = with_profiler(|profiler| {
            profiler.record_signal_read();
            profiler.signal_operations.total_reads
        });
        
        assert_eq!(result, Some(1));
    }

    #[test]
    fn test_performance_status_colors() {
        assert_eq!(PerformanceStatus::Excellent.color(), "green");
        assert_eq!(PerformanceStatus::Good.color(), "blue");
        assert_eq!(PerformanceStatus::Warning.color(), "yellow");
        assert_eq!(PerformanceStatus::Critical.color(), "red");
        assert_eq!(PerformanceStatus::Unknown.color(), "gray");
    }
}