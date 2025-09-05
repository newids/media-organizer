use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use crate::state::{LayoutState, Theme, ActivityBarView, SidebarContent, EditorLayoutConfig, PanelTab};

/// Advanced batch update system for optimizing layout state changes
/// Implements intelligent batching, deduplication, and scheduling

const DEFAULT_BATCH_DELAY: Duration = Duration::from_millis(16); // ~60fps
const MAX_BATCH_SIZE: usize = 50;
const MAX_BATCH_AGE: Duration = Duration::from_millis(100);

/// Enhanced layout update with priority and deduplication support
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutUpdateV2 {
    // Theme and visual
    Theme(Theme),
    AnimationsEnabled(bool),
    ReducedMotion(bool),
    
    // Activity bar
    ActivityBarVisible(bool),
    ActivityBarView(ActivityBarView),
    ActivityBarWidth(f64),
    
    // Sidebar
    SidebarCollapsed(bool),
    SidebarWidth(f64),
    SidebarContent(SidebarContent),
    
    // Editor
    EditorLayout(EditorLayoutConfig),
    EditorActiveGroup(usize),
    EditorShowTabs(bool),
    
    // Panel
    PanelVisible(bool),
    PanelHeight(f64),
    PanelActiveTab(PanelTab),
    
    // Viewport
    ViewportDimensions { width: f64, height: f64 },
    WindowMaximized(bool),
    WindowFullscreen(bool),
    
    // Batch operations
    CompoundUpdate(Vec<LayoutUpdateV2>),
}

impl LayoutUpdateV2 {
    /// Get the priority of this update (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            // Critical updates that affect layout immediately
            LayoutUpdateV2::ViewportDimensions { .. } => 10,
            LayoutUpdateV2::WindowMaximized(_) | LayoutUpdateV2::WindowFullscreen(_) => 9,
            
            // High priority visual updates
            LayoutUpdateV2::Theme(_) => 8,
            LayoutUpdateV2::SidebarCollapsed(_) => 8,
            LayoutUpdateV2::PanelVisible(_) => 8,
            
            // Medium priority layout updates
            LayoutUpdateV2::EditorLayout(_) => 6,
            LayoutUpdateV2::SidebarWidth(_) => 6,
            LayoutUpdateV2::PanelHeight(_) => 6,
            
            // Lower priority state updates
            LayoutUpdateV2::ActivityBarView(_) => 4,
            LayoutUpdateV2::EditorActiveGroup(_) => 4,
            LayoutUpdateV2::PanelActiveTab(_) => 4,
            
            // Lowest priority cosmetic updates
            LayoutUpdateV2::AnimationsEnabled(_) => 2,
            LayoutUpdateV2::ReducedMotion(_) => 2,
            LayoutUpdateV2::ActivityBarWidth(_) => 2,
            LayoutUpdateV2::EditorShowTabs(_) => 2,
            LayoutUpdateV2::ActivityBarVisible(_) => 2,
            LayoutUpdateV2::SidebarContent(_) => 2,
            
            // Compound updates inherit highest priority
            LayoutUpdateV2::CompoundUpdate(updates) => {
                updates.iter().map(|u| u.priority()).max().unwrap_or(5)
            }
        }
    }

    /// Get the update category for deduplication
    pub fn category(&self) -> &'static str {
        match self {
            LayoutUpdateV2::Theme(_) => "theme",
            LayoutUpdateV2::AnimationsEnabled(_) => "animations_enabled",
            LayoutUpdateV2::ReducedMotion(_) => "reduced_motion",
            LayoutUpdateV2::ActivityBarVisible(_) => "activity_bar_visible",
            LayoutUpdateV2::ActivityBarView(_) => "activity_bar_view",
            LayoutUpdateV2::ActivityBarWidth(_) => "activity_bar_width",
            LayoutUpdateV2::SidebarCollapsed(_) => "sidebar_collapsed",
            LayoutUpdateV2::SidebarWidth(_) => "sidebar_width",
            LayoutUpdateV2::SidebarContent(_) => "sidebar_content",
            LayoutUpdateV2::EditorLayout(_) => "editor_layout",
            LayoutUpdateV2::EditorActiveGroup(_) => "editor_active_group",
            LayoutUpdateV2::EditorShowTabs(_) => "editor_show_tabs",
            LayoutUpdateV2::PanelVisible(_) => "panel_visible",
            LayoutUpdateV2::PanelHeight(_) => "panel_height",
            LayoutUpdateV2::PanelActiveTab(_) => "panel_active_tab",
            LayoutUpdateV2::ViewportDimensions { .. } => "viewport_dimensions",
            LayoutUpdateV2::WindowMaximized(_) => "window_maximized",
            LayoutUpdateV2::WindowFullscreen(_) => "window_fullscreen",
            LayoutUpdateV2::CompoundUpdate(_) => "compound",
        }
    }

    /// Check if this update can be merged with another update of the same category
    pub fn can_merge_with(&self, other: &LayoutUpdateV2) -> bool {
        self.category() == other.category() && self.category() != "compound"
    }

    /// Apply this update to a layout state
    pub fn apply_to_state(&self, state: &mut LayoutState) {
        match self {
            LayoutUpdateV2::Theme(theme) => state.theme = theme.clone(),
            LayoutUpdateV2::AnimationsEnabled(enabled) => state.ui_preferences.enable_animations = *enabled,
            LayoutUpdateV2::ReducedMotion(reduced) => state.ui_preferences.reduced_motion = *reduced,
            LayoutUpdateV2::ActivityBarVisible(visible) => state.activity_bar.is_visible = *visible,
            LayoutUpdateV2::ActivityBarView(view) => state.activity_bar.active_view = view.clone(),
            LayoutUpdateV2::ActivityBarWidth(width) => state.activity_bar.width = *width,
            LayoutUpdateV2::SidebarCollapsed(collapsed) => state.sidebar.is_collapsed = *collapsed,
            LayoutUpdateV2::SidebarWidth(width) => {
                state.sidebar.width = width.max(state.sidebar.min_width).min(state.sidebar.max_width);
            },
            LayoutUpdateV2::SidebarContent(content) => state.sidebar.active_content = content.clone(),
            LayoutUpdateV2::EditorLayout(layout) => state.editor_layout.layout_config = layout.clone(),
            LayoutUpdateV2::EditorActiveGroup(group) => state.editor_layout.active_group = *group,
            LayoutUpdateV2::EditorShowTabs(show) => state.editor_layout.show_tabs = *show,
            LayoutUpdateV2::PanelVisible(visible) => state.panel.is_visible = *visible,
            LayoutUpdateV2::PanelHeight(height) => {
                let max_height = state.viewport.height * state.panel.max_height_fraction;
                state.panel.height = height.max(state.panel.min_height).min(max_height);
            },
            LayoutUpdateV2::PanelActiveTab(tab) => state.panel.active_tab = tab.clone(),
            LayoutUpdateV2::ViewportDimensions { width, height } => {
                state.viewport.width = *width;
                state.viewport.height = *height;
            },
            LayoutUpdateV2::WindowMaximized(maximized) => state.viewport.is_maximized = *maximized,
            LayoutUpdateV2::WindowFullscreen(fullscreen) => state.viewport.is_fullscreen = *fullscreen,
            LayoutUpdateV2::CompoundUpdate(updates) => {
                for update in updates {
                    update.apply_to_state(state);
                }
            },
        }
    }
}

/// Batch metadata for tracking and optimization
#[derive(Debug, Clone)]
struct BatchMetadata {
    /// When this batch was created
    created_at: Instant,
    /// When this batch was last modified
    last_modified: Instant,
    /// Priority of the highest priority update in the batch
    max_priority: u8,
    /// Number of updates in the batch
    update_count: usize,
    /// Estimated cost of applying this batch
    estimated_cost: u32,
}

impl BatchMetadata {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            created_at: now,
            last_modified: now,
            max_priority: 0,
            update_count: 0,
            estimated_cost: 0,
        }
    }

    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    fn time_since_modification(&self) -> Duration {
        self.last_modified.elapsed()
    }

    fn should_flush(&self, max_size: usize, max_age: Duration, min_delay: Duration) -> bool {
        self.update_count >= max_size 
            || self.age() >= max_age
            || (self.max_priority >= 8 && self.time_since_modification() >= min_delay)
    }
}

/// Advanced batch optimization system
pub struct BatchOptimizer {
    /// Pending updates organized by category for deduplication
    pending_updates: HashMap<String, LayoutUpdateV2>,
    /// Batch metadata
    metadata: BatchMetadata,
    /// Update statistics
    stats: BatchOptimizerStats,
    /// Configuration
    config: BatchOptimizerConfig,
}

/// Configuration for the batch optimizer
#[derive(Debug, Clone)]
pub struct BatchOptimizerConfig {
    /// Maximum number of updates in a batch
    pub max_batch_size: usize,
    /// Maximum age of a batch before forced flush
    pub max_batch_age: Duration,
    /// Minimum delay between batches
    pub min_batch_delay: Duration,
    /// Enable intelligent priority-based batching
    pub enable_priority_batching: bool,
    /// Enable update deduplication
    pub enable_deduplication: bool,
}

impl Default for BatchOptimizerConfig {
    fn default() -> Self {
        Self {
            max_batch_size: MAX_BATCH_SIZE,
            max_batch_age: MAX_BATCH_AGE,
            min_batch_delay: DEFAULT_BATCH_DELAY,
            enable_priority_batching: true,
            enable_deduplication: true,
        }
    }
}

/// Statistics for batch optimizer performance
#[derive(Debug, Clone, Default)]
pub struct BatchOptimizerStats {
    pub total_updates_received: u64,
    pub total_updates_deduplicated: u64,
    pub total_batches_processed: u64,
    pub total_updates_applied: u64,
    pub avg_batch_size: f64,
    pub avg_processing_time: Duration,
    pub deduplication_rate: f64,
}

impl BatchOptimizer {
    /// Create a new batch optimizer with default configuration
    pub fn new() -> Self {
        Self::with_config(BatchOptimizerConfig::default())
    }

    /// Create a new batch optimizer with custom configuration
    pub fn with_config(config: BatchOptimizerConfig) -> Self {
        Self {
            pending_updates: HashMap::new(),
            metadata: BatchMetadata::new(),
            stats: BatchOptimizerStats::default(),
            config,
        }
    }

    /// Add an update to the batch
    pub fn add_update(&mut self, update: LayoutUpdateV2) {
        self.stats.total_updates_received += 1;

        if self.config.enable_deduplication {
            let category = update.category().to_string();
            
            // Check if we're replacing an existing update (deduplication)
            if self.pending_updates.contains_key(&category) {
                self.stats.total_updates_deduplicated += 1;
            }
            
            self.pending_updates.insert(category, update.clone());
        } else {
            // Without deduplication, use a unique key for each update
            let key = format!("{}_{}", update.category(), self.stats.total_updates_received);
            self.pending_updates.insert(key, update.clone());
        }

        // Update metadata
        self.metadata.last_modified = Instant::now();
        self.metadata.max_priority = self.metadata.max_priority.max(update.priority());
        self.metadata.update_count = self.pending_updates.len();
        self.metadata.estimated_cost += self.estimate_update_cost(&update);

        // Update deduplication rate
        if self.stats.total_updates_received > 0 {
            self.stats.deduplication_rate = 
                self.stats.total_updates_deduplicated as f64 / self.stats.total_updates_received as f64;
        }
    }

    /// Check if the current batch should be flushed
    pub fn should_flush(&self) -> bool {
        if self.pending_updates.is_empty() {
            return false;
        }

        self.metadata.should_flush(
            self.config.max_batch_size,
            self.config.max_batch_age,
            self.config.min_batch_delay,
        )
    }

    /// Flush the current batch and return optimized updates
    pub fn flush_batch(&mut self) -> Vec<LayoutUpdateV2> {
        if self.pending_updates.is_empty() {
            return Vec::new();
        }

        let start_time = Instant::now();

        // Collect and sort updates by priority if enabled
        let mut updates: Vec<LayoutUpdateV2> = self.pending_updates.values().cloned().collect();
        
        if self.config.enable_priority_batching {
            updates.sort_by(|a, b| b.priority().cmp(&a.priority()));
        }

        // Update statistics
        self.stats.total_batches_processed += 1;
        self.stats.total_updates_applied += updates.len() as u64;
        
        let current_avg = self.stats.avg_batch_size;
        let batch_count = self.stats.total_batches_processed as f64;
        self.stats.avg_batch_size = 
            (current_avg * (batch_count - 1.0) + updates.len() as f64) / batch_count;

        // Update processing time
        let processing_time = start_time.elapsed();
        let current_avg_time = self.stats.avg_processing_time;
        self.stats.avg_processing_time = Duration::from_nanos(
            ((current_avg_time.as_nanos() as f64 * (batch_count - 1.0) + processing_time.as_nanos() as f64) / batch_count) as u64
        );

        // Clear pending updates and reset metadata
        self.pending_updates.clear();
        self.metadata = BatchMetadata::new();

        updates
    }

    /// Get current batch statistics
    pub fn get_stats(&self) -> &BatchOptimizerStats {
        &self.stats
    }

    /// Get current batch size
    pub fn pending_update_count(&self) -> usize {
        self.pending_updates.len()
    }

    /// Get current batch age
    pub fn current_batch_age(&self) -> Duration {
        self.metadata.age()
    }

    /// Clear all pending updates without applying them
    pub fn clear_pending(&mut self) {
        self.pending_updates.clear();
        self.metadata = BatchMetadata::new();
    }

    /// Estimate the cost of applying an update (for optimization)
    fn estimate_update_cost(&self, update: &LayoutUpdateV2) -> u32 {
        match update {
            // High cost operations (affect layout)
            LayoutUpdateV2::ViewportDimensions { .. } => 10,
            LayoutUpdateV2::EditorLayout(_) => 8,
            LayoutUpdateV2::SidebarCollapsed(_) => 7,
            LayoutUpdateV2::PanelVisible(_) => 7,
            
            // Medium cost operations
            LayoutUpdateV2::Theme(_) => 5,
            LayoutUpdateV2::SidebarWidth(_) => 4,
            LayoutUpdateV2::PanelHeight(_) => 4,
            
            // Low cost operations
            LayoutUpdateV2::ActivityBarView(_) => 2,
            LayoutUpdateV2::EditorActiveGroup(_) => 2,
            LayoutUpdateV2::PanelActiveTab(_) => 2,
            
            // Very low cost operations
            LayoutUpdateV2::AnimationsEnabled(_) => 1,
            LayoutUpdateV2::ReducedMotion(_) => 1,
            LayoutUpdateV2::ActivityBarWidth(_) => 1,
            LayoutUpdateV2::EditorShowTabs(_) => 1,
            LayoutUpdateV2::ActivityBarVisible(_) => 1,
            LayoutUpdateV2::SidebarContent(_) => 1,
            LayoutUpdateV2::WindowMaximized(_) => 3,
            LayoutUpdateV2::WindowFullscreen(_) => 3,
            
            // Compound updates sum their parts
            LayoutUpdateV2::CompoundUpdate(updates) => {
                updates.iter().map(|u| self.estimate_update_cost(u)).sum()
            }
        }
    }

    /// Create an optimized compound update from multiple related updates
    pub fn create_compound_update(&self, updates: Vec<LayoutUpdateV2>) -> LayoutUpdateV2 {
        LayoutUpdateV2::CompoundUpdate(updates)
    }

    /// Analyze current batching efficiency and provide recommendations
    pub fn analyze_efficiency(&self) -> BatchEfficiencyReport {
        let dedup_efficiency = if self.stats.total_updates_received > 0 {
            (self.stats.total_updates_deduplicated as f64 / self.stats.total_updates_received as f64) * 100.0
        } else {
            0.0
        };

        let avg_processing_time_ms = self.stats.avg_processing_time.as_secs_f64() * 1000.0;
        
        let efficiency_grade = if dedup_efficiency > 20.0 && avg_processing_time_ms < 5.0 {
            'A'
        } else if dedup_efficiency > 10.0 && avg_processing_time_ms < 10.0 {
            'B'
        } else if dedup_efficiency > 5.0 && avg_processing_time_ms < 20.0 {
            'C'
        } else {
            'D'
        };

        let mut recommendations = Vec::new();
        
        if dedup_efficiency < 10.0 {
            recommendations.push("Consider reducing update frequency to improve deduplication".to_string());
        }
        
        if avg_processing_time_ms > 10.0 {
            recommendations.push("Batch processing time is high, consider reducing batch size".to_string());
        }
        
        if self.stats.avg_batch_size < 3.0 {
            recommendations.push("Batch sizes are small, consider increasing batch delay".to_string());
        }

        if self.stats.avg_batch_size > 20.0 {
            recommendations.push("Batch sizes are large, consider reducing max batch size".to_string());
        }

        BatchEfficiencyReport {
            deduplication_efficiency: dedup_efficiency,
            avg_processing_time_ms,
            avg_batch_size: self.stats.avg_batch_size,
            efficiency_grade,
            recommendations,
        }
    }
}

/// Batch efficiency analysis report
#[derive(Debug, Clone)]
pub struct BatchEfficiencyReport {
    pub deduplication_efficiency: f64,
    pub avg_processing_time_ms: f64,
    pub avg_batch_size: f64,
    pub efficiency_grade: char,
    pub recommendations: Vec<String>,
}

impl BatchEfficiencyReport {
    pub fn print_report(&self) {
        println!("📊 Batch Optimizer Efficiency Report");
        println!("=====================================");
        println!("Deduplication Efficiency: {:.1}%", self.deduplication_efficiency);
        println!("Avg Processing Time: {:.2}ms", self.avg_processing_time_ms);
        println!("Avg Batch Size: {:.1} updates", self.avg_batch_size);
        println!("Efficiency Grade: {}", self.efficiency_grade);
        
        if !self.recommendations.is_empty() {
            println!("\nRecommendations:");
            for (i, rec) in self.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_optimizer_creation() {
        let optimizer = BatchOptimizer::new();
        assert_eq!(optimizer.pending_update_count(), 0);
        assert!(optimizer.current_batch_age() < Duration::from_millis(1));
    }

    #[test]
    fn test_update_deduplication() {
        let mut optimizer = BatchOptimizer::new();
        
        // Add two theme updates - should deduplicate
        optimizer.add_update(LayoutUpdateV2::Theme(Theme::Light));
        optimizer.add_update(LayoutUpdateV2::Theme(Theme::Dark));
        
        assert_eq!(optimizer.pending_update_count(), 1); // Should be deduplicated
        assert_eq!(optimizer.get_stats().total_updates_deduplicated, 1);
        
        let updates = optimizer.flush_batch();
        assert_eq!(updates.len(), 1);
        
        if let LayoutUpdateV2::Theme(theme) = &updates[0] {
            assert_eq!(*theme, Theme::Dark); // Should keep the latest
        } else {
            panic!("Expected theme update");
        }
    }

    #[test]
    fn test_priority_ordering() {
        let mut optimizer = BatchOptimizer::new();
        
        // Add updates in random order
        optimizer.add_update(LayoutUpdateV2::AnimationsEnabled(true)); // Priority 2
        optimizer.add_update(LayoutUpdateV2::ViewportDimensions { width: 1920.0, height: 1080.0 }); // Priority 10
        optimizer.add_update(LayoutUpdateV2::Theme(Theme::Dark)); // Priority 8
        
        let updates = optimizer.flush_batch();
        assert_eq!(updates.len(), 3);
        
        // Should be ordered by priority (highest first)
        assert_eq!(updates[0].priority(), 10); // Viewport
        assert_eq!(updates[1].priority(), 8);  // Theme
        assert_eq!(updates[2].priority(), 2);  // Animations
    }

    #[test]
    fn test_update_cost_estimation() {
        let optimizer = BatchOptimizer::new();
        
        let high_cost = LayoutUpdateV2::ViewportDimensions { width: 1920.0, height: 1080.0 };
        let low_cost = LayoutUpdateV2::AnimationsEnabled(true);
        
        assert!(optimizer.estimate_update_cost(&high_cost) > optimizer.estimate_update_cost(&low_cost));
    }

    #[test]
    fn test_batch_should_flush() {
        let mut optimizer = BatchOptimizer::with_config(BatchOptimizerConfig {
            max_batch_size: 2,
            max_batch_age: Duration::from_millis(10),
            min_batch_delay: Duration::from_millis(1),
            enable_priority_batching: true,
            enable_deduplication: true,
        });
        
        assert!(!optimizer.should_flush()); // Empty batch
        
        optimizer.add_update(LayoutUpdateV2::Theme(Theme::Dark));
        assert!(!optimizer.should_flush()); // Not full yet
        
        optimizer.add_update(LayoutUpdateV2::AnimationsEnabled(true));
        assert!(optimizer.should_flush()); // Full batch
    }

    #[test]
    fn test_compound_updates() {
        let optimizer = BatchOptimizer::new();
        
        let updates = vec![
            LayoutUpdateV2::Theme(Theme::Dark),
            LayoutUpdateV2::SidebarCollapsed(true),
        ];
        
        let compound = optimizer.create_compound_update(updates.clone());
        
        if let LayoutUpdateV2::CompoundUpdate(compound_updates) = compound {
            assert_eq!(compound_updates.len(), 2);
        } else {
            panic!("Expected compound update");
        }
    }

    #[test]
    fn test_efficiency_analysis() {
        let mut optimizer = BatchOptimizer::new();
        
        // Add some updates to generate statistics
        for i in 0..10 {
            optimizer.add_update(LayoutUpdateV2::Theme(if i % 2 == 0 { Theme::Dark } else { Theme::Light }));
        }
        
        optimizer.flush_batch();
        
        let report = optimizer.analyze_efficiency();
        assert!(report.deduplication_efficiency > 0.0);
        assert!(report.avg_batch_size > 0.0);
    }
}