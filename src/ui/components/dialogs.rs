// Dialog components for user confirmation and feedback
// Provides confirmation dialogs, progress dialogs, toast notifications, and operation summaries

use dioxus::prelude::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::services::operations::{
    ErrorSeverity, RecoverySuggestion, ProgressInfo, BatchProgress
};

/// Types of confirmation dialogs
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationAction {
    Delete { items: Vec<String>, total_size: u64 },
    Overwrite { target: String, source: String },
    MoveToTrash { items: Vec<String> },
    BatchOperation { operation_type: String, count: usize },
}

/// Confirmation dialog result
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationResult {
    Confirmed,
    Cancelled,
    Pending,
}

/// Toast notification types
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Toast notification data
#[derive(Debug, Clone)]
pub struct ToastNotification {
    pub id: String,
    pub toast_type: ToastType,
    pub title: String,
    pub message: String,
    pub duration: Duration,
    pub created_at: Instant,
    pub auto_dismiss: bool,
}

/// Progress dialog state
#[derive(Debug, Clone)]
pub struct ProgressDialogState {
    pub visible: bool,
    pub title: String,
    pub operation: String,
    pub progress: ProgressInfo,
    pub cancellable: bool,
    pub details: Vec<String>,
    pub error_count: usize,
}

/// Operation summary for completed operations
#[derive(Debug, Clone)]
pub struct OperationSummary {
    pub operation_type: String,
    pub total_items: usize,
    pub successful_items: usize,
    pub failed_items: usize,
    pub duration: Duration,
    pub errors: Vec<(String, String)>, // (item, error_message)
    pub warnings: Vec<(String, String)>, // (item, warning_message)
    pub recovery_suggestions: Vec<RecoverySuggestion>,
}

/// Props for confirmation dialog
#[derive(Props)]
pub struct ConfirmationDialogProps<'a> {
    pub visible: bool,
    pub action: ConfirmationAction,
    pub on_result: EventHandler<'a, ConfirmationResult>,
    pub danger_level: ErrorSeverity,
    #[props(default = false)]
    pub show_details: bool,
}

/// Confirmation dialog component for destructive operations
pub fn ConfirmationDialog<'a>(cx: Scope<'a, ConfirmationDialogProps<'a>>) -> Element {
    if !cx.props.visible {
        return render! { div {} };
    }

    let title = match &cx.props.action {
        ConfirmationAction::Delete { .. } => "Confirm Delete",
        ConfirmationAction::Overwrite { .. } => "Confirm Overwrite", 
        ConfirmationAction::MoveToTrash { .. } => "Move to Trash",
        ConfirmationAction::BatchOperation { operation_type, .. } => operation_type,
    };
    
    let message = match &cx.props.action {
        ConfirmationAction::Delete { items, total_size } => {
            let item_text = if items.len() == 1 {
                format!("\"{}\"", items[0])
            } else {
                format!("{} items", items.len())
            };
            let size_text = format_file_size(*total_size);
            format!("Are you sure you want to delete {}? ({} total)\n\nThis action cannot be undone.", item_text, size_text)
        }
        ConfirmationAction::Overwrite { target, source } => {
            format!("\"{}\" already exists.\n\nDo you want to replace it with \"{}\"?", target, source)
        }
        ConfirmationAction::MoveToTrash { items } => {
            let item_text = if items.len() == 1 {
                format!("\"{}\"", items[0])
            } else {
                format!("{} items", items.len())
            };
            format!("Move {} to trash?", item_text)
        }
        ConfirmationAction::BatchOperation { operation_type, count } => {
            format!("Perform {} operation on {} items?", operation_type.to_lowercase(), count)
        }
    };
    
    let button_text = match &cx.props.action {
        ConfirmationAction::Delete { .. } => "Delete",
        ConfirmationAction::Overwrite { .. } => "Replace",
        ConfirmationAction::MoveToTrash { .. } => "Move to Trash",
        ConfirmationAction::BatchOperation { .. } => "Continue",
    };
    
    let icon = match &cx.props.action {
        ConfirmationAction::Delete { .. } | ConfirmationAction::MoveToTrash { .. } => "🗑️",
        ConfirmationAction::Overwrite { .. } => "⚠️", 
        ConfirmationAction::BatchOperation { .. } => "📦",
    };

    let danger_class = match cx.props.danger_level {
        ErrorSeverity::Critical => "confirmation-dialog critical",
        ErrorSeverity::High => "confirmation-dialog high",
        ErrorSeverity::Medium => "confirmation-dialog medium",
        ErrorSeverity::Low => "confirmation-dialog low",
    };

    render! {
        div {
            class: "dialog-overlay",
            onclick: move |_| cx.props.on_result.call(ConfirmationResult::Cancelled),
            
            div {
                class: danger_class,
                onclick: |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    span { class: "dialog-icon", icon }
                    h3 { title }
                }
                
                div {
                    class: "dialog-content",
                    p { 
                        class: "dialog-message",
                        style: "white-space: pre-line;",
                        message 
                    }
                    
                    if cx.props.show_details {
                        render! {
                            div {
                                class: "dialog-details",
                                match &cx.props.action {
                                    ConfirmationAction::Delete { items, .. } => {
                                        let remaining_count = if items.len() > 10 { items.len() - 10 } else { 0 };
                                        render! {
                                            div {
                                                h4 { "Items to delete:" }
                                                ul {
                                                    class: "item-list",
                                                    for item in items.iter().take(10) {
                                                        li { item.clone() }
                                                    }
                                                    li { 
                                                        class: "more-items",
                                                        if items.len() > 10 { "... and more items" }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => render! { div {} }
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-actions",
                    button {
                        class: "button secondary",
                        onclick: move |_| cx.props.on_result.call(ConfirmationResult::Cancelled),
                        "Cancel"
                    }
                    button {
                        class: match cx.props.danger_level {
                            ErrorSeverity::Critical | ErrorSeverity::High => "button danger",
                            _ => "button primary"
                        },
                        onclick: move |_| cx.props.on_result.call(ConfirmationResult::Confirmed),
                        button_text
                    }
                }
            }
        }
    }
}

/// Props for progress dialog
#[derive(Props)]
pub struct ProgressDialogProps<'a> {
    pub state: ProgressDialogState,
    pub on_cancel: Option<EventHandler<'a, ()>>,
}

/// Progress dialog component for long-running operations
pub fn ProgressDialog<'a>(cx: Scope<'a, ProgressDialogProps<'a>>) -> Element {
    if !cx.props.state.visible {
        return render! { div {} };
    }

    let progress = &cx.props.state.progress;
    let percentage = if progress.total > 0 {
        (progress.current as f64 / progress.total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let speed_text = if progress.speed_bps > 0 {
        format!(" at {}/s", format_file_size(progress.speed_bps))
    } else {
        String::new()
    };

    let eta_text = if let Some(eta_secs) = progress.eta_seconds {
        format!(" • {} remaining", format_duration(Duration::from_secs(eta_secs)))
    } else {
        String::new()
    };

    render! {
        div {
            class: "dialog-overlay progress-overlay",
            
            div {
                class: "progress-dialog",
                onclick: |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    span { class: "dialog-icon", "⚙️" }
                    h3 { cx.props.state.title.clone() }
                }
                
                div {
                    class: "dialog-content",
                    p { 
                        class: "operation-description",
                        cx.props.state.operation.clone() 
                    }
                    
                    div {
                        class: "progress-container",
                        div {
                            class: "progress-bar-background",
                            div {
                                class: "progress-bar-fill",
                                style: "width: {percentage}%",
                            }
                        }
                        
                        div {
                            class: "progress-text",
                            {
                                format!("{:.1}% ({} of {}){}{}", 
                                    percentage, 
                                    progress.current, 
                                    progress.total,
                                    speed_text,
                                    eta_text
                                )
                            }
                        }
                    }
                    
                    if cx.props.state.error_count > 0 {
                        render! {
                            div {
                                class: "progress-errors",
                                { format!("⚠️ {} errors encountered", cx.props.state.error_count) }
                            }
                        }
                    }
                    
                    if !cx.props.state.details.is_empty() {
                        render! {
                            div {
                                class: "progress-details",
                                h4 { "Details:" }
                                ul {
                                    for detail in cx.props.state.details.iter().rev().take(5) {
                                        li { detail.clone() }
                                    }
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-actions",
                    if cx.props.state.cancellable {
                        render! {
                            button {
                                class: "button secondary",
                                onclick: move |_| {
                                    if let Some(handler) = &cx.props.on_cancel {
                                        handler.call(());
                                    }
                                },
                                "Cancel"
                            }
                        }
                    } else {
                        render! { div {} }
                    }
                }
            }
        }
    }
}

/// Props for toast container
#[derive(Props)]
pub struct ToastContainerProps<'a> {
    pub toasts: Vec<ToastNotification>,
    pub on_dismiss: EventHandler<'a, String>,
}

/// Toast notification container
pub fn ToastContainer<'a>(cx: Scope<'a, ToastContainerProps<'a>>) -> Element {
    render! {
        div {
            class: "toast-container",
            for toast in &cx.props.toasts {
                ToastNotificationComponent {
                    toast: toast.clone(),
                    on_dismiss: |id| cx.props.on_dismiss.call(id)
                }
            }
        }
    }
}

/// Props for individual toast notification
#[derive(Props)]
pub struct ToastNotificationComponentProps<'a> {
    pub toast: ToastNotification,
    pub on_dismiss: EventHandler<'a, String>,
}

/// Individual toast notification component
pub fn ToastNotificationComponent<'a>(cx: Scope<'a, ToastNotificationComponentProps<'a>>) -> Element {
    let toast = &cx.props.toast;
    let toast_class = match toast.toast_type {
        ToastType::Success => "toast success",
        ToastType::Error => "toast error",
        ToastType::Warning => "toast warning",
        ToastType::Info => "toast info",
    };

    let icon = match toast.toast_type {
        ToastType::Success => "✅",
        ToastType::Error => "❌",
        ToastType::Warning => "⚠️",
        ToastType::Info => "ℹ️",
    };

    render! {
        div {
            class: toast_class,
            
            div {
                class: "toast-content",
                span { class: "toast-icon", icon }
                div {
                    class: "toast-text",
                    div { class: "toast-title", toast.title.clone() }
                    div { class: "toast-message", toast.message.clone() }
                }
            }
            
            button {
                class: "toast-dismiss",
                onclick: move |_| cx.props.on_dismiss.call(toast.id.clone()),
                "×"
            }
        }
    }
}

/// Props for operation summary dialog
#[derive(Props)]
pub struct OperationSummaryDialogProps<'a> {
    pub visible: bool,
    pub summary: Option<OperationSummary>,
    pub on_close: EventHandler<'a, ()>,
}

/// Operation summary dialog for completed operations
pub fn OperationSummaryDialog<'a>(cx: Scope<'a, OperationSummaryDialogProps<'a>>) -> Element {
    if !cx.props.visible || cx.props.summary.is_none() {
        return render! { div {} };
    }

    let summary = cx.props.summary.as_ref().unwrap();
    let success_rate = if summary.total_items > 0 {
        (summary.successful_items as f64 / summary.total_items as f64 * 100.0)
    } else {
        0.0
    };

    let status_icon = if summary.failed_items == 0 {
        "✅"
    } else if summary.successful_items == 0 {
        "❌"
    } else {
        "⚠️"
    };

    render! {
        div {
            class: "dialog-overlay",
            onclick: move |_| cx.props.on_close.call(()),
            
            div {
                class: "operation-summary-dialog",
                onclick: |evt| evt.stop_propagation(),
                
                div {
                    class: "dialog-header",
                    span { class: "dialog-icon", status_icon }
                    h3 { format!("{} Complete", summary.operation_type) }
                }
                
                div {
                    class: "dialog-content",
                    div {
                        class: "summary-stats",
                        div { class: "stat", 
                            span { class: "stat-label", "Total Items:" }
                            span { class: "stat-value", format!("{}", summary.total_items) }
                        }
                        div { class: "stat", 
                            span { class: "stat-label", "Successful:" }
                            span { class: "stat-value success", format!("{}", summary.successful_items) }
                        }
                        if summary.failed_items > 0 {
                            render! {
                                div { class: "stat", 
                                    span { class: "stat-label", "Failed:" }
                                    span { class: "stat-value error", format!("{}", summary.failed_items) }
                                }
                            }
                        }
                        div { class: "stat", 
                            span { class: "stat-label", "Success Rate:" }
                            span { class: "stat-value", format!("{:.1}%", success_rate) }
                        }
                        div { class: "stat", 
                            span { class: "stat-label", "Duration:" }
                            span { class: "stat-value", format_duration(summary.duration) }
                        }
                    }
                    
                    if !summary.errors.is_empty() {
                        let remaining_errors = if summary.errors.len() > 10 { summary.errors.len() - 10 } else { 0 };
                        render! {
                            div {
                                class: "summary-section",
                                h4 { "Errors:" }
                                ul {
                                    class: "error-list",
                                    for (item, error) in summary.errors.iter().take(10) {
                                        li { 
                                            span { class: "error-item", item.clone() }
                                            span { class: "error-message", error.clone() }
                                        }
                                    }
                                    li { 
                                        class: "more-items",
                                        if summary.errors.len() > 10 { "... and more errors" }
                                    }
                                }
                            }
                        }
                    }
                    
                    if !summary.warnings.is_empty() {
                        render! {
                            div {
                                class: "summary-section",
                                h4 { "Warnings:" }
                                ul {
                                    class: "warning-list",
                                    for (item, warning) in summary.warnings.iter().take(5) {
                                        li { 
                                            span { class: "warning-item", item.clone() }
                                            span { class: "warning-message", warning.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    if !summary.recovery_suggestions.is_empty() {
                        render! {
                            div {
                                class: "summary-section",
                                h4 { "Suggestions:" }
                                ul {
                                    class: "suggestion-list",
                                    for suggestion in &summary.recovery_suggestions {
                                        li { 
                                            div { class: "suggestion-title", suggestion.description.clone() }
                                            div { class: "suggestion-text", suggestion.suggestion.clone() }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                div {
                    class: "dialog-actions",
                    button {
                        class: "button primary",
                        onclick: move |_| cx.props.on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}

// Helper functions for formatting

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
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

/// Format duration in human-readable format
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

/// Toast manager for handling multiple toast notifications
pub struct ToastManager {
    toasts: Vec<ToastNotification>,
    next_id: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            next_id: 1,
        }
    }
    
    pub fn show_toast(&mut self, toast_type: ToastType, title: String, message: String) -> String {
        let id = format!("toast_{}", self.next_id);
        self.next_id += 1;
        
        let duration = match toast_type {
            ToastType::Error => Duration::from_secs(8),
            ToastType::Warning => Duration::from_secs(6),
            ToastType::Success => Duration::from_secs(4),
            ToastType::Info => Duration::from_secs(5),
        };
        
        let toast = ToastNotification {
            id: id.clone(),
            toast_type,
            title,
            message,
            duration,
            created_at: Instant::now(),
            auto_dismiss: true,
        };
        
        self.toasts.push(toast);
        id
    }
    
    pub fn dismiss_toast(&mut self, id: &str) {
        self.toasts.retain(|toast| toast.id != id);
    }
    
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|toast| {
            !toast.auto_dismiss || now.duration_since(toast.created_at) < toast.duration
        });
    }
    
    pub fn get_toasts(&self) -> &[ToastNotification] {
        &self.toasts
    }
    
    pub fn clear_all(&mut self) {
        self.toasts.clear();
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}