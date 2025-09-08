use dioxus::prelude::*;
use dioxus::events::{KeyboardEvent, MouseEvent, DragData, MouseData, DragEvent};
use dioxus_free_icons::{Icon, icons::fa_solid_icons};
use crate::state::{ActivityBarView, use_activity_bar_view, use_sidebar_state, use_editor_state, use_panel_state, EditorGroup, EditorTab, TabType, EditorLayoutConfig, TabDragOperation, TabContextMenu, PanelTab};
use crate::services::preview::{PreviewData, PreviewContent};
use crate::ui::components::{WorkingFileTree, PreviewPanel};
use crate::ui::components::preview_panel::FileSystemEntry;

/// Global resize state for tracking active resize operations
static mut GLOBAL_RESIZE_STATE: Option<GlobalResizeState> = None;
static mut RESIZE_STATE_INIT: std::sync::Once = std::sync::Once::new();

#[derive(Debug, Clone)]
struct GlobalResizeState {
    is_panel_resizing: bool,
    resize_start_y: f64,
    resize_start_height: f64,
}

impl Default for GlobalResizeState {
    fn default() -> Self {
        Self {
            is_panel_resizing: false,
            resize_start_y: 0.0,
            resize_start_height: 0.0,
        }
    }
}

/// Get or initialize the global resize state
fn get_global_resize_state() -> &'static mut GlobalResizeState {
    unsafe {
        RESIZE_STATE_INIT.call_once(|| {
            GLOBAL_RESIZE_STATE = Some(GlobalResizeState::default());
        });
        GLOBAL_RESIZE_STATE.as_mut().unwrap()
    }
}

/// Main VS Code-like layout component
/// Implements the grid-based layout system with Activity Bar, Sidebar, Editor Groups, Panel, and Status Bar
#[component]
pub fn VSCodeLayout(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let mut activity_view = use_activity_bar_view(); // Get shared activity bar state
    let mut sidebar_state = use_sidebar_state();
    let mut panel_state = use_panel_state();
    
    // Calculate dynamic panel height with integration validation
    let panel_height = {
        let panel = panel_state.read();
        let calculated_height = if panel.is_visible {
            panel.height
        } else {
            0.0
        };
        
        // Ensure minimum space for editor area (at least 200px)
        let min_editor_space = 200.0_f64;
        let available_space = 800.0_f64 - 22.0_f64 - min_editor_space; // viewport - status bar - min editor
        let constrained_height = calculated_height.min(available_space.max(0.0_f64));
        
        tracing::debug!("Panel height integration: requested={}, constrained={}, editor_space_remaining={}", 
            calculated_height, constrained_height, 800.0_f64 - 22.0_f64 - constrained_height);
        
        constrained_height
    };
    
    rsx! {
        div {
            class: "vscode-layout",
            id: "vscode-layout",
            style: format!("
                display: grid;
                grid-template-areas: 
                    'activity sidebar editor'
                    'activity sidebar panel'
                    'status status status';
                grid-template-columns: 48px 240px 1fr;
                grid-template-rows: 1fr {}px 22px;
                height: 100vh;
                width: 100vw;
            ", panel_height),
            role: "application",
            "aria-label": "MediaOrganizer - VS Code style interface",
            "aria-describedby": "app-instructions",
            tabindex: "0",
            onkeydown: move |evt| {
                // Handle global keyboard shortcuts following VS Code patterns
                match evt.data.key() {
                    Key::Character(ch) if ch == "E" || ch == "e" => {
                        // Ctrl+Shift+E (toggle sidebar)
                        if evt.data.modifiers().ctrl() && evt.data.modifiers().shift() {
                            evt.prevent_default();
                            let mut sidebar = sidebar_state.write();
                            sidebar.is_collapsed = !sidebar.is_collapsed;
                            
                            // Announce the change for screen readers
                            let announcement = if sidebar.is_collapsed { "collapsed" } else { "expanded" };
                            tracing::info!("Global shortcut: Toggled sidebar via Ctrl+Shift+E - {}", announcement);
                        }
                    },
                    Key::Character(ch) if ch == "P" || ch == "p" => {
                        // Ctrl+Shift+P (command palette)
                        if evt.data.modifiers().ctrl() && evt.data.modifiers().shift() {
                            evt.prevent_default();
                            // TODO: Activate command palette when implemented
                            tracing::info!("Global shortcut: Command Palette via Ctrl+Shift+P");
                        }
                    },
                    Key::Character(ch) if ch == "J" || ch == "j" => {
                        // Ctrl+J (toggle panel)
                        if evt.data.modifiers().ctrl() {
                            evt.prevent_default();
                            let mut panel = panel_state.write();
                            panel.is_visible = !panel.is_visible;
                            
                            let announcement = if panel.is_visible { "opened" } else { "closed" };
                            tracing::info!("Global shortcut: Toggled panel via Ctrl+J - {}", announcement);
                        }
                    },
                    Key::Character(ch) if ch == "B" || ch == "b" => {
                        // Ctrl+B (toggle sidebar)
                        if evt.data.modifiers().ctrl() {
                            evt.prevent_default();
                            let mut sidebar = sidebar_state.write();
                            sidebar.is_collapsed = !sidebar.is_collapsed;
                            
                            let announcement = if sidebar.is_collapsed { "collapsed" } else { "expanded" };
                            tracing::info!("Global shortcut: Toggled sidebar via Ctrl+B - {}", announcement);
                        }
                    },
                    Key::Escape => {
                        // Escape key - return focus to main content area
                        evt.prevent_default();
                        // Focus the main editor area or first interactive element
                        // Web-only functionality, skip for desktop builds
                        #[cfg(feature = "web")]
                        if let Some(window) = web_sys::window() {
                            if let Some(document) = window.document() {
                                // Try to focus the editor groups container first
                                if let Ok(Some(editor_area)) = document.query_selector(".editor-groups") {
                                    let _ = editor_area.focus();
                                    tracing::info!("Focus returned to editor area via Escape");
                                } else if let Ok(Some(main_area)) = document.query_selector("[role=\"main\"]") {
                                    let _ = main_area.focus();
                                    tracing::info!("Focus returned to main content via Escape");
                                }
                            }
                        }
                    },
                    Key::F1 => {
                        // F1 - Command palette (VS Code standard)
                        evt.prevent_default();
                        // TODO: Activate command palette when implemented
                        tracing::info!("Global shortcut: Command Palette via F1");
                    },
                    _ => {}
                }
            },
            onmousemove: {
                let panel_state = panel_state.clone();
                move |evt: Event<MouseData>| {
                    // Handle panel resizing if it's being dragged
                    handle_global_mouse_move(evt, panel_state.clone());
                }
            },
            onmouseup: {
                let panel_state = panel_state.clone();
                move |_: Event<MouseData>| {
                    // Handle end of panel resizing
                    handle_global_mouse_up(panel_state.clone());
                }
            },
            
            // Skip links for keyboard navigation
            nav {
                class: "skip-links",
                "aria-label": "Skip navigation links",
                style: "
                    position: absolute;
                    top: -40px;
                    left: 6px;
                    z-index: 1000;
                ",
                
                a {
                    href: "#main-content",
                    style: "
                        position: absolute;
                        left: -10000px;
                        top: auto;
                        width: 1px;
                        height: 1px;
                        overflow: hidden;
                        background: #000;
                        color: #fff;
                        padding: 8px 16px;
                        text-decoration: none;
                        border-radius: 4px;
                        
                        &:focus {{
                            position: static;
                            width: auto;
                            height: auto;
                            overflow: visible;
                        }}
                    ",
                    onfocus: move |_| {
                        tracing::info!("Skip link focused: Skip to main content");
                    },
                    "Skip to main content"
                }
                
                a {
                    href: "#activity-bar",
                    style: "
                        position: absolute;
                        left: -10000px;
                        top: auto;
                        width: 1px;
                        height: 1px;
                        overflow: hidden;
                        background: #000;
                        color: #fff;
                        padding: 8px 16px;
                        text-decoration: none;
                        border-radius: 4px;
                        margin-left: 8px;
                        
                        &:focus {{
                            position: static;
                            width: auto;
                            height: auto;
                            overflow: visible;
                        }}
                    ",
                    onfocus: move |_| {
                        tracing::info!("Skip link focused: Skip to navigation");
                    },
                    "Skip to navigation"
                }
            }
            
            // Screen reader instructions for the application
            div {
                id: "app-instructions",
                class: "sr-only",
                style: "
                    position: absolute;
                    width: 1px;
                    height: 1px;
                    padding: 0;
                    margin: -1px;
                    overflow: hidden;
                    clip: rect(0, 0, 0, 0);
                    white-space: nowrap;
                    border: 0;
                ",
                "This is MediaOrganizer, a file management application with VS Code-style interface. 
                Use Tab to navigate between regions. Press Ctrl+1 through Ctrl+5 for direct activity bar access. 
                Press Ctrl+Shift+P for command palette. Use Escape to return focus to main content area."
            }
            
            // Activity Bar (leftmost vertical bar)
            ActivityBar {
                app_state: app_state
            }
            
            // Primary Sidebar (file explorer, search, etc.)
            Sidebar {
                app_state: app_state
            }
            
            // Editor Groups (main content area with tabs)
            EditorGroups {
                app_state: app_state
            }
            
            // Bottom Panel (terminal, problems, output)
            Panel {
                app_state: app_state
            }
            
            // Status Bar (bottom information bar)
            StatusBar {
                app_state: app_state
            }
        }
    }
}

/// Activity Bar component - vertical navigation on the left
#[component]
pub fn ActivityBar(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let mut activity_view = use_activity_bar_view(); // Get shared activity bar state
    let mut focused_item = use_signal(|| 0usize); // Track which item is focused
    
    // Activity items configuration - enhanced tooltips for icon-only design
    let activity_items = vec![
        (ActivityBarView::Explorer, "files", "File Explorer - Browse and manage project files (Ctrl+1)"),
        (ActivityBarView::Search, "search", "Search - Find text across all workspace files (Ctrl+2)"),
        (ActivityBarView::SourceControl, "source-control", "Source Control - Git integration and version management (Ctrl+3)"),
        (ActivityBarView::Debug, "debug-alt", "Run and Debug - Execute applications and debug code (Ctrl+4)"),
        (ActivityBarView::Extensions, "extensions", "Extensions - Install and manage VS Code extensions (Ctrl+5)"),
    ];
    
    // Handle keyboard activation of focused item
    let mut handle_focused_item_activation = {
        let activity_items = activity_items.clone();
        let mut activity_view = activity_view.clone();
        move || {
            let focused_index = *focused_item.read();
            if focused_index < activity_items.len() {
                let (view, _, _) = &activity_items[focused_index];
                activity_view.set(view.clone());
                tracing::info!("Activity Bar: Activated {:?} view via keyboard", view);
            } else if focused_index == activity_items.len() {
                // Settings item
                activity_view.set(ActivityBarView::Settings);
                tracing::info!("Activity Bar: Activated Settings view via keyboard");
            }
        }
    };
    
    rsx! {
        nav {
            id: "activity-bar",
            class: "activity-bar",
            style: "
                grid-area: activity;
                background: var(--vscode-activity-bar-background, #333333);
                border-right: 1px solid var(--vscode-border, #464647);
                display: flex;
                flex-direction: column;
                width: 48px;
            ",
            role: "navigation",
            "aria-label": "Primary navigation",
            "aria-describedby": "activity-bar-instructions",
            tabindex: "0",
            onkeydown: move |evt| {
                match evt.data.key() {
                    Key::ArrowDown => {
                        evt.prevent_default();
                        let current = *focused_item.read();
                        focused_item.set((current + 1).min(activity_items.len())); // Include settings item
                    },
                    Key::ArrowUp => {
                        evt.prevent_default();
                        let current = *focused_item.read();
                        focused_item.set(current.saturating_sub(1));
                    },
                    Key::Enter => {
                        evt.prevent_default();
                        handle_focused_item_activation();
                    },
                    Key::Character(s) if s == " " => {
                        evt.prevent_default();
                        handle_focused_item_activation();
                    },
                    // Enhanced keyboard shortcuts: Ctrl+1-5 for direct access
                    Key::Character(ch) if evt.data.modifiers().ctrl() => {
                        match ch.as_str() {
                            "1" => {
                                evt.prevent_default();
                                activity_view.set(ActivityBarView::Explorer);
                                focused_item.set(0); // Update focus indicator
                                tracing::info!("Activity Bar: Activated Explorer via Ctrl+1");
                            },
                            "2" => {
                                evt.prevent_default();
                                activity_view.set(ActivityBarView::Search);
                                focused_item.set(1); // Update focus indicator
                                tracing::info!("Activity Bar: Activated Search via Ctrl+2");
                            },
                            "3" => {
                                evt.prevent_default();
                                activity_view.set(ActivityBarView::SourceControl);
                                focused_item.set(2); // Update focus indicator
                                tracing::info!("Activity Bar: Activated Source Control via Ctrl+3");
                            },
                            "4" => {
                                evt.prevent_default();
                                activity_view.set(ActivityBarView::Debug);
                                focused_item.set(3); // Update focus indicator
                                tracing::info!("Activity Bar: Activated Debug via Ctrl+4");
                            },
                            "5" => {
                                evt.prevent_default();
                                activity_view.set(ActivityBarView::Extensions);
                                focused_item.set(4); // Update focus indicator
                                tracing::info!("Activity Bar: Activated Extensions via Ctrl+5");
                            },
                            _ => {}
                        }
                    },
                    // Home/End keys for navigation
                    Key::Home => {
                        evt.prevent_default();
                        focused_item.set(0);
                        tracing::info!("Activity Bar: Focused first item via Home");
                    },
                    Key::End => {
                        evt.prevent_default();
                        focused_item.set(activity_items.len().saturating_sub(1));
                        tracing::info!("Activity Bar: Focused last item via End");
                    },
                    _ => {}
                }
            },
            
            // Hidden instructions for screen readers
            div {
                id: "activity-bar-instructions",
                style: "
                    position: absolute;
                    width: 1px;
                    height: 1px;
                    padding: 0;
                    margin: -1px;
                    overflow: hidden;
                    clip: rect(0, 0, 0, 0);
                    white-space: nowrap;
                    border: 0;
                ",
                "Use arrow keys to navigate between tools. Press Enter or Space to activate a tool. Direct access: Ctrl+1 for Explorer, Ctrl+2 for Search, Ctrl+3 for Source Control, Ctrl+4 for Debug, Ctrl+5 for Extensions. Ctrl+Shift+E to toggle sidebar."
            }
            
            // Activity Bar Items
            div {
                class: "activity-bar-items",
                style: "display: flex; flex-direction: column; flex: 1;",
                
                for (index, (view, icon, label)) in activity_items.iter().enumerate() {
                    ActivityBarItem {
                        key: "{index}",
                        icon: icon.to_string(),
                        label: label.to_string(),
                        active: *activity_view.read() == *view,
                        focused: *focused_item.read() == index,
                        item_index: index,
                        on_click: {
                            let view = view.clone();
                            let mut activity_view = activity_view.clone();
                            move |_| {
                                activity_view.set(view.clone());
                                tracing::info!("Activity Bar: Switched to {:?} view", view);
                            }
                        }
                    }
                }
            }
            
            // Bottom items (Settings, etc.)
            div {
                class: "activity-bar-footer",
                style: "display: flex; flex-direction: column;",
                
                ActivityBarItem {
                    icon: "settings-gear".to_string(),
                    label: "Settings".to_string(),
                    active: *activity_view.read() == ActivityBarView::Settings,
                    focused: *focused_item.read() == activity_items.len(),
                    item_index: activity_items.len(),
                    on_click: {
                        let mut activity_view = activity_view.clone();
                        move |_| {
                            activity_view.set(ActivityBarView::Settings);
                            tracing::info!("Activity Bar: Switched to Settings view");
                        }
                    }
                }
            }
        }
    }
}

/// Individual Activity Bar item
#[component]
pub fn ActivityBarItem(
    icon: String,
    label: String,
    active: bool,
    focused: bool,
    item_index: usize,
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let item_style = format!(
        "
            width: 48px;
            height: 48px;
            border: {};
            background: {};
            color: var(--vscode-foreground, #cccccc);
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            position: relative;
            outline: none;
        ",
        if focused { "2px solid var(--vscode-focusBorder, #007acc)" } else { "none" },
        if active { 
            "var(--vscode-list-activeSelectionBackground, rgba(0, 122, 204, 0.3))" 
        } else if focused {
            "var(--vscode-list-hoverBackground, rgba(255, 255, 255, 0.1))"
        } else { 
            "transparent" 
        }
    );
    
    rsx! {
        button {
            class: "activity-bar-item",
            style: item_style,
            title: label.clone(),
            "aria-label": label.clone(),
            "aria-pressed": if active { "true" } else { "false" },
            "aria-describedby": format!("activity-item-{}", item_index),
            "data-index": format!("{}", item_index + 1), // For voice control
            tabindex: "-1", // Managed by parent container
            onclick: move |evt| on_click.call(evt),
            
            // Icon using dioxus-free-icons
            {get_activity_bar_icon(&icon)}
            
            // Enhanced active indicator with better visibility
            if active {
                div {
                    class: "activity-bar-indicator",
                    style: "
                        position: absolute;
                        left: 0;
                        top: 50%;
                        transform: translateY(-50%);
                        width: 3px;
                        height: 20px;
                        background: var(--vscode-activityBarBadge-background, #007acc);
                        border-radius: 0 2px 2px 0;
                    "
                }
            }
        }
    }
}

/// Sidebar component - collapsible file explorer
#[component]
pub fn Sidebar(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let mut sidebar_state = use_sidebar_state();
    let activity_view = use_activity_bar_view();
    
    // Resizing state
    let mut is_resizing = use_signal(|| false);
    let mut resize_start_x = use_signal(|| 0.0);
    let mut resize_start_width = use_signal(|| 0.0);
    
    let sidebar_width = sidebar_state.read().width;
    let is_collapsed = sidebar_state.read().is_collapsed;
    
    rsx! {
        aside {
            class: "sidebar",
            style: format!("
                grid-area: sidebar;
                background: var(--vscode-sidebar-background, #252526);
                border-right: 1px solid var(--vscode-border, #464647);
                display: {};
                flex-direction: column;
                width: {}px;
                min-width: {}px;
                max-width: {}px;
                position: relative;
            ", 
                if is_collapsed { "none" } else { "flex" },
                if is_collapsed { 0.0 } else { sidebar_width },
                sidebar_state.read().min_width,
                sidebar_state.read().max_width
            ),
            role: "complementary",
            "aria-label": "File explorer sidebar",
            "aria-labelledby": "sidebar-header",
            tabindex: "0",
            
            if !is_collapsed {
                // Sidebar Header
                header {
                    id: "sidebar-header",
                    class: "sidebar-header",
                    style: "
                        height: 35px;
                        padding: 0 16px;
                        display: flex;
                        align-items: center;
                        border-bottom: 1px solid var(--vscode-border, #464647);
                        font-size: var(--vscode-font-size-small);
                        font-weight: bold;
                        color: var(--vscode-foreground, #cccccc);
                    ",
                    role: "banner",
                    title: "File Explorer",
                    // Icon-based navigation instead of text label
                    span {
                        style: "
                            display: inline-block;
                            width: 16px;
                            height: 16px;
                            opacity: 0.7;
                        ",
                        "📁"
                    }
                }
                
                // Sidebar Content
                div {
                    class: "sidebar-content",
                    style: "
                        flex: 1;
                        overflow: hidden;
                    ",
                    
                    // Render content based on active activity view
                    match *activity_view.read() {
                        ActivityBarView::Explorer => rsx! {
                            WorkingFileTree {}
                        },
                        _ => rsx! {
                            div {
                                style: "padding: 16px; color: var(--vscode-foreground, #cccccc); text-align: center;",
                                "Feature coming soon..."
                            }
                        }
                    }
                }
                
                // Resize handle
                div {
                    class: "sidebar-resize-handle",
                    style: "
                        position: absolute;
                        top: 0;
                        right: -2px;
                        width: 4px;
                        height: 100%;
                        cursor: col-resize;
                        background: transparent;
                        z-index: 10;
                    ",
                    title: "Resize sidebar",
                    onmousedown: {
                        let mut is_resizing = is_resizing.clone();
                        let mut resize_start_x = resize_start_x.clone();
                        let mut resize_start_width = resize_start_width.clone();
                        let sidebar_state = sidebar_state.clone();
                        move |evt: Event<MouseData>| {
                            is_resizing.set(true);
                            resize_start_x.set(evt.data.client_coordinates().x as f64);
                            resize_start_width.set(sidebar_state.read().width);
                            tracing::info!("Started resizing sidebar");
                        }
                    }
                }
            }
        }
    }
}

/// Editor Groups component - tabbed interface for content with full split view support
/// Integrates seamlessly with Panel component via CSS Grid layout system
/// - Occupies the 'editor' grid area 
/// - Automatically adjusts to available space when panel height changes
/// - Maintains responsive design across all layout configurations
#[component]
pub fn EditorGroups(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let mut editor_state = use_editor_state();
    let mut focused_tab_index = use_signal(|| 0usize);
    
    // Clone app_state for use in rsx! closures
    let app_state_clone = app_state.clone();
    
    // Get active group for keyboard navigation
    let editor_data = editor_state.read();
    let active_group = editor_data.editor_groups.get(editor_data.active_group)
        .cloned()
        .unwrap_or_else(|| editor_data.editor_groups[0].clone());
    drop(editor_data);
    
    // Handle tab switching via keyboard
    let handle_tab_keyboard = {
        let mut editor_state = editor_state.clone();
        let mut focused_tab_index = focused_tab_index.clone();
        move |evt: Event<KeyboardData>| {
            match evt.data.key() {
                Key::ArrowLeft => {
                    evt.prevent_default();
                    let current = *focused_tab_index.read();
                    focused_tab_index.set(current.saturating_sub(1));
                },
                Key::ArrowRight => {
                    evt.prevent_default();
                    let current = *focused_tab_index.read();
                    let editor_data = editor_state.read();
                    let active_group = &editor_data.editor_groups[editor_data.active_group];
                    let new_focus = (current + 1).min(active_group.tabs.len().saturating_sub(1));
                    drop(editor_data);
                    focused_tab_index.set(new_focus);
                },
                Key::Enter => {
                    evt.prevent_default();
                    let focused_idx = *focused_tab_index.read();
                    let active_group_idx = {
                        let editor_data = editor_state.read();
                        editor_data.active_group
                    };
                    let mut editor_data = editor_state.write();
                    if let Some(group) = editor_data.editor_groups.get_mut(active_group_idx) {
                        if focused_idx < group.tabs.len() {
                            // Update active tab in the group
                            group.active_tab = focused_idx;
                            // Update all tabs' active state
                            for (i, tab) in group.tabs.iter_mut().enumerate() {
                                tab.is_active = i == focused_idx;
                            }
                            tracing::info!("Switched to tab {} via keyboard", focused_idx);
                        }
                    }
                },
                Key::Character(s) if s == " " => {
                    evt.prevent_default();
                    let focused_idx = *focused_tab_index.read();
                    let active_group_idx = {
                        let editor_data = editor_state.read();
                        editor_data.active_group
                    };
                    let mut editor_data = editor_state.write();
                    if let Some(group) = editor_data.editor_groups.get_mut(active_group_idx) {
                        if focused_idx < group.tabs.len() {
                            // Update active tab in the group
                            group.active_tab = focused_idx;
                            // Update all tabs' active state
                            for (i, tab) in group.tabs.iter_mut().enumerate() {
                                tab.is_active = i == focused_idx;
                            }
                            tracing::info!("Switched to tab {} via space", focused_idx);
                        }
                    }
                },
                // VS Code-style shortcuts: Ctrl+1, Ctrl+2, Ctrl+3, etc.
                Key::Character(ch) if evt.data.modifiers().ctrl() => {
                    match ch.as_str() {
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                            evt.prevent_default();
                            if let Some(tab_num) = ch.parse::<usize>().ok() {
                                let target_tab = tab_num.saturating_sub(1); // Convert to 0-based index
                                let active_group_idx = {
                                    let editor_data = editor_state.read();
                                    editor_data.active_group
                                };
                                let mut editor_data = editor_state.write();
                                if let Some(group) = editor_data.editor_groups.get_mut(active_group_idx) {
                                    if target_tab < group.tabs.len() {
                                        // Update active tab in the group
                                        group.active_tab = target_tab;
                                        // Update all tabs' active state
                                        for (i, tab) in group.tabs.iter_mut().enumerate() {
                                            tab.is_active = i == target_tab;
                                        }
                                        // Update focused tab index
                                        focused_tab_index.set(target_tab);
                                        tracing::info!("Switched to tab {} via Ctrl+{}", target_tab, ch);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                },
                // Ctrl+Tab: cycle to next tab
                Key::Tab if evt.data.modifiers().ctrl() => {
                    evt.prevent_default();
                    let active_group_idx = {
                        let editor_data = editor_state.read();
                        editor_data.active_group
                    };
                    let mut editor_data = editor_state.write();
                    if let Some(group) = editor_data.editor_groups.get_mut(active_group_idx) {
                        if !group.tabs.is_empty() {
                            let current_tab = group.active_tab;
                            let next_tab = (current_tab + 1) % group.tabs.len();
                            
                            // Update active tab in the group
                            group.active_tab = next_tab;
                            // Update all tabs' active state
                            for (i, tab) in group.tabs.iter_mut().enumerate() {
                                tab.is_active = i == next_tab;
                            }
                            // Update focused tab index
                            focused_tab_index.set(next_tab);
                            tracing::info!("Switched to tab {} via Ctrl+Tab", next_tab);
                        }
                    }
                },
                // Ctrl+Shift+Tab: cycle to previous tab
                Key::Tab if evt.data.modifiers().ctrl() && evt.data.modifiers().shift() => {
                    evt.prevent_default();
                    let active_group_idx = {
                        let editor_data = editor_state.read();
                        editor_data.active_group
                    };
                    let mut editor_data = editor_state.write();
                    if let Some(group) = editor_data.editor_groups.get_mut(active_group_idx) {
                        if !group.tabs.is_empty() {
                            let current_tab = group.active_tab;
                            let prev_tab = if current_tab == 0 {
                                group.tabs.len() - 1
                            } else {
                                current_tab - 1
                            };
                            
                            // Update active tab in the group
                            group.active_tab = prev_tab;
                            // Update all tabs' active state
                            for (i, tab) in group.tabs.iter_mut().enumerate() {
                                tab.is_active = i == prev_tab;
                            }
                            // Update focused tab index
                            focused_tab_index.set(prev_tab);
                            tracing::info!("Switched to tab {} via Ctrl+Shift+Tab", prev_tab);
                        }
                    }
                },
                // Ctrl+W: close active tab
                Key::Character(ch) if ch == "w" || ch == "W" => {
                    if evt.data.modifiers().ctrl() {
                        evt.prevent_default();
                        let active_group_idx = {
                            let editor_data = editor_state.read();
                            editor_data.active_group
                        };
                        let active_tab_idx = {
                            let editor_data = editor_state.read();
                            editor_data.editor_groups.get(active_group_idx)
                                .map(|group| group.active_tab)
                                .unwrap_or(0)
                        };
                        close_tab(editor_state.clone(), active_group_idx, active_tab_idx);
                        tracing::info!("Closed active tab via Ctrl+W");
                    }
                },
                // Ctrl+K, P: toggle pin for active tab (simplified as Ctrl+P for now)
                Key::Character(ch) if ch == "p" || ch == "P" => {
                    if evt.data.modifiers().ctrl() {
                        evt.prevent_default();
                        let active_group_idx = {
                            let editor_data = editor_state.read();
                            editor_data.active_group
                        };
                        let active_tab_idx = {
                            let editor_data = editor_state.read();
                            editor_data.editor_groups.get(active_group_idx)
                                .map(|group| group.active_tab)
                                .unwrap_or(0)
                        };
                        toggle_pin_tab(editor_state.clone(), active_group_idx, active_tab_idx);
                        tracing::info!("Toggled pin for active tab via Ctrl+P");
                    }
                },
                _ => {}
            }
        }
    };
    
    rsx! {
        main {
            id: "main-content",
            class: "editor-groups-container",
            style: "
                grid-area: editor;
                background: var(--vscode-background, #1e1e1e);
                display: flex;
                flex-direction: column;
                position: relative;
            ",
            role: "main",
            "aria-label": "Editor groups with tabbed interface - main content area",
            tabindex: "-1", // Allows programmatic focus via skip links
            onkeydown: handle_tab_keyboard,
            onclick: {
                let mut editor_state = editor_state.clone();
                move |_| {
                    // Close context menu when clicking outside
                    let mut editor_data = editor_state.write();
                    if editor_data.context_menu.is_some() {
                        editor_data.context_menu = None;
                        tracing::info!("Context menu closed by click outside");
                    }
                }
            },
            
            // Render all editor groups based on layout configuration
            EditorGroupsLayoutComponent {
                editor_state: editor_state,
                focused_tab_index: focused_tab_index,
                app_state: app_state_clone,
            }
            
            // Render context menu if visible
            TabContextMenuComponent {
                editor_state: editor_state,
            }
        }
    }
}

/// Panel component - bottom panel for terminal, problems, etc.
/// Integrated with EditorGroups via unified CSS Grid layout system
/// - Occupies the 'panel' grid area with dynamic height allocation
/// - Maintains proper space distribution with editor area
/// - Supports resizable layout with height constraints (150px-50% viewport)
#[component]
pub fn Panel(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let mut panel_state = use_panel_state();
    
    // Resizing state
    let mut is_resizing = use_signal(|| false);
    let mut resize_start_y = use_signal(|| 0.0);
    let mut resize_start_height = use_signal(|| 0.0);
    
    let panel_data = panel_state.read();
    let panel_height = panel_data.height;
    let is_visible = panel_data.is_visible;
    let active_tab = panel_data.active_tab.clone();
    drop(panel_data);
    
    if !is_visible {
        return rsx! { div { style: "display: none;" } };
    }
    
    rsx! {
        aside {
            class: "panel",
            style: format!("
                grid-area: panel;
                background: var(--vscode-sidebar-background, #252526);
                border-top: 1px solid var(--vscode-border, #464647);
                display: flex;
                flex-direction: column;
                height: {}px;
                position: relative;
            ", panel_height),
            role: "complementary",
            "aria-label": "Bottom panel with terminal and output",
            "aria-labelledby": "panel-header",
            tabindex: "0",
            
            // Resize handle at the top
            div {
                class: "panel-resize-handle",
                style: "
                    position: absolute;
                    top: -2px;
                    left: 0;
                    right: 0;
                    height: 4px;
                    cursor: row-resize;
                    background: transparent;
                    z-index: 10;
                ",
                title: "Resize panel",
                onmousedown: {
                    let mut is_resizing = is_resizing.clone();
                    let mut resize_start_y = resize_start_y.clone();
                    let mut resize_start_height = resize_start_height.clone();
                    let panel_state = panel_state.clone();
                    move |evt: Event<MouseData>| {
                        let resize_state = get_global_resize_state();
                        
                        // Set local component state
                        is_resizing.set(true);
                        let start_y = evt.data.client_coordinates().y as f64;
                        let start_height = panel_state.read().height;
                        resize_start_y.set(start_y);
                        resize_start_height.set(start_height);
                        
                        // Set global resize state
                        resize_state.is_panel_resizing = true;
                        resize_state.resize_start_y = start_y;
                        resize_state.resize_start_height = start_height;
                        
                        tracing::info!("Started resizing panel from y={}, height={}", start_y, start_height);
                    }
                }
            }
            
            // Panel Header
            header {
                id: "panel-header",
                class: "panel-header",
                style: "
                    height: 35px;
                    display: flex;
                    align-items: center;
                    background: var(--vscode-tab-inactive-background, #2d2d30);
                    border-bottom: 1px solid var(--vscode-border, #464647);
                ",
                role: "tablist",
                "aria-label": "Panel tabs",
                
                // Panel tabs - dynamically generate based on available tabs
                {get_panel_tabs(panel_state.clone(), active_tab.clone())}
            }
            
            // Panel Content
            div {
                class: "panel-content",
                style: "
                    flex: 1;
                    padding: 8px;
                    color: var(--vscode-foreground, #cccccc);
                    overflow-y: auto;
                ",
                role: "tabpanel",
                tabindex: "0",
                
                {get_panel_content(active_tab)}
            }
        }
    }
}

/// Status Bar component - bottom information bar
#[component]
pub fn StatusBar(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    rsx! {
        footer {
            class: "status-bar",
            style: "
                grid-area: status;
                background: var(--vscode-status-bar-background, #007acc);
                color: var(--vscode-status-bar-foreground, #ffffff);
                height: 22px;
                display: flex;
                align-items: center;
                padding: 0 8px;
                font-size: var(--vscode-font-size-small);
            ",
            role: "status",
            "aria-label": "Application status bar",
            "aria-live": "polite",
            tabindex: "0",
            
            // Left side status items
            div {
                class: "status-bar-left",
                style: "display: flex; align-items: center; gap: 8px;",
                
                span { "Ready" }
                span { "MediaOrganizer v0.1.0" }
            }
            
            // Spacer
            div {
                style: "flex: 1;"
            }
            
            // Right side status items
            div {
                class: "status-bar-right",
                style: "display: flex; align-items: center; gap: 8px;",
                
                span { "VS Code Layout" }
                span { "Task 11.1" }
            }
        }
    }
}

/// Editor Groups Layout Component - renders editor groups in different split configurations
#[component]
pub fn EditorGroupsLayoutComponent(
    editor_state: Signal<crate::state::EditorState>,
    focused_tab_index: Signal<usize>,
    app_state: Signal<crate::state::AppState>,
) -> Element {
    let editor_data = editor_state.read();
    let layout_config = &editor_data.layout_config;
    
    match layout_config {
        crate::state::EditorLayoutConfig::Single => {
            // Single editor group taking full space
            rsx! {
                div {
                    class: "editor-layout-single",
                    style: "
                        width: 100%;
                        height: 100%;
                        display: flex;
                        flex-direction: column;
                    ",
                    
                    if let Some(group) = editor_data.editor_groups.get(0) {
                        EditorGroupComponent {
                            key: "{group.id}",
                            group: group.clone(),
                            group_index: 0,
                            editor_state: editor_state,
                            focused_tab_index: focused_tab_index,
                        }
                    }
                }
            }
        },
        crate::state::EditorLayoutConfig::SplitHorizontal => {
            // Two groups split horizontally (left/right)
            rsx! {
                div {
                    class: "editor-layout-split-horizontal",
                    style: "
                        width: 100%;
                        height: 100%;
                        display: flex;
                        flex-direction: row;
                    ",
                    
                    for (index, group) in editor_data.editor_groups.iter().enumerate() {
                        EditorGroupComponent {
                            key: "{group.id}",
                            group: group.clone(),
                            group_index: index,
                            editor_state: editor_state,
                            focused_tab_index: focused_tab_index,
                            style: "flex: 1; min-width: 0;".to_string(),
                        }
                    }
                }
            }
        },
        crate::state::EditorLayoutConfig::SplitVertical => {
            // Two groups split vertically (top/bottom)
            rsx! {
                div {
                    class: "editor-layout-split-vertical",
                    style: "
                        width: 100%;
                        height: 100%;
                        display: flex;
                        flex-direction: column;
                    ",
                    
                    for (index, group) in editor_data.editor_groups.iter().enumerate() {
                        EditorGroupComponent {
                            key: "{group.id}",
                            group: group.clone(),
                            group_index: index,
                            editor_state: editor_state,
                            focused_tab_index: focused_tab_index,
                            style: "flex: 1; min-height: 0;".to_string(),
                        }
                    }
                }
            }
        },
        crate::state::EditorLayoutConfig::Grid { rows, cols } => {
            // Grid layout with multiple groups
            rsx! {
                div {
                    class: "editor-layout-grid",
                    style: format!("
                        width: 100%;
                        height: 100%;
                        display: grid;
                        grid-template-rows: repeat({}, 1fr);
                        grid-template-columns: repeat({}, 1fr);
                        gap: 1px;
                    ", rows, cols),
                    
                    for (index, group) in editor_data.editor_groups.iter().enumerate() {
                        EditorGroupComponent {
                            key: "{group.id}",
                            group: group.clone(),
                            group_index: index,
                            editor_state: editor_state,
                            focused_tab_index: focused_tab_index,
                            style: "min-width: 0; min-height: 0;".to_string(),
                        }
                    }
                }
            }
        },
    }
}

/// Individual Editor Group Component - renders a single editor group with tabs
#[component]
pub fn EditorGroupComponent(
    group: crate::state::EditorGroup,
    group_index: usize,
    editor_state: Signal<crate::state::EditorState>,
    focused_tab_index: Signal<usize>,
    #[props(default = String::new())] style: String,
) -> Element {
    let is_active_group = {
        let editor_data = editor_state.read();
        editor_data.active_group == group_index
    };
    
    rsx! {
        div {
            class: format!("editor-group {}", if is_active_group { "active" } else { "" }),
            style: format!("
                display: flex;
                flex-direction: column;
                background: var(--vscode-background, #1e1e1e);
                border: 1px solid var(--vscode-border, #464647);
                {}
            ", style),
            "data-group-id": group.id,
            "aria-label": format!("Editor group {}", group.id),
            
            // Tab bar
            div {
                class: "editor-group-tabs",
                style: "
                    display: flex;
                    height: 35px;
                    background: var(--vscode-tab-inactive-background, #2d2d30);
                    border-bottom: 1px solid var(--vscode-border, #464647);
                ",
                role: "tablist",
                "aria-label": "Editor tabs",
                ondragover: {
                    move |evt: Event<DragData>| {
                        evt.prevent_default();
                        // Note: set_drop_effect not available in current Dioxus version
                    }
                },
                ondrop: {
                    let mut editor_state = editor_state.clone();
                    move |evt: Event<DragData>| {
                        evt.prevent_default();
                        handle_tab_drop(evt, editor_state.clone(), group_index);
                    }
                },
                
                for (tab_index, tab) in group.tabs.iter().enumerate() {
                    EditorTabComponent {
                        key: "{tab.id}",
                        tab: tab.clone(),
                        tab_index: tab_index,
                        group_index: group_index,
                        is_focused: is_active_group && *focused_tab_index.read() == tab_index,
                        editor_state: editor_state,
                    }
                }
            }
            
            // Tab content
            div {
                class: "editor-group-content",
                style: "
                    flex: 1;
                    overflow: hidden;
                    background: var(--vscode-background, #1e1e1e);
                ",
                role: "tabpanel",
                "aria-label": "Editor content",
                
                if let Some(active_tab) = group.tabs.get(group.active_tab) {
                    EditorTabContentComponent {
                        tab: active_tab.clone(),
                        group_index: group_index,
                    }
                }
            }
        }
    }
}

/// Individual Editor Tab Component - renders a single tab
#[component]
pub fn EditorTabComponent(
    tab: crate::state::EditorTab,
    tab_index: usize,
    group_index: usize,
    is_focused: bool,
    editor_state: Signal<crate::state::EditorState>,
) -> Element {
    let tab_style = format!(
        "
            display: flex;
            align-items: center;
            height: 35px;
            min-width: 120px;
            max-width: 200px;
            padding: 0 12px;
            background: {};
            color: {};
            border-right: 1px solid var(--vscode-border, #464647);
            cursor: pointer;
            position: relative;
            outline: {};
        ",
        if tab.is_active { 
            "var(--vscode-tab-active-background, #1e1e1e)" 
        } else if is_focused {
            "var(--vscode-list-hoverBackground, rgba(255, 255, 255, 0.1))"
        } else { 
            "var(--vscode-tab-inactive-background, #2d2d30)" 
        },
        if tab.is_active { 
            "var(--vscode-foreground, #cccccc)" 
        } else { 
            "var(--vscode-text-secondary, #999999)" 
        },
        if is_focused { "2px solid var(--vscode-focusBorder, #007acc)" } else { "none" }
    );
    
    rsx! {
        button {
            class: format!("editor-tab {}", if tab.is_active { "active" } else { "" }),
            style: tab_style,
            role: "tab",
            "aria-selected": if tab.is_active { "true" } else { "false" },
            "aria-controls": format!("tabpanel-{}-{}", group_index, tab_index),
            tabindex: if is_focused { "0" } else { "-1" },
            "data-tab-id": tab.id,
            title: tab.title.clone(),
            draggable: "true",
            onclick: {
                let mut editor_state = editor_state.clone();
                move |_| {
                    let mut editor_data = editor_state.write();
                    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
                        // Update active tab in the group
                        group.active_tab = tab_index;
                        // Update all tabs' active state
                        for (i, t) in group.tabs.iter_mut().enumerate() {
                            t.is_active = i == tab_index;
                        }
                        // Set this group as active
                        editor_data.active_group = group_index;
                        tracing::info!("Switched to tab {} in group {}", tab_index, group_index);
                    }
                }
            },
            ondragstart: {
                let mut editor_state = editor_state.clone();
                let tab_id = tab.id;
                move |evt: Event<DragData>| {
                    // Set up drag operation
                    let mut editor_data = editor_state.write();
                    editor_data.drag_operation = Some(TabDragOperation {
                        tab_id,
                        tab_index,
                        source_group_id: group_index,
                    });
                    
                    // Note: Dioxus 0.6 DragData methods may be different
                    // For now, we'll rely on the drag operation state management
                    
                    tracing::info!("Started dragging tab {} from group {}", tab_index, group_index);
                }
            },
            ondragend: {
                let mut editor_state = editor_state.clone();
                move |_: Event<DragData>| {
                    // Clear drag operation
                    let mut editor_data = editor_state.write();
                    editor_data.drag_operation = None;
                    tracing::info!("Drag operation ended");
                }
            },
            oncontextmenu: {
                let mut editor_state = editor_state.clone();
                let tab_id = tab.id;
                move |evt: Event<MouseData>| {
                    evt.prevent_default(); // Prevent browser context menu
                    let client_x = evt.data.client_coordinates().x;
                    let client_y = evt.data.client_coordinates().y;
                    
                    let mut editor_data = editor_state.write();
                    editor_data.context_menu = Some(TabContextMenu {
                        tab_id,
                        tab_index,
                        group_index,
                        x: client_x,
                        y: client_y,
                        is_visible: true,
                    });
                    
                    tracing::info!("Context menu opened for tab {} at ({}, {})", tab_index, client_x, client_y);
                }
            },
            
            // Tab icon
            div {
                class: "tab-icon",
                style: "margin-right: 6px; display: flex; align-items: center;",
                {get_tab_icon(&tab.tab_type)}
            }
            
            // Tab title
            span {
                class: "tab-title",
                style: "
                    flex: 1;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                ",
                "{tab.title}"
                if tab.is_dirty {
                    " •"
                }
            }
            
            // Pin indicator (if tab is pinned)
            if tab.is_pinned {
                div {
                    class: "tab-pin-indicator",
                    style: "
                        margin-left: 4px;
                        width: 12px;
                        height: 12px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        color: var(--vscode-tab-activeBackground, #1e1e1e);
                    ",
                    title: "Pinned tab",
                    "aria-label": "Tab is pinned",
                    Icon { 
                        icon: fa_solid_icons::FaThumbtack, 
                        width: 10, 
                        height: 10,
                        fill: "currentColor"
                    }
                }
            }
            
            // Close button
            button {
                class: "tab-close",
                style: "
                    margin-left: 4px;
                    width: 16px;
                    height: 16px;
                    border: none;
                    background: transparent;
                    color: inherit;
                    cursor: pointer;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    border-radius: 2px;
                ",
                title: "Close tab",
                "aria-label": format!("Close {}", tab.title),
                onclick: {
                    let mut editor_state = editor_state.clone();
                    move |evt: Event<MouseData>| {
                        evt.stop_propagation();
                        let mut editor_data = editor_state.write();
                        if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
                            if group.tabs.len() > 1 {
                                group.tabs.remove(tab_index);
                                // Adjust active tab if necessary
                                if group.active_tab >= group.tabs.len() {
                                    group.active_tab = group.tabs.len().saturating_sub(1);
                                }
                                // Update active states
                                for (i, t) in group.tabs.iter_mut().enumerate() {
                                    t.is_active = i == group.active_tab;
                                }
                                tracing::info!("Closed tab {} in group {}", tab_index, group_index);
                            }
                        }
                    }
                },
                "×"
            }
        }
    }
}

/// Tab Context Menu Component - renders the right-click context menu for tabs
#[component]
pub fn TabContextMenuComponent(
    editor_state: Signal<crate::state::EditorState>,
) -> Element {
    let editor_data = editor_state.read();
    let context_menu = match &editor_data.context_menu {
        Some(menu) if menu.is_visible => menu,
        _ => return rsx! {},
    };
    
    // Get tab and group information
    let group = editor_data.editor_groups.get(context_menu.group_index);
    let tab = group.and_then(|g| g.tabs.get(context_menu.tab_index));
    
    if let (Some(group), Some(tab)) = (group, tab) {
        rsx! {
            div {
                class: "tab-context-menu",
                style: format!("
                    position: fixed;
                    left: {}px;
                    top: {}px;
                    background: var(--vscode-menu-background, #2c2c2c);
                    border: 1px solid var(--vscode-menu-border, #454545);
                    border-radius: 3px;
                    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
                    z-index: 1000;
                    min-width: 180px;
                    padding: 4px 0;
                    font-size: var(--vscode-font-size-normal);
                    color: var(--vscode-foreground, #cccccc);
                ", context_menu.x, context_menu.y),
                role: "menu",
                "aria-label": "Tab actions menu",
                
                // Close menu when clicking outside (handled by global click handler)
                
                // Close
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    tabindex: "0",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            close_tab(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    onkeydown: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |evt: Event<KeyboardData>| {
                            match evt.data.key() {
                                Key::Enter => {
                                    evt.prevent_default();
                                    close_tab(editor_state.clone(), group_index, tab_index);
                                }
                                Key::Character(s) if s == " " => {
                                    evt.prevent_default();
                                    close_tab(editor_state.clone(), group_index, tab_index);
                                }
                                _ => {}
                            }
                        }
                    },
                    onmouseenter: |_| {}, // Handled by CSS hover
                    onmouseleave: |_| {}, // Handled by CSS hover
                    Icon { icon: fa_solid_icons::FaXmark, width: 12, height: 12 }
                    "Close"
                    span {
                        style: "margin-left: auto; color: var(--vscode-text-secondary, #999);",
                        "Ctrl+W"
                    }
                }
                
                // Close Others
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            close_others(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    Icon { icon: fa_solid_icons::FaXmark, width: 12, height: 12 }
                    "Close Others"
                }
                
                // Close Tabs to the Right
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            close_tabs_to_right(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    Icon { icon: fa_solid_icons::FaAnglesRight, width: 12, height: 12 }
                    "Close Tabs to the Right"
                }
                
                // Pin/Unpin Tab (moved up to group with close actions)
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            toggle_pin_tab(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    Icon { 
                        icon: if tab.is_pinned { fa_solid_icons::FaThumbtack } else { fa_solid_icons::FaThumbtack }, 
                        width: 12, 
                        height: 12 
                    }
                    if tab.is_pinned { "Unpin Tab" } else { "Pin Tab" }
                }
                
                // SEPARATOR 1: Between tab management and layout actions
                div {
                    class: "menu-separator",
                    style: "
                        margin: 4px 8px;
                        height: 1px;
                        background: var(--vscode-menu-separatorBackground, #454545);
                    ",
                    role: "separator"
                }
                
                // Split Right
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            split_tab_right(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    Icon { icon: fa_solid_icons::FaArrowRight, width: 12, height: 12 }
                    "Split Right"
                }
                
                // Split Down
                div {
                    class: "menu-item",
                    style: "
                        padding: 8px 12px;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        border-radius: 2px;
                        transition: background-color 0.1s ease;
                    ",
                    role: "menuitem",
                    onclick: {
                        let mut editor_state = editor_state.clone();
                        let tab_index = context_menu.tab_index;
                        let group_index = context_menu.group_index;
                        move |_| {
                            split_tab_down(editor_state.clone(), group_index, tab_index);
                        }
                    },
                    Icon { icon: fa_solid_icons::FaArrowDown, width: 12, height: 12 }
                    "Split Down"
                }
                
                // Copy Path (if file tab)
                if tab.file_path.is_some() {
                    // SEPARATOR 2: Between actions and utilities (only if file tab)
                    div {
                        class: "menu-separator",
                        style: "
                            margin: 4px 8px;
                            height: 1px;
                            background: var(--vscode-menu-separatorBackground, #454545);
                        ",
                        role: "separator"
                    }
                    div {
                        class: "menu-item",
                        style: "
                            padding: 6px 12px;
                            cursor: pointer;
                            display: flex;
                            align-items: center;
                            gap: 8px;
                        ",
                        role: "menuitem",
                        onclick: {
                            let file_path = tab.file_path.clone();
                            move |_| {
                                if let Some(path) = &file_path {
                                    // TODO: Copy to clipboard functionality
                                    tracing::info!("Copy path: {}", path.display());
                                }
                            }
                        },
                        Icon { icon: fa_solid_icons::FaCopy, width: 12, height: 12 }
                        "Copy Path"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

/// Editor Tab Content Component - renders the content of a tab
#[component]
pub fn EditorTabContentComponent(
    tab: crate::state::EditorTab,
    group_index: usize,
) -> Element {
    let app_state = crate::state::use_app_state();
    rsx! {
        div {
            class: "editor-tab-content",
            style: "
                width: 100%;
                height: 100%;
                padding: 16px;
                overflow: auto;
                background: var(--vscode-background, #1e1e1e);
                color: var(--vscode-foreground, #cccccc);
            ",
            id: format!("tabpanel-{}-{}", group_index, tab.id),
            role: "tabpanel",
            tabindex: "0",
            
            match &tab.tab_type {
                crate::state::TabType::Welcome => rsx! {
                    div {
                        class: "welcome-content",
                        style: "text-align: center; margin-top: 40px;",
                        h1 { style: "margin-bottom: 16px;", "Welcome to MediaOrganizer" }
                        p { style: "margin-bottom: 8px;", "A VS Code-style media and file management application" }
                        p { "Select files from the explorer to open them in tabs" }
                    }
                },
                crate::state::TabType::FileEditor { content } => rsx! {
                    div {
                        class: "file-editor-content",
                        style: "
                            font-family: var(--vscode-font-mono);
                            white-space: pre-wrap;
                            line-height: 1.4;
                        ",
                        "{content}"
                    }
                },
                crate::state::TabType::Preview { preview_type } => rsx! {
                    div {
                        class: "preview-content",
                        style: "
                            width: 100%;
                            height: 100%;
                            display: flex;
                            flex-direction: column;
                        ",
                        
                        // Use the PreviewPanel component with real preview data from app state
                        PreviewPanel {
                            selected_file: Signal::new(tab.file_path.as_ref().map(|path| {
                                FileSystemEntry {
                                    path: path.clone(),
                                    name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                                    is_directory: false,
                                    size: 0, // Will be filled by actual file system
                                    modified: std::time::SystemTime::UNIX_EPOCH, // Will be filled by actual file system
                                    file_type: match preview_type {
                                        crate::state::PreviewType::Image => Some("image".to_string()),
                                        crate::state::PreviewType::Video => Some("video".to_string()),
                                        crate::state::PreviewType::Audio => Some("audio".to_string()),
                                        crate::state::PreviewType::Pdf => Some("pdf".to_string()),
                                        crate::state::PreviewType::Archive => Some("archive".to_string()),
                                        _ => None,
                                    }
                                }
                            })),
                            preview_data: app_state.preview_data, // Connected to real preview data from AppState
                        }
                    }
                },
                crate::state::TabType::Settings => rsx! {
                    div {
                        class: "settings-content",
                        h2 { style: "margin-bottom: 16px;", "Settings" }
                        p { "Settings panel coming soon..." }
                    }
                },
                crate::state::TabType::SearchResults => rsx! {
                    div {
                        class: "search-results-content",
                        h2 { style: "margin-bottom: 16px;", "Search Results" }
                        p { "Search results will be displayed here..." }
                    }
                },
            }
        }
    }
}

/// Helper function to get the appropriate icon for different tab types
fn get_tab_icon(tab_type: &crate::state::TabType) -> Element {
    match tab_type {
        crate::state::TabType::Welcome => rsx! {
            Icon {
                width: 14,
                height: 14,
                fill: "currentColor",
                icon: fa_solid_icons::FaHouse,
            }
        },
        crate::state::TabType::FileEditor { .. } => rsx! {
            Icon {
                width: 14,
                height: 14,
                fill: "currentColor",
                icon: fa_solid_icons::FaFile,
            }
        },
        crate::state::TabType::Preview { preview_type } => {
            match preview_type {
                crate::state::PreviewType::Image => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaImage,
                    }
                },
                crate::state::PreviewType::Video => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaVideo,
                    }
                },
                crate::state::PreviewType::Audio => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaMusic,
                    }
                },
                crate::state::PreviewType::Pdf => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaFilePdf,
                    }
                },
                crate::state::PreviewType::Archive => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaFileZipper,
                    }
                },
                crate::state::PreviewType::Unknown => rsx! {
                    Icon {
                        width: 14,
                        height: 14,
                        fill: "currentColor",
                        icon: fa_solid_icons::FaQuestion,
                    }
                },
            }
        },
        crate::state::TabType::Settings => rsx! {
            Icon {
                width: 14,
                height: 14,
                fill: "currentColor",
                icon: fa_solid_icons::FaGear,
            }
        },
        crate::state::TabType::SearchResults => rsx! {
            Icon {
                width: 14,
                height: 14,
                fill: "currentColor",
                icon: fa_solid_icons::FaMagnifyingGlass,
            }
        },
    }
}

/// Helper function to generate panel tabs
fn get_panel_tabs(panel_state: Signal<crate::state::PanelState>, active_tab: crate::state::PanelTab) -> Element {
    let tabs = vec![
        (crate::state::PanelTab::Problems, "PROBLEMS", "problems-panel"),
        (crate::state::PanelTab::Output, "OUTPUT", "output-panel"),
        (crate::state::PanelTab::Terminal, "TERMINAL", "terminal-panel"),
        (crate::state::PanelTab::Debug, "DEBUG", "debug-panel"),
    ];
    
    rsx! {
        for (tab_type, label, panel_id) in tabs {
            PanelTabButton {
                key: "{label}",
                tab_type: tab_type.clone(),
                label: label.to_string(),
                panel_id: panel_id.to_string(),
                is_active: active_tab == tab_type,
                panel_state: panel_state,
            }
        }
    }
}

/// Individual panel tab button component
#[component]
fn PanelTabButton(
    tab_type: crate::state::PanelTab,
    label: String,
    panel_id: String,
    is_active: bool,
    panel_state: Signal<crate::state::PanelState>,
) -> Element {
    let tab_style = format!(
        "
            padding: 0 12px;
            height: 100%;
            display: flex;
            align-items: center;
            background: {};
            color: {};
            border: none;
            border-right: 1px solid var(--vscode-border, #464647);
            cursor: pointer;
            outline: none;
        ",
        if is_active { 
            "var(--vscode-tab-active-background, #1e1e1e)" 
        } else { 
            "transparent" 
        },
        if is_active { 
            "var(--vscode-foreground, #cccccc)" 
        } else { 
            "var(--vscode-text-secondary, #999999)" 
        }
    );
    
    rsx! {
        button {
            class: format!("panel-tab {}", if is_active { "active" } else { "" }),
            style: tab_style,
            role: "tab",
            "aria-selected": if is_active { "true" } else { "false" },
            "aria-controls": panel_id,
            tabindex: if is_active { "0" } else { "-1" },
            onclick: {
                let mut panel_state = panel_state.clone();
                let tab_type = tab_type.clone();
                move |_| {
                    let mut panel = panel_state.write();
                    panel.active_tab = tab_type.clone();
                    tracing::info!("Switched to panel tab: {:?}", tab_type);
                }
            },
            "{label}"
        }
    }
}

/// Helper function to get panel content based on active tab
fn get_panel_content(active_tab: crate::state::PanelTab) -> Element {
    rsx! {
        match active_tab {
            crate::state::PanelTab::Problems => rsx! {
                div {
                    id: "problems-panel",
                    "aria-labelledby": "problems-tab",
                    h3 { style: "margin-bottom: 8px;", "Problems" }
                    p { "No problems found in the workspace." }
                }
            },
            crate::state::PanelTab::Output => rsx! {
                div {
                    id: "output-panel",
                    "aria-labelledby": "output-tab",
                    h3 { style: "margin-bottom: 8px;", "Output" }
                    pre {
                        style: "
                            font-family: var(--vscode-font-mono);
                            font-size: var(--vscode-font-size-small);
                            line-height: 1.4;
                            margin: 0;
                        ",
                        "MediaOrganizer v0.1.0\n"
                        "Initialized VS Code-style interface\n"
                        "Ready for file operations...\n"
                    }
                }
            },
            crate::state::PanelTab::Terminal => rsx! {
                div {
                    id: "terminal-panel",
                    "aria-labelledby": "terminal-tab",
                    h3 { style: "margin-bottom: 8px;", "Terminal" }
                    div {
                        style: "
                            background: var(--vscode-terminal-background, #1e1e1e);
                            border: 1px solid var(--vscode-border, #464647);
                            border-radius: 4px;
                            padding: 12px;
                            font-family: var(--vscode-font-mono);
                            font-size: var(--vscode-font-size-normal);
                            color: var(--vscode-terminal-foreground, #ffffff);
                        ",
                        p { "Terminal integration coming soon..." }
                        p { style: "margin-top: 8px; opacity: 0.7;", "$ " }
                    }
                }
            },
            crate::state::PanelTab::Debug => rsx! {
                div {
                    id: "debug-panel",
                    "aria-labelledby": "debug-tab",
                    h3 { style: "margin-bottom: 8px;", "Debug Console" }
                    p { "Debug console will be available when debugging is active." }
                }
            },
        }
    }
}

/// Helper function to get the appropriate icon for Activity Bar items
fn get_activity_bar_icon(icon_name: &str) -> Element {
    match icon_name {
        "files" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaFolder,
            }
        },
        "search" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaMagnifyingGlass,
            }
        },
        "source-control" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaCodeBranch,
            }
        },
        "debug-alt" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaBug,
            }
        },
        "extensions" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaPuzzlePiece,
            }
        },
        "settings-gear" => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaGear,
            }
        },
        _ => rsx! {
            Icon {
                width: 16,
                height: 16,
                fill: "currentColor",
                icon: fa_solid_icons::FaQuestion,
            }
        },
    }
}

/// Global mouse move handler for panel resizing
fn handle_global_mouse_move(evt: Event<MouseData>, mut panel_state: Signal<crate::state::PanelState>) {
    let resize_state = get_global_resize_state();
    
    // Only process if we're actively resizing the panel
    if !resize_state.is_panel_resizing {
        return;
    }
    
    let current_y = evt.data.client_coordinates().y as f64;
    
    // Calculate the height delta (negative because drag handle is at top)
    // Moving mouse up (decreasing Y) should increase panel height
    // Moving mouse down (increasing Y) should decrease panel height  
    let height_delta = resize_state.resize_start_y - current_y;
    let new_height_raw = resize_state.resize_start_height + height_delta;
    
    // Get panel constraints and apply them
    let mut panel = panel_state.write();
    let min_height = panel.min_height;
    
    // For desktop app, use a reasonable fallback for max height calculation
    // Assume a typical viewport height of 800px minimum, but allow larger panels
    let estimated_viewport_height = 800.0_f64.max(resize_state.resize_start_height * 2.5); 
    let max_height = estimated_viewport_height * panel.max_height_fraction;
    
    let new_height = new_height_raw.max(min_height).min(max_height);
    
    // Only update if the height actually changed significantly
    if (panel.height - new_height).abs() > 1.0 {
        panel.height = new_height;
        tracing::debug!("Panel height updated to: {:.1}px (constrained: {:.1}-{:.1})", new_height, min_height, max_height);
    }
}

/// Global mouse up handler to end panel resizing
fn handle_global_mouse_up(panel_state: Signal<crate::state::PanelState>) {
    let resize_state = get_global_resize_state();
    
    // Only process if we were actually resizing
    if resize_state.is_panel_resizing {
        // Clear the resize state
        resize_state.is_panel_resizing = false;
        resize_state.resize_start_y = 0.0;
        resize_state.resize_start_height = 0.0;
        
        let panel_height = panel_state.read().height;
        tracing::info!("Panel resizing ended, final height: {}px", panel_height);
    }
}

/// Handle tab drop for reordering and moving tabs between groups
fn handle_tab_drop(evt: Event<DragData>, mut editor_state: Signal<crate::state::EditorState>, target_group_index: usize) {
    let mut editor_data = editor_state.write();
    
    // Get the drag operation
    let drag_op = match &editor_data.drag_operation {
        Some(op) => op.clone(),
        None => {
            tracing::warn!("No drag operation in progress");
            return;
        }
    };
    
    let source_group_index = drag_op.source_group_id;
    let source_tab_index = drag_op.tab_index;
    
    // Calculate drop position based on mouse coordinates
    // For now, we'll append to the target group - a more sophisticated implementation
    // would calculate the exact insertion position based on mouse coordinates
    
    if source_group_index == target_group_index {
        // Reordering within the same group
        handle_tab_reorder_within_group(&mut editor_data, source_group_index, source_tab_index, evt);
    } else {
        // Moving between different groups
        handle_tab_move_between_groups(&mut editor_data, source_group_index, target_group_index, source_tab_index);
    }
}

/// Handle tab reordering within the same group
fn handle_tab_reorder_within_group(
    editor_data: &mut crate::state::EditorState,
    group_index: usize,
    source_tab_index: usize,
    _evt: Event<DragData>
) {
    // For now, we'll implement a simple reordering logic
    // A more sophisticated version would calculate exact drop position
    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
        if source_tab_index < group.tabs.len() {
            // For simplicity, move the tab to the end of the group
            let target_index = group.tabs.len() - 1;
            
            if source_tab_index != target_index {
                let tab = group.tabs.remove(source_tab_index);
                group.tabs.insert(target_index, tab);
                
                // Update active tab index if needed
                if group.active_tab == source_tab_index {
                    group.active_tab = target_index;
                } else if group.active_tab > source_tab_index && group.active_tab <= target_index {
                    group.active_tab -= 1;
                } else if group.active_tab < source_tab_index && group.active_tab >= target_index {
                    group.active_tab += 1;
                }
                
                // Update all tabs' active state
                for (i, tab) in group.tabs.iter_mut().enumerate() {
                    tab.is_active = i == group.active_tab;
                }
                
                tracing::info!("Reordered tab from index {} to {} within group {}", source_tab_index, target_index, group_index);
            }
        }
    }
}

/// Handle tab moving between different groups
fn handle_tab_move_between_groups(
    editor_data: &mut crate::state::EditorState,
    source_group_index: usize,
    target_group_index: usize,
    source_tab_index: usize
) {
    // Ensure both groups exist
    if source_group_index >= editor_data.editor_groups.len() || target_group_index >= editor_data.editor_groups.len() {
        tracing::warn!("Invalid group indices for tab move: source={}, target={}", source_group_index, target_group_index);
        return;
    }
    
    // Remove tab from source group
    let moved_tab = {
        let source_group = &mut editor_data.editor_groups[source_group_index];
        if source_tab_index >= source_group.tabs.len() {
            tracing::warn!("Invalid tab index {} in source group {}", source_tab_index, source_group_index);
            return;
        }
        
        let tab = source_group.tabs.remove(source_tab_index);
        
        // Update source group's active tab
        if source_group.tabs.is_empty() {
            // If no tabs left, we might need to handle this case
            source_group.active_tab = 0;
        } else if source_group.active_tab >= source_tab_index && source_group.active_tab > 0 {
            source_group.active_tab -= 1;
        }
        
        // Update active states in source group
        for (i, t) in source_group.tabs.iter_mut().enumerate() {
            t.is_active = i == source_group.active_tab;
        }
        
        tab
    };
    
    // Add tab to target group
    {
        let target_group = &mut editor_data.editor_groups[target_group_index];
        
        // Add to end of target group
        target_group.tabs.push(moved_tab);
        let new_tab_index = target_group.tabs.len() - 1;
        
        // Make the moved tab active in the target group
        target_group.active_tab = new_tab_index;
        
        // Update active states in target group
        for (i, tab) in target_group.tabs.iter_mut().enumerate() {
            tab.is_active = i == new_tab_index;
        }
        
        // Set target group as the active group
        editor_data.active_group = target_group_index;
    }
    
    tracing::info!("Moved tab from group {} index {} to group {} (new index: {})", 
        source_group_index, source_tab_index, target_group_index, 
        editor_data.editor_groups[target_group_index].tabs.len() - 1);
}

// Context menu action helper functions

/// Close a specific tab
fn close_tab(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
        if group.tabs.len() > 1 && tab_index < group.tabs.len() {
            group.tabs.remove(tab_index);
            // Adjust active tab if necessary
            if group.active_tab >= group.tabs.len() {
                group.active_tab = group.tabs.len().saturating_sub(1);
            }
            // Update active states
            for (i, t) in group.tabs.iter_mut().enumerate() {
                t.is_active = i == group.active_tab;
            }
            tracing::info!("Closed tab {} in group {} via context menu", tab_index, group_index);
        }
    }
}

/// Close all tabs except the specified one
fn close_others(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, keep_tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
        if keep_tab_index < group.tabs.len() {
            let kept_tab = group.tabs[keep_tab_index].clone();
            group.tabs = vec![kept_tab];
            group.active_tab = 0;
            group.tabs[0].is_active = true;
            tracing::info!("Closed all other tabs in group {}, kept tab {}", group_index, keep_tab_index);
        }
    }
}

/// Close all tabs to the right of the specified tab
fn close_tabs_to_right(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
        if tab_index < group.tabs.len() && tab_index < group.tabs.len() - 1 {
            let keep_count = tab_index + 1;
            group.tabs.truncate(keep_count);
            
            // Adjust active tab if it was removed
            if group.active_tab >= keep_count {
                group.active_tab = keep_count - 1;
            }
            
            // Update active states
            for (i, t) in group.tabs.iter_mut().enumerate() {
                t.is_active = i == group.active_tab;
            }
            
            tracing::info!("Closed tabs to the right of tab {} in group {}", tab_index, group_index);
        }
    }
}

/// Toggle pin status of a tab
fn toggle_pin_tab(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get_mut(group_index) {
        if let Some(tab) = group.tabs.get_mut(tab_index) {
            tab.is_pinned = !tab.is_pinned;
            tracing::info!("Toggled pin for tab {} in group {}: {}", tab_index, group_index, tab.is_pinned);
        }
    }
}

/// Split tab to the right (create new editor group)
fn split_tab_right(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get(group_index) {
        if let Some(tab) = group.tabs.get(tab_index) {
            // Create a duplicate tab for the new group
            let mut new_tab = tab.clone();
            new_tab.id = editor_data.next_tab_id;
            new_tab.is_active = true;
            editor_data.next_tab_id += 1;
            
            // Create new editor group
            let new_group = crate::state::EditorGroup {
                id: editor_data.editor_groups.len() + 1,
                tabs: vec![new_tab],
                active_tab: 0,
                layout_position: crate::state::EditorGroupPosition {
                    x: 0.5,
                    y: 0.0,
                    width: 0.5,
                    height: 1.0,
                },
            };
            
            editor_data.editor_groups.push(new_group);
            editor_data.layout_config = crate::state::EditorLayoutConfig::SplitHorizontal;
            editor_data.active_group = editor_data.editor_groups.len() - 1;
            
            tracing::info!("Split tab {} from group {} to new group on the right", tab_index, group_index);
        }
    }
}

/// Split tab down (create new editor group below)
fn split_tab_down(mut editor_state: Signal<crate::state::EditorState>, group_index: usize, tab_index: usize) {
    let mut editor_data = editor_state.write();
    editor_data.context_menu = None; // Close context menu
    
    if let Some(group) = editor_data.editor_groups.get(group_index) {
        if let Some(tab) = group.tabs.get(tab_index) {
            // Create a duplicate tab for the new group
            let mut new_tab = tab.clone();
            new_tab.id = editor_data.next_tab_id;
            new_tab.is_active = true;
            editor_data.next_tab_id += 1;
            
            // Create new editor group
            let new_group = crate::state::EditorGroup {
                id: editor_data.editor_groups.len() + 1,
                tabs: vec![new_tab],
                active_tab: 0,
                layout_position: crate::state::EditorGroupPosition {
                    x: 0.0,
                    y: 0.5,
                    width: 1.0,
                    height: 0.5,
                },
            };
            
            editor_data.editor_groups.push(new_group);
            editor_data.layout_config = crate::state::EditorLayoutConfig::SplitVertical;
            editor_data.active_group = editor_data.editor_groups.len() - 1;
            
            tracing::info!("Split tab {} from group {} to new group below", tab_index, group_index);
        }
    }
}