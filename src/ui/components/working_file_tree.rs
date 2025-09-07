use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::fa_solid_icons};
use std::path::PathBuf;
use crate::services::FileEntry;
use crate::state::{use_file_tree_state, use_app_state};
use crate::utils::{normalize_path_display, path_to_element_id};

/// Working file tree component for sidebar navigation
#[component]
pub fn WorkingFileTree() -> Element {
    let mut file_tree_state = use_file_tree_state();
    let app_state = use_app_state();
    let mut focused_item = use_signal(|| None::<std::path::PathBuf>); // Track focused item for keyboard nav
    let file_service = app_state.file_service.clone(); // Clone early to avoid borrow issues
    
    // Initialize root directory if not set
    use_effect({
        let file_service = file_service.clone();
        let mut file_tree_state = file_tree_state.clone();
        move || {
            let current_root = file_tree_state.read().root_directory.clone();
            if current_root.is_none() {
                let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                file_tree_state.write().root_directory = Some(home_dir.clone());
                
                // Load root directory contents
                let file_service = file_service.clone();
                let mut file_tree_state_clone = file_tree_state.clone();
                spawn(async move {
                    if let Ok(entries) = file_service.list_directory(&home_dir).await {
                        let mut tree_state = file_tree_state_clone.write();
                        tree_state.directory_children.insert(home_dir.clone(), entries);
                        tree_state.expanded_directories.insert(home_dir, true);
                        tracing::info!("WorkingFileTree: Loaded root directory");
                    }
                });
            }
        }
    });
    
    let tree_state = file_tree_state.read();
    let root_dir = tree_state.root_directory.clone();
    let children = root_dir.as_ref()
        .and_then(|root| tree_state.directory_children.get(root))
        .cloned()
        .unwrap_or_default();
    drop(tree_state);
    
    rsx! {
        div {
            class: "working-file-tree",
            style: "
                padding: 4px 0;
                font-size: 13px;
                color: var(--vscode-foreground, #cccccc);
                user-select: none;
                --list-item-padding: 1px 4px;
                --list-item-margin: 0;
                --list-indent-margin: 16px;
            ",
            role: "tree",
            "aria-label": "File explorer tree - Navigate files and folders",
            "aria-describedby": "file-tree-instructions",
            "aria-multiselectable": "false",
            tabindex: "0",
            onkeydown: {
                let file_service = file_service.clone();
                let mut file_tree_state = file_tree_state.clone();
                let mut focused_item = focused_item.clone();
                move |evt: Event<KeyboardData>| {
                    match evt.data.key() {
                        dioxus::events::keyboard_types::Key::ArrowDown => {
                            evt.prevent_default();
                            // Compute visible entries dynamically
                            let tree_state = file_tree_state.read();
                            let root_dir = tree_state.root_directory.clone();
                            let children = root_dir.as_ref()
                                .and_then(|root| tree_state.directory_children.get(root))
                                .cloned()
                                .unwrap_or_default();
                            let mut visible_entries = Vec::new();
                            if let Some(root) = root_dir.as_ref() {
                                collect_visible_entries(&tree_state, root, &children, &mut visible_entries);
                            }
                            drop(tree_state);

                            let current_focused = focused_item.read().clone();
                            if let Some(current_focused) = current_focused.as_ref() {
                                if let Some(current_index) = visible_entries.iter().position(|e| &e.path == current_focused) {
                                    let next_index = (current_index + 1).min(visible_entries.len().saturating_sub(1));
                                    focused_item.set(Some(visible_entries[next_index].path.clone()));
                                }
                            } else if !visible_entries.is_empty() {
                                focused_item.set(Some(visible_entries[0].path.clone()));
                            }
                        },
                        dioxus::events::keyboard_types::Key::ArrowUp => {
                            evt.prevent_default();
                            // Compute visible entries dynamically
                            let tree_state = file_tree_state.read();
                            let root_dir = tree_state.root_directory.clone();
                            let children = root_dir.as_ref()
                                .and_then(|root| tree_state.directory_children.get(root))
                                .cloned()
                                .unwrap_or_default();
                            let mut visible_entries = Vec::new();
                            if let Some(root) = root_dir.as_ref() {
                                collect_visible_entries(&tree_state, root, &children, &mut visible_entries);
                            }
                            drop(tree_state);

                            let current_focused = focused_item.read().clone();
                            if let Some(current_focused) = current_focused.as_ref() {
                                if let Some(current_index) = visible_entries.iter().position(|e| &e.path == current_focused) {
                                    let prev_index = current_index.saturating_sub(1);
                                    focused_item.set(Some(visible_entries[prev_index].path.clone()));
                                }
                            } else if !visible_entries.is_empty() {
                                focused_item.set(Some(visible_entries[visible_entries.len() - 1].path.clone()));
                            }
                        },
                        dioxus::events::keyboard_types::Key::ArrowRight => {
                            evt.prevent_default();
                            if let Some(current_focused) = focused_item.read().as_ref() {
                                // Find the entry directly from tree state
                                let tree_state = file_tree_state.read();
                                let root_dir = tree_state.root_directory.clone();
                                let children = root_dir.as_ref()
                                    .and_then(|root| tree_state.directory_children.get(root))
                                    .cloned()
                                    .unwrap_or_default();
                                let mut visible_entries = Vec::new();
                                if let Some(root) = root_dir.as_ref() {
                                    collect_visible_entries(&tree_state, root, &children, &mut visible_entries);
                                }
                                let current_entry = visible_entries.iter().find(|e| &e.path == current_focused);
                                drop(tree_state);

                                if let Some(entry) = current_entry {
                                    if entry.is_directory {
                                        let is_expanded = file_tree_state.read().expanded_directories.get(&entry.path).copied().unwrap_or(false);
                                        if !is_expanded {
                                            file_tree_state.write().expanded_directories.insert(entry.path.clone(), true);
                                            file_tree_state.write().loading_directories.insert(entry.path.clone());
                                            
                                            let file_service = file_service.clone();
                                            let path_clone = entry.path.clone();
                                            let mut file_tree_state_clone = file_tree_state.clone();
                                            spawn(async move {
                                                match file_service.list_directory(&path_clone).await {
                                                    Ok(entries) => {
                                                        let mut tree_state = file_tree_state_clone.write();
                                                        tree_state.directory_children.insert(path_clone.clone(), entries);
                                                        tree_state.loading_directories.remove(&path_clone);
                                                        tracing::info!("Keyboard: Expanded directory {:?}", path_clone);
                                                    }
                                                    Err(e) => {
                                                        let mut tree_state = file_tree_state_clone.write();
                                                        tree_state.loading_directories.remove(&path_clone);
                                                        tree_state.error_directories.insert(path_clone.clone(), format!("Error: {}", e));
                                                        tree_state.expanded_directories.insert(path_clone.clone(), false);
                                                        tracing::error!("Keyboard: Failed to expand directory {:?}: {}", path_clone, e);
                                                    }
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        },
                        dioxus::events::keyboard_types::Key::ArrowLeft => {
                            evt.prevent_default();
                            if let Some(current_focused) = focused_item.read().as_ref() {
                                // Find current entry to check if it's a directory
                                let tree_state = file_tree_state.read();
                                let root_dir = tree_state.root_directory.clone();
                                let children = root_dir.as_ref()
                                    .and_then(|root| tree_state.directory_children.get(root))
                                    .cloned()
                                    .unwrap_or_default();
                                let mut visible_entries = Vec::new();
                                if let Some(root) = root_dir.as_ref() {
                                    collect_visible_entries(&tree_state, root, &children, &mut visible_entries);
                                }
                                let current_entry = visible_entries.iter().find(|e| &e.path == current_focused);
                                drop(tree_state);

                                if let Some(entry) = current_entry {
                                    if entry.is_directory {
                                        let is_expanded = file_tree_state.read().expanded_directories.get(&entry.path).copied().unwrap_or(false);
                                        if is_expanded {
                                            file_tree_state.write().expanded_directories.insert(entry.path.clone(), false);
                                            tracing::info!("Keyboard: Collapsed directory {:?}", entry.path);
                                        }
                                    }
                                }
                            }
                        },
                        dioxus::events::keyboard_types::Key::Enter => {
                            evt.prevent_default();
                            if let Some(current_focused) = focused_item.read().as_ref() {
                                file_tree_state.write().selected_path = Some(current_focused.clone());
                                tracing::info!("Keyboard: Selected {:?}", current_focused);
                            }
                        },
                        dioxus::events::keyboard_types::Key::Character(s) if s == " " => {
                            evt.prevent_default();
                            if let Some(current_focused) = focused_item.read().as_ref() {
                                // Find current entry to determine if it's a directory
                                let tree_state = file_tree_state.read();
                                let root_dir = tree_state.root_directory.clone();
                                let children = root_dir.as_ref()
                                    .and_then(|root| tree_state.directory_children.get(root))
                                    .cloned()
                                    .unwrap_or_default();
                                let mut visible_entries = Vec::new();
                                if let Some(root) = root_dir.as_ref() {
                                    collect_visible_entries(&tree_state, root, &children, &mut visible_entries);
                                }
                                let current_entry = visible_entries.iter().find(|e| &e.path == current_focused);
                                drop(tree_state);

                                if let Some(entry) = current_entry {
                                    if entry.is_directory {
                                        let is_expanded = file_tree_state.read().expanded_directories.get(&entry.path).copied().unwrap_or(false);
                                        file_tree_state.write().expanded_directories.insert(entry.path.clone(), !is_expanded);
                                        tracing::info!("Keyboard: Toggled directory {:?} to {}", entry.path, !is_expanded);
                                    } else {
                                        file_tree_state.write().selected_path = Some(current_focused.clone());
                                        tracing::info!("Keyboard: Selected file {:?}", current_focused);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            },
            
            // Hidden instructions for screen readers
            div {
                id: "file-tree-instructions",
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
                "Use arrow keys to navigate. Right arrow or Space to expand folders, Left arrow to collapse. Enter to select files or folders."
            }
            
            if let Some(root) = root_dir {
                div {
                    // Root directory header
                    div {
                        style: "
                            padding: 4px 8px;
                            font-weight: bold;
                            font-size: 11px;
                            color: var(--vscode-text-secondary, #999999);
                            text-transform: uppercase;
                        ",
                        "{root.file_name().and_then(|n| n.to_str()).unwrap_or(\"Root\")}"
                    }
                    
                    // Directory contents
                    if children.is_empty() {
                        div {
                            style: "padding: 8px 16px; color: var(--vscode-text-secondary, #999999); font-style: italic;",
                            "Loading directory..."
                        }
                    } else {
                        for (_index, entry) in children.iter().enumerate() {
                            WorkingFileTreeItem {
                                key: format!("{}-{}", entry.name, normalize_path_display(&entry.path)),
                                entry: entry.clone(),
                                is_focused: focused_item.read().as_ref().map(|f| f == &entry.path).unwrap_or(false),
                            }
                        }
                    }
                }
            } else {
                div {
                    style: "padding: 8px 16px; color: var(--vscode-text-secondary, #999999);",
                    "Initializing file tree..."
                }
            }
        }
    }
}

/// Simple file tree item component with hierarchical display
#[component]
pub fn WorkingFileTreeItem(entry: FileEntry, is_focused: bool) -> Element {
    let mut file_tree_state = use_file_tree_state();
    let is_directory = entry.is_directory;
    let name = entry.name.clone();
    let path = entry.path.clone();
    
    // Check state values
    let tree_state = file_tree_state.read();
    let is_selected = tree_state.selected_path.as_ref().map(|p| p == &path).unwrap_or(false);
    let is_expanded = if is_directory {
        tree_state.expanded_directories.get(&path).copied().unwrap_or(false)
    } else {
        false
    };
    let is_loading = tree_state.loading_directories.contains(&path);
    let has_error = tree_state.error_directories.contains_key(&path);
    let children = if is_expanded && is_directory {
        tree_state.directory_children.get(&path).cloned().unwrap_or_default()
    } else {
        Vec::new()
    };
    drop(tree_state);
    
    // Pre-calculate styles to avoid interpolation in rsx!
    let background = if is_selected { 
        "var(--vscode-list-activeSelectionBackground, rgba(0, 122, 204, 0.3))" 
    } else if is_focused {
        "var(--vscode-list-focusBackground, rgba(255, 255, 255, 0.1))"
    } else { 
        "transparent" 
    };
    let text_color = if is_selected { 
        "var(--vscode-list-activeSelectionForeground, #ffffff)" 
    } else { 
        "inherit" 
    };
    let border = if is_focused {
        "2px solid var(--vscode-focusBorder, #007acc)"
    } else {
        "2px solid transparent"
    };
    
    let outline = if is_focused {
        "2px solid var(--vscode-focusBorder, #007acc)"
    } else {
        "none"
    };
    
    let box_shadow = if is_focused {
        "0 0 0 1px var(--vscode-focusBorder, #007acc)"
    } else {
        "none"
    };
    let arrow_rotation = if is_expanded { "90deg" } else { "0deg" };
    
    // Clone path for closures and display title
    let click_path = path.clone();
    let expand_path = path.clone();
    let title_path = normalize_path_display(&path);
    
    rsx! {
        div {
            class: "file-tree-node",
            role: "treeitem",
            "aria-expanded": if is_directory { is_expanded.to_string() } else { "false".to_string() },
            "aria-selected": is_selected.to_string(),
            "aria-level": "1", // This would need to be dynamic for nested items
            "aria-label": format!("{} {}", if is_directory { "Folder" } else { "File" }, name),
            
            // Node header with expand arrow for directories
            div {
                class: "file-tree-item",
                style: "
                    display: flex;
                    align-items: center;
                    padding: var(--list-item-padding, 1px 4px);
                    cursor: pointer;
                    white-space: nowrap;
                    background: {background};
                    color: {text_color};
                    border: {border};
                    border-radius: 3px;
                    outline: {outline};
                    box-shadow: {box_shadow};
                    transition: all 0.2s ease;
                ",
                role: "button",
                tabindex: "0",
                "aria-describedby": format!("file-item-{}", path_to_element_id(&path)),
                onclick: move |_| {
                    file_tree_state.write().selected_path = Some(click_path.clone());
                    tracing::info!("Selected: {:?}", click_path);
                },
                ondoubleclick: move |_| {
                    if is_directory {
                        let app_state = use_app_state();
                        let mut file_tree_state = file_tree_state.clone();
                        
                        // Toggle expansion state
                        let current_expanded = {
                            let tree_state = file_tree_state.read();
                            tree_state.expanded_directories.get(&expand_path).copied().unwrap_or(false)
                        };
                        
                        if current_expanded {
                            // Collapse directory
                            file_tree_state.write().expanded_directories.insert(expand_path.clone(), false);
                            tracing::info!("Collapsed directory: {:?}", expand_path);
                        } else {
                            // Expand directory - load children if not already loaded
                            file_tree_state.write().expanded_directories.insert(expand_path.clone(), true);
                            file_tree_state.write().loading_directories.insert(expand_path.clone());
                            
                            let file_service = app_state.file_service.clone();
                            let expand_path_clone = expand_path.clone();
                            spawn(async move {
                                match file_service.list_directory(&expand_path_clone).await {
                                    Ok(entries) => {
                                        let mut tree_state = file_tree_state.write();
                                        tree_state.directory_children.insert(expand_path_clone.clone(), entries);
                                        tree_state.loading_directories.remove(&expand_path_clone);
                                        tree_state.error_directories.remove(&expand_path_clone);
                                        tracing::info!("Expanded directory: {:?}", expand_path_clone);
                                    }
                                    Err(e) => {
                                        let mut tree_state = file_tree_state.write();
                                        tree_state.loading_directories.remove(&expand_path_clone);
                                        tree_state.error_directories.insert(expand_path_clone.clone(), format!("Error: {}", e));
                                        tree_state.expanded_directories.insert(expand_path_clone.clone(), false);
                                        tracing::error!("Failed to load directory {:?}: {}", expand_path_clone, e);
                                    }
                                }
                            });
                        }
                    }
                },
                
                // Expansion arrow for directories
                if is_directory {
                    div {
                        class: "expand-arrow",
                        style: "
                            width: 16px;
                            height: 16px;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            margin-right: 1px;
                            transform: rotate({arrow_rotation});
                            transition: transform 0.15s ease;
                        ",
                        
                        if is_loading {
                            // Loading spinner
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
                                width: 8,
                                height: 8,
                                fill: "currentColor",
                                icon: fa_solid_icons::FaChevronRight,
                            }
                        }
                    }
                } else {
                    // Spacer for files
                    div {
                        style: "width: 18px; height: 16px;"
                    }
                }
                
                // File/folder icon
                div {
                    style: "
                        width: 16px;
                        height: 16px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        margin-right: 4px;
                    ",
                    
                    if is_directory {
                        Icon {
                            width: 14,
                            height: 14,
                            fill: "var(--vscode-icon-folder-color, #dcb67a)",
                            icon: fa_solid_icons::FaFolder,
                        }
                    } else {
                        // Show different icons based on file extension
                        {get_file_icon(&entry)}
                    }
                }
                
                // File/folder name
                span {
                    style: "
                        flex: 1;
                        overflow: hidden;
                        text-overflow: ellipsis;
                        font-size: 13px;
                    ",
                    title: title_path.clone(),
                    "{name}"
                }
                
                // Error indicator
                if has_error {
                    div {
                        style: "
                            width: 12px;
                            height: 12px;
                            margin-left: 4px;
                        ",
                        title: "Failed to load directory",
                        
                        Icon {
                            width: 12,
                            height: 12,
                            fill: "var(--vscode-error-color, #f48771)",
                            icon: fa_solid_icons::FaExclamation,
                        }
                    }
                }
            }
            
            // Children (if expanded and has children)
            if is_expanded && is_directory && !children.is_empty() {
                div {
                    class: "file-tree-children",
                    style: "margin-left: var(--list-indent-margin, 16px);",
                    role: "group",
                    "aria-label": format!("Contents of {}", name),
                    
                    for child in children.iter() {
                        WorkingFileTreeItem {
                            key: format!("child-{}", normalize_path_display(&child.path)),
                            entry: child.clone(),
                            is_focused: false, // Focus is handled at the top level
                        }
                    }
                }
            }
            
            // Hidden description for screen readers
            div {
                id: format!("file-item-{}", path_to_element_id(&path)),
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
                {format!("{} {}. {}",
                    if is_directory { "Folder" } else { "File" },
                    name,
                    if is_directory {
                        if is_expanded { "Expanded. Press Left arrow to collapse." } else { "Collapsed. Press Right arrow or Space to expand." }
                    } else {
                        "Press Enter to select."
                    }
                )}
            }
        }
    }
}

/// Helper function to get appropriate icon for file types
fn get_file_icon(entry: &FileEntry) -> Element {
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

/// Helper function to collect all visible entries in tree order for keyboard navigation
fn collect_visible_entries(
    tree_state: &crate::state::FileTreeState,
    root: &std::path::Path,
    children: &[FileEntry],
    visible_entries: &mut Vec<FileEntry>,
) {
    // Add root entries
    for child in children {
        visible_entries.push(child.clone());
        
        // If this is an expanded directory, add its children recursively
        if child.is_directory && tree_state.expanded_directories.get(&child.path).copied().unwrap_or(false) {
            if let Some(grandchildren) = tree_state.directory_children.get(&child.path) {
                collect_visible_entries(tree_state, &child.path, grandchildren, visible_entries);
            }
        }
    }
}