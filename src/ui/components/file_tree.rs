use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_solid_icons};
use std::path::PathBuf;
use crate::services::FileEntry;
use crate::state::{use_file_tree_state, use_app_state};

/// File tree component for sidebar navigation
#[component]
pub fn FileTree() -> Element {
    let mut file_tree_state = use_file_tree_state();
    let app_state = use_app_state();
    
    // Initialize root directory if not set
    use_effect(move || {
        let current_root = file_tree_state.read().root_directory.clone();
        if current_root.is_none() {
            let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
            file_tree_state.write().root_directory = Some(home_dir.clone());
            
            // Load root directory contents
            let file_service = app_state.file_service.clone();
            spawn(async move {
                if let Ok(entries) = file_service.list_directory(&home_dir).await {
                    file_tree_state.write().directory_children.insert(home_dir.clone(), entries);
                    file_tree_state.write().expanded_directories.insert(home_dir, true);
                }
            });
        }
    });
    
    let tree_state = file_tree_state.read();
    let root_dir = tree_state.root_directory.clone();
    drop(tree_state);
    
    rsx! {
        div {
            class: "file-tree",
            style: "
                padding: 8px 0;
                font-size: 13px;
                color: var(--vscode-foreground, #cccccc);
                user-select: none;
            ",
            role: "tree",
            "aria-label": "File explorer tree",
            
            if let Some(root) = root_dir {
                FileTreeNode {
                    entry: create_root_entry(root.clone()),
                    depth: 0,
                    is_root: true
                }
            } else {
                div {
                    style: "padding: 16px; color: var(--vscode-text-secondary, #999999);",
                    "Loading file tree..."
                }
            }
        }
    }
}

/// Individual file tree node component
#[component]
pub fn FileTreeNode(
    entry: FileEntry,
    depth: u32,
    is_root: bool,
) -> Element {
    let file_tree_state = use_file_tree_state();
    let is_directory = entry.is_directory;
    let path = entry.path.clone();
    let name = if is_root {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Root")
            .to_string()
    } else {
        entry.name.clone()
    };
    
    // Read state values
    let tree_state = file_tree_state.read();
    let is_expanded = tree_state.expanded_directories.get(&path).copied().unwrap_or(false);
    let is_loading = tree_state.loading_directories.contains(&path);
    let is_selected = tree_state.selected_path.as_ref().map(|p| p == &path).unwrap_or(false);
    let children = if is_expanded && is_directory {
        tree_state.directory_children.get(&path).cloned().unwrap_or_default()
    } else {
        Vec::new()
    };
    drop(tree_state); // Release the read lock
    
    // Calculate indentation
    let indent_px = depth * 12;
    
    rsx! {
        div {
            class: "file-tree-node",
            role: "treeitem",
            "aria-expanded": if is_directory { is_expanded.to_string() } else { "false".to_string() },
            "aria-selected": is_selected.to_string(),
            
            // Node header
            div {
                class: "file-tree-node-header",
                style: "
                    display: flex;
                    align-items: center;
                    padding: 2px 8px 2px {indent_px + 8}px;
                    cursor: pointer;
                    white-space: nowrap;
                    background: {if is_selected { \"var(--vscode-list-activeSelectionBackground, rgba(0, 122, 204, 0.3))\" } else { \"transparent\" }};
                    color: {if is_selected { \"var(--vscode-list-activeSelectionForeground, #ffffff)\" } else { \"inherit\" }};
                ",
                onclick: move |_| {
                    file_tree_state.write().selected_path = Some(path.clone());
                },
                ondoubleclick: move |_| {
                    if is_directory {
                        let current_expanded = file_tree_state.read()
                            .expanded_directories
                            .get(&path)
                            .copied()
                            .unwrap_or(false);
                        
                        if current_expanded {
                            file_tree_state.write().expanded_directories.insert(path.clone(), false);
                        } else {
                            file_tree_state.write().expanded_directories.insert(path.clone(), true);
                        }
                    }
                },
                
                // Expand/collapse icon for directories
                if is_directory {
                    div {
                        class: "expand-icon",
                        style: "
                            width: 16px;
                            height: 16px;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            margin-right: 4px;
                            cursor: pointer;
                            transform: {if is_expanded { \"rotate(90deg)\" } else { \"rotate(0deg)\" }};
                            transition: transform 0.15s ease;
                        ",
                        
                        if is_loading {
                            div {
                                style: "
                                    width: 8px;
                                    height: 8px;
                                    border: 1px solid var(--vscode-foreground, #cccccc);
                                    border-top: 1px solid transparent;
                                    border-radius: 50%;
                                    animation: spin 1s linear infinite;
                                "
                            }
                        } else {
                            Icon {
                                width: 12,
                                height: 12,
                                fill: "currentColor",
                                icon: fa_solid_icons::FaChevronRight,
                            }
                        }
                    }
                } else {
                    div {
                        style: "width: 20px; height: 16px;"
                    }
                }
                
                // File/folder icon
                div {
                    class: "file-icon",
                    style: "
                        width: 16px;
                        height: 16px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        margin-right: 6px;
                        opacity: 0.8;
                    ",
                    
                    {get_file_icon(&entry)}
                }
                
                // File/folder name
                span {
                    class: "file-name",
                    style: "
                        flex: 1;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        font-size: 13px;
                    ",
                    title: path.display().to_string(),
                    "{name}"
                }
            }
            
            // Children (if expanded)
            if is_expanded && is_directory && !children.is_empty() {
                div {
                    class: "file-tree-children",
                    role: "group",
                    
                    for child in children.iter() {
                        FileTreeNode {
                            key: child.path.to_string_lossy(),
                            entry: child.clone(),
                            depth: depth + 1,
                            is_root: false
                        }
                    }
                }
            }
        }
    }
}

/// Helper function to create a root directory entry
fn create_root_entry(path: PathBuf) -> FileEntry {
    FileEntry {
        name: path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Root")
            .to_string(),
        path: path.clone(),
        file_type: crate::services::file_system::FileType::Directory,
        size: 0,
        modified: std::time::SystemTime::now(),
        created: std::time::SystemTime::now(),
        is_directory: true,
        is_hidden: false,
        permissions: crate::services::file_system::FilePermissions {
            readable: true,
            writable: true,
            executable: true,
        },
    }
}

/// Helper function to get appropriate icon for file/directory
fn get_file_icon(entry: &FileEntry) -> Element {
    if entry.is_directory {
        rsx! {
            Icon {
                width: 14,
                height: 14,
                fill: "var(--vscode-icon-folder-color, #dcb67a)",
                icon: fa_solid_icons::FaFolder,
            }
        }
    } else {
        // Determine icon based on file extension
        let extension = entry.path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "rs" => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-rust-color, #ce422b)",
                    icon: fa_solid_icons::FaFileCode,
                }
            },
            "js" | "ts" | "jsx" | "tsx" => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-javascript-color, #f7df1e)",
                    icon: fa_solid_icons::FaFileCode,
                }
            },
            "json" | "xml" | "yaml" | "yml" | "toml" => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-json-color, #519aba)",
                    icon: fa_solid_icons::FaFileLines,
                }
            },
            "md" | "txt" | "rtf" => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-text-color, #519aba)",
                    icon: fa_solid_icons::FaFileLines,
                }
            },
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-image-color, #a074c4)",
                    icon: fa_solid_icons::FaFileImage,
                }
            },
            _ => rsx! {
                Icon {
                    width: 14,
                    height: 14,
                    fill: "var(--vscode-icon-file-color, #c5c5c5)",
                    icon: fa_solid_icons::FaFile,
                }
            },
        }
    }
}