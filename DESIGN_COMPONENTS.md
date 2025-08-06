# MediaOrganizer - Component Interface Design

## Overview

This document defines the detailed component interfaces and API specifications for the MediaOrganizer application, focusing on Dioxus component design patterns and inter-component communication.

## Component Design Principles

1. **Single Responsibility**: Each component has one clear purpose
2. **Composability**: Components can be easily combined and reused
3. **Props-Driven**: State flows down through props, events flow up
4. **Performance Optimized**: Minimal re-renders through strategic memoization
5. **Accessibility First**: All components include proper ARIA support

## Root Application Component

```rust
#[derive(Props)]
pub struct AppProps {
    // Initial configuration
    initial_path: Option<PathBuf>,
    config: AppConfig,
}

pub fn App(cx: Scope<AppProps>) -> Element {
    // Global state management
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    let services = use_shared_state::<Services>(cx).unwrap();
    
    // Global event handlers
    let on_file_system_event = use_callback(cx, (), move |event: FileSystemEvent, _| {
        // Handle file system changes
        to_owned![app_state];
        async move {
            app_state.write().handle_fs_event(event).await;
        }
    });
    
    render! {
        div {
            class: "app-container",
            onkeydown: move |event| {
                // Global keyboard shortcuts
                handle_global_shortcuts(event, &app_state, &services)
            },
            
            TopBar {
                current_path: app_state.read().current_path.clone(),
                breadcrumbs: app_state.read().get_breadcrumbs(),
                on_navigate: move |path| {
                    app_state.write().navigate_to(path);
                }
            }
            
            div {
                class: "main-content",
                style: "display: flex; height: calc(100vh - 40px);",
                
                LeftPanel {
                    width: app_state.read().layout.left_panel_width,
                    is_visible: app_state.read().layout.is_left_panel_visible,
                    current_path: app_state.read().current_path.clone(),
                    selected_path: app_state.read().selected_files.iter().next().cloned(),
                    on_resize: move |width| {
                        app_state.write().layout.left_panel_width = width;
                    },
                    on_navigate: move |path| {
                        app_state.write().navigate_to(path);
                    },
                    on_select: move |path| {
                        app_state.write().select_file(path, false);
                    }
                }
                
                ResizeHandle {
                    orientation: Orientation::Vertical,
                    on_resize: move |delta| {
                        let mut state = app_state.write();
                        state.layout.left_panel_width += delta;
                        state.layout.left_panel_width = state.layout.left_panel_width.clamp(200.0, 600.0);
                    }
                }
                
                RightPanel {
                    width: app_state.read().layout.right_panel_width,
                    current_path: app_state.read().current_path.clone(),
                    view_mode: app_state.read().view_mode.clone(),
                    sort_criteria: app_state.read().sort_criteria.clone(),
                    selected_files: app_state.read().selected_files.clone(),
                    on_view_mode_change: move |mode| {
                        app_state.write().view_mode = mode;
                    },
                    on_sort_change: move |criteria| {
                        app_state.write().sort_criteria = criteria;
                    },
                    on_file_select: move |path, multi_select| {
                        app_state.write().select_file(path, multi_select);
                    },
                    on_file_action: move |action| {
                        // Handle file operations
                        spawn_local(async move {
                            services.read().operations.execute_action(action).await;
                        });
                    }
                }
            }
            
            BottomBar {
                selected_count: app_state.read().selected_files.len(),
                total_size: app_state.read().get_selected_total_size(),
                current_operation: services.read().operations.get_current_operation(),
            }
        }
    }
}
```

## File Tree Component

```rust
#[derive(Props)]
pub struct FileTreeProps {
    /// Current directory being displayed
    current_path: PathBuf,
    /// Currently selected path (if any)
    selected_path: Option<PathBuf>,
    /// Width of the file tree panel
    width: f32,
    /// Whether the panel is visible
    is_visible: bool,
    /// Callback when user navigates to a new path
    on_navigate: EventHandler<PathBuf>,
    /// Callback when user selects a file/folder
    on_select: EventHandler<PathBuf>,
    /// Callback when panel is resized
    on_resize: EventHandler<f32>,
}

pub fn FileTree(cx: Scope<FileTreeProps>) -> Element {
    // Local state for expanded folders
    let expanded_folders = use_state(cx, HashSet::<PathBuf>::new);
    let tree_data = use_state(cx, Vec::<FileTreeNode>::new);
    let loading_state = use_state(cx, LoadingState::NotStarted);
    
    // Services
    let services = use_shared_state::<Services>(cx).unwrap();
    
    // Load tree data when current_path changes
    use_effect(cx, &cx.props.current_path, |current_path| {
        to_owned![tree_data, loading_state, services];
        async move {
            loading_state.set(LoadingState::Loading);
            
            match services.read().file_system.build_tree(&current_path).await {
                Ok(nodes) => {
                    tree_data.set(nodes);
                    loading_state.set(LoadingState::Loaded);
                }
                Err(e) => {
                    loading_state.set(LoadingState::Error(e.to_string()));
                }
            }
        }
    });
    
    // Keyboard navigation handler
    let handle_keydown = move |event: KeyboardEvent| {
        match event.key().as_str() {
            "ArrowUp" => {
                // Navigate to previous item
                event.prevent_default();
            }
            "ArrowDown" => {
                // Navigate to next item
                event.prevent_default();
            }
            "ArrowRight" | " " => {
                // Expand folder or navigate into it
                if let Some(selected) = &cx.props.selected_path {
                    if selected.is_dir() {
                        expanded_folders.modify(|folders| {
                            folders.insert(selected.clone());
                        });
                    }
                }
                event.prevent_default();
            }
            "ArrowLeft" => {
                // Collapse folder or navigate to parent
                if let Some(selected) = &cx.props.selected_path {
                    if expanded_folders.contains(selected) {
                        expanded_folders.modify(|folders| {
                            folders.remove(selected);
                        });
                    } else if let Some(parent) = selected.parent() {
                        cx.props.on_navigate.call(parent.to_path_buf());
                    }
                }
                event.prevent_default();
            }
            "Enter" => {
                // Navigate into selected folder
                if let Some(selected) = &cx.props.selected_path {
                    if selected.is_dir() {
                        cx.props.on_navigate.call(selected.clone());
                    }
                }
                event.prevent_default();
            }
            _ => {}
        }
    };
    
    if !cx.props.is_visible {
        return render! { div { class: "file-tree-hidden" } };
    }
    
    render! {
        div {
            class: "file-tree-container",
            style: "width: {cx.props.width}px;",
            onkeydown: handle_keydown,
            tabindex: 0,
            role: "tree",
            "aria-label": "File tree navigation",
            
            match loading_state.get() {
                LoadingState::Loading => render! {
                    div { class: "loading-indicator", "Loading..." }
                },
                LoadingState::Error(error) => render! {
                    div { class: "error-message", "Error: {error}" }
                },
                LoadingState::Loaded => render! {
                    div { class: "tree-content",
                        tree_data.iter().map(|node| render! {
                            FileTreeNode {
                                key: "{node.path}",
                                node: node.clone(),
                                level: 0,
                                is_expanded: expanded_folders.contains(&node.path),
                                is_selected: cx.props.selected_path.as_ref() == Some(&node.path),
                                on_toggle: move |path| {
                                    expanded_folders.modify(|folders| {
                                        if folders.contains(&path) {
                                            folders.remove(&path);
                                        } else {
                                            folders.insert(path);
                                        }
                                    });
                                },
                                on_select: move |path| {
                                    cx.props.on_select.call(path);
                                },
                                on_navigate: move |path| {
                                    cx.props.on_navigate.call(path);
                                }
                            }
                        })
                    }
                },
                _ => render! { div {} }
            }
        }
    }
}

#[derive(Props)]
pub struct FileTreeNodeProps {
    node: FileTreeNode,
    level: usize,
    is_expanded: bool,
    is_selected: bool,
    on_toggle: EventHandler<PathBuf>,
    on_select: EventHandler<PathBuf>,
    on_navigate: EventHandler<PathBuf>,
}

pub fn FileTreeNode(cx: Scope<FileTreeNodeProps>) -> Element {
    let node = &cx.props.node;
    let indent = cx.props.level * 16;
    
    render! {
        div {
            class: format!("tree-node {}", if cx.props.is_selected { "selected" } else { "" }),
            role: "treeitem",
            "aria-expanded": if node.is_directory { cx.props.is_expanded.to_string() } else { "false".to_string() },
            "aria-selected": cx.props.is_selected.to_string(),
            
            div {
                class: "tree-node-content",
                style: "padding-left: {indent}px;",
                onclick: move |_| {
                    if node.is_directory && !cx.props.is_expanded {
                        cx.props.on_toggle.call(node.path.clone());
                    }
                    cx.props.on_select.call(node.path.clone());
                },
                ondoubleclick: move |_| {
                    if node.is_directory {
                        cx.props.on_navigate.call(node.path.clone());
                    }
                },
                
                if node.is_directory {
                    render! {
                        button {
                            class: "expand-button",
                            onclick: move |event| {
                                event.stop_propagation();
                                cx.props.on_toggle.call(node.path.clone());
                            },
                            "aria-label": if cx.props.is_expanded { "Collapse folder" } else { "Expand folder" },
                            if cx.props.is_expanded { "▼" } else { "▶" }
                        }
                    }
                } else {
                    render! { span { class: "file-spacer" } }
                }
                
                FileIcon { file_type: node.file_type.clone() }
                
                span { class: "file-name", "{node.name}" }
            }
            
            if node.is_directory && cx.props.is_expanded {
                render! {
                    div { class: "tree-children",
                        node.children.iter().map(|child| render! {
                            FileTreeNode {
                                key: "{child.path}",
                                node: child.clone(),
                                level: cx.props.level + 1,
                                is_expanded: false, // Will be managed by parent
                                is_selected: false, // Will be managed by parent
                                on_toggle: move |path| cx.props.on_toggle.call(path),
                                on_select: move |path| cx.props.on_select.call(path),
                                on_navigate: move |path| cx.props.on_navigate.call(path),
                            }
                        })
                    }
                }
            }
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