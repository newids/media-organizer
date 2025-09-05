# Layout State Design - MediaOrganizer

## Overview

The `LayoutState` model provides a unified, centralized approach to managing all layout-related state in MediaOrganizer. This design consolidates previously scattered state components into a single, coherent structure optimized for performance and maintainability.

## Design Principles

### 1. Centralization
All layout-related state is managed through a single `LayoutState` struct, eliminating the need to synchronize multiple independent state components.

### 2. Separation of Concerns
The layout state is cleanly separated from business logic (file management, operations) and application data (file entries, search results).

### 3. Performance Optimization
- Structured for <100ms UI transitions
- Minimal state updates through targeted signal changes
- Optimized serialization for persistence

### 4. Extensibility
Designed to accommodate future layout features without breaking existing functionality.

## Architecture

```rust
LayoutState
├── theme: Theme                           // Visual theme (dark/light/auto)
├── activity_bar: ActivityBarLayout        // Left activity bar state
├── sidebar: SidebarLayout                 // Main sidebar configuration
├── editor_layout: EditorLayoutState       // Editor groups and tabs
├── panel: PanelLayout                     // Bottom panel state
├── viewport: ViewportState                // Window dimensions
├── ui_preferences: UIPreferences          // Animation and interaction settings
└── persistence: LayoutPersistenceSettings // State persistence configuration
```

## Component Details

### ActivityBarLayout
```rust
pub struct ActivityBarLayout {
    pub active_view: ActivityBarView,    // Explorer, Search, etc.
    pub is_visible: bool,                // Show/hide activity bar
    pub position: ActivityBarPosition,   // Left or Right
    pub width: f64,                      // Width in pixels
}
```

**Purpose**: Manages the VS Code-style activity bar that provides quick access to different application views.

**Key Features**:
- Supports both left and right positioning
- Configurable visibility for distraction-free modes
- Dynamic width adjustment

### SidebarLayout
```rust
pub struct SidebarLayout {
    pub is_collapsed: bool,              // Collapsed state
    pub width: f64,                      // Current width
    pub min_width: f64,                  // Minimum allowed width
    pub max_width: f64,                  // Maximum allowed width
    pub is_resizable: bool,              // Can be resized by user
    pub position: SidebarPosition,       // Left or Right
    pub active_content: SidebarContent,  // Current content type
}
```

**Purpose**: Controls the main sidebar that contains file trees, search results, and other navigational content.

**Key Features**:
- Collapsible for more screen space
- Configurable width constraints
- Multiple content types (FileTree, Search, Extensions, Settings)
- Position flexibility

### EditorLayoutState
```rust
pub struct EditorLayoutState {
    pub layout_config: EditorLayoutConfig,  // Single, Split, Grid
    pub active_group: usize,                // Currently active group
    pub split_ratios: Vec<f64>,             // Split proportions
    pub show_tabs: bool,                    // Tab visibility
    pub tab_height: f64,                    // Tab bar height
    pub max_visible_tabs: usize,            // Before scrolling
}
```

**Purpose**: Manages the central editor area with support for multiple groups and tab layouts.

**Key Features**:
- Multiple layout configurations (Single, SplitHorizontal, SplitVertical, Grid)
- Configurable split ratios for custom layouts
- Tab management with overflow handling
- Performance-optimized tab rendering

### PanelLayout
```rust
pub struct PanelLayout {
    pub is_visible: bool,                // Show/hide panel
    pub height: f64,                     // Current height
    pub min_height: f64,                 // Minimum height
    pub max_height_fraction: f64,        // Max as viewport fraction
    pub active_tab: PanelTab,            // Active panel tab
    pub is_resizable: bool,              // Resizable by user
    pub position: PanelPosition,         // Bottom or Top
}
```

**Purpose**: Controls the bottom/top panel for problems, output, terminal, and debug information.

**Key Features**:
- Flexible positioning (bottom or top)
- Height constraints with viewport-relative maximums
- Multiple tab support (Problems, Output, Terminal, Debug)
- Collapsible for focused work

### ViewportState
```rust
pub struct ViewportState {
    pub width: f64,                      // Window width
    pub height: f64,                     // Window height
    pub pixel_ratio: f64,                // Device pixel ratio
    pub is_maximized: bool,              // Window maximized state
    pub is_fullscreen: bool,             // Fullscreen mode
}
```

**Purpose**: Tracks window and display characteristics for responsive layout calculations.

**Key Features**:
- High-DPI display support
- Window state tracking
- Responsive design support

### UIPreferences
```rust
pub struct UIPreferences {
    pub enable_animations: bool,          // Enable UI animations
    pub animation_duration: u32,          // Animation duration (ms)
    pub smooth_scrolling: bool,           // Smooth scrolling
    pub visual_feedback: bool,            // Interaction feedback
    pub reduced_motion: bool,             // Accessibility setting
}
```

**Purpose**: Controls visual effects and interaction preferences.

**Key Features**:
- Accessibility support (reduced motion)
- Performance optimization options
- Customizable animation timing

### LayoutPersistenceSettings
```rust
pub struct LayoutPersistenceSettings {
    pub persist_layout: bool,             // Save layout state
    pub persist_theme: bool,              // Save theme preference
    pub restore_window_state: bool,       // Restore window size/position
    pub auto_save_interval: u32,          // Auto-save frequency (seconds)
}
```

**Purpose**: Configures how layout state is persisted between application sessions.

**Key Features**:
- Selective persistence options
- Configurable auto-save intervals
- Window state restoration

## State Management Integration

### Signal-Based Reactivity
The `LayoutState` integrates with Dioxus signals for reactive updates:

```rust
pub struct AppState {
    pub layout_state: Signal<LayoutState>,
    // ... other state
}
```

### Performance Considerations

1. **Targeted Updates**: Individual layout components can be updated without affecting others
2. **Batch Operations**: Multiple layout changes can be batched into single signal updates
3. **Lazy Evaluation**: Layout calculations are performed only when needed
4. **Caching**: Computed layout values are cached to avoid recalculation

### Migration Strategy

The new `LayoutState` coexists with existing state structures during migration:

```rust
// DEPRECATED: Use layout_state.sidebar instead
pub sidebar_state: Signal<SidebarState>,

// DEPRECATED: Use layout_state.panel instead  
pub panel_state: Signal<PanelState>,

// DEPRECATED: Use layout_state for layout-related settings
pub settings: Signal<SettingsState>,
```

This allows gradual migration without breaking existing functionality.

## Usage Examples

### Updating Theme
```rust
app_state.layout_state.write().theme = Theme::Dark;
```

### Toggling Sidebar
```rust
let current_collapsed = app_state.layout_state.read().sidebar.is_collapsed;
app_state.layout_state.write().sidebar.is_collapsed = !current_collapsed;
```

### Configuring Split Editor
```rust
app_state.layout_state.write().editor_layout.layout_config = EditorLayoutConfig::SplitVertical;
app_state.layout_state.write().editor_layout.split_ratios = vec![0.6, 0.4];
```

### Responsive Panel Height
```rust
let viewport_height = app_state.layout_state.read().viewport.height;
let max_panel_height = viewport_height * 0.4; // 40% of viewport
app_state.layout_state.write().panel.height = max_panel_height.min(300.0);
```

## Performance Targets

- **UI Transitions**: <100ms for layout changes
- **Theme Switches**: <50ms for theme transitions  
- **Memory Usage**: Minimal overhead compared to existing scattered state
- **Persistence**: <10ms for state serialization/deserialization

## Future Enhancements

1. **Layout Presets**: Save and restore complete layout configurations
2. **Multi-Monitor Support**: Per-monitor layout state
3. **Workspace Layouts**: Different layouts for different project types
4. **Layout Animation**: Smooth transitions between layout states
5. **Advanced Grid Layouts**: More complex editor group arrangements

## Testing Strategy

1. **Unit Tests**: Individual layout component state transitions
2. **Integration Tests**: Cross-component layout interactions
3. **Performance Tests**: Layout update timing and memory usage
4. **Accessibility Tests**: Reduced motion and high contrast compliance
5. **Persistence Tests**: State saving and restoration accuracy

## Conclusion

The unified `LayoutState` design provides a solid foundation for MediaOrganizer's VS Code-style interface while maintaining performance and extensibility. The modular structure allows for targeted updates and future enhancements without compromising existing functionality.