use dioxus::prelude::*;
use crate::services::{file_system::FileEntry, preview::PreviewData};
use crate::utils::{FileTypeDetectionUtil, FilePreviewSupport};
use crate::ui::components::{PreviewPanel, InfoPanel};
use crate::ui::components::preview_panel::FileSystemEntry;

/// Dynamic Content Panel that switches between PreviewPanel and InfoPanel
/// based on file type detection and preview support
#[component]
pub fn DynamicContentPanel(
    selected_file: Signal<Option<FileEntry>>,
    preview_data: Signal<Option<PreviewData>>,
) -> Element {
    // Create a computed signal that determines which panel to show
    let panel_type = use_memo(move || {
        if let Some(file_entry) = selected_file.read().as_ref() {
            let support = FileTypeDetectionUtil::detect_preview_support(
                &file_entry.file_type, 
                &file_entry.path
            );
            
            match support {
                FilePreviewSupport::PreviewSupported(_) => PanelType::Preview,
                FilePreviewSupport::InfoOnly(_) | FilePreviewSupport::Unsupported => PanelType::Info,
            }
        } else {
            PanelType::Empty
        }
    });

    // Create a signal that converts FileEntry to FileSystemEntry for PreviewPanel
    let mut filesystem_entry_signal = use_signal(|| {
        selected_file.read().as_ref().map(|entry| {
            FileSystemEntry {
                path: entry.path.clone(),
                name: entry.name.clone(),
                is_directory: entry.is_directory,
                size: entry.size,
                modified: entry.modified,
                file_type: match &entry.file_type {
                    crate::services::file_system::FileType::Image(_) => Some("image".to_string()),
                    crate::services::file_system::FileType::Video(_) => Some("video".to_string()),
                    crate::services::file_system::FileType::Audio(_) => Some("audio".to_string()),
                    crate::services::file_system::FileType::Document(_) => Some("document".to_string()),
                    _ => None,
                }
            }
        })
    });

    // Update the filesystem entry signal when selected_file changes
    use_effect(move || {
        let new_value = selected_file.read().as_ref().map(|entry| {
            FileSystemEntry {
                path: entry.path.clone(),
                name: entry.name.clone(),
                is_directory: entry.is_directory,
                size: entry.size,
                modified: entry.modified,
                file_type: match &entry.file_type {
                    crate::services::file_system::FileType::Image(_) => Some("image".to_string()),
                    crate::services::file_system::FileType::Video(_) => Some("video".to_string()),
                    crate::services::file_system::FileType::Audio(_) => Some("audio".to_string()),
                    crate::services::file_system::FileType::Document(_) => Some("document".to_string()),
                    _ => None,
                }
            }
        });
        filesystem_entry_signal.set(new_value);
    });
    
    rsx! {
        div {
            class: "dynamic-content-panel",
            role: "region",
            "aria-label": "Content panel",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                overflow: hidden;
                background-color: var(--vscode-background);
            ",
            
            match panel_type() {
                PanelType::Preview => rsx! {
                    PreviewPanel {
                        selected_file: filesystem_entry_signal,
                        preview_data: preview_data,
                    }
                },
                PanelType::Info => rsx! {
                    InfoPanel {
                        selected_file: selected_file,
                    }
                },
                PanelType::Empty => rsx! {
                    div {
                        class: "empty-content-panel",
                        style: "
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            height: 100%;
                            color: var(--vscode-text-secondary);
                            font-style: italic;
                            text-align: center;
                            padding: 32px;
                        ",
                        div {
                            h3 {
                                style: "
                                    margin: 0 0 8px 0;
                                    color: var(--vscode-text-primary);
                                    font-size: 18px;
                                    font-weight: 500;
                                ",
                                "No file selected"
                            }
                            p {
                                style: "
                                    margin: 0;
                                    color: var(--vscode-text-secondary);
                                    font-size: 14px;
                                ",
                                "Select a file from the explorer to view its preview or information"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Panel type enum to determine which panel to show
#[derive(PartialEq, Clone)]
enum PanelType {
    Preview,
    Info,
    Empty,
}

/// Panel type indicator component for debugging/status display (optional)
#[component]
pub fn PanelTypeIndicator(
    selected_file: Signal<Option<FileEntry>>,
) -> Element {
    rsx! {
        div {
            class: "panel-type-indicator",
            style: "
                position: absolute;
                top: 8px;
                right: 8px;
                padding: 4px 8px;
                background-color: var(--vscode-secondary-background);
                border: 1px solid var(--vscode-border);
                border-radius: 4px;
                font-size: 11px;
                color: var(--vscode-text-secondary);
                z-index: 1000;
                pointer-events: none;
                user-select: none;
            ",
            
{
                if let Some(file_entry) = selected_file.read().as_ref() {
                    let support = FileTypeDetectionUtil::detect_preview_support(
                        &file_entry.file_type, 
                        &file_entry.path
                    );
                    
                    match support {
                        FilePreviewSupport::PreviewSupported(category) => {
                            format!("Preview: {:?}", category)
                        },
                        FilePreviewSupport::InfoOnly(category) => {
                            format!("Info: {:?}", category)
                        },
                        FilePreviewSupport::Unsupported => {
                            "Unsupported".to_string()
                        }
                    }
                } else {
                    "No selection".to_string()
                }
            }
        }
    }
}