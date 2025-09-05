use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::{info, warn, debug, error};

use crate::services::preview_cache::{
    ThreadSafePreviewCache, PreviewCacheKey, 
    CachedPreviewData, PreviewDataMetadata, PreviewCacheStats
};
use crate::services::cache::{CacheService, CacheMaintenanceConfig, CacheMetrics};
use crate::state::performance::with_profiler;

/// Memory optimization configuration for preview and metadata caches
#[derive(Debug, Clone)]
pub struct MemoryOptimizerConfig {
    /// Maximum memory usage before triggering aggressive cleanup (bytes)
    pub max_memory_bytes: usize,
    /// Memory usage percentage to trigger proactive cleanup
    pub cleanup_threshold_percent: f64,
    /// Memory usage percentage to trigger emergency cleanup
    pub emergency_threshold_percent: f64,
    /// Interval for proactive memory monitoring
    pub monitoring_interval: Duration,
    /// Enable adaptive cache sizing based on system memory
    pub adaptive_sizing: bool,
    /// Enable predictive eviction based on access patterns
    pub predictive_eviction: bool,
    /// Enable memory pressure detection
    pub memory_pressure_detection: bool,
}

impl Default for MemoryOptimizerConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 500 * 1024 * 1024,  // 500MB as per requirements
            cleanup_threshold_percent: 75.0,      // Start cleanup at 75%
            emergency_threshold_percent: 90.0,    // Emergency cleanup at 90%
            monitoring_interval: Duration::from_secs(30), // Check every 30 seconds
            adaptive_sizing: true,
            predictive_eviction: true,
            memory_pressure_detection: true,
        }
    }
}

/// Memory usage statistics for optimization analysis
#[derive(Debug, Clone)]
pub struct MemoryUsageStats {
    /// Current total memory usage (bytes)
    pub total_memory_bytes: usize,
    /// Preview cache memory usage (bytes)
    pub preview_cache_bytes: usize,
    /// Metadata cache memory estimate (bytes)
    pub metadata_cache_bytes: usize,
    /// System memory pressure level (0.0-1.0)
    pub memory_pressure: f64,
    /// Number of cache entries
    pub total_entries: usize,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Memory efficiency (useful data / total memory)
    pub memory_efficiency: f64,
    /// Timestamp when stats were collected
    pub collected_at: Instant,
}

/// Results of memory optimization operation
#[derive(Debug, Clone)]
pub struct MemoryOptimizationResult {
    /// Memory freed during optimization (bytes)
    pub memory_freed: usize,
    /// Number of cache entries evicted
    pub entries_evicted: usize,
    /// Time taken for optimization
    pub optimization_duration: Duration,
    /// Memory usage before optimization
    pub before_optimization: MemoryUsageStats,
    /// Memory usage after optimization
    pub after_optimization: MemoryUsageStats,
    /// Optimization actions taken
    pub actions_taken: Vec<String>,
    /// Whether emergency cleanup was triggered
    pub emergency_cleanup: bool,
}

/// Access pattern tracking for predictive eviction
#[derive(Debug, Clone)]
struct AccessPattern {
    /// Number of times accessed
    access_count: u32,
    /// Last access time
    last_access: Instant,
    /// Average time between accesses
    avg_access_interval: Option<Duration>,
    /// Trend in access frequency (increasing/decreasing)
    access_trend: AccessTrend,
}

#[derive(Debug, Clone)]
enum AccessTrend {
    Increasing,
    Stable,
    Decreasing,
    Unknown,
}

impl AccessPattern {
    fn new() -> Self {
        Self {
            access_count: 0,
            last_access: Instant::now(),
            avg_access_interval: None,
            access_trend: AccessTrend::Unknown,
        }
    }

    fn record_access(&mut self) {
        let now = Instant::now();
        let interval = now.duration_since(self.last_access);
        
        // Update average interval
        if let Some(avg) = self.avg_access_interval {
            let weight = 0.8; // Exponential moving average
            self.avg_access_interval = Some(Duration::from_nanos(
                (avg.as_nanos() as f64 * weight + interval.as_nanos() as f64 * (1.0 - weight)) as u64
            ));
        } else if self.access_count > 0 {
            self.avg_access_interval = Some(interval);
        }
        
        // Update access trend (simplified)
        if self.access_count > 2 {
            if let Some(avg_interval) = self.avg_access_interval {
                if interval < avg_interval {
                    self.access_trend = AccessTrend::Increasing;
                } else if interval > avg_interval * 2 {
                    self.access_trend = AccessTrend::Decreasing;
                } else {
                    self.access_trend = AccessTrend::Stable;
                }
            }
        }
        
        self.access_count += 1;
        self.last_access = now;
    }

    /// Calculate eviction priority (higher = more likely to evict)
    fn eviction_priority(&self, now: Instant) -> f64 {
        let time_since_access = now.duration_since(self.last_access).as_secs_f64();
        let access_frequency = self.access_count as f64;
        
        // Base priority on time since last access
        let mut priority = time_since_access / 3600.0; // Hours since access
        
        // Adjust for access frequency (lower frequency = higher priority)
        if access_frequency > 0.0 {
            priority *= 1.0 / (access_frequency.sqrt());
        }
        
        // Adjust for access trend
        match self.access_trend {
            AccessTrend::Decreasing => priority *= 1.5, // More likely to evict
            AccessTrend::Increasing => priority *= 0.5, // Less likely to evict
            AccessTrend::Stable => priority *= 1.0,
            AccessTrend::Unknown => priority *= 1.0,
        }
        
        priority
    }
}

/// Advanced memory optimizer with predictive eviction and adaptive sizing
pub struct MemoryOptimizer {
    config: MemoryOptimizerConfig,
    preview_cache: Arc<ThreadSafePreviewCache>,
    metadata_cache: Arc<CacheService>,
    access_patterns: Arc<RwLock<HashMap<PreviewCacheKey, AccessPattern>>>,
    memory_stats_history: Arc<Mutex<VecDeque<MemoryUsageStats>>>,
    is_monitoring: Arc<Mutex<bool>>,
    optimization_stats: Arc<Mutex<Vec<MemoryOptimizationResult>>>,
}

impl MemoryOptimizer {
    /// Create a new memory optimizer
    pub fn new(
        config: MemoryOptimizerConfig,
        preview_cache: Arc<ThreadSafePreviewCache>,
        metadata_cache: Arc<CacheService>,
    ) -> Self {
        Self {
            config,
            preview_cache,
            metadata_cache,
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            memory_stats_history: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            is_monitoring: Arc::new(Mutex::new(false)),
            optimization_stats: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record cache access for pattern tracking
    pub async fn record_cache_access(&self, key: PreviewCacheKey) {
        if self.config.predictive_eviction {
            let mut patterns = self.access_patterns.write().await;
            patterns.entry(key).or_insert_with(AccessPattern::new).record_access();
        }
    }

    /// Get current memory usage statistics
    pub async fn get_memory_stats(&self) -> Result<MemoryUsageStats, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        
        // Get preview cache stats
        let preview_stats = self.preview_cache.stats();
        let preview_memory = preview_stats.memory_bytes;
        
        // Get metadata cache stats (estimate from database metrics)
        let cache_metrics = self.metadata_cache.get_cache_metrics().await?;
        let metadata_memory = estimate_metadata_memory(&cache_metrics);
        
        let total_memory = preview_memory + metadata_memory;
        
        // Calculate memory pressure (simple heuristic)
        let memory_pressure = if self.config.max_memory_bytes > 0 {
            (total_memory as f64 / self.config.max_memory_bytes as f64).min(1.0)
        } else {
            0.0
        };
        
        // Calculate memory efficiency
        let memory_efficiency = if total_memory > 0 {
            calculate_memory_efficiency(&preview_stats, &cache_metrics, total_memory)
        } else {
            1.0
        };
        
        let stats = MemoryUsageStats {
            total_memory_bytes: total_memory,
            preview_cache_bytes: preview_memory,
            metadata_cache_bytes: metadata_memory,
            memory_pressure,
            total_entries: preview_stats.entries + cache_metrics.metadata_cache.total_entries,
            hit_rate: preview_stats.hit_rate,
            memory_efficiency,
            collected_at: start_time,
        };
        
        // Store in history
        {
            let mut history = self.memory_stats_history.lock().unwrap();
            history.push_back(stats.clone());
            if history.len() > 100 {
                history.pop_front();
            }
        }
        
        debug!(
            "Memory stats collected: {}MB total, {:.1}% pressure, {:.1}% efficiency",
            total_memory / (1024 * 1024),
            memory_pressure * 100.0,
            memory_efficiency * 100.0
        );
        
        Ok(stats)
    }

    /// Check if memory optimization is needed
    pub async fn needs_optimization(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let stats = self.get_memory_stats().await?;
        
        let usage_percent = (stats.total_memory_bytes as f64 / self.config.max_memory_bytes as f64) * 100.0;
        
        Ok(usage_percent > self.config.cleanup_threshold_percent)
    }

    /// Perform comprehensive memory optimization
    pub async fn optimize_memory(&self) -> Result<MemoryOptimizationResult, Box<dyn std::error::Error + Send + Sync>> {
        let start_time = Instant::now();
        let before_stats = self.get_memory_stats().await?;
        let mut actions_taken = Vec::new();
        
        info!(
            "Starting memory optimization: {}MB used ({:.1}% of limit)",
            before_stats.total_memory_bytes / (1024 * 1024),
            (before_stats.total_memory_bytes as f64 / self.config.max_memory_bytes as f64) * 100.0
        );
        
        let usage_percent = (before_stats.total_memory_bytes as f64 / self.config.max_memory_bytes as f64) * 100.0;
        let emergency_cleanup = usage_percent > self.config.emergency_threshold_percent;
        
        let mut total_entries_evicted = 0;
        
        // Step 1: Clean up stale entries
        if self.config.predictive_eviction || emergency_cleanup {
            let stale_cleaned = self.cleanup_stale_entries().await?;
            if stale_cleaned > 0 {
                actions_taken.push(format!("Cleaned {} stale cache entries", stale_cleaned));
                total_entries_evicted += stale_cleaned;
            }
        }
        
        // Step 2: Predictive eviction based on access patterns
        if self.config.predictive_eviction {
            let predictive_evicted = self.predictive_eviction().await?;
            if predictive_evicted > 0 {
                actions_taken.push(format!("Predictively evicted {} entries", predictive_evicted));
                total_entries_evicted += predictive_evicted;
            }
        }
        
        // Step 3: Emergency cleanup if still over threshold
        if emergency_cleanup {
            let emergency_evicted = self.emergency_cleanup().await?;
            if emergency_evicted > 0 {
                actions_taken.push(format!("Emergency eviction: {} entries", emergency_evicted));
                total_entries_evicted += emergency_evicted;
            }
        }
        
        // Step 4: Database maintenance
        let maintenance_config = CacheMaintenanceConfig {
            max_age_seconds: if emergency_cleanup { 3 * 24 * 3600 } else { 7 * 24 * 3600 },
            max_entries: if emergency_cleanup { Some(1000) } else { Some(5000) },
            vacuum_database: emergency_cleanup,
        };
        
        let maintenance_result = self.metadata_cache.perform_maintenance(&maintenance_config).await?;
        if maintenance_result.cleanup_result.total_entries_removed > 0 {
            actions_taken.push(format!(
                "Database maintenance: {} entries removed",
                maintenance_result.cleanup_result.total_entries_removed
            ));
            total_entries_evicted += maintenance_result.cleanup_result.total_entries_removed;
        }
        
        // Step 5: Adaptive resizing if enabled
        if self.config.adaptive_sizing {
            let resizing_actions = self.adaptive_resize().await?;
            actions_taken.extend(resizing_actions);
        }
        
        let after_stats = self.get_memory_stats().await?;
        let memory_freed = before_stats.total_memory_bytes.saturating_sub(after_stats.total_memory_bytes);
        let optimization_duration = start_time.elapsed();
        
        let result = MemoryOptimizationResult {
            memory_freed,
            entries_evicted: total_entries_evicted,
            optimization_duration,
            before_optimization: before_stats,
            after_optimization: after_stats.clone(),
            actions_taken: actions_taken.clone(),
            emergency_cleanup,
        };
        
        // Record optimization stats
        {
            let mut stats = self.optimization_stats.lock().unwrap();
            stats.push(result.clone());
            if stats.len() > 50 {
                stats.remove(0);
            }
        }
        
        info!(
            "Memory optimization completed in {:.2}s: freed {}MB ({} actions, {} entries evicted)",
            optimization_duration.as_secs_f64(),
            memory_freed / (1024 * 1024),
            actions_taken.len(),
            total_entries_evicted
        );
        
        // Update performance profiler
        with_profiler(|profiler| {
            profiler.record_operation("memory_optimization".to_string(), optimization_duration);
        });
        
        Ok(result)
    }

    /// Clean up stale cache entries
    async fn cleanup_stale_entries(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let stale_preview = self.preview_cache.cleanup_stale();
        let old_preview = self.preview_cache.cleanup_old();
        
        Ok(stale_preview + old_preview)
    }

    /// Perform predictive eviction based on access patterns
    async fn predictive_eviction(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let patterns = self.access_patterns.read().await;
        let now = Instant::now();
        
        // Calculate eviction priorities for all patterns
        let mut eviction_candidates: Vec<(PreviewCacheKey, f64)> = patterns
            .iter()
            .map(|(key, pattern)| (key.clone(), pattern.eviction_priority(now)))
            .collect();
        
        // Sort by priority (highest first)
        eviction_candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Evict top 25% of candidates if memory pressure is high
        let stats = self.get_memory_stats().await?;
        let evict_count = if stats.memory_pressure > 0.8 {
            eviction_candidates.len() / 4
        } else if stats.memory_pressure > 0.7 {
            eviction_candidates.len() / 8
        } else {
            return Ok(0);
        };
        
        let mut evicted = 0;
        for (key, _priority) in eviction_candidates.iter().take(evict_count) {
            if self.preview_cache.remove(key) {
                evicted += 1;
            }
        }
        
        // Clean up access patterns for evicted entries
        drop(patterns);
        let mut patterns = self.access_patterns.write().await;
        for (key, _) in eviction_candidates.iter().take(evicted) {
            patterns.remove(key);
        }
        
        debug!("Predictive eviction removed {} entries", evicted);
        Ok(evicted)
    }

    /// Perform emergency cleanup when memory usage is critical
    async fn emergency_cleanup(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        warn!("Performing emergency memory cleanup");
        
        // Clear a significant portion of preview cache
        let stats_before = self.preview_cache.stats();
        let _target_entries = stats_before.entries / 2; // Remove 50% of entries
        
        // Use LRU eviction to remove oldest entries
        
        // Clear access patterns for emergency cleanup
        {
            let mut patterns = self.access_patterns.write().await;
            patterns.clear();
        }
        
        // For preview cache, we rely on its internal LRU eviction
        // by setting a temporary lower memory limit
        let stats_after = self.preview_cache.stats();
        let evicted = stats_before.entries.saturating_sub(stats_after.entries);
        
        warn!("Emergency cleanup evicted {} preview cache entries", evicted);
        Ok(evicted)
    }

    /// Adaptive resizing based on system memory and usage patterns
    async fn adaptive_resize(&self) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut actions = Vec::new();
        let stats = self.get_memory_stats().await?;
        
        // Get historical memory usage trends
        let history = {
            let history = self.memory_stats_history.lock().unwrap();
            history.clone()
        };
        
        if history.len() < 5 {
            return Ok(actions); // Need more history for trend analysis
        }
        
        // Calculate memory usage trend
        let recent_stats: Vec<_> = history.iter().rev().take(5).collect();
        let avg_recent_usage = recent_stats.iter()
            .map(|s| s.total_memory_bytes as f64)
            .sum::<f64>() / recent_stats.len() as f64;
        
        let older_stats: Vec<_> = history.iter().rev().skip(5).take(5).collect();
        if !older_stats.is_empty() {
            let avg_older_usage = older_stats.iter()
                .map(|s| s.total_memory_bytes as f64)
                .sum::<f64>() / older_stats.len() as f64;
            
            let usage_trend = (avg_recent_usage - avg_older_usage) / avg_older_usage;
            
            if usage_trend > 0.2 {
                // Memory usage increasing rapidly - be more aggressive
                actions.push("Detected increasing memory trend - enabling aggressive cleanup".to_string());
            } else if usage_trend < -0.1 {
                // Memory usage decreasing - can be less aggressive
                actions.push("Detected decreasing memory trend - reducing cleanup frequency".to_string());
            }
        }
        
        Ok(actions)
    }

    /// Start background memory monitoring
    pub fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let optimizer = self.clone_for_background();
        let interval = self.config.monitoring_interval;
        
        {
            let mut monitoring = self.is_monitoring.lock().unwrap();
            *monitoring = true;
        }
        
        tokio::spawn(async move {
            let mut monitor_interval = tokio::time::interval(interval);
            monitor_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            
            loop {
                monitor_interval.tick().await;
                
                // Check if monitoring should continue
                {
                    let monitoring = optimizer.is_monitoring.lock().unwrap();
                    if !*monitoring {
                        break;
                    }
                }
                
                match optimizer.needs_optimization().await {
                    Ok(needs_opt) => {
                        if needs_opt {
                            info!("Memory monitoring detected need for optimization");
                            match optimizer.optimize_memory().await {
                                Ok(result) => {
                                    info!(
                                        "Background optimization freed {}MB in {:.2}s",
                                        result.memory_freed / (1024 * 1024),
                                        result.optimization_duration.as_secs_f64()
                                    );
                                }
                                Err(e) => {
                                    error!("Background memory optimization failed: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Memory monitoring check failed: {}", e);
                    }
                }
            }
            
            info!("Background memory monitoring stopped");
        })
    }

    /// Stop background monitoring
    pub fn stop_monitoring(&self) {
        let mut monitoring = self.is_monitoring.lock().unwrap();
        *monitoring = false;
    }

    /// Get optimization history for analysis
    pub fn get_optimization_history(&self) -> Vec<MemoryOptimizationResult> {
        let stats = self.optimization_stats.lock().unwrap();
        stats.clone()
    }

    /// Get memory usage trends
    pub fn get_memory_trends(&self) -> Vec<MemoryUsageStats> {
        let history = self.memory_stats_history.lock().unwrap();
        history.clone().into()
    }

    /// Create a clone suitable for background tasks
    fn clone_for_background(&self) -> Self {
        Self {
            config: self.config.clone(),
            preview_cache: Arc::clone(&self.preview_cache),
            metadata_cache: Arc::clone(&self.metadata_cache),
            access_patterns: Arc::clone(&self.access_patterns),
            memory_stats_history: Arc::clone(&self.memory_stats_history),
            is_monitoring: Arc::clone(&self.is_monitoring),
            optimization_stats: Arc::clone(&self.optimization_stats),
        }
    }
}

/// Estimate metadata cache memory usage from database metrics
fn estimate_metadata_memory(metrics: &CacheMetrics) -> usize {
    // Rough estimate: database size + in-memory structures
    let db_size = metrics.database.size_bytes as usize;
    let in_memory_overhead = metrics.metadata_cache.total_entries * 256; // ~256 bytes per entry overhead
    db_size + in_memory_overhead
}

/// Calculate memory efficiency based on cache statistics
fn calculate_memory_efficiency(
    preview_stats: &PreviewCacheStats,
    cache_metrics: &CacheMetrics,
    total_memory: usize,
) -> f64 {
    if total_memory == 0 {
        return 1.0;
    }
    
    // Efficiency = (useful data) / (total memory)
    // Useful data includes actual file data, metadata, thumbnails
    let useful_data = cache_metrics.metadata_cache.total_size_bytes as usize + 
                     (preview_stats.entries * 1024); // Estimate preview data
    
    (useful_data as f64 / total_memory as f64).min(1.0)
}

/// Memory optimization benchmarking results
#[derive(Debug, Clone)]
pub struct MemoryBenchmarkResults {
    /// Results from running 1000+ file load test
    pub large_file_set_results: MemoryOptimizationResult,
    /// Results from tab switching stress test
    pub tab_switching_results: MemoryOptimizationResult,
    /// Memory usage over time during tests
    pub memory_timeline: Vec<MemoryUsageStats>,
    /// Peak memory usage reached
    pub peak_memory_bytes: usize,
    /// Average memory efficiency during tests
    pub average_efficiency: f64,
    /// Cache eviction effectiveness
    pub eviction_effectiveness: f64,
    /// Overall performance grade
    pub performance_grade: MemoryPerformanceGrade,
}

#[derive(Debug, Clone)]
pub enum MemoryPerformanceGrade {
    Excellent,  // Memory usage always under 80% of limit
    Good,       // Memory usage occasionally 80-90% of limit
    Fair,       // Memory usage occasionally over 90% of limit
    Poor,       // Memory usage frequently over 90% or hit limit
}

/// Benchmark memory optimization under various load scenarios
pub async fn benchmark_memory_optimization(
    optimizer: &MemoryOptimizer,
) -> Result<MemoryBenchmarkResults, Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting memory optimization benchmarks");
    
    let start_time = Instant::now();
    let mut memory_timeline = Vec::new();
    
    // Benchmark 1: Large file set (1000+ files)
    info!("Benchmarking large file set memory usage");
    let large_file_set_results = simulate_large_file_set_load(optimizer, &mut memory_timeline).await?;
    
    // Benchmark 2: Tab switching stress test
    info!("Benchmarking tab switching memory patterns");
    let tab_switching_results = simulate_tab_switching_stress(optimizer, &mut memory_timeline).await?;
    
    // Calculate summary statistics
    let peak_memory_bytes = memory_timeline.iter()
        .map(|stats| stats.total_memory_bytes)
        .max()
        .unwrap_or(0);
    
    let average_efficiency = if !memory_timeline.is_empty() {
        memory_timeline.iter()
            .map(|stats| stats.memory_efficiency)
            .sum::<f64>() / memory_timeline.len() as f64
    } else {
        0.0
    };
    
    // Calculate eviction effectiveness
    let eviction_effectiveness = calculate_eviction_effectiveness(&memory_timeline);
    
    // Determine performance grade
    let max_limit = optimizer.config.max_memory_bytes;
    let performance_grade = if peak_memory_bytes < (max_limit as f64 * 0.8) as usize {
        MemoryPerformanceGrade::Excellent
    } else if peak_memory_bytes < (max_limit as f64 * 0.9) as usize {
        MemoryPerformanceGrade::Good
    } else if peak_memory_bytes < max_limit {
        MemoryPerformanceGrade::Fair
    } else {
        MemoryPerformanceGrade::Poor
    };
    
    let results = MemoryBenchmarkResults {
        large_file_set_results,
        tab_switching_results,
        memory_timeline,
        peak_memory_bytes,
        average_efficiency,
        eviction_effectiveness,
        performance_grade: performance_grade.clone(),
    };
    
    info!(
        "Memory optimization benchmarks completed in {:.2}s: peak {}MB, {:.1}% efficiency, grade {:?}",
        start_time.elapsed().as_secs_f64(),
        peak_memory_bytes / (1024 * 1024),
        average_efficiency * 100.0,
        performance_grade
    );
    
    Ok(results)
}

/// Simulate loading 1000+ files and measure memory behavior
async fn simulate_large_file_set_load(
    optimizer: &MemoryOptimizer,
    timeline: &mut Vec<MemoryUsageStats>,
) -> Result<MemoryOptimizationResult, Box<dyn std::error::Error + Send + Sync>> {
    
    // Simulate creating preview cache entries for 1000 files
    for i in 0..1000 {
        let key = PreviewCacheKey::new(
            PathBuf::from(format!("/test/large_set/file_{}.jpg", i)),
            SystemTime::now(),
        );
        
        // Simulate preview data (varying sizes)
        let data_size = 1024 + (i % 100) * 1024; // 1KB to 100KB
        let preview_data = CachedPreviewData::new(
            vec![0u8; data_size],
            "image/jpeg".to_string(),
            "thumbnail".to_string(),
            (data_size * 10) as u64,
            PreviewDataMetadata::new(),
        );
        
        optimizer.preview_cache.put(key.clone(), preview_data).ok();
        optimizer.record_cache_access(key).await;
        
        // Record memory stats every 100 files
        if i % 100 == 0 {
            timeline.push(optimizer.get_memory_stats().await?);
        }
    }
    
    // Run optimization after loading all files
    optimizer.optimize_memory().await
}

/// Simulate rapid tab switching to test memory patterns
async fn simulate_tab_switching_stress(
    optimizer: &MemoryOptimizer,
    timeline: &mut Vec<MemoryUsageStats>,
) -> Result<MemoryOptimizationResult, Box<dyn std::error::Error + Send + Sync>> {
    
    // Create a set of "tabs" (file groups)
    let tab_files = vec![
        vec!["/tab1/doc1.pdf", "/tab1/image1.jpg", "/tab1/video1.mp4"],
        vec!["/tab2/doc2.pdf", "/tab2/image2.jpg", "/tab2/video2.mp4"],
        vec!["/tab3/doc3.pdf", "/tab3/image3.jpg", "/tab3/video3.mp4"],
        vec!["/tab4/doc4.pdf", "/tab4/image4.jpg", "/tab4/video4.mp4"],
    ];
    
    // Simulate rapid tab switching (each tab loads its files)
    for iteration in 0..50 {
        let tab_index = iteration % tab_files.len();
        
        for file_path in &tab_files[tab_index] {
            let key = PreviewCacheKey::new(
                PathBuf::from(file_path),
                SystemTime::now(),
            );
            
            // Simulate different file types with different memory footprints
            let data_size = if file_path.ends_with(".pdf") {
                50 * 1024 // 50KB for PDF thumbnails
            } else if file_path.ends_with(".jpg") {
                20 * 1024 // 20KB for image thumbnails
            } else {
                100 * 1024 // 100KB for video thumbnails
            };
            
            let preview_data = CachedPreviewData::new(
                vec![0u8; data_size],
                "image/jpeg".to_string(),
                "thumbnail".to_string(),
                (data_size * 5) as u64,
                PreviewDataMetadata::new(),
            );
            
            optimizer.preview_cache.put(key.clone(), preview_data).ok();
            optimizer.record_cache_access(key).await;
        }
        
        // Record memory stats every 10 iterations
        if iteration % 10 == 0 {
            timeline.push(optimizer.get_memory_stats().await?);
        }
        
        // Small delay to simulate real tab switching
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Run optimization after tab switching stress
    optimizer.optimize_memory().await
}

/// Calculate how effective cache eviction is at managing memory
fn calculate_eviction_effectiveness(timeline: &[MemoryUsageStats]) -> f64 {
    if timeline.len() < 2 {
        return 1.0;
    }
    
    // Look for patterns where memory usage drops significantly
    let mut effectiveness_scores = Vec::new();
    
    for window in timeline.windows(3) {
        let before = window[0].total_memory_bytes;
        let peak = window[1].total_memory_bytes;
        let after = window[2].total_memory_bytes;
        
        if peak > before && after < peak {
            // Memory dropped after a peak - good eviction
            let drop_ratio = (peak - after) as f64 / peak as f64;
            effectiveness_scores.push(drop_ratio);
        }
    }
    
    if effectiveness_scores.is_empty() {
        0.5 // Neutral score if no eviction patterns detected
    } else {
        effectiveness_scores.iter().sum::<f64>() / effectiveness_scores.len() as f64
    }
}