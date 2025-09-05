use media_organizer::{
    run_performance_benchmarks, create_benchmark_suite, init_performance_systems,
    BenchmarkResults, PerformanceGrade
};
use std::time::Duration;
use tokio;
use tracing_test::traced_test;

/// Test comprehensive performance profiling as per Task 22.1
#[tokio::test]
#[traced_test]
async fn test_performance_profiling() {
    println!("🔍 Task 22.1: Profiling UI Rendering and GPU-Accelerated Preview Performance");
    
    // Initialize tracing for performance monitoring
    tracing::info!("Initializing performance profiling test");
    
    // Test 1: UI Performance Profiler Initialization
    println!("\n📊 Testing UI Performance Profiler Initialization...");
    let ui_profiler = init_performance_systems().await.unwrap();
    let metrics = ui_profiler.get_metrics();
    
    println!("✅ UI Performance Profiler initialized");
    println!("   - Layout samples: {}", metrics.layout_performance.layout_samples);
    println!("   - Theme samples: {}", metrics.theme_performance.theme_samples);
    println!("   - GPU samples: {}", metrics.gpu_performance.gpu_samples);
    
    // Test 2: Benchmark Suite Creation
    println!("\n🏗️ Testing Benchmark Suite Creation...");
    let suite = create_benchmark_suite().await.unwrap();
    println!("✅ Benchmark suite created successfully");
    
    // Test 3: Run Performance Benchmarks
    println!("\n⚡ Running Performance Benchmarks...");
    let start_time = std::time::Instant::now();
    let results = run_performance_benchmarks().await.unwrap();
    let benchmark_duration = start_time.elapsed();
    
    println!("✅ Benchmark completed in {:.2}s", benchmark_duration.as_secs_f64());
    
    // Test 4: Validate Performance Targets (Task 22.1 requirements)
    validate_performance_targets(&results).await;
    
    // Test 5: Display Results Summary
    display_performance_summary(&results).await;
}

/// Test that performance targets are met as specified in Task 22.1
#[tokio::test]
async fn test_performance_targets() {
    println!("🎯 Task 22.1: Validating Performance Targets");
    
    let results = run_performance_benchmarks().await.unwrap();
    
    // Target: <100ms layout
    let layout_target_met = results.layout_benchmarks.target_met;
    println!("📐 Layout Performance Target (<100ms): {}", 
        if layout_target_met { "✅ MET" } else { "❌ NOT MET" });
    println!("   Average: {:.1}ms, P95: {:.1}ms", 
        results.layout_benchmarks.average_ms,
        results.layout_benchmarks.p95_ms);
    
    // Target: <50ms theme switch
    let theme_target_met = results.theme_benchmarks.target_met;
    println!("🎨 Theme Switch Performance Target (<50ms): {}", 
        if theme_target_met { "✅ MET" } else { "❌ NOT MET" });
    println!("   Average: {:.1}ms, P95: {:.1}ms", 
        results.theme_benchmarks.average_ms,
        results.theme_benchmarks.p95_ms);
    
    // GPU Performance (if available)
    if let Some(ref gpu_results) = results.gpu_benchmarks {
        let gpu_60fps_target = gpu_results.average_render_ms <= 16.67; // 60 FPS
        println!("🖥️ GPU Render Performance (60 FPS target): {}", 
            if gpu_60fps_target { "✅ MET" } else { "⚠️ REVIEW" });
        println!("   Average: {:.1}ms, Throughput: {:.1} textures/sec", 
            gpu_results.average_render_ms,
            gpu_results.throughput_textures_per_second);
        
        if let Some(ref adapter_info) = gpu_results.adapter_info {
            println!("   GPU: {}", adapter_info);
        }
    } else {
        println!("🖥️ GPU Acceleration: ❌ NOT AVAILABLE");
    }
    
    // Overall Performance Grade
    println!("\n📊 Overall Performance Grade: {:?}", results.performance_grade);
    
    // Assert critical requirements are met
    assert!(
        matches!(results.performance_grade, PerformanceGrade::Good | PerformanceGrade::Excellent),
        "Performance grade should be Good or Excellent, got {:?}", results.performance_grade
    );
}

/// Test UI profiler functionality independently
#[tokio::test]
async fn test_ui_profiler() {
    println!("🔬 Testing UI Profiler Functionality");
    
    let ui_profiler = init_performance_systems().await.unwrap();
    
    // Test layout measurement
    let layout_measurement = ui_profiler.start_layout_measurement(
        "test_layout_operation".to_string(),
        media_organizer::services::ui_profiler::LayoutType::ComponentUpdate
    );
    
    // Simulate some work
    tokio::time::sleep(Duration::from_millis(10)).await;
    layout_measurement.finish();
    
    // Test theme measurement
    let theme_measurement = ui_profiler.start_theme_measurement(
        "dark".to_string(),
        "light".to_string()
    );
    
    tokio::time::sleep(Duration::from_millis(5)).await;
    theme_measurement.finish();
    
    // Check metrics were recorded
    let metrics = ui_profiler.get_metrics();
    println!("✅ UI Profiler recorded {} layout samples", metrics.layout_performance.layout_samples);
    println!("✅ UI Profiler recorded {} theme samples", metrics.theme_performance.theme_samples);
    
    assert!(metrics.layout_performance.layout_samples > 0);
    assert!(metrics.theme_performance.theme_samples > 0);
}

/// Validate that performance targets are met
async fn validate_performance_targets(results: &BenchmarkResults) {
    println!("\n🎯 Performance Target Validation:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut passed = 0;
    let mut total = 0;
    
    // Layout Performance Target: <100ms
    total += 1;
    if results.layout_benchmarks.target_met {
        passed += 1;
        println!("✅ Layout performance: {:.1}ms (target: <100ms)", 
            results.layout_benchmarks.average_ms);
    } else {
        println!("❌ Layout performance: {:.1}ms (target: <100ms) - FAILED", 
            results.layout_benchmarks.average_ms);
    }
    
    // Theme Switch Target: <50ms
    total += 1;
    if results.theme_benchmarks.target_met {
        passed += 1;
        println!("✅ Theme switch: {:.1}ms (target: <50ms)", 
            results.theme_benchmarks.average_ms);
    } else {
        println!("❌ Theme switch: {:.1}ms (target: <50ms) - FAILED", 
            results.theme_benchmarks.average_ms);
    }
    
    // GPU Performance (if available)
    if let Some(ref gpu_results) = results.gpu_benchmarks {
        total += 1;
        if gpu_results.average_render_ms <= 16.67 {
            passed += 1;
            println!("✅ GPU render: {:.1}ms (target: <16.67ms for 60 FPS)", 
                gpu_results.average_render_ms);
        } else {
            println!("⚠️ GPU render: {:.1}ms (target: <16.67ms for 60 FPS) - REVIEW", 
                gpu_results.average_render_ms);
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Performance Summary: {}/{} targets met", passed, total);
    
    if passed == total {
        println!("🎉 All performance targets achieved!");
    } else {
        println!("⚠️ Some performance targets need attention");
    }
}

/// Display detailed performance summary
async fn display_performance_summary(results: &BenchmarkResults) {
    println!("\n📈 Detailed Performance Analysis:");
    println!("════════════════════════════════════════");
    
    // Layout Benchmarks
    println!("🏗️ Layout Performance:");
    println!("   Average: {:.1}ms", results.layout_benchmarks.average_ms);
    println!("   Median: {:.1}ms", results.layout_benchmarks.median_ms);
    println!("   P95: {:.1}ms", results.layout_benchmarks.p95_ms);
    println!("   P99: {:.1}ms", results.layout_benchmarks.p99_ms);
    println!("   Operations/sec: {:.1}", results.layout_benchmarks.operations_per_second);
    println!("   Component Update: {:.1}ms", results.layout_benchmarks.component_update_ms);
    println!("   State Change: {:.1}ms", results.layout_benchmarks.state_change_ms);
    
    // Theme Benchmarks
    println!("\n🎨 Theme Switch Performance:");
    println!("   Average: {:.1}ms", results.theme_benchmarks.average_ms);
    println!("   P95: {:.1}ms", results.theme_benchmarks.p95_ms);
    println!("   CSS Recalc: {:.1}ms", results.theme_benchmarks.css_recalc_ms);
    println!("   Style Invalidation: {:.1}ms", results.theme_benchmarks.style_invalidation_ms);
    println!("   Repaint: {:.1}ms", results.theme_benchmarks.repaint_ms);
    
    // GPU Benchmarks
    if let Some(ref gpu_results) = results.gpu_benchmarks {
        println!("\n🖥️ GPU Performance:");
        println!("   Render Time: {:.1}ms", gpu_results.average_render_ms);
        println!("   Texture Upload: {:.1}ms", gpu_results.texture_upload_ms);
        println!("   Shader Compile: {:.1}ms", gpu_results.shader_compile_ms);
        println!("   Memory Usage: {:.1}MB", gpu_results.memory_usage_mb);
        println!("   Throughput: {:.1} textures/sec", gpu_results.throughput_textures_per_second);
    }
    
    // Memory Usage
    println!("\n💾 Memory Performance:");
    println!("   Peak Usage: {:.1}MB", results.memory_benchmarks.peak_usage_mb);
    println!("   Average Usage: {:.1}MB", results.memory_benchmarks.average_usage_mb);
    println!("   Efficiency: {:.1}%", results.memory_benchmarks.memory_efficiency * 100.0);
    
    // Recommendations
    if !results.recommendations.is_empty() {
        println!("\n💡 Performance Recommendations:");
        for (i, rec) in results.recommendations.iter().enumerate() {
            println!("   {}. {}", i + 1, rec);
        }
    }
    
    println!("\n🏆 Overall Grade: {:?}", results.performance_grade);
    println!("════════════════════════════════════════");
}