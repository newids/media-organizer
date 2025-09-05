use std::sync::Arc;
// use std::path::Path;  // Currently unused
use tempfile::TempDir;
use tokio::time::Instant;
use tracing::{info, warn, error};
use serde_json;

use crate::services::{
    ValidationProfiler, ValidationConfig, ValidationReport, ValidationError,
    PreviewService, PreviewServiceConfig,
    file_system::{NativeFileSystemService, FileSystemConfig}
};

/// Comprehensive validation runner for the preview system
pub struct ValidationRunner;

impl ValidationRunner {
    /// Run complete validation suite and return detailed report
    pub async fn run_comprehensive_validation() -> Result<ValidationSummary, ValidationError> {
        info!("🧪 Starting comprehensive preview system validation");
        let overall_start = Instant::now();
        
        // Create temporary directory for test files
        let temp_dir = TempDir::new().map_err(|e| ValidationError::Io(e))?;
        info!("📁 Created test directory: {}", temp_dir.path().display());
        
        // Set up services with validation-optimized configuration
        let (preview_service, config) = Self::create_validation_services().await?;
        
        // Create validation profiler
        let mut profiler = ValidationProfiler::new(preview_service, config.clone());
        
        // Run validation tests
        info!("🔍 Running validation tests...");
        let validation_report = profiler.run_full_validation(temp_dir.path()).await?;
        
        // Generate summary
        let summary = Self::generate_validation_summary(&validation_report, overall_start.elapsed());
        
        // Log results
        Self::log_validation_results(&summary);
        
        // Save detailed report (optional)
        if let Err(e) = Self::save_validation_report(&validation_report).await {
            warn!("Failed to save detailed validation report: {}", e);
        }
        
        info!("✅ Validation completed in {:.2} seconds", summary.total_duration.as_secs_f64());
        
        Ok(summary)
    }
    
    /// Create services optimized for validation testing
    async fn create_validation_services() -> Result<(Arc<PreviewService<NativeFileSystemService>>, ValidationConfig), ValidationError> {
        info!("⚙️ Setting up validation services");
        
        // Create file system service
        let fs_config = FileSystemConfig::default();
        let fs_service = Arc::new(NativeFileSystemService::with_config(fs_config));
        
        // Create preview service with validation-optimized config
        let mut preview_config = PreviewServiceConfig::default();
        preview_config.cache_config.max_entries = 50;                           // Target: 50 previews
        preview_config.cache_config.max_memory_bytes = 500 * 1024 * 1024;       // Target: 500MB
        preview_config.progressive_threshold = 10 * 1024 * 1024;                // 10MB progressive threshold
        preview_config.max_file_size = 100 * 1024 * 1024;                       // 100MB max file size
        
        let preview_service = Arc::new(PreviewService::new(fs_service, preview_config));
        
        // Create validation config
        let validation_config = ValidationConfig {
            small_files_count: 25,
            medium_files_count: 15,
            large_files_count: 8,
            small_file_size: 1024 * 1024,        // 1MB
            medium_file_size: 25 * 1024 * 1024,  // 25MB
            large_file_size: 80 * 1024 * 1024,   // 80MB
            concurrent_operations: 12,
            stress_test_duration: std::time::Duration::from_secs(45),
            target_cache_hit_time_us: 10.0,
            target_cache_miss_time_ms: 100.0,
            target_memory_limit_mb: 500,
            target_cache_entries: 50,
            acceptable_memory_usage_percent: 90.0,
            minimum_hit_rate: 0.75,
            maximum_response_time_ms: 150.0,
        };
        
        Ok((preview_service, validation_config))
    }
    
    /// Generate a comprehensive validation summary
    fn generate_validation_summary(report: &ValidationReport, total_duration: std::time::Duration) -> ValidationSummary {
        let memory = &report.memory_validation;
        let cache = &report.cache_validation;
        let progressive = &report.progressive_validation;
        let performance = &report.performance_benchmarks;
        let stress = &report.stress_test_results;
        
        // Calculate overall scores
        let memory_score = Self::calculate_memory_score(memory);
        let cache_score = Self::calculate_cache_score(cache);
        let progressive_score = Self::calculate_progressive_score(progressive);
        let performance_score = Self::calculate_performance_score(performance);
        let integration_score = Self::calculate_integration_score(&report.integration_validation);
        let stress_score = Self::calculate_stress_score(stress);
        
        let overall_score = (memory_score + cache_score + progressive_score + 
                           performance_score + integration_score + stress_score) / 6.0;
        
        // Determine validation status
        let status = if overall_score >= 90.0 {
            ValidationStatus::Excellent
        } else if overall_score >= 80.0 {
            ValidationStatus::Good
        } else if overall_score >= 70.0 {
            ValidationStatus::Acceptable
        } else if overall_score >= 60.0 {
            ValidationStatus::NeedsImprovement
        } else {
            ValidationStatus::Failed
        };
        
        ValidationSummary {
            status,
            overall_score,
            total_duration,
            memory_score,
            cache_score,
            progressive_score,
            performance_score,
            integration_score,
            stress_score,
            key_metrics: KeyMetrics {
                peak_memory_mb: memory.peak_memory_usage / (1024 * 1024),
                memory_efficiency_percent: memory.memory_efficiency,
                cache_hit_rate_percent: cache.hit_rate * 100.0,
                max_cache_entries: cache.max_entries_reached,
                largest_file_processed_mb: progressive.largest_file_processed / (1024 * 1024),
                average_response_time_ms: performance.cache_miss_time_ms,
                stress_test_throughput: stress.throughput_files_per_second,
                failure_rate_percent: stress.failure_rate * 100.0,
            },
            critical_issues: Self::identify_critical_issues(report),
            recommendations_count: report.recommendations.len(),
        }
    }
    
    /// Calculate memory validation score (0-100)
    fn calculate_memory_score(memory: &super::validation_profiler::MemoryValidationResult) -> f64 {
        let mut score = 100.0;
        
        // Penalty for high memory usage
        if memory.memory_efficiency > 90.0 {
            score -= 30.0;
        } else if memory.memory_efficiency > 80.0 {
            score -= 15.0;
        } else if memory.memory_efficiency > 70.0 {
            score -= 5.0;
        }
        
        // Major penalty for memory leaks
        if memory.memory_leaks_detected {
            score -= 50.0;
        }
        
        score.max(0.0f64)
    }
    
    /// Calculate cache validation score (0-100)
    fn calculate_cache_score(cache: &super::validation_profiler::CacheValidationResult) -> f64 {
        let mut score = 100.0;
        
        // Score based on hit rate
        let hit_rate_score = (cache.hit_rate * 100.0).min(100.0);
        score = score * 0.4 + hit_rate_score * 0.6;
        
        // Bonus for LRU correctness
        if cache.lru_correctness_validated {
            score += 10.0;
        }
        
        // Penalty for poor eviction efficiency
        if cache.eviction_efficiency < 0.8 {
            score -= 10.0;
        }
        
        score.min(100.0f64).max(0.0f64)
    }
    
    /// Calculate progressive loading score (0-100)
    fn calculate_progressive_score(progressive: &super::validation_profiler::ProgressiveValidationResult) -> f64 {
        let mut score = 100.0;
        
        // Penalty for slow cancellation
        if progressive.cancellation_response_time.as_millis() > 50 {
            score -= 20.0;
        }
        
        // Penalty for high memory usage during loading
        if progressive.memory_efficiency_during_loading > 50.0 {
            score -= 15.0;
        }
        
        // Score based on progress reporting accuracy
        score = score * 0.8 + progressive.progress_reporting_accuracy * 100.0 * 0.2;
        
        score.min(100.0f64).max(0.0f64)
    }
    
    /// Calculate performance score (0-100)  
    fn calculate_performance_score(performance: &super::validation_profiler::PerformanceBenchmarks) -> f64 {
        let mut score = 100.0;
        
        // Cache hit time penalty
        if performance.cache_hit_time_us > 20.0 {
            score -= 20.0;
        } else if performance.cache_hit_time_us > 10.0 {
            score -= 10.0;
        }
        
        // Cache miss time penalty  
        if performance.cache_miss_time_ms > 200.0 {
            score -= 30.0;
        } else if performance.cache_miss_time_ms > 100.0 {
            score -= 15.0;
        }
        
        // Progressive loading throughput bonus
        if performance.progressive_loading_throughput_mbps > 100.0 {
            score += 10.0;
        }
        
        score.min(100.0f64).max(0.0f64)
    }
    
    /// Calculate integration score (0-100)
    fn calculate_integration_score(integration: &super::validation_profiler::IntegrationValidationResult) -> f64 {
        let mut score = 0.0;
        
        if integration.cache_file_system_integration { score += 25.0; }
        if integration.progressive_cache_integration { score += 25.0; }
        if integration.service_health_monitoring { score += 20.0; }
        if integration.api_contract_compliance { score += 20.0; }
        
        // Error handling robustness score
        score += integration.error_handling_robustness * 10.0;
        
        score.min(100.0)
    }
    
    /// Calculate stress test score (0-100)
    fn calculate_stress_score(stress: &super::validation_profiler::StressTestResults) -> f64 {
        let mut score = 100.0;
        
        // Penalty for high failure rate
        score -= stress.failure_rate * 100.0;
        
        // Penalty for resource exhaustion
        if stress.resource_exhaustion_detected {
            score -= 30.0;
        }
        
        // Penalty for slow response times
        if stress.p95_response_time.as_millis() > 500 {
            score -= 20.0;
        } else if stress.p95_response_time.as_millis() > 200 {
            score -= 10.0;
        }
        
        score.max(0.0f64)
    }
    
    /// Identify critical issues from validation report
    fn identify_critical_issues(report: &ValidationReport) -> Vec<String> {
        let mut issues = Vec::new();
        
        // Memory issues
        if report.memory_validation.memory_leaks_detected {
            issues.push("Memory leaks detected".to_string());
        }
        
        if report.memory_validation.memory_efficiency > 95.0 {
            issues.push("Memory usage exceeds 95% of limit".to_string());
        }
        
        // Cache issues
        if report.cache_validation.hit_rate < 0.5 {
            issues.push("Cache hit rate below 50%".to_string());
        }
        
        // Performance issues
        if report.performance_benchmarks.cache_hit_time_us > 50.0 {
            issues.push("Cache hit time exceeds 50μs".to_string());
        }
        
        // Stress test issues
        if report.stress_test_results.resource_exhaustion_detected {
            issues.push("Resource exhaustion detected during stress testing".to_string());
        }
        
        if report.stress_test_results.failure_rate > 0.1 {
            issues.push("Failure rate exceeds 10% during stress testing".to_string());
        }
        
        issues
    }
    
    /// Log validation results in a structured format
    fn log_validation_results(summary: &ValidationSummary) {
        match summary.status {
            ValidationStatus::Excellent => info!("🏆 VALIDATION PASSED - EXCELLENT (Score: {:.1})", summary.overall_score),
            ValidationStatus::Good => info!("✅ VALIDATION PASSED - GOOD (Score: {:.1})", summary.overall_score),
            ValidationStatus::Acceptable => info!("👍 VALIDATION PASSED - ACCEPTABLE (Score: {:.1})", summary.overall_score),
            ValidationStatus::NeedsImprovement => warn!("⚠️ VALIDATION NEEDS IMPROVEMENT (Score: {:.1})", summary.overall_score),
            ValidationStatus::Failed => error!("❌ VALIDATION FAILED (Score: {:.1})", summary.overall_score),
        }
        
        info!("📊 Validation Scores:");
        info!("  Memory:      {:.1}/100", summary.memory_score);
        info!("  Cache:       {:.1}/100", summary.cache_score);
        info!("  Progressive: {:.1}/100", summary.progressive_score);
        info!("  Performance: {:.1}/100", summary.performance_score);
        info!("  Integration: {:.1}/100", summary.integration_score);
        info!("  Stress:      {:.1}/100", summary.stress_score);
        
        info!("🔑 Key Metrics:");
        info!("  Peak Memory:     {} MB", summary.key_metrics.peak_memory_mb);
        info!("  Memory Usage:    {:.1}%", summary.key_metrics.memory_efficiency_percent);
        info!("  Cache Hit Rate:  {:.1}%", summary.key_metrics.cache_hit_rate_percent);
        info!("  Cache Entries:   {}", summary.key_metrics.max_cache_entries);
        info!("  Max File Size:   {} MB", summary.key_metrics.largest_file_processed_mb);
        info!("  Response Time:   {:.1} ms", summary.key_metrics.average_response_time_ms);
        info!("  Throughput:      {:.1} files/sec", summary.key_metrics.stress_test_throughput);
        info!("  Failure Rate:    {:.2}%", summary.key_metrics.failure_rate_percent);
        
        if !summary.critical_issues.is_empty() {
            error!("🚨 Critical Issues Found:");
            for issue in &summary.critical_issues {
                error!("  - {}", issue);
            }
        }
        
        info!("📋 Generated {} optimization recommendations", summary.recommendations_count);
    }
    
    /// Save detailed validation report to file
    async fn save_validation_report(report: &ValidationReport) -> Result<(), ValidationError> {
        let report_path = "validation_report.json";
        let json_data = serde_json::to_string_pretty(report)
            .map_err(|e| ValidationError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        
        tokio::fs::write(report_path, json_data).await?;
        info!("💾 Detailed validation report saved to: {}", report_path);
        
        Ok(())
    }
}

/// Summary of validation results
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub status: ValidationStatus,
    pub overall_score: f64,
    pub total_duration: std::time::Duration,
    pub memory_score: f64,
    pub cache_score: f64,
    pub progressive_score: f64,
    pub performance_score: f64,
    pub integration_score: f64,
    pub stress_score: f64,
    pub key_metrics: KeyMetrics,
    pub critical_issues: Vec<String>,
    pub recommendations_count: usize,
}

/// Key performance metrics
#[derive(Debug, Clone)]
pub struct KeyMetrics {
    pub peak_memory_mb: usize,
    pub memory_efficiency_percent: f64,
    pub cache_hit_rate_percent: f64,
    pub max_cache_entries: usize,
    pub largest_file_processed_mb: u64,
    pub average_response_time_ms: f64,
    pub stress_test_throughput: f64,
    pub failure_rate_percent: f64,
}

/// Validation status levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationStatus {
    Excellent,        // 90-100
    Good,             // 80-89
    Acceptable,       // 70-79
    NeedsImprovement, // 60-69
    Failed,           // <60
}

impl ValidationStatus {
    pub fn is_passing(&self) -> bool {
        matches!(self, ValidationStatus::Excellent | ValidationStatus::Good | ValidationStatus::Acceptable)
    }
}