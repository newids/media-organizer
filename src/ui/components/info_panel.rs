use dioxus::prelude::*;
use crate::services::file_system::FileEntry;
use crate::utils::{FileTypeDetectionUtil, FilePreviewSupport, InfoCategory};
use std::path::PathBuf;

/// Info Panel component for displaying file metadata and properties
/// Used when files don't support direct preview or are better shown as info
#[component]
pub fn InfoPanel(
    selected_file: Signal<Option<FileEntry>>,
) -> Element {
    rsx! {
        div {
            class: "info-panel",
            role: "region",
            "aria-label": "File information panel",
            style: "
                width: 100%;
                height: 100%;
                display: flex;
                flex-direction: column;
                overflow: hidden;
                background-color: var(--vscode-background);
                color: var(--vscode-text-primary);
                padding: 16px;
            ",
            
            if let Some(file_entry) = selected_file.read().as_ref() {
                InfoPanelContent { file_entry: file_entry.clone() }
            } else {
                div {
                    class: "info-panel-empty",
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        height: 100%;
                        color: var(--vscode-text-secondary);
                        font-style: italic;
                    ",
                    "Select a file to view its information"
                }
            }
        }
    }
}

/// Content component for displaying file information
#[component]
fn InfoPanelContent(file_entry: FileEntry) -> Element {
    let file_path = &file_entry.path;
    let support_info = FileTypeDetectionUtil::detect_preview_support(&file_entry.file_type, file_path);
    let panel_description = FileTypeDetectionUtil::get_panel_description(&support_info);
    
    rsx! {
        div {
            class: "info-panel-content",
            style: "
                display: flex;
                flex-direction: column;
                gap: 20px;
                height: 100%;
                overflow-y: auto;
            ",
            
            // Header section with file icon and name
            div {
                class: "info-header",
                style: "
                    display: flex;
                    align-items: center;
                    gap: 12px;
                    padding-bottom: 16px;
                    border-bottom: 1px solid var(--vscode-border);
                ",
                
                div {
                    class: "file-icon",
                    style: "
                        width: 48px;
                        height: 48px;
                        background-color: var(--vscode-secondary-background);
                        border-radius: 8px;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 20px;
                        font-weight: 600;
                        color: var(--vscode-accent);
                    ",
                    {get_file_icon(&file_entry.file_type)}
                }
                
                div {
                    class: "file-header-info",
                    style: "flex: 1;",
                    
                    h2 {
                        style: "
                            margin: 0 0 4px 0;
                            font-size: 18px;
                            font-weight: 600;
                            color: var(--vscode-text-primary);
                            word-break: break-all;
                        ",
                        {file_entry.name.clone()}
                    }
                    
                    p {
                        style: "
                            margin: 0;
                            color: var(--vscode-text-secondary);
                            font-size: 14px;
                        ",
                        {panel_description}
                    }
                }
            }
            
            // Basic file properties
            div {
                class: "info-section",
                h3 {
                    style: "
                        margin: 0 0 12px 0;
                        font-size: 16px;
                        font-weight: 600;
                        color: var(--vscode-text-primary);
                    ",
                    "File Properties"
                }
                
                InfoPropertyGrid {
                    properties: vec![
                        ("Type".to_string(), get_file_type_description(&file_entry.file_type)),
                        ("Size".to_string(), format_file_size(file_entry.size)),
                        ("Location".to_string(), format_file_path(&file_entry.path)),
                        ("Modified".to_string(), format_timestamp(file_entry.modified)),
                        ("Created".to_string(), format_timestamp(file_entry.created)),
                        ("Permissions".to_string(), format_permissions(&file_entry.permissions)),
                    ]
                }
            }
            
            // Category-specific information
            {
                match &support_info {
                    FilePreviewSupport::InfoOnly(category) => {
                        match category {
                            InfoCategory::LargeDocument => rsx! {
                                DocumentInfoSection { file_entry: file_entry.clone() }
                            },
                            InfoCategory::Archive => rsx! {
                                ArchiveInfoSection { file_entry: file_entry.clone() }
                            },
                            InfoCategory::Binary => rsx! {
                                BinaryInfoSection { file_entry: file_entry.clone() }
                            },
                            InfoCategory::Unknown => rsx! {
                                UnknownFileInfoSection { file_entry: file_entry.clone() }
                            },
                        }
                    },
                    _ => rsx! { div {} }, // No additional info for preview-supported files
                }
            }
        }
    }
}

/// Property grid component for displaying key-value pairs
#[component]
fn InfoPropertyGrid(properties: Vec<(String, String)>) -> Element {
    rsx! {
        div {
            class: "property-grid",
            style: "
                display: grid;
                grid-template-columns: 1fr 2fr;
                gap: 8px 16px;
                font-size: 14px;
            ",
            
            for (key, value) in properties {
                dt {
                    style: "
                        color: var(--vscode-text-secondary);
                        font-weight: 500;
                        margin: 0;
                        padding: 4px 0;
                    ",
                    {key}
                }
                dd {
                    style: "
                        color: var(--vscode-text-primary);
                        margin: 0;
                        padding: 4px 0;
                        word-break: break-all;
                    ",
                    {value}
                }
            }
        }
    }
}

// Category-specific info sections

#[component]
fn DocumentInfoSection(file_entry: FileEntry) -> Element {
    rsx! {
        div {
            class: "document-info-section",
            h3 {
                style: "
                    margin: 0 0 12px 0;
                    font-size: 16px;
                    font-weight: 600;
                    color: var(--vscode-text-primary);
                ",
                "Document Information"
            }
            
            p {
                style: "
                    color: var(--vscode-text-secondary);
                    font-size: 14px;
                    line-height: 1.5;
                ",
                "This document is too large or complex for direct preview. Basic metadata and properties are shown above."
            }
            
            // TODO: Add document-specific metadata like page count, author, etc.
        }
    }
}

#[component]
fn ArchiveInfoSection(file_entry: FileEntry) -> Element {
    rsx! {
        div {
            class: "archive-info-section",
            h3 {
                style: "
                    margin: 0 0 12px 0;
                    font-size: 16px;
                    font-weight: 600;
                    color: var(--vscode-text-primary);
                ",
                "Archive Contents"
            }
            
            p {
                style: "
                    color: var(--vscode-text-secondary);
                    font-size: 14px;
                    line-height: 1.5;
                ",
                "Archive file detected. Contents listing would be shown here."
            }
            
            // TODO: Implement archive contents listing
        }
    }
}

#[component]
fn BinaryInfoSection(file_entry: FileEntry) -> Element {
    rsx! {
        div {
            class: "binary-info-section",
            h3 {
                style: "
                    margin: 0 0 12px 0;
                    font-size: 16px;
                    font-weight: 600;
                    color: var(--vscode-text-primary);
                ",
                "Binary File Details"
            }
            
            p {
                style: "
                    color: var(--vscode-text-secondary);
                    font-size: 14px;
                    line-height: 1.5;
                ",
                "This is a binary file. Technical details and file signature information would be shown here."
            }
            
            // TODO: Add binary file analysis (file signature, executable info, etc.)
        }
    }
}

#[component]
fn UnknownFileInfoSection(file_entry: FileEntry) -> Element {
    rsx! {
        div {
            class: "unknown-info-section",
            p {
                style: "
                    color: var(--vscode-text-secondary);
                    font-size: 14px;
                    line-height: 1.5;
                ",
                "File type not recognized or no additional information available."
            }
        }
    }
}

// Helper functions

fn get_file_icon(file_type: &crate::services::file_system::FileType) -> &'static str {
    match file_type {
        crate::services::file_system::FileType::Directory => "📁",
        crate::services::file_system::FileType::Image(_) => "🖼️",
        crate::services::file_system::FileType::Video(_) => "🎬",
        crate::services::file_system::FileType::Audio(_) => "🎵",
        crate::services::file_system::FileType::Document(_) => "📄",
        crate::services::file_system::FileType::Text(_) => "📝",
        crate::services::file_system::FileType::Other(_) => "📎",
    }
}

fn get_file_type_description(file_type: &crate::services::file_system::FileType) -> String {
    match file_type {
        crate::services::file_system::FileType::Directory => "Directory".to_string(),
        crate::services::file_system::FileType::Image(format) => format!("{:?} Image", format),
        crate::services::file_system::FileType::Video(format) => format!("{:?} Video", format),
        crate::services::file_system::FileType::Audio(format) => format!("{:?} Audio", format),
        crate::services::file_system::FileType::Document(format) => format!("{:?} Document", format),
        crate::services::file_system::FileType::Text(format) => format!("{:?} Text File", format),
        crate::services::file_system::FileType::Other(ext) => format!("{} File", ext.to_uppercase()),
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

fn format_file_path(path: &PathBuf) -> String {
    if let Some(parent) = path.parent() {
        parent.to_string_lossy().to_string()
    } else {
        "/".to_string()
    }
}

fn format_timestamp(timestamp: std::time::SystemTime) -> String {
    use std::time::{Duration, UNIX_EPOCH};
    
    match timestamp.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let naive_datetime = chrono::NaiveDateTime::from_timestamp_opt(secs as i64, 0)
                .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
            naive_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        Err(_) => "Unknown".to_string()
    }
}

fn format_permissions(permissions: &crate::services::file_system::FilePermissions) -> String {
    let mut result = String::new();
    
    if permissions.readable {
        result.push_str("Read ");
    }
    if permissions.writable {
        result.push_str("Write ");
    }
    if permissions.executable {
        result.push_str("Execute ");
    }
    
    if result.is_empty() {
        "None".to_string()
    } else {
        result.trim().to_string()
    }
}