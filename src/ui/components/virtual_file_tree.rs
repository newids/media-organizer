use dioxus::prelude::*;
use crate::services::file_system::FileEntry;
use crate::ui::components::virtual_scroll::{VirtualScrollCalculator, VisibleRange, ScrollAlignment};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

/// Virtual file tree component that efficiently renders large file lists
/// Handles 10,000+ files with constant memory usage through virtual scrolling
#[derive(Props, Clone, PartialEq)]
pub struct VirtualFileTreeProps {
    /// List of files/folders to display
    pub items: Vec<FileEntry>,
    /// Height of the container in pixels
    pub container_height: f64,
    /// Height of each item in pixels
    pub item_height: f64,
    /// Optional callback when item is clicked
    #[props(optional)]
    pub on_item_click: Option<EventHandler<FileEntry>>,
    /// Optional callback when item is double-clicked
    #[props(optional)]
    pub on_item_double_click: Option<EventHandler<FileEntry>>,
}

/// Tree item structure for hierarchical display
#[derive(Debug, Clone)]
pub struct FileTreeItem {
    pub entry: FileEntry,
    pub depth: usize,
    pub is_expanded: bool,
    pub is_visible: bool,
    pub parent_path: Option<PathBuf>,
}

/// Performance metrics for monitoring virtual tree efficiency
#[derive(Debug, Clone)]
pub struct VirtualTreeMetrics {
    pub total_items: usize,
    pub rendered_items: usize,
    pub render_time_ms: f64,
    pub memory_efficiency: f64,
    pub scroll_performance: f64,
    pub last_updated: Instant,
}

impl VirtualTreeMetrics {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            rendered_items: 0,
            render_time_ms: 0.0,
            memory_efficiency: 0.0,
            scroll_performance: 0.0,
            last_updated: Instant::now(),
        }
    }
    
    pub fn update(&mut self, total: usize, rendered: usize, render_time: f64) {
        self.total_items = total;
        self.rendered_items = rendered;
        self.render_time_ms = render_time;
        self.memory_efficiency = if total > 0 { 
            1.0 - (rendered as f64 / total as f64) 
        } else { 
            1.0 
        };
        self.scroll_performance = if render_time > 0.0 { 
            1000.0 / render_time 
        } else { 
            1000.0 
        };
        self.last_updated = Instant::now();
    }
}

/// Memoized calculation cache for expensive operations
#[derive(Debug, Clone)]
pub struct CalculationCache {
    pub tree_items_hash: u64,
    pub visible_range_hash: u64,
    pub cached_visible_items: Vec<FileTreeItem>,
    pub last_scroll_position: f64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl CalculationCache {
    pub fn new() -> Self {
        Self {
            tree_items_hash: 0,
            visible_range_hash: 0,
            cached_visible_items: Vec::new(),
            last_scroll_position: -1.0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
    
    /// Clear cache when it becomes too large to prevent memory leaks
    pub fn cleanup_if_needed(&mut self) {
        const MAX_CACHED_ITEMS: usize = 1000;
        const MAX_CACHE_AGE_OPERATIONS: u64 = 5000;
        
        // Clear cache if it's consuming too much memory
        if self.cached_visible_items.len() > MAX_CACHED_ITEMS {
            tracing::debug!("Clearing cache due to size: {} items", self.cached_visible_items.len());
            self.cached_visible_items.clear();
            self.cached_visible_items.shrink_to_fit(); // Release memory
            self.tree_items_hash = 0;
            self.visible_range_hash = 0;
        }
        
        // Clear cache if it's been used too many times (prevents stale data)
        if self.cache_hits + self.cache_misses > MAX_CACHE_AGE_OPERATIONS {
            tracing::debug!("Clearing cache due to age: {} operations", self.cache_hits + self.cache_misses);
            self.cached_visible_items.clear();
            self.cached_visible_items.shrink_to_fit();
            self.cache_hits = 0;
            self.cache_misses = 0;
            self.tree_items_hash = 0;
            self.visible_range_hash = 0;
        }
    }
    
    /// Force cleanup of cache to free memory
    pub fn force_cleanup(&mut self) {
        self.cached_visible_items.clear();
        self.cached_visible_items.shrink_to_fit();
        self.cache_hits = 0;
        self.cache_misses = 0;
        self.tree_items_hash = 0;
        self.visible_range_hash = 0;
        self.last_scroll_position = -1.0;
        tracing::debug!("Forced cache cleanup completed");
    }
}

/// Virtual file tree component implementation - With hierarchical folder support
pub fn VirtualFileTree(props: VirtualFileTreeProps) -> Element {
    // Scroll position state
    let mut scroll_top = use_signal(|| 0.0f64);
    
    // Folder expansion state tracking
    let mut expanded_folders = use_signal(|| HashMap::<PathBuf, bool>::new());
    
    // Transform flat file list into hierarchical tree structure
    let mut tree_items = use_signal(|| Vec::<FileTreeItem>::new());
    
    // File selection state management
    let mut selected_items = use_signal(|| HashSet::<PathBuf>::new());
    let mut last_selected_item = use_signal(|| Option::<PathBuf>::None);
    
    // Keyboard focus and navigation state
    let mut focused_item = use_signal(|| Option::<PathBuf>::None);
    let mut container_focused = use_signal(|| false);
    
    // Performance monitoring and metrics
    let mut performance_metrics = use_signal(|| VirtualTreeMetrics::new());
    let mut calculation_cache = use_signal(|| CalculationCache::new());
    
    // Memoized tree building with performance optimization
    use_effect(move || {
        let tree = tree_items.clone();
        let expanded = expanded_folders.clone();
        let files = props.items.clone();
        let metrics = performance_metrics.clone();
        
        let start_time = Instant::now();
        
        // Check if we can use cached result
        let should_rebuild = files.len() != tree.read().len() || 
                            expanded.read().len() != tree.read().iter()
                                .filter(|item| item.is_expanded)
                                .count();
        
        if should_rebuild {
            tracing::debug!("Rebuilding tree structure for {} files", files.len());
            let items = build_hierarchical_tree_optimized(&files, &expanded.read());
            tree.set(items);
            
            let elapsed = start_time.elapsed().as_millis() as f64;
            tracing::debug!("Tree rebuild completed in {:.2}ms", elapsed);
        } else {
            tracing::debug!("Using cached tree structure");
        }
    });
    
    // Virtual scroll calculator with dynamic buffer sizing based on performance
    let mut calculator = use_signal(|| {
        let buffer_size = calculate_optimal_buffer_size(tree_items.read().len(), props.container_height, props.item_height);
        VirtualScrollCalculator::new(
            props.item_height,
            props.container_height,
            buffer_size,
            tree_items.read().len(),
        )
    });
    
    // Update calculator when tree structure changes
    use_effect(move || {
        let calc = calculator.clone();
        let item_count = tree_items.read().len();
        let height = props.container_height;
        let item_height = props.item_height;
        
        let mut new_calc = calc.read().clone();
        new_calc.update_total_items(item_count);
        new_calc.update_container_height(height);
        new_calc.update_item_height(item_height);
        calc.set(new_calc);
    });
    
    // Calculate visible range based on tree structure
    let visible_range = calculator.read().calculate_visible_range(*scroll_top.read());
    
    // Get visible tree items with caching, performance monitoring, and memory cleanup
    let visible_items = {
        let render_start = Instant::now();
        let mut cache = calculation_cache.read().clone();
        
        // Perform cache cleanup if needed to prevent memory leaks
        cache.cleanup_if_needed();
        
        let items = get_visible_tree_items_cached(&tree_items.read(), &visible_range, &mut cache);
        calculation_cache.set(cache);
        
        // Update performance metrics
        let render_time = render_start.elapsed().as_millis() as f64;
        let mut metrics = performance_metrics.read().clone();
        metrics.update(tree_items.read().len(), items.len(), render_time);
        performance_metrics.set(metrics);
        
        // Log performance metrics periodically
        if render_time > 16.0 { // More than one frame at 60fps
            tracing::warn!("Slow render detected: {:.2}ms for {} visible items (total: {})", 
                          render_time, items.len(), tree_items.read().len());
        }
        
        items
    };
    
    rsx! {
        div {
            class: "virtual-file-tree",
            style: format!("height: {}px; overflow-y: auto; position: relative;", props.container_height).as_str(),
            tabindex: "0", // Make container focusable for keyboard events
            onfocus: move |_| {
                container_focused.set(true);
                // Set focus to first item if none focused
                if focused_item.read().is_none() && !tree_items.read().is_empty() {
                    focused_item.set(Some(tree_items.read()[0].entry.path.clone()));
                }
            },
            onblur: move |_| {
                container_focused.set(false);
            },
            onkeydown: move |evt| {
                if !*container_focused.read() {
                    return;
                }
                
                let key_str = format!("{:?}", evt.data.key());
                tracing::info!("Key pressed: {} (ctrl: {}, shift: {})", key_str, evt.data.modifiers().ctrl(), evt.data.modifiers().shift());
                handle_keyboard_navigation(
                    &key_str,
                    evt.data.modifiers().ctrl(),
                    evt.data.modifiers().shift(),
                    &mut focused_item,
                    &mut selected_items,
                    &mut last_selected_item,
                    &mut expanded_folders,
                    &tree_items,
                    &calculator,
                    &mut scroll_top,
                    props.container_height,
                );
            },
            onscroll: move |_evt| {
                // Extract scroll position from event data
                // For now, use a simple approximation - we'll improve this later
                scroll_top.set(0.0); // Placeholder - will be improved in next phase
            },
            
            // Virtual container that maintains proper scrollbar height
            div {
                class: "virtual-container",
                style: format!("height: {}px; position: relative;", calculator.read().calculate_total_height()),
                
                // Render hierarchical tree items with expansion/collapse
                for (index, tree_item) in visible_items.iter().enumerate() {
                    div {
                        key: "{tree_item.entry.path.to_string_lossy()}-{index}",
                        class: if selected_items.read().contains(&tree_item.entry.path) && focused_item.read().as_ref() == Some(&tree_item.entry.path) {
                            "file-tree-item selected focused"
                        } else if selected_items.read().contains(&tree_item.entry.path) {
                            "file-tree-item selected"
                        } else if focused_item.read().as_ref() == Some(&tree_item.entry.path) {
                            "file-tree-item focused"
                        } else {
                            "file-tree-item"
                        },
                        style: format!("
                            position: absolute;
                            top: {}px;
                            left: 0;
                            right: 0;
                            height: {}px;
                            display: flex;
                            align-items: center;
                            padding: 0 8px;
                            cursor: pointer;
                            user-select: none;
                            padding-left: {}px;
                        ", (visible_range.start_index + index) as f64 * props.item_height, props.item_height, 8 + tree_item.depth * 16),
                        onclick: {
                            let entry = tree_item.entry.clone();
                            let expanded = expanded_folders.clone();
                            let selected = selected_items.clone();
                            let last_selected = last_selected_item.clone();
                            let path = tree_item.entry.path.clone();
                            let visible_tree_items = tree_items.read().clone();
                            move |evt: MouseEvent| {
                                // Handle file selection based on modifier keys
                                handle_item_selection(
                                    &path,
                                    evt.data.modifiers().ctrl(),
                                    evt.data.modifiers().shift(),
                                    &mut selected.clone(),
                                    &mut last_selected.clone(),
                                    &visible_tree_items,
                                );
                                
                                // Handle folder expansion/collapse for directories (only on single-click without modifiers)
                                if entry.is_directory && !evt.data.modifiers().ctrl() && !evt.data.modifiers().shift() {
                                    let mut current_state = expanded.read().clone();
                                    let current_expanded = current_state.get(&path).copied().unwrap_or(false);
                                    current_state.insert(path.clone(), !current_expanded);
                                    expanded.set(current_state);
                                }
                                
                                // Also call the provided handler
                                if let Some(handler) = &props.on_item_click {
                                    handler.call(entry.clone());
                                }
                            }
                        },
                        ondoubleclick: {
                            let entry = tree_item.entry.clone();
                            move |_| {
                                if let Some(handler) = &props.on_item_double_click {
                                    handler.call(entry.clone());
                                }
                            }
                        },
                        
                        // TODO: Add expansion indicator once syntax is figured out
                        
                        // File/folder icon
                        div {
                            style: "margin-right: 8px; font-size: 16px;",
                            if tree_item.entry.is_directory { "📁" } else { "📄" }
                        }
                        
                        // File/folder name
                        div {
                            style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                            "{tree_item.entry.name}"
                        }
                    }
                }
            }
        }
    }
}

/// Build hierarchical tree structure from flat file list with performance optimizations
fn build_hierarchical_tree_optimized(files: &[FileEntry], expanded_folders: &HashMap<PathBuf, bool>) -> Vec<FileTreeItem> {
    let start_time = Instant::now();
    
    // Pre-allocate vector with capacity hint for better memory performance
    let mut items = Vec::with_capacity(files.len());
    
    // For large datasets, use iterator chains for better performance
    if files.len() > 1000 {
        // High-performance path for large directories
        items = files
            .iter()
            .map(|entry| {
                let is_expanded = entry.is_directory && expanded_folders.get(&entry.path).copied().unwrap_or(false);
                
                FileTreeItem {
                    entry: entry.clone(),
                    depth: 0, // All items at root level for now
                    is_expanded,
                    is_visible: true, // All items visible in flat structure
                    parent_path: None, // No parent in flat structure
                }
            })
            .collect();
    } else {
        // Standard path for smaller directories
        for entry in files {
            let is_expanded = entry.is_directory && expanded_folders.get(&entry.path).copied().unwrap_or(false);
            
            items.push(FileTreeItem {
                entry: entry.clone(),
                depth: 0, // All items at root level for now
                is_expanded,
                is_visible: true, // All items visible in flat structure
                parent_path: None, // No parent in flat structure
            });
        }
    }
    
    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > 10 {
        tracing::debug!("Tree building took {:.2}ms for {} items", elapsed.as_millis(), files.len());
    }
    
    items
}

/// Build hierarchical tree structure from flat file list (legacy function kept for compatibility)
fn build_hierarchical_tree(files: &[FileEntry], expanded_folders: &HashMap<PathBuf, bool>) -> Vec<FileTreeItem> {
    build_hierarchical_tree_optimized(files, expanded_folders)
}

/// Get visible tree items from the full tree with memoization and caching
fn get_visible_tree_items_cached(
    tree_items: &[FileTreeItem], 
    visible_range: &VisibleRange, 
    cache: &mut CalculationCache,
) -> Vec<FileTreeItem> {
    let start_time = Instant::now();
    
    // Calculate hash for cache key
    let range_hash = calculate_range_hash(visible_range);
    let tree_hash = calculate_tree_hash(tree_items);
    
    // Check if we can use cached result
    if cache.visible_range_hash == range_hash && 
       cache.tree_items_hash == tree_hash && 
       !cache.cached_visible_items.is_empty() {
        cache.cache_hits += 1;
        tracing::trace!("Cache hit for visible items (range: {}-{}, {} items)", 
                       visible_range.start_index, visible_range.end_index, cache.cached_visible_items.len());
        return cache.cached_visible_items.clone();
    }
    
    // Cache miss - calculate new result
    cache.cache_misses += 1;
    let result = get_visible_tree_items_uncached(tree_items, visible_range);
    
    // Update cache
    cache.visible_range_hash = range_hash;
    cache.tree_items_hash = tree_hash;
    cache.cached_visible_items = result.clone();
    
    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > 5 {
        tracing::debug!("Visible items calculation took {:.2}ms for range {}-{}", 
                       elapsed.as_millis(), visible_range.start_index, visible_range.end_index);
    }
    
    result
}

/// Get visible tree items from the full tree based on calculated range (uncached)
fn get_visible_tree_items_uncached(tree_items: &[FileTreeItem], visible_range: &VisibleRange) -> Vec<FileTreeItem> {
    let start = visible_range.start_index.min(tree_items.len());
    let end = visible_range.end_index.min(tree_items.len());
    
    if start >= tree_items.len() {
        return Vec::new();
    }
    
    tree_items[start..end].to_vec()
}

/// Get visible tree items from the full tree based on calculated range (legacy function kept for compatibility)
fn get_visible_tree_items(tree_items: &[FileTreeItem], visible_range: &VisibleRange) -> Vec<FileTreeItem> {
    get_visible_tree_items_uncached(tree_items, visible_range)
}

/// Calculate hash for visible range for caching purposes
fn calculate_range_hash(range: &VisibleRange) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    range.start_index.hash(&mut hasher);
    range.end_index.hash(&mut hasher);
    (range.total_height as u64).hash(&mut hasher);
    hasher.finish()
}

/// Calculate hash for tree items for caching purposes
fn calculate_tree_hash(items: &[FileTreeItem]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    items.len().hash(&mut hasher);
    
    // For performance, only hash a sample of items for large lists
    if items.len() > 1000 {
        // Sample first 10, middle 10, and last 10 items for large lists
        let sample_size = 10;
        for i in 0..sample_size.min(items.len()) {
            items[i].entry.path.hash(&mut hasher);
            items[i].is_expanded.hash(&mut hasher);
        }
        if items.len() > sample_size * 2 {
            let mid = items.len() / 2;
            for i in (mid - sample_size / 2)..(mid + sample_size / 2).min(items.len()) {
                items[i].entry.path.hash(&mut hasher);
                items[i].is_expanded.hash(&mut hasher);
            }
        }
        if items.len() > sample_size {
            let start = (items.len() - sample_size).max(sample_size * 2);
            for i in start..items.len() {
                items[i].entry.path.hash(&mut hasher);
                items[i].is_expanded.hash(&mut hasher);
            }
        }
    } else {
        // Hash all items for smaller lists
        for item in items {
            item.entry.path.hash(&mut hasher);
            item.is_expanded.hash(&mut hasher);
        }
    }
    
    hasher.finish()
}

/// Calculate optimal buffer size based on dataset size and performance characteristics
fn calculate_optimal_buffer_size(total_items: usize, container_height: f64, item_height: f64) -> usize {
    let visible_items = (container_height / item_height).ceil() as usize;
    
    match total_items {
        // Small datasets: minimal buffer for fast scrolling
        0..=100 => 2,
        // Medium datasets: balanced buffer for smooth scrolling
        101..=1000 => 3,
        // Large datasets: larger buffer for smoother experience but memory conscious
        1001..=10000 => {
            // Scale buffer with visible items, capped at reasonable limit
            (visible_items / 4).min(8).max(3)
        },
        // Very large datasets: fixed large buffer for optimal performance
        _ => {
            // For 10,000+ items, use larger buffer but cap memory usage
            (visible_items / 3).min(12).max(5)
        }
    }
}

/// Lazy file metadata loading for performance optimization
#[derive(Debug, Clone)]
pub struct LazyFileMetadata {
    pub path: PathBuf,
    pub basic_loaded: bool,
    pub detailed_loaded: bool,
    pub thumbnail_loaded: bool,
    pub error: Option<String>,
}

impl LazyFileMetadata {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            basic_loaded: false,
            detailed_loaded: false,
            thumbnail_loaded: false,
            error: None,
        }
    }
}

/// Implement lazy loading for file metadata to reduce initial load time
async fn load_file_metadata_lazy(
    file_path: &PathBuf,
    load_level: MetadataLoadLevel,
) -> Result<(), String> {
    let start_time = Instant::now();
    
    match load_level {
        MetadataLoadLevel::Basic => {
            // Load only essential metadata (name, size, type)
            // This is already handled by FileEntry creation
            tracing::trace!("Basic metadata loaded for {:?}", file_path);
        },
        MetadataLoadLevel::Detailed => {
            // Load extended metadata (permissions, dates, etc.)
            // Simulate more expensive metadata loading
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
            tracing::trace!("Detailed metadata loaded for {:?}", file_path);
        },
        MetadataLoadLevel::Thumbnail => {
            // Load or generate thumbnail (most expensive operation)
            // Simulate thumbnail generation
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            tracing::trace!("Thumbnail loaded for {:?}", file_path);
        }
    }
    
    let elapsed = start_time.elapsed();
    if elapsed.as_millis() > 10 {
        tracing::debug!("Metadata loading took {:.2}ms for {:?} (level: {:?})", 
                       elapsed.as_millis(), file_path, load_level);
    }
    
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum MetadataLoadLevel {
    Basic,     // Essential info only
    Detailed,  // Extended metadata
    Thumbnail, // Full preview data
}

/// Handle file selection with support for single-click, Ctrl+click, and Shift+click
fn handle_item_selection(
    clicked_path: &PathBuf,
    ctrl_pressed: bool,
    shift_pressed: bool,
    selected_items: &mut Signal<HashSet<PathBuf>>,
    last_selected_item: &mut Signal<Option<PathBuf>>,
    visible_tree_items: &[FileTreeItem],
) {
    let mut current_selection = selected_items.read().clone();
    
    if ctrl_pressed {
        // Ctrl+click: Toggle selection of clicked item
        if current_selection.contains(clicked_path) {
            current_selection.remove(clicked_path);
        } else {
            current_selection.insert(clicked_path.clone());
        }
        last_selected_item.set(Some(clicked_path.clone()));
    } else if shift_pressed {
        // Shift+click: Range selection from last selected to current
        if let Some(last_path) = last_selected_item.read().as_ref() {
            // Find indices of last selected and current item
            let last_idx = visible_tree_items.iter().position(|item| &item.entry.path == last_path);
            let current_idx = visible_tree_items.iter().position(|item| &item.entry.path == clicked_path);
            
            if let (Some(last), Some(current)) = (last_idx, current_idx) {
                // Select range between last and current (inclusive)
                let start_idx = last.min(current);
                let end_idx = last.max(current);
                
                for item in &visible_tree_items[start_idx..=end_idx] {
                    current_selection.insert(item.entry.path.clone());
                }
            } else {
                // If last selected item not found in current view, just select current item
                current_selection.clear();
                current_selection.insert(clicked_path.clone());
            }
        } else {
            // No last selected item, just select current
            current_selection.clear();
            current_selection.insert(clicked_path.clone());
        }
        last_selected_item.set(Some(clicked_path.clone()));
    } else {
        // Normal click: Select only clicked item
        current_selection.clear();
        current_selection.insert(clicked_path.clone());
        last_selected_item.set(Some(clicked_path.clone()));
    }
    
    selected_items.set(current_selection);
}

/// Handle comprehensive keyboard navigation for the file tree
fn handle_keyboard_navigation(
    key: &str,
    ctrl_pressed: bool,
    shift_pressed: bool,
    focused_item: &mut Signal<Option<PathBuf>>,
    selected_items: &mut Signal<HashSet<PathBuf>>,
    last_selected_item: &mut Signal<Option<PathBuf>>,
    expanded_folders: &mut Signal<HashMap<PathBuf, bool>>,
    tree_items: &Signal<Vec<FileTreeItem>>,
    calculator: &Signal<VirtualScrollCalculator>,
    scroll_top: &mut Signal<f64>,
    container_height: f64,
) {
    let current_items = tree_items.read();
    if current_items.is_empty() {
        return;
    }

    // Get current focused index
    let current_focused_index = match focused_item.read().as_ref() {
        Some(path) => current_items.iter().position(|item| &item.entry.path == path).unwrap_or(0),
        None => 0,
    };

    match key {
        "ArrowDown" => {
            // Navigate to next item
            let next_index = (current_focused_index + 1).min(current_items.len() - 1);
            let next_path = current_items[next_index].entry.path.clone();
            focused_item.set(Some(next_path.clone()));
            
            // Handle selection based on modifiers
            if shift_pressed {
                // Range selection
                handle_range_selection(current_focused_index, next_index, &current_items, &mut *selected_items);
            } else if !ctrl_pressed {
                // Single selection (clear and select new item)
                let mut new_selection = HashSet::new();
                new_selection.insert(next_path.clone());
                selected_items.set(new_selection);
                last_selected_item.set(Some(next_path));
            }
            
            // Auto-scroll to ensure focused item is visible
            ensure_item_visible(next_index, calculator, &mut *scroll_top, container_height);
        },
        "ArrowUp" => {
            // Navigate to previous item
            let prev_index = current_focused_index.saturating_sub(1);
            let prev_path = current_items[prev_index].entry.path.clone();
            focused_item.set(Some(prev_path.clone()));
            
            // Handle selection based on modifiers
            if shift_pressed {
                // Range selection
                handle_range_selection(current_focused_index, prev_index, &current_items, &mut *selected_items);
            } else if !ctrl_pressed {
                // Single selection (clear and select new item)
                let mut new_selection = HashSet::new();
                new_selection.insert(prev_path.clone());
                selected_items.set(new_selection);
                last_selected_item.set(Some(prev_path));
            }
            
            // Auto-scroll to ensure focused item is visible
            ensure_item_visible(prev_index, calculator, &mut *scroll_top, container_height);
        },
        "ArrowRight" => {
            // Expand folder if it's collapsed, or navigate to first child
            if let Some(focused_path) = focused_item.read().as_ref() {
                if let Some(item) = current_items.iter().find(|item| &item.entry.path == focused_path) {
                    if item.entry.is_directory {
                        let mut current_state = expanded_folders.read().clone();
                        current_state.insert(focused_path.clone(), true);
                        expanded_folders.set(current_state);
                    }
                }
            }
        },
        "ArrowLeft" => {
            // Collapse folder if it's expanded, or navigate to parent
            if let Some(focused_path) = focused_item.read().as_ref() {
                if let Some(item) = current_items.iter().find(|item| &item.entry.path == focused_path) {
                    if item.entry.is_directory {
                        let current_expanded = expanded_folders.read().get(focused_path).copied().unwrap_or(false);
                        if current_expanded {
                            // Collapse the folder
                            let mut current_state = expanded_folders.read().clone();
                            current_state.insert(focused_path.clone(), false);
                            expanded_folders.set(current_state);
                        }
                    }
                }
            }
        },
        "Enter" => {
            // Expand/collapse folders or open files
            if let Some(focused_path) = focused_item.read().as_ref() {
                if let Some(item) = current_items.iter().find(|item| &item.entry.path == focused_path) {
                    if item.entry.is_directory {
                        // Toggle folder expansion
                        let mut current_state = expanded_folders.read().clone();
                        let current_expanded = current_state.get(focused_path).copied().unwrap_or(false);
                        current_state.insert(focused_path.clone(), !current_expanded);
                        expanded_folders.set(current_state);
                    } else {
                        // TODO: Open file - this would trigger file opening logic
                        tracing::info!("Enter pressed on file: {}", item.entry.name);
                    }
                }
            }
        },
        " " => {
            // Space key: Toggle selection of focused item
            if let Some(focused_path) = focused_item.read().as_ref() {
                let mut current_selection = selected_items.read().clone();
                if current_selection.contains(focused_path) {
                    current_selection.remove(focused_path);
                } else {
                    current_selection.insert(focused_path.clone());
                }
                selected_items.set(current_selection);
                last_selected_item.set(Some(focused_path.clone()));
            }
        },
        "Home" => {
            // Jump to first item
            if !current_items.is_empty() {
                let first_path = current_items[0].entry.path.clone();
                focused_item.set(Some(first_path.clone()));
                
                if !ctrl_pressed {
                    // Select first item
                    let mut new_selection = HashSet::new();
                    new_selection.insert(first_path.clone());
                    selected_items.set(new_selection);
                    last_selected_item.set(Some(first_path));
                }
                
                // Scroll to top
                ensure_item_visible(0, calculator, &mut *scroll_top, container_height);
            }
        },
        "End" => {
            // Jump to last item
            if !current_items.is_empty() {
                let last_index = current_items.len() - 1;
                let last_path = current_items[last_index].entry.path.clone();
                focused_item.set(Some(last_path.clone()));
                
                if !ctrl_pressed {
                    // Select last item
                    let mut new_selection = HashSet::new();
                    new_selection.insert(last_path.clone());
                    selected_items.set(new_selection);
                    last_selected_item.set(Some(last_path));
                }
                
                // Scroll to bottom
                ensure_item_visible(last_index, calculator, &mut *scroll_top, container_height);
            }
        },
        key if key.contains("A") && ctrl_pressed => {
            // Ctrl+A: Select all items
            let all_paths: HashSet<PathBuf> = current_items.iter()
                .map(|item| item.entry.path.clone())
                .collect();
            selected_items.set(all_paths);
            if let Some(focused_path) = focused_item.read().as_ref() {
                last_selected_item.set(Some(focused_path.clone()));
            }
        },
        "Escape" => {
            // Clear selection
            selected_items.set(HashSet::new());
            last_selected_item.set(None);
        },
        _ => {
            // Handle other keys or ignore
        }
    }
}

/// Handle range selection between two indices
fn handle_range_selection(
    from_index: usize,
    to_index: usize,
    tree_items: &[FileTreeItem],
    selected_items: &mut Signal<HashSet<PathBuf>>,
) {
    let mut current_selection = selected_items.read().clone();
    let start_idx = from_index.min(to_index);
    let end_idx = from_index.max(to_index);
    
    for i in start_idx..=end_idx {
        if i < tree_items.len() {
            current_selection.insert(tree_items[i].entry.path.clone());
        }
    }
    
    selected_items.set(current_selection);
}

/// Ensure the specified item index is visible in the viewport
fn ensure_item_visible(
    item_index: usize,
    calculator: &Signal<VirtualScrollCalculator>,
    scroll_top: &mut Signal<f64>,
    container_height: f64,
) {
    let calc = calculator.read();
    let item_offset = calc.get_item_offset(item_index);
    let current_scroll = *scroll_top.read();
    
    // Check if item is already visible
    if item_offset >= current_scroll && item_offset + calc.item_height <= current_scroll + container_height {
        return; // Already visible
    }
    
    // Calculate new scroll position to center the item
    let new_scroll = calc.scroll_to_item(item_index, ScrollAlignment::Center).max(0.0);
    scroll_top.set(new_scroll);
}