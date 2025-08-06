use dioxus::prelude::*;
use crate::state::{AppState, SelectionMode};
use std::path::PathBuf;

#[component]
pub fn App() -> Element {
    let mut app_state = use_signal(|| AppState::new());
    
    // Load initial directory contents
    use_effect(move || {
        let app_state = app_state.read();
        spawn(async move {
            let current_path = app_state.get_current_path();
            if let Err(e) = app_state.load_directory_contents(current_path).await {
                tracing::warn!("Failed to load initial directory: {}", e);
            }
        });
    });

    let navigate_to_directory = move |path: PathBuf| {
        let app_state = app_state.read().clone();
        spawn(async move {
            if let Err(e) = app_state.navigate_to(path).await {
                tracing::warn!("Failed to navigate: {}", e);
            }
        });
    };

    let navigate_back = move |_| {
        let app_state = app_state.read().clone();
        spawn(async move {
            if let Err(e) = app_state.navigate_back().await {
                tracing::warn!("Failed to navigate back: {}", e);
            }
        });
    };

    let navigate_forward = move |_| {
        let app_state = app_state.read().clone();
        spawn(async move {
            if let Err(e) = app_state.navigate_forward().await {
                tracing::warn!("Failed to navigate forward: {}", e);
            }
        });
    };

    let navigate_up = move |_| {
        let app_state = app_state.read().clone();
        spawn(async move {
            if let Err(e) = app_state.navigate_up().await {
                tracing::warn!("Failed to navigate up: {}", e);
            }
        });
    };
    render! {
        div {
            class: "app-container",
            style: "
                height: 100vh;
                display: flex;
                flex-direction: column;
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
                background-color: #1e1e1e;
                color: #cccccc;
            ",

            // Top Bar
            div {
                class: "top-bar",
                style: "
                    height: 40px;
                    background: #2d2d30;
                    border-bottom: 1px solid #3e3e42;
                    display: flex;
                    align-items: center;
                    padding: 0 12px;
                ",

                div {
                    class: "app-title",
                    style: "
                        font-size: 14px;
                        font-weight: 600;
                        color: #cccccc;
                    ",
                    "MediaOrganizer"
                }

                div {
                    class: "spacer",
                    style: "flex: 1;"
                }

                div {
                    class: "window-controls",
                    style: "
                        display: flex;
                        gap: 8px;
                    ",
                    "⚙️ Settings"
                }
            }

            // Main Content Area
            div {
                class: "main-content",
                style: "
                    flex: 1;
                    display: flex;
                    overflow: hidden;
                ",

                // Left Panel - File Tree
                div {
                    class: "left-panel",
                    style: "
                        width: 300px;
                        background: #252526;
                        border-right: 1px solid #3e3e42;
                        padding: 8px;
                    ",

                    div {
                        class: "panel-header",
                        style: "
                            font-size: 12px;
                            font-weight: 600;
                            color: #cccccc;
                            margin-bottom: 8px;
                            text-transform: uppercase;
                        ",
                        "Explorer"
                    }

                    div {
                        class: "file-tree",
                        style: "
                            font-size: 13px;
                            line-height: 22px;
                        ",

                        // Home
                        div {
                            class: "folder-item",
                            style: "
                                display: flex;
                                align-items: center;
                                padding: 4px 8px;
                                cursor: pointer;
                                border-radius: 2px;
                                margin-bottom: 2px;
                            ",
                            onclick: move |_| {
                                spawn({
                                    let app_state = app_state.clone();
                                    async move {
                                        if let Ok(home_path) = app_state.file_service.get_home_directory().await {
                                            let _ = app_state.navigate_to(home_path).await;
                                        }
                                    }
                                });
                            },

                            span {
                                class: "folder-icon",
                                style: "margin-right: 8px;",
                                "🏠"
                            }
                            span { "Home" }
                        }

                        // Documents
                        div {
                            class: "folder-item",
                            style: "
                                display: flex;
                                align-items: center;
                                padding: 4px 8px;
                                cursor: pointer;
                                border-radius: 2px;
                                margin-bottom: 2px;
                            ",
                            onclick: move |_| {
                                spawn({
                                    let app_state = app_state.clone();
                                    async move {
                                        if let Ok(docs_path) = app_state.file_service.get_documents_directory().await {
                                            let _ = app_state.navigate_to(docs_path).await;
                                        }
                                    }
                                });
                            },

                            span {
                                class: "folder-icon",
                                style: "margin-right: 8px;",
                                "🗂️"
                            }
                            span { "Documents" }
                        }

                        // Desktop  
                        div {
                            class: "folder-item",
                            style: "
                                display: flex;
                                align-items: center;
                                padding: 4px 8px;
                                cursor: pointer;
                                border-radius: 2px;
                                margin-bottom: 2px;
                            ",
                            onclick: move |_| {
                                spawn({
                                    let app_state = app_state.clone();
                                    async move {
                                        if let Ok(desktop_path) = app_state.file_service.get_desktop_directory().await {
                                            let _ = app_state.navigate_to(desktop_path).await;
                                        }
                                    }
                                });
                            },

                            span {
                                class: "folder-icon",
                                style: "margin-right: 8px;",
                                "💾"
                            }
                            span { "Desktop" }
                        }
                    }
                }

                // Right Panel - Content Viewer
                div {
                    class: "right-panel",
                    style: "
                        flex: 1;
                        background: #1e1e1e;
                        display: flex;
                        flex-direction: column;
                    ",

                    // Navigation Bar
                    div {
                        class: "navigation-bar",
                        style: "
                            height: 35px;
                            background: #2d2d30;
                            border-bottom: 1px solid #3e3e42;
                            display: flex;
                            align-items: center;
                            padding: 0 12px;
                            gap: 8px;
                        ",

                        // Navigation buttons
                        button {
                            class: "nav-button",
                            style: format!("
                                background: {};
                                border: 1px solid #3e3e42;
                                color: {};
                                padding: 4px 8px;
                                border-radius: 2px;
                                font-size: 12px;
                                cursor: {};
                                min-width: 24px;
                            ", 
                                if app_state.can_navigate_back() { "#0e639c" } else { "#3c3c3c" },
                                if app_state.can_navigate_back() { "white" } else { "#666" },
                                if app_state.can_navigate_back() { "pointer" } else { "default" }
                            ),
                            disabled: !app_state.can_navigate_back(),
                            onclick: navigate_back,
                            "←"
                        }

                        button {
                            class: "nav-button",
                            style: format!("
                                background: {};
                                border: 1px solid #3e3e42;
                                color: {};
                                padding: 4px 8px;
                                border-radius: 2px;
                                font-size: 12px;
                                cursor: {};
                                min-width: 24px;
                            ", 
                                if app_state.can_navigate_forward() { "#0e639c" } else { "#3c3c3c" },
                                if app_state.can_navigate_forward() { "white" } else { "#666" },
                                if app_state.can_navigate_forward() { "pointer" } else { "default" }
                            ),
                            disabled: !app_state.can_navigate_forward(),
                            onclick: navigate_forward,
                            "→"
                        }

                        button {
                            class: "nav-button",
                            style: format!("
                                background: {};
                                border: 1px solid #3e3e42;
                                color: {};
                                padding: 4px 8px;
                                border-radius: 2px;
                                font-size: 12px;
                                cursor: {};
                                min-width: 24px;
                            ", 
                                if app_state.can_navigate_up() { "#0e639c" } else { "#3c3c3c" },
                                if app_state.can_navigate_up() { "white" } else { "#666" },
                                if app_state.can_navigate_up() { "pointer" } else { "default" }
                            ),
                            disabled: !app_state.can_navigate_up(),
                            onclick: navigate_up,
                            "↑"
                        }

                        // Breadcrumb navigation
                        div {
                            class: "breadcrumbs",
                            style: "
                                flex: 1;
                                display: flex;
                                align-items: center;
                                gap: 4px;
                                margin-left: 12px;
                                font-size: 12px;
                                min-width: 0;
                            ",

                            app_state.get_breadcrumbs().iter().enumerate().map(|(i, breadcrumb)| rsx! {
                                if i > 0 {
                                    span {
                                        style: "color: #666; margin: 0 4px;",
                                        "/"
                                    }
                                }
                                
                                button {
                                    class: "breadcrumb-item",
                                    key: "{breadcrumb.path.display()}",
                                    style: "
                                        background: transparent;
                                        border: none;
                                        color: #cccccc;
                                        cursor: pointer;
                                        padding: 2px 4px;
                                        border-radius: 2px;
                                        font-size: 12px;
                                    ",
                                    onclick: {
                                        let path = breadcrumb.path.clone();
                                        move |_| navigate_to_directory(path.clone())
                                    },
                                    "{breadcrumb.name}"
                                }
                            })
                        }

                        // Search input
                        input {
                            class: "search-input",
                            placeholder: "Search files...",
                            style: "
                                background: #3c3c3c;
                                border: 1px solid #3e3e42;
                                color: #cccccc;
                                padding: 4px 8px;
                                border-radius: 2px;
                                font-size: 12px;
                                width: 200px;
                            "
                        }
                    }

                    // Content Area
                    div {
                        class: "content-area",
                        style: "
                            flex: 1;
                            overflow: auto;
                        ",

                        if app_state.is_current_directory_loading() {
                            div {
                                class: "loading-message",
                                style: "
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    height: 100%;
                                    color: #8c8c8c;
                                    font-size: 14px;
                                ",
                                "Loading..."
                            }
                        } else if let Some(files) = app_state.get_current_directory_contents() {
                            if files.is_empty() {
                                div {
                                    class: "empty-message",
                                    style: "
                                        display: flex;
                                        align-items: center;
                                        justify-content: center;
                                        height: 100%;
                                        color: #8c8c8c;
                                        font-size: 14px;
                                    ",
                                    "This folder is empty"
                                }
                            } else {
                                div {
                                    class: "file-grid",
                                    style: "
                                        display: grid;
                                        grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
                                        gap: 16px;
                                        padding: 16px;
                                    ",

                                    files.iter().map(|file| rsx! {
                                        div {
                                            class: "file-item",
                                            key: "{file.path.display()}",
                                            style: format!("
                                                display: flex;
                                                flex-direction: column;
                                                align-items: center;
                                                padding: 12px;
                                                border-radius: 4px;
                                                cursor: pointer;
                                                background: {};
                                                border: 1px solid {};
                                                transition: background-color 0.2s;
                                            ",
                                                if app_state.is_selected(&file.path) { "#264f78" } else { "transparent" },
                                                if app_state.is_selected(&file.path) { "#007acc" } else { "transparent" }
                                            ),
                                            onclick: {
                                                let path = file.path.clone();
                                                let is_dir = file.is_directory;
                                                move |_| {
                                                    if is_dir {
                                                        navigate_to_directory(path.clone());
                                                    } else {
                                                        // Select file
                                                        app_state.select_files(vec![path.clone()], SelectionMode::Replace);
                                                    }
                                                }
                                            },
                                            oncontextmenu: {
                                                let path = file.path.clone();
                                                move |evt| {
                                                    evt.prevent_default();
                                                    // Right-click context menu would go here
                                                }
                                            },

                                            div {
                                                class: "file-icon",
                                                style: "
                                                    font-size: 48px;
                                                    margin-bottom: 8px;
                                                ",
                                                "{file.file_type.icon()}"
                                            }

                                            div {
                                                class: "file-name",
                                                style: "
                                                    font-size: 12px;
                                                    text-align: center;
                                                    color: #cccccc;
                                                    word-wrap: break-word;
                                                    line-height: 1.2;
                                                    max-width: 100%;
                                                ",
                                                title: "{file.name}",
                                                "{file.name}"
                                            }

                                            if !file.is_directory {
                                                div {
                                                    class: "file-size",
                                                    style: "
                                                        font-size: 10px;
                                                        color: #8c8c8c;
                                                        margin-top: 4px;
                                                    ",
                                                    "{format_file_size(file.size)}"
                                                }
                                            }
                                        }
                                    })
                                }
                            }
                        } else {
                            div {
                                class: "welcome-message",
                                style: "
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    height: 100%;
                                    text-align: center;
                                    color: #8c8c8c;
                                ",

                                div {
                                    div {
                                        style: "font-size: 48px; margin-bottom: 16px;",
                                        "📁"
                                    }

                                    h2 {
                                        style: "
                                            font-size: 24px;
                                            font-weight: 300;
                                            margin: 0 0 8px 0;
                                            color: #cccccc;
                                        ",
                                        "Welcome to MediaOrganizer"
                                    }

                                    p {
                                        style: "
                                            font-size: 14px;
                                            margin: 0;
                                            line-height: 1.4;
                                        ",
                                        "Navigate to a folder to browse your files"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Status Bar
            div {
                class: "status-bar",
                style: "
                    height: 22px;
                    background: #007acc;
                    color: white;
                    display: flex;
                    align-items: center;
                    padding: 0 12px;
                    font-size: 12px;
                ",

                span { 
                    if app_state.is_current_directory_loading() {
                        "Loading..."
                    } else {
                        "Ready"
                    }
                }

                div {
                    class: "spacer",
                    style: "flex: 1;"
                }

                span { 
                    let selection_count = app_state.get_selection_count();
                    let metadata = app_state.get_selection_metadata();
                    
                    if selection_count == 0 {
                        if let Some(files) = app_state.get_current_directory_contents() {
                            format!("{} items", files.len())
                        } else {
                            "No items".to_string()
                        }
                    } else if selection_count == 1 {
                        format!("1 item selected ({})", format_file_size(metadata.total_size))
                    } else {
                        format!("{} items selected ({})", selection_count, format_file_size(metadata.total_size))
                    }
                }
            }
        }
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
