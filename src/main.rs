use tracing::info;

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

    // File menu
    let file_menu = Submenu::new("File", true);
    file_menu.append_items(&[
        &MenuItem::with_id("new_folder", "New Folder", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyN))),
        &MenuItem::with_id("refresh", "Refresh", true, Some(Accelerator::new(Some(Modifiers::SUPER), Code::KeyR))),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("show_hidden", "Show Hidden Files", true, Some(Accelerator::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::Period))),
    ]).unwrap();
    menu.append(&file_menu).unwrap();

    // Edit menu - enhance existing one
    let edit_menu = Submenu::new("Edit", true);
    edit_menu.append_items(&[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
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

    // Help menu - enhance existing one  
    let help_menu = Submenu::new("Help", true);
    help_menu.append_items(&[
        &MenuItem::with_id("about", "About Media Organizer", true, None),
        &MenuItem::with_id("keyboard_shortcuts", "Keyboard Shortcuts", true, None),
    ]).unwrap();
    menu.append(&help_menu).unwrap();

    menu
}


// Root app component with state provider
fn app() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    use dioxus::desktop::use_muda_event_handler;
    use crate::state::use_app_state;
    
    // Handle menu events
    use_muda_event_handler(move |menu_event| {
        let event_id = menu_event.id.0.as_str();
        info!("Menu item clicked: {}", event_id);
        
        match event_id {
            "settings" => {
                info!("Opening settings dialog...");
                // The settings dialog opening will be handled by the UI state
            },
            "new_folder" => {
                info!("Creating new folder...");
                // TODO: Implement new folder creation
            },
            "refresh" => {
                info!("Refreshing file view...");
                // TODO: Implement refresh functionality  
            },
            "show_hidden" => {
                info!("Toggling hidden files visibility...");
                // TODO: Implement hidden files toggle
            },
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
            "about" => {
                info!("Showing about dialog...");
                // TODO: Implement about dialog
            },
            "keyboard_shortcuts" => {
                info!("Showing keyboard shortcuts...");
                // TODO: Implement keyboard shortcuts dialog
            },
            _ => {
                info!("Unhandled menu item: {}", event_id);
            }
        }
    });
    
    rsx! {
        AppStateProvider {
            phase2_app {}
        }
    }
}
