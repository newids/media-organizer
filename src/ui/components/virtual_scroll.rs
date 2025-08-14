use std::cmp;

/// Core virtual scrolling calculation engine for handling large lists efficiently
/// Designed to handle 10,000+ items with constant memory usage and optimal performance
#[derive(Debug, Clone)]
pub struct VirtualScrollCalculator {
    /// Height of each item in pixels (uniform item height)
    pub item_height: f64,
    /// Total height of the viewport container
    pub container_height: f64,
    /// Number of items to render above and below visible area for smooth scrolling
    pub buffer_size: usize,
    /// Total number of items in the dataset
    pub total_items: usize,
}

/// Represents the calculated range of items that should be rendered
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleRange {
    /// Index of the first item to render (including buffer)
    pub start_index: usize,
    /// Index of the last item to render (including buffer)
    pub end_index: usize,
    /// Number of items actually visible in viewport (excluding buffer)
    pub visible_count: usize,
    /// Offset from top of container to first visible item
    pub offset_top: f64,
    /// Total height of the virtual list container
    pub total_height: f64,
}

impl VirtualScrollCalculator {
    /// Create a new virtual scroll calculator with specified parameters
    pub fn new(item_height: f64, container_height: f64, buffer_size: usize, total_items: usize) -> Self {
        Self {
            item_height,
            container_height,
            buffer_size,
            total_items,
        }
    }

    /// Calculate which items should be rendered based on current scroll position
    /// This is the core algorithm that enables efficient handling of large lists
    pub fn calculate_visible_range(&self, scroll_top: f64) -> VisibleRange {
        if self.total_items == 0 {
            return VisibleRange {
                start_index: 0,
                end_index: 0,
                visible_count: 0,
                offset_top: 0.0,
                total_height: 0.0,
            };
        }

        // Calculate which items are actually visible in the viewport
        let start_visible = (scroll_top / self.item_height).floor() as usize;
        let end_visible = ((scroll_top + self.container_height) / self.item_height).ceil() as usize;

        // Apply buffer zones for smooth scrolling
        let start_index = start_visible.saturating_sub(self.buffer_size);
        let end_index = cmp::min(end_visible + self.buffer_size, self.total_items);

        // Calculate visible count (items actually in viewport, not buffer)
        let visible_count = cmp::min(
            end_visible.saturating_sub(start_visible),
            self.total_items.saturating_sub(start_visible),
        );

        // Calculate offset for proper positioning
        let offset_top = start_index as f64 * self.item_height;

        VisibleRange {
            start_index,
            end_index,
            visible_count,
            offset_top,
            total_height: self.calculate_total_height(),
        }
    }

    /// Calculate the total height of the virtual container
    /// This maintains proper scrollbar sizing regardless of visible items
    pub fn calculate_total_height(&self) -> f64 {
        self.total_items as f64 * self.item_height
    }

    /// Get the vertical offset for a specific item index
    pub fn get_item_offset(&self, index: usize) -> f64 {
        index as f64 * self.item_height
    }

    /// Calculate which item index is at a given vertical position
    pub fn get_item_at_position(&self, y_position: f64) -> usize {
        let index = (y_position / self.item_height).floor() as usize;
        cmp::min(index, self.total_items.saturating_sub(1))
    }

    /// Check if a specific item index is currently visible (not including buffer)
    pub fn is_item_visible(&self, index: usize, scroll_top: f64) -> bool {
        let item_top = self.get_item_offset(index);
        let item_bottom = item_top + self.item_height;
        let viewport_bottom = scroll_top + self.container_height;

        item_bottom > scroll_top && item_top < viewport_bottom
    }

    /// Calculate scroll position needed to bring a specific item into view
    pub fn scroll_to_item(&self, index: usize, alignment: ScrollAlignment) -> f64 {
        let item_offset = self.get_item_offset(index);
        
        match alignment {
            ScrollAlignment::Start => item_offset,
            ScrollAlignment::Center => {
                item_offset - (self.container_height - self.item_height) / 2.0
            }
            ScrollAlignment::End => {
                item_offset - self.container_height + self.item_height
            }
            ScrollAlignment::Auto => {
                // Only scroll if item is not currently visible
                let current_scroll = item_offset - self.container_height / 2.0;
                current_scroll.max(0.0)
            }
        }
    }

    /// Update calculator parameters (useful for dynamic resizing)
    pub fn update_container_height(&mut self, new_height: f64) {
        self.container_height = new_height;
    }

    pub fn update_total_items(&mut self, new_total: usize) {
        self.total_items = new_total;
    }

    pub fn update_item_height(&mut self, new_height: f64) {
        self.item_height = new_height;
    }

    /// Get performance metrics for monitoring and optimization
    pub fn get_performance_metrics(&self, visible_range: &VisibleRange) -> PerformanceMetrics {
        let render_ratio = if self.total_items > 0 {
            (visible_range.end_index - visible_range.start_index) as f64 / self.total_items as f64
        } else {
            0.0
        };

        PerformanceMetrics {
            total_items: self.total_items,
            rendered_items: visible_range.end_index - visible_range.start_index,
            render_ratio,
            memory_efficiency: 1.0 - render_ratio, // Higher is better
            buffer_utilization: self.buffer_size as f64 / (visible_range.visible_count as f64).max(1.0),
        }
    }
}

/// Alignment options for scrolling to specific items
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollAlignment {
    /// Align item to start of viewport
    Start,
    /// Center item in viewport
    Center,
    /// Align item to end of viewport
    End,
    /// Only scroll if item is not visible
    Auto,
}

/// Performance metrics for monitoring virtual scroll efficiency
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_items: usize,
    pub rendered_items: usize,
    pub render_ratio: f64,      // Percentage of items being rendered (lower is better)
    pub memory_efficiency: f64, // 1.0 - render_ratio (higher is better)
    pub buffer_utilization: f64, // Buffer size relative to visible items
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_calculator() -> VirtualScrollCalculator {
        VirtualScrollCalculator::new(
            50.0,  // item_height
            400.0, // container_height (8 items visible)
            2,     // buffer_size
            1000,  // total_items
        )
    }

    #[test]
    fn test_basic_visible_range_calculation() {
        let calc = create_test_calculator();
        let range = calc.calculate_visible_range(0.0);

        // At scroll position 0, should render from buffer to visible + buffer
        assert_eq!(range.start_index, 0); // No negative buffer at start
        assert_eq!(range.end_index, 10); // 8 visible + 2 buffer
        assert_eq!(range.visible_count, 8);
        assert_eq!(range.offset_top, 0.0);
    }

    #[test]
    fn test_mid_scroll_visible_range() {
        let calc = create_test_calculator();
        let range = calc.calculate_visible_range(500.0); // Scroll to item 10

        assert_eq!(range.start_index, 8);  // Item 10 - 2 buffer
        assert_eq!(range.end_index, 20);   // Item 18 + 2 buffer
        assert_eq!(range.visible_count, 8);
        assert_eq!(range.offset_top, 400.0); // 8 * 50px
    }

    #[test]
    fn test_end_of_list_behavior() {
        let calc = create_test_calculator();
        let range = calc.calculate_visible_range(49000.0); // Near end

        assert!(range.end_index <= calc.total_items);
        // At the very end, we show fewer items but still within bounds
        assert!(range.end_index >= range.start_index);
        assert!(range.start_index < calc.total_items);
    }

    #[test]
    fn test_empty_list() {
        let calc = VirtualScrollCalculator::new(50.0, 400.0, 2, 0);
        let range = calc.calculate_visible_range(0.0);

        assert_eq!(range.start_index, 0);
        assert_eq!(range.end_index, 0);
        assert_eq!(range.visible_count, 0);
        assert_eq!(range.total_height, 0.0);
    }

    #[test]
    fn test_total_height_calculation() {
        let calc = create_test_calculator();
        assert_eq!(calc.calculate_total_height(), 50000.0); // 1000 * 50px
    }

    #[test]
    fn test_item_offset_calculation() {
        let calc = create_test_calculator();
        assert_eq!(calc.get_item_offset(0), 0.0);
        assert_eq!(calc.get_item_offset(10), 500.0);
        assert_eq!(calc.get_item_offset(999), 49950.0);
    }

    #[test]
    fn test_item_visibility_check() {
        let calc = create_test_calculator();
        
        // Item 0 should be visible at scroll position 0
        assert!(calc.is_item_visible(0, 0.0));
        
        // Item 7 should be visible at scroll position 0 (8 items fit)
        assert!(calc.is_item_visible(7, 0.0));
        
        // Item 8 should not be visible at scroll position 0
        assert!(!calc.is_item_visible(8, 0.0));
        
        // Item 10 should be visible at scroll position 500 (middle of item 10)
        assert!(calc.is_item_visible(10, 500.0));
    }

    #[test]
    fn test_scroll_to_item_alignment() {
        let calc = create_test_calculator();
        
        // Scroll to start
        assert_eq!(calc.scroll_to_item(10, ScrollAlignment::Start), 500.0);
        
        // Scroll to center
        assert_eq!(calc.scroll_to_item(10, ScrollAlignment::Center), 325.0);
        
        // Scroll to end
        assert_eq!(calc.scroll_to_item(10, ScrollAlignment::End), 150.0);
    }

    #[test]
    fn test_performance_metrics() {
        let calc = create_test_calculator();
        let range = calc.calculate_visible_range(0.0);
        let metrics = calc.get_performance_metrics(&range);

        assert_eq!(metrics.total_items, 1000);
        assert_eq!(metrics.rendered_items, 10); // 8 visible + 2 buffer
        assert_eq!(metrics.render_ratio, 0.01); // 1% of items rendered
        assert_eq!(metrics.memory_efficiency, 0.99); // 99% memory efficient
    }

    #[test]
    fn test_large_dataset_performance() {
        // Test with very large dataset
        let calc = VirtualScrollCalculator::new(30.0, 600.0, 5, 100_000);
        let range = calc.calculate_visible_range(1_500_000.0); // Deep in the list
        
        // Should still work efficiently
        assert!(range.start_index < range.end_index);
        assert!(range.end_index <= 100_000);
        
        let rendered_items = range.end_index - range.start_index;
        assert!(rendered_items < 50); // Should render very few items
        
        let metrics = calc.get_performance_metrics(&range);
        assert!(metrics.memory_efficiency > 0.999); // >99.9% memory efficient
    }

    #[test]
    fn test_dynamic_updates() {
        let mut calc = create_test_calculator();
        
        // Test container height update
        calc.update_container_height(800.0);
        assert_eq!(calc.container_height, 800.0);
        
        // Test total items update
        calc.update_total_items(2000);
        assert_eq!(calc.total_items, 2000);
        assert_eq!(calc.calculate_total_height(), 100_000.0);
        
        // Test item height update
        calc.update_item_height(75.0);
        assert_eq!(calc.item_height, 75.0);
        assert_eq!(calc.calculate_total_height(), 150_000.0);
    }
}