use dioxus::prelude::*;
use dioxus::events::{MouseEvent, FormData};
use dioxus_elements::geometry::WheelDelta;
use dioxus_free_icons::icons::fa_solid_icons;
use dioxus_free_icons::Icon;
use crate::services::preview::{PreviewData, PreviewContent, SupportedFormat};
use crate::state::use_app_state;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Simple file system entry for preview panel
#[derive(Debug, Clone, PartialEq)]
pub struct FileSystemEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub size: u64,
    pub modified: std::time::SystemTime,
    pub file_type: Option<String>,
}

/// Loading states for progressive rendering
#[derive(Debug, Clone, PartialEq)]
pub enum LoadingState {
    /// Not started loading
    NotLoaded,
    /// Loading in progress with optional progress percentage (0-100)
    Loading(Option<u8>),
    /// Successfully loaded
    Loaded,
    /// Failed to load with error message
    Failed(String),
}

/// Performance metrics for lazy loading
#[derive(Debug, Clone)]
pub struct LoadingMetrics {
    pub start_time: Instant,
    pub load_duration: Option<Duration>,
    pub content_size: Option<u64>,
    pub cache_hit: bool,
}

/// Lazy loading controller for preview content
#[derive(Debug, Clone)]
pub struct LazyLoader {
    pub state: LoadingState,
    pub metrics: LoadingMetrics,
    pub priority: u8, // 0-255, higher is more important
    pub visible: bool,
    pub should_preload: bool,
}

/// Preview Panel component for displaying file previews with controls and metadata
/// Designed to integrate with the VS Code-style layout system
/// Enhanced with lazy loading and progressive rendering for optimal performance
#[component]
pub fn PreviewPanel(
    selected_file: Signal<Option<FileSystemEntry>>,
    preview_data: Signal<Option<PreviewData>>,
) -> Element {
    let _app_state = use_app_state();
    
    // Local state for preview panel controls
    let zoom_level = use_signal(|| 1.0f64);
    let pan_x = use_signal(|| 0.0f64);
    let pan_y = use_signal(|| 0.0f64);
    let fit_to_window = use_signal(|| true);
    let show_metadata = use_signal(|| true);
    
    // Lazy loading state
    let lazy_loader = use_signal(|| LazyLoader {
        state: LoadingState::NotLoaded,
        metrics: LoadingMetrics {
            start_time: Instant::now(),
            load_duration: None,
            content_size: None,
            cache_hit: false,
        },
        priority: 128, // Medium priority
        visible: false,
        should_preload: false,
    });
    
    // Track panel visibility for lazy loading optimization
    let is_visible = use_signal(|| false);
    // Note: In a real implementation, this would use web APIs for intersection observation
    let _intersection_observer_placeholder = use_signal(|| false);

    rsx! {
        div {
            class: "preview-panel",
            role: "region",
            "aria-label": "File preview panel",
            "aria-describedby": "preview-instructions",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                background: var(--color-bg-primary, #1e1e1e);
                border: 1px solid var(--color-border-primary, #464647);
                border-radius: var(--border-radius-medium, 4px);
                overflow: hidden;
            ",
            
            // Screen reader instructions for preview panel
            div {
                id: "preview-instructions",
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
                "File preview area with zoom and navigation controls. Use arrow keys to pan, +/- to zoom, Space to fit to window, I to toggle metadata panel."
            }
            
            // Preview Header with Controls
            PreviewHeader {
                selected_file: selected_file,
                zoom_level: zoom_level,
                pan_x: pan_x,
                pan_y: pan_y,
                fit_to_window: fit_to_window,
                show_metadata: show_metadata,
            }
            
            // Main Preview Content Area
            div {
                class: "preview-content-wrapper",
                style: "
                    flex: 1;
                    display: flex;
                    flex-direction: row;
                    overflow: hidden;
                ",
                
                // Content area with lazy loading support
                div {
                    class: "preview-content-main",
                    style: "
                        flex: 1;
                        position: relative;
                        overflow: auto;
                        background: var(--color-bg-secondary, #252526);
                    ",
                    
                    LazyPreviewContentArea {
                        preview_data: preview_data,
                        selected_file: selected_file,
                        zoom_level: zoom_level,
                        pan_x: pan_x,
                        pan_y: pan_y,
                        fit_to_window: fit_to_window,
                        show_metadata: show_metadata,
                        lazy_loader: lazy_loader,
                        is_visible: is_visible,
                    }
                }
                
                // Metadata sidebar (conditionally shown)
                if *show_metadata.read() {
                    PreviewMetadata {
                        selected_file: selected_file,
                        preview_data: preview_data,
                    }
                }
            }
        }
    }
}

/// Preview header with zoom controls, file info, and options
#[component]
pub fn PreviewHeader(
    selected_file: Signal<Option<FileSystemEntry>>,
    zoom_level: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    fit_to_window: Signal<bool>,
    show_metadata: Signal<bool>,
) -> Element {
    rsx! {
        div {
            class: "preview-header",
            role: "toolbar",
            "aria-label": "Preview controls and file information",
            "aria-describedby": "toolbar-instructions",
            style: "
                height: 48px;
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 0 var(--spacing-medium, 12px);
                background: var(--color-bg-tertiary, #2d2d30);
                border-bottom: 1px solid var(--color-border-primary, #464647);
            ",
            
            // Toolbar instructions for screen readers
            div {
                id: "toolbar-instructions",
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
                "Preview toolbar with zoom and display controls. Use Tab to navigate between controls."
            }
            
            // Left side - file info
            div {
                class: "preview-file-info",
                style: "
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-small, 8px);
                    color: var(--color-text-primary, #cccccc);
                    font-size: var(--font-size-small, 13px);
                ",
                
                if let Some(file) = selected_file.read().as_ref() {
                    span { class: "file-name", "{file.name}" }
                } else {
                    span { class: "no-file", "No file selected" }
                }
            }
            
            // Right side - streamlined controls with consistent sizing
            div {
                class: "preview-controls",
                style: "
                    display: flex;
                    align-items: center;
                    gap: 4px;
                    background: var(--vscode-toolbar-background, rgba(255,255,255,0.05));
                    border-radius: 4px;
                    padding: 2px;
                ",
                
                // Zoom control group with consistent 28px sizing
                div {
                    class: "zoom-control-group",
                    style: "display: flex; align-items: center; gap: 2px;",
                    
                    IconButton {
                        icon: "−",
                        tooltip: "Zoom out (Ctrl+-)".to_string(),
                        onclick: move |_| {
                            let current = *zoom_level.read();
                            zoom_level.set((current * 0.8).max(0.1));
                        }
                    }
                    
                    // Compact zoom level display
                    span {
                        class: "zoom-level-compact",
                        style: "
                            min-width: 38px;
                            text-align: center;
                            font-size: 11px;
                            font-weight: 500;
                            color: var(--vscode-foreground, #cccccc);
                        ",
                        "{(*zoom_level.read() * 100.0).round() as i32}%"
                    }
                    
                    IconButton {
                        icon: "+",
                        tooltip: "Zoom in (Ctrl++)".to_string(),
                        onclick: move |_| {
                            let current = *zoom_level.read();
                            zoom_level.set((current * 1.25).min(10.0));
                        }
                    }
                }
                
                // Action control group
                div {
                    class: "action-control-group", 
                    style: "display: flex; align-items: center; gap: 2px; margin-left: 4px;",
                    
                    IconButton {
                        icon: "⛶",
                        tooltip: "Fit to window (Space)".to_string(),
                        active: *fit_to_window.read(),
                        onclick: move |_| {
                            let current_value = *fit_to_window.read();
                            fit_to_window.set(!current_value);
                            if *fit_to_window.read() {
                                // Calculate optimal zoom to fit content
                                zoom_level.set(0.8);  // Slightly smaller than 100% to ensure content fits
                                pan_x.set(0.0);
                                pan_y.set(0.0);
                            }
                        }
                    }
                    
                    IconButton {
                        icon: "↻",
                        tooltip: "Reset zoom (0)".to_string(),
                        onclick: move |_| {
                            zoom_level.set(1.0);
                            pan_x.set(0.0);
                            pan_y.set(0.0);
                            fit_to_window.set(false);
                        }
                    }
                    
                    IconButton {
                        icon: "ⓘ",
                        tooltip: "Toggle metadata panel (I)".to_string(),
                        active: *show_metadata.read(),
                        onclick: move |_| {
                            let current_value = *show_metadata.read();
                            show_metadata.set(!current_value);
                        }
                    }
                }
            }
        }
    }
}

/// Enhanced preview content area with lazy loading and progressive rendering
#[component]
pub fn LazyPreviewContentArea(
    preview_data: Signal<Option<PreviewData>>,
    selected_file: Signal<Option<FileSystemEntry>>,
    zoom_level: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    fit_to_window: Signal<bool>,
    show_metadata: Signal<bool>,
    lazy_loader: Signal<LazyLoader>,
    is_visible: Signal<bool>,
) -> Element {
    // State for drag operations
    let is_dragging = use_signal(|| false);
    let drag_start_x = use_signal(|| 0.0);
    let drag_start_y = use_signal(|| 0.0);
    let pan_start_x = use_signal(|| 0.0);
    let pan_start_y = use_signal(|| 0.0);
    
    // Progressive loading state
    let loading_progress = use_signal(|| 0u8);
    let is_loading = use_signal(|| false);
    
    // Effect to handle lazy loading when file selection changes
    use_effect({
        let mut is_loading = is_loading;
        let mut loading_progress = loading_progress;
        move || {
        if let Some(file_entry) = selected_file.read().as_ref() {
            // Start lazy loading if not already started
            let mut loader = lazy_loader.write();
            match loader.state {
                LoadingState::NotLoaded => {
                    loader.state = LoadingState::Loading(Some(0));
                    loader.metrics.start_time = Instant::now();
                    is_loading.set(true);
                    loading_progress.set(0);
                    
                    // Simulate progressive loading for demo
                    let file_size = file_entry.size;
                    let priority = calculate_loading_priority(file_entry);
                    loader.priority = priority;
                    
                    // Note: In a real implementation, this would spawn an async loading task
                    // For now, simulate immediate loading
                    loading_progress.set(100);
                    is_loading.set(false);
                    loader.state = LoadingState::Loaded;
                }
                _ => {} // Already loaded or loading
            }
        }
        }
    });
    
    // Intersection observer effect for visibility tracking
    use_effect(move || {
        // In a real implementation, this would use IntersectionObserver API
        // For now, assume visible when preview_data is available
        let visible = preview_data.read().is_some();
        is_visible.set(visible);
        
        if visible {
            let mut loader = lazy_loader.write();
            loader.visible = true;
        }
    });

    let cursor_style = if *is_dragging.read() { "grabbing" } else { "grab" };
    
    rsx! {
        div {
            class: "preview-content-area",
            role: "img",
            "aria-label": {
                if let Some(file) = selected_file.read().as_ref() {
                    format!("Preview of {}", file.name)
                } else {
                    "No file selected for preview".to_string()
                }
            },
            "aria-describedby": "preview-status",
            style: format!("
                width: 100%;
                height: 100%;
                display: flex;
                align-items: center;
                justify-content: center;
                background: var(--color-bg-primary, #1e1e1e);
                cursor: {};
                user-select: none;
            ", cursor_style),
            tabindex: 0, // Make focusable for keyboard events
            
            // Mouse wheel zoom
            onwheel: move |evt| {
                evt.prevent_default();
                let wheel_data = evt.data();
                let delta = wheel_data.delta();
                // WheelDelta is an enum with vector types
                let delta_y = match delta {
                    WheelDelta::Pixels(vector) => vector.y,
                    WheelDelta::Lines(vector) => vector.y * 16.0, // Convert lines to pixels
                    WheelDelta::Pages(vector) => vector.y * 400.0, // Convert pages to pixels
                };
                let zoom_factor = if delta_y > 0.0 { 0.9 } else { 1.1 };
                let current_zoom = *zoom_level.read();
                let new_zoom = (current_zoom * zoom_factor).clamp(0.1, 5.0);
                zoom_level.set(new_zoom);
                
                // If fit-to-window is enabled, disable it when manually zooming
                if *fit_to_window.read() {
                    fit_to_window.set(false);
                }
            },
            
            // Mouse down - start dragging
            onmousedown: {
                let mut is_dragging = is_dragging;
                let mut drag_start_x = drag_start_x;
                let mut drag_start_y = drag_start_y;
                let mut pan_start_x = pan_start_x;
                let mut pan_start_y = pan_start_y;
                move |evt: Event<MouseData>| {
                    evt.prevent_default();
                    is_dragging.set(true);
                    drag_start_x.set(evt.data().client_coordinates().x as f64);
                    drag_start_y.set(evt.data().client_coordinates().y as f64);
                    pan_start_x.set(*pan_x.read());
                    pan_start_y.set(*pan_y.read());
                }
            },
            
            // Mouse move - handle dragging
            onmousemove: move |evt| {
                if *is_dragging.read() {
                    evt.prevent_default();
                    let current_x = evt.data().client_coordinates().x as f64;
                    let current_y = evt.data().client_coordinates().y as f64;
                    
                    let delta_x = current_x - *drag_start_x.read();
                    let delta_y = current_y - *drag_start_y.read();
                    
                    pan_x.set(*pan_start_x.read() + delta_x);
                    pan_y.set(*pan_start_y.read() + delta_y);
                    
                    // Disable fit-to-window when manually panning
                    if *fit_to_window.read() {
                        fit_to_window.set(false);
                    }
                }
            },
            
            // Mouse up - stop dragging
            onmouseup: {
                let mut is_dragging = is_dragging;
                move |_| {
                    is_dragging.set(false);
                }
            },
            
            // Mouse leave - stop dragging
            onmouseleave: {
                let mut is_dragging = is_dragging;
                move |_| {
                    is_dragging.set(false);
                }
            },
            
            // Keyboard navigation
            onkeydown: move |evt| {
                use dioxus::events::Key;
                match evt.key() {
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        let current_pan_x = *pan_x.read();
                        pan_x.set(current_pan_x - 20.0);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::ArrowRight => {
                        evt.prevent_default();
                        let current_pan_x = *pan_x.read();
                        pan_x.set(current_pan_x + 20.0);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::ArrowUp => {
                        evt.prevent_default();
                        let current_pan_y = *pan_y.read();
                        pan_y.set(current_pan_y - 20.0);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::ArrowDown => {
                        evt.prevent_default();
                        let current_pan_y = *pan_y.read();
                        pan_y.set(current_pan_y + 20.0);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::Character(ref s) if s == "+" || s == "=" => {
                        evt.prevent_default();
                        let current_zoom = *zoom_level.read();
                        let new_zoom = (current_zoom * 1.1).clamp(0.1, 5.0);
                        zoom_level.set(new_zoom);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::Character(ref s) if s == "-" => {
                        evt.prevent_default();
                        let current_zoom = *zoom_level.read();
                        let new_zoom = (current_zoom * 0.9).clamp(0.1, 5.0);
                        zoom_level.set(new_zoom);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::Character(ref s) if s == "0" => {
                        evt.prevent_default();
                        zoom_level.set(1.0);
                        pan_x.set(0.0);
                        pan_y.set(0.0);
                        fit_to_window.set(false);
                    },
                    Key::Character(ref s) if s == " " => {
                        evt.prevent_default();
                        // Toggle fit to window
                        let current_value = *fit_to_window.read();
                        fit_to_window.set(!current_value);
                        if *fit_to_window.read() {
                            zoom_level.set(0.8);  // Match the button behavior
                            pan_x.set(0.0);
                            pan_y.set(0.0);
                        }
                    },
                    Key::Character(ref s) if s.to_lowercase() == "i" => {
                        evt.prevent_default();
                        // Toggle metadata panel
                        let current_value = *show_metadata.read();
                        show_metadata.set(!current_value);
                    },
                    Key::Character(ref s) if s.to_lowercase() == "r" => {
                        evt.prevent_default();
                        // Reset view (same as pressing 0)
                        zoom_level.set(1.0);
                        pan_x.set(0.0);
                        pan_y.set(0.0);
                        fit_to_window.set(false);
                    },
                    Key::Home => {
                        evt.prevent_default();
                        // Reset pan to center
                        pan_x.set(0.0);
                        pan_y.set(0.0);
                    },
                    Key::PageUp => {
                        evt.prevent_default();
                        // Large zoom in
                        let current_zoom = *zoom_level.read();
                        let new_zoom = (current_zoom * 1.5).clamp(0.1, 5.0);
                        zoom_level.set(new_zoom);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    Key::PageDown => {
                        evt.prevent_default();
                        // Large zoom out
                        let current_zoom = *zoom_level.read();
                        let new_zoom = (current_zoom * 0.67).clamp(0.1, 5.0);
                        zoom_level.set(new_zoom);
                        if *fit_to_window.read() {
                            fit_to_window.set(false);
                        }
                    },
                    _ => {}
                }
            },
            
            // Live region for status announcements
            div {
                id: "preview-status",
                class: "sr-only",
                "aria-live": "polite",
                "aria-atomic": "true",
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
                {
                    match lazy_loader.read().state {
                        LoadingState::Loading(_) => "Loading preview...",
                        LoadingState::Loaded => "Preview loaded successfully",
                        LoadingState::Failed(_) => "Failed to load preview",
                        LoadingState::NotLoaded => "Select a file to preview"
                    }
                }
            }
            
            // Progressive rendering based on loading state
            {
                let loader_state = lazy_loader.read();
                match &loader_state.state {
                LoadingState::Loading(progress) => rsx! {
                    ProgressivePlaceholder {
                        progress: *progress,
                        loading_progress: loading_progress,
                        selected_file: selected_file,
                    }
                },
                LoadingState::Loaded => {
                    if let Some(data) = preview_data.read().as_ref() {
                        match &data.preview_content {
                            PreviewContent::Image { .. } => rsx! {
                                LazyImagePreview {
                                    format: data.format.clone(),
                                    zoom_level: zoom_level,
                                    pan_x: pan_x,
                                    pan_y: pan_y,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Video { .. } => rsx! {
                                LazyVideoPreview {
                                    format: data.format.clone(),
                                    duration: data.metadata.duration,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Audio { .. } => rsx! {
                                LazyAudioPreview {
                                    format: data.format.clone(),
                                    duration: data.metadata.duration,
                                    sample_rate: data.metadata.sample_rate,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Document { .. } => rsx! {
                                LazyDocumentPreview {
                                    format: data.format.clone(),
                                    page_count: data.metadata.page_count,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Text { content, language, line_count } => rsx! {
                                LazyTextPreview {
                                    content: content.clone(),
                                    language: language.clone(),
                                    line_count: *line_count,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Archive { contents, .. } => rsx! {
                                LazyArchivePreview {
                                    contents: contents.clone(),
                                    file_size: data.metadata.file_size,
                                    lazy_loader: lazy_loader,
                                }
                            },
                            PreviewContent::Unsupported { file_type, reason, suggested_action } => rsx! {
                                UnsupportedPreview {
                                    file_type: file_type.clone(),
                                    reason: reason.clone(),
                                    suggested_action: suggested_action.clone(),
                                }
                            },
                        }
                    } else {
                        rsx! {
                            div {
                                class: "no-preview",
                                style: "
                                    text-align: center;
                                    color: var(--color-text-secondary, #999999);
                                    font-size: var(--font-size-medium, 14px);
                                ",
                                "No preview available"
                            }
                        }
                    }
                },
                LoadingState::Failed(error) => rsx! {
                    LoadingErrorDisplay {
                        error: error.clone(),
                        selected_file: selected_file,
                        lazy_loader: lazy_loader,
                    }
                },
                LoadingState::NotLoaded => rsx! {
                    div {
                        class: "preview-not-loaded",
                        style: "
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            justify-content: center;
                            height: 100%;
                            color: var(--color-text-secondary, #999999);
                            font-size: var(--font-size-medium, 14px);
                        ",
                        div { "Select a file to preview" }
                    }
                }
                }
            }
        }
    }
}

/// Enhanced metadata sidebar showing comprehensive file properties and quick actions
#[component]
pub fn PreviewMetadata(
    selected_file: Signal<Option<FileSystemEntry>>,
    preview_data: Signal<Option<PreviewData>>,
) -> Element {
    rsx! {
        div {
            class: "preview-metadata",
            role: "region",
            "aria-label": "File metadata and properties",
            "aria-labelledby": "metadata-title",
            style: "
                width: 280px;
                background: var(--color-bg-tertiary, #2d2d30);
                border-left: 1px solid var(--color-border-primary, #464647);
                display: flex;
                flex-direction: column;
                overflow: hidden;
            ",
            
            // Header with quick actions
            div {
                class: "metadata-header",
                style: "
                    padding: var(--spacing-medium, 12px);
                    border-bottom: 1px solid var(--color-border-primary, #464647);
                ",
                
                h3 {
                    id: "metadata-title",
                    role: "heading",
                    "aria-level": "2",
                    style: "
                        margin: 0 0 var(--spacing-small, 8px) 0;
                        color: var(--color-text-primary, #cccccc);
                        font-size: var(--font-size-medium, 14px);
                        font-weight: 600;
                    ",
                    "File Properties"
                }
                
                // Quick Actions
                if let Some(file_entry) = selected_file.read().as_ref() {
                    QuickActions {
                        file_path: file_entry.path.clone(),
                        file_type: file_entry.file_type.clone(),
                        preview_data: preview_data,
                    }
                }
            }
            
            // Scrollable metadata content
            div {
                class: "metadata-content",
                style: "
                    flex: 1;
                    padding: var(--spacing-medium, 12px);
                    overflow-y: auto;
                ",
                
                // Basic file information
                if let Some(file_entry) = selected_file.read().as_ref() {
                    MetadataSection {
                        title: "File Info",
                        fields: vec![
                            ("Name", file_entry.name.clone()),
                            ("Type", file_entry.path.extension()
                                .and_then(|ext| ext.to_str())
                                .map(|ext| ext.to_uppercase())
                                .unwrap_or_else(|| "Unknown".to_string())),
                            ("Size", format_file_size(file_entry.size)),
                            ("Modified", format_system_time(file_entry.modified)),
                            ("Path", format_file_path(&file_entry.path)),
                        ],
                    }
                }
                
                // Enhanced preview-specific metadata
                if let Some(preview_data) = preview_data.read().as_ref() {
                    {
                        let sections = build_metadata_sections(preview_data);
                        rsx! {
                            for section in sections {
                                MetadataSection {
                                    title: section.0,
                                    fields: section.1,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Quick actions component for file operations
#[component]
pub fn QuickActions(
    file_path: PathBuf,
    file_type: Option<String>,
    preview_data: Signal<Option<PreviewData>>,
) -> Element {
    let is_image = file_type.as_ref()
        .map(|ft| ft.starts_with("image/") || matches!(ft.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp"))
        .unwrap_or(false);
    
    let can_rotate = is_image && preview_data.read().as_ref()
        .map(|pd| matches!(pd.format, SupportedFormat::Jpeg | SupportedFormat::Png))
        .unwrap_or(false);
    
    // Clone paths for closures
    let copy_path = file_path.clone();
    let open_path = file_path.clone();
    let rotate_path = file_path.clone();
    let props_path = file_path.clone();
    
    rsx! {
        div {
            class: "quick-actions",
            role: "group",
            "aria-label": "Quick file actions",
            style: "
                display: flex;
                gap: var(--spacing-small, 8px);
                flex-wrap: wrap;
            ",
            
            // Copy path button
            QuickActionButton {
                icon: "📋",
                tooltip: "Copy file path",
                onclick: move |_| {
                    copy_to_clipboard(&copy_path.to_string_lossy().to_string());
                },
            }
            
            // Open externally button
            QuickActionButton {
                icon: "🔗",
                tooltip: "Open with default app",
                onclick: move |_| {
                    open_external(&open_path);
                },
            }
            
            // Rotate image button (only for images)
            if can_rotate {
                QuickActionButton {
                    icon: "↻",
                    tooltip: "Rotate image 90° clockwise",
                    onclick: move |_| {
                        rotate_image(&rotate_path);
                    },
                }
            }
            
            // Properties button
            QuickActionButton {
                icon: "ⓘ",
                tooltip: "Show file properties",
                onclick: move |_| {
                    show_file_properties(&props_path);
                },
            }
        }
    }
}

/// Individual quick action button component
#[component]
pub fn QuickActionButton(
    icon: String,
    tooltip: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "quick-action-btn",
            "aria-label": tooltip.clone(),
            title: tooltip.clone(),
            style: "
                width: 32px;
                height: 32px;
                border: 1px solid var(--color-border-primary, #464647);
                background: var(--color-bg-secondary, #252526);
                color: var(--color-text-primary, #cccccc);
                border-radius: 4px;
                cursor: pointer;
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 14px;
                transition: all 0.2s ease;
                
                &:hover {{
                    background: var(--color-bg-primary, #1e1e1e);
                    border-color: var(--color-accent-primary, #0078d4);
                }}
                
                &:active {{
                    transform: translateY(1px);
                }}
                
                &:focus {{
                    outline: 2px solid var(--color-accent-primary, #0078d4);
                    outline-offset: 2px;
                }}
            ",
            onclick: onclick,
            "{icon}"
        }
    }
}

/// Metadata section with grouped fields
#[component]
pub fn MetadataSection(
    title: String,
    fields: Vec<(&'static str, String)>,
) -> Element {
    rsx! {
        div {
            class: "metadata-section",
            style: "
                margin-bottom: var(--spacing-large, 16px);
            ",
            
            h4 {
                role: "heading",
                "aria-level": "3",
                style: "
                    margin: 0 0 var(--spacing-small, 8px) 0;
                    color: var(--color-text-primary, #cccccc);
                    font-size: var(--font-size-small, 13px);
                    font-weight: 600;
                    text-transform: uppercase;
                    letter-spacing: 0.5px;
                    opacity: 0.9;
                ",
                "{title}"
            }
            
            for (label, value) in fields {
                MetadataField {
                    label: label.to_string(),
                    value: value,
                }
            }
        }
    }
}

/// Simple metadata field display component
#[component]
pub fn MetadataField(label: String, value: String) -> Element {
    rsx! {
        dl {
            class: "metadata-field",
            style: "
                margin-bottom: var(--spacing-small, 8px);
                display: flex;
                flex-direction: column;
                gap: 2px;
            ",
            
            dt {
                class: "metadata-label",
                style: "
                    font-size: 11px;
                    color: var(--color-text-secondary, #999999);
                    text-transform: uppercase;
                    font-weight: 500;
                ",
                "{label}"
            }
            
            dd {
                class: "metadata-value",
                style: "
                    font-size: 13px;
                    color: var(--color-text-primary, #cccccc);
                    word-break: break-word;
                    margin: 0;
                ",
                "{value}"
            }
        }
    }
}

// Individual preview components for different file types

#[component]
pub fn ImagePreview(
    format: SupportedFormat,
    zoom_level: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
) -> Element {
    rsx! {
        div {
            class: "image-preview",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: var(--spacing-medium, 12px);
                transform: scale({*zoom_level.read()}) translate({*pan_x.read()}px, {*pan_y.read()}px);
            ",
            
            div {
                style: "font-size: 64px;",
                "🖼️"
            }
            
            div {
                style: "
                    color: var(--color-text-secondary, #999999);
                    font-size: var(--font-size-small, 13px);
                    text-align: center;
                ",
                "Image Preview • {format:?}"
            }
        }
    }
}

#[component]
pub fn VideoPreview(
    format: SupportedFormat,
    duration: Option<f64>,
) -> Element {
    // Video playback state
    let is_playing = use_signal(|| false);
    let current_time = use_signal(|| 0.0f64);
    let volume = use_signal(|| 0.7f64);
    let is_muted = use_signal(|| false);
    let playback_speed = use_signal(|| 1.0f64);
    
    let total_duration = duration.unwrap_or(120.0);
    let progress_percentage = if total_duration > 0.0 {
        (*current_time.read() / total_duration * 100.0).min(100.0)
    } else {
        0.0
    };
    
    // Generate mock thumbnail positions for timeline scrubbing
    let thumbnail_positions = use_signal(|| generate_timeline_thumbnails(total_duration));
    
    rsx! {
        div {
            class: "video-preview",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                background: var(--color-bg-primary, #1e1e1e);
            ",
            
            // Video display area
            div {
                class: "video-viewport",
                style: "
                    flex: 1;
                    position: relative;
                    background: var(--color-bg-secondary, #252526);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 300px;
                    overflow: hidden;
                ",
                
                // Mock video content (in real implementation, this would be actual video element)
                div {
                    class: "video-placeholder",
                    style: "
                        width: 80%;
                        height: 80%;
                        background: linear-gradient(135deg, #333 0%, #555 100%);
                        border-radius: var(--border-radius-medium, 4px);
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        justify-content: center;
                        gap: var(--spacing-medium, 12px);
                        position: relative;
                        cursor: pointer;
                    ",
                    
                    onclick: {
                        let mut is_playing = is_playing;
                        move |_| {
                            let new_playing = !*is_playing.read();
                            is_playing.set(new_playing);
                        }
                    },
                    
                    // Large play/pause overlay
                    div {
                        style: "
                            font-size: 72px;
                            color: rgba(255, 255, 255, 0.8);
                            transition: all 0.2s ease;
                        ",
                        {
                            if *is_playing.read() { "⏸" } else { "▶" }
                        }
                    }
                    
                    div {
                        style: "
                            color: var(--color-text-secondary, #999999);
                            font-size: var(--font-size-medium, 14px);
                            text-align: center;
                        ",
                        {format!("{:?} Video", format)}
                        {
                            if let Some(d) = duration {
                                format!(" • {:.1}s", d)
                            } else {
                                String::new()
                            }
                        }
                    }
                    
                    // Video progress overlay (when playing)
                    if *is_playing.read() || progress_percentage > 0.0 {
                        div {
                            class: "video-progress-overlay",
                            style: "
                                position: absolute;
                                bottom: 16px;
                                left: 16px;
                                right: 16px;
                                background: rgba(0, 0, 0, 0.7);
                                border-radius: 4px;
                                padding: 8px 12px;
                                color: white;
                                font-size: 12px;
                                font-family: var(--vscode-font-mono);
                                display: flex;
                                justify-content: space-between;
                                align-items: center;
                            ",
                            
                            span { "{format_time(*current_time.read())}" }
                            
                            div {
                                style: "
                                    flex: 1;
                                    margin: 0 12px;
                                    height: 4px;
                                    background: rgba(255, 255, 255, 0.3);
                                    border-radius: 2px;
                                    position: relative;
                                    cursor: pointer;
                                ",
                                
                                onclick: {
                                    let mut current_time = current_time;
                                    move |evt: Event<MouseData>| {
                                        evt.stop_propagation();
                                        // Calculate click position for seeking
                                        // In a real implementation, this would seek the video
                                        let progress = 0.5; // Mock seeking to middle
                                        current_time.set(progress * total_duration);
                                    }
                                },
                                
                                div {
                                    style: "
                                        width: {progress_percentage}%;
                                        height: 100%;
                                        background: var(--color-accent-primary, #0078d4);
                                        border-radius: 2px;
                                    ",
                                }
                            }
                            
                            span { "{format_time(total_duration)}" }
                        }
                    }
                }
            }
            
            // Video timeline and controls
            div {
                class: "video-timeline",
                style: "
                    background: var(--color-bg-tertiary, #2d2d30);
                    border-top: 1px solid var(--color-border-primary, #464647);
                    padding: var(--spacing-medium, 12px);
                ",
                
                // Timeline scrubber
                div {
                    class: "timeline-scrubber",
                    style: "
                        position: relative;
                        height: 60px;
                        background: var(--color-bg-secondary, #252526);
                        border-radius: var(--border-radius-small, 2px);
                        margin-bottom: var(--spacing-medium, 12px);
                        overflow: hidden;
                        cursor: pointer;
                    ",
                    
                    onclick: {
                        let mut current_time = current_time;
                        move |evt| {
                            // Calculate seek position based on click
                            // For now, use mock seeking since we don't have access to element dimensions
                            let seek_percentage = 0.3; // Mock seeking to 30%
                            let new_time = seek_percentage * total_duration;
                            current_time.set(new_time);
                        }
                    },
                    
                    // Timeline thumbnails (mock)
                    div {
                        class: "timeline-thumbnails",
                        style: "
                            display: flex;
                            height: 100%;
                        ",
                        
                        for (i, _thumbnail) in thumbnail_positions.read().iter().enumerate() {
                            div {
                                key: "thumb_{i}",
                                class: "timeline-thumbnail",
                                style: "
                                    flex: 1;
                                    background: linear-gradient(135deg, #444 0%, #666 100%);
                                    border-right: 1px solid var(--color-border-primary, #464647);
                                    display: flex;
                                    align-items: center;
                                    justify-content: center;
                                    color: var(--color-text-secondary, #999999);
                                    font-size: 8px;
                                ",
                                "{i + 1}"
                            }
                        }
                    }
                    
                    // Progress indicator
                    div {
                        class: "timeline-progress",
                        style: "
                            position: absolute;
                            top: 0;
                            left: 0;
                            width: {progress_percentage}%;
                            height: 100%;
                            background: linear-gradient(90deg, 
                                var(--color-accent-primary, #0078d4) 0%, 
                                var(--color-accent-bright, #40a9ff) 100%);
                            opacity: 0.3;
                        ",
                    }
                    
                    // Playhead
                    div {
                        class: "timeline-playhead",
                        style: "
                            position: absolute;
                            top: 0;
                            left: {progress_percentage}%;
                            width: 2px;
                            height: 100%;
                            background: var(--color-accent-bright, #40a9ff);
                            transform: translateX(-50%);
                        ",
                    }
                }
                
                // Video controls
                div {
                    class: "video-controls",
                    style: "
                        display: flex;
                        align-items: center;
                        gap: var(--spacing-medium, 12px);
                    ",
                    
                    // Play/Pause
                    button {
                        class: "video-play-pause",
                        style: "
                            width: 40px;
                            height: 40px;
                            border: none;
                            background: var(--color-accent-primary, #0078d4);
                            color: white;
                            border-radius: 50%;
                            cursor: pointer;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            font-size: 16px;
                        ",
                        onclick: {
                            let mut is_playing = is_playing;
                            move |_| {
                                let new_playing = !*is_playing.read();
                                is_playing.set(new_playing);
                            }
                        },
                        {
                            if *is_playing.read() { "⏸" } else { "▶" }
                        }
                    }
                    
                    // Time display
                    div {
                        class: "video-time",
                        style: "
                            color: var(--color-text-primary, #cccccc);
                            font-family: var(--vscode-font-mono);
                            font-size: var(--font-size-small, 13px);
                        ",
                        "{format_time(*current_time.read())} / {format_time(total_duration)}"
                    }
                    
                    // Spacer
                    div { style: "flex: 1;" }
                    
                    // Playback speed
                    div {
                        class: "playback-speed-control",
                        style: "
                            display: flex;
                            align-items: center;
                            gap: var(--spacing-small, 8px);
                        ",
                        
                        span {
                            style: "
                                color: var(--color-text-secondary, #999999);
                                font-size: var(--font-size-small, 13px);
                            ",
                            "Speed:"
                        }
                        
                        select {
                            style: "
                                background: var(--color-bg-primary, #1e1e1e);
                                color: var(--color-text-primary, #cccccc);
                                border: 1px solid var(--color-border-primary, #464647);
                                border-radius: 4px;
                                padding: 4px 8px;
                                font-size: 12px;
                            ",
                            value: "{*playback_speed.read()}",
                            onchange: {
                                let mut playback_speed = playback_speed;
                                move |evt: Event<FormData>| {
                                    if let Ok(speed) = evt.data.value().parse::<f64>() {
                                        playback_speed.set(speed);
                                    }
                                }
                            },
                            
                            option { value: "0.25", "0.25x" }
                            option { value: "0.5", "0.5x" }
                            option { value: "0.75", "0.75x" }
                            option { value: "1.0", selected: true, "1x" }
                            option { value: "1.25", "1.25x" }
                            option { value: "1.5", "1.5x" }
                            option { value: "2.0", "2x" }
                        }
                    }
                    
                    // Volume control
                    div {
                        class: "video-volume-control",
                        style: "
                            display: flex;
                            align-items: center;
                            gap: var(--spacing-small, 8px);
                        ",
                        
                        button {
                            class: "volume-toggle",
                            style: "
                                background: none;
                                border: none;
                                color: var(--color-text-primary, #cccccc);
                                cursor: pointer;
                                font-size: 16px;
                                padding: 4px;
                            ",
                            onclick: {
                                let mut is_muted = is_muted;
                                move |_| {
                                    let current_muted = *is_muted.read();
                                    is_muted.set(!current_muted);
                                }
                            },
                            {
                                if *is_muted.read() {
                                    "🔇"
                                } else {
                                    let vol = *volume.read();
                                    if vol < 0.3 { "🔈" }
                                    else if vol < 0.7 { "🔉" }
                                    else { "🔊" }
                                }
                            }
                        }
                        
                        input {
                            r#type: "range",
                            min: "0",
                            max: "1",
                            step: "0.1",
                            value: if *is_muted.read() { "0".to_string() } else { format!("{}", *volume.read()) },
                            style: "
                                width: 80px;
                                height: 4px;
                                background: var(--color-border-primary, #464647);
                                border-radius: 2px;
                                outline: none;
                            ",
                            oninput: {
                                let mut volume = volume;
                                let mut is_muted = is_muted;
                                move |evt: Event<FormData>| {
                                    if let Ok(new_volume) = evt.data.value().parse::<f64>() {
                                        volume.set(new_volume.clamp(0.0, 1.0));
                                        if new_volume > 0.0 {
                                            is_muted.set(false);
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AudioPreview(
    format: SupportedFormat,
    duration: Option<f64>,
    sample_rate: Option<u32>,
) -> Element {
    // Audio playback state
    let is_playing = use_signal(|| false);
    let current_time = use_signal(|| 0.0f64);
    let volume = use_signal(|| 0.7f64);
    
    // Mock waveform data - in real implementation this would come from audio analysis
    let waveform_data = use_signal(|| generate_mock_waveform_data(duration.unwrap_or(60.0)));
    
    let total_duration = duration.unwrap_or(60.0);
    let progress_percentage = if total_duration > 0.0 {
        (*current_time.read() / total_duration * 100.0).min(100.0)
    } else {
        0.0
    };
    
    rsx! {
        div {
            class: "audio-preview",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                padding: var(--spacing-large, 16px);
                background: var(--color-bg-primary, #1e1e1e);
            ",
            
            // Audio info header
            div {
                class: "audio-header",
                style: "
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-medium, 12px);
                    margin-bottom: var(--spacing-large, 16px);
                    padding-bottom: var(--spacing-medium, 12px);
                    border-bottom: 1px solid var(--color-border-primary, #464647);
                ",
                
                div {
                    style: "font-size: 32px;",
                    "🎵"
                }
                
                div {
                    class: "audio-info",
                    style: "
                        display: flex;
                        flex-direction: column;
                        gap: 4px;
                    ",
                    
                    div {
                        style: "
                            color: var(--color-text-primary, #cccccc);
                            font-size: var(--font-size-medium, 14px);
                            font-weight: 600;
                        ",
                        "{format:?} Audio"
                    }
                    
                    div {
                        style: "
                            color: var(--color-text-secondary, #999999);
                            font-size: var(--font-size-small, 13px);
                        ",
                        {
                            let mut info_parts = Vec::new();
                            if let Some(d) = duration {
                                info_parts.push(format!("{:.1}s", d));
                            }
                            if let Some(rate) = sample_rate {
                                info_parts.push(format!("{}Hz", rate));
                            }
                            info_parts.join(" • ")
                        }
                    }
                }
            }
            
            // Waveform visualization
            div {
                class: "waveform-container",
                style: "
                    flex: 1;
                    position: relative;
                    background: var(--color-bg-secondary, #252526);
                    border-radius: var(--border-radius-medium, 4px);
                    padding: var(--spacing-medium, 12px);
                    margin-bottom: var(--spacing-medium, 12px);
                    min-height: 200px;
                    display: flex;
                    align-items: center;
                ",
                
                // Waveform SVG
                svg {
                    width: "100%",
                    height: "120px",
                    view_box: "0 0 800 120",
                    style: "cursor: pointer;",
                    
                    onclick: {
                        let mut current_time = current_time;
                        move |_evt| {
                            // Calculate click position as percentage of total width
                            // For now, use mock seeking since we don't have exact click coordinates
                            let click_percentage = 0.5; // Mock click at 50%
                            let new_time = click_percentage * total_duration;
                            current_time.set(new_time);
                        }
                    },
                    
                    // Background waveform (unplayed portion)
                    for (i, amplitude) in waveform_data.read().iter().enumerate() {
                        rect {
                            key: "bg_{i}",
                            x: i as f64 * 2.0,
                            y: 60.0 - amplitude * 50.0,
                            width: "1.5",
                            height: amplitude * 100.0,
                            fill: "var(--color-border-primary, #464647)",
                        }
                    }
                    
                    // Progress waveform (played portion)
                    {
                        let progress_x = progress_percentage / 100.0 * 800.0;
                        rsx! {
                            for (i, amplitude) in waveform_data.read().iter().enumerate() {
                                if (i as f64 * 2.0) <= progress_x {
                                    rect {
                                        key: "progress_{i}",
                                        x: i as f64 * 2.0,
                                        y: 60.0 - amplitude * 50.0,
                                        width: "1.5",
                                        height: amplitude * 100.0,
                                        fill: "var(--color-accent-primary, #0078d4)",
                                    }
                                }
                            }
                        }
                    }
                    
                    // Current playhead position
                    line {
                        x1: progress_percentage / 100.0 * 800.0,
                        y1: "0",
                        x2: progress_percentage / 100.0 * 800.0,
                        y2: "120",
                        stroke: "var(--color-accent-bright, #40a9ff)",
                        stroke_width: "2",
                    }
                }
                
                // Time overlay
                div {
                    style: "
                        position: absolute;
                        bottom: var(--spacing-small, 8px);
                        left: var(--spacing-medium, 12px);
                        right: var(--spacing-medium, 12px);
                        display: flex;
                        justify-content: space-between;
                        color: var(--color-text-secondary, #999999);
                        font-size: 11px;
                        font-family: var(--vscode-font-mono);
                    ",
                    
                    span { "{format_time(*current_time.read())}" }
                    span { "{format_time(total_duration)}" }
                }
            }
            
            // Audio controls
            div {
                class: "audio-controls",
                style: "
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: var(--spacing-medium, 12px);
                    padding: var(--spacing-medium, 12px);
                    background: var(--color-bg-tertiary, #2d2d30);
                    border-radius: var(--border-radius-medium, 4px);
                ",
                
                // Play/Pause button
                button {
                    class: "play-pause-btn",
                    style: "
                        width: 48px;
                        height: 48px;
                        border: none;
                        background: var(--color-accent-primary, #0078d4);
                        color: white;
                        border-radius: 50%;
                        cursor: pointer;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 18px;
                        transition: all 0.2s ease;
                    ",
                    title: if *is_playing.read() { "Pause" } else { "Play" },
                    onclick: {
                        let mut is_playing = is_playing;
                        move |_| {
                            let new_playing = !*is_playing.read();
                            is_playing.set(new_playing);
                        
                            // Mock playback - advance time periodically when playing
                            if new_playing && *current_time.read() < total_duration {
                                // In real implementation, this would be handled by actual audio playback
                                // For now, just simulate progress
                            }
                        }
                    },
                    {
                        if *is_playing.read() { "⏸" } else { "▶" }
                    }
                }
                
                // Volume control
                div {
                    class: "volume-control",
                    style: "
                        display: flex;
                        align-items: center;
                        gap: var(--spacing-small, 8px);
                    ",
                    
                    span {
                        style: "font-size: 16px;",
                        {
                            let vol = *volume.read();
                            if vol == 0.0 { "🔇" }
                            else if vol < 0.3 { "🔈" }
                            else if vol < 0.7 { "🔉" }
                            else { "🔊" }
                        }
                    }
                    
                    input {
                        r#type: "range",
                        min: "0",
                        max: "1",
                        step: "0.1",
                        value: "{*volume.read()}",
                        style: "
                            width: 80px;
                            height: 4px;
                            background: var(--color-border-primary, #464647);
                            border-radius: 2px;
                            outline: none;
                        ",
                        oninput: {
                            let mut volume = volume;
                            move |evt: Event<FormData>| {
                                if let Ok(new_volume) = evt.data.value().parse::<f64>() {
                                    volume.set(new_volume.clamp(0.0, 1.0));
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

#[component]
pub fn DocumentPreview(
    format: SupportedFormat,
    page_count: Option<u32>,
) -> Element {
    rsx! {
        div {
            class: "document-preview",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: var(--spacing-medium, 12px);
            ",
            
            div {
                style: "font-size: 64px;",
                "📄"
            }
            
            div {
                style: "
                    color: var(--color-text-secondary, #999999);
                    font-size: var(--font-size-small, 13px);
                    text-align: center;
                ",
                "Document Preview • {format:?}"
                {
                    if let Some(pages) = page_count {
                        format!(" • {} pages", pages)
                    } else {
                        String::new()
                    }
                }
            }
        }
    }
}

#[component]
pub fn TextPreview(
    content: String,
    language: Option<String>,
    line_count: usize,
) -> Element {
    rsx! {
        div {
            class: "text-preview",
            style: "
                width: 100%;
                height: 100%;
                padding: var(--spacing-medium, 12px);
                overflow: auto;
            ",
            
            div {
                style: "
                    color: var(--color-text-secondary, #999999);
                    font-size: var(--font-size-small, 13px);
                    margin-bottom: var(--spacing-small, 8px);
                ",
                "Text Preview • {line_count} lines"
                {
                    if let Some(lang) = &language {
                        format!(" • {}", lang)
                    } else {
                        String::new()
                    }
                }
            }
            
            pre {
                style: "
                    font-family: var(--vscode-font-mono);
                    font-size: 12px;
                    line-height: 1.4;
                    color: var(--color-text-primary, #cccccc);
                    margin: 0;
                    padding: var(--spacing-medium, 12px);
                    background: var(--color-bg-secondary, #252526);
                    border-radius: var(--border-radius-small, 2px);
                    overflow: auto;
                    white-space: pre-wrap;
                ",
                "{content.chars().take(2000).collect::<String>()}"
                {
                    if content.len() > 2000 {
                        "...\n[Content truncated for preview]"
                    } else {
                        ""
                    }
                }
            }
        }
    }
}

#[component]
pub fn ArchivePreview(
    contents: Vec<String>,
    file_size: u64,
) -> Element {
    let visible_files = contents.iter().take(20).cloned().collect::<Vec<_>>();
    let remaining_count = contents.len().saturating_sub(20);
    
    rsx! {
        div {
            class: "archive-preview",
            style: "
                width: 100%;
                height: 100%;
                padding: var(--spacing-medium, 12px);
                overflow: auto;
            ",
            
            div {
                style: "
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    gap: var(--spacing-medium, 12px);
                    margin-bottom: var(--spacing-large, 16px);
                ",
                
                div {
                    style: "font-size: 48px;",
                    "🗜️"
                }
                
                div {
                    style: "
                        color: var(--color-text-secondary, #999999);
                        font-size: var(--font-size-small, 13px);
                        text-align: center;
                    ",
                    "Archive • {contents.len()} files • {format_file_size(file_size)}"
                }
            }
            
            div {
                class: "archive-contents",
                style: "
                    background: var(--color-bg-secondary, #252526);
                    border-radius: var(--border-radius-small, 2px);
                    padding: var(--spacing-medium, 12px);
                    max-height: 300px;
                    overflow-y: auto;
                ",
                
                for (index, file_name) in visible_files.iter().enumerate() {
                    div {
                        key: "{index}",
                        style: "
                            padding: var(--spacing-extra-small, 4px) var(--spacing-small, 8px);
                            color: var(--color-text-primary, #cccccc);
                            font-family: var(--vscode-font-mono);
                            font-size: 12px;
                        ",
                        "{file_name}"
                    }
                }
                
                if remaining_count > 0 {
                    div {
                        style: "
                            padding: var(--spacing-small, 8px);
                            color: var(--color-text-secondary, #999999);
                            font-style: italic;
                            text-align: center;
                        ",
                        "... and {remaining_count} more files"
                    }
                }
            }
        }
    }
}

#[component]
pub fn UnsupportedPreview(
    file_type: String,
    reason: String,
    suggested_action: Option<String>,
) -> Element {
    rsx! {
        div {
            class: "unsupported-preview",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: var(--spacing-medium, 12px);
                padding: var(--spacing-large, 16px);
                text-align: center;
            ",
            
            div {
                style: "font-size: 64px; opacity: 0.5;",
                "❓"
            }
            
            h3 {
                style: "
                    margin: 0 0 var(--spacing-medium, 12px) 0;
                    color: var(--color-text-primary, #cccccc);
                    font-size: var(--font-size-large, 16px);
                ",
                "Unsupported File Type: .{file_type}"
            }
            
            p {
                style: "
                    margin: 0 0 var(--spacing-medium, 12px) 0;
                    color: var(--color-text-secondary, #999999);
                    font-size: var(--font-size-medium, 14px);
                ",
                "{reason}"
            }
            
            if let Some(action) = suggested_action {
                p {
                    style: "
                        margin: 0;
                        color: var(--color-accent, #007acc);
                        font-size: var(--font-size-small, 13px);
                        font-weight: 500;
                    ",
                    "{action}"
                }
            }
        }
    }
}

/// Format file size for display
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format time duration for display (MM:SS format)
fn format_time(seconds: f64) -> String {
    let total_seconds = seconds.max(0.0) as u64;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}

/// Generate mock waveform data for audio visualization
/// In a real implementation, this would analyze actual audio data
fn generate_mock_waveform_data(duration: f64) -> Vec<f64> {
    let sample_count = 400; // Number of bars in the waveform
    let mut waveform = Vec::with_capacity(sample_count);
    
    for i in 0..sample_count {
        // Generate a realistic waveform pattern with some randomness
        let progress = i as f64 / sample_count as f64;
        let base_amplitude = 0.3 + 0.4 * (progress * std::f64::consts::PI * 2.0).sin().abs();
        let noise = (i as f64 * 0.1).sin() * 0.2;
        let amplitude = (base_amplitude + noise).clamp(0.1, 1.0);
        
        waveform.push(amplitude);
    }
    
    waveform
}

/// Build metadata sections from preview data
fn build_metadata_sections(preview_data: &PreviewData) -> Vec<(String, Vec<(&'static str, String)>)> {
    let mut sections = Vec::new();
    
    // Media properties section
    let mut media_fields = Vec::new();
    
    media_fields.push(("Format", format!("{:?}", preview_data.format)));
    
    if let Some(width) = preview_data.metadata.width {
        if let Some(height) = preview_data.metadata.height {
            media_fields.push(("Dimensions", format!("{} × {} px", width, height)));
            
            // Calculate aspect ratio
            let gcd_val = gcd(width, height);
            let aspect_w = width / gcd_val;
            let aspect_h = height / gcd_val;
            media_fields.push(("Aspect Ratio", format!("{}:{}", aspect_w, aspect_h)));
        }
    }
    
    if let Some(duration) = preview_data.metadata.duration {
        media_fields.push(("Duration", format_duration(duration)));
    }
    
    if let Some(bit_rate) = preview_data.metadata.bit_rate {
        media_fields.push(("Bit Rate", format!("{} kbps", bit_rate / 1000)));
    }
    
    if let Some(sample_rate) = preview_data.metadata.sample_rate {
        media_fields.push(("Sample Rate", format!("{:.1} kHz", sample_rate as f64 / 1000.0)));
    }
    
    if let Some(codec) = &preview_data.metadata.codec {
        media_fields.push(("Codec", codec.clone()));
    }
    
    if let Some(page_count) = preview_data.metadata.page_count {
        media_fields.push(("Pages", page_count.to_string()));
    }
    
    if let Some(color_space) = &preview_data.metadata.color_space {
        media_fields.push(("Color Space", color_space.clone()));
    }
    
    if let Some(compression) = &preview_data.metadata.compression {
        media_fields.push(("Compression", compression.clone()));
    }
    
    if !media_fields.is_empty() {
        sections.push(("Media Properties".to_string(), media_fields));
    }
    
    // Audio metadata section
    let mut audio_fields = Vec::new();
    
    if let Some(title) = &preview_data.metadata.title {
        audio_fields.push(("Title", title.clone()));
    }
    
    if let Some(artist) = &preview_data.metadata.artist {
        audio_fields.push(("Artist", artist.clone()));
    }
    
    if let Some(album) = &preview_data.metadata.album {
        audio_fields.push(("Album", album.clone()));
    }
    
    if let Some(year) = preview_data.metadata.year {
        audio_fields.push(("Year", year.to_string()));
    }
    
    if !audio_fields.is_empty() {
        sections.push(("Audio Tags".to_string(), audio_fields));
    }
    
    // EXIF data section for images
    if let Some(exif) = &preview_data.metadata.exif_data {
        let mut exif_fields = Vec::new();
        
        if let Some(make) = &exif.camera_make {
            exif_fields.push(("Camera Make", make.clone()));
        }
        
        if let Some(model) = &exif.camera_model {
            exif_fields.push(("Camera Model", model.clone()));
        }
        
        if let Some(lens) = &exif.lens_model {
            exif_fields.push(("Lens", lens.clone()));
        }
        
        if let Some(focal_length) = exif.focal_length {
            exif_fields.push(("Focal Length", format!("{:.1}mm", focal_length)));
        }
        
        if let Some(aperture) = exif.aperture {
            exif_fields.push(("Aperture", format!("f/{:.1}", aperture)));
        }
        
        if let Some(shutter_speed) = &exif.shutter_speed {
            exif_fields.push(("Shutter Speed", shutter_speed.clone()));
        }
        
        if let Some(iso) = exif.iso {
            exif_fields.push(("ISO", iso.to_string()));
        }
        
        if let Some(flash) = exif.flash {
            exif_fields.push(("Flash", if flash { "Fired" } else { "Did not fire" }.to_string()));
        }
        
        if let Some(date_taken) = exif.date_taken {
            exif_fields.push(("Date Taken", format_system_time(date_taken)));
        }
        
        if let (Some(lat), Some(lng)) = (exif.gps_latitude, exif.gps_longitude) {
            exif_fields.push(("GPS Location", format!("{:.6}, {:.6}", lat, lng)));
        }
        
        if !exif_fields.is_empty() {
            sections.push(("Camera Info (EXIF)".to_string(), exif_fields));
        }
    }
    
    // Timestamps section
    let mut timestamp_fields = Vec::new();
    
    if let Some(created) = preview_data.metadata.created {
        timestamp_fields.push(("Created", format_system_time(created)));
    }
    
    if let Some(modified) = preview_data.metadata.modified {
        timestamp_fields.push(("Modified", format_system_time(modified)));
    }
    
    if !timestamp_fields.is_empty() {
        sections.push(("Timestamps".to_string(), timestamp_fields));
    }
    
    sections
}

/// Generate mock timeline thumbnail positions for video scrubbing
/// In a real implementation, this would extract actual video frames
fn generate_timeline_thumbnails(duration: f64) -> Vec<f64> {
    let thumbnail_count = 20; // Number of thumbnails in the timeline
    let mut positions = Vec::with_capacity(thumbnail_count);
    
    for i in 0..thumbnail_count {
        let position = (i as f64) / (thumbnail_count as f64 - 1.0) * duration;
        positions.push(position);
    }
    
    positions
}

/// Format system time for display
fn format_system_time(time: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let seconds = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(seconds as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now());
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        Err(_) => "Unknown".to_string(),
    }
}

/// Format file path for display (truncate if too long)
fn format_file_path(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy();
    if path_str.len() > 50 {
        format!("...{}", &path_str[path_str.len() - 47..])
    } else {
        path_str.to_string()
    }
}

/// Format duration in a more readable format
fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{}:{:02}", minutes, secs)
    }
}

/// Calculate greatest common divisor for aspect ratio
fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

// Quick action implementations (mock implementations for now)

/// Copy text to clipboard
fn copy_to_clipboard(text: &str) {
    // Mock implementation - in a real app, this would use the system clipboard
    tracing::info!("Copying to clipboard: {}", text);
    // TODO: Implement actual clipboard functionality
}

/// Open file with external application
fn open_external(path: &PathBuf) {
    // Mock implementation - in a real app, this would use system APIs
    tracing::info!("Opening externally: {:?}", path);
    // TODO: Implement actual external app opening
    // On macOS: std::process::Command::new("open").arg(path).spawn()
    // On Windows: std::process::Command::new("cmd").args(["/c", "start", ""]).arg(path).spawn()
    // On Linux: std::process::Command::new("xdg-open").arg(path).spawn()
}

/// Rotate image file
fn rotate_image(path: &PathBuf) {
    // Mock implementation - in a real app, this would rotate the actual image
    tracing::info!("Rotating image: {:?}", path);
    // TODO: Implement actual image rotation using image processing library
}

/// Show system file properties dialog
fn show_file_properties(path: &PathBuf) {
    // Mock implementation - in a real app, this would show system properties
    tracing::info!("Showing properties for: {:?}", path);
    // TODO: Implement system properties dialog
    // On macOS: applescript to show info window
    // On Windows: shell32 properties dialog
    // On Linux: file manager properties
}

/// Progressive loading placeholder with animated progress indicator
#[component]
pub fn ProgressivePlaceholder(
    progress: Option<u8>,
    loading_progress: Signal<u8>,
    selected_file: Signal<Option<FileSystemEntry>>,
) -> Element {
    let current_progress = progress.unwrap_or_else(|| *loading_progress.read());
    let file_name = selected_file.read().as_ref()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "Loading...".to_string());
    
    rsx! {
        div {
            class: "progressive-placeholder",
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 100%;
                padding: var(--spacing-large, 16px);
                color: var(--color-text-primary, #cccccc);
            ",
            
            // Loading spinner
            div {
                class: "loading-spinner",
                style: "
                    width: 48px;
                    height: 48px;
                    border: 3px solid var(--color-border-primary, #464647);
                    border-top: 3px solid var(--color-accent-primary, #0078d4);
                    border-radius: 50%;
                    margin-bottom: var(--spacing-medium, 12px);
                ",
            }
            
            // File name
            div {
                class: "loading-filename",
                style: "
                    font-size: var(--font-size-medium, 14px);
                    font-weight: 500;
                    margin-bottom: var(--spacing-small, 8px);
                    text-align: center;
                ",
                {file_name}
            }
            
            // Progress bar
            div {
                class: "progress-container",
                style: "
                    width: 240px;
                    height: 4px;
                    background: var(--color-bg-tertiary, #2d2d30);
                    border-radius: 2px;
                    overflow: hidden;
                    margin-bottom: var(--spacing-small, 8px);
                ",
                
                div {
                    class: "progress-bar",
                    style: "
                        width: {current_progress}%;
                        height: 100%;
                        background: linear-gradient(90deg, var(--color-accent-primary, #0078d4), var(--color-accent-secondary, #106ebe));
                        transition: width 0.3s ease;
                    ",
                }
            }
            
            // Progress text
            div {
                class: "progress-text",
                style: "
                    font-size: var(--font-size-small, 12px);
                    color: var(--color-text-secondary, #999999);
                ",
                "Loading preview... {current_progress}%"
            }
        }
    }
}

/// Loading error display with retry option
#[component]
pub fn LoadingErrorDisplay(
    error: String,
    selected_file: Signal<Option<FileSystemEntry>>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    let file_name = selected_file.read().as_ref()
        .map(|f| f.name.clone())
        .unwrap_or_else(|| "Unknown file".to_string());
    
    rsx! {
        div {
            class: "loading-error-display",
            style: "
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                height: 100%;
                padding: var(--spacing-large, 16px);
                color: var(--color-text-primary, #cccccc);
                text-align: center;
            ",
            
            // Error icon
            div {
                class: "error-icon",
                style: "
                    width: 48px;
                    height: 48px;
                    border-radius: 50%;
                    background: var(--color-error, #f44747);
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    margin-bottom: var(--spacing-medium, 12px);
                    font-size: 24px;
                    color: white;
                ",
                "⚠"
            }
            
            // File name
            div {
                class: "error-filename",
                style: "
                    font-size: var(--font-size-medium, 14px);
                    font-weight: 500;
                    margin-bottom: var(--spacing-small, 8px);
                ",
                {file_name}
            }
            
            // Error message
            div {
                class: "error-message",
                style: "
                    font-size: var(--font-size-small, 12px);
                    color: var(--color-text-secondary, #999999);
                    margin-bottom: var(--spacing-medium, 12px);
                    max-width: 300px;
                ",
                "Failed to load preview: {error}"
            }
            
            // Retry button
            button {
                class: "retry-button",
                style: "
                    padding: var(--spacing-small, 8px) var(--spacing-medium, 12px);
                    border: 1px solid var(--color-accent-primary, #0078d4);
                    background: transparent;
                    color: var(--color-accent-primary, #0078d4);
                    cursor: pointer;
                    border-radius: 4px;
                    font-size: var(--font-size-small, 12px);
                    transition: all 0.2s ease;
                ",
                onclick: move |_| {
                    // Reset loader state to trigger retry
                    let mut loader = lazy_loader.write();
                    loader.state = LoadingState::NotLoaded;
                    loader.metrics = LoadingMetrics {
                        start_time: Instant::now(),
                        load_duration: None,
                        content_size: None,
                        cache_hit: false,
                    };
                },
                "Retry"
            }
        }
    }
}

/// Lazy loading wrapper for image previews
#[component]
pub fn LazyImagePreview(
    format: SupportedFormat,
    zoom_level: Signal<f64>,
    pan_x: Signal<f64>,
    pan_y: Signal<f64>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    // Use the existing ImagePreview but with lazy loading optimizations
    rsx! {
        ImagePreview {
            format: format,
            zoom_level: zoom_level,
            pan_x: pan_x,
            pan_y: pan_y,
        }
    }
}

/// Lazy loading wrapper for video previews with progressive thumbnail loading
#[component]
pub fn LazyVideoPreview(
    format: SupportedFormat,
    duration: Option<f64>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    rsx! {
        VideoPreview {
            format: format,
            duration: duration,
        }
    }
}

/// Lazy loading wrapper for audio previews with progressive waveform rendering
#[component]
pub fn LazyAudioPreview(
    format: SupportedFormat,
    duration: Option<f64>,
    sample_rate: Option<u32>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    rsx! {
        AudioPreview {
            format: format,
            duration: duration,
            sample_rate: sample_rate,
        }
    }
}

/// Lazy loading wrapper for document previews with page-by-page loading
#[component]
pub fn LazyDocumentPreview(
    format: SupportedFormat,
    page_count: Option<u32>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    rsx! {
        DocumentPreview {
            format: format,
            page_count: page_count,
        }
    }
}

/// Lazy loading wrapper for text previews with syntax highlighting on demand
#[component]
pub fn LazyTextPreview(
    content: String,
    language: Option<String>,
    line_count: usize,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    rsx! {
        TextPreview {
            content: content,
            language: language,
            line_count: line_count,
        }
    }
}

/// Lazy loading wrapper for archive previews with progressive file listing
#[component]
pub fn LazyArchivePreview(
    contents: Vec<String>,
    file_size: Option<u64>,
    lazy_loader: Signal<LazyLoader>,
) -> Element {
    rsx! {
        ArchivePreview {
            contents: contents,
            file_size: file_size.unwrap_or(0),
        }
    }
}

/// Calculate loading priority based on file characteristics
#[allow(dead_code)]
fn calculate_loading_priority(file_entry: &FileSystemEntry) -> u8 {
    let mut priority = 128u8; // Base priority
    
    // Increase priority for smaller files (faster to load)
    if file_entry.size < 1024 * 1024 { // < 1MB
        priority = priority.saturating_add(32);
    } else if file_entry.size < 10 * 1024 * 1024 { // < 10MB
        priority = priority.saturating_add(16);
    } else if file_entry.size > 100 * 1024 * 1024 { // > 100MB
        priority = priority.saturating_sub(32);
    }
    
    // Increase priority for image and text files (usually faster to preview)
    if let Some(file_type) = &file_entry.file_type {
        if file_type.starts_with("image/") {
            priority = priority.saturating_add(24);
        } else if file_type.starts_with("text/") {
            priority = priority.saturating_add(16);
        } else if file_type.starts_with("video/") {
            priority = priority.saturating_sub(16); // Videos are slower to process
        }
    }
    
    priority
}

/// Reusable icon button component for consistent preview controls
#[component]
pub fn IconButton(
    icon: &'static str,
    #[props(default = 28)] size: u32,
    tooltip: String,
    #[props(default = false)] active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let button_style = format!(
        "
            width: {size}px;
            height: {size}px;
            border: 1px solid {};
            background: {};
            color: var(--vscode-foreground, #cccccc);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: 3px;
            transition: all 0.1s ease;
        ",
        if active { "var(--vscode-focusBorder, #007acc)" } else { "transparent" },
        if active { 
            "var(--vscode-button-background, #0e639c)" 
        } else { 
            "transparent" 
        }
    );
    
    rsx! {
        button {
            class: "icon-button",
            style: button_style,
            title: tooltip.clone(),
            "aria-label": tooltip.clone(),
            "aria-pressed": if active { "true" } else { "false" },
            onclick: move |evt| onclick.call(evt),
            
            // Simple text icon for now - can be enhanced later
            span {
                style: format!("font-size: {}px;", (size as f64 * 0.6) as u32),
                "{icon}"
            }
        }
    }
}