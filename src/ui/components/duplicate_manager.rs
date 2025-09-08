use dioxus::prelude::*;
use std::path::PathBuf;

use crate::services::{
    DuplicateDetector, DuplicateDetectionResults, DuplicateGroup,
    ComparisonMethod, DuplicateDetectionConfig, DetectionProgress,
    PrimarySelectionStrategy, FileEntry
};
use crate::state::{use_app_state, use_selection_state};
use crate::ui::components::{
    ConfirmationDialog, ConfirmationResult,
    ProgressDialog
};
use crate::ui::components::dialogs::{ConfirmationAction, ProgressDialogState};
use crate::utils::normalize_path_display;

/// State for the duplicate manager
#[derive(Debug, Clone, PartialEq)]
pub enum DuplicateManagerState {
    /// Ready to start detection
    Ready,
    /// Currently detecting duplicates
    Detecting,
    /// Results available for review
    ReviewingResults,
    /// Processing user actions (delete, move, etc.)
    ProcessingActions,
    /// Error state
    Error(String),
}

/// Configuration for duplicate detection UI
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionSettings {
    pub comparison_method: ComparisonMethod,
    pub min_file_size: u64,
    pub max_file_size: u64,
    pub include_hidden: bool,
    pub include_extensions: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub primary_selection: PrimarySelectionStrategy,
}

impl Default for DetectionSettings {
    fn default() -> Self {
        Self {
            comparison_method: ComparisonMethod::Content,
            min_file_size: 1024, // 1KB minimum
            max_file_size: u64::MAX,
            include_hidden: false,
            include_extensions: Vec::new(),
            exclude_extensions: vec![
                "tmp".to_string(),
                "temp".to_string(),
                "log".to_string(),
                "cache".to_string(),
            ],
            primary_selection: PrimarySelectionStrategy::Oldest,
        }
    }
}

/// Action that can be performed on duplicate files
#[derive(Debug, Clone, PartialEq)]
pub enum DuplicateAction {
    /// Delete selected files
    Delete(Vec<PathBuf>),
    /// Move selected files to trash
    MoveToTrash(Vec<PathBuf>),
    /// Move selected files to a directory
    MoveTo(Vec<PathBuf>, PathBuf),
    /// Keep only primary files and delete others
    KeepPrimary(String), // group_id
    /// Keep selected files and delete others in group
    KeepSelected(String, Vec<PathBuf>), // group_id, files_to_keep
}

/// Props for the DuplicateManager component
#[derive(Props, Clone, PartialEq)]
pub struct DuplicateManagerProps {
    /// Whether the duplicate manager is visible
    pub visible: bool,
    /// Callback when the manager is closed
    pub on_close: EventHandler<()>,
    /// Files to analyze (if None, uses current directory)
    pub files: Option<Vec<FileEntry>>,
}

/// Main duplicate manager component
#[component]
pub fn DuplicateManager(mut props: DuplicateManagerProps) -> Element {
    // State management
    let mut manager_state = use_signal(|| DuplicateManagerState::Ready);
    let mut detection_settings = use_signal(DetectionSettings::default);
    let mut detection_results = use_signal(|| None::<DuplicateDetectionResults>);
    let mut detection_progress = use_signal(|| None::<DetectionProgress>);
    let mut expanded_groups = use_signal(|| std::collections::HashSet::<String>::new());
    let mut selected_files = use_signal(|| std::collections::HashMap::<String, Vec<PathBuf>>::new());
    let mut confirmation_dialog = use_signal(|| None::<DuplicateAction>);
    let mut progress_dialog = use_signal(|| false);
    
    // App state integration
    let app_state = use_app_state();
    let _selection_state = use_selection_state();

    if !props.visible {
        return rsx! { div {} };
    }

    let container_style = "
        position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; 
        background: rgba(0,0,0,0.5); z-index: 1000; display: flex; 
        align-items: center; justify-content: center;
    ";

    let dialog_style = "
        background: white; border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.15);
        width: 90vw; height: 90vh; max-width: 1200px; max-height: 800px;
        display: flex; flex-direction: column; overflow: hidden;
    ";

    rsx! {
        div { style: "{container_style}",
            div { style: "{dialog_style}",
                // Header
                DuplicateManagerHeader {
                    state: manager_state.read().clone(),
                    settings: detection_settings.read().clone(),
                    results: detection_results.read().clone(),
                    on_close: move |_| props.on_close.call(()),
                    on_start_detection: move |settings: DetectionSettings| {
                        detection_settings.set(settings.clone());
                        manager_state.set(DuplicateManagerState::Detecting);
                        progress_dialog.set(true);
                        spawn_detection_task(
                            props.files.clone(),
                            settings,
                            manager_state,
                            detection_results,
                            detection_progress
                        );
                    },
                    on_settings_change: move |settings: DetectionSettings| {
                        detection_settings.set(settings);
                    }
                }

                // Main content area
                match manager_state.read().clone() {
                    DuplicateManagerState::Ready => rsx! {
                        DuplicateDetectionSetup {
                            settings: detection_settings.read().clone(),
                            on_settings_change: move |settings: DetectionSettings| {
                                detection_settings.set(settings);
                            }
                        }
                    },
                    DuplicateManagerState::Detecting => rsx! {
                        DuplicateDetectionProgress {
                            progress: detection_progress.read().clone()
                        }
                    },
                    DuplicateManagerState::ReviewingResults => rsx! {
                        DuplicateResultsReview {
                            results: detection_results.read().clone().unwrap(),
                            expanded_groups: expanded_groups,
                            selected_files: selected_files,
                            on_action: move |action: DuplicateAction| {
                                confirmation_dialog.set(Some(action));
                            }
                        }
                    },
                    DuplicateManagerState::ProcessingActions => rsx! {
                        div {
                            style: "display: flex; align-items: center; justify-content: center; flex: 1;",
                            div { style: "text-align: center;",
                                div { style: "font-size: 18px; margin-bottom: 16px;", "Processing actions..." }
                                div { style: "font-size: 14px; color: #666;", "Please wait while the operations complete." }
                            }
                        }
                    },
                    DuplicateManagerState::Error(ref error) => rsx! {
                        div {
                            style: "display: flex; align-items: center; justify-content: center; flex: 1;",
                            div { style: "text-align: center; color: #d32f2f;",
                                div { style: "font-size: 18px; margin-bottom: 16px;", "❌ Error" }
                                div { style: "font-size: 14px;", "{error}" }
                                button {
                                    style: "margin-top: 16px; padding: 8px 16px; background: #1976d2; color: white; border: none; border-radius: 4px; cursor: pointer;",
                                    onclick: move |_| manager_state.set(DuplicateManagerState::Ready),
                                    "Try Again"
                                }
                            }
                        }
                    }
                }

                // Confirmation dialog
                if let Some(action) = confirmation_dialog.read().clone() {
                    ConfirmationDialog {
                        visible: true,
                        action: match &action {
                            DuplicateAction::Delete(files) => {
                                crate::ui::components::dialogs::ConfirmationAction::Delete {
                                    items: files.iter().map(|p| normalize_path_display(p.as_path())).collect(),
                                    total_size: 0, // TODO: Calculate actual size
                                }
                            }
                            DuplicateAction::MoveToTrash(files) => {
                                crate::ui::components::dialogs::ConfirmationAction::MoveToTrash {
                                    items: files.iter().map(|p| normalize_path_display(p.as_path())).collect(),
                                }
                            }
                            DuplicateAction::MoveTo(files, _) => {
                                crate::ui::components::dialogs::ConfirmationAction::BatchOperation {
                                    operation_type: "Move Files".to_string(),
                                    count: files.len(),
                                }
                            }
                            DuplicateAction::KeepPrimary(_) => {
                                crate::ui::components::dialogs::ConfirmationAction::BatchOperation {
                                    operation_type: "Keep Primary File".to_string(),
                                    count: 1,
                                }
                            }
                            DuplicateAction::KeepSelected(_, files) => {
                                crate::ui::components::dialogs::ConfirmationAction::BatchOperation {
                                    operation_type: "Keep Selected Files".to_string(),
                                    count: files.len(),
                                }
                            }
                        },
                        danger_level: match &action {
                            DuplicateAction::Delete(_) => crate::services::ErrorSeverity::Critical,
                            DuplicateAction::MoveToTrash(_) => crate::services::ErrorSeverity::High,
                            _ => crate::services::ErrorSeverity::Medium,
                        },
                        on_result: move |result: ConfirmationResult| {
                            if result == ConfirmationResult::Confirmed {
                                execute_duplicate_action(action.clone(), manager_state, detection_results);
                            }
                            confirmation_dialog.set(None);
                        }
                    }
                }

                // Progress dialog
                if progress_dialog.read().clone() {
                    ProgressDialog {
                        state: crate::ui::components::dialogs::ProgressDialogState {
                            visible: true,
                            title: "Detecting Duplicates".to_string(),
                            operation: match detection_progress.read().clone() {
                                Some(progress) => format!("{:?}: {:.1}%", progress.phase, progress.progress_percentage),
                                None => "Initializing...".to_string()
                            },
                            progress: crate::services::ProgressInfo {
                                current: detection_progress.read().as_ref().map(|p| p.progress_percentage as u64).unwrap_or(0),
                                total: 100,
                                bytes_processed: 0,
                                total_bytes: 0,
                                speed_bps: 0,
                                eta_seconds: None,
                                current_operation: detection_progress.read().as_ref().map(|p| format!("{:?}", p.phase)).unwrap_or_default(),
                                started_at: std::time::SystemTime::now(),
                                last_update: std::time::SystemTime::now(),
                            },
                            cancellable: true,
                            details: Vec::new(),
                            error_count: 0,
                        },
                        on_cancel: Some(EventHandler::new(move |_| {
                            manager_state.set(DuplicateManagerState::Ready);
                            progress_dialog.set(false);
                        }))
                    }
                }
            }
        }
    }
}

/// Header component for the duplicate manager
#[component]
fn DuplicateManagerHeader(
    state: DuplicateManagerState,
    settings: DetectionSettings,
    results: Option<DuplicateDetectionResults>,
    on_close: EventHandler<()>,
    on_start_detection: EventHandler<DetectionSettings>,
    on_settings_change: EventHandler<DetectionSettings>,
) -> Element {
    let header_style = "
        display: flex; align-items: center; justify-content: space-between;
        padding: 16px 20px; border-bottom: 1px solid #e0e0e0;
        background: #f5f5f5;
    ";

    let title_style = "font-size: 18px; font-weight: 600; margin: 0;";
    let stats_style = "font-size: 14px; color: #666; margin-left: 16px;";

    rsx! {
        div { style: "{header_style}",
            div { style: "display: flex; align-items: center;",
                h2 { style: "{title_style}", "🔍 Duplicate File Manager" }
                
                if let Some(ref results) = results {
                    div { style: "{stats_style}",
                        "Found {results.duplicate_group_count()} groups with {results.total_duplicates} duplicates"
                        span { style: "margin-left: 8px; color: #1976d2;", 
                            "Potential savings: {results.format_savings()}" 
                        }
                    }
                }
            }

            div { style: "display: flex; gap: 8px; align-items: center;",
                match state {
                    DuplicateManagerState::Ready => rsx! {
                        button {
                            style: "padding: 8px 16px; background: #1976d2; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| on_start_detection.call(settings.clone()),
                            "🔍 Start Detection"
                        }
                    },
                    DuplicateManagerState::ReviewingResults => rsx! {
                        button {
                            style: "padding: 8px 16px; background: #1976d2; color: white; border: none; border-radius: 4px; cursor: pointer;",
                            onclick: move |_| on_start_detection.call(settings.clone()),
                            "🔄 Detect Again"
                        }
                    },
                    _ => rsx! { div {} }
                }

                button {
                    style: "padding: 8px 16px; background: #666; color: white; border: none; border-radius: 4px; cursor: pointer;",
                    onclick: move |_| on_close.call(()),
                    "✕ Close"
                }
            }
        }
    }
}

/// Detection setup component
#[component]
fn DuplicateDetectionSetup(
    settings: DetectionSettings,
    on_settings_change: EventHandler<DetectionSettings>,
) -> Element {
    // Use a signal to manage settings state within the component
    let mut local_settings = use_signal(|| settings.clone());
    let content_style = "
        flex: 1; padding: 20px; overflow-y: auto;
        display: flex; flex-direction: column; gap: 20px;
    ";

    let section_style = "
        background: #f9f9f9; padding: 16px; border-radius: 8px;
        border: 1px solid #e0e0e0;
    ";

    rsx! {
        div { style: "{content_style}",
            div { style: "{section_style}",
                h3 { style: "margin: 0 0 12px 0; color: #333;", "Detection Method" }
                select {
                    style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                    value: format!("{:?}", local_settings.read().comparison_method),
                    onchange: move |e| {
                        let method = match e.value().as_str() {
                            "Content" => ComparisonMethod::Content,
                            "Size" => ComparisonMethod::Size,
                            "Name" => ComparisonMethod::Name,
                            "SizeAndName" => ComparisonMethod::SizeAndName,
                            "ContentAndSize" => ComparisonMethod::ContentAndSize,
                            _ => ComparisonMethod::Content,
                        };
                        let mut new_settings = local_settings.read().clone();
                        new_settings.comparison_method = method;
                        local_settings.set(new_settings.clone());
                        on_settings_change.call(new_settings);
                    },
                    option { value: "Content", "Content Hash (Most Accurate)" }
                    option { value: "Size", "File Size (Fast)" }
                    option { value: "Name", "File Name" }
                    option { value: "SizeAndName", "Size + Name" }
                    option { value: "ContentAndSize", "Content + Size (Recommended)" }
                }
                div { style: "margin-top: 8px; font-size: 12px; color: #666;",
                    match local_settings.read().comparison_method {
                        ComparisonMethod::Content => "Compares file content using SHA-256 hash. Most accurate but slower.",
                        ComparisonMethod::Size => "Fast comparison by file size only. May have false positives.",
                        ComparisonMethod::Name => "Compares files with identical names.",
                        ComparisonMethod::SizeAndName => "Combines size and name comparison.",
                        ComparisonMethod::ContentAndSize => "Most reliable: combines content hash with size verification.",
                    }
                }
            }

            div { style: "{section_style}",
                h3 { style: "margin: 0 0 12px 0; color: #333;", "File Filters" }
                
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px; margin-bottom: 16px;",
                    div {
                        label { 
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Minimum File Size (bytes)" 
                        }
                        input {
                            r#type: "number",
                            style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            value: "{local_settings.read().min_file_size}",
                            min: "0",
                            onchange: move |e| {
                                let size = e.value().parse().unwrap_or(1024);
                                let mut new_settings = local_settings.read().clone();
                                new_settings.min_file_size = size;
                                local_settings.set(new_settings.clone());
                                on_settings_change.call(new_settings);
                            }
                        }
                    }
                    
                    div {
                        label {
                            style: "display: block; margin-bottom: 4px; font-weight: 500;",
                            "Primary Selection Strategy"
                        }
                        select {
                            style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                            value: format!("{:?}", local_settings.read().primary_selection),
                            onchange: move |e| {
                                let strategy = match e.value().as_str() {
                                    "Oldest" => PrimarySelectionStrategy::Oldest,
                                    "Newest" => PrimarySelectionStrategy::Newest,
                                    "ShortestPath" => PrimarySelectionStrategy::ShortestPath,
                                    "LongestPath" => PrimarySelectionStrategy::LongestPath,
                                    "First" => PrimarySelectionStrategy::First,
                                    _ => PrimarySelectionStrategy::Oldest,
                                };
                                let mut new_settings = local_settings.read().clone();
                                new_settings.primary_selection = strategy;
                                local_settings.set(new_settings.clone());
                                on_settings_change.call(new_settings);
                            },
                            option { value: "Oldest", "Oldest File" }
                            option { value: "Newest", "Newest File" }
                            option { value: "ShortestPath", "Shortest Path" }
                            option { value: "LongestPath", "Longest Path" }
                            option { value: "First", "First Found" }
                        }
                    }
                }

                label {
                    style: "display: flex; align-items: center; gap: 8px; margin-bottom: 12px;",
                    input {
                        r#type: "checkbox",
                        checked: local_settings.read().include_hidden,
                        onchange: move |e| {
                            let mut new_settings = local_settings.read().clone();
                            new_settings.include_hidden = e.checked();
                            local_settings.set(new_settings.clone());
                            on_settings_change.call(new_settings);
                        }
                    }
                    "Include hidden files"
                }

                div {
                    label {
                        style: "display: block; margin-bottom: 4px; font-weight: 500;",
                        "Excluded Extensions (comma-separated)"
                    }
                    input {
                        r#type: "text",
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                        value: local_settings.read().exclude_extensions.join(", "),
                        placeholder: "tmp, log, cache",
                        onchange: move |e| {
                            let extensions: Vec<String> = e.value()
                                .split(',')
                                .map(|s| s.trim().to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                            let mut new_settings = local_settings.read().clone();
                            new_settings.exclude_extensions = extensions;
                            local_settings.set(new_settings.clone());
                            on_settings_change.call(new_settings);
                        }
                    }
                }
            }

            div { style: "text-align: center; padding: 20px;",
                div { style: "font-size: 16px; color: #666; margin-bottom: 8px;",
                    "🚀 Ready to find duplicate files!"
                }
                div { style: "font-size: 14px; color: #999;",
                    "Click 'Start Detection' in the header to begin scanning."
                }
            }
        }
    }
}

/// Progress display component
#[component]
fn DuplicateDetectionProgress(progress: Option<DetectionProgress>) -> Element {
    let content_style = "
        flex: 1; display: flex; align-items: center; justify-content: center;
        flex-direction: column; gap: 20px; padding: 40px;
    ";

    rsx! {
        div { style: "{content_style}",
            if let Some(progress) = progress {
                div { style: "text-align: center; max-width: 500px;",
                    div { style: "font-size: 24px; margin-bottom: 16px;", "🔍 Detecting Duplicates..." }
                    
                    div { style: "margin-bottom: 20px;",
                        div { style: "font-size: 18px; margin-bottom: 8px;", "{progress.phase:?}" }
                        div { 
                            style: "width: 100%; height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden;",
                            div {
                                style: "height: 100%; background: #1976d2; transition: width 0.3s ease; width: {progress.progress_percentage}%;",
                            }
                        }
                        div { style: "margin-top: 8px; font-size: 14px; color: #666;",
                            "{progress.progress_percentage:.1}% complete"
                        }
                    }

                    div { style: "font-size: 14px; color: #666; line-height: 1.5;",
                        div { "Files processed: {progress.files_processed} / {progress.total_files}" }
                        if progress.groups_found > 0 {
                            div { "Duplicate groups found: {progress.groups_found}" }
                        }
                        if let Some(ref current_file) = progress.current_file {
                            div { style: "margin-top: 8px; font-style: italic;",
                                "Processing: {current_file.display()}"
                            }
                        }
                    }
                }
            } else {
                div { style: "text-align: center;",
                    div { style: "font-size: 18px;", "Initializing..." }
                }
            }
        }
    }
}

/// Results review component
#[component]
fn DuplicateResultsReview(
    results: DuplicateDetectionResults,
    mut expanded_groups: Signal<std::collections::HashSet<String>>,
    mut selected_files: Signal<std::collections::HashMap<String, Vec<PathBuf>>>,
    on_action: EventHandler<DuplicateAction>,
) -> Element {
    let content_style = "
        flex: 1; display: flex; flex-direction: column; overflow: hidden;
    ";

    let toolbar_style = "
        padding: 16px 20px; border-bottom: 1px solid #e0e0e0;
        display: flex; justify-content: space-between; align-items: center;
        background: #f9f9f9;
    ";

    let groups_area_style = "
        flex: 1; display: flex; overflow: hidden;
    ";

    let groups_list_style = "
        flex: 1; overflow-y: auto; padding: 16px;
        border-right: 1px solid #e0e0e0;
    ";

    let preview_area_style = "
        width: 300px; padding: 16px; background: #f5f5f5;
        overflow-y: auto;
    ";

    let action_bar_style = "
        padding: 16px 20px; border-top: 1px solid #e0e0e0;
        display: flex; gap: 12px; align-items: center;
        background: #f9f9f9;
    ";

    // Get selected file for preview
    let selected_file_for_preview = selected_files.read()
        .values()
        .flat_map(|files| files.iter())
        .next()
        .cloned();

    // Get total selected files count
    let total_selected = selected_files.read()
        .values()
        .map(|files| files.len())
        .sum::<usize>();

    // Get potential savings for selected files
    let selected_savings = calculate_selected_savings(&results, &selected_files.read());

    rsx! {
        div { style: "{content_style}",
            // Toolbar with summary stats
            div { style: "{toolbar_style}",
                div { style: "display: flex; align-items: center; gap: 20px;",
                    div { style: "font-weight: 600; color: #333;",
                        "📊 {results.duplicate_group_count()} groups • {results.total_duplicates} duplicates"
                    }
                    div { style: "color: #1976d2;",
                        "💾 Potential savings: {results.format_savings()}"
                    }
                    if total_selected > 0 {
                        div { style: "color: #4caf50;",
                            "✅ {total_selected} files selected ({selected_savings})"
                        }
                    }
                }

                div { style: "display: flex; gap: 8px; align-items: center;",
                    button {
                        style: "padding: 6px 12px; background: #2196f3; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: {
                            let mut expanded_groups = expanded_groups.clone();
                            let results_groups = results.groups.clone();
                            move |_| {
                                // Expand all groups
                                let mut expanded = expanded_groups.write();
                                for group in results_groups.iter() {
                                    expanded.insert(group.id.clone());
                                }
                            }
                        },
                        "Expand All"
                    }
                    button {
                        style: "padding: 6px 12px; background: #666; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            expanded_groups.write().clear();
                        },
                        "Collapse All"
                    }
                    button {
                        style: "padding: 6px 12px; background: #ff9800; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: {
                            let mut selected_files = selected_files.clone();
                            let results_groups = results.groups.clone();
                            move |_| {
                                // Select all non-primary files
                                let mut new_selected = std::collections::HashMap::new();
                                for group in results_groups.iter() {
                                    let non_primary_files: Vec<PathBuf> = group.files.iter()
                                        .filter(|f| !f.is_primary)
                                        .map(|f| f.path().to_path_buf())
                                        .collect();
                                    if !non_primary_files.is_empty() {
                                        new_selected.insert(group.id.clone(), non_primary_files);
                                    }
                                }
                                selected_files.set(new_selected);
                            }
                        },
                        "Select All Duplicates"
                    }
                    button {
                        style: "padding: 6px 12px; background: #666; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 12px;",
                        onclick: move |_| {
                            selected_files.write().clear();
                        },
                        "Clear Selection"
                    }
                }
            }

            // Main content area with groups and preview
            div { style: "{groups_area_style}",
                // Groups list
                div { style: "{groups_list_style}",
                    {results.groups.iter().enumerate().map(|(index, group)| {
                        let group_id = group.id.clone();
                        let is_expanded = expanded_groups.read().contains(&group_id);
                        let group_selected_files = selected_files.read()
                            .get(&group_id)
                            .cloned()
                            .unwrap_or_default();

                        rsx! {
                            DuplicateGroupCard {
                                key: "group-{index}",
                                group: group.clone(),
                                is_expanded: is_expanded,
                                selected_files: group_selected_files,
                                on_toggle_expand: move |group_id: String| {
                                    let mut expanded = expanded_groups.write();
                                    if expanded.contains(&group_id) {
                                        expanded.remove(&group_id);
                                    } else {
                                        expanded.insert(group_id);
                                    }
                                },
                                on_file_select: {
                                    let mut selected_files = selected_files.clone();
                                    EventHandler::new(move |params: (String, PathBuf, bool)| {
                                        let (group_id, file_path, selected) = params;
                                        let mut selected_map = selected_files.write();
                                        let group_files = selected_map.entry(group_id.clone()).or_insert_with(Vec::new);
                                        
                                        if selected {
                                            if !group_files.contains(&file_path) {
                                                group_files.push(file_path);
                                            }
                                        } else {
                                            group_files.retain(|p| p != &file_path);
                                            if group_files.is_empty() {
                                                selected_map.remove(&group_id);
                                            }
                                        }
                                    })
                                }
                            }
                        }
                    })}
                }

                // Preview area
                div { style: "{preview_area_style}",
                    if let Some(ref preview_file) = selected_file_for_preview {
                        FilePreviewPane { file_path: preview_file.clone() }
                    } else {
                        div { style: "text-align: center; color: #666; padding: 40px 20px;",
                            div { style: "font-size: 48px; margin-bottom: 16px;", "👁️" }
                            div { style: "font-size: 16px; margin-bottom: 8px;", "File Preview" }
                            div { style: "font-size: 14px; line-height: 1.5;",
                                "Select a file from the duplicate groups to see its preview, metadata, and comparison details."
                            }
                        }
                    }
                }
            }

            // Action bar
            div { style: "{action_bar_style}",
                if total_selected > 0 {
                    button {
                        style: "padding: 8px 16px; background: #f44336; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let files_to_delete: Vec<PathBuf> = selected_files.read()
                                .values()
                                .flat_map(|files| files.iter().cloned())
                                .collect();
                            on_action.call(DuplicateAction::Delete(files_to_delete));
                        },
                        "🗑️ Delete Selected ({total_selected})"
                    }
                    button {
                        style: "padding: 8px 16px; background: #ff9800; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: move |_| {
                            let files_to_trash: Vec<PathBuf> = selected_files.read()
                                .values()
                                .flat_map(|files| files.iter().cloned())
                                .collect();
                            on_action.call(DuplicateAction::MoveToTrash(files_to_trash));
                        },
                        "🗂️ Move to Trash ({total_selected})"
                    }
                } else {
                    div { style: "color: #666; font-style: italic;",
                        "Select files to enable bulk actions"
                    }
                }

                div { style: "margin-left: auto; display: flex; gap: 8px;",
                    button {
                        style: "padding: 8px 16px; background: #4caf50; color: white; border: none; border-radius: 4px; cursor: pointer;",
                        onclick: {
                            let on_action = on_action.clone();
                            let results_groups = results.groups.clone();
                            move |_| {
                                // Auto-select and keep primary files for all groups
                                for group in results_groups.iter() {
                                    on_action.call(DuplicateAction::KeepPrimary(group.id.clone()));
                                }
                            }
                        },
                        "🎯 Keep All Primary Files"
                    }
                }
            }
        }
    }
}

/// Spawn the duplicate detection task
fn spawn_detection_task(
    files: Option<Vec<FileEntry>>,
    settings: DetectionSettings,
    mut manager_state: Signal<DuplicateManagerState>,
    mut detection_results: Signal<Option<DuplicateDetectionResults>>,
    mut detection_progress: Signal<Option<DetectionProgress>>,
) {
    spawn(async move {
        // Convert settings to detection config
        let config = DuplicateDetectionConfig {
            comparison_method: settings.comparison_method,
            min_file_size: settings.min_file_size,
            max_file_size: settings.max_file_size,
            include_hidden: settings.include_hidden,
            include_extensions: Vec::new(), // TODO: Implement
            exclude_extensions: settings.exclude_extensions,
            primary_selection: settings.primary_selection,
            max_files: None,
        };

        // Create detector
        let detector = DuplicateDetector::with_config(config);

        // Get files to analyze
        let files_to_analyze = match files {
            Some(files) => files,
            None => {
                // TODO: Get files from current directory
                vec![]
            }
        };

        if files_to_analyze.is_empty() {
            manager_state.set(DuplicateManagerState::Error(
                "No files to analyze. Please select files first.".to_string()
            ));
            return;
        }

        // Create progress callback that is Send + Sync
        let progress_callback: crate::services::duplicate_detection::DetectionProgressCallback = {
            std::sync::Arc::new(move |progress: crate::services::duplicate_detection::DetectionProgress| {
                // Since we can't capture the signal directly in a Send + Sync closure,
                // we'll spawn a task to update it
                tokio::spawn(async move {
                    // Note: This approach logs progress but doesn't update UI directly
                    // In a real implementation, you'd use a channel or other async communication
                    tracing::info!("Detection progress: phase={:?}, progress={:.1}%", 
                        progress.phase, 
                        progress.progress_percentage
                    );
                });
            })
        };

        // Run detection
        match detector.detect_duplicates(files_to_analyze, Some(progress_callback)).await {
            Ok(results) => {
                detection_results.set(Some(results));
                manager_state.set(DuplicateManagerState::ReviewingResults);
            }
            Err(error) => {
                manager_state.set(DuplicateManagerState::Error(error.to_string()));
            }
        }
    });
}

/// Get confirmation dialog title for an action
fn get_action_confirmation_title(action: &DuplicateAction) -> String {
    match action {
        DuplicateAction::Delete(_) => "Delete Files".to_string(),
        DuplicateAction::MoveToTrash(_) => "Move to Trash".to_string(),
        DuplicateAction::MoveTo(_, _) => "Move Files".to_string(),
        DuplicateAction::KeepPrimary(_) => "Keep Primary File".to_string(),
        DuplicateAction::KeepSelected(_, _) => "Keep Selected Files".to_string(),
    }
}

/// Get confirmation dialog message for an action
fn get_action_confirmation_message(action: &DuplicateAction) -> String {
    match action {
        DuplicateAction::Delete(files) => {
            format!("Are you sure you want to permanently delete {} files?", files.len())
        }
        DuplicateAction::MoveToTrash(files) => {
            format!("Move {} files to trash?", files.len())
        }
        DuplicateAction::MoveTo(files, dest) => {
            format!("Move {} files to {}?", files.len(), dest.display())
        }
        DuplicateAction::KeepPrimary(_) => {
            "Keep only the primary file and delete all other duplicates in this group?".to_string()
        }
        DuplicateAction::KeepSelected(_, files) => {
            format!("Keep {} selected files and delete others in the group?", files.len())
        }
    }
}

/// Execute a duplicate action
fn execute_duplicate_action(
    action: DuplicateAction,
    mut manager_state: Signal<DuplicateManagerState>,
    mut _detection_results: Signal<Option<DuplicateDetectionResults>>,
) {
    spawn(async move {
        manager_state.set(DuplicateManagerState::ProcessingActions);

        // TODO: Implement actual file operations
        // For now, just simulate the action
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        match action {
            DuplicateAction::Delete(files) => {
                tracing::info!("Would delete {} files", files.len());
                // TODO: Implement file deletion
            }
            DuplicateAction::MoveToTrash(files) => {
                tracing::info!("Would move {} files to trash", files.len());
                // TODO: Implement move to trash
            }
            DuplicateAction::MoveTo(files, dest) => {
                tracing::info!("Would move {} files to {}", files.len(), dest.display());
                // TODO: Implement file move
            }
            DuplicateAction::KeepPrimary(group_id) => {
                tracing::info!("Would keep primary file in group {}", group_id);
                // TODO: Implement keep primary
            }
            DuplicateAction::KeepSelected(group_id, files) => {
                tracing::info!("Would keep {} files in group {}", files.len(), group_id);
                // TODO: Implement keep selected
            }
        }

        manager_state.set(DuplicateManagerState::ReviewingResults);
    });
}

/// Calculate savings for selected files
fn calculate_selected_savings(
    results: &DuplicateDetectionResults,
    selected_files: &std::collections::HashMap<String, Vec<PathBuf>>,
) -> String {
    let total_size: u64 = selected_files.iter()
        .flat_map(|(group_id, files)| {
            results.groups.iter()
                .find(|g| &g.id == group_id)
                .map(|group| {
                    files.iter()
                        .filter_map(|selected_path| {
                            group.files.iter()
                                .find(|f| &f.path() == selected_path)
                                .map(|f| f.size())
                        })
                        .sum::<u64>()
                })
        })
        .sum();

    format_file_size(total_size)
}

/// Format file size for display
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;
    
    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}

/// Individual duplicate group card component
#[component]
fn DuplicateGroupCard(
    group: DuplicateGroup,
    is_expanded: bool,
    selected_files: Vec<PathBuf>,
    on_toggle_expand: EventHandler<String>,
    on_file_select: EventHandler<(String, PathBuf, bool)>,
) -> Element {
    // Clone required values to avoid borrowing issues
    let group_id = group.id.clone();
    let group_id_for_toggle = group.id.clone();
    let files = group.files.clone();
    let card_style = "
        margin-bottom: 12px; border: 1px solid #e0e0e0; border-radius: 8px;
        background: white; overflow: hidden;
    ";

    let header_style = "
        padding: 12px 16px; background: #f5f5f5; border-bottom: 1px solid #e0e0e0;
        display: flex; align-items: center; justify-content: space-between;
        cursor: pointer;
    ";

    let expand_icon = if is_expanded { "▼" } else { "▶" };
    let group_savings = calculate_group_savings(&group);
    let primary_file = group.files.iter().find(|f| f.is_primary);
    let duplicate_count = group.files.len() - 1; // Exclude primary

    rsx! {
        div { style: "{card_style}",
            // Group header
            div {
                style: "{header_style}",
                onclick: move |_| on_toggle_expand.call(group_id_for_toggle.clone()),
                
                div { style: "display: flex; align-items: center; gap: 12px;",
                    span { style: "font-size: 14px; color: #666;", "{expand_icon}" }
                    div {
                        div { style: "font-weight: 600; color: #333;",
                            {if let Some(primary) = primary_file {
                                primary.path().file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Unknown file")
                            } else {
                                "Unknown file"
                            }}
                        }
                        div { style: "font-size: 12px; color: #666; margin-top: 2px;",
                            "{duplicate_count} duplicates • Savings: {group_savings}"
                        }
                    }
                }

                div { style: "display: flex; align-items: center; gap: 8px;",
                    if let Some(primary) = primary_file {
                        div { style: "font-size: 12px; padding: 4px 8px; background: #e3f2fd; color: #1976d2; border-radius: 12px;",
                            "📍 Primary: {format_file_size(primary.size())}"
                        }
                    }
                    div { style: "font-size: 12px; color: #666;",
                        "Modified: {group.files.first().map(|f| format_timestamp(f.modified())).unwrap_or_default()}"
                    }
                }
            }

            // Expanded file list
            if is_expanded {
                div { style: "padding: 0;",
                    {files.clone().into_iter().enumerate().map(|(index, file)| {
                        let is_selected = selected_files.contains(&file.path().to_path_buf());
                        let file_style = if file.is_primary {
                            "padding: 12px 16px; border-bottom: 1px solid #f0f0f0; background: #f8f9fa;"
                        } else {
                            "padding: 12px 16px; border-bottom: 1px solid #f0f0f0;"
                        };
                        let group_id_for_file = group_id.clone();
                        let file_path_for_callback = file.path().to_path_buf();
                        let file_path_for_display = file_path_for_callback.clone();
                        let file_is_primary = file.is_primary;
                        let file_size = file.size();
                        let file_modified = file.modified();
                        let file_name = file.path().file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown").to_string();

                        rsx! {
                            div {
                                key: "file-{index}",
                                style: "{file_style}",
                                
                                div { style: "display: flex; align-items: center; gap: 12px;",
                                    if !file_is_primary {
                                        input {
                                            r#type: "checkbox",
                                            checked: is_selected,
                                            onchange: move |e| {
                                                on_file_select.call((group_id_for_file.clone(), file_path_for_callback.clone(), e.checked()));
                                            }
                                        }
                                    } else {
                                        div { style: "width: 16px;" } // Spacer for alignment
                                    }

                                    div { style: "flex: 1;",
                                        div { style: "display: flex; align-items: center; gap: 8px;",
                                            if file_is_primary {
                                                span { style: "font-size: 12px; padding: 2px 6px; background: #4caf50; color: white; border-radius: 8px;",
                                                    "PRIMARY"
                                                }
                                            }
                                            span { style: "font-weight: 500; color: #333;",
                                                "{file_name}"
                                            }
                                        }
                                        div { style: "font-size: 12px; color: #666; margin-top: 4px;",
                                            "{file_path_for_display.display()}"
                                        }
                                    }

                                    div { style: "text-align: right; font-size: 12px; color: #666;",
                                        div { "{format_file_size(file_size)}" }
                                        div { style: "margin-top: 2px;",
                                            "{format_timestamp(file_modified)}"
                                        }
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// File preview pane component  
#[component]
fn FilePreviewPane(file_path: PathBuf) -> Element {
    rsx! {
        div { style: "height: 100%;",
            div { style: "padding: 16px; border-bottom: 1px solid #e0e0e0;",
                h4 { style: "margin: 0 0 8px 0; color: #333;", "File Preview" }
                div { style: "font-size: 12px; color: #666; word-break: break-all;",
                    "{normalize_path_display(&file_path)}"
                }
            }

            div { style: "flex: 1; padding: 16px;",
                // Basic file info
                div { style: "margin-bottom: 20px;",
                    div { style: "font-weight: 500; margin-bottom: 8px;", "File Information" }
                    div { style: "font-size: 14px; line-height: 1.5;",
                        div { style: "margin-bottom: 4px;",
                            "Name: {file_path.file_name().and_then(|n| n.to_str()).unwrap_or(\"Unknown\")}"
                        }
                        div { style: "margin-bottom: 4px;",
                            "Extension: {file_path.extension().and_then(|e| e.to_str()).unwrap_or(\"None\")}"
                        }
                        // TODO: Add actual file size, modified date, etc.
                    }
                }

                // Placeholder for actual preview content
                div { style: "text-align: center; padding: 40px 20px; border: 2px dashed #e0e0e0; border-radius: 8px;",
                    div { style: "font-size: 32px; margin-bottom: 12px;", "📄" }
                    div { style: "color: #666; margin-bottom: 8px;", "Preview not available" }
                    div { style: "font-size: 12px; color: #999;",
                        "File preview integration coming soon..."
                    }
                }
            }
        }
    }
}

/// Calculate savings for a specific group
fn calculate_group_savings(group: &DuplicateGroup) -> String {
    let duplicate_size: u64 = group.files.iter()
        .filter(|f| !f.is_primary)
        .map(|f| f.size())
        .sum();
    format_file_size(duplicate_size)
}

/// Format timestamp for display
fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    
    let duration = timestamp.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    
    let secs = duration.as_secs();
    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.naive_utc())
        .unwrap_or_default();
    
    datetime.format("%Y-%m-%d %H:%M").to_string()
}