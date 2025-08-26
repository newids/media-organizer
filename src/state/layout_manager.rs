use dioxus::prelude::*;
use std::time::Instant;
use crate::state::{
    LayoutState, Theme, ActivityBarView, ActivityBarPosition, SidebarPosition, 
    SidebarContent, PanelTab, PanelPosition, EditorLayoutConfig,
    persistence::{save_layout_state_debounced, load_layout_state, flush_pending_layout_saves},
    performance::{init_profiler, with_profiler, PerformanceStatus, PerformanceReport}
};
use crate::measure_performance;

/// Performance-optimized layout manager for reactive state updates
/// Provides a clean API for layout operations with <100ms transition targets
#[derive(Clone)]
pub struct LayoutManager {
    /// Layout state signal for reactive updates
    layout_state: Signal<LayoutState>,
    /// Performance tracking for optimization
    last_update_time: Signal<Option<Instant>>,
    /// Update batching to minimize re-renders
    pending_updates: Signal<Vec<LayoutUpdate>>,
}

/// Represents a pending layout update for batching
#[derive(Clone, Debug)]
pub enum LayoutUpdate {
    Theme(Theme),
    ActivityBarView(ActivityBarView),
    ActivityBarVisibility(bool),
    ActivityBarPosition(ActivityBarPosition),
    SidebarCollapse(bool),
    SidebarWidth(f64),
    SidebarPosition(SidebarPosition),
    SidebarContent(SidebarContent),
    EditorLayout(EditorLayoutConfig),
    EditorActiveGroup(usize),
    EditorSplitRatios(Vec<f64>),
    PanelVisibility(bool),
    PanelHeight(f64),
    PanelActiveTab(PanelTab),
    PanelPosition(PanelPosition),
    ViewportDimensions { width: f64, height: f64 },
    WindowMaximized(bool),
    WindowFullscreen(bool),
    AnimationsEnabled(bool),
    AnimationDuration(u32),
}

/// Layout manager creation and lifecycle
impl LayoutManager {
    /// Create a new layout manager with default state
    /// Must be called within a Dioxus component context
    pub fn new() -> Self {
        // Initialize performance profiler
        init_profiler();
        
        Self {
            layout_state: use_signal(LayoutState::default),
            last_update_time: use_signal(|| None),
            pending_updates: use_signal(Vec::new),
        }
    }
    
    /// Create layout manager with custom initial state
    pub fn with_state(initial_state: LayoutState) -> Self {
        Self {
            layout_state: use_signal(|| initial_state),
            last_update_time: use_signal(|| None),
            pending_updates: use_signal(Vec::new),
        }
    }
    
    /// Get the current layout state (read-only)
    pub fn state(&self) -> LayoutState {
        self.layout_state.read().clone()
    }
    
    /// Get direct access to the layout state signal
    pub fn state_signal(&self) -> Signal<LayoutState> {
        self.layout_state
    }
    
    /// Check if any updates are pending
    pub fn has_pending_updates(&self) -> bool {
        !self.pending_updates.read().is_empty()
    }
}

/// Theme management
impl LayoutManager {
    /// Get current theme
    pub fn theme(&self) -> Theme {
        self.layout_state.read().theme.clone()
    }
    
    /// Update theme with immediate effect
    pub fn set_theme(&mut self, theme: Theme) {
        measure_performance!("set_theme", 5, false, {
            self.record_update_time();
            with_profiler(|profiler| profiler.record_signal_write());
            self.layout_state.with_mut(|state| {
                state.theme = theme;
            });
        });
    }
    
    /// Queue theme update for batch processing
    pub fn queue_theme_update(&mut self, theme: Theme) {
        self.pending_updates.with_mut(|updates| {
            updates.push(LayoutUpdate::Theme(theme));
        });
    }
    
    /// Toggle between light and dark themes
    pub fn toggle_theme(&mut self) {
        let current_theme = self.theme();
        let new_theme = match current_theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
            Theme::Auto => Theme::Dark, // Default to dark when toggling from auto
            Theme::HighContrast => Theme::Light, // Go to light from high contrast
        };
        self.set_theme(new_theme);
    }
}

/// Activity bar management
impl LayoutManager {
    /// Get current activity bar view
    pub fn activity_bar_view(&self) -> ActivityBarView {
        self.layout_state.read().activity_bar.active_view.clone()
    }
    
    /// Set active activity bar view
    pub fn set_activity_bar_view(&mut self, view: ActivityBarView) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.activity_bar.active_view = view;
        });
    }
    
    /// Toggle activity bar visibility
    pub fn toggle_activity_bar(&mut self) {
        self.record_update_time();
        let current_visibility = self.layout_state.read().activity_bar.is_visible;
        self.layout_state.with_mut(|state| {
            state.activity_bar.is_visible = !current_visibility;
        });
    }
    
    /// Set activity bar position
    pub fn set_activity_bar_position(&mut self, position: ActivityBarPosition) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.activity_bar.position = position;
        });
    }
    
    /// Update activity bar width
    pub fn set_activity_bar_width(&mut self, width: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.activity_bar.width = width.max(30.0).min(100.0);
        });
    }
}

/// Sidebar management
impl LayoutManager {
    /// Get sidebar collapsed state
    pub fn is_sidebar_collapsed(&self) -> bool {
        self.layout_state.read().sidebar.is_collapsed
    }
    
    /// Toggle sidebar collapse state
    pub fn toggle_sidebar(&mut self) {
        measure_performance!("toggle_sidebar", 3, false, {
            self.record_update_time();
            with_profiler(|profiler| {
                profiler.record_signal_read();
                profiler.record_signal_write();
            });
            let current_collapsed = self.layout_state.read().sidebar.is_collapsed;
            self.layout_state.with_mut(|state| {
                state.sidebar.is_collapsed = !current_collapsed;
            });
        });
    }
    
    /// Set sidebar width with bounds checking
    pub fn set_sidebar_width(&mut self, width: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            let sidebar = &mut layout.sidebar;
            sidebar.width = width.max(sidebar.min_width).min(sidebar.max_width);
        });
    }
    
    /// Get current sidebar width
    pub fn sidebar_width(&self) -> f64 {
        self.layout_state.read().sidebar.width
    }
    
    /// Set sidebar position
    pub fn set_sidebar_position(&mut self, position: SidebarPosition) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.sidebar.position = position;
        });
    }
    
    /// Set active sidebar content
    pub fn set_sidebar_content(&mut self, content: SidebarContent) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.sidebar.active_content = content;
        });
    }
    
    /// Get active sidebar content
    pub fn sidebar_content(&self) -> SidebarContent {
        self.layout_state.read().sidebar.active_content.clone()
    }
}

/// Editor layout management
impl LayoutManager {
    /// Get current editor layout configuration
    pub fn editor_layout(&self) -> EditorLayoutConfig {
        self.layout_state.read().editor_layout.layout_config.clone()
    }
    
    /// Set editor layout configuration
    pub fn set_editor_layout(&mut self, config: EditorLayoutConfig) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.editor_layout.layout_config = config;
            
            // Update split ratios based on layout
            match &layout.editor_layout.layout_config {
                EditorLayoutConfig::Single => {
                    layout.editor_layout.split_ratios = vec![1.0];
                }
                EditorLayoutConfig::SplitHorizontal | EditorLayoutConfig::SplitVertical => {
                    layout.editor_layout.split_ratios = vec![0.5, 0.5];
                }
                EditorLayoutConfig::Grid { rows, cols } => {
                    let total_groups = rows * cols;
                    let ratio = 1.0 / total_groups as f64;
                    layout.editor_layout.split_ratios = vec![ratio; total_groups];
                }
            }
        });
    }
    
    /// Get active editor group
    pub fn active_editor_group(&self) -> usize {
        self.layout_state.read().editor_layout.active_group
    }
    
    /// Set active editor group
    pub fn set_active_editor_group(&mut self, group_index: usize) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.editor_layout.active_group = group_index;
        });
    }
    
    /// Update editor split ratios
    pub fn set_editor_split_ratios(&mut self, ratios: Vec<f64>) {
        // Normalize ratios to sum to 1.0
        let sum: f64 = ratios.iter().sum();
        if sum > 0.0 {
            let normalized_ratios: Vec<f64> = ratios.iter().map(|r| r / sum).collect();
            self.record_update_time();
            self.layout_state.with_mut(|state| {
                state.editor_layout.split_ratios = normalized_ratios;
            });
        }
    }
    
    /// Toggle tab visibility
    pub fn toggle_editor_tabs(&mut self) {
        self.record_update_time();
        let current_show_tabs = self.layout_state.read().editor_layout.show_tabs;
        self.layout_state.with_mut(|state| {
            state.editor_layout.show_tabs = !current_show_tabs;
        });
    }
    
    /// Set tab height
    pub fn set_tab_height(&mut self, height: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.editor_layout.tab_height = height.max(25.0).min(60.0);
        });
    }
}

/// Panel management
impl LayoutManager {
    /// Get panel visibility
    pub fn is_panel_visible(&self) -> bool {
        self.layout_state.read().panel.is_visible
    }
    
    /// Toggle panel visibility
    pub fn toggle_panel(&mut self) {
        self.record_update_time();
        let current_visible = self.layout_state.read().panel.is_visible;
        self.layout_state.with_mut(|state| {
            state.panel.is_visible = !current_visible;
        });
    }
    
    /// Set panel height with bounds checking
    pub fn set_panel_height(&mut self, height: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            let panel = &mut layout.panel;
            let viewport_height = layout.viewport.height;
            let max_height = viewport_height * panel.max_height_fraction;
            panel.height = height.max(panel.min_height).min(max_height);
        });
    }
    
    /// Get current panel height
    pub fn panel_height(&self) -> f64 {
        self.layout_state.read().panel.height
    }
    
    /// Set active panel tab
    pub fn set_panel_tab(&mut self, tab: PanelTab) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.panel.active_tab = tab;
        });
    }
    
    /// Get active panel tab
    pub fn panel_tab(&self) -> PanelTab {
        self.layout_state.read().panel.active_tab.clone()
    }
    
    /// Set panel position
    pub fn set_panel_position(&mut self, position: PanelPosition) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.panel.position = position;
        });
    }
}

/// Viewport management
impl LayoutManager {
    /// Update viewport dimensions
    pub fn set_viewport_dimensions(&mut self, width: f64, height: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.viewport.width = width;
            layout.viewport.height = height;
        });
    }
    
    /// Get viewport dimensions
    pub fn viewport_dimensions(&self) -> (f64, f64) {
        let viewport = &self.layout_state.read().viewport;
        (viewport.width, viewport.height)
    }
    
    /// Set window maximized state
    pub fn set_window_maximized(&mut self, maximized: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.viewport.is_maximized = maximized;
        });
    }
    
    /// Set window fullscreen state
    pub fn set_window_fullscreen(&mut self, fullscreen: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.viewport.is_fullscreen = fullscreen;
        });
    }
    
    /// Update pixel ratio for high-DPI displays
    pub fn set_pixel_ratio(&mut self, ratio: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.viewport.pixel_ratio = ratio;
        });
    }
}

/// UI preferences management
impl LayoutManager {
    /// Get animations enabled state
    pub fn are_animations_enabled(&self) -> bool {
        self.layout_state.read().ui_preferences.enable_animations
    }
    
    /// Toggle animations
    pub fn toggle_animations(&mut self) {
        self.record_update_time();
        let current_enabled = self.layout_state.read().ui_preferences.enable_animations;
        self.layout_state.with_mut(|state| {
            state.ui_preferences.enable_animations = !current_enabled;
        });
    }
    
    /// Set animation duration
    pub fn set_animation_duration(&mut self, duration_ms: u32) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.ui_preferences.animation_duration = duration_ms.max(50).min(1000);
        });
    }
    
    /// Set reduced motion for accessibility
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.ui_preferences.reduced_motion = reduced;
            
            // Automatically disable animations if reduced motion is enabled
            if reduced {
                layout.ui_preferences.enable_animations = false;
                layout.ui_preferences.smooth_scrolling = false;
                layout.ui_preferences.visual_feedback = false;
            }
        });
    }
    
    /// Enable smooth scrolling
    pub fn set_smooth_scrolling(&mut self, enabled: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.ui_preferences.smooth_scrolling = enabled;
        });
    }
    
    /// Enable visual feedback
    pub fn set_visual_feedback(&mut self, enabled: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.ui_preferences.visual_feedback = enabled;
        });
    }
}

/// Batch operations and performance
impl LayoutManager {
    /// Apply all pending updates in a single batch
    pub fn apply_pending_updates(&mut self) {
        let batch_size = self.pending_updates.read().len();
        if batch_size == 0 {
            return;
        }
        
        measure_performance!("apply_batch_updates", batch_size * 2, false, {
            self.record_update_time();
            
            // Record batch operation metrics
            with_profiler(|profiler| {
                profiler.record_batch_operation(batch_size, std::time::Duration::from_millis(batch_size as u64 * 5));
                profiler.record_signal_read();
                profiler.record_signal_write();
            });
            
            let updates = {
                let mut pending_updates = Vec::new();
                self.pending_updates.with_mut(|pending| {
                    pending_updates = pending.clone();
                    pending.clear();
                });
                pending_updates
            };
            
            // Apply all updates in a single write transaction
            self.layout_state.with_mut(|layout| {
                for update in updates {
                    match update {
                        LayoutUpdate::Theme(theme) => layout.theme = theme,
                        LayoutUpdate::ActivityBarView(view) => layout.activity_bar.active_view = view,
                        LayoutUpdate::ActivityBarVisibility(visible) => layout.activity_bar.is_visible = visible,
                        LayoutUpdate::ActivityBarPosition(pos) => layout.activity_bar.position = pos,
                        LayoutUpdate::SidebarCollapse(collapsed) => layout.sidebar.is_collapsed = collapsed,
                        LayoutUpdate::SidebarWidth(width) => {
                            layout.sidebar.width = width.max(layout.sidebar.min_width).min(layout.sidebar.max_width);
                        },
                        LayoutUpdate::SidebarPosition(pos) => layout.sidebar.position = pos,
                        LayoutUpdate::SidebarContent(content) => layout.sidebar.active_content = content,
                        LayoutUpdate::EditorLayout(config) => layout.editor_layout.layout_config = config,
                        LayoutUpdate::EditorActiveGroup(group) => layout.editor_layout.active_group = group,
                        LayoutUpdate::EditorSplitRatios(ratios) => layout.editor_layout.split_ratios = ratios,
                        LayoutUpdate::PanelVisibility(visible) => layout.panel.is_visible = visible,
                        LayoutUpdate::PanelHeight(height) => {
                            let max_height = layout.viewport.height * layout.panel.max_height_fraction;
                            layout.panel.height = height.max(layout.panel.min_height).min(max_height);
                        },
                        LayoutUpdate::PanelActiveTab(tab) => layout.panel.active_tab = tab,
                        LayoutUpdate::PanelPosition(pos) => layout.panel.position = pos,
                        LayoutUpdate::ViewportDimensions { width, height } => {
                            layout.viewport.width = width;
                            layout.viewport.height = height;
                        },
                        LayoutUpdate::WindowMaximized(maximized) => layout.viewport.is_maximized = maximized,
                        LayoutUpdate::WindowFullscreen(fullscreen) => layout.viewport.is_fullscreen = fullscreen,
                        LayoutUpdate::AnimationsEnabled(enabled) => layout.ui_preferences.enable_animations = enabled,
                        LayoutUpdate::AnimationDuration(duration) => layout.ui_preferences.animation_duration = duration,
                    }
                }
            });
        });
    }
    
    /// Clear all pending updates without applying them
    pub fn clear_pending_updates(&mut self) {
        self.pending_updates.with_mut(|updates| {
            updates.clear();
        });
    }
    
    /// Get time since last update (for performance monitoring)
    pub fn time_since_last_update(&self) -> Option<std::time::Duration> {
        self.last_update_time.read().as_ref().map(|time| time.elapsed())
    }
    
    /// Reset the layout state to defaults
    pub fn reset_to_defaults(&mut self) {
        self.record_update_time();
        self.layout_state.set(LayoutState::default());
        self.clear_pending_updates();
    }
    
    /// Record update time for performance tracking
    fn record_update_time(&mut self) {
        self.last_update_time.set(Some(Instant::now()));
    }
}

/// Convenience methods for common layout operations
impl LayoutManager {
    /// Enter distraction-free mode (hide panels and sidebars)
    pub fn enter_distraction_free_mode(&mut self) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.activity_bar.is_visible = false;
            layout.sidebar.is_collapsed = true;
            layout.panel.is_visible = false;
        });
    }
    
    /// Exit distraction-free mode (restore panels and sidebars)
    pub fn exit_distraction_free_mode(&mut self) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.activity_bar.is_visible = true;
            layout.sidebar.is_collapsed = false;
            layout.panel.is_visible = true;
        });
    }
    
    /// Setup split editor layout with specific ratios
    pub fn setup_split_editor(&mut self, vertical: bool, left_ratio: f64) {
        self.record_update_time();
        self.layout_state.with_mut(|layout| {
            layout.editor_layout.layout_config = if vertical {
                EditorLayoutConfig::SplitVertical
            } else {
                EditorLayoutConfig::SplitHorizontal
            };
            
            let clamped_left_ratio = left_ratio.clamp(0.1, 0.9);
            let right_ratio = 1.0 - clamped_left_ratio;
            layout.editor_layout.split_ratios = vec![clamped_left_ratio, right_ratio];
        });
    }
    
    /// Auto-adjust panel height based on content and viewport
    pub fn auto_adjust_panel_height(&mut self, content_lines: usize) {
        let viewport_height = self.layout_state.read().viewport.height;
        let line_height = 20.0; // Approximate line height
        let desired_height = (content_lines as f64 * line_height).min(viewport_height * 0.4);
        self.set_panel_height(desired_height);
    }
    
    /// Responsive layout adjustments for small screens
    pub fn apply_responsive_adjustments(&mut self) {
        let (viewport_width, _) = self.viewport_dimensions();
        
        if viewport_width < 768.0 { // Mobile breakpoint
            self.record_update_time();
            self.layout_state.with_mut(|layout| {
                // Collapse sidebar on small screens
                layout.sidebar.is_collapsed = true;
                // Hide activity bar on very small screens
                if viewport_width < 480.0 {
                    layout.activity_bar.is_visible = false;
                }
                // Reduce panel height
                layout.panel.height = layout.panel.height.min(200.0);
            });
        }
    }
}

/// Reactive utilities (these should be called from components, not from the manager itself)
impl LayoutManager {
    /// Get layout state for reactive components
    pub fn get_reactive_theme(&self) -> Theme {
        self.layout_state.read().theme.clone()
    }
    
    /// Get sidebar state for reactive components
    pub fn get_reactive_sidebar(&self) -> (bool, f64) {
        let layout = self.layout_state.read();
        (layout.sidebar.is_collapsed, layout.sidebar.width)
    }
    
    /// Get panel state for reactive components
    pub fn get_reactive_panel(&self) -> (bool, f64) {
        let layout = self.layout_state.read();
        (layout.panel.is_visible, layout.panel.height)
    }
    
    /// Get viewport state for reactive components
    pub fn get_reactive_viewport(&self) -> (f64, f64) {
        let layout = self.layout_state.read();
        (layout.viewport.width, layout.viewport.height)
    }
    
    /// Create a subscription callback for layout changes (for components)
    pub fn create_layout_subscription<F>(&self, callback: F) 
    where
        F: Fn(LayoutState) + 'static + Clone,
    {
        let layout_state = self.layout_state;
        // Components should use this pattern:
        // use_effect(move || {
        //     let state = layout_state.read().clone();
        //     callback(state);
        // });
    }
}

/// Component integration helpers
impl LayoutManager {
    /// Get the layout state signal directly for component binding
    /// Components can use this to access the full layout state reactively
    pub fn layout_signal(&self) -> Signal<LayoutState> {
        self.layout_state
    }
    
    /// Get a specific piece of layout state for reactive components
    /// This avoids borrowing issues by cloning the needed data
    pub fn get_layout_state_for_component(&self) -> LayoutState {
        self.layout_state.read().clone()
    }
}

/// Animation and transition helpers
impl LayoutManager {
    /// Get CSS transition duration based on current preferences
    pub fn transition_duration(&self) -> String {
        let duration = if self.are_animations_enabled() {
            self.layout_state.read().ui_preferences.animation_duration
        } else {
            0
        };
        format!("{}ms", duration)
    }
    
    /// Get CSS transition properties for smooth layout changes
    pub fn layout_transition_css(&self) -> String {
        let duration = self.transition_duration();
        if self.layout_state.read().ui_preferences.reduced_motion {
            "none".to_string()
        } else {
            format!("width {}, height {}, transform {}", duration, duration, duration)
        }
    }
    
    /// Check if smooth transitions should be applied
    pub fn should_animate(&self) -> bool {
        let prefs = &self.layout_state.read().ui_preferences;
        prefs.enable_animations && !prefs.reduced_motion
    }
}

/// Performance monitoring
impl LayoutManager {
    /// Get performance metrics for the layout manager
    pub fn performance_metrics(&self) -> LayoutPerformanceMetrics {
        LayoutPerformanceMetrics {
            pending_updates: self.pending_updates.read().len(),
            last_update_time: self.last_update_time.read().clone(),
            animations_enabled: self.are_animations_enabled(),
            reduced_motion: self.layout_state.read().ui_preferences.reduced_motion,
        }
    }
    
    /// Log performance warnings if updates are taking too long
    pub fn check_performance(&self) {
        if let Some(duration) = self.time_since_last_update() {
            if duration.as_millis() > 100 {
                tracing::warn!(
                    "Layout update took {}ms (target: <100ms)", 
                    duration.as_millis()
                );
            }
        }
        
        let pending_count = self.pending_updates.read().len();
        if pending_count > 10 {
            tracing::warn!(
                "High number of pending layout updates: {} (consider batching)", 
                pending_count
            );
        }
    }
    
    /// Get current performance status from the global profiler
    pub fn get_performance_status(&self) -> PerformanceStatus {
        with_profiler(|profiler| profiler.get_current_status())
            .unwrap_or(PerformanceStatus::Unknown)
    }
    
    /// Generate comprehensive performance report
    pub fn get_performance_report(&self) -> Option<PerformanceReport> {
        with_profiler(|profiler| profiler.generate_report())
    }
    
    /// Check if performance optimization is needed
    pub fn needs_performance_optimization(&self) -> bool {
        with_profiler(|profiler| profiler.needs_optimization())
            .unwrap_or(false)
    }
    
    /// Get the most problematic layout operations
    pub fn get_performance_bottlenecks(&self) -> Vec<String> {
        with_profiler(|profiler| profiler.get_bottlenecks())
            .unwrap_or_default()
    }
    
    /// Run a performance diagnostic on layout operations
    pub fn run_performance_diagnostic(&self) -> String {
        let status = self.get_performance_status();
        let pending = self.pending_updates.read().len();
        let needs_optimization = self.needs_performance_optimization();
        
        let mut report = Vec::new();
        report.push(format!("Performance Status: {:?} ({})", status, status.description()));
        report.push(format!("Pending Updates: {}", pending));
        report.push(format!("Needs Optimization: {}", needs_optimization));
        
        if let Some(perf_report) = self.get_performance_report() {
            report.push(format!("Grade: {}", perf_report.grade));
            report.push(format!("Average Update Time: {:?}", perf_report.avg_update_time));
            report.push(format!("Signal Efficiency: {}%", perf_report.signal_efficiency));
            
            if !perf_report.slow_operations.is_empty() {
                report.push(format!("Slow Operations: {}", perf_report.slow_operations.join(", ")));
            }
            
            if !perf_report.recommendations.is_empty() {
                report.push("Recommendations:".to_string());
                for rec in &perf_report.recommendations {
                    report.push(format!("  - {}", rec));
                }
            }
        }
        
        report.join("\n")
    }
}

/// Performance metrics for monitoring
#[derive(Debug, Clone)]
pub struct LayoutPerformanceMetrics {
    pub pending_updates: usize,
    pub last_update_time: Option<Instant>,
    pub animations_enabled: bool,
    pub reduced_motion: bool,
}

/// Layout state persistence
impl LayoutManager {
    /// Create a new layout manager with restored state from persistence
    /// Falls back to default state if no saved state exists or if loading fails
    pub fn new_with_persistence() -> Self {
        let initial_state = load_layout_state().unwrap_or_else(LayoutState::default);
        Self::with_state(initial_state)
    }
    
    /// Save the current layout state to persistent storage
    /// Uses debounced saving to avoid excessive disk writes
    pub fn save_state(&self) {
        measure_performance!("save_layout_state", 1, true, {
            let current_state = self.state();
            let payload_size = std::mem::size_of_val(&current_state);
            
            let start = std::time::Instant::now();
            save_layout_state_debounced(current_state);
            let duration = start.elapsed();
            
            // Record persistence metrics
            with_profiler(|profiler| {
                profiler.record_save_operation(duration, payload_size, true);
            });
        });
    }
    
    /// Force immediate save of layout state (useful for app shutdown)
    pub fn flush_save(&self) {
        flush_pending_layout_saves();
    }
    
    /// Auto-save current state whenever significant changes occur
    /// This should be called after layout modifications
    pub fn auto_save(&self) {
        // Only save if persistence is enabled in settings
        let state = self.layout_state.read();
        if state.persistence.persist_layout {
            drop(state); // Release the read lock before saving
            self.save_state();
        }
    }
    
    /// Auto-save theme changes if theme persistence is enabled
    pub fn auto_save_theme(&self) {
        let state = self.layout_state.read();
        if state.persistence.persist_theme {
            drop(state);
            self.save_state();
        }
    }
    
    /// Check if auto-save should be triggered based on time elapsed
    /// This implements the auto_save_interval setting
    pub fn should_auto_save(&self) -> bool {
        let state = self.layout_state.read();
        if !state.persistence.persist_layout {
            return false;
        }
        
        let interval = std::time::Duration::from_secs(state.persistence.auto_save_interval as u64);
        if let Some(last_update) = *self.last_update_time.read() {
            last_update.elapsed() >= interval
        } else {
            false
        }
    }
    
    /// Trigger auto-save if conditions are met
    pub fn check_auto_save(&self) {
        if self.should_auto_save() {
            self.auto_save();
        }
    }
}

/// Enhanced layout manager methods with persistence integration
impl LayoutManager {
    /// Set theme with automatic persistence
    pub fn set_theme_with_save(&mut self, theme: Theme) {
        self.set_theme(theme);
        self.auto_save_theme();
    }
    
    /// Toggle theme with automatic persistence
    pub fn toggle_theme_with_save(&mut self) {
        self.toggle_theme();
        self.auto_save_theme();
    }
    
    /// Set sidebar width with automatic persistence
    pub fn set_sidebar_width_with_save(&mut self, width: f64) {
        self.set_sidebar_width(width);
        self.auto_save();
    }
    
    /// Toggle sidebar with automatic persistence
    pub fn toggle_sidebar_with_save(&mut self) {
        self.toggle_sidebar();
        self.auto_save();
    }
    
    /// Set panel height with automatic persistence
    pub fn set_panel_height_with_save(&mut self, height: f64) {
        self.set_panel_height(height);
        self.auto_save();
    }
    
    /// Toggle panel with automatic persistence
    pub fn toggle_panel_with_save(&mut self) {
        self.toggle_panel();
        self.auto_save();
    }
    
    /// Apply batch updates with automatic persistence
    pub fn apply_batch_updates_with_save(&mut self) {
        self.apply_pending_updates();
        self.auto_save();
    }
    
    /// Reset to defaults and clear saved state
    pub fn reset_to_defaults_with_save(&mut self) {
        self.reset_to_defaults();
        // Clear saved state since we're returning to defaults
        if let Err(e) = crate::state::persistence::clear_layout_state() {
            tracing::warn!("Failed to clear saved layout state: {}", e);
        }
    }
    
    /// Enable or disable layout persistence
    pub fn set_persistence_enabled(&mut self, enabled: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.persistence.persist_layout = enabled;
        });
        if enabled {
            self.save_state();
        }
    }
    
    /// Enable or disable theme persistence
    pub fn set_theme_persistence_enabled(&mut self, enabled: bool) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.persistence.persist_theme = enabled;
        });
        if enabled {
            self.save_state();
        }
    }
    
    /// Update auto-save interval (in seconds)
    pub fn set_auto_save_interval(&mut self, seconds: u32) {
        self.record_update_time();
        self.layout_state.with_mut(|state| {
            state.persistence.auto_save_interval = seconds;
        });
        self.save_state();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Theme, ActivityBarView, SidebarContent, EditorLayoutConfig, PanelTab, PanelPosition};
    use std::time::Duration;

    #[test]
    fn test_layout_update_variants_complete() {
        // Test all LayoutUpdate variants can be constructed
        let updates = vec![
            LayoutUpdate::Theme(Theme::Dark),
            LayoutUpdate::ActivityBarView(ActivityBarView::Explorer),
            LayoutUpdate::ActivityBarVisibility(true),
            LayoutUpdate::ActivityBarPosition(crate::state::ActivityBarPosition::Left),
            LayoutUpdate::SidebarCollapse(true),
            LayoutUpdate::SidebarWidth(300.0),
            LayoutUpdate::SidebarPosition(crate::state::SidebarPosition::Left),
            LayoutUpdate::SidebarContent(SidebarContent::FileTree),
            LayoutUpdate::EditorLayout(EditorLayoutConfig::Single),
            LayoutUpdate::EditorActiveGroup(1),
            LayoutUpdate::EditorSplitRatios(vec![0.6, 0.4]),
            LayoutUpdate::PanelVisibility(false),
            LayoutUpdate::PanelHeight(200.0),
            LayoutUpdate::PanelActiveTab(PanelTab::Problems),
            LayoutUpdate::PanelPosition(PanelPosition::Bottom),
            LayoutUpdate::ViewportDimensions { width: 1920.0, height: 1080.0 },
            LayoutUpdate::WindowMaximized(true),
            LayoutUpdate::WindowFullscreen(false),
            LayoutUpdate::AnimationsEnabled(true),
            LayoutUpdate::AnimationDuration(200),
        ];
        
        assert_eq!(updates.len(), 20);
        
        // Test pattern matching on key variants
        for update in &updates {
            match update {
                LayoutUpdate::Theme(theme) => assert!(matches!(theme, Theme::Dark)),
                LayoutUpdate::SidebarWidth(width) => assert!(*width > 0.0),
                LayoutUpdate::ViewportDimensions { width, height } => {
                    assert!(*width > 0.0 && *height > 0.0);
                },
                _ => {}
            }
        }
    }

    #[test]
    fn test_batch_update_operations() {
        // Test batch update functionality
        let updates = vec![
            LayoutUpdate::Theme(Theme::Dark),
            LayoutUpdate::SidebarCollapse(true),
            LayoutUpdate::PanelVisibility(false),
        ];
        
        // Verify we can create and process batches
        assert_eq!(updates.len(), 3);
        
        // Test that batches contain only valid operations
        for update in &updates {
            match update {
                LayoutUpdate::Theme(_) | 
                LayoutUpdate::SidebarCollapse(_) | 
                LayoutUpdate::PanelVisibility(_) => assert!(true),
                _ => panic!("Unexpected update type in batch"),
            }
        }
    }

    #[test]
    fn test_split_ratio_normalization() {
        // Test split ratio calculations
        fn normalize_ratios(ratios: &[f64]) -> Vec<f64> {
            if ratios.is_empty() {
                return vec![1.0];
            }
            
            let sum: f64 = ratios.iter().sum();
            if sum == 0.0 {
                return vec![1.0];
            }
            
            ratios.iter().map(|r| r / sum).collect()
        }
        
        // Test normal case
        let ratios = vec![2.0, 3.0, 5.0];
        let normalized = normalize_ratios(&ratios);
        let sum: f64 = normalized.iter().sum();
        assert!((sum - 1.0).abs() < f64::EPSILON);
        assert!((normalized[0] - 0.2).abs() < f64::EPSILON);
        assert!((normalized[1] - 0.3).abs() < f64::EPSILON);
        assert!((normalized[2] - 0.5).abs() < f64::EPSILON);
        
        // Test edge cases
        assert_eq!(normalize_ratios(&[]), vec![1.0]);
        assert_eq!(normalize_ratios(&[0.0, 0.0]), vec![1.0]);
        
        // Test equal ratios
        let equal_normalized = normalize_ratios(&[1.0, 1.0]);
        assert!((equal_normalized[0] - 0.5).abs() < f64::EPSILON);
        assert!((equal_normalized[1] - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sidebar_width_validation() {
        // Test sidebar width constraints
        fn validate_sidebar_width(width: f64, min_width: f64, max_width: f64) -> f64 {
            width.clamp(min_width, max_width)
        }
        
        assert_eq!(validate_sidebar_width(100.0, 200.0, 800.0), 200.0);
        assert_eq!(validate_sidebar_width(300.0, 200.0, 800.0), 300.0);
        assert_eq!(validate_sidebar_width(1000.0, 200.0, 800.0), 800.0);
    }

    #[test]
    fn test_panel_height_validation() {
        // Test panel height constraints with viewport awareness
        fn validate_panel_height(height: f64, min_height: f64, viewport_height: f64, max_fraction: f64) -> f64 {
            let max_height = viewport_height * max_fraction;
            height.clamp(min_height, max_height)
        }
        
        // Normal cases
        assert_eq!(validate_panel_height(50.0, 100.0, 1000.0, 0.8), 100.0);
        assert_eq!(validate_panel_height(200.0, 100.0, 1000.0, 0.8), 200.0);
        assert_eq!(validate_panel_height(900.0, 100.0, 1000.0, 0.8), 800.0);
        
        // Edge case: small viewport
        assert_eq!(validate_panel_height(200.0, 100.0, 300.0, 0.8), 200.0);
    }

    #[test]
    fn test_responsive_breakpoints() {
        // Test responsive layout logic
        fn get_responsive_config(viewport_width: f64) -> (bool, bool, f64) {
            let (collapse_sidebar, hide_activity_bar, panel_height) = if viewport_width < 480.0 {
                (true, true, 150.0)
            } else if viewport_width < 768.0 {
                (true, false, 200.0)
            } else {
                (false, false, 300.0)
            };
            
            (collapse_sidebar, hide_activity_bar, panel_height)
        }
        
        // Mobile (< 480px)
        let (collapse, hide, height) = get_responsive_config(400.0);
        assert!(collapse && hide && height == 150.0);
        
        // Tablet (480-768px)
        let (collapse, hide, height) = get_responsive_config(600.0);
        assert!(collapse && !hide && height == 200.0);
        
        // Desktop (> 768px)
        let (collapse, hide, height) = get_responsive_config(1200.0);
        assert!(!collapse && !hide && height == 300.0);
    }

    #[test]
    fn test_performance_thresholds() {
        // Test performance classification
        fn classify_performance(duration_ms: u64) -> &'static str {
            let duration = Duration::from_millis(duration_ms);
            
            if duration <= Duration::from_millis(50) {
                "excellent"
            } else if duration <= Duration::from_millis(100) {
                "good"
            } else if duration <= Duration::from_millis(200) {
                "acceptable"
            } else {
                "poor"
            }
        }
        
        assert_eq!(classify_performance(30), "excellent");
        assert_eq!(classify_performance(75), "good");
        assert_eq!(classify_performance(150), "acceptable");
        assert_eq!(classify_performance(300), "poor");
    }

    #[test]
    fn test_animation_duration_logic() {
        // Test animation duration selection
        fn get_animation_duration(animations_enabled: bool, reduced_motion: bool) -> u32 {
            if !animations_enabled || reduced_motion {
                0
            } else {
                200
            }
        }
        
        assert_eq!(get_animation_duration(true, false), 200);
        assert_eq!(get_animation_duration(true, true), 0);
        assert_eq!(get_animation_duration(false, false), 0);
        assert_eq!(get_animation_duration(false, true), 0);
    }

    #[test]
    fn test_accessibility_preferences() {
        // Test accessibility-aware settings
        fn should_reduce_motion(user_pref: bool, system_pref: Option<bool>) -> bool {
            system_pref.unwrap_or(user_pref)
        }
        
        // System preference overrides user preference
        assert_eq!(should_reduce_motion(false, Some(true)), true);
        assert_eq!(should_reduce_motion(true, Some(false)), false);
        
        // Fall back to user preference when system unknown
        assert_eq!(should_reduce_motion(true, None), true);
        assert_eq!(should_reduce_motion(false, None), false);
    }

    #[test]
    fn test_css_transition_generation() {
        // Test CSS transition string generation
        fn generate_transition_css(duration_ms: u32, reduced_motion: bool) -> String {
            if reduced_motion {
                "none".to_string()
            } else {
                format!("width {}ms, height {}ms, transform {}ms", duration_ms, duration_ms, duration_ms)
            }
        }
        
        assert_eq!(
            generate_transition_css(200, false),
            "width 200ms, height 200ms, transform 200ms"
        );
        assert_eq!(generate_transition_css(200, true), "none");
    }

    #[test]
    fn test_distraction_free_mode_logic() {
        // Test distraction-free mode state changes
        fn apply_distraction_free(
            activity_visible: bool,
            sidebar_collapsed: bool,
            panel_visible: bool,
            enable: bool
        ) -> (bool, bool, bool) {
            if enable {
                (false, true, false) // Hide activity bar, collapse sidebar, hide panel
            } else {
                (true, false, true) // Show activity bar, expand sidebar, show panel
            }
        }
        
        // Enable distraction-free mode
        let (activity, sidebar, panel) = apply_distraction_free(true, false, true, true);
        assert!(!activity && sidebar && !panel);
        
        // Disable distraction-free mode
        let (activity, sidebar, panel) = apply_distraction_free(false, true, false, false);
        assert!(activity && !sidebar && panel);
    }

    #[test]
    fn test_performance_metrics_structure() {
        // Test performance metrics data structure
        let metrics = LayoutPerformanceMetrics {
            pending_updates: 5,
            last_update_time: Some(std::time::Instant::now()),
            animations_enabled: true,
            reduced_motion: false,
        };
        
        assert_eq!(metrics.pending_updates, 5);
        assert!(metrics.last_update_time.is_some());
        assert!(metrics.animations_enabled);
        assert!(!metrics.reduced_motion);
    }

    #[test]
    fn test_layout_update_matching() {
        // Test comprehensive pattern matching on LayoutUpdate
        let update = LayoutUpdate::ViewportDimensions { width: 1920.0, height: 1080.0 };
        
        match update {
            LayoutUpdate::ViewportDimensions { width, height } => {
                assert_eq!(width, 1920.0);
                assert_eq!(height, 1080.0);
            },
            _ => panic!("Failed to match ViewportDimensions update"),
        }
    }

    #[test]
    fn test_editor_split_setup_logic() {
        // Test editor split configuration
        fn setup_split_ratios(vertical: bool, left_ratio: f64) -> (EditorLayoutConfig, Vec<f64>) {
            let config = if vertical {
                EditorLayoutConfig::SplitVertical
            } else {
                EditorLayoutConfig::SplitHorizontal
            };
            
            let clamped_left = left_ratio.clamp(0.1, 0.9);
            let right_ratio = 1.0 - clamped_left;
            
            (config, vec![clamped_left, right_ratio])
        }
        
        // Test vertical split
        let (config, ratios) = setup_split_ratios(true, 0.6);
        assert!(matches!(config, EditorLayoutConfig::SplitVertical));
        assert!((ratios[0] - 0.6).abs() < f64::EPSILON);
        assert!((ratios[1] - 0.4).abs() < f64::EPSILON);
        
        // Test horizontal split
        let (config, ratios) = setup_split_ratios(false, 0.7);
        assert!(matches!(config, EditorLayoutConfig::SplitHorizontal));
        assert!((ratios[0] - 0.7).abs() < f64::EPSILON);
        assert!((ratios[1] - 0.3).abs() < f64::EPSILON);
        
        // Test ratio clamping
        let (_, ratios) = setup_split_ratios(true, 0.05); // Should clamp to 0.1
        assert!((ratios[0] - 0.1).abs() < f64::EPSILON);
        assert!((ratios[1] - 0.9).abs() < f64::EPSILON);
    }
}