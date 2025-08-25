use std::time::{Duration, Instant};
use crate::state::{LayoutState, LayoutManager, Theme, ActivityBarView, SidebarContent, EditorLayoutConfig, PanelTab};
use crate::state::performance::{init_profiler, with_profiler, PerformanceStatus};

/// Comprehensive benchmark suite for layout state management performance
/// Tests various operations under different load conditions to ensure <100ms targets

pub struct LayoutBenchmarkSuite {
    /// Number of iterations for each benchmark
    iterations: usize,
    /// Whether to run warmup iterations
    run_warmup: bool,
    /// Results storage
    results: Vec<BenchmarkResult>,
}

/// Individual benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration: Duration,
    pub iterations: usize,
    pub operations_per_second: f64,
    pub avg_operation_time: Duration,
    pub success: bool,
    pub notes: String,
}

/// Benchmark test categories
#[derive(Debug, Clone)]
pub enum BenchmarkCategory {
    SignalOperations,
    BatchOperations,
    PersistenceOperations,
    ComplexScenarios,
    MemoryUsage,
    EdgeCases,
}

impl LayoutBenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(iterations: usize) -> Self {
        init_profiler();
        Self {
            iterations,
            run_warmup: true,
            results: Vec::new(),
        }
    }

    /// Run all benchmarks and return comprehensive results
    pub fn run_all_benchmarks(&mut self) -> BenchmarkSummary {
        println!("🚀 Starting Layout State Performance Benchmark Suite");
        println!("Target: All operations < 100ms, Excellent < 50ms");
        println!("Iterations per test: {}\n", self.iterations);

        // Signal operations benchmarks
        self.run_signal_benchmarks();
        
        // Batch operations benchmarks
        self.run_batch_benchmarks();
        
        // Persistence benchmarks
        self.run_persistence_benchmarks();
        
        // Complex scenario benchmarks
        self.run_complex_scenario_benchmarks();
        
        // Memory usage benchmarks
        self.run_memory_benchmarks();
        
        // Edge case benchmarks
        self.run_edge_case_benchmarks();

        self.generate_summary()
    }

    /// Benchmark basic signal operations
    fn run_signal_benchmarks(&mut self) {
        println!("📊 Running Signal Operations Benchmarks...");

        // Theme changes
        self.benchmark_theme_operations();
        
        // Sidebar operations
        self.benchmark_sidebar_operations();
        
        // Panel operations  
        self.benchmark_panel_operations();
        
        // Activity bar operations
        self.benchmark_activity_bar_operations();
        
        // Editor operations
        self.benchmark_editor_operations();
    }

    fn benchmark_theme_operations(&mut self) {
        let result = self.run_benchmark("Theme Toggle Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            // Warmup
            if self.run_warmup {
                for _ in 0..10 {
                    manager.toggle_theme();
                }
            }
            
            let start = Instant::now();
            for _ in 0..self.iterations {
                manager.toggle_theme();
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    fn benchmark_sidebar_operations(&mut self) {
        let result = self.run_benchmark("Sidebar Toggle Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..10 {
                    manager.toggle_sidebar();
                }
            }
            
            let start = Instant::now();
            for _ in 0..self.iterations {
                manager.toggle_sidebar();
                manager.set_sidebar_width(300.0 + (250.0 * (rand::random::<f64>())));
                manager.set_sidebar_content(SidebarContent::FileTree);
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    fn benchmark_panel_operations(&mut self) {
        let result = self.run_benchmark("Panel Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..10 {
                    manager.toggle_panel();
                }
            }
            
            let start = Instant::now();
            for i in 0..self.iterations {
                manager.toggle_panel();
                manager.set_panel_height(200.0 + (100.0 * (i as f64 % 3.0)));
                manager.set_panel_tab(match i % 4 {
                    0 => PanelTab::Problems,
                    1 => PanelTab::Output,
                    2 => PanelTab::Terminal,
                    _ => PanelTab::Debug,
                });
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    fn benchmark_activity_bar_operations(&mut self) {
        let result = self.run_benchmark("Activity Bar Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..10 {
                    manager.toggle_activity_bar();
                }
            }
            
            let start = Instant::now();
            for i in 0..self.iterations {
                manager.toggle_activity_bar();
                manager.set_activity_bar_view(match i % 5 {
                    0 => ActivityBarView::Explorer,
                    1 => ActivityBarView::Search,
                    2 => ActivityBarView::SourceControl,
                    3 => ActivityBarView::Debug,
                    _ => ActivityBarView::Extensions,
                });
                manager.set_activity_bar_width(40.0 + (i as f64 % 20.0));
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    fn benchmark_editor_operations(&mut self) {
        let result = self.run_benchmark("Editor Layout Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..10 {
                    manager.set_editor_layout(EditorLayoutConfig::Single);
                }
            }
            
            let start = Instant::now();
            for i in 0..self.iterations {
                let layout = match i % 4 {
                    0 => EditorLayoutConfig::Single,
                    1 => EditorLayoutConfig::SplitHorizontal,
                    2 => EditorLayoutConfig::SplitVertical,
                    _ => EditorLayoutConfig::Grid { rows: 2, cols: 2 },
                };
                manager.set_editor_layout(layout);
                manager.set_active_editor_group(i % 4);
                manager.toggle_editor_tabs();
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    /// Benchmark batch operations
    fn run_batch_benchmarks(&mut self) {
        println!("📦 Running Batch Operations Benchmarks...");

        let result = self.run_benchmark("Batch Update Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..5 {
                    manager.queue_theme_update(Theme::Dark);
                    manager.apply_pending_updates();
                }
            }
            
            let start = Instant::now();
            for i in 0..self.iterations {
                // Queue multiple updates
                manager.queue_theme_update(if i % 2 == 0 { Theme::Dark } else { Theme::Light });
                
                // Apply batch
                manager.apply_pending_updates();
            }
            start.elapsed()
        });
        
        self.results.push(result);

        // Large batch test
        let large_batch_result = self.run_benchmark("Large Batch Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            let start = Instant::now();
            for batch in 0..(self.iterations / 10) {
                // Queue 10 updates per batch
                for i in 0..10 {
                    manager.queue_theme_update(if (batch + i) % 2 == 0 { Theme::Dark } else { Theme::Light });
                }
                manager.apply_pending_updates();
            }
            start.elapsed()
        });
        
        self.results.push(large_batch_result);
    }

    /// Benchmark persistence operations
    fn run_persistence_benchmarks(&mut self) {
        println!("💾 Running Persistence Operations Benchmarks...");

        let result = self.run_benchmark("State Save Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            if self.run_warmup {
                for _ in 0..5 {
                    manager.save_state();
                }
            }
            
            let start = Instant::now();
            for _ in 0..self.iterations {
                manager.save_state();
            }
            start.elapsed()
        });
        
        self.results.push(result);
    }

    /// Benchmark complex real-world scenarios
    fn run_complex_scenario_benchmarks(&mut self) {
        println!("🎭 Running Complex Scenario Benchmarks...");

        // VS Code-like workflow simulation
        let vscode_workflow_result = self.run_benchmark("VS Code Workflow Simulation", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            let start = Instant::now();
            for i in 0..(self.iterations / 5) {
                // Simulate opening file explorer
                manager.set_activity_bar_view(ActivityBarView::Explorer);
                manager.set_sidebar_content(SidebarContent::FileTree);
                
                // Simulate opening files and splitting editor
                manager.set_editor_layout(EditorLayoutConfig::SplitVertical);
                manager.set_active_editor_group(0);
                
                // Simulate search
                manager.set_activity_bar_view(ActivityBarView::Search);
                
                // Simulate opening terminal
                manager.toggle_panel();
                manager.set_panel_tab(PanelTab::Terminal);
                
                // Simulate theme change (happens occasionally)
                if i % 10 == 0 {
                    manager.toggle_theme();
                }
                
                // Auto-save simulation
                if i % 3 == 0 {
                    manager.save_state();
                }
            }
            start.elapsed()
        });
        
        self.results.push(vscode_workflow_result);

        // Responsive layout changes
        let responsive_result = self.run_benchmark("Responsive Layout Changes", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            let start = Instant::now();
            for i in 0..self.iterations {
                let width = 400.0 + (1200.0 * rand::random::<f64>());
                let height = 300.0 + (800.0 * rand::random::<f64>());
                
                manager.set_viewport_dimensions(width, height);
                manager.apply_responsive_adjustments();
                
                if i % 10 == 0 {
                    manager.apply_pending_updates();
                }
            }
            start.elapsed()
        });
        
        self.results.push(responsive_result);
    }

    /// Benchmark memory usage patterns
    fn run_memory_benchmarks(&mut self) {
        println!("🧠 Running Memory Usage Benchmarks...");

        let result = self.run_benchmark("Memory Usage Pattern", || {
            let start = Instant::now();
            
            // Create many layout managers to test memory scaling
            let mut managers = Vec::new();
            for _ in 0..(self.iterations / 10) {
                let layout_state = LayoutState::default();
                let mut manager = LayoutManager::with_state(layout_state);
                
                // Perform operations to accumulate state
                manager.toggle_theme();
                manager.toggle_sidebar();
                manager.save_state();
                
                managers.push(manager);
            }
            
            // Force cleanup
            drop(managers);
            
            start.elapsed()
        });
        
        self.results.push(result);
    }

    /// Benchmark edge cases and stress conditions
    fn run_edge_case_benchmarks(&mut self) {
        println!("⚡ Running Edge Case Benchmarks...");

        // Rapid consecutive operations
        let rapid_ops_result = self.run_benchmark("Rapid Consecutive Operations", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            let start = Instant::now();
            for _ in 0..self.iterations {
                manager.toggle_theme();
                manager.toggle_sidebar();
                manager.toggle_panel();
                manager.toggle_activity_bar();
                manager.set_sidebar_width(300.0);
                manager.set_panel_height(200.0);
            }
            start.elapsed()
        });
        
        self.results.push(rapid_ops_result);

        // Extreme values test
        let extreme_values_result = self.run_benchmark("Extreme Values Handling", || {
            let layout_state = LayoutState::default();
            let mut manager = LayoutManager::with_state(layout_state);
            
            let start = Instant::now();
            for i in 0..self.iterations {
                // Test with extreme dimensions
                manager.set_viewport_dimensions(if i % 2 == 0 { 1.0 } else { 10000.0 }, 
                                              if i % 3 == 0 { 1.0 } else { 10000.0 });
                
                // Test with extreme sidebar width
                manager.set_sidebar_width(if i % 4 == 0 { 1.0 } else { 2000.0 });
                
                // Test with extreme panel height
                manager.set_panel_height(if i % 5 == 0 { 1.0 } else { 1000.0 });
            }
            start.elapsed()
        });
        
        self.results.push(extreme_values_result);
    }

    /// Helper method to run a single benchmark
    fn run_benchmark<F>(&self, name: &str, benchmark_fn: F) -> BenchmarkResult
    where
        F: FnOnce() -> Duration,
    {
        print!("  Running {}... ", name);
        
        let duration = benchmark_fn();
        let avg_operation_time = duration / self.iterations as u32;
        let operations_per_second = if duration.as_secs_f64() > 0.0 {
            self.iterations as f64 / duration.as_secs_f64()
        } else {
            f64::INFINITY
        };
        
        let success = avg_operation_time.as_millis() < 100;
        let grade = if avg_operation_time.as_millis() < 50 {
            "✅ EXCELLENT"
        } else if avg_operation_time.as_millis() < 100 {
            "✅ GOOD"
        } else if avg_operation_time.as_millis() < 200 {
            "⚠️  ACCEPTABLE"
        } else {
            "❌ POOR"
        };
        
        println!("{} ({:.2}ms avg)", grade, avg_operation_time.as_secs_f64() * 1000.0);
        
        BenchmarkResult {
            test_name: name.to_string(),
            duration,
            iterations: self.iterations,
            operations_per_second,
            avg_operation_time,
            success,
            notes: format!("Grade: {}", grade),
        }
    }

    /// Generate comprehensive benchmark summary
    fn generate_summary(&self) -> BenchmarkSummary {
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let failed_tests = total_tests - successful_tests;
        
        let avg_operation_time = if !self.results.is_empty() {
            Duration::from_nanos(
                (self.results.iter().map(|r| r.avg_operation_time.as_nanos()).sum::<u128>() 
                 / self.results.len() as u128) as u64
            )
        } else {
            Duration::ZERO
        };
        
        let slowest_operation = self.results.iter()
            .max_by_key(|r| r.avg_operation_time.as_nanos())
            .cloned();
            
        let fastest_operation = self.results.iter()
            .min_by_key(|r| r.avg_operation_time.as_nanos())
            .cloned();

        let performance_grade = if failed_tests == 0 && avg_operation_time.as_millis() < 50 {
            'A'
        } else if failed_tests <= 1 && avg_operation_time.as_millis() < 100 {
            'B'
        } else if failed_tests <= 2 && avg_operation_time.as_millis() < 150 {
            'C'
        } else if failed_tests <= 3 {
            'D'
        } else {
            'F'
        };

        // Get profiler status
        let profiler_status = with_profiler(|profiler| profiler.get_current_status())
            .unwrap_or(PerformanceStatus::Unknown);

        BenchmarkSummary {
            total_tests,
            successful_tests,
            failed_tests,
            avg_operation_time,
            slowest_operation,
            fastest_operation,
            performance_grade,
            profiler_status,
            detailed_results: self.results.clone(),
        }
    }
}

/// Summary of all benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    pub total_tests: usize,
    pub successful_tests: usize,
    pub failed_tests: usize,
    pub avg_operation_time: Duration,
    pub slowest_operation: Option<BenchmarkResult>,
    pub fastest_operation: Option<BenchmarkResult>,
    pub performance_grade: char,
    pub profiler_status: PerformanceStatus,
    pub detailed_results: Vec<BenchmarkResult>,
}

impl BenchmarkSummary {
    /// Print a detailed summary report
    pub fn print_detailed_report(&self) {
        println!("\n{}", "=".repeat(80));
        println!("🎯 LAYOUT STATE MANAGEMENT PERFORMANCE REPORT");
        println!("{}", "=".repeat(80));
        
        println!("\n📊 SUMMARY STATISTICS");
        println!("  Total Tests:      {}", self.total_tests);
        println!("  Successful:       {} ({:.1}%)", 
                self.successful_tests, 
                (self.successful_tests as f64 / self.total_tests as f64) * 100.0);
        println!("  Failed:           {} ({:.1}%)", 
                self.failed_tests,
                (self.failed_tests as f64 / self.total_tests as f64) * 100.0);
        println!("  Overall Grade:    {}", self.performance_grade);
        println!("  Profiler Status:  {:?}", self.profiler_status);
        println!("  Avg Operation:    {:.2}ms", self.avg_operation_time.as_secs_f64() * 1000.0);
        
        if let Some(slowest) = &self.slowest_operation {
            println!("  Slowest Test:     {} ({:.2}ms)", 
                    slowest.test_name, 
                    slowest.avg_operation_time.as_secs_f64() * 1000.0);
        }
        
        if let Some(fastest) = &self.fastest_operation {
            println!("  Fastest Test:     {} ({:.2}ms)", 
                    fastest.test_name, 
                    fastest.avg_operation_time.as_secs_f64() * 1000.0);
        }

        println!("\n📈 DETAILED RESULTS");
        println!("{:<35} {:<12} {:<15} {:<10}", "Test Name", "Avg Time", "Ops/Sec", "Status");
        println!("{:-<72}", "");
        
        for result in &self.detailed_results {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            println!("{:<35} {:<12.2}ms {:<15.0} {:<10}", 
                    result.test_name,
                    result.avg_operation_time.as_secs_f64() * 1000.0,
                    result.operations_per_second,
                    status);
        }

        println!("\n🎯 PERFORMANCE RECOMMENDATIONS");
        
        if self.performance_grade == 'A' {
            println!("  🎉 Excellent performance! All targets met.");
            println!("  💡 Consider enabling more advanced features or optimizations.");
        } else if self.performance_grade == 'B' {
            println!("  ✅ Good performance with room for minor improvements.");
            println!("  💡 Focus on optimizing the slowest operations.");
        } else if self.performance_grade >= 'C' {
            println!("  ⚠️  Performance needs attention.");
            println!("  💡 Consider implementing more aggressive batching.");
            println!("  💡 Review signal usage patterns for unnecessary re-renders.");
        } else {
            println!("  🚨 Critical performance issues detected!");
            println!("  💡 Implement immediate performance optimizations.");
            println!("  💡 Consider redesigning state management approach.");
            println!("  💡 Profile individual operations for bottlenecks.");
        }

        if self.failed_tests > 0 {
            println!("\n❌ FAILED TESTS ANALYSIS");
            for result in &self.detailed_results {
                if !result.success {
                    println!("  {} - {:.2}ms (target: <100ms)", 
                            result.test_name,
                            result.avg_operation_time.as_secs_f64() * 1000.0);
                }
            }
        }

        println!("\n{}", "=".repeat(80));
    }

    /// Get performance score (0-100)
    pub fn get_performance_score(&self) -> u8 {
        let success_ratio = self.successful_tests as f64 / self.total_tests as f64;
        let time_score = if self.avg_operation_time.as_millis() <= 50 {
            1.0
        } else if self.avg_operation_time.as_millis() <= 100 {
            0.8
        } else if self.avg_operation_time.as_millis() <= 150 {
            0.6
        } else {
            0.3
        };
        
        ((success_ratio * 0.7 + time_score * 0.3) * 100.0) as u8
    }
}

/// Run quick performance check (subset of full benchmarks)
pub fn run_quick_performance_check() -> BenchmarkSummary {
    let mut suite = LayoutBenchmarkSuite::new(50);
    suite.run_warmup = false;
    
    println!("🔍 Running Quick Performance Check...");
    
    // Run core operations only
    suite.benchmark_theme_operations();
    suite.benchmark_sidebar_operations(); 
    suite.benchmark_panel_operations();
    
    let summary = suite.generate_summary();
    println!("Quick check complete - Grade: {}", summary.performance_grade);
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            test_name: "Test Operation".to_string(),
            duration: Duration::from_millis(100),
            iterations: 100,
            operations_per_second: 1000.0,
            avg_operation_time: Duration::from_millis(1),
            success: true,
            notes: "Test notes".to_string(),
        };
        
        assert_eq!(result.test_name, "Test Operation");
        assert!(result.success);
        assert_eq!(result.iterations, 100);
    }

    #[test]
    fn test_performance_score_calculation() {
        let summary = BenchmarkSummary {
            total_tests: 10,
            successful_tests: 8,
            failed_tests: 2,
            avg_operation_time: Duration::from_millis(75),
            slowest_operation: None,
            fastest_operation: None,
            performance_grade: 'B',
            profiler_status: PerformanceStatus::Good,
            detailed_results: vec![],
        };
        
        let score = summary.get_performance_score();
        assert!(score > 50); // Should be a reasonable score
        assert!(score <= 100);
    }

    #[test] 
    fn test_benchmark_suite_creation() {
        let suite = LayoutBenchmarkSuite::new(10);
        assert_eq!(suite.iterations, 10);
        assert!(suite.run_warmup);
        assert_eq!(suite.results.len(), 0);
    }
}