//! Rendering optimization utilities for MediaOrganizer UI performance
//!
//! This module provides optimizations to minimize reflows, reduce unnecessary renders,
//! and improve overall UI responsiveness for large file collections.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use dioxus::prelude::*;

/// Performance thresholds for UI optimization decisions
#[derive(Debug, Clone)]
pub struct RenderingThresholds {
    /// Maximum theme detection frequency (milliseconds)
    pub theme_detection_interval_ms: u64,
    /// Debounce time for drag operations (milliseconds)
    pub drag_debounce_ms: u64,
    /// Maximum file count for full rendering
    pub virtual_scroll_threshold: usize,
    /// Batch size for DOM updates
    pub dom_update_batch_size: usize,
    /// Minimum time between layout recalculations
    pub layout_recalc_throttle_ms: u64,
}

impl Default for RenderingThresholds {
    fn default() -> Self {
        Self {
            theme_detection_interval_ms: 5000, // Reduce from current ~5s polling
            drag_debounce_ms: 100,
            virtual_scroll_threshold: 1000,
            dom_update_batch_size: 50,
            layout_recalc_throttle_ms: 16, // ~60 FPS
        }
    }
}

/// Rendering performance tracker
#[derive(Debug, Clone)]
pub struct RenderingProfiler {
    frame_times: Vec<Duration>,
    theme_detections: Vec<Instant>,
    drag_operations: Vec<Instant>,
    layout_recalculations: u32,
    batch_updates: u32,
    thresholds: RenderingThresholds,
}

impl RenderingProfiler {
    pub fn new() -> Self {
        Self {
            frame_times: Vec::with_capacity(60), // Track last 60 frames
            theme_detections: Vec::new(),
            drag_operations: Vec::new(),
            layout_recalculations: 0,
            batch_updates: 0,
            thresholds: RenderingThresholds::default(),
        }
    }

    /// Record a frame render time
    pub fn record_frame(&mut self, duration: Duration) {
        self.frame_times.push(duration);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
    }

    /// Check if theme detection should be throttled
    pub fn should_detect_theme(&mut self) -> bool {
        let now = Instant::now();
        let threshold = Duration::from_millis(self.thresholds.theme_detection_interval_ms);
        
        if let Some(&last_detection) = self.theme_detections.last() {
            if now.duration_since(last_detection) < threshold {
                return false;
            }
        }
        
        self.theme_detections.push(now);
        true
    }

    /// Check if drag operation should be processed (debounced)
    pub fn should_process_drag(&mut self) -> bool {
        let now = Instant::now();
        let threshold = Duration::from_millis(self.thresholds.drag_debounce_ms);
        
        if let Some(&last_drag) = self.drag_operations.last() {
            if now.duration_since(last_drag) < threshold {
                return false;
            }
        }
        
        self.drag_operations.push(now);
        true
    }

    /// Get average frame time
    pub fn average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }
        
        let total: Duration = self.frame_times.iter().sum();
        Some(total / self.frame_times.len() as u32)
    }

    /// Get current FPS estimate
    pub fn current_fps(&self) -> Option<f64> {
        self.average_frame_time().map(|avg| {
            1000.0 / avg.as_millis() as f64
        })
    }

    /// Record a layout recalculation
    pub fn record_layout_recalc(&mut self) {
        self.layout_recalculations += 1;
    }

    /// Record a batch update
    pub fn record_batch_update(&mut self) {
        self.batch_updates += 1;
    }

    /// Get optimization recommendations
    pub fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if let Some(fps) = self.current_fps() {
            if fps < 30.0 {
                recommendations.push("Frame rate below 30 FPS - consider virtual scrolling".to_string());
            }
            if fps < 60.0 && fps >= 30.0 {
                recommendations.push("Frame rate below 60 FPS - optimize rendering paths".to_string());
            }
        }
        
        if self.theme_detections.len() > 10 {
            recommendations.push("High theme detection frequency - implement caching".to_string());
        }
        
        if self.drag_operations.len() > 20 {
            recommendations.push("High drag operation frequency - increase debounce threshold".to_string());
        }
        
        recommendations
    }
}

/// Theme detection optimizer - reduces unnecessary system theme polling
#[derive(Debug, Clone)]
pub struct ThemeOptimizer {
    last_detection: Option<Instant>,
    cached_theme: Option<String>,
    profiler: Arc<std::sync::Mutex<RenderingProfiler>>,
}

impl ThemeOptimizer {
    pub fn new(profiler: Arc<std::sync::Mutex<RenderingProfiler>>) -> Self {
        Self {
            last_detection: None,
            cached_theme: None,
            profiler,
        }
    }

    /// Get theme with throttling
    pub fn get_optimized_theme(&mut self) -> Option<String> {
        let should_detect = {
            let mut profiler = self.profiler.lock().unwrap();
            profiler.should_detect_theme()
        };

        if !should_detect {
            return self.cached_theme.clone();
        }

        // In real implementation, this would call the actual theme detection
        // For now, simulate the detection
        let theme = self.detect_system_theme();
        self.cached_theme = Some(theme.clone());
        self.last_detection = Some(Instant::now());
        
        Some(theme)
    }

    fn detect_system_theme(&self) -> String {
        // Placeholder for actual theme detection
        // In real code, this would interface with the OS
        "dark".to_string()
    }
}

/// Drag operation optimizer - reduces drag event noise
#[derive(Debug, Clone)]
pub struct DragOptimizer {
    profiler: Arc<std::sync::Mutex<RenderingProfiler>>,
    pending_drag: Option<(String, Instant)>,
}

impl DragOptimizer {
    pub fn new(profiler: Arc<std::sync::Mutex<RenderingProfiler>>) -> Self {
        Self {
            profiler,
            pending_drag: None,
        }
    }

    /// Process drag event with debouncing
    pub fn process_drag_event(&mut self, operation: String) -> bool {
        let should_process = {
            let mut profiler = self.profiler.lock().unwrap();
            profiler.should_process_drag()
        };

        if should_process {
            self.pending_drag = Some((operation, Instant::now()));
            true
        } else {
            false
        }
    }

    /// Get the latest pending drag operation
    pub fn get_pending_drag(&mut self) -> Option<String> {
        self.pending_drag.take().map(|(op, _)| op)
    }
}

/// Virtual scrolling optimizer for large file lists
#[derive(Debug, Clone)]
pub struct VirtualScrollOptimizer {
    viewport_height: f64,
    item_height: f64,
    total_items: usize,
    scroll_position: f64,
    profiler: Arc<std::sync::Mutex<RenderingProfiler>>,
}

impl VirtualScrollOptimizer {
    pub fn new(profiler: Arc<std::sync::Mutex<RenderingProfiler>>) -> Self {
        Self {
            viewport_height: 600.0, // Default viewport height
            item_height: 40.0,      // Default item height
            total_items: 0,
            scroll_position: 0.0,
            profiler,
        }
    }

    /// Update virtual scroll parameters
    pub fn update_viewport(&mut self, height: f64, scroll_pos: f64) {
        self.viewport_height = height;
        self.scroll_position = scroll_pos;
    }

    /// Update total item count
    pub fn set_total_items(&mut self, count: usize) {
        self.total_items = count;
    }

    /// Calculate visible item range for rendering
    pub fn get_visible_range(&self) -> (usize, usize) {
        let items_per_viewport = (self.viewport_height / self.item_height).ceil() as usize;
        let start_index = (self.scroll_position / self.item_height) as usize;
        
        // Add buffer items above and below viewport
        let buffer_size = 10;
        let start_with_buffer = start_index.saturating_sub(buffer_size);
        let end_with_buffer = (start_index + items_per_viewport + buffer_size * 2)
            .min(self.total_items);
        
        (start_with_buffer, end_with_buffer)
    }

    /// Check if virtual scrolling is recommended
    pub fn should_use_virtual_scrolling(&self) -> bool {
        let threshold = {
            let profiler = self.profiler.lock().unwrap();
            profiler.thresholds.virtual_scroll_threshold
        };
        
        self.total_items > threshold
    }

    /// Get virtual scrolling recommendations
    pub fn get_virtual_scroll_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        
        let (start, end) = self.get_visible_range();
        let rendered_items = end - start;
        let total_height = self.total_items as f64 * self.item_height;
        let reduction_ratio = 1.0 - (rendered_items as f64 / self.total_items as f64);
        
        stats.insert("total_items".to_string(), self.total_items as f64);
        stats.insert("rendered_items".to_string(), rendered_items as f64);
        stats.insert("total_height".to_string(), total_height);
        stats.insert("viewport_height".to_string(), self.viewport_height);
        stats.insert("reduction_ratio".to_string(), reduction_ratio);
        
        stats
    }
}

/// DOM batch update optimizer - reduces layout thrashing
#[derive(Debug, Clone)]
pub struct DOMBatchOptimizer {
    pending_updates: Vec<DOMUpdate>,
    profiler: Arc<std::sync::Mutex<RenderingProfiler>>,
    last_batch: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct DOMUpdate {
    pub element_id: String,
    pub property: String,
    pub value: String,
    pub timestamp: Instant,
}

impl DOMBatchOptimizer {
    pub fn new(profiler: Arc<std::sync::Mutex<RenderingProfiler>>) -> Self {
        Self {
            pending_updates: Vec::new(),
            profiler,
            last_batch: None,
        }
    }

    /// Queue a DOM update for batching
    pub fn queue_update(&mut self, element_id: String, property: String, value: String) {
        let update = DOMUpdate {
            element_id,
            property,
            value,
            timestamp: Instant::now(),
        };
        
        self.pending_updates.push(update);
        
        // Auto-flush if batch size threshold reached
        let batch_size = {
            let profiler = self.profiler.lock().unwrap();
            profiler.thresholds.dom_update_batch_size
        };
        
        if self.pending_updates.len() >= batch_size {
            self.flush_batch();
        }
    }

    /// Force flush pending updates
    pub fn flush_batch(&mut self) {
        if self.pending_updates.is_empty() {
            return;
        }

        // Group updates by element for efficiency
        let mut grouped_updates: HashMap<String, Vec<(String, String)>> = HashMap::new();
        
        for update in self.pending_updates.drain(..) {
            grouped_updates
                .entry(update.element_id)
                .or_default()
                .push((update.property, update.value));
        }

        // In a real implementation, this would apply the batched DOM updates
        // For now, just record the batch operation
        {
            let mut profiler = self.profiler.lock().unwrap();
            profiler.record_batch_update();
        }

        self.last_batch = Some(Instant::now());
        
        tracing::debug!(
            "Flushed DOM batch with {} element groups", 
            grouped_updates.len()
        );
    }

    /// Check if batch should be flushed based on time threshold
    pub fn should_flush_by_time(&self) -> bool {
        if let Some(last_batch) = self.last_batch {
            let threshold = {
                let profiler = self.profiler.lock().unwrap();
                Duration::from_millis(profiler.thresholds.layout_recalc_throttle_ms)
            };
            
            Instant::now().duration_since(last_batch) > threshold
        } else {
            !self.pending_updates.is_empty()
        }
    }

    /// Get pending update count
    pub fn pending_count(&self) -> usize {
        self.pending_updates.len()
    }
}

/// Complete rendering optimization suite
#[derive(Debug, Clone)]
pub struct RenderingOptimizationSuite {
    profiler: Arc<std::sync::Mutex<RenderingProfiler>>,
    theme_optimizer: ThemeOptimizer,
    drag_optimizer: DragOptimizer,
    virtual_scroll: VirtualScrollOptimizer,
    dom_batch: DOMBatchOptimizer,
}

impl RenderingOptimizationSuite {
    pub fn new() -> Self {
        let profiler = Arc::new(std::sync::Mutex::new(RenderingProfiler::new()));
        
        Self {
            theme_optimizer: ThemeOptimizer::new(profiler.clone()),
            drag_optimizer: DragOptimizer::new(profiler.clone()),
            virtual_scroll: VirtualScrollOptimizer::new(profiler.clone()),
            dom_batch: DOMBatchOptimizer::new(profiler.clone()),
            profiler,
        }
    }

    /// Get optimized theme with caching and throttling
    pub fn get_optimized_theme(&mut self) -> Option<String> {
        self.theme_optimizer.get_optimized_theme()
    }

    /// Process drag event with debouncing
    pub fn process_drag_event(&mut self, operation: String) -> bool {
        self.drag_optimizer.process_drag_event(operation)
    }

    /// Update virtual scroll viewport
    pub fn update_virtual_scroll(&mut self, height: f64, scroll_pos: f64, total_items: usize) {
        self.virtual_scroll.update_viewport(height, scroll_pos);
        self.virtual_scroll.set_total_items(total_items);
    }

    /// Get visible item range for virtual scrolling
    pub fn get_visible_range(&self) -> (usize, usize) {
        self.virtual_scroll.get_visible_range()
    }

    /// Queue DOM update for batching
    pub fn queue_dom_update(&mut self, element_id: String, property: String, value: String) {
        self.dom_batch.queue_update(element_id, property, value);
    }

    /// Force flush pending DOM updates
    pub fn flush_dom_batch(&mut self) {
        self.dom_batch.flush_batch();
    }

    /// Record frame time for performance tracking
    pub fn record_frame_time(&self, duration: Duration) {
        let mut profiler = self.profiler.lock().unwrap();
        profiler.record_frame(duration);
    }

    /// Get performance report
    pub fn get_performance_report(&self) -> HashMap<String, serde_json::Value> {
        let profiler = self.profiler.lock().unwrap();
        let mut report = HashMap::new();
        
        if let Some(avg_frame) = profiler.average_frame_time() {
            report.insert("avg_frame_time_ms".to_string(), 
                         serde_json::Value::Number(serde_json::Number::from(avg_frame.as_millis() as u64)));
        }
        
        if let Some(fps) = profiler.current_fps() {
            report.insert("current_fps".to_string(), 
                         serde_json::Value::Number(serde_json::Number::from_f64(fps).unwrap_or_else(|| serde_json::Number::from(0))));
        }
        
        report.insert("theme_detections".to_string(), 
                     serde_json::Value::Number(serde_json::Number::from(profiler.theme_detections.len())));
        report.insert("drag_operations".to_string(), 
                     serde_json::Value::Number(serde_json::Number::from(profiler.drag_operations.len())));
        report.insert("layout_recalcs".to_string(), 
                     serde_json::Value::Number(serde_json::Number::from(profiler.layout_recalculations)));
        report.insert("batch_updates".to_string(), 
                     serde_json::Value::Number(serde_json::Number::from(profiler.batch_updates)));
        
        let virtual_stats = self.virtual_scroll.get_virtual_scroll_stats();
        for (key, value) in virtual_stats {
            report.insert(key, serde_json::Value::Number(serde_json::Number::from_f64(value).unwrap_or_else(|| serde_json::Number::from(0))));
        }
        
        let recommendations = profiler.get_recommendations();
        report.insert("recommendations".to_string(), 
                     serde_json::Value::Array(recommendations.into_iter().map(serde_json::Value::String).collect()));
        
        report
    }

    /// Check if optimizations are working effectively
    pub fn is_performing_well(&self) -> bool {
        let profiler = self.profiler.lock().unwrap();
        
        if let Some(fps) = profiler.current_fps() {
            fps >= 30.0 && profiler.theme_detections.len() < 10
        } else {
            true // No data yet, assume good
        }
    }
}

/// Hook for using rendering optimizations in Dioxus components
pub fn use_rendering_optimization() -> Signal<RenderingOptimizationSuite> {
    use_signal(|| RenderingOptimizationSuite::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_theme_optimization() {
        let profiler = Arc::new(std::sync::Mutex::new(RenderingProfiler::new()));
        let mut optimizer = ThemeOptimizer::new(profiler);
        
        // First call should work
        assert!(optimizer.get_optimized_theme().is_some());
        
        // Immediate second call should return cached result
        assert!(optimizer.get_optimized_theme().is_some());
    }

    #[test]
    fn test_drag_debouncing() {
        let profiler = Arc::new(std::sync::Mutex::new(RenderingProfiler::new()));
        let mut optimizer = DragOptimizer::new(profiler);
        
        // First drag should be processed
        assert!(optimizer.process_drag_event("move".to_string()));
        
        // Immediate second drag should be debounced
        assert!(!optimizer.process_drag_event("move".to_string()));
        
        // After delay, should process again
        thread::sleep(Duration::from_millis(150));
        assert!(optimizer.process_drag_event("move".to_string()));
    }

    #[test]
    fn test_virtual_scrolling() {
        let profiler = Arc::new(std::sync::Mutex::new(RenderingProfiler::new()));
        let mut optimizer = VirtualScrollOptimizer::new(profiler);
        
        optimizer.set_total_items(2000);
        optimizer.update_viewport(600.0, 0.0);
        
        let (start, end) = optimizer.get_visible_range();
        
        // Should render significantly fewer items than total
        assert!(end - start < 2000);
        assert!(optimizer.should_use_virtual_scrolling());
    }

    #[test]
    fn test_dom_batching() {
        let profiler = Arc::new(std::sync::Mutex::new(RenderingProfiler::new()));
        let mut optimizer = DOMBatchOptimizer::new(profiler);
        
        // Queue several updates
        for i in 0..10 {
            optimizer.queue_update(
                format!("element_{}", i),
                "style".to_string(),
                "color: red".to_string(),
            );
        }
        
        assert_eq!(optimizer.pending_count(), 10);
        
        // Flush batch
        optimizer.flush_batch();
        assert_eq!(optimizer.pending_count(), 0);
    }
}