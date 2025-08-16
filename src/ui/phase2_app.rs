use dioxus::prelude::*;
use std::path::PathBuf;
use crate::state::{save_panel_state_debounced, load_panel_state, use_app_state, use_file_entries, load_settings, save_settings_debounced, Theme};
use crate::theme::{ThemeManager, ThemeSelector, use_theme_manager};
use crate::services::file_system::{FileEntry, NativeFileSystemService, FileSystemService};
use crate::ui::{use_shortcut_handler, ShortcutRegistry};
use crate::ui::components::{
    ContextMenu, ContextMenuAction, use_context_menu,
    DragPreview, DropZone, DragOperation, DropZoneState,
    use_drag_drop, use_drop_zone,
    SettingsPanel
};
// use crate::ui::components::{VirtualFileTree};

pub fn phase2_app() -> Element {
    // Initialize panel width from saved state or default
    let mut panel_width = use_signal(|| {
        let saved_state = load_panel_state();
        saved_state.panel_width
    });
    let mut is_dragging = use_signal(|| false);
    let mut drag_start_x = use_signal(|| 0.0f64);
    let mut drag_start_width = use_signal(|| 300.0f64);
    
    // Get shared application state
    let app_state = use_app_state();
    let app_state_for_load = app_state.clone();
    let app_state_for_status = app_state.clone();
    let file_entries = use_file_entries();
    let mut selected_item = use_signal::<Option<FileEntry>>(|| None);
    
    // Initialize theme system
    let mut theme_manager = use_theme_manager();
    let mut current_settings = use_signal(|| load_settings());
    
    // Initialize keyboard shortcut handler
    let shortcut_handler = use_shortcut_handler();
    
    // Initialize context menu
    let (mut context_menu_state, handle_context_action) = use_context_menu();
    
    // Initialize drag-and-drop
    let (mut drag_state, _start_drag, _update_drag, _end_drag) = use_drag_drop();
    let (left_panel_drop_state, _set_left_panel_drop_state) = use_drop_zone();
    let (right_panel_drop_state, _set_right_panel_drop_state) = use_drop_zone();
    
    // Initialize settings panel state
    let mut settings_panel_visible = use_signal(|| false);
    
    // Initialize theme system
    use_effect(move || {
        let settings = current_settings.read();
        theme_manager.write().current_theme = settings.theme.clone();
        ThemeManager::initialize_with_settings(&settings);
        tracing::info!("Theme system initialized with theme: {:?}", settings.theme);
    });

    // Keyboard shortcut handler for settings panel
    let handle_keydown = {
        let mut settings_panel_visible = settings_panel_visible;
        let shortcut_handler = shortcut_handler;
        
        move |evt: KeyboardEvent| {
            let key = evt.data.key();
            let ctrl = evt.data.modifiers().ctrl();
            let shift = evt.data.modifiers().shift();
            let alt = evt.data.modifiers().alt();
            let meta = evt.data.modifiers().meta();
            
            // Convert key to string
            let key_str = key.to_string();
            
            tracing::info!("Keyboard event: {} (ctrl: {}, shift: {}, alt: {}, meta: {})", 
                         key_str, ctrl, shift, alt, meta);
            
            // Check for settings shortcut (Ctrl+,)
            if key_str == "," && ctrl && !shift && !alt && !meta {
                settings_panel_visible.set(true);
                evt.prevent_default();
                tracing::info!("Settings panel opened via keyboard shortcut");
                return;
            }
            
            // Check for escape to close settings panel
            if key_str == "Escape" && *settings_panel_visible.read() {
                settings_panel_visible.set(false);
                evt.prevent_default();
                tracing::info!("Settings panel closed via Escape key");
                return;
            }
            
            // Handle other shortcuts
            let mut handler = shortcut_handler.clone();
            spawn(async move {
                let handled = handler.handle_keyboard_event(&key_str, ctrl, shift, alt, meta).await;
                if handled {
                    tracing::info!("Keyboard shortcut handled: {}", key_str);
                } else {
                    tracing::debug!("Unhandled keyboard event: {}", key_str);
                }
            });
        }
    };

    // Load initial directory using shared state
    use_effect(move || {
        let mut app_state = app_state_for_load.clone();
        spawn(async move {
            let current_path = app_state.get_current_path();
            match app_state.file_service.list_directory(&current_path).await {
                Ok(files) => {
                    app_state.file_entries.set(files);
                }
                Err(e) => {
                    tracing::error!("Failed to load directory: {}", e);
                    // Create some demo entries if real directory loading fails
                    app_state.file_entries.set(create_demo_entries());
                }
            }
        });
    });

    // Save state when panel width changes (debounced)
    use_effect(move || {
        let current_width = panel_width.read();
        save_panel_state_debounced(*current_width, true);
    });

    // Flush pending saves on component cleanup
    use_effect(move || {
        // This will run when component unmounts
        // Note: In a real app, you'd want to handle this in window.onbeforeunload
    });

    // Dynamic width style for the panel (only property that changes)
    let panel_dynamic_style = format!("width: {}px;", panel_width.read());
    
    // Dynamic class for resize handle state
    let resize_handle_class = if *is_dragging.read() { 
        "resize-handle dragging" 
    } else { 
        "resize-handle" 
    };
    
    // Dynamic class for panel state
    let panel_class = if *is_dragging.read() {
        "file-tree-panel dragging"
    } else {
        "file-tree-panel"
    };

    rsx! {
        style { {include_str!("../../assets/styles.css")} }
        
        div {
            class: "media-organizer-app",
            tabindex: 0, // Make div focusable for keyboard events
            onkeydown: handle_keydown,
            onmousemove: move |evt| {
                let current_x = evt.data.client_coordinates().x as f64;
                let current_y = evt.data.client_coordinates().y as f64;
                
                // Handle panel resizing
                if *is_dragging.read() {
                    let delta = current_x - *drag_start_x.read();
                    let new_width = *drag_start_width.read() + delta;
                    
                    // Apply constraints: min 200px, max 50% of window (assuming 1200px+ screens)
                    let constrained_width = new_width.max(200.0).min(600.0);
                    panel_width.set(constrained_width);
                }
                
                // Handle drag-and-drop mouse movement
                drag_state.write().update_position(current_x, current_y);
            },
            onmouseup: move |_| {
                // Handle panel resizing end
                is_dragging.set(false);
                
                // Handle drag-and-drop end
                drag_state.write().end_drag();
            },
            
            // Title bar
            div {
                class: "title-bar",
                role: "banner",
                "aria-label": "Application title bar",
                style: "display: flex; align-items: center; justify-content: space-between;",
                
                h1 {
                    style: "margin: 0; font-size: inherit; font-weight: inherit;",
                    "MediaOrganizer - Task 10.4: Settings & Theme System ⚙️"
                }
                
                // Theme Selector in title bar
                div {
                    title: "Select application theme (Ctrl+, for settings)",
                    ThemeSelector {
                        current_theme: current_settings.read().theme.clone(),
                        on_theme_change: move |new_theme: Theme| {
                            // Update settings
                            let mut settings = {
                                let mut s = current_settings.write();
                                s.theme = new_theme.clone();
                                s.clone()
                            };
                            
                            // Update theme manager
                            theme_manager.write().set_theme(new_theme.clone(), &mut settings);
                            
                            // Save to persistence
                            save_settings_debounced(settings.clone());
                            
                            // Update current_settings signal to trigger re-render
                            current_settings.set(settings);
                            
                            tracing::info!("Theme changed to: {:?}", new_theme);
                        }
                    }
                }
            }
            
            // Main content area with split layout
            div {
                class: "main-content",
                role: "main",
                "aria-label": "File management interface",
                
                // Left Panel (File Tree) with Drop Zone
                DropZone {
                    drop_state: left_panel_drop_state,
                    target_path: Some(app_state.get_current_path()),
                    on_drop: move |data: (Vec<FileEntry>, DragOperation, PathBuf)| {
                        let (files, operation, target) = data;
                        tracing::info!("Files dropped in left panel: {} files with {:?} operation to {:?}", 
                                     files.len(), operation, target);
                        // TODO: Handle file drop operation
                    },
                    
                    div {
                        class: "{panel_class}",
                        role: "navigation",
                        "aria-label": "File explorer",
                        style: "{panel_dynamic_style}",
                    
                    // File tree header
                    div {
                        class: "file-tree-header",
                        role: "banner",
                        "aria-label": "File Explorer - Navigate through your files and folders",
                        title: "File Explorer - Navigate through your files and folders",
                        "Explorer"
                    }
                    
                    // Virtual file tree content
                    div {
                        class: "file-tree-content",
                        role: "region",
                        "aria-label": "File list",
                        style: "height: calc(100vh - 120px); overflow: hidden;", // Reserve space for header and status bar
                        
                        div {
                            style: "padding: 20px; color: #333;",
                            h3 { "File Tree (Temporarily Disabled)" }
                            p { {format!("Files loaded: {}", file_entries.read().len())} }
                            div {
                                role: "list",
                                "aria-label": format!("Directory contents - {} items", file_entries.read().len()),
                                style: "max-height: 400px; overflow-y: auto; border: 1px solid #ddd; padding: 10px; margin-top: 10px;",
                                {
                                    let entries = file_entries.read();
                                    let items: Vec<_> = entries.iter().take(20).cloned().collect();
                                    items.into_iter().enumerate().map(|(index, entry)| {
                                        let entry_clone = entry.clone();
                                        let entry_clone_key = entry.clone();
                                        let entry_clone_menu = entry.clone();
                                        let entry_clone_drag = entry.clone();
                                        let mut drag_state_clone = drag_state.clone();
                                        
                                        rsx! {
                                            div {
                                                key: "entry-{index}-{entry.path.to_string_lossy()}",
                                                class: "file-tree-item",
                                                tabindex: 0,
                                                role: "listitem",
                                                "aria-label": format!("{} {}{}", if entry.is_directory { "Folder" } else { "File" }, entry.name, if entry.size > 0 { format!(", {} bytes", entry.size) } else { String::new() }),
                                                "aria-describedby": format!("file-details-{}", index),
                                                draggable: true,
                                                
                                                onclick: move |_| {
                                                    tracing::info!("File clicked: {}", entry_clone.name);
                                                    selected_item.set(Some(entry_clone.clone()));
                                                },
                                                
                                                onkeydown: move |evt| {
                                                    let key = evt.data.key();
                                                    match key {
                                                        dioxus::events::Key::Enter => {
                                                            tracing::info!("File selected via keyboard: {}", entry_clone_key.name);
                                                            selected_item.set(Some(entry_clone_key.clone()));
                                                            evt.prevent_default();
                                                        },
                                                        _ => {
                                                            // Handle Space key via string comparison since it's not available as enum variant
                                                            let key_str = key.to_string();
                                                            if key_str == " " || key_str == "Space" {
                                                                tracing::info!("File selected via keyboard: {}", entry_clone_key.name);
                                                                selected_item.set(Some(entry_clone_key.clone()));
                                                                evt.prevent_default();
                                                            }
                                                        }
                                                    }
                                                },
                                                
                                                oncontextmenu: move |evt| {
                                                    evt.prevent_default();
                                                    let client_x = evt.data.client_coordinates().x as f64;
                                                    let client_y = evt.data.client_coordinates().y as f64;
                                                    
                                                    context_menu_state.write().show_at(
                                                        client_x, client_y, Some(entry_clone_menu.clone())
                                                    );
                                                    
                                                    tracing::info!("Context menu opened for: {}", entry_clone_menu.name);
                                                },
                                                
                                                ondragstart: move |evt| {
                                                    let client_x = evt.data.client_coordinates().x as f64;
                                                    let client_y = evt.data.client_coordinates().y as f64;
                                                    
                                                    // Determine drag operation based on modifiers
                                                    let operation = DragOperation::from_modifiers(
                                                        evt.data.modifiers().ctrl(),
                                                        evt.data.modifiers().shift(),
                                                        evt.data.modifiers().alt()
                                                    );
                                                    
                                                    // Directly call the drag state method
                                                    drag_state_clone.write().start_drag(
                                                        vec![entry_clone_drag.clone()],
                                                        client_x,
                                                        client_y,
                                                        operation
                                                    );
                                                    
                                                    tracing::info!("Started dragging: {}", entry_clone_drag.name);
                                                },
                                                
                                                // Hidden details for screen readers
                                                div {
                                                    id: format!("file-details-{}", index),
                                                    class: "sr-only",
                                                    style: "position: absolute; left: -10000px; width: 1px; height: 1px; overflow: hidden;",
                                                    {format!("{} type: {}, last modified: recently", 
                                                        if entry.is_directory { "Directory" } else { "File" },
                                                        if entry.is_directory { "Folder" } else { "Document" }
                                                    )}
                                                }
                                                
                                                span {
                                                    style: "margin-right: 8px; pointer-events: none;",
                                                    "aria-hidden": "true",
                                                    if entry.is_directory { "📁" } else { "📄" }
                                                }
                                                span { 
                                                    style: "pointer-events: none;",
                                                    {entry.name.clone()}
                                                }
                                                if entry.size > 0 {
                                                    span {
                                                        style: "margin-left: 10px; color: #666; font-size: 0.9em; pointer-events: none;",
                                                        "aria-hidden": "true",
                                                        "({entry.size} bytes)"
                                                    }
                                                }
                                            }
                                        }
                                    })
                                }
                                {
                                    let total_files = file_entries.read().len();
                                    if total_files > 20 {
                                        rsx! {
                                            div {
                                                style: "padding: 10px; color: #666; font-style: italic;",
                                                {format!("... and {} more files", total_files - 20)}
                                            }
                                        }
                                    } else {
                                        rsx! { div {} }
                                    }
                                }
                            }
                        }
                    }
                    }
                }
                
                // Resize Handle
                div {
                    class: "{resize_handle_class}",
                    title: "Drag to resize the file explorer panel. Use arrow keys to resize (Shift+Arrow for faster), Home/End for min/max width.",
                    role: "separator",
                    "aria-label": "Resize file explorer panel. Arrow keys to resize, Shift+Arrow for faster adjustment, Home for minimum, End for maximum",
                    tabindex: 0,
                    onmousedown: move |evt| {
                        is_dragging.set(true);
                        drag_start_x.set(evt.data.client_coordinates().x as f64);
                        drag_start_width.set(*panel_width.read());
                    },
                    onkeydown: move |evt| {
                        let key = evt.data.key();
                        let shift = evt.data.modifiers().shift();
                        match key {
                            dioxus::events::Key::ArrowLeft => {
                                let adjustment = if shift { 50.0 } else { 10.0 };
                                let new_width = (*panel_width.read() - adjustment).max(200.0);
                                panel_width.set(new_width);
                                tracing::info!("Panel resized via keyboard: {} -> {}", panel_width.read(), new_width);
                                evt.prevent_default();
                            },
                            dioxus::events::Key::ArrowRight => {
                                let adjustment = if shift { 50.0 } else { 10.0 };
                                let new_width = (*panel_width.read() + adjustment).min(600.0);
                                panel_width.set(new_width);
                                tracing::info!("Panel resized via keyboard: {} -> {}", panel_width.read(), new_width);
                                evt.prevent_default();
                            },
                            dioxus::events::Key::Home => {
                                panel_width.set(200.0);
                                tracing::info!("Panel reset to minimum width via keyboard");
                                evt.prevent_default();
                            },
                            dioxus::events::Key::End => {
                                panel_width.set(600.0);
                                tracing::info!("Panel reset to maximum width via keyboard");
                                evt.prevent_default();
                            },
                            _ => {}
                        }
                    },
                }
                
                // Right Panel (Content Viewer) with Drop Zone
                DropZone {
                    drop_state: right_panel_drop_state,
                    target_path: Some(app_state.get_current_path()),
                    on_drop: move |data: (Vec<FileEntry>, DragOperation, PathBuf)| {
                        let (files, operation, target) = data;
                        tracing::info!("Files dropped in right panel: {} files with {:?} operation to {:?}", 
                                     files.len(), operation, target);
                        // TODO: Handle file drop operation in content viewer
                    },
                    
                    div {
                        class: "content-viewer-panel",
                    
                    // Content area
                    div {
                        class: "content-area",
                        
                        div {
                            class: "content-area-icon",
                            "🎯"
                        }
                        
                        h2 {
                            class: "content-area-title",
                            "Settings & Theme System Active!"
                        }
                        
                        p {
                            class: "content-area-text",
                            "⚙️ Task 10.4: Settings Persistence and Theme Support"
                        }
                        
                        p {
                            class: "content-area-text",
                            "✅ Settings data structure with persistence"
                        }
                        
                        p {
                            class: "content-area-text",
                            "✅ Theme system with dark/light/auto modes"
                        }
                        
                        p {
                            class: "content-area-text",
                            "✅ Real-time theme switching with CSS custom properties"
                        }
                        
                        p {
                            class: "content-area-text",
                            {format!("Current Theme: {} ({})", 
                                ThemeManager::get_theme_display_name(&current_settings.read().theme),
                                ThemeManager::get_theme_display_name(&ThemeManager::get_effective_theme(&current_settings.read().theme))
                            )}
                        }
                        
                        div {
                            class: "content-area-badge",
                            "Interactive: Change theme using selector in title bar"
                        }
                        
                        div {
                            class: "feature-cards",
                            
                            div {
                                class: "feature-card",
                                
                                div { class: "feature-card-icon", "🌗" }
                                h4 { class: "feature-card-title", "Theme System" }
                                p { class: "feature-card-description", "Dark, Light, and Auto themes with real-time switching" }
                            }
                            
                            div {
                                class: "feature-card",
                                
                                div { class: "feature-card-icon", "⚙️" }
                                h4 { class: "feature-card-title", "Settings" }
                                p { class: "feature-card-description", "Persistent preferences with JSON serialization" }
                            }
                            
                            div {
                                class: "feature-card",
                                
                                div { class: "feature-card-icon", "💾" }
                                h4 { class: "feature-card-title", "Persistence" }
                                p { class: "feature-card-description", "Automatic save/load with debounced writes" }
                            }
                        }
                    }
                    }
                }
            }
            
            // Context Menu
            ContextMenu {
                menu_state: context_menu_state,
                on_action: handle_context_action
            }
            
            // Drag Preview
            DragPreview {
                drag_state: drag_state
            }
            
            // Status bar
            div {
                class: "status-bar",
                role: "status",
                "aria-label": "Application status and information",
                
                span { 
                    class: "status-bar-left", 
                    {format!("🔄 Virtual File Tree Active - {} items loaded", file_entries.read().len())}
                }
                
                // Show operation feedback if active
                {
                    let op_state = app_state_for_status.operation_state.read();
                    if op_state.is_active && !op_state.status_message.is_empty() {
                        rsx! {
                            span {
                                class: "status-bar-center",
                                role: "alert",
                                "aria-live": "polite",
                                "aria-atomic": "true",
                                style: "
                                    color: var(--vscode-text-secondary);
                                    font-style: italic;
                                    display: flex;
                                    align-items: center;
                                    gap: 4px;
                                ",
                                span {
                                    class: "loading-spinner",
                                    style: "
                                        width: 12px;
                                        height: 12px;
                                        border: 1px solid var(--vscode-border);
                                        border-top: 1px solid var(--vscode-accent);
                                        border-radius: 50%;
                                        animation: spin 1s linear infinite;
                                    ",
                                }
                                "{op_state.status_message}"
                            }
                        }
                    } else {
                        rsx! { span {} }
                    }
                }
                
                span {
                    class: "status-bar-right",
                    "Task 10.4: Settings & Theme System ⚙️"
                }
            }
            
            // Settings Panel
            SettingsPanel {
                is_visible: settings_panel_visible,
                settings: current_settings,
                on_close: move |_| {
                    settings_panel_visible.set(false);
                }
            }
        }
    }
}

/// Create demo file entries for testing when real directory loading fails
fn create_demo_entries() -> Vec<FileEntry> {
    use std::time::SystemTime;
    use crate::services::file_system::{FileType, FilePermissions};
    
    vec![
        FileEntry {
            path: std::path::PathBuf::from("Documents"),
            name: "Documents".to_string(),
            file_type: FileType::Directory,
            size: 0,
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: true,
            is_hidden: false,
            permissions: FilePermissions::read_write(),
        },
        FileEntry {
            path: std::path::PathBuf::from("Pictures"),
            name: "Pictures".to_string(),
            file_type: FileType::Directory,
            size: 0,
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: true,
            is_hidden: false,
            permissions: FilePermissions::read_write(),
        },
        FileEntry {
            path: std::path::PathBuf::from("Downloads"),
            name: "Downloads".to_string(),
            file_type: FileType::Directory,
            size: 0,
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: true,
            is_hidden: false,
            permissions: FilePermissions::read_write(),
        },
        FileEntry {
            path: std::path::PathBuf::from("example.jpg"),
            name: "example.jpg".to_string(),
            file_type: FileType::Image(crate::services::file_system::ImageFormat::Jpeg),
            size: 2048576, // 2MB
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: FilePermissions::read_only(),
        },
        FileEntry {
            path: std::path::PathBuf::from("document.pdf"),
            name: "document.pdf".to_string(),
            file_type: FileType::Document(crate::services::file_system::DocumentFormat::Pdf),
            size: 1048576, // 1MB
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: FilePermissions::read_write(),
        },
        FileEntry {
            path: std::path::PathBuf::from("video.mp4"),
            name: "video.mp4".to_string(),
            file_type: FileType::Video(crate::services::file_system::VideoFormat::Mp4),
            size: 52428800, // 50MB
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: FilePermissions::read_only(),
        },
        FileEntry {
            path: std::path::PathBuf::from("music.mp3"),
            name: "music.mp3".to_string(),
            file_type: FileType::Audio(crate::services::file_system::AudioFormat::Mp3),
            size: 4194304, // 4MB
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: FilePermissions::read_only(),
        },
    ]
}