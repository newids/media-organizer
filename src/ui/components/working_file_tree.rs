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
                            // Navigate to next item using optimized method
                            let tree_state = file_tree_state.read();
                            let visible_entries = tree_state.get_visible_entries();
                            drop(tree_state);

                            let current_focused = focused_item.read().clone();
                            if let Some(current_focused) = current_focused.as_ref() {
                                if let Some(current_index) = visible_entries.iter().position(|path| path == current_focused) {
                                    let next_index = (current_index + 1).min(visible_entries.len().saturating_sub(1));
                                    if next_index != current_index {
                                        focused_item.set(Some(visible_entries[next_index].clone()));
                                        tracing::info!("Keyboard: Moved down to {:?}", visible_entries[next_index]);
                                    }
                                }
                            } else if !visible_entries.is_empty() {
                                focused_item.set(Some(visible_entries[0].clone()));
                                tracing::info!("Keyboard: Focused first entry {:?}", visible_entries[0]);
                            }
                        },
                        dioxus::events::keyboard_types::Key::ArrowUp => {
                            evt.prevent_default();
                            // Navigate to previous item using optimized method
                            let tree_state = file_tree_state.read();
                            let visible_entries = tree_state.get_visible_entries();
                            drop(tree_state);

                            let current_focused = focused_item.read().clone();
                            if let Some(current_focused) = current_focused.as_ref() {
                                if let Some(current_index) = visible_entries.iter().position(|path| path == current_focused) {
                                    let prev_index = current_index.saturating_sub(1);
                                    if prev_index != current_index {
                                        focused_item.set(Some(visible_entries[prev_index].clone()));
                                        tracing::info!("Keyboard: Moved up to {:?}", visible_entries[prev_index]);
                                    }
                                }
                            } else if !visible_entries.is_empty() {
                                focused_item.set(Some(visible_entries[visible_entries.len() - 1].clone()));
                                tracing::info!("Keyboard: Focused last entry {:?}", visible_entries[visible_entries.len() - 1]);
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
                        dioxus::events::keyboard_types::Key::Home => {
                            evt.prevent_default();
                            // Jump to first item in the tree
                            let tree_state = file_tree_state.read();
                            let visible_entries = tree_state.get_visible_entries();
                            drop(tree_state);
                            
                            if let Some(first_entry) = visible_entries.first() {
                                focused_item.set(Some(first_entry.clone()));
                                tracing::info!("Keyboard: Jumped to first item {:?}", first_entry);
                            }
                        },
                        dioxus::events::keyboard_types::Key::End => {
                            evt.prevent_default();
                            // Jump to last item in the tree
                            let tree_state = file_tree_state.read();
                            let visible_entries = tree_state.get_visible_entries();
                            drop(tree_state);
                            
                            if let Some(last_entry) = visible_entries.last() {
                                focused_item.set(Some(last_entry.clone()));
                                tracing::info!("Keyboard: Jumped to last item {:?}", last_entry);
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
                                depth: Some(0), // Root level children start at depth 0
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

/// Hierarchical file tree item component with proper nesting depth
#[component]
pub fn WorkingFileTreeItem(entry: FileEntry, is_focused: bool, depth: Option<usize>) -> Element {
    let mut file_tree_state = use_file_tree_state();
    let is_directory = entry.is_directory;
    let name = entry.name.clone();
    let path = entry.path.clone();
    
    // Calculate nesting depth using our enhanced state management
    let nesting_depth = depth.unwrap_or_else(|| {
        let tree_state = file_tree_state.read();
        let depth = tree_state.get_nesting_depth(&path);
        drop(tree_state);
        depth
    });
    
    // Check state values using enhanced methods
    let tree_state = file_tree_state.read();
    let is_selected = tree_state.selected_path.as_ref().map(|p| p == &path).unwrap_or(false);
    let is_expanded = if is_directory {
        tree_state.is_expanded(&path)
    } else {
        false
    };
    let is_loading = tree_state.is_loading(&path);
    let has_error = tree_state.get_directory_error(&path).is_some();
    let has_children = tree_state.has_children(&path);
    let children_count = tree_state.get_children_count(&path);
    let children = if is_expanded && is_directory {
        tree_state.get_directory_children(&path).cloned().unwrap_or_default()
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
    
    // Calculate proper indentation based on nesting depth
    let indent_pixels = nesting_depth * 16; // 16px per level
    let item_padding_left = format!("{}px", 4 + indent_pixels); // Base 4px + depth indentation
    
    // Enhanced visual indicators - both branches must return the same type
    let use_expanded_icon = is_expanded && is_directory;
    
    let folder_color = if is_expanded && is_directory {
        "var(--vscode-icon-folder-expanded-color, #dcb67a)"
    } else {
        "var(--vscode-icon-folder-color, #dcb67a)"
    };
    
    // Clone path for closures and display title
    let click_path = path.clone();
    let expand_path = path.clone();
    let title_path = normalize_path_display(&path);
    
    // Enhanced aria-level for proper screen reader support
    let aria_level = (nesting_depth + 1).to_string();
    
    rsx! {
        div {
            class: "file-tree-node",
            role: "treeitem",
            "aria-expanded": if is_directory { is_expanded.to_string() } else { "false".to_string() },
            "aria-selected": is_selected.to_string(),
            "aria-level": aria_level,
            "aria-label": format!("{} {} (level {}{})", 
                if is_directory { "Folder" } else { "File" }, 
                name,
                nesting_depth + 1,
                if is_directory && has_children {
                    format!(", {} items", children_count)
                } else {
                    "".to_string()
                }
            ),
            
            // Node header with expand arrow for directories
            div {
                class: "file-tree-item",
                style: "
                    display: flex;
                    align-items: center;
                    padding: 1px {item_padding_left} 1px 4px;
                    cursor: pointer;
                    white-space: nowrap;
                    background: {background};
                    color: {text_color};
                    border: {border};
                    border-radius: 3px;
                    outline: {outline};
                    box-shadow: {box_shadow};
                    transition: all 0.2s ease;
                    min-height: 22px;
                ",
                role: "button",
                tabindex: "0",
                "aria-describedby": format!("file-item-{}", path_to_element_id(&path)),
                onclick: move |_| {
                    file_tree_state.write().selected_path = Some(click_path.clone());
                    tracing::info!("Selected: {:?}", click_path);
                    
                    // Generate preview (works for both files and directories)
                    let app_state = use_app_state();
                    let preview_path = click_path.clone();
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
                },
                ondoubleclick: move |evt| {
                    if is_directory {
                        let app_state = use_app_state();
                        let mut file_tree_state = file_tree_state.clone();
                        
                        // Enhanced toggle expansion with lazy loading
                        let current_expanded = {
                            let tree_state = file_tree_state.read();
                            tree_state.is_expanded(&expand_path)
                        };
                        
                        if current_expanded {
                            // Collapse directory - support recursive collapse with Shift+Click
                            let should_collapse_recursively = evt.data.modifiers().shift();
                            
                            if should_collapse_recursively {
                                // Collapse all children recursively
                                file_tree_state.write().collapse_all_under(&expand_path);
                                tracing::info!("Recursively collapsed directory: {:?}", expand_path);
                            } else {
                                // Just collapse this directory
                                file_tree_state.write().expanded_directories.insert(expand_path.clone(), false);
                                tracing::info!("Collapsed directory: {:?}", expand_path);
                            }
                        } else {
                            // Expand directory with enhanced lazy loading
                            let needs_loading = {
                                let tree_state = file_tree_state.read();
                                !tree_state.has_children(&expand_path) && !tree_state.is_loading(&expand_path)
                            };
                            
                            // Set expanded state immediately for better UX
                            file_tree_state.write().expanded_directories.insert(expand_path.clone(), true);
                            
                            if needs_loading {
                                // Start loading with enhanced error handling
                                file_tree_state.write().set_loading(expand_path.clone(), true);
                                
                                let file_service = app_state.file_service.clone();
                                let expand_path_clone = expand_path.clone();
                                let mut file_tree_state_async = file_tree_state.clone();
                                
                                spawn(async move {
                                    // Add a small delay for better visual feedback on fast loads
                                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                                    
                                    match file_service.list_directory(&expand_path_clone).await {
                                        Ok(entries) => {
                                            let mut tree_state = file_tree_state_async.write();
                                            tree_state.set_directory_children(expand_path_clone.clone(), entries.clone());
                                            
                                            tracing::info!(
                                                "Successfully expanded directory: {:?} ({} items)", 
                                                expand_path_clone,
                                                entries.len()
                                            );
                                        }
                                        Err(e) => {
                                            let mut tree_state = file_tree_state_async.write();
                                            tree_state.set_directory_error(
                                                expand_path_clone.clone(), 
                                                format!("Failed to load: {}", e)
                                            );
                                            // Revert expansion state on error
                                            tree_state.expanded_directories.insert(expand_path_clone.clone(), false);
                                            
                                            tracing::error!(
                                                "Failed to expand directory {:?}: {}", 
                                                expand_path_clone, e
                                            );
                                        }
                                    }
                                });
                            } else {
                                tracing::info!("Directory already loaded, expanding: {:?}", expand_path);
                            }
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
                        if use_expanded_icon {
                            Icon {
                                width: 14,
                                height: 14,
                                fill: folder_color,
                                icon: fa_solid_icons::FaFolderOpen,
                            }
                        } else {
                            Icon {
                                width: 14,
                                height: 14,
                                fill: folder_color,
                                icon: fa_solid_icons::FaFolder,
                            }
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
            
            // Children (if expanded and has children) - now with proper recursive nesting
            if is_expanded && is_directory && !children.is_empty() {
                for child in children.iter() {
                    WorkingFileTreeItem {
                        key: format!("nested-{}-{}", nesting_depth, normalize_path_display(&child.path)),
                        entry: child.clone(),
                        is_focused: false, // Focus is handled at the top level
                        depth: Some(nesting_depth + 1), // Pass incremented depth for proper nesting
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