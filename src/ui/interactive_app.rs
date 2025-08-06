use dioxus::prelude::*;
use crate::services::{FileSystemService, NativeFileSystemService, FileEntry};
use crate::state::{NavigationState, SelectionState, SelectionMode};
use std::sync::Arc;
use std::path::PathBuf;

#[component]
pub fn InteractiveApp(cx: Scope) -> Element {
    // Create services and state
    let file_service = use_state(cx, || Arc::new(NativeFileSystemService::new()));
    let navigation = use_state(cx, || NavigationState::new(dirs::home_dir()));
    let selection = use_state(cx, || SelectionState::new());
    let current_files = use_state(cx, || Vec::<FileEntry>::new());
    let loading = use_state(cx, || false);

    // Load initial directory
    use_effect(cx, (), move |_| {
        let file_service = file_service.get().clone();
        let current_path = navigation.get().current_path.clone();
        spawn(async move {
            loading.set(true);
            match file_service.list_directory(&current_path).await {
                Ok(files) => {
                    current_files.set(files);
                }
                Err(e) => {
                    tracing::warn!("Failed to load directory: {}", e);
                }
            }
            loading.set(false);
        });
        std::future::ready(())
    });

    let navigate_to = move |path: PathBuf| {
        let file_service = file_service.get().clone();
        spawn(async move {
            loading.set(true);
            
            // Update navigation state
            navigation.with_mut(|nav| {
                if let Err(e) = nav.navigate_to(path.clone()) {
                    tracing::warn!("Navigation error: {}", e);
                }
            });

            // Load directory contents
            match file_service.list_directory(&path).await {
                Ok(files) => {
                    current_files.set(files);
                }
                Err(e) => {
                    tracing::warn!("Failed to load directory: {}", e);
                }
            }

            // Clear selection
            selection.with_mut(|sel| {
                sel.clear_selection();
            });

            loading.set(false);
        });
    };

    let navigate_up = move |_| {
        let current_path = navigation.get().current_path.clone();
        if let Some(parent) = current_path.parent() {
            navigate_to(parent.to_path_buf());
        }
    };

    let nav_state = navigation.get();
    let sel_state = selection.get();
    let files = current_files.get();
    let is_loading = *loading.get();

    render! {
        div {
            style: "
                height: 100vh;
                display: flex;
                flex-direction: column;
                font-family: -apple-system, BlinkMacSystemFont, sans-serif;
                background-color: #1e1e1e;
                color: #cccccc;
            ",

            // Top Bar
            div {
                style: "
                    height: 40px;
                    background: #2d2d30;
                    border-bottom: 1px solid #3e3e42;
                    display: flex;
                    align-items: center;
                    padding: 0 12px;
                ",
                
                div { 
                    style: "font-size: 14px; font-weight: 600;",
                    "MediaOrganizer - Interactive Mode" 
                }
            }

            // Navigation Bar
            div {
                style: "
                    height: 35px;
                    background: #2d2d30;
                    border-bottom: 1px solid #3e3e42;
                    display: flex;
                    align-items: center;
                    padding: 0 12px;
                    gap: 8px;
                ",

                button {
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
                        if nav_state.can_navigate_up() { "#0e639c" } else { "#3c3c3c" },
                        if nav_state.can_navigate_up() { "white" } else { "#666" },
                        if nav_state.can_navigate_up() { "pointer" } else { "default" }
                    ),
                    disabled: !nav_state.can_navigate_up(),
                    onclick: navigate_up,
                    "↑ Up"
                }

                // Current path display
                div {
                    style: "
                        flex: 1;
                        font-size: 12px;
                        padding: 4px 8px;
                        background: #3c3c3c;
                        border: 1px solid #3e3e42;
                        border-radius: 2px;
                        color: #cccccc;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        white-space: nowrap;
                    ",
                    "{nav_state.current_path.display()}"
                }

                // Quick navigation buttons
                button {
                    style: "
                        background: #0e639c;
                        border: 1px solid #3e3e42;
                        color: white;
                        padding: 4px 8px;
                        border-radius: 2px;
                        font-size: 12px;
                        cursor: pointer;
                    ",
                    onclick: move |_| {
                        if let Ok(home_path) = std::env::var("HOME").map(PathBuf::from) {
                            navigate_to(home_path);
                        } else if let Some(home_path) = dirs::home_dir() {
                            navigate_to(home_path);
                        }
                    },
                    "🏠 Home"
                }
            }

            // Content Area
            div {
                style: "flex: 1; overflow: auto; padding: 16px;",

                if is_loading {
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            height: 200px;
                            color: #8c8c8c;
                            font-size: 14px;
                        ",
                        "Loading directory contents..."
                    }
                } else if files.is_empty() {
                    div {
                        style: "
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            height: 200px;
                            color: #8c8c8c;
                            font-size: 14px;
                        ",
                        "This folder is empty"
                    }
                } else {
                    div {
                        style: "
                            display: grid;
                            grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
                            gap: 16px;
                        ",

                        files.iter().map(|file| rsx! {
                            div {
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
                                    if sel_state.is_selected(&file.path) { "#264f78" } else { "transparent" },
                                    if sel_state.is_selected(&file.path) { "#007acc" } else { "transparent" }
                                ),
                                onclick: {
                                    let path = file.path.clone();
                                    let is_dir = file.is_directory;
                                    move |_| {
                                        if is_dir {
                                            navigate_to(path.clone());
                                        } else {
                                            selection.with_mut(|sel| {
                                                sel.select_files(vec![path.clone()], SelectionMode::Replace);
                                            });
                                        }
                                    }
                                },

                                div {
                                    style: "font-size: 48px; margin-bottom: 8px;",
                                    "{file.file_type.icon()}"
                                }

                                div {
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
            }

            // Status Bar
            div {
                style: "
                    height: 22px;
                    background: #007acc;
                    color: white;
                    display: flex;
                    align-items: center;
                    padding: 0 12px;
                    font-size: 12px;
                    justify-content: space-between;
                ",

                span { 
                    if is_loading {
                        "Loading..."
                    } else {
                        "Ready"
                    }
                }

                span { 
                    let selection_count = sel_state.selection_count();
                    
                    if selection_count == 0 {
                        format!("{} items", files.len())
                    } else if selection_count == 1 {
                        "1 item selected".to_string()
                    } else {
                        format!("{} items selected", selection_count)
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