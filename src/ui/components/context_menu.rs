use dioxus::prelude::*;
use std::path::PathBuf;
use crate::services::FileEntry;
use crate::state::{use_app_state, use_selection_state};
use crate::ui::{use_shortcut_handler};
use crate::ui::shortcuts::ShortcutAction;

/// Context menu item action types
#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuAction {
    Copy,
    Cut,
    Paste,
    Delete,
    Rename,
    Properties,
    NewFolder,
    NewFile,
    Refresh,
    SelectAll,
    OpenWith,
    OpenInExplorer,
    Separator, // Visual separator in menu
}

impl ContextMenuAction {
    pub fn label(&self) -> &'static str {
        match self {
            ContextMenuAction::Copy => "Copy",
            ContextMenuAction::Cut => "Cut",
            ContextMenuAction::Paste => "Paste",
            ContextMenuAction::Delete => "Delete",
            ContextMenuAction::Rename => "Rename",
            ContextMenuAction::Properties => "Properties",
            ContextMenuAction::NewFolder => "New Folder",
            ContextMenuAction::NewFile => "New File",
            ContextMenuAction::Refresh => "Refresh",
            ContextMenuAction::SelectAll => "Select All",
            ContextMenuAction::OpenWith => "Open With...",
            ContextMenuAction::OpenInExplorer => "Show in Explorer",
            ContextMenuAction::Separator => "",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ContextMenuAction::Copy => "📋",
            ContextMenuAction::Cut => "✂️",
            ContextMenuAction::Paste => "📄",
            ContextMenuAction::Delete => "🗑️",
            ContextMenuAction::Rename => "✏️",
            ContextMenuAction::Properties => "ℹ️",
            ContextMenuAction::NewFolder => "📁",
            ContextMenuAction::NewFile => "📄",
            ContextMenuAction::Refresh => "🔄",
            ContextMenuAction::SelectAll => "☑️",
            ContextMenuAction::OpenWith => "🔗",
            ContextMenuAction::OpenInExplorer => "🔍",
            ContextMenuAction::Separator => "",
        }
    }

    pub fn shortcut(&self) -> Option<&'static str> {
        match self {
            ContextMenuAction::Copy => Some("Ctrl+C"),
            ContextMenuAction::Cut => Some("Ctrl+X"),
            ContextMenuAction::Paste => Some("Ctrl+V"),
            ContextMenuAction::Delete => Some("Del"),
            ContextMenuAction::Rename => Some("F2"),
            ContextMenuAction::Refresh => Some("F5"),
            ContextMenuAction::SelectAll => Some("Ctrl+A"),
            ContextMenuAction::NewFolder => Some("Ctrl+Shift+N"),
            _ => None,
        }
    }

    /// Convert to shortcut action for execution
    pub fn to_shortcut_action(&self) -> Option<ShortcutAction> {
        match self {
            ContextMenuAction::Copy => Some(ShortcutAction::Copy),
            ContextMenuAction::Cut => Some(ShortcutAction::Cut),
            ContextMenuAction::Paste => Some(ShortcutAction::Paste),
            ContextMenuAction::Delete => Some(ShortcutAction::Delete),
            ContextMenuAction::Rename => Some(ShortcutAction::Rename),
            ContextMenuAction::Refresh => Some(ShortcutAction::Refresh),
            ContextMenuAction::SelectAll => Some(ShortcutAction::SelectAll),
            ContextMenuAction::NewFolder => Some(ShortcutAction::NewFolder),
            ContextMenuAction::Properties => Some(ShortcutAction::ShowProperties),
            _ => None,
        }
    }

    /// Check if action is enabled for the current context
    pub fn is_enabled(&self, selected_files: &[PathBuf], has_clipboard: bool) -> bool {
        match self {
            ContextMenuAction::Copy | ContextMenuAction::Cut | ContextMenuAction::Delete => {
                !selected_files.is_empty()
            }
            ContextMenuAction::Paste => has_clipboard,
            ContextMenuAction::Rename => selected_files.len() == 1,
            ContextMenuAction::Properties => selected_files.len() == 1,
            ContextMenuAction::OpenWith => selected_files.len() == 1,
            ContextMenuAction::OpenInExplorer => selected_files.len() == 1,
            _ => true, // Actions like New Folder, Refresh, Select All are always enabled
        }
    }
}

/// Context menu position
#[derive(Debug, Clone)]
pub struct MenuPosition {
    pub x: f64,
    pub y: f64,
}

/// Context menu state
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    pub is_visible: bool,
    pub position: MenuPosition,
    pub target_file: Option<FileEntry>,
    pub menu_items: Vec<ContextMenuAction>,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            is_visible: false,
            position: MenuPosition { x: 0.0, y: 0.0 },
            target_file: None,
            menu_items: Vec::new(),
        }
    }
}

impl ContextMenuState {
    pub fn show_at(&mut self, x: f64, y: f64, target_file: Option<FileEntry>) {
        self.is_visible = true;
        self.position = MenuPosition { x, y };
        self.target_file = target_file.clone();
        
        // Generate appropriate menu items based on context
        self.menu_items = if target_file.is_some() {
            // File/folder selected
            vec![
                ContextMenuAction::Copy,
                ContextMenuAction::Cut,
                ContextMenuAction::Paste,
                ContextMenuAction::Separator,
                ContextMenuAction::Delete,
                ContextMenuAction::Rename,
                ContextMenuAction::Separator,
                ContextMenuAction::OpenWith,
                ContextMenuAction::OpenInExplorer,
                ContextMenuAction::Separator,
                ContextMenuAction::Properties,
            ]
        } else {
            // Empty space / background
            vec![
                ContextMenuAction::Paste,
                ContextMenuAction::Separator,
                ContextMenuAction::NewFolder,
                ContextMenuAction::NewFile,
                ContextMenuAction::Separator,
                ContextMenuAction::Refresh,
                ContextMenuAction::SelectAll,
            ]
        };
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
        self.target_file = None;
        self.menu_items.clear();
    }
}

/// Props for the context menu component
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    pub menu_state: Signal<ContextMenuState>,
    pub on_action: EventHandler<ContextMenuAction>,
}

/// Context menu component
#[component]
pub fn ContextMenu(mut props: ContextMenuProps) -> Element {
    let menu_state = props.menu_state.read();
    let app_state = use_app_state();
    let _selection_state = use_selection_state();
    
    if !menu_state.is_visible {
        return rsx! { div {} };
    }

    let menu_style = format!(
        "position: fixed; left: {}px; top: {}px; z-index: 1000; 
         background: white; border: 1px solid #ccc; border-radius: 4px; 
         box-shadow: 0 2px 8px rgba(0,0,0,0.15); min-width: 200px; 
         padding: 4px 0; font-size: 14px;",
        menu_state.position.x, menu_state.position.y
    );

    let selected_files = app_state.get_selected_files();
    let has_clipboard = false; // TODO: Implement clipboard detection

    rsx! {
        // Invisible overlay to close menu when clicking outside
        div {
            style: "position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; z-index: 999;",
            onclick: move |_| {
                props.menu_state.write().hide();
            }
        }
        
        // Context menu
        div {
            style: "{menu_style}",
            onclick: move |e| {
                // Prevent clicks inside menu from closing it
                e.stop_propagation();
            },
            
            {menu_state.menu_items.iter().map(|action| {
                match action {
                    ContextMenuAction::Separator => rsx! {
                        div {
                            key: "separator-{action:?}",
                            style: "height: 1px; background: #e0e0e0; margin: 4px 12px;"
                        }
                    },
                    _ => {
                        let is_enabled = action.is_enabled(&selected_files, has_clipboard);
                        let item_style = if is_enabled {
                            "padding: 8px 16px; cursor: pointer; display: flex; align-items: center; justify-content: space-between; color: #333;"
                        } else {
                            "padding: 8px 16px; cursor: not-allowed; display: flex; align-items: center; justify-content: space-between; color: #999;"
                        };
                        
                        let action_clone = action.clone();
                        rsx! {
                            div {
                                key: "item-{action:?}",
                                style: "{item_style}",
                                onmouseenter: move |_| {
                                    // TODO: Add hover effect
                                },
                                onclick: move |e| {
                                    if is_enabled {
                                        e.stop_propagation();
                                        props.on_action.call(action_clone.clone());
                                        props.menu_state.write().hide();
                                    }
                                },
                                
                                div {
                                    style: "display: flex; align-items: center; gap: 8px;",
                                    span { style: "font-size: 16px;", "{action.icon()}" }
                                    span { "{action.label()}" }
                                }
                                
                                {action.shortcut().map(|shortcut| rsx! {
                                    span {
                                        style: "font-size: 12px; color: #666; margin-left: 16px;",
                                        "{shortcut}"
                                    }
                                })}
                            }
                        }
                    }
                }
            })}
        }
    }
}

/// Hook to manage context menu state and actions
pub fn use_context_menu() -> (Signal<ContextMenuState>, impl Fn(ContextMenuAction)) {
    let menu_state = use_signal(ContextMenuState::default);
    let mut shortcut_handler = use_shortcut_handler();
    
    let handle_action = move |action: ContextMenuAction| {
        tracing::info!("Context menu action triggered: {:?}", action);
        
        // Convert to shortcut action and execute if possible
        if let Some(shortcut_action) = action.to_shortcut_action() {
            let mut handler = shortcut_handler.clone();
            spawn(async move {
                handler.execute_action(shortcut_action).await;
            });
        } else {
            // Handle custom actions that don't have shortcut equivalents
            match action {
                ContextMenuAction::OpenWith => {
                    tracing::info!("Open with action - TODO: implement");
                }
                ContextMenuAction::OpenInExplorer => {
                    tracing::info!("Open in explorer action - TODO: implement");
                }
                ContextMenuAction::NewFile => {
                    tracing::info!("New file action - TODO: implement");
                }
                _ => {
                    tracing::warn!("Unhandled context menu action: {:?}", action);
                }
            }
        }
    };
    
    (menu_state, handle_action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_menu_action_properties() {
        assert_eq!(ContextMenuAction::Copy.label(), "Copy");
        assert_eq!(ContextMenuAction::Copy.icon(), "📋");
        assert_eq!(ContextMenuAction::Copy.shortcut(), Some("Ctrl+C"));
        
        assert!(ContextMenuAction::Copy.is_enabled(&[PathBuf::from("test")], false));
        assert!(!ContextMenuAction::Copy.is_enabled(&[], false));
        
        assert!(ContextMenuAction::Paste.is_enabled(&[], true));
        assert!(!ContextMenuAction::Paste.is_enabled(&[], false));
    }

    #[test]
    fn test_context_menu_state() {
        let mut state = ContextMenuState::default();
        assert!(!state.is_visible);
        
        state.show_at(100.0, 200.0, None);
        assert!(state.is_visible);
        assert_eq!(state.position.x, 100.0);
        assert_eq!(state.position.y, 200.0);
        assert!(!state.menu_items.is_empty());
        
        state.hide();
        assert!(!state.is_visible);
        assert!(state.menu_items.is_empty());
    }

    #[test]
    fn test_menu_items_generation() {
        let mut state = ContextMenuState::default();
        
        // Test file context menu
        let file_entry = FileEntry {
            path: PathBuf::from("test.txt"),
            name: "test.txt".to_string(),
            file_type: crate::services::file_system::FileType::Text(
                crate::services::file_system::TextFormat::Plain
            ),
            size: 100,
            modified: std::time::SystemTime::now(),
            created: std::time::SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: crate::services::file_system::FilePermissions::read_write(),
        };
        
        state.show_at(0.0, 0.0, Some(file_entry));
        assert!(state.menu_items.contains(&ContextMenuAction::Copy));
        assert!(state.menu_items.contains(&ContextMenuAction::Properties));
        
        // Test background context menu
        state.show_at(0.0, 0.0, None);
        assert!(state.menu_items.contains(&ContextMenuAction::NewFolder));
        assert!(state.menu_items.contains(&ContextMenuAction::Refresh));
    }
}