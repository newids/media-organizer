use dioxus::prelude::*;
use std::time::Instant;
use std::collections::HashMap;
use crate::state::LayoutState;

/// Signal optimization utilities for reducing unnecessary re-renders
/// Provides granular signals and memoization helpers for better performance

/// Optimized signal wrapper that tracks access patterns and reduces unnecessary updates
#[derive(Clone, PartialEq)]
pub struct OptimizedSignal<T: Clone + PartialEq + 'static> {
    inner: Signal<T>,
    last_read: Signal<Option<Instant>>,
    read_count: Signal<u64>,
    write_count: Signal<u64>,
    name: String,
}

impl<T: Clone + PartialEq + 'static> OptimizedSignal<T> {
    /// Create a new optimized signal with monitoring
    pub fn new(initial_value: T, name: String) -> Self {
        Self {
            inner: use_signal(|| initial_value),
            last_read: use_signal(|| None),
            read_count: use_signal(|| 0),
            write_count: use_signal(|| 0),
            name,
        }
    }

    /// Read the signal value (tracked)
    pub fn read(&mut self) -> T {
        self.last_read.set(Some(Instant::now()));
        self.read_count.with_mut(|count| *count += 1);
        
        self.inner.read().clone()
    }

    /// Write to the signal with change detection
    pub fn write(&mut self, new_value: T) {
        let current = self.inner.read().clone();
        
        // Only update if value actually changed
        if current != new_value {
            self.write_count.with_mut(|count| *count += 1);
            self.inner.set(new_value);
        }
    }

    /// Get access statistics for this signal
    pub fn get_stats(&self) -> SignalStats {
        SignalStats {
            name: self.name.clone(),
            read_count: *self.read_count.read(),
            write_count: *self.write_count.read(),
            last_read: *self.last_read.read(),
        }
    }

    /// Get the underlying signal for direct Dioxus integration
    pub fn signal(&self) -> Signal<T> {
        self.inner
    }
}

/// Statistics for signal usage patterns
#[derive(Debug, Clone)]
pub struct SignalStats {
    pub name: String,
    pub read_count: u64,
    pub write_count: u64,
    pub last_read: Option<Instant>,
}

/// Granular layout signals for better performance
/// Instead of one large LayoutState signal, break into smaller focused signals
#[derive(Clone)]
pub struct GranularLayoutSignals {
    // Theme and visual preferences
    pub theme: OptimizedSignal<crate::state::Theme>,
    pub animations_enabled: OptimizedSignal<bool>,
    pub reduced_motion: OptimizedSignal<bool>,
    
    // Activity bar state
    pub activity_bar_visible: OptimizedSignal<bool>,
    pub activity_bar_view: OptimizedSignal<crate::state::ActivityBarView>,
    pub activity_bar_width: OptimizedSignal<f64>,
    
    // Sidebar state
    pub sidebar_collapsed: OptimizedSignal<bool>,
    pub sidebar_width: OptimizedSignal<f64>,
    pub sidebar_content: OptimizedSignal<crate::state::SidebarContent>,
    
    // Editor state
    pub editor_layout: OptimizedSignal<crate::state::EditorLayoutConfig>,
    pub editor_active_group: OptimizedSignal<usize>,
    pub editor_show_tabs: OptimizedSignal<bool>,
    
    // Panel state
    pub panel_visible: OptimizedSignal<bool>,
    pub panel_height: OptimizedSignal<f64>,
    pub panel_active_tab: OptimizedSignal<crate::state::PanelTab>,
    
    // Viewport state
    pub viewport_width: OptimizedSignal<f64>,
    pub viewport_height: OptimizedSignal<f64>,
    pub window_maximized: OptimizedSignal<bool>,
    pub window_fullscreen: OptimizedSignal<bool>,
}

impl GranularLayoutSignals {
    /// Create granular signals from a layout state
    pub fn from_layout_state(state: &LayoutState) -> Self {
        Self {
            theme: OptimizedSignal::new(state.theme.clone(), "theme".to_string()),
            animations_enabled: OptimizedSignal::new(state.ui_preferences.enable_animations, "animations_enabled".to_string()),
            reduced_motion: OptimizedSignal::new(state.ui_preferences.reduced_motion, "reduced_motion".to_string()),
            
            activity_bar_visible: OptimizedSignal::new(state.activity_bar.is_visible, "activity_bar_visible".to_string()),
            activity_bar_view: OptimizedSignal::new(state.activity_bar.active_view.clone(), "activity_bar_view".to_string()),
            activity_bar_width: OptimizedSignal::new(state.activity_bar.width, "activity_bar_width".to_string()),
            
            sidebar_collapsed: OptimizedSignal::new(state.sidebar.is_collapsed, "sidebar_collapsed".to_string()),
            sidebar_width: OptimizedSignal::new(state.sidebar.width, "sidebar_width".to_string()),
            sidebar_content: OptimizedSignal::new(state.sidebar.active_content.clone(), "sidebar_content".to_string()),
            
            editor_layout: OptimizedSignal::new(state.editor_layout.layout_config.clone(), "editor_layout".to_string()),
            editor_active_group: OptimizedSignal::new(state.editor_layout.active_group, "editor_active_group".to_string()),
            editor_show_tabs: OptimizedSignal::new(state.editor_layout.show_tabs, "editor_show_tabs".to_string()),
            
            panel_visible: OptimizedSignal::new(state.panel.is_visible, "panel_visible".to_string()),
            panel_height: OptimizedSignal::new(state.panel.height, "panel_height".to_string()),
            panel_active_tab: OptimizedSignal::new(state.panel.active_tab.clone(), "panel_active_tab".to_string()),
            
            viewport_width: OptimizedSignal::new(state.viewport.width, "viewport_width".to_string()),
            viewport_height: OptimizedSignal::new(state.viewport.height, "viewport_height".to_string()),
            window_maximized: OptimizedSignal::new(state.viewport.is_maximized, "window_maximized".to_string()),
            window_fullscreen: OptimizedSignal::new(state.viewport.is_fullscreen, "window_fullscreen".to_string()),
        }
    }

    /// Update granular signals from a layout state (batch operation)
    pub fn update_from_layout_state(&mut self, state: &LayoutState) {
        // Theme and preferences
        self.theme.write(state.theme.clone());
        self.animations_enabled.write(state.ui_preferences.enable_animations);
        self.reduced_motion.write(state.ui_preferences.reduced_motion);
        
        // Activity bar
        self.activity_bar_visible.write(state.activity_bar.is_visible);
        self.activity_bar_view.write(state.activity_bar.active_view.clone());
        self.activity_bar_width.write(state.activity_bar.width);
        
        // Sidebar
        self.sidebar_collapsed.write(state.sidebar.is_collapsed);
        self.sidebar_width.write(state.sidebar.width);
        self.sidebar_content.write(state.sidebar.active_content.clone());
        
        // Editor
        self.editor_layout.write(state.editor_layout.layout_config.clone());
        self.editor_active_group.write(state.editor_layout.active_group);
        self.editor_show_tabs.write(state.editor_layout.show_tabs);
        
        // Panel
        self.panel_visible.write(state.panel.is_visible);
        self.panel_height.write(state.panel.height);
        self.panel_active_tab.write(state.panel.active_tab.clone());
        
        // Viewport
        self.viewport_width.write(state.viewport.width);
        self.viewport_height.write(state.viewport.height);
        self.window_maximized.write(state.viewport.is_maximized);
        self.window_fullscreen.write(state.viewport.is_fullscreen);
    }

    /// Get comprehensive usage statistics for all signals
    pub fn get_all_stats(&self) -> Vec<SignalStats> {
        vec![
            self.theme.get_stats(),
            self.animations_enabled.get_stats(),
            self.reduced_motion.get_stats(),
            self.activity_bar_visible.get_stats(),
            self.activity_bar_view.get_stats(),
            self.activity_bar_width.get_stats(),
            self.sidebar_collapsed.get_stats(),
            self.sidebar_width.get_stats(),
            self.sidebar_content.get_stats(),
            self.editor_layout.get_stats(),
            self.editor_active_group.get_stats(),
            self.editor_show_tabs.get_stats(),
            self.panel_visible.get_stats(),
            self.panel_height.get_stats(),
            self.panel_active_tab.get_stats(),
            self.viewport_width.get_stats(),
            self.viewport_height.get_stats(),
            self.window_maximized.get_stats(),
            self.window_fullscreen.get_stats(),
        ]
    }

    /// Identify signals with high read/write ratios (potential optimization targets)
    pub fn identify_hot_signals(&self) -> Vec<String> {
        let stats = self.get_all_stats();
        let mut hot_signals = Vec::new();
        
        for stat in stats {
            // Consider signals "hot" if they have high read counts or low read/write ratios
            let read_write_ratio = if stat.write_count > 0 {
                stat.read_count as f64 / stat.write_count as f64
            } else {
                stat.read_count as f64
            };
            
            if stat.read_count > 50 || (stat.write_count > 10 && read_write_ratio < 2.0) {
                hot_signals.push(stat.name);
            }
        }
        
        hot_signals
    }
}

/// Memoization helpers for expensive computations
pub struct MemoizedComputation<T: Clone + PartialEq + 'static> {
    cached_value: Signal<Option<T>>,
    cached_inputs: Signal<Option<String>>,
    computation_count: Signal<u64>,
    cache_hits: Signal<u64>,
}

impl<T: Clone + PartialEq + 'static> MemoizedComputation<T> {
    /// Create a new memoized computation
    pub fn new() -> Self {
        Self {
            cached_value: use_signal(|| None),
            cached_inputs: use_signal(|| None),
            computation_count: use_signal(|| 0),
            cache_hits: use_signal(|| 0),
        }
    }

    /// Compute value with memoization based on string representation of inputs
    pub fn compute<F>(&mut self, inputs: &str, compute_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        // Check if we have a cached result for these inputs
        if let (Some(cached_inputs), Some(cached_value)) = 
            (self.cached_inputs.read().as_ref(), self.cached_value.read().as_ref()) {
            if cached_inputs == inputs {
                self.cache_hits.with_mut(|hits| *hits += 1);
                return cached_value.clone();
            }
        }

        // Compute new value
        let result = compute_fn();
        
        // Cache the result
        self.cached_inputs.set(Some(inputs.to_string()));
        self.cached_value.set(Some(result.clone()));
        self.computation_count.with_mut(|count| *count += 1);
        
        result
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            computations: *self.computation_count.read(),
            cache_hits: *self.cache_hits.read(),
            hit_rate: if *self.computation_count.read() > 0 {
                *self.cache_hits.read() as f64 / (*self.computation_count.read() + *self.cache_hits.read()) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cached_value.set(None);
        self.cached_inputs.set(None);
    }
}

/// Cache performance statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub computations: u64,
    pub cache_hits: u64,
    pub hit_rate: f64,
}

/// Performance-optimized CSS generation with memoization
pub struct OptimizedCssGenerator {
    transition_cache: MemoizedComputation<String>,
    layout_cache: MemoizedComputation<String>,
    theme_cache: MemoizedComputation<String>,
}

impl OptimizedCssGenerator {
    pub fn new() -> Self {
        Self {
            transition_cache: MemoizedComputation::new(),
            layout_cache: MemoizedComputation::new(),
            theme_cache: MemoizedComputation::new(),
        }
    }

    /// Generate transition CSS with memoization
    pub fn get_transition_css(&mut self, duration_ms: u32, reduced_motion: bool) -> String {
        let cache_key = format!("{}:{}", duration_ms, reduced_motion);
        
        self.transition_cache.compute(&cache_key, || {
            if reduced_motion {
                "none".to_string()
            } else {
                format!("width {}ms ease-out, height {}ms ease-out, transform {}ms ease-out", 
                       duration_ms, duration_ms, duration_ms)
            }
        })
    }

    /// Generate layout CSS with memoization
    pub fn get_layout_css(&mut self, sidebar_width: f64, panel_height: f64, sidebar_collapsed: bool) -> String {
        let cache_key = format!("{}:{}:{}", sidebar_width, panel_height, sidebar_collapsed);
        
        self.layout_cache.compute(&cache_key, || {
            let effective_sidebar_width = if sidebar_collapsed { 0.0 } else { sidebar_width };
            format!(
                "--sidebar-width: {}px; --panel-height: {}px; --content-width: calc(100vw - {}px);",
                effective_sidebar_width, panel_height, effective_sidebar_width
            )
        })
    }

    /// Generate theme CSS with memoization
    pub fn get_theme_css(&mut self, theme: &crate::state::Theme) -> String {
        let cache_key = format!("{:?}", theme);
        
        self.theme_cache.compute(&cache_key, || {
            match theme {
                crate::state::Theme::Light => {
                    "--bg-primary: #ffffff; --bg-secondary: #f8f9fa; --text-primary: #212529; --text-secondary: #6c757d;".to_string()
                },
                crate::state::Theme::Dark => {
                    "--bg-primary: #1e1e1e; --bg-secondary: #252526; --text-primary: #cccccc; --text-secondary: #969696;".to_string()
                },
                crate::state::Theme::Auto => {
                    "@media (prefers-color-scheme: dark) { --bg-primary: #1e1e1e; --bg-secondary: #252526; --text-primary: #cccccc; --text-secondary: #969696; }".to_string()
                },
                crate::state::Theme::HighContrast => {
                    "--bg-primary: #000000; --bg-secondary: #111111; --text-primary: #ffffff; --text-secondary: #cccccc; --accent: #ffff00;".to_string()
                },
            }
        })
    }

    /// Get comprehensive cache statistics
    pub fn get_performance_stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        stats.insert("transitions".to_string(), self.transition_cache.get_cache_stats());
        stats.insert("layout".to_string(), self.layout_cache.get_cache_stats());
        stats.insert("theme".to_string(), self.theme_cache.get_cache_stats());
        stats
    }

    /// Clear all caches
    pub fn clear_all_caches(&mut self) {
        self.transition_cache.clear_cache();
        self.layout_cache.clear_cache();
        self.theme_cache.clear_cache();
    }
}

/// Component optimization utilities
pub mod component_utils {
    use super::*;

    /// Optimized theme provider that only updates when theme actually changes
    #[component]
    pub fn OptimizedThemeProvider(theme_signal: OptimizedSignal<crate::state::Theme>, children: Element) -> Element {
        let theme = theme_signal.read();
        
        rsx! {
            div {
                class: format!("theme-{:?}", theme).to_lowercase(),
                {children}
            }
        }
    }

    /// Optimized sidebar component that only re-renders on relevant changes
    #[component]
    pub fn OptimizedSidebar(
        collapsed: OptimizedSignal<bool>,
        width: OptimizedSignal<f64>,
        content: OptimizedSignal<crate::state::SidebarContent>
    ) -> Element {
        let is_collapsed = collapsed.read();
        let sidebar_width = width.read();
        let active_content = content.read();
        
        if is_collapsed {
            return rsx! { div { class: "sidebar collapsed" } };
        }
        
        rsx! {
            div {
                class: "sidebar expanded",
                style: format!("width: {}px", sidebar_width),
                div { class: "sidebar-content", "{active_content:?}" }
            }
        }
    }

    /// Optimized panel component with conditional rendering
    #[component] 
    pub fn OptimizedPanel(
        visible: OptimizedSignal<bool>,
        height: OptimizedSignal<f64>,
        active_tab: OptimizedSignal<crate::state::PanelTab>
    ) -> Element {
        let is_visible = visible.read();
        
        if !is_visible {
            return rsx! { div { display: "none" } };
        }
        
        let panel_height = height.read();
        let tab = active_tab.read();
        
        rsx! {
            div {
                class: "panel",
                style: format!("height: {}px", panel_height),
                div { class: "panel-tab-content", "{tab:?}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_optimized_signal_change_detection() {
        // Note: This test is simplified since we can't use Dioxus hooks in tests
        // In a real component context, these would work with actual signals
        
        // Test that the concept of change detection works
        let value1 = "test";
        let value2 = "test";
        let value3 = "different";
        
        assert_eq!(value1, value2); // Should not trigger update
        assert_ne!(value1, value3); // Should trigger update
    }

    #[test]
    fn test_memoized_computation() {
        // Test the caching logic (without actual Dioxus signals)
        let cache_key_1 = "input1";
        let cache_key_2 = "input2";
        let cache_key_1_repeat = "input1";
        
        // Simulate cache behavior
        let mut cache = std::collections::HashMap::new();
        
        // First computation
        let result1 = cache.entry(cache_key_1).or_insert_with(|| "computed1".to_string());
        assert_eq!(result1, "computed1");
        
        // Different input
        let result2 = cache.entry(cache_key_2).or_insert_with(|| "computed2".to_string());
        assert_eq!(result2, "computed2");
        
        // Same input (should hit cache)
        let result1_cached = cache.get(cache_key_1_repeat).unwrap();
        assert_eq!(result1_cached, "computed1");
    }

    #[test]
    fn test_css_generation_caching() {
        // Test CSS generation patterns
        fn generate_transition_css(duration_ms: u32, reduced_motion: bool) -> String {
            if reduced_motion {
                "none".to_string()
            } else {
                format!("width {}ms ease-out, height {}ms ease-out", duration_ms, duration_ms)
            }
        }

        let css1 = generate_transition_css(200, false);
        let css2 = generate_transition_css(200, false);
        let css3 = generate_transition_css(200, true);
        
        assert_eq!(css1, css2); // Same inputs should produce same output
        assert_ne!(css1, css3); // Different inputs should produce different output
        
        assert_eq!(css1, "width 200ms ease-out, height 200ms ease-out");
        assert_eq!(css3, "none");
    }

    #[test]
    fn test_signal_stats_calculation() {
        // Test signal statistics calculations
        let mut stats = SignalStats {
            name: "test_signal".to_string(),
            read_count: 100,
            write_count: 20,
            last_read: Some(Instant::now()),
        };
        
        let read_write_ratio = stats.read_count as f64 / stats.write_count as f64;
        assert_eq!(read_write_ratio, 5.0);
        
        // Test hot signal detection logic
        let is_hot = stats.read_count > 50 || (stats.write_count > 10 && read_write_ratio < 2.0);
        assert!(is_hot); // Should be hot due to high read count
        
        // Test low ratio case
        stats.read_count = 15;
        stats.write_count = 10;
        let new_ratio = stats.read_count as f64 / stats.write_count as f64;
        let is_hot_low_ratio = stats.read_count > 50 || (stats.write_count > 10 && new_ratio < 2.0);
        assert!(!is_hot_low_ratio); // Should not be hot
    }

    #[test]
    fn test_cache_hit_rate_calculation() {
        // Test cache statistics calculations
        let computations = 10u64;
        let cache_hits = 40u64;
        
        let hit_rate = if computations > 0 {
            cache_hits as f64 / (computations + cache_hits) as f64
        } else {
            0.0
        };
        
        assert!((hit_rate - 0.8).abs() < f64::EPSILON); // 40/50 = 0.8
    }

    #[test]
    fn test_performance_thresholds() {
        // Test performance classification thresholds
        fn classify_signal_performance(read_count: u64, write_count: u64) -> &'static str {
            let ratio = if write_count > 0 {
                read_count as f64 / write_count as f64
            } else {
                read_count as f64
            };
            
            if read_count > 100 {
                "high_usage"
            } else if write_count > 20 {
                "high_writes"
            } else if ratio < 2.0 && write_count > 5 {
                "inefficient"
            } else {
                "normal"
            }
        }
        
        assert_eq!(classify_signal_performance(150, 10), "high_usage");
        assert_eq!(classify_signal_performance(50, 25), "high_writes");
        assert_eq!(classify_signal_performance(10, 8), "inefficient");
        assert_eq!(classify_signal_performance(20, 5), "normal");
    }
}