use tracing::info;
use std::path::PathBuf;
use dioxus::prelude::{component, Element};

mod models;
mod performance;
mod services;
mod state;
mod theme;
mod ui;
mod utils;

use models::AppConfig;
use state::AppStateProvider;
use ui::phase2_app;
use services::FileEntry;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting File Manager application");

    // Load configuration
    let _config = AppConfig::default();

    // Create custom menu bar
    let menu = create_menu_bar();

    // Launch Dioxus desktop application with custom menu
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::default()
                .with_menu(menu)
                .with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("Media Organizer")
                )
        )
        .launch(app);
}

fn create_menu_bar() -> dioxus::desktop::muda::Menu {
    use dioxus::desktop::muda::{Menu, Submenu, MenuItem, PredefinedMenuItem};
    use dioxus::desktop::muda::accelerator::{Accelerator, Modifiers, Code};

    let menu = Menu::new();

    // App menu (media-organizer)
    let app_menu = Submenu::new("Media Organizer", true);
    app_menu.append_items(&[
        &MenuItem::with_id("about_app", "About Media Organizer", true, None),
        &MenuItem::with_id("check_updates", "Check for Updates...", true, None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ]).unwrap();
    menu.append(&app_menu).unwrap();

    // File menu
    let file_menu = Submenu::new("File", true);
    file_menu.append_items(&[
        &MenuItem::with_id("open_folder", "Open Folder...", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyO))),
        &MenuItem::with_id("new_window", "New Window", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("new_folder", "New Folder", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyN))),
        &MenuItem::with_id("new_file", "New File", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("refresh", "Refresh", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyR))),
        &MenuItem::with_id("show_hidden", "Show Hidden Files", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Period))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("open_with", "Open With...", true, None),
        &MenuItem::with_id("show_in_finder", "Show in Finder", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyR))),
    ]).unwrap();
    menu.append(&file_menu).unwrap();

    // Edit menu
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::select_all(None),
        &MenuItem::with_id("clear_selection", "Clear Selection", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("copy_to", "Copy to...", true, None),
        &MenuItem::with_id("move_to", "Move to...", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("delete", "Delete", true, Some(Accelerator::new(None, Code::Delete))),
        &MenuItem::with_id("rename", "Rename", true, Some(Accelerator::new(None, Code::Enter))),
        &MenuItem::with_id("duplicate", "Duplicate", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyD))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("settings", "Settings...", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::Comma))),
    ]).unwrap();
    menu.append(&edit_menu).unwrap();

    // View menu
    let view_menu = Submenu::new("View", true);
    view_menu.append_items(&[
        &MenuItem::with_id("toggle_sidebar", "Toggle Sidebar", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyB))),
        &MenuItem::with_id("toggle_panel", "Toggle Panel", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyJ))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("theme_light", "Light Theme", true, None),
        &MenuItem::with_id("theme_dark", "Dark Theme", true, None),
        &MenuItem::with_id("theme_auto", "Auto Theme", true, None),
    ]).unwrap();
    menu.append(&view_menu).unwrap();

    // Help menu
    let help_menu = Submenu::new("Help", true);
    help_menu.append_items(&[
        &MenuItem::with_id("keyboard_shortcuts", "Keyboard Shortcuts", true, None),
        &MenuItem::with_id("help_documentation", "Media Organizer Help", true, None),
    ]).unwrap();
    menu.append(&help_menu).unwrap();

    menu
}

/// Show a native folder picker dialog
async fn open_folder_dialog() -> Option<PathBuf> {
    use rfd::AsyncFileDialog;
    
    // Show folder picker dialog
    let folder = AsyncFileDialog::new()
        .set_title("Select Folder")
        .pick_folder()
        .await;
    
    folder.map(|handle| handle.path().to_path_buf())
}

/// Create a new folder with user input dialog
async fn create_new_folder_dialog(parent_path: &PathBuf) -> Result<PathBuf, String> {
    use rfd::AsyncMessageDialog;
    use std::fs;
    
    // Show input dialog for folder name
    let folder_name = show_input_dialog("New Folder", "Enter folder name:", "New Folder").await?;
    
    if folder_name.trim().is_empty() {
        return Err("Folder name cannot be empty".to_string());
    }
    
    // Validate folder name (no invalid characters)
    if folder_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("Folder name contains invalid characters".to_string());
    }
    
    let new_folder_path = parent_path.join(&folder_name);
    
    // Check if folder already exists
    if new_folder_path.exists() {
        return Err(format!("Folder '{}' already exists", folder_name));
    }
    
    // Create the folder
    match fs::create_dir(&new_folder_path) {
        Ok(_) => Ok(new_folder_path),
        Err(e) => Err(format!("Failed to create folder: {}", e))
    }
}

/// Show a simple input dialog (using message dialog as fallback)
async fn show_input_dialog(title: &str, message: &str, default_value: &str) -> Result<String, String> {
    // For now, we'll use a simple approach - in a full implementation, you'd want a proper input dialog
    // This is a simplified version that creates a folder with a default name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // For now, return a default name with timestamp to ensure uniqueness
    Ok(format!("New Folder {}", timestamp % 1000))
}

/// Show a confirmation dialog
async fn show_confirmation_dialog(title: &str, message: &str) -> Result<bool, String> {
    use rfd::AsyncMessageDialog;
    
    let result = AsyncMessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(rfd::MessageLevel::Warning)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        .await;
    
    Ok(result == rfd::MessageDialogResult::Yes)
}

/// Delete selected files
async fn delete_selected_files(selected_files: &[FileEntry]) -> Result<usize, String> {
    use std::fs;
    use trash;
    
    let mut deleted_count = 0;
    let mut errors = Vec::new();
    
    for file_entry in selected_files {
        let file_path = &file_entry.path;
        
        // Try to move to trash first (safer), fallback to permanent delete
        match trash::delete(file_path) {
            Ok(_) => {
                deleted_count += 1;
                info!("Moved to trash: {:?}", file_path);
            },
            Err(e) => {
                // If trash fails, try permanent deletion as fallback
                match if file_path.is_dir() {
                    fs::remove_dir_all(file_path)
                } else {
                    fs::remove_file(file_path)
                } {
                    Ok(_) => {
                        deleted_count += 1;
                        info!("Permanently deleted: {:?}", file_path);
                    },
                    Err(delete_err) => {
                        let error_msg = format!("Failed to delete '{}': {} (trash error: {})", 
                                               file_entry.name, delete_err, e);
                        errors.push(error_msg);
                    }
                }
            }
        }
    }
    
    if !errors.is_empty() {
        return Err(format!("Some files could not be deleted: {}", errors.join("; ")));
    }
    
    Ok(deleted_count)
}

/// Show a file or folder in the system file manager
async fn show_in_system_file_manager(path: &std::path::PathBuf) -> Result<(), String> {
    use std::process::Command;
    
    let result = if cfg!(target_os = "macos") {
        // macOS: Use 'open' command with -R to reveal in Finder
        Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "windows") {
        // Windows: Use 'explorer' with /select parameter
        Command::new("explorer")
            .arg("/select,")
            .arg(path)
            .spawn()
    } else {
        // Linux: Try different file managers
        // First try to open the parent directory if the path is a file
        let dir_to_open = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };
        
        // Try different Linux file managers in order of preference
        Command::new("xdg-open")
            .arg(dir_to_open)
            .spawn()
            .or_else(|_| Command::new("nautilus").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("dolphin").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("thunar").arg(dir_to_open).spawn())
            .or_else(|_| Command::new("pcmanfm").arg(dir_to_open).spawn())
    };
    
    match result {
        Ok(mut child) => {
            // Wait for the command to complete
            tokio::task::spawn_blocking(move || child.wait()).await
                .map_err(|e| format!("Failed to wait for file manager process: {}", e))?
                .map_err(|e| format!("File manager process failed: {}", e))?;
            Ok(())
        },
        Err(e) => Err(format!("Failed to open file manager: {}", e))
    }
}

/// Open a file with the system default application
async fn open_with_system_default(path: &std::path::PathBuf) -> Result<(), String> {
    use std::process::Command;
    
    let result = if cfg!(target_os = "macos") {
        // macOS: Use 'open' command
        Command::new("open")
            .arg(path)
            .spawn()
    } else if cfg!(target_os = "windows") {
        // Windows: Use 'cmd' with 'start' command
        Command::new("cmd")
            .args(&["/C", "start", "", &path.to_string_lossy()])
            .spawn()
    } else {
        // Linux: Use xdg-open
        Command::new("xdg-open")
            .arg(path)
            .spawn()
    };
    
    match result {
        Ok(mut child) => {
            // Don't wait for the command to complete since we want to launch and return
            tokio::task::spawn(async move {
                let _ = child.wait();
            });
            Ok(())
        },
        Err(e) => Err(format!("Failed to open file with system default application: {}", e))
    }
}

/// Show a rename dialog with current filename
async fn show_rename_dialog(current_name: &str) -> Result<String, String> {
    // For now, create a simple timestamped version for demonstration
    // In a full implementation, you'd want a proper text input dialog
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    // Extract file extension if it exists
    if let Some(dot_pos) = current_name.rfind('.') {
        let (name_part, ext_part) = current_name.split_at(dot_pos);
        Ok(format!("{}_renamed_{}{}", name_part, timestamp % 1000, ext_part))
    } else {
        Ok(format!("{}_renamed_{}", current_name, timestamp % 1000))
    }
}

/// Rename a file or folder
async fn rename_file(current_path: &std::path::PathBuf, new_name: &str) -> Result<std::path::PathBuf, String> {
    use std::fs;
    
    // Validate new name (no invalid characters and not empty)
    if new_name.trim().is_empty() {
        return Err("New name cannot be empty".to_string());
    }
    
    if new_name.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err("New name contains invalid characters".to_string());
    }
    
    // Get parent directory and create new path
    let parent_dir = current_path.parent()
        .ok_or_else(|| "Cannot determine parent directory".to_string())?;
    
    let new_path = parent_dir.join(new_name);
    
    // Check if target already exists
    if new_path.exists() && new_path != *current_path {
        return Err(format!("A file or folder named '{}' already exists", new_name));
    }
    
    // Perform the rename
    match fs::rename(current_path, &new_path) {
        Ok(_) => Ok(new_path),
        Err(e) => Err(format!("Failed to rename file: {}", e))
    }
}

// Root app component with state provider
fn app() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    use dioxus::desktop::use_muda_event_handler;
    
    rsx! {
        AppStateProvider {
            AppWithMenuHandlers {}
        }
    }
}

// App component with menu handlers that has access to the app state context
fn AppWithMenuHandlers() -> Element {
    use dioxus::prelude::*;
    use dioxus::desktop::use_muda_event_handler;
    use crate::state::use_app_state;
    
    // Get app state for menu actions
    let app_state = use_app_state();
    
    // Handle menu events
    use_muda_event_handler(move |menu_event| {
        let event_id = menu_event.id.0.as_str();
        info!("Menu item clicked: {}", event_id);
        
        match event_id {
            // App menu items
            "about_app" => {
                info!("Showing about dialog...");
                // TODO: Implement about dialog
            },
            "check_updates" => {
                info!("Checking for updates...");
                // TODO: Implement update check
            },
            
            // File menu items
            "open_folder" => {
                info!("Opening folder selection dialog...");
                let app_state_clone = app_state.clone();
                
                // Spawn async task to handle folder dialog
                spawn(async move {
                    if let Some(folder_path) = open_folder_dialog().await {
                        info!("Selected folder: {:?}", folder_path);
                        
                        // Update the file tree root to the selected folder
                        match app_state_clone.clone().set_root_folder_with_persistence(folder_path.clone()).await {
                            Ok(_) => {
                                info!("Successfully changed file tree root to: {:?}", folder_path);
                            },
                            Err(e) => {
                                info!("Error changing file tree root: {}", e);
                            }
                        }
                    } else {
                        info!("Folder selection cancelled");
                    }
                });
            },
            "new_window" => {
                info!("Creating new window...");
                // TODO: Implement new window creation
            },
            "new_folder" => {
                info!("Creating new folder...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let current_folder = {
                        let nav_state = app_state_clone.navigation.read();
                        Some(nav_state.current_path.clone())
                    };
                    
                    if let Some(parent_path) = current_folder {
                        match create_new_folder_dialog(&parent_path).await {
                            Ok(new_folder_path) => {
                                info!("Successfully created folder: {:?}", new_folder_path);
                                // Refresh the file tree to show the new folder
                                if let Err(e) = app_state_clone.refresh_current_directory().await {
                                    info!("Error refreshing directory after folder creation: {}", e);
                                }
                            },
                            Err(e) => {
                                info!("Error creating new folder: {}", e);
                            }
                        }
                    } else {
                        info!("No current folder selected - cannot create new folder");
                    }
                });
            },
            "new_file" => {
                info!("Creating new file...");
                // TODO: Implement new file creation
            },
            "refresh" => {
                info!("Refreshing file view...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let current_path = app_state_clone.navigation.read().current_path.clone();
                    if let Err(e) = app_state_clone.navigate_to(current_path.clone()).await {
                        info!("Error refreshing directory: {}", e);
                    } else {
                        info!("Successfully refreshed directory: {:?}", current_path);
                    }
                });
            },
            "show_hidden" => {
                info!("Toggling hidden files visibility...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let show_hidden_files = {
                        let mut settings = app_state_clone.settings.write();
                        settings.show_hidden_files = !settings.show_hidden_files;
                        crate::state::save_settings_debounced(settings.clone());
                        settings.show_hidden_files
                    }; // settings write lock is dropped here
                    
                    info!("Hidden files visibility toggled to: {}", show_hidden_files);
                    
                    // Refresh the current directory to show/hide hidden files
                    let current_path = app_state_clone.navigation.read().current_path.clone();
                    if let Err(e) = app_state_clone.navigate_to(current_path).await {
                        info!("Error refreshing directory after hidden files toggle: {}", e);
                    }
                });
            },
            "open_with" => {
                info!("Opening with external application...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for open with");
                        return;
                    }
                    
                    // For now, just open with system default application
                    // In a full implementation, you'd show an application picker dialog
                    let file_path = &selected_files[0].path;
                    
                    match open_with_system_default(file_path).await {
                        Ok(_) => {
                            info!("Successfully opened file with system default: {:?}", file_path);
                        },
                        Err(e) => {
                            info!("Error opening file with system default: {}", e);
                        }
                    }
                });
            },
            "show_in_finder" => {
                info!("Showing selected item in Finder...");
                let app_state_clone = app_state.clone();
                
                spawn(async move {
                    let (selected_files, current_folder) = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        let nav_state = app_state_clone.navigation.read();
                        
                        let selected = file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>();
                        
                        (selected, Some(nav_state.current_path.clone()))
                    };
                    
                    // Determine what path to show in Finder
                    let path_to_show = if !selected_files.is_empty() {
                        // If files are selected, show the first selected file
                        Some(selected_files[0].path.clone())
                    } else if let Some(current) = current_folder {
                        // If no files selected, show the current folder
                        Some(current)
                    } else {
                        None
                    };
                    
                    if let Some(path) = path_to_show {
                        match show_in_system_file_manager(&path).await {
                            Ok(_) => {
                                info!("Successfully showed path in system file manager: {:?}", path);
                            },
                            Err(e) => {
                                info!("Error showing path in system file manager: {}", e);
                            }
                        }
                    } else {
                        info!("No path available to show in system file manager");
                    }
                });
            },
            
            // Edit menu items
            "clear_selection" => {
                info!("Clearing file selection...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.selection.write().clear_selection();
                info!("File selection cleared");
            },
            "copy_to" => {
                info!("Copying files to location...");
                // TODO: Implement copy to dialog
            },
            "move_to" => {
                info!("Moving files to location...");
                // TODO: Implement move to dialog
            },
            "delete" => {
                info!("Deleting selected files...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for deletion");
                        return;
                    }
                    
                    // Show confirmation dialog
                    let file_count = selected_files.len();
                    let confirmation_message = if file_count == 1 {
                        format!("Are you sure you want to delete '{}'?", selected_files[0].name)
                    } else {
                        format!("Are you sure you want to delete {} files?", file_count)
                    };
                    
                    match show_confirmation_dialog("Delete Files", &confirmation_message).await {
                        Ok(true) => {
                            match delete_selected_files(&selected_files).await {
                                Ok(deleted_count) => {
                                    info!("Successfully deleted {} files", deleted_count);
                                    // Refresh the file tree to reflect changes
                                    if let Err(e) = app_state_clone.refresh_current_directory().await {
                                        info!("Error refreshing directory after deletion: {}", e);
                                    }
                                },
                                Err(e) => {
                                    info!("Error deleting files: {}", e);
                                }
                            }
                        },
                        Ok(false) => {
                            info!("File deletion cancelled by user");
                        },
                        Err(e) => {
                            info!("Error showing confirmation dialog: {}", e);
                        }
                    }
                });
            },
            "rename" => {
                info!("Renaming selected file...");
                let mut app_state_clone = app_state.clone();
                
                spawn(async move {
                    let selected_files = {
                        let selection_state = app_state_clone.selection.read();
                        let file_entries = app_state_clone.file_entries.read();
                        
                        // Filter file entries to get only the selected ones
                        file_entries.iter()
                            .filter(|entry| selection_state.selected_files.contains(&entry.path))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    
                    if selected_files.is_empty() {
                        info!("No files selected for renaming");
                        return;
                    }
                    
                    if selected_files.len() > 1 {
                        info!("Multiple files selected - rename only works with single file selection");
                        return;
                    }
                    
                    let file_to_rename = &selected_files[0];
                    let current_name = &file_to_rename.name;
                    
                    // Show input dialog for new name
                    match show_rename_dialog(current_name).await {
                        Ok(new_name) => {
                            if new_name != *current_name {
                                match rename_file(&file_to_rename.path, &new_name).await {
                                    Ok(new_path) => {
                                        info!("Successfully renamed '{}' to '{}' (path: {:?})", current_name, new_name, new_path);
                                        // Refresh the file tree to reflect the rename
                                        if let Err(e) = app_state_clone.refresh_current_directory().await {
                                            info!("Error refreshing directory after rename: {}", e);
                                        }
                                    },
                                    Err(e) => {
                                        info!("Error renaming file: {}", e);
                                    }
                                }
                            } else {
                                info!("Rename cancelled - same name provided");
                            }
                        },
                        Err(e) => {
                            info!("Error showing rename dialog: {}", e);
                        }
                    }
                });
            },
            "duplicate" => {
                info!("Duplicating selected files...");
                // TODO: Implement file duplication
            },
            "settings" => {
                info!("Opening settings dialog...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.settings_dialog_visible.set(true);
                info!("Settings dialog opened via menu");
            },
            
            // View menu items
            "toggle_sidebar" => {
                info!("Toggling sidebar...");
                // TODO: Implement sidebar toggle
            },
            "toggle_panel" => {
                info!("Toggling panel...");
                // TODO: Implement panel toggle
            },
            "theme_light" => {
                info!("Switching to light theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Light;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to light theme");
                });
            },
            "theme_dark" => {
                info!("Switching to dark theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Dark;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to dark theme");
                });
            },
            "theme_auto" => {
                info!("Switching to auto theme...");
                let mut app_state_clone = app_state.clone();
                spawn(async move {
                    let mut settings = app_state_clone.settings.write();
                    settings.theme = crate::state::Theme::Auto;
                    crate::theme::ThemeManager::apply_theme(&settings.theme);
                    crate::state::save_settings_debounced(settings.clone());
                    info!("Successfully switched to auto theme");
                });
            },
            
            // Help menu items
            "keyboard_shortcuts" => {
                info!("Showing keyboard shortcuts...");
                let mut app_state_clone = app_state.clone();
                app_state_clone.cheat_sheet_visible.set(true);
                info!("Keyboard shortcuts dialog opened via menu");
            },
            "help_documentation" => {
                info!("Opening help documentation...");
                // TODO: Implement help documentation
            },
            
            _ => {
                info!("Unhandled menu item: {}", event_id);
            }
        }
    });
    
    rsx! {
        phase2_app {}
    }
}
