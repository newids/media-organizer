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
                // TODO: Implement new folder creation
            },
            "new_file" => {
                info!("Creating new file...");
                // TODO: Implement new file creation
            },
            "refresh" => {
                info!("Refreshing file view...");
                // TODO: Implement refresh functionality  
            },
            "show_hidden" => {
                info!("Toggling hidden files visibility...");
                // TODO: Implement hidden files toggle
            },
            "open_with" => {
                info!("Opening with external application...");
                // TODO: Implement open with dialog
            },
            "show_in_finder" => {
                info!("Showing selected item in Finder...");
                // TODO: Implement show in Finder
            },
            
            // Edit menu items
            "clear_selection" => {
                info!("Clearing file selection...");
                // TODO: Implement clear selection
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
                // TODO: Implement delete confirmation and action
            },
            "rename" => {
                info!("Renaming selected file...");
                // TODO: Implement inline rename functionality
            },
            "duplicate" => {
                info!("Duplicating selected files...");
                // TODO: Implement file duplication
            },
            "settings" => {
                info!("Opening settings dialog...");
                // TODO: The settings dialog opening will be handled by the UI state
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
                // TODO: Implement theme switching
            },
            "theme_dark" => {
                info!("Switching to dark theme...");
                // TODO: Implement theme switching
            },
            "theme_auto" => {
                info!("Switching to auto theme...");
                // TODO: Implement theme switching
            },
            
            // Help menu items
            "keyboard_shortcuts" => {
                info!("Showing keyboard shortcuts...");
                // TODO: Implement keyboard shortcuts dialog
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
