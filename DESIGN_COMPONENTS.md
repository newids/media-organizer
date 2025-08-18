# MediaOrganizer Component Design
*Based on PRD.md - VS Code UI Component System*

## 1. Component Architecture Overview

### 1.1 Design Principles
- **Composition over Inheritance**: Build complex UI from simple, reusable components
- **Single Responsibility**: Each component handles one specific UI concern
- **Props-Based Communication**: Data flows down via props, events flow up via handlers
- **State Locality**: State lives at the appropriate level in the component hierarchy
- **Accessibility First**: ARIA labels, keyboard navigation, and screen reader support built-in

### 1.2 Component Hierarchy

```
VSCodeLayout
├── ActivityBar
│   └── ActivityBarItem[]
├── Sidebar
│   ├── SidebarHeader
│   ├── FileTree
│   │   ├── TreeNode[]
│   │   └── ContextMenu
│   └── SidebarFooter
├── EditorGroups
│   └── EditorGroup[]
│       ├── TabBar
│       │   └── Tab[]
│       └── PreviewContainer
│           ├── ImagePreview
│           ├── TextPreview
│           ├── VideoPreview
│           ├── AudioPreview
│           ├── DocumentPreview
│           └── UnsupportedPreview
├── Panel
│   ├── PanelHeader
│   ├── PanelContent
│   │   ├── TerminalView
│   │   ├── ProblemsView
│   │   └── OutputView
│   └── PanelResizer
├── StatusBar
│   ├── StatusBarLeft
│   ├── StatusBarCenter
│   └── StatusBarRight
└── CommandPalette
    ├── CommandInput
    ├── CommandList
    └── CommandItem[]
```

## 2. Core Layout Components

### 2.1 VSCodeLayout Component

```rust
use dioxus::prelude::*;
use crate::state::{AppState, LayoutState, Theme};
use crate::services::layout::LayoutManager;

#[derive(Props, PartialEq)]
pub struct VSCodeLayoutProps {
    pub app_state: Signal<AppState>,
}

pub fn VSCodeLayout(props: VSCodeLayoutProps) -> Element {
    let layout_manager = use_context::<LayoutManager>();
    let layout_state = props.app_state.read().layout.read().clone();
    
    // Calculate layout dimensions based on current state
    let dimensions = layout_manager.calculate_layout(
        (layout_state.window_dimensions.0, layout_state.window_dimensions.1)
    );
    
    rsx! {
        div {
            id: "vscode-layout",
            class: "vscode-layout",
            style: format!(
                "width: {}px; height: {}px; display: grid; grid-template-areas: {grid_areas}; grid-template-columns: {columns}; grid-template-rows: {rows};",
                layout_state.window_dimensions.0,
                layout_state.window_dimensions.1,
                grid_areas = get_grid_areas(&layout_state),
                columns = get_grid_columns(&layout_state),
                rows = get_grid_rows(&layout_state)
            ),
            
            // Activity Bar
            if layout_state.activity_bar_visible {
                ActivityBar {
                    state: layout_state.activity_bar.clone(),
                    on_action: move |action| handle_activity_action(props.app_state, action)
                }
            }
            
            // Primary Sidebar
            if layout_state.sidebar_visible {
                Sidebar {
                    state: layout_state.sidebar.clone(),
                    width: layout_state.sidebar_width,
                    on_resize: move |width| handle_sidebar_resize(props.app_state, width),
                    on_file_select: move |path| handle_file_select(props.app_state, path)
                }
            }
            
            // Editor Groups
            EditorGroups {
                groups: layout_state.editor_groups.clone(),
                on_tab_change: move |group_id, tab_id| handle_tab_change(props.app_state, group_id, tab_id),
                on_tab_close: move |group_id, tab_id| handle_tab_close(props.app_state, group_id, tab_id),
                on_file_drop: move |files| handle_file_drop(props.app_state, files)
            }
            
            // Bottom Panel
            if layout_state.panel_visible {
                Panel {
                    state: layout_state.panel.clone(),
                    height: layout_state.panel_height,
                    on_resize: move |height| handle_panel_resize(props.app_state, height),
                    on_close: move |_| handle_panel_close(props.app_state)
                }
            }
            
            // Status Bar
            if layout_state.status_bar_visible {
                StatusBar {
                    state: layout_state.status_bar.clone()
                }
            }
            
            // Command Palette (overlay)
            if layout_state.command_palette_visible {
                CommandPalette {
                    visible: true,
                    on_command: move |command| handle_command(props.app_state, command),
                    on_close: move |_| handle_command_palette_close(props.app_state)
                }
            }
        }
    }
}

fn get_grid_areas(layout: &LayoutState) -> String {
    let activity = if layout.activity_bar_visible { "activity" } else { "." };
    let sidebar = if layout.sidebar_visible { "sidebar" } else { "." };
    let panel = if layout.panel_visible { "panel" } else { "editor" };
    let status = if layout.status_bar_visible { "status" } else { "." };
    
    format!(
        "'{activity} {sidebar} editor' '{activity} {sidebar} {panel}' '{status} {status} {status}'",
        activity = activity,
        sidebar = sidebar,
        panel = panel,
        status = status
    )
}

fn get_grid_columns(layout: &LayoutState) -> String {
    let activity_width = if layout.activity_bar_visible { "48px" } else { "0px" };
    let sidebar_width = if layout.sidebar_visible { 
        format!("{}px", layout.sidebar_width)
    } else { 
        "0px".to_string() 
    };
    
    format!("{} {} 1fr", activity_width, sidebar_width)
}

fn get_grid_rows(layout: &LayoutState) -> String {
    let panel_height = if layout.panel_visible { 
        format!("1fr {}px", layout.panel_height)
    } else { 
        "1fr 0px".to_string() 
    };
    let status_height = if layout.status_bar_visible { "22px" } else { "0px" };
    
    format!("{} {}", panel_height, status_height)
}
```

### 2.2 ActivityBar Component

```rust
#[derive(Props, PartialEq)]
pub struct ActivityBarProps {
    pub state: ActivityBarState,
    pub on_action: EventHandler<ActivityAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityAction {
    Select(ActivityType),
    Settings,
    Extensions,
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityType {
    Explorer,
    Search,
    SourceControl,
    RunDebug,
    Extensions,
}

pub fn ActivityBar(props: ActivityBarProps) -> Element {
    rsx! {
        div {
            class: "activity-bar",
            style: "grid-area: activity; background: var(--vscode-activity-bar-background); border-right: 1px solid var(--vscode-border);",
            role: "navigation",
            "aria-label": "Primary navigation",
            
            div {
                class: "activity-bar-items",
                style: "display: flex; flex-direction: column; height: 100%;",
                
                for activity in props.state.activities {
                    ActivityBarItem {
                        activity: activity.clone(),
                        active: props.state.active_activity == activity.activity_type,
                        on_click: move |_| props.on_action.call(ActivityAction::Select(activity.activity_type))
                    }
                }
                
                div {
                    class: "activity-bar-spacer",
                    style: "flex: 1;"
                }
                
                // Bottom items
                ActivityBarItem {
                    activity: ActivityItem::settings(),
                    active: false,
                    on_click: move |_| props.on_action.call(ActivityAction::Settings)
                }
            }
        }
    }
}

#[derive(Props, PartialEq)]
pub struct ActivityBarItemProps {
    pub activity: ActivityItem,
    pub active: bool,
    pub on_click: EventHandler<MouseEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivityItem {
    pub activity_type: ActivityType,
    pub icon: String,
    pub label: String,
    pub badge_count: Option<u32>,
}

impl ActivityItem {
    pub fn explorer() -> Self {
        Self {
            activity_type: ActivityType::Explorer,
            icon: "files".to_string(),
            label: "Explorer".to_string(),
            badge_count: None,
        }
    }
    
    pub fn search() -> Self {
        Self {
            activity_type: ActivityType::Search,
            icon: "search".to_string(),
            label: "Search".to_string(),
            badge_count: None,
        }
    }
    
    pub fn settings() -> Self {
        Self {
            activity_type: ActivityType::Extensions, // Using Extensions as placeholder
            icon: "settings-gear".to_string(),
            label: "Settings".to_string(),
            badge_count: None,
        }
    }
}

pub fn ActivityBarItem(props: ActivityBarItemProps) -> Element {
    let item_class = if props.active {
        "activity-bar-item active"
    } else {
        "activity-bar-item"
    };
    
    rsx! {
        button {
            class: item_class,
            style: format!(
                "width: 48px; height: 48px; border: none; background: {}; color: var(--vscode-foreground); display: flex; align-items: center; justify-content: center; position: relative; cursor: pointer;",
                if props.active { "var(--vscode-list-activeSelectionBackground)" } else { "transparent" }
            ),
            title: props.activity.label.clone(),
            "aria-label": props.activity.label.clone(),
            onclick: move |evt| props.on_click.call(evt),
            
            // Icon
            Icon {
                name: props.activity.icon.clone(),
                size: 16
            }
            
            // Badge
            if let Some(count) = props.activity.badge_count {
                div {
                    class: "activity-bar-badge",
                    style: "position: absolute; top: 4px; right: 4px; background: var(--vscode-badge-background); color: var(--vscode-badge-foreground); border-radius: 10px; min-width: 16px; height: 16px; font-size: 9px; display: flex; align-items: center; justify-content: center;",
                    "{count}"
                }
            }
        }
    }
}

### 2.3 Sidebar Component

```rust
#[derive(Props, PartialEq)]
pub struct SidebarProps {
    pub state: SidebarState,
    pub width: f64,
    pub on_resize: EventHandler<f64>,
    pub on_file_select: EventHandler<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SidebarState {
    pub title: String,
    pub content: SidebarContent,
    pub show_header: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SidebarContent {
    FileTree(FileTreeState),
    Search(SearchState),
    SourceControl(SourceControlState),
}

pub fn Sidebar(props: SidebarProps) -> Element {
    let is_resizing = use_signal(|| false);
    let resize_start_x = use_signal(|| 0.0);
    let resize_start_width = use_signal(|| props.width);
    
    rsx! {
        div {
            class: "sidebar",
            style: format!(
                "grid-area: sidebar; width: {}px; background: var(--vscode-sidebar-background); border-right: 1px solid var(--vscode-border); display: flex; flex-direction: column;",
                props.width
            ),
            
            // Sidebar Header
            if props.state.show_header {
                SidebarHeader {
                    title: props.state.title.clone()
                }
            }
            
            // Sidebar Content
            div {
                class: "sidebar-content",
                style: "flex: 1; overflow: hidden;",
                
                match props.state.content {
                    SidebarContent::FileTree(ref state) => rsx! {
                        FileTree {
                            state: state.clone(),
                            on_file_select: move |path| props.on_file_select.call(path)
                        }
                    },
                    SidebarContent::Search(ref state) => rsx! {
                        SearchPanel {
                            state: state.clone()
                        }
                    },
                    SidebarContent::SourceControl(ref state) => rsx! {
                        SourceControlPanel {
                            state: state.clone()
                        }
                    }
                }
            }
            
            // Resize Handle
            div {
                class: "sidebar-resize-handle",
                style: "position: absolute; right: -2px; top: 0; bottom: 0; width: 4px; cursor: col-resize; background: transparent;",
                onmousedown: move |evt| {
                    is_resizing.set(true);
                    resize_start_x.set(evt.client_x() as f64);
                    resize_start_width.set(props.width);
                    evt.prevent_default();
                },
                onmousemove: move |evt| {
                    if *is_resizing.read() {
                        let delta = evt.client_x() as f64 - *resize_start_x.read();
                        let new_width = (*resize_start_width.read() + delta).clamp(200.0, 400.0);
                        props.on_resize.call(new_width);
                    }
                },
                onmouseup: move |_| {
                    is_resizing.set(false);
                }
            }
        }
    }
}

#[derive(Props, PartialEq)]
pub struct SidebarHeaderProps {
    pub title: String,
}

pub fn SidebarHeader(props: SidebarHeaderProps) -> Element {
    rsx! {
        div {
            class: "sidebar-header",
            style: "height: 35px; padding: 0 16px; display: flex; align-items: center; border-bottom: 1px solid var(--vscode-border); font-size: 11px; font-weight: bold; color: var(--vscode-foreground); text-transform: uppercase;",
            "{props.title}"
        }
    }
}
```

## Content Viewer Components

```rust
#[derive(Props)]
pub struct ContentViewerProps {
    current_path: PathBuf,
    view_mode: ViewMode,
    sort_criteria: SortCriteria,
    selected_files: HashSet<PathBuf>,
    on_file_select: EventHandler<(PathBuf, bool)>, // (path, multi_select)
    on_file_action: EventHandler<FileAction>,
    on_view_mode_change: EventHandler<ViewMode>,
    on_sort_change: EventHandler<SortCriteria>,
}

pub fn ContentViewer(cx: Scope<ContentViewerProps>) -> Element {
    let files = use_state(cx, Vec::<FileEntry>::new);
    let loading_state = use_state(cx, LoadingState::NotStarted);
    let virtual_scroll = use_state(cx, || VirtualScrollState::new());
    
    // Services
    let services = use_shared_state::<Services>(cx).unwrap();
    
    // Load directory contents
    use_effect(cx, (&cx.props.current_path, &cx.props.sort_criteria), |(current_path, sort_criteria)| {
        to_owned![files, loading_state, services];
        async move {
            loading_state.set(LoadingState::Loading);
            
            match services.read().file_system.list_directory(current_path).await {
                Ok(mut entries) => {
                    // Sort entries
                    entries.sort_by(|a, b| sort_criteria.compare(a, b));
                    files.set(entries);
                    loading_state.set(LoadingState::Loaded);
                }
                Err(e) => {
                    loading_state.set(LoadingState::Error(e.to_string()));
                }
            }
        }
    });
    
    render! {
        div { class: "content-viewer",
            ToolBar {
                view_mode: cx.props.view_mode.clone(),
                sort_criteria: cx.props.sort_criteria.clone(),
                on_view_mode_change: move |mode| cx.props.on_view_mode_change.call(mode),
                on_sort_change: move |criteria| cx.props.on_sort_change.call(criteria),
            }
            
            match loading_state.get() {
                LoadingState::Loading => render! {
                    div { class: "loading-indicator", "Loading directory..." }
                },
                LoadingState::Error(error) => render! {
                    ErrorDisplay { error: error.clone() }
                },
                LoadingState::Loaded => {
                    match cx.props.view_mode {
                        ViewMode::Grid(ref config) => render! {
                            VirtualGridView {
                                files: files.get().clone(),
                                config: config.clone(),
                                selected_files: cx.props.selected_files.clone(),
                                on_file_select: move |(path, multi)| cx.props.on_file_select.call((path, multi)),
                                on_file_action: move |action| cx.props.on_file_action.call(action),
                            }
                        },
                        ViewMode::List(ref config) => render! {
                            VirtualListView {
                                files: files.get().clone(),
                                config: config.clone(),
                                selected_files: cx.props.selected_files.clone(),
                                on_file_select: move |(path, multi)| cx.props.on_file_select.call((path, multi)),
                                on_file_action: move |action| cx.props.on_file_action.call(action),
                            }
                        },
                        ViewMode::Preview(ref config) => render! {
                            PreviewView {
                                files: files.get().clone(),
                                config: config.clone(),
                                selected_files: cx.props.selected_files.clone(),
                                on_file_select: move |(path, multi)| cx.props.on_file_select.call((path, multi)),
                                on_file_action: move |action| cx.props.on_file_action.call(action),
                            }
                        }
                    }
                }
                _ => render! { div {} }
            }
        }
    }
}
```

## Virtual Grid Component

```rust
#[derive(Props)]
pub struct VirtualGridViewProps {
    files: Vec<FileEntry>,
    config: GridConfig,
    selected_files: HashSet<PathBuf>,
    on_file_select: EventHandler<(PathBuf, bool)>,
    on_file_action: EventHandler<FileAction>,
}

pub fn VirtualGridView(cx: Scope<VirtualGridViewProps>) -> Element {
    let container_ref = use_ref(cx, || None::<web_sys::Element>);
    let virtual_state = use_state(cx, || VirtualScrollState::new());
    let scroll_top = use_state(cx, || 0.0);
    
    // Calculate grid dimensions
    let item_width = cx.props.config.icon_size.width() + 20.0;
    let item_height = cx.props.config.icon_size.height() + 40.0;
    let columns = cx.props.config.columns.unwrap_or_else(|| {
        // Auto-calculate columns based on container width
        (800.0 / item_width).floor() as usize
    });
    
    // Virtual scrolling calculations
    let total_rows = (cx.props.files.len() + columns - 1) / columns;
    let visible_rows = (600.0 / item_height).ceil() as usize + 2; // +2 for buffer
    let start_row = (*scroll_top.get() / item_height).floor() as usize;
    let end_row = (start_row + visible_rows).min(total_rows);
    
    let visible_items: Vec<(usize, &FileEntry)> = cx.props.files
        .iter()
        .enumerate()
        .skip(start_row * columns)
        .take(visible_rows * columns)
        .collect();
    
    let total_height = total_rows as f32 * item_height;
    let offset_y = start_row as f32 * item_height;
    
    render! {
        div {
            class: "virtual-grid-container",
            style: "height: 600px; overflow-y: auto;",
            onscroll: move |event| {
                if let Some(target) = event.target_dyn_into::<web_sys::Element>() {
                    scroll_top.set(target.scroll_top() as f32);
                }
            },
            
            div {
                class: "virtual-grid-spacer",
                style: "height: {total_height}px; position: relative;",
                
                div {
                    class: "virtual-grid-items",
                    style: "position: absolute; top: {offset_y}px; width: 100%;",
                    
                    visible_items.iter().map(|(index, file)| {
                        let row = index / columns;
                        let col = index % columns;
                        let x = col as f32 * item_width;
                        let y = (row - start_row) as f32 * item_height;
                        
                        render! {
                            GridItem {
                                key: "{file.path}",
                                file: file.clone(),
                                config: cx.props.config.clone(),
                                is_selected: cx.props.selected_files.contains(&file.path),
                                position: (x, y),
                                on_select: move |multi_select| {
                                    cx.props.on_file_select.call((file.path.clone(), multi_select));
                                },
                                on_action: move |action| {
                                    cx.props.on_file_action.call(action);
                                }
                            }
                        }
                    })
                }
            }
        }
    }
}

#[derive(Props)]
pub struct GridItemProps {
    file: FileEntry,
    config: GridConfig,
    is_selected: bool,
    position: (f32, f32),
    on_select: EventHandler<bool>, // multi_select
    on_action: EventHandler<FileAction>,
}

pub fn GridItem(cx: Scope<GridItemProps>) -> Element {
    let (x, y) = cx.props.position;
    let services = use_shared_state::<Services>(cx).unwrap();
    let thumbnail = use_state(cx, || None::<Thumbnail>);
    
    // Load thumbnail asynchronously
    use_effect(cx, &cx.props.file.path, |path| {
        to_owned![thumbnail, services];
        async move {
            if let Ok(thumb) = services.read().preview.generate_thumbnail(&path, ThumbnailSize::Medium).await {
                thumbnail.set(Some(thumb));
            }
        }
    });
    
    render! {
        div {
            class: format!("grid-item {}", if cx.props.is_selected { "selected" } else { "" }),
            style: "position: absolute; left: {x}px; top: {y}px; width: {cx.props.config.icon_size.width()}px;",
            onclick: move |event| {
                let multi_select = event.modifiers().ctrl() || event.modifiers().meta();
                cx.props.on_select.call(multi_select);
            },
            ondoubleclick: move |_| {
                cx.props.on_action.call(FileAction::Open(cx.props.file.path.clone()));
            },
            oncontextmenu: move |event| {
                event.prevent_default();
                // Show context menu
            },
            
            div { class: "grid-item-thumbnail",
                if let Some(thumb) = thumbnail.get() {
                    render! {
                        img {
                            src: "{thumb.data_url}",
                            alt: "{cx.props.file.name}",
                            width: "{cx.props.config.icon_size.width()}",
                            height: "{cx.props.config.icon_size.height()}",
                        }
                    }
                } else {
                    render! {
                        FileIcon {
                            file_type: cx.props.file.file_type.clone(),
                            size: cx.props.config.icon_size.clone(),
                        }
                    }
                }
            }
            
            div { class: "grid-item-label",
                span { class: "file-name", "{cx.props.file.name}" }
                
                if cx.props.config.show_metadata {
                    render! {
                        div { class: "file-metadata",
                            span { class: "file-size", "{format_file_size(cx.props.file.size)}" }
                            span { class: "file-date", "{format_date(cx.props.file.modified)}" }
                        }
                    }
                }
            }
        }
    }
}
```

This component design provides a solid foundation for the MediaOrganizer application with proper separation of concerns, performance optimization through virtual scrolling, and comprehensive accessibility support.