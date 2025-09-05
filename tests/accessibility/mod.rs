// Accessibility Testing Framework for MediaOrganizer
// Comprehensive automated accessibility testing using axe-core and custom rules

use std::time::Duration;
use std::path::PathBuf;
use headless_chrome::{Browser, LaunchOptions, Tab};
use serde_json::{json, Value};
use tokio::time::sleep;

pub mod axe_rules;
pub mod test_scenarios;

/// Accessibility test result containing violations and passes
#[derive(Debug, Clone)]
pub struct AccessibilityTestResult {
    pub url: String,
    pub violations: Vec<AxeViolation>,
    pub passes: Vec<AxeResult>,
    pub incomplete: Vec<AxeResult>,
    pub inapplicable: Vec<AxeResult>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub test_duration_ms: u64,
}

/// Individual axe-core violation
#[derive(Debug, Clone)]
pub struct AxeViolation {
    pub id: String,
    pub impact: String, // critical, serious, moderate, minor
    pub description: String,
    pub help: String,
    pub help_url: String,
    pub tags: Vec<String>,
    pub nodes: Vec<AxeNode>,
}

/// Axe test result for passes, incomplete, or inapplicable
#[derive(Debug, Clone)]
pub struct AxeResult {
    pub id: String,
    pub description: String,
    pub tags: Vec<String>,
    pub nodes: Vec<AxeNode>,
}

/// Individual DOM node with accessibility info
#[derive(Debug, Clone)]
pub struct AxeNode {
    pub html: String,
    pub target: Vec<String>,
    pub failure_summary: Option<String>,
}

/// Accessibility testing engine
pub struct AccessibilityTester {
    browser: Browser,
    axe_script: String,
}

impl AccessibilityTester {
    /// Create a new accessibility tester with headless Chrome
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let launch_options = LaunchOptions::default_builder()
            .headless(true)
            .window_size(Some((1920, 1080)))
            .args(vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                "--disable-extensions".to_string(),
            ])
            .build()?;

        let browser = Browser::new(launch_options)?;
        let axe_script = Self::load_axe_core_script()?;

        Ok(AccessibilityTester {
            browser,
            axe_script,
        })
    }

    /// Load axe-core JavaScript from CDN or local copy
    fn load_axe_core_script() -> Result<String, Box<dyn std::error::Error>> {
        // First try to load from a local copy, then fallback to CDN
        let local_axe_path = PathBuf::from("tests/accessibility/axe-core.min.js");
        
        if local_axe_path.exists() {
            Ok(std::fs::read_to_string(local_axe_path)?)
        } else {
            // Download axe-core from CDN for testing
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let response = reqwest::get("https://unpkg.com/axe-core@latest/axe.min.js").await?;
                let axe_script = response.text().await?;
                
                // Cache it locally for future use
                std::fs::write(&local_axe_path, &axe_script)?;
                Ok::<String, Box<dyn std::error::Error>>(axe_script)
            })
        }
    }

    /// Test accessibility of a URL/page
    pub async fn test_page(&self, url: &str) -> Result<AccessibilityTestResult, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let tab = self.browser.new_tab()?;
        
        // Navigate to the page
        tab.navigate_to(url)?;
        
        // Wait for page to load
        tab.wait_until_navigated()?;
        sleep(Duration::from_secs(2)).await;

        // Inject axe-core
        tab.evaluate(&self.axe_script, false)?;

        // Configure axe with our custom rules
        let axe_config = self.get_axe_configuration();
        tab.evaluate(&format!(
            "window.axe.configure({});", 
            serde_json::to_string(&axe_config)?
        ), false)?;

        // Run axe accessibility scan
        let axe_results = tab.evaluate(
            "axe.run().then(results => JSON.stringify(results))",
            true
        )?.value.unwrap().as_str().unwrap().to_string();

        let results: Value = serde_json::from_str(&axe_results)?;
        
        let test_duration = start_time.elapsed().as_millis() as u64;
        
        Ok(self.parse_axe_results(url, results, test_duration))
    }

    /// Get axe-core configuration with MediaOrganizer-specific rules
    fn get_axe_configuration(&self) -> Value {
        json!({
            "rules": {
                // Enable comprehensive rule set
                "aria-allowed-attr": { "enabled": true },
                "aria-command-name": { "enabled": true },
                "aria-hidden-body": { "enabled": true },
                "aria-hidden-focus": { "enabled": true },
                "aria-input-field-name": { "enabled": true },
                "aria-meter-name": { "enabled": true },
                "aria-progressbar-name": { "enabled": true },
                "aria-required-attr": { "enabled": true },
                "aria-required-children": { "enabled": true },
                "aria-required-parent": { "enabled": true },
                "aria-roles": { "enabled": true },
                "aria-toggle-field-name": { "enabled": true },
                "aria-tooltip-name": { "enabled": true },
                "aria-valid-attr": { "enabled": true },
                "aria-valid-attr-value": { "enabled": true },
                "button-name": { "enabled": true },
                "bypass": { "enabled": true },
                "color-contrast": { "enabled": true },
                "focus-order-semantics": { "enabled": true },
                "frame-title": { "enabled": true },
                "html-has-lang": { "enabled": true },
                "html-lang-valid": { "enabled": true },
                "image-alt": { "enabled": true },
                "input-image-alt": { "enabled": true },
                "keyboard": { "enabled": true },
                "label": { "enabled": true },
                "landmark-banner": { "enabled": true },
                "landmark-main": { "enabled": true },
                "landmark-one-main": { "enabled": true },
                "landmark-unique": { "enabled": true },
                "link-name": { "enabled": true },
                "list": { "enabled": true },
                "listitem": { "enabled": true },
                "meta-viewport": { "enabled": true },
                "nested-interactive": { "enabled": true },
                "no-autoplay-audio": { "enabled": true },
                "page-has-heading-one": { "enabled": true },
                "region": { "enabled": true },
                "scope-attr-valid": { "enabled": true },
                "server-side-image-map": { "enabled": true },
                "svg-img-alt": { "enabled": true },
                "tabindex": { "enabled": true },
                "valid-lang": { "enabled": true }
            },
            "tags": ["wcag2a", "wcag2aa", "wcag21aa", "best-practice"],
            "resultTypes": ["violations", "passes", "incomplete", "inapplicable"],
            "runOnly": {
                "type": "tag",
                "values": ["wcag2a", "wcag2aa", "wcag21aa"]
            }
        })
    }

    /// Parse axe-core results into our format
    fn parse_axe_results(&self, url: &str, results: Value, duration: u64) -> AccessibilityTestResult {
        let violations = results["violations"].as_array().unwrap_or(&vec![])
            .iter()
            .map(|v| self.parse_violation(v))
            .collect();

        let passes = results["passes"].as_array().unwrap_or(&vec![])
            .iter()
            .map(|p| self.parse_result(p))
            .collect();

        let incomplete = results["incomplete"].as_array().unwrap_or(&vec![])
            .iter()
            .map(|i| self.parse_result(i))
            .collect();

        let inapplicable = results["inapplicable"].as_array().unwrap_or(&vec![])
            .iter()
            .map(|i| self.parse_result(i))
            .collect();

        AccessibilityTestResult {
            url: url.to_string(),
            violations,
            passes,
            incomplete,
            inapplicable,
            timestamp: chrono::Utc::now(),
            test_duration_ms: duration,
        }
    }

    fn parse_violation(&self, violation: &Value) -> AxeViolation {
        AxeViolation {
            id: violation["id"].as_str().unwrap_or("unknown").to_string(),
            impact: violation["impact"].as_str().unwrap_or("unknown").to_string(),
            description: violation["description"].as_str().unwrap_or("").to_string(),
            help: violation["help"].as_str().unwrap_or("").to_string(),
            help_url: violation["helpUrl"].as_str().unwrap_or("").to_string(),
            tags: violation["tags"].as_array().unwrap_or(&vec![])
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect(),
            nodes: violation["nodes"].as_array().unwrap_or(&vec![])
                .iter()
                .map(|n| self.parse_node(n))
                .collect(),
        }
    }

    fn parse_result(&self, result: &Value) -> AxeResult {
        AxeResult {
            id: result["id"].as_str().unwrap_or("unknown").to_string(),
            description: result["description"].as_str().unwrap_or("").to_string(),
            tags: result["tags"].as_array().unwrap_or(&vec![])
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect(),
            nodes: result["nodes"].as_array().unwrap_or(&vec![])
                .iter()
                .map(|n| self.parse_node(n))
                .collect(),
        }
    }

    fn parse_node(&self, node: &Value) -> AxeNode {
        AxeNode {
            html: node["html"].as_str().unwrap_or("").to_string(),
            target: node["target"].as_array().unwrap_or(&vec![])
                .iter()
                .filter_map(|t| t.as_str().map(|s| s.to_string()))
                .collect(),
            failure_summary: node["failureSummary"].as_str().map(|s| s.to_string()),
        }
    }

    /// Generate detailed accessibility report
    pub fn generate_report(&self, results: &AccessibilityTestResult) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("# Accessibility Test Report\n\n"));
        report.push_str(&format!("**URL:** {}\n", results.url));
        report.push_str(&format!("**Test Date:** {}\n", results.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
        report.push_str(&format!("**Duration:** {}ms\n\n", results.test_duration_ms));

        // Summary
        report.push_str(&format!("## Summary\n\n"));
        report.push_str(&format!("- **Violations:** {} (Critical/Serious issues that must be fixed)\n", results.violations.len()));
        report.push_str(&format!("- **Passes:** {} (Tests that passed successfully)\n", results.passes.len()));
        report.push_str(&format!("- **Incomplete:** {} (Tests that could not be completed)\n", results.incomplete.len()));
        report.push_str(&format!("- **Inapplicable:** {} (Tests not applicable to this page)\n\n", results.inapplicable.len()));

        // Violations by severity
        let critical_violations: Vec<_> = results.violations.iter().filter(|v| v.impact == "critical").collect();
        let serious_violations: Vec<_> = results.violations.iter().filter(|v| v.impact == "serious").collect();
        let moderate_violations: Vec<_> = results.violations.iter().filter(|v| v.impact == "moderate").collect();
        let minor_violations: Vec<_> = results.violations.iter().filter(|v| v.impact == "minor").collect();

        if !results.violations.is_empty() {
            report.push_str("## Violations by Severity\n\n");
            report.push_str(&format!("- **Critical:** {} (Must fix immediately)\n", critical_violations.len()));
            report.push_str(&format!("- **Serious:** {} (Should fix as priority)\n", serious_violations.len()));
            report.push_str(&format!("- **Moderate:** {} (Fix when possible)\n", moderate_violations.len()));
            report.push_str(&format!("- **Minor:** {} (Consider fixing)\n\n", minor_violations.len()));

            // Detailed violations
            report.push_str("## Detailed Violations\n\n");
            
            for (severity, violations) in [
                ("Critical", critical_violations),
                ("Serious", serious_violations),
                ("Moderate", moderate_violations),
                ("Minor", minor_violations),
            ] {
                if !violations.is_empty() {
                    report.push_str(&format!("### {} Issues\n\n", severity));
                    
                    for violation in violations {
                        report.push_str(&format!("#### {}\n\n", violation.id));
                        report.push_str(&format!("**Impact:** {}\n\n", violation.impact));
                        report.push_str(&format!("**Description:** {}\n\n", violation.description));
                        report.push_str(&format!("**Help:** {}\n\n", violation.help));
                        report.push_str(&format!("**Help URL:** {}\n\n", violation.help_url));
                        report.push_str(&format!("**WCAG Tags:** {}\n\n", violation.tags.join(", ")));
                        
                        if !violation.nodes.is_empty() {
                            report.push_str("**Affected Elements:**\n\n");
                            for (i, node) in violation.nodes.iter().enumerate() {
                                report.push_str(&format!("{}. **Target:** `{}`\n", i + 1, node.target.join(" > ")));
                                report.push_str(&format!("   **HTML:** `{}`\n", node.html));
                                if let Some(failure) = &node.failure_summary {
                                    report.push_str(&format!("   **Issue:** {}\n", failure));
                                }
                                report.push_str("\n");
                            }
                        }
                        report.push_str("---\n\n");
                    }
                }
            }
        } else {
            report.push_str("## ✅ No Violations Found!\n\nAll accessibility tests passed successfully.\n\n");
        }

        // Incomplete tests (may need manual review)
        if !results.incomplete.is_empty() {
            report.push_str("## Incomplete Tests (Manual Review Needed)\n\n");
            for incomplete in &results.incomplete {
                report.push_str(&format!("- **{}:** {}\n", incomplete.id, incomplete.description));
            }
            report.push_str("\n");
        }

        report.push_str("---\n\n");
        report.push_str("*Report generated by MediaOrganizer Accessibility Testing Framework*\n");

        report
    }
}

impl Drop for AccessibilityTester {
    fn drop(&mut self) {
        // Clean up browser instance
        let _ = self.browser.close();
    }
}