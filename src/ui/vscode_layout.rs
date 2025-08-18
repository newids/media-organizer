use dioxus::prelude::*;
use dioxus::events::KeyboardEvent;
use dioxus_free_icons::{Icon, icons::fa_solid_icons};

/// Main VS Code-like layout component
/// Implements the grid-based layout system with Activity Bar, Sidebar, Editor Groups, Panel, and Status Bar
#[component]
pub fn VSCodeLayout(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    rsx! {
        div {
            class: "vscode-layout",
            id: "vscode-layout",
            style: "
                display: grid;
                grid-template-areas: 
                    'activity sidebar editor'
                    'activity sidebar panel'
                    'status status status';
                grid-template-columns: 48px 240px 1fr;
                grid-template-rows: 1fr 200px 22px;
                height: 100vh;
                width: 100vw;
            ",
            role: "application",
            "aria-label": "MediaOrganizer - VS Code style interface",
            
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
    let mut focused_item = use_signal(|| 0usize); // Track which item is focused
    
    rsx! {
        div {
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
            tabindex: "0",
            onkeydown: move |evt| {
                match evt.data.key() {
                    Key::ArrowDown => {
                        evt.prevent_default();
                        let current = *focused_item.read();
                        focused_item.set((current + 1).min(5)); // 6 total items (0-5)
                    },
                    Key::ArrowUp => {
                        evt.prevent_default();
                        let current = *focused_item.read();
                        focused_item.set(current.saturating_sub(1));
                    },
                    Key::Enter => {
                        evt.prevent_default();
                        // TODO: Activate focused item
                    },
                    Key::Character(s) if s == " " => {
                        evt.prevent_default();
                        // TODO: Activate focused item
                    },
                    _ => {}
                }
            },
            
            // Activity Bar Items
            div {
                class: "activity-bar-items",
                style: "display: flex; flex-direction: column; flex: 1;",
                
                ActivityBarItem {
                    icon: "files",
                    label: "Explorer",
                    active: true,
                    focused: *focused_item.read() == 0,
                    item_index: 0,
                    on_click: move |_| {
                        // TODO: Switch to explorer view
                    }
                }
                
                ActivityBarItem {
                    icon: "search",
                    label: "Search",
                    active: false,
                    focused: *focused_item.read() == 1,
                    item_index: 1,
                    on_click: move |_| {
                        // TODO: Switch to search view
                    }
                }
                
                ActivityBarItem {
                    icon: "source-control",
                    label: "Source Control",
                    active: false,
                    focused: *focused_item.read() == 2,
                    item_index: 2,
                    on_click: move |_| {
                        // TODO: Switch to source control view
                    }
                }
                
                ActivityBarItem {
                    icon: "debug-alt",
                    label: "Run and Debug",
                    active: false,
                    focused: *focused_item.read() == 3,
                    item_index: 3,
                    on_click: move |_| {
                        // TODO: Switch to debug view
                    }
                }
                
                ActivityBarItem {
                    icon: "extensions",
                    label: "Extensions",
                    active: false,
                    focused: *focused_item.read() == 4,
                    item_index: 4,
                    on_click: move |_| {
                        // TODO: Switch to extensions view
                    }
                }
            }
            
            // Bottom items (Settings, etc.)
            div {
                class: "activity-bar-footer",
                style: "display: flex; flex-direction: column;",
                
                ActivityBarItem {
                    icon: "settings-gear",
                    label: "Settings",
                    active: false,
                    focused: *focused_item.read() == 5,
                    item_index: 5,
                    on_click: move |_| {
                        // TODO: Open settings
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
            tabindex: "-1", // Managed by parent container
            onclick: move |evt| on_click.call(evt),
            
            // Icon using dioxus-free-icons
            {get_activity_bar_icon(&icon)}
            
            // Active indicator
            if active {
                div {
                    class: "activity-bar-indicator",
                    style: "
                        position: absolute;
                        left: 0;
                        top: 50%;
                        transform: translateY(-50%);
                        width: 2px;
                        height: 16px;
                        background: var(--vscode-accent, #007acc);
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
    rsx! {
        aside {
            class: "sidebar",
            style: "
                grid-area: sidebar;
                background: var(--vscode-sidebar-background, #252526);
                border-right: 1px solid var(--vscode-border, #464647);
                display: flex;
                flex-direction: column;
                width: 240px;
            ",
            role: "complementary",
            "aria-label": "File explorer sidebar",
            "aria-labelledby": "sidebar-header",
            tabindex: "0",
            
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
                    font-size: 11px;
                    font-weight: bold;
                    color: var(--vscode-foreground, #cccccc);
                    text-transform: uppercase;
                ",
                role: "banner",
                "EXPLORER"
            }
            
            // Sidebar Content
            div {
                class: "sidebar-content",
                style: "
                    flex: 1;
                    overflow: hidden;
                    padding: 8px 0;
                ",
                
                // File tree will go here
                div {
                    class: "file-tree-placeholder",
                    style: "padding: 16px; color: var(--vscode-foreground, #cccccc);",
                    "File tree component will be integrated here"
                }
            }
        }
    }
}

/// Editor Groups component - tabbed interface for content
#[component]
pub fn EditorGroups(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    rsx! {
        main {
            class: "editor-groups",
            style: "
                grid-area: editor;
                background: var(--vscode-background, #1e1e1e);
                display: flex;
                flex-direction: column;
            ",
            role: "main",
            "aria-label": "Main editor area",
            "aria-labelledby": "editor-tab-list",
            tabindex: "0",
            
            // Tab Bar
            div {
                id: "editor-tab-list",
                class: "tab-bar",
                style: "
                    height: 35px;
                    display: flex;
                    background: var(--vscode-tab-inactive-background, #2d2d30);
                    border-bottom: 1px solid var(--vscode-border, #464647);
                ",
                role: "tablist",
                "aria-label": "Editor tabs",
                
                // Sample tab
                button {
                    class: "tab",
                    style: "
                        padding: 0 12px;
                        display: flex;
                        align-items: center;
                        background: var(--vscode-tab-active-background, #1e1e1e);
                        color: var(--vscode-foreground, #cccccc);
                        border: none;
                        border-right: 1px solid var(--vscode-border, #464647);
                        cursor: pointer;
                        min-width: 120px;
                        max-width: 240px;
                        outline: none;
                    ",
                    role: "tab",
                    "aria-selected": "true",
                    "aria-controls": "welcome-panel",
                    tabindex: "0",
                    
                    "Welcome"
                }
            }
            
            // Editor Content
            div {
                id: "welcome-panel",
                class: "editor-content",
                style: "
                    flex: 1;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    background: var(--vscode-background, #1e1e1e);
                    color: var(--vscode-foreground, #cccccc);
                ",
                role: "tabpanel",
                "aria-labelledby": "welcome-tab",
                tabindex: "0",
                
                div {
                    class: "welcome-content",
                    style: "text-align: center;",
                    
                    h2 { "MediaOrganizer" }
                    p { "VS Code-like interface is being constructed..." }
                    p { 
                        style: "color: var(--vscode-text-secondary, #999999);",
                        "Preview panels and file content will appear here"
                    }
                }
            }
        }
    }
}

/// Panel component - bottom panel for terminal, problems, etc.
#[component]
pub fn Panel(
    app_state: Signal<crate::state::AppState>,
) -> Element {
    rsx! {
        aside {
            class: "panel",
            style: "
                grid-area: panel;
                background: var(--vscode-sidebar-background, #252526);
                border-top: 1px solid var(--vscode-border, #464647);
                display: flex;
                flex-direction: column;
                height: 200px;
            ",
            role: "complementary",
            "aria-label": "Bottom panel with terminal and output",
            "aria-labelledby": "panel-header",
            tabindex: "0",
            
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
                
                // Panel tabs
                button {
                    class: "panel-tab active",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        background: var(--vscode-tab-active-background, #1e1e1e);
                        color: var(--vscode-foreground, #cccccc);
                        border: none;
                        border-right: 1px solid var(--vscode-border, #464647);
                        cursor: pointer;
                        outline: none;
                    ",
                    role: "tab",
                    "aria-selected": "true",
                    "aria-controls": "problems-panel",
                    tabindex: "0",
                    "PROBLEMS"
                }
                
                button {
                    class: "panel-tab",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        background: transparent;
                        color: var(--vscode-text-secondary, #999999);
                        border: none;
                        cursor: pointer;
                        outline: none;
                    ",
                    role: "tab",
                    "aria-selected": "false",
                    "aria-controls": "output-panel",
                    tabindex: "-1",
                    "OUTPUT"
                }
                
                button {
                    class: "panel-tab",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        background: transparent;
                        color: var(--vscode-text-secondary, #999999);
                        border: none;
                        cursor: pointer;
                        outline: none;
                    ",
                    role: "tab",
                    "aria-selected": "false",
                    "aria-controls": "terminal-panel",
                    tabindex: "-1",
                    "TERMINAL"
                }
            }
            
            // Panel Content
            div {
                id: "problems-panel",
                class: "panel-content",
                style: "
                    flex: 1;
                    padding: 8px;
                    color: var(--vscode-foreground, #cccccc);
                    overflow-y: auto;
                ",
                role: "tabpanel",
                "aria-labelledby": "problems-tab",
                tabindex: "0",
                
                "Panel content area - problems, output, and terminal will be displayed here"
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
                font-size: 12px;
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