use dioxus::prelude::*;

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
            
            // Activity Bar Items
            div {
                class: "activity-bar-items",
                style: "display: flex; flex-direction: column; flex: 1;",
                
                ActivityBarItem {
                    icon: "files",
                    label: "Explorer",
                    active: true,
                    on_click: move |_| {
                        // TODO: Switch to explorer view
                    }
                }
                
                ActivityBarItem {
                    icon: "search",
                    label: "Search",
                    active: false,
                    on_click: move |_| {
                        // TODO: Switch to search view
                    }
                }
                
                ActivityBarItem {
                    icon: "source-control",
                    label: "Source Control",
                    active: false,
                    on_click: move |_| {
                        // TODO: Switch to source control view
                    }
                }
                
                ActivityBarItem {
                    icon: "debug-alt",
                    label: "Run and Debug",
                    active: false,
                    on_click: move |_| {
                        // TODO: Switch to debug view
                    }
                }
                
                ActivityBarItem {
                    icon: "extensions",
                    label: "Extensions",
                    active: false,
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
    on_click: EventHandler<MouseEvent>,
) -> Element {
    let item_style = if active {
        "
            width: 48px;
            height: 48px;
            border: none;
            background: var(--vscode-list-activeSelectionBackground, rgba(0, 122, 204, 0.3));
            color: var(--vscode-foreground, #cccccc);
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            position: relative;
        "
    } else {
        "
            width: 48px;
            height: 48px;
            border: none;
            background: transparent;
            color: var(--vscode-foreground, #cccccc);
            display: flex;
            align-items: center;
            justify-content: center;
            cursor: pointer;
            position: relative;
        "
    };
    
    rsx! {
        button {
            class: "activity-bar-item",
            style: item_style,
            title: label.clone(),
            "aria-label": label.clone(),
            onclick: move |evt| on_click.call(evt),
            
            // Icon placeholder (will be replaced with actual icons later)
            div {
                class: "codicon",
                style: "
                    width: 16px;
                    height: 16px;
                    background: currentColor;
                    mask: url('/assets/icons/{icon}.svg') no-repeat center;
                    mask-size: contain;
                ",
                "aria-hidden": "true"
            }
            
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
        div {
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
            "aria-label": "File explorer",
            
            // Sidebar Header
            div {
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
        div {
            class: "editor-groups",
            style: "
                grid-area: editor;
                background: var(--vscode-background, #1e1e1e);
                display: flex;
                flex-direction: column;
            ",
            role: "main",
            "aria-label": "Editor area",
            
            // Tab Bar
            div {
                class: "tab-bar",
                style: "
                    height: 35px;
                    display: flex;
                    background: var(--vscode-tab-inactive-background, #2d2d30);
                    border-bottom: 1px solid var(--vscode-border, #464647);
                ",
                role: "tablist",
                
                // Sample tab
                div {
                    class: "tab",
                    style: "
                        padding: 0 12px;
                        display: flex;
                        align-items: center;
                        background: var(--vscode-tab-active-background, #1e1e1e);
                        color: var(--vscode-foreground, #cccccc);
                        border-right: 1px solid var(--vscode-border, #464647);
                        cursor: pointer;
                        min-width: 120px;
                        max-width: 240px;
                    ",
                    role: "tab",
                    "aria-selected": "true",
                    
                    "Welcome"
                }
            }
            
            // Editor Content
            div {
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
        div {
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
            "aria-label": "Panel",
            
            // Panel Header
            div {
                class: "panel-header",
                style: "
                    height: 35px;
                    display: flex;
                    align-items: center;
                    background: var(--vscode-tab-inactive-background, #2d2d30);
                    border-bottom: 1px solid var(--vscode-border, #464647);
                ",
                
                // Panel tabs
                div {
                    class: "panel-tab active",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        background: var(--vscode-tab-active-background, #1e1e1e);
                        color: var(--vscode-foreground, #cccccc);
                        border-right: 1px solid var(--vscode-border, #464647);
                        cursor: pointer;
                    ",
                    "PROBLEMS"
                }
                
                div {
                    class: "panel-tab",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        color: var(--vscode-text-secondary, #999999);
                        cursor: pointer;
                    ",
                    "OUTPUT"
                }
                
                div {
                    class: "panel-tab",
                    style: "
                        padding: 0 12px;
                        height: 100%;
                        display: flex;
                        align-items: center;
                        color: var(--vscode-text-secondary, #999999);
                        cursor: pointer;
                    ",
                    "TERMINAL"
                }
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
        div {
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
            "aria-label": "Application status",
            
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