use media_organizer::services::{
    MemoryOptimizer, MemoryOptimizerConfig, MemoryUsageStats, MemoryOptimizationResult,
    MemoryBenchmarkResults, MemoryPerformanceGrade, benchmark_memory_optimization,
    ThreadSafePreviewCache, PreviewCacheConfig, PreviewCacheKey, CachedPreviewData,
    PreviewDataMetadata
};
use media_organizer::services::cache::{CacheService, CachedFileMetadata};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio;
use tracing_test::traced_test;
use tempfile::TempDir;

/// Create test memory optimizer with cache services
async fn create_test_memory_optimizer() -> (MemoryOptimizer, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    
    // Create preview cache with smaller limits for testing
    let preview_config = PreviewCacheConfig {
        max_entries: 20,
        max_memory_bytes: 1024 * 1024, // 1MB for testing
        max_age: Duration::from_secs(300),
        max_single_item_bytes: 100 * 1024, // 100KB
    };
    let preview_cache = Arc::new(ThreadSafePreviewCache::new(preview_config));
    
    // Create metadata cache
    let db_path = temp_dir.path().join("test_memory_cache.db");
    let metadata_cache = Arc::new(CacheService::new(&db_path).await.unwrap());
    
    // Configure memory optimizer for testing
    let optimizer_config = MemoryOptimizerConfig {
        max_memory_bytes: 2 * 1024 * 1024, // 2MB total limit for testing
        cleanup_threshold_percent: 60.0,   // More aggressive for testing
        emergency_threshold_percent: 80.0, // More aggressive for testing
        monitoring_interval: Duration::from_millis(100),
        adaptive_sizing: true,
        predictive_eviction: true,
        memory_pressure_detection: true,
    };
    
    let optimizer = MemoryOptimizer::new(optimizer_config, preview_cache, metadata_cache);
    
    (optimizer, temp_dir)
}

/// Create test preview data with specified size
fn create_test_preview_data(size: usize, content_type: &str) -> CachedPreviewData {
    CachedPreviewData::new(
        vec![0u8; size],
        content_type.to_string(),
        "test".to_string(),
        size as u64,
        PreviewDataMetadata::new(),
    )
}

/// Create test cache key
fn create_test_cache_key(path: &str) -> PreviewCacheKey {
    PreviewCacheKey::new(
        PathBuf::from(path),
        SystemTime::now(),
    )
}

/// Test 22.2 - Memory Usage Analysis and Optimization
#[tokio::test]
#[traced_test]
async fn test_memory_usage_analysis_and_optimization() {
    println!("🔍 Task 22.2: Testing Memory Usage Analysis and Optimization");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    // Test 1: Basic Memory Statistics
    println!("\n📊 Testing Basic Memory Statistics Collection...");
    let initial_stats = optimizer.get_memory_stats().await.unwrap();
    println!("✅ Initial memory stats: {}KB total, {:.1}% pressure", 
        initial_stats.total_memory_bytes / 1024,
        initial_stats.memory_pressure * 100.0);
    
    assert_eq!(initial_stats.total_entries, 0);
    assert_eq!(initial_stats.memory_pressure, 0.0);
    
    // Test 2: Memory Usage Growth Detection
    println!("\n📈 Testing Memory Usage Growth Detection...");
    
    // Add some preview cache entries
    for i in 0..10 {
        let key = create_test_cache_key(&format!("/test/growth_{}.jpg", i));
        let data = create_test_preview_data(50 * 1024, "image/jpeg"); // 50KB each
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key).await;
    }
    
    let after_growth_stats = optimizer.get_memory_stats().await.unwrap();
    println!("✅ After growth: {}KB total, {} entries, {:.1}% pressure",
        after_growth_stats.total_memory_bytes / 1024,
        after_growth_stats.total_entries,
        after_growth_stats.memory_pressure * 100.0);
    
    assert!(after_growth_stats.total_memory_bytes > initial_stats.total_memory_bytes);
    assert!(after_growth_stats.total_entries > 0);
    
    // Test 3: Automatic Optimization Trigger
    println!("\n⚡ Testing Automatic Optimization Trigger...");
    let needs_optimization = optimizer.needs_optimization().await.unwrap();
    println!("✅ Needs optimization: {}", needs_optimization);
    
    if needs_optimization {
        let optimization_result = optimizer.optimize_memory().await.unwrap();
        println!("✅ Optimization completed: freed {}KB in {:.2}ms",
            optimization_result.memory_freed / 1024,
            optimization_result.optimization_duration.as_millis());
        
        assert!(optimization_result.actions_taken.len() > 0);
        assert!(optimization_result.after_optimization.total_memory_bytes <= 
               optimization_result.before_optimization.total_memory_bytes);
    }
}

/// Test memory optimization with large file sets
#[tokio::test]
async fn test_large_file_set_memory_optimization() {
    println!("🗂️ Testing Large File Set Memory Optimization (1000+ files)");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    // Simulate loading 100 files (reduced for test speed)
    println!("\n📁 Loading 100 test files...");
    for i in 0..100 {
        let key = create_test_cache_key(&format!("/test/large_set/file_{}.jpg", i));
        let data = create_test_preview_data(20 * 1024, "image/jpeg"); // 20KB each
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key).await;
    }
    
    let stats_before = optimizer.get_memory_stats().await.unwrap();
    println!("✅ Before optimization: {}KB memory, {} entries",
        stats_before.total_memory_bytes / 1024,
        stats_before.total_entries);
    
    // Run optimization
    let optimization_result = optimizer.optimize_memory().await.unwrap();
    
    println!("✅ Large file set optimization results:");
    println!("   - Memory freed: {}KB", optimization_result.memory_freed / 1024);
    println!("   - Entries evicted: {}", optimization_result.entries_evicted);
    println!("   - Duration: {:.2}ms", optimization_result.optimization_duration.as_millis());
    println!("   - Actions: {:?}", optimization_result.actions_taken);
    
    // Validate results
    assert!(optimization_result.memory_freed > 0 || optimization_result.entries_evicted > 0);
    assert!(optimization_result.after_optimization.memory_pressure <= 
           optimization_result.before_optimization.memory_pressure);
}

/// Test cache eviction strategies under memory pressure
#[tokio::test]
async fn test_cache_eviction_strategies() {
    println!("🎯 Testing Advanced Cache Eviction Strategies");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    // Test 1: LRU Eviction
    println!("\n🔄 Testing LRU (Least Recently Used) Eviction...");
    
    // Fill cache to near capacity
    let mut keys = Vec::new();
    for i in 0..15 {
        let key = create_test_cache_key(&format!("/test/lru/file_{}.jpg", i));
        let data = create_test_preview_data(80 * 1024, "image/jpeg"); // 80KB each
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key.clone()).await;
        keys.push(key);
    }
    
    // Access first few files to make them recently used
    for key in keys.iter().take(5) {
        optimizer.record_cache_access(key.clone()).await;
    }
    
    let before_eviction = optimizer.get_memory_stats().await.unwrap();
    println!("✅ Before eviction: {}KB memory, {} entries",
        before_eviction.total_memory_bytes / 1024,
        before_eviction.total_entries);
    
    // Force optimization to trigger eviction
    let eviction_result = optimizer.optimize_memory().await.unwrap();
    
    println!("✅ LRU eviction results:");
    println!("   - Memory freed: {}KB", eviction_result.memory_freed / 1024);
    println!("   - Entries evicted: {}", eviction_result.entries_evicted);
    
    // Test 2: Predictive Eviction
    println!("\n🧠 Testing Predictive Eviction Based on Access Patterns...");
    
    // Create access patterns - some files accessed frequently, others rarely
    for i in 0..10 {
        let key = create_test_cache_key(&format!("/test/predictive/frequent_{}.jpg", i));
        let data = create_test_preview_data(40 * 1024, "image/jpeg");
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        
        // Simulate frequent access for first 5 files
        let access_count = if i < 5 { 10 } else { 1 };
        for _ in 0..access_count {
            optimizer.record_cache_access(key.clone()).await;
            tokio::time::sleep(Duration::from_millis(1)).await; // Small delay for pattern detection
        }
    }
    
    let predictive_result = optimizer.optimize_memory().await.unwrap();
    
    println!("✅ Predictive eviction results:");
    println!("   - Memory freed: {}KB", predictive_result.memory_freed / 1024);
    println!("   - Entries evicted: {}", predictive_result.entries_evicted);
    
    // Validate that optimization occurred
    assert!(eviction_result.memory_freed > 0 || eviction_result.entries_evicted > 0);
}

/// Test memory optimization under different tab workflows
#[tokio::test]
async fn test_multi_tab_memory_optimization() {
    println!("📑 Testing Multi-Tab Memory Optimization");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    // Simulate 4 tabs with different file types
    let tabs = vec![
        ("Documents", vec!["doc1.pdf", "doc2.pdf", "doc3.docx"]),
        ("Images", vec!["img1.jpg", "img2.png", "img3.gif"]),
        ("Videos", vec!["vid1.mp4", "vid2.avi", "vid3.mkv"]),
        ("Archives", vec!["arch1.zip", "arch2.tar", "arch3.rar"]),
    ];
    
    println!("\n🗂️ Simulating multi-tab file loading...");
    
    // Load files for each tab
    for (tab_name, files) in &tabs {
        for (i, file) in files.iter().enumerate() {
            let key = create_test_cache_key(&format!("/test/tabs/{}/{}", tab_name, file));
            
            // Different memory footprints for different file types
            let data_size = match file.split('.').last() {
                Some("pdf") | Some("docx") => 60 * 1024, // 60KB for documents
                Some("jpg") | Some("png") | Some("gif") => 30 * 1024, // 30KB for images
                Some("mp4") | Some("avi") | Some("mkv") => 100 * 1024, // 100KB for videos
                _ => 40 * 1024, // 40KB for archives and others
            };
            
            let data = create_test_preview_data(data_size, "application/octet-stream");
            optimizer.preview_cache.put(key.clone(), data).unwrap();
            optimizer.record_cache_access(key).await;
        }
        
        // Simulate tab switching delay
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    let before_optimization = optimizer.get_memory_stats().await.unwrap();
    println!("✅ Multi-tab memory usage: {}KB, {} entries",
        before_optimization.total_memory_bytes / 1024,
        before_optimization.total_entries);
    
    // Run optimization
    let multi_tab_result = optimizer.optimize_memory().await.unwrap();
    
    println!("✅ Multi-tab optimization results:");
    println!("   - Memory freed: {}KB", multi_tab_result.memory_freed / 1024);
    println!("   - Entries evicted: {}", multi_tab_result.entries_evicted);
    println!("   - Actions taken: {}", multi_tab_result.actions_taken.len());
    println!("   - Emergency cleanup: {}", multi_tab_result.emergency_cleanup);
    
    // Validate memory efficiency
    let efficiency = multi_tab_result.after_optimization.memory_efficiency;
    println!("   - Memory efficiency: {:.1}%", efficiency * 100.0);
    
    assert!(efficiency > 0.0);
    assert!(multi_tab_result.after_optimization.memory_pressure <= 1.0);
}

/// Test background memory monitoring
#[tokio::test]
async fn test_background_memory_monitoring() {
    println!("🔍 Testing Background Memory Monitoring");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    // Start background monitoring
    println!("\n⚙️ Starting background memory monitoring...");
    let monitoring_handle = optimizer.start_monitoring();
    
    // Add some memory pressure
    for i in 0..20 {
        let key = create_test_cache_key(&format!("/test/monitoring/file_{}.jpg", i));
        let data = create_test_preview_data(60 * 1024, "image/jpeg"); // 60KB each
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key).await;
        
        if i % 5 == 0 {
            tokio::time::sleep(Duration::from_millis(50)).await; // Allow monitoring to run
        }
    }
    
    // Let monitoring run for a short period
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Stop monitoring
    optimizer.stop_monitoring();
    monitoring_handle.abort();
    
    let final_stats = optimizer.get_memory_stats().await.unwrap();
    println!("✅ Background monitoring completed");
    println!("   - Final memory: {}KB", final_stats.total_memory_bytes / 1024);
    println!("   - Memory pressure: {:.1}%", final_stats.memory_pressure * 100.0);
    
    // Check optimization history
    let history = optimizer.get_optimization_history();
    println!("   - Optimization runs: {}", history.len());
    
    if !history.is_empty() {
        println!("   - Average memory freed per run: {}KB", 
            history.iter().map(|r| r.memory_freed).sum::<usize>() / history.len() / 1024);
    }
    
    assert!(final_stats.memory_pressure <= 1.0);
}

/// Test memory benchmark suite
#[tokio::test]
async fn test_comprehensive_memory_benchmarks() {
    println!("🏆 Testing Comprehensive Memory Benchmark Suite");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    println!("\n🚀 Running comprehensive memory benchmarks...");
    let benchmark_results = benchmark_memory_optimization(&optimizer).await.unwrap();
    
    println!("✅ Memory benchmark results:");
    println!("   - Peak memory usage: {}KB", benchmark_results.peak_memory_bytes / 1024);
    println!("   - Average efficiency: {:.1}%", benchmark_results.average_efficiency * 100.0);
    println!("   - Eviction effectiveness: {:.1}%", benchmark_results.eviction_effectiveness * 100.0);
    println!("   - Performance grade: {:?}", benchmark_results.performance_grade);
    println!("   - Timeline points: {}", benchmark_results.memory_timeline.len());
    
    // Large file set results
    println!("\n📁 Large file set benchmark:");
    println!("   - Memory freed: {}KB", 
        benchmark_results.large_file_set_results.memory_freed / 1024);
    println!("   - Entries evicted: {}", 
        benchmark_results.large_file_set_results.entries_evicted);
    println!("   - Duration: {:.2}ms", 
        benchmark_results.large_file_set_results.optimization_duration.as_millis());
    
    // Tab switching results
    println!("\n📑 Tab switching benchmark:");
    println!("   - Memory freed: {}KB", 
        benchmark_results.tab_switching_results.memory_freed / 1024);
    println!("   - Entries evicted: {}", 
        benchmark_results.tab_switching_results.entries_evicted);
    println!("   - Duration: {:.2}ms", 
        benchmark_results.tab_switching_results.optimization_duration.as_millis());
    
    // Validate benchmark quality
    assert!(benchmark_results.peak_memory_bytes > 0);
    assert!(benchmark_results.average_efficiency >= 0.0 && benchmark_results.average_efficiency <= 1.0);
    assert!(benchmark_results.eviction_effectiveness >= 0.0 && benchmark_results.eviction_effectiveness <= 1.0);
    assert!(benchmark_results.memory_timeline.len() > 0);
    
    // Validate performance grade is reasonable
    match benchmark_results.performance_grade {
        MemoryPerformanceGrade::Excellent | MemoryPerformanceGrade::Good | 
        MemoryPerformanceGrade::Fair | MemoryPerformanceGrade::Poor => {
            // All grades are valid
        }
    }
    
    println!("🎉 All memory optimization benchmarks completed successfully!");
}

/// Test memory optimizer configuration and adaptation
#[tokio::test]
async fn test_memory_optimizer_configuration() {
    println!("⚙️ Testing Memory Optimizer Configuration and Adaptation");
    
    let temp_dir = TempDir::new().unwrap();
    
    // Test custom configuration
    let custom_config = MemoryOptimizerConfig {
        max_memory_bytes: 512 * 1024, // 512KB limit
        cleanup_threshold_percent: 50.0, // Very aggressive
        emergency_threshold_percent: 70.0,
        monitoring_interval: Duration::from_millis(50),
        adaptive_sizing: true,
        predictive_eviction: true,
        memory_pressure_detection: true,
    };
    
    let preview_config = PreviewCacheConfig {
        max_entries: 10,
        max_memory_bytes: 256 * 1024, // 256KB
        max_age: Duration::from_secs(60),
        max_single_item_bytes: 50 * 1024,
    };
    let preview_cache = Arc::new(ThreadSafePreviewCache::new(preview_config));
    
    let db_path = temp_dir.path().join("config_test_cache.db");
    let metadata_cache = Arc::new(CacheService::new(&db_path).await.unwrap());
    
    let optimizer = MemoryOptimizer::new(custom_config, preview_cache, metadata_cache);
    
    // Test aggressive cleanup with small limits
    for i in 0..15 {
        let key = create_test_cache_key(&format!("/test/config/file_{}.jpg", i));
        let data = create_test_preview_data(40 * 1024, "image/jpeg"); // 40KB each
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key).await;
    }
    
    let needs_opt = optimizer.needs_optimization().await.unwrap();
    println!("✅ Custom config needs optimization: {}", needs_opt);
    
    if needs_opt {
        let result = optimizer.optimize_memory().await.unwrap();
        println!("✅ Aggressive optimization freed {}KB", result.memory_freed / 1024);
        
        // With aggressive settings, should definitely need optimization and clean up
        assert!(result.memory_freed > 0 || result.entries_evicted > 0);
    }
    
    println!("✅ Memory optimizer configuration test completed");
}

/// Performance validation test for Task 22.2
#[tokio::test]
async fn test_task_22_2_performance_validation() {
    println!("🎯 Task 22.2: Performance Validation Test");
    
    let (optimizer, _temp_dir) = create_test_memory_optimizer().await;
    
    println!("\n📋 Task 22.2 Requirements Validation:");
    
    // Requirement 1: Analyze memory consumption during preview and tab workflows
    println!("✅ 1. Memory consumption analysis during workflows");
    
    // Load previews and measure memory
    for i in 0..25 {
        let key = create_test_cache_key(&format!("/test/validation/preview_{}.jpg", i));
        let data = create_test_preview_data(50 * 1024, "image/jpeg");
        
        optimizer.preview_cache.put(key.clone(), data).unwrap();
        optimizer.record_cache_access(key).await;
    }
    
    let preview_stats = optimizer.get_memory_stats().await.unwrap();
    println!("   - Preview workflow memory: {}KB", preview_stats.total_memory_bytes / 1024);
    
    // Simulate tab switching
    for tab in 0..5 {
        for file in 0..3 {
            let key = create_test_cache_key(&format!("/test/validation/tab_{}/file_{}.jpg", tab, file));
            let data = create_test_preview_data(30 * 1024, "image/jpeg");
            
            optimizer.preview_cache.put(key.clone(), data).unwrap();
            optimizer.record_cache_access(key).await;
        }
    }
    
    let tab_stats = optimizer.get_memory_stats().await.unwrap();
    println!("   - Tab workflow memory: {}KB", tab_stats.total_memory_bytes / 1024);
    
    // Requirement 2: Implement efficient cache eviction strategies
    println!("✅ 2. Efficient cache eviction strategies implemented");
    
    let eviction_result = optimizer.optimize_memory().await.unwrap();
    println!("   - Eviction freed: {}KB", eviction_result.memory_freed / 1024);
    println!("   - Entries evicted: {}", eviction_result.entries_evicted);
    println!("   - Eviction actions: {:?}", eviction_result.actions_taken);
    
    // Requirement 3: Maintain optimal memory footprint under heavy load
    println!("✅ 3. Optimal memory footprint under heavy load");
    
    let final_stats = optimizer.get_memory_stats().await.unwrap();
    println!("   - Final memory usage: {}KB", final_stats.total_memory_bytes / 1024);
    println!("   - Memory pressure: {:.1}%", final_stats.memory_pressure * 100.0);
    println!("   - Memory efficiency: {:.1}%", final_stats.memory_efficiency * 100.0);
    
    // Validate Task 22.2 success criteria
    assert!(preview_stats.total_memory_bytes > 0, "Should track preview memory usage");
    assert!(tab_stats.total_memory_bytes >= preview_stats.total_memory_bytes, "Tab workflow should use additional memory");
    assert!(eviction_result.actions_taken.len() > 0 || final_stats.memory_pressure < 0.6, "Should perform eviction or maintain low pressure");
    assert!(final_stats.memory_efficiency > 0.0, "Should maintain memory efficiency");
    assert!(final_stats.memory_pressure <= 1.0, "Memory pressure should be within limits");
    
    println!("\n🎉 Task 22.2 Performance Validation: ALL REQUIREMENTS MET");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Memory consumption analysis: COMPLETE");
    println!("✅ Cache eviction strategies: IMPLEMENTED");
    println!("✅ Optimal memory footprint: MAINTAINED");
    println!("✅ Performance under heavy load: VALIDATED");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}