use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_solid_icons};
use std::path::PathBuf;
use crate::services::FileEntry;
use crate::state::{use_file_tree_state, use_app_state};
use crate::ui::icon_packs::FileIconComponent;
use crate::ui::icon_manager::use_icon_manager;

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
    let mut file_tree_state = use_file_tree_state();
    let icon_manager = use_icon_manager();
    let current_icon_pack = icon_manager.settings.read().current_pack.clone();
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
    let header_background = if is_selected { "var(--vscode-list-activeSelectionBackground, rgba(0, 122, 204, 0.3))" } else { "transparent" };
    let header_color = if is_selected { "var(--vscode-list-activeSelectionForeground, #ffffff)" } else { "inherit" };
    let aria_expanded = if is_directory { is_expanded.to_string() } else { "false".to_string() };
    let expand_transform = if is_expanded { "rotate(90deg)" } else { "rotate(0deg)" };
    
    rsx! {
        div {
            class: "file-tree-node",
            role: "treeitem",
            "aria-expanded": "{aria_expanded}",
            "aria-selected": "{is_selected}",
            
            // Node header
            div {
                class: "file-tree-node-header",
                style: "
                    display: flex;
                    align-items: center;
                    padding: 2px 8px 2px {indent_px + 8}px;
                    cursor: pointer;
                    white-space: nowrap;
                    background: {header_background};
                    color: {header_color};
                ",
                onclick: {
                    let path_clone = path.clone();
                    let mut file_tree_state_clone = file_tree_state.clone();
                    move |_| {
                        file_tree_state_clone.write().selected_path = Some(path_clone.clone());
                        
                        // Generate preview (works for both files and directories)
                        let app_state = use_app_state();
                        let preview_path = path_clone.clone();
                        let is_dir = is_directory;
                        spawn(async move {
                            match app_state.handle_file_selection(preview_path.clone(), is_dir).await {
                                Ok(maybe_preview) => {
                                    if maybe_preview.is_some() {
                                        tracing::info!("Preview generated successfully for: {:?}", preview_path);
                                    } else {
                                        tracing::info!("No preview generated for directory: {:?}", preview_path);
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to handle file selection for {:?}: {}", preview_path, e);
                                }
                            }
                        });
                    }
                },
                ondoubleclick: {
                    let path_clone = path.clone();
                    let mut file_tree_state_clone = file_tree_state.clone();
                    move |_| {
                        if is_directory {
                            let current_expanded = file_tree_state_clone.read()
                                .expanded_directories
                                .get(&path_clone)
                                .copied()
                                .unwrap_or(false);
                            
                            if current_expanded {
                                file_tree_state_clone.write().expanded_directories.insert(path_clone.clone(), false);
                            } else {
                                file_tree_state_clone.write().expanded_directories.insert(path_clone.clone(), true);
                            }
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
                            transform: {expand_transform};
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
                    
                    FileIconComponent {
                        file_name: entry.name.clone(),
                        extension: entry.path.extension().and_then(|ext| ext.to_str()).map(|s| s.to_string()),
                        is_directory: entry.is_directory,
                        is_expanded: is_expanded,
                        pack: Some(current_icon_pack)  // Use the current icon pack from settings
                    }
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
        preview_metadata: None,
    }
}

