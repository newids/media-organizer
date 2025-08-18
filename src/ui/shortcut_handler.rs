use dioxus::prelude::*;
use std::path::PathBuf;
use crate::state::{AppState, use_app_state, ViewMode};
use crate::ui::shortcuts::{ShortcutAction, ShortcutRegistry};

/// Keyboard shortcut action handler that executes actions using app state
#[derive(Clone)]
pub struct ShortcutHandler {
    app_state: AppState,
    registry: ShortcutRegistry,
}

impl ShortcutHandler {
    pub fn new(app_state: AppState, registry: ShortcutRegistry) -> Self {
        Self {
            app_state,
            registry,
        }
    }

    /// Handle a keyboard event and execute the corresponding action if any
    pub async fn handle_keyboard_event(&mut self, key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> bool {
        // Try to trigger a shortcut
        if let Some(action) = self.registry.try_trigger(key, ctrl, shift, alt, meta) {
            self.execute_action(action).await;
            true // Event was handled
        } else {
            false // Event was not handled
        }
    }

    /// Execute a shortcut action
    pub async fn execute_action(&mut self, action: ShortcutAction) {
        tracing::info!("Executing shortcut action: {:?}", action);

        match action {
            ShortcutAction::Copy => self.handle_copy().await,
            ShortcutAction::Paste => self.handle_paste().await,
            ShortcutAction::Cut => self.handle_cut().await,
            ShortcutAction::Delete => self.handle_delete().await,
            ShortcutAction::SelectAll => self.handle_select_all(),
            ShortcutAction::ClearSelection => self.handle_clear_selection(),
            ShortcutAction::Rename => self.handle_rename().await,
            ShortcutAction::NavigateUp => self.handle_navigate_up().await,
            ShortcutAction::NavigateBack => self.handle_navigate_back().await,
            ShortcutAction::NavigateForward => self.handle_navigate_forward().await,
            ShortcutAction::NavigateHome => self.handle_navigate_home().await,
            ShortcutAction::Refresh => self.handle_refresh().await,
            ShortcutAction::OpenFile => self.handle_open_file().await,
            ShortcutAction::ShowProperties => self.handle_show_properties(),
            ShortcutAction::TogglePreview => self.handle_toggle_preview(),
            ShortcutAction::ToggleSearch => self.handle_toggle_search(),
            ShortcutAction::NewFolder => self.handle_new_folder().await,
            ShortcutAction::ShowSettings => self.handle_show_settings(),
            ShortcutAction::Custom(name) => self.handle_custom_action(&name).await,
        }
    }

    // File operation handlers
    async fn handle_copy(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if !selected_files.is_empty() {
            // TODO: Implement clipboard operations
            tracing::info!("Copy action: {} files selected", selected_files.len());
            self.set_operation_feedback("Copied to clipboard", false).await;
        } else {
            self.set_operation_feedback("No files selected", true).await;
        }
    }

    async fn handle_paste(&mut self) {
        // TODO: Implement clipboard operations
        tracing::info!("Paste action");
        self.set_operation_feedback("Paste operation", false).await;
    }

    async fn handle_cut(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if !selected_files.is_empty() {
            // TODO: Implement clipboard operations with cut mode
            tracing::info!("Cut action: {} files selected", selected_files.len());
            self.set_operation_feedback("Cut to clipboard", false).await;
        } else {
            self.set_operation_feedback("No files selected", true).await;
        }
    }

    async fn handle_delete(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if !selected_files.is_empty() {
            // TODO: Implement safe delete with confirmation
            tracing::info!("Delete action: {} files selected", selected_files.len());
            self.set_operation_feedback("Delete operation (confirmation needed)", false).await;
        } else {
            self.set_operation_feedback("No files selected", true).await;
        }
    }

    // Selection handlers
    fn handle_select_all(&mut self) {
        let all_paths: Vec<PathBuf> = self.app_state
            .get_file_entries()
            .iter()
            .map(|entry| entry.path.clone())
            .collect();
        
        if !all_paths.is_empty() {
            self.app_state.select_files(all_paths.clone(), crate::state::navigation::SelectionMode::Replace);
            tracing::info!("Select all: {} files selected", all_paths.len());
        }
    }

    fn handle_clear_selection(&mut self) {
        let count = self.app_state.get_selection_count();
        self.app_state.clear_selection();
        tracing::info!("Selection cleared: {} files deselected", count);
    }

    async fn handle_rename(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if selected_files.len() == 1 {
            // TODO: Implement rename dialog
            tracing::info!("Rename action: {:?}", selected_files[0]);
            self.set_operation_feedback("Rename operation", false).await;
        } else if selected_files.is_empty() {
            self.set_operation_feedback("No file selected for rename", true).await;
        } else {
            self.set_operation_feedback("Cannot rename multiple files", true).await;
        }
    }

    // Navigation handlers
    async fn handle_navigate_up(&mut self) {
        if self.app_state.can_navigate_up() {
            if let Err(e) = self.app_state.navigate_up().await {
                tracing::error!("Failed to navigate up: {}", e);
                self.set_operation_feedback("Navigation failed", true).await;
            } else {
                tracing::info!("Navigated to parent directory");
            }
        } else {
            self.set_operation_feedback("Already at root directory", true).await;
        }
    }

    async fn handle_navigate_back(&mut self) {
        if self.app_state.can_navigate_back() {
            if let Err(e) = self.app_state.navigate_back().await {
                tracing::error!("Failed to navigate back: {}", e);
                self.set_operation_feedback("Navigation failed", true).await;
            } else {
                tracing::info!("Navigated back");
            }
        } else {
            self.set_operation_feedback("No previous directory", true).await;
        }
    }

    async fn handle_navigate_forward(&mut self) {
        if self.app_state.can_navigate_forward() {
            if let Err(e) = self.app_state.navigate_forward().await {
                tracing::error!("Failed to navigate forward: {}", e);
                self.set_operation_feedback("Navigation failed", true).await;
            } else {
                tracing::info!("Navigated forward");
            }
        } else {
            self.set_operation_feedback("No forward directory", true).await;
        }
    }

    async fn handle_navigate_home(&mut self) {
        if let Some(home_dir) = dirs::home_dir() {
            if let Err(e) = self.app_state.navigate_to(home_dir).await {
                tracing::error!("Failed to navigate home: {}", e);
                self.set_operation_feedback("Failed to navigate home", true).await;
            } else {
                tracing::info!("Navigated to home directory");
            }
        } else {
            self.set_operation_feedback("Home directory not found", true).await;
        }
    }

    async fn handle_refresh(&mut self) {
        if let Err(e) = self.app_state.refresh_current_directory().await {
            tracing::error!("Failed to refresh directory: {}", e);
            self.set_operation_feedback("Refresh failed", true).await;
        } else {
            tracing::info!("Directory refreshed");
            self.set_operation_feedback("Directory refreshed", false).await;
        }
    }

    async fn handle_open_file(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if selected_files.len() == 1 {
            let path = &selected_files[0];
            
            // Check if it's a directory
            let file_entries = self.app_state.get_file_entries();
            if let Some(entry) = file_entries.iter().find(|e| e.path == *path) {
                if entry.is_directory {
                    // Navigate into directory
                    if let Err(e) = self.app_state.navigate_to(path.clone()).await {
                        tracing::error!("Failed to navigate to directory: {}", e);
                        self.set_operation_feedback("Failed to open directory", true).await;
                    } else {
                        tracing::info!("Opened directory: {:?}", path);
                    }
                } else {
                    // TODO: Open file with system default application
                    tracing::info!("Open file: {:?}", path);
                    self.set_operation_feedback("File open operation", false).await;
                }
            }
        } else if selected_files.is_empty() {
            self.set_operation_feedback("No file selected", true).await;
        } else {
            self.set_operation_feedback("Cannot open multiple files", true).await;
        }
    }

    // View operation handlers
    fn handle_show_properties(&mut self) {
        let selected_files = self.app_state.get_selected_files();
        if !selected_files.is_empty() {
            // TODO: Implement properties dialog
            tracing::info!("Show properties for {} files", selected_files.len());
        }
    }

    fn handle_toggle_preview(&mut self) {
        let current_mode = self.app_state.get_view_mode();
        let new_mode = match current_mode {
            ViewMode::Preview => ViewMode::Grid,
            _ => ViewMode::Preview,
        };
        
        self.app_state.view_mode.set(new_mode.clone());
        tracing::info!("Toggled view mode to: {:?}", new_mode);
    }

    fn handle_toggle_search(&mut self) {
        let mut search_state = self.app_state.search_state.write();
        search_state.is_active = !search_state.is_active;
        tracing::info!("Toggled search: {}", search_state.is_active);
    }

    fn handle_show_settings(&mut self) {
        // TODO: This will be triggered by the app UI when settings panel is integrated
        tracing::info!("Show settings action");
    }

    async fn handle_new_folder(&mut self) {
        // TODO: Implement new folder creation dialog
        tracing::info!("New folder action");
        self.set_operation_feedback("New folder creation", false).await;
    }

    async fn handle_custom_action(&mut self, action_name: &str) {
        tracing::info!("Custom action: {}", action_name);
        self.set_operation_feedback(&format!("Custom action: {}", action_name), false).await;
    }

    // Helper methods
    async fn set_operation_feedback(&mut self, message: &str, is_error: bool) {
        // Set the operation feedback
        {
            let mut op_state = self.app_state.operation_state.write();
            op_state.status_message = message.to_string();
            op_state.is_active = true;
        }
        
        // Log the operation
        if is_error {
            tracing::warn!("Operation feedback (error): {}", message);
        } else {
            tracing::info!("Operation feedback: {}", message);
        }
        
        // Clear the message after a timeout
        let mut op_state_clone = self.app_state.operation_state.clone();
        spawn(async move {
            let timeout = if is_error { 5000 } else { 2000 }; // Error messages stay longer
            tokio::time::sleep(std::time::Duration::from_millis(timeout)).await;
            op_state_clone.write().is_active = false;
        });
    }
}

/// Hook to create and use a shortcut handler with current app state
pub fn use_shortcut_handler() -> ShortcutHandler {
    let app_state = use_app_state();
    let registry = use_signal(ShortcutRegistry::new);
    
    let registry_clone = {
        let registry_ref = registry.read();
        registry_ref.clone()
    };
    
    ShortcutHandler::new(app_state, registry_clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: These tests would require a Dioxus component context to work properly
    // In a real application, integration tests with actual components would be preferred
    
    #[test]
    fn test_shortcut_action_descriptions() {
        assert_eq!(ShortcutAction::Copy.description(), "Copy selected items");
        assert_eq!(ShortcutAction::NavigateHome.description(), "Navigate to home directory");
        assert_eq!(ShortcutAction::TogglePreview.description(), "Toggle preview panel");
    }
}