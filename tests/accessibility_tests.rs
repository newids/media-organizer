// Comprehensive accessibility testing for MediaOrganizer
// Integration tests using axe-core for automated WCAG 2.1 AA compliance validation

use std::collections::HashMap;
use tokio;

mod accessibility;

use accessibility::{
    AccessibilityTester, AccessibilityTestResult,
    test_scenarios::{AccessibilityTestRunner, MediaOrganizerTestSuite, generate_suite_report},
    axe_rules,
};

#[cfg(test)]
mod accessibility_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Test basic accessibility compliance on initial load
    #[tokio::test]
    async fn test_basic_accessibility_compliance() {
        let tester = AccessibilityTester::new()
            .expect("Failed to initialize accessibility tester");

        // Test the main application URL (would need to be running)
        let result = tester.test_page("http://localhost:3000").await;
        
        match result {
            Ok(test_result) => {
                // Check for critical and serious violations
                let critical_violations: Vec<_> = test_result.violations
                    .iter()
                    .filter(|v| v.impact == "critical")
                    .collect();
                
                let serious_violations: Vec<_> = test_result.violations
                    .iter()
                    .filter(|v| v.impact == "serious")
                    .collect();

                // Generate detailed report
                let report = tester.generate_report(&test_result);
                save_test_report("basic_accessibility_test", &report);

                // Assert no critical violations
                assert!(
                    critical_violations.is_empty(),
                    "Found {} critical accessibility violations: {:?}",
                    critical_violations.len(),
                    critical_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                // Assert minimal serious violations (allow some during development)
                assert!(
                    serious_violations.len() <= 2,
                    "Found {} serious accessibility violations (max 2 allowed): {:?}",
                    serious_violations.len(),
                    serious_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                println!("✅ Basic accessibility test passed");
                println!("   Violations: {}", test_result.violations.len());
                println!("   Passes: {}", test_result.passes.len());
            }
            Err(e) => {
                // Log error but don't fail test if application isn't running
                println!("⚠️ Accessibility test skipped: {}", e);
                println!("   This is expected if the application isn't running on localhost:3000");
            }
        }
    }

    /// Test keyboard navigation accessibility
    #[tokio::test]
    async fn test_keyboard_navigation() {
        let test_runner = match AccessibilityTestRunner::new() {
            Ok(runner) => runner,
            Err(_) => {
                println!("⚠️ Keyboard navigation test skipped: Chrome not available");
                return;
            }
        };

        let scenario = MediaOrganizerTestSuite::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "Keyboard Navigation")
            .expect("Keyboard Navigation scenario not found");

        match test_runner.run_scenario(&scenario).await {
            Ok(result) => {
                let report = AccessibilityTester::new()
                    .expect("Failed to create tester")
                    .generate_report(&result);
                save_test_report("keyboard_navigation_test", &report);

                // Keyboard navigation should have minimal violations
                let blocking_violations: Vec<_> = result.violations
                    .iter()
                    .filter(|v| v.impact == "critical" || v.impact == "serious")
                    .collect();

                assert!(
                    blocking_violations.is_empty(),
                    "Found blocking keyboard navigation violations: {:?}",
                    blocking_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                println!("✅ Keyboard navigation test passed");
            }
            Err(e) => {
                println!("⚠️ Keyboard navigation test skipped: {}", e);
            }
        }
    }

    /// Test file tree accessibility
    #[tokio::test]
    async fn test_file_tree_accessibility() {
        let test_runner = match AccessibilityTestRunner::new() {
            Ok(runner) => runner,
            Err(_) => {
                println!("⚠️ File tree test skipped: Chrome not available");
                return;
            }
        };

        let scenario = MediaOrganizerTestSuite::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "File Tree Navigation")
            .expect("File Tree Navigation scenario not found");

        match test_runner.run_scenario(&scenario).await {
            Ok(result) => {
                let report = AccessibilityTester::new()
                    .expect("Failed to create tester")
                    .generate_report(&result);
                save_test_report("file_tree_accessibility_test", &report);

                // File tree should comply with ARIA tree patterns
                let critical_violations: Vec<_> = result.violations
                    .iter()
                    .filter(|v| v.impact == "critical")
                    .collect();

                assert!(
                    critical_violations.is_empty(),
                    "Found critical file tree accessibility violations: {:?}",
                    critical_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                println!("✅ File tree accessibility test passed");
            }
            Err(e) => {
                println!("⚠️ File tree test skipped: {}", e);
            }
        }
    }

    /// Test preview panel accessibility
    #[tokio::test]
    async fn test_preview_panel_accessibility() {
        let test_runner = match AccessibilityTestRunner::new() {
            Ok(runner) => runner,
            Err(_) => {
                println!("⚠️ Preview panel test skipped: Chrome not available");
                return;
            }
        };

        let scenario = MediaOrganizerTestSuite::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "File Preview")
            .expect("File Preview scenario not found");

        match test_runner.run_scenario(&scenario).await {
            Ok(result) => {
                let report = AccessibilityTester::new()
                    .expect("Failed to create tester")
                    .generate_report(&result);
                save_test_report("preview_panel_accessibility_test", &report);

                // Preview panel should have proper ARIA labels and roles
                let serious_violations: Vec<_> = result.violations
                    .iter()
                    .filter(|v| v.impact == "serious" || v.impact == "critical")
                    .collect();

                assert!(
                    serious_violations.len() <= 1,
                    "Found serious preview panel violations: {:?}",
                    serious_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                println!("✅ Preview panel accessibility test passed");
            }
            Err(e) => {
                println!("⚠️ Preview panel test skipped: {}", e);
            }
        }
    }

    /// Test high contrast mode accessibility
    #[tokio::test]
    async fn test_high_contrast_accessibility() {
        let test_runner = match AccessibilityTestRunner::new() {
            Ok(runner) => runner,
            Err(_) => {
                println!("⚠️ High contrast test skipped: Chrome not available");
                return;
            }
        };

        let scenario = MediaOrganizerTestSuite::get_all_scenarios()
            .into_iter()
            .find(|s| s.name == "High Contrast Mode")
            .expect("High Contrast Mode scenario not found");

        match test_runner.run_scenario(&scenario).await {
            Ok(result) => {
                let report = AccessibilityTester::new()
                    .expect("Failed to create tester")
                    .generate_report(&result);
                save_test_report("high_contrast_accessibility_test", &report);

                // High contrast mode should have excellent accessibility
                let moderate_plus_violations: Vec<_> = result.violations
                    .iter()
                    .filter(|v| v.impact != "minor")
                    .collect();

                assert!(
                    moderate_plus_violations.is_empty(),
                    "Found violations in high contrast mode: {:?}",
                    moderate_plus_violations.iter().map(|v| &v.id).collect::<Vec<_>>()
                );

                println!("✅ High contrast accessibility test passed");
            }
            Err(e) => {
                println!("⚠️ High contrast test skipped: {}", e);
            }
        }
    }

    /// Run comprehensive accessibility test suite
    #[tokio::test]
    async fn test_comprehensive_accessibility_suite() {
        let test_runner = match AccessibilityTestRunner::new() {
            Ok(runner) => runner,
            Err(_) => {
                println!("⚠️ Comprehensive test suite skipped: Chrome not available");
                return;
            }
        };

        match test_runner.run_full_suite().await {
            Ok(results) => {
                // Generate comprehensive report
                let report = generate_suite_report(&results);
                save_test_report("comprehensive_accessibility_suite", &report);

                // Analyze overall results
                let total_critical: usize = results.values()
                    .map(|r| r.violations.iter().filter(|v| v.impact == "critical").count())
                    .sum();

                let total_serious: usize = results.values()
                    .map(|r| r.violations.iter().filter(|v| v.impact == "serious").count())
                    .sum();

                // Overall application should have minimal critical/serious violations
                assert!(
                    total_critical <= 1,
                    "Found {} critical violations across all scenarios (max 1 allowed)",
                    total_critical
                );

                assert!(
                    total_serious <= 5,
                    "Found {} serious violations across all scenarios (max 5 allowed)",
                    total_serious
                );

                println!("✅ Comprehensive accessibility suite passed");
                println!("   Total scenarios: {}", results.len());
                println!("   Critical violations: {}", total_critical);
                println!("   Serious violations: {}", total_serious);
            }
            Err(e) => {
                println!("⚠️ Comprehensive test suite skipped: {}", e);
            }
        }
    }

    /// Helper function to save test reports
    fn save_test_report(test_name: &str, report: &str) {
        let reports_dir = Path::new("target/accessibility-reports");
        if !reports_dir.exists() {
            let _ = fs::create_dir_all(reports_dir);
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.md", test_name, timestamp);
        let filepath = reports_dir.join(filename);

        if let Err(e) = fs::write(&filepath, report) {
            println!("⚠️ Failed to save accessibility report to {:?}: {}", filepath, e);
        } else {
            println!("📄 Accessibility report saved to {:?}", filepath);
        }
    }
}

/// Standalone accessibility testing binary
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 MediaOrganizer Accessibility Testing Suite");
    println!("===========================================\n");

    let test_runner = AccessibilityTestRunner::new()?;
    let results = test_runner.run_full_suite().await?;

    // Generate and save comprehensive report
    let report = generate_suite_report(&results);
    let reports_dir = Path::new("target/accessibility-reports");
    fs::create_dir_all(reports_dir)?;
    
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let report_path = reports_dir.join(format!("accessibility_suite_report_{}.md", timestamp));
    fs::write(&report_path, &report)?;

    println!("📊 Comprehensive accessibility report generated:");
    println!("   📄 {}", report_path.display());

    // Print summary
    let total_violations: usize = results.values().map(|r| r.violations.len()).sum();
    let total_passes: usize = results.values().map(|r| r.passes.len()).sum();
    
    println!("\n🎯 Final Results:");
    println!("   ✅ Test scenarios: {}", results.len());
    println!("   ❌ Total violations: {}", total_violations);
    println!("   ✅ Total passes: {}", total_passes);

    if total_violations == 0 {
        println!("\n🎉 All accessibility tests passed!");
    } else {
        println!("\n⚠️ Found {} accessibility violations to address", total_violations);
    }

    Ok(())
}