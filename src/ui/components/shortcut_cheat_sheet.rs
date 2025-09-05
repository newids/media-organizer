use dioxus::prelude::*;
use dioxus::events::KeyboardEvent;
use crate::state::app_state::CommandRegistry;
use crate::ui::shortcuts::{ShortcutRegistry, KeyCombination};
use std::collections::HashMap;

/// Shortcut cheat sheet component that displays all available keyboard shortcuts
/// organized by category, with platform-aware display (Cmd vs Ctrl)
#[component]
pub fn ShortcutCheatSheet(
    /// Whether the cheat sheet is visible
    is_visible: bool,
    /// Callback to close the cheat sheet
    on_close: EventHandler<()>,
    /// Command registry for getting command information
    command_registry: Signal<CommandRegistry>,
) -> Element {
    if !is_visible {
        return rsx! { div {} };
    }

    // Get shortcuts from the shortcut handler
    let shortcut_handler = crate::ui::use_shortcut_handler();
    
    // Get all shortcuts organized by category (computed fresh each render for now)
    let shortcut_categories = get_shortcuts_by_category(&command_registry.read(), &shortcut_handler.registry);

    rsx! {
        div {
            class: "shortcut-cheat-sheet-overlay",
            onclick: move |_| on_close.call(()),
            
            div {
                class: "shortcut-cheat-sheet-modal",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "cheat-sheet-title",
                "aria-describedby": "cheat-sheet-description",
                tabindex: "-1",
                onclick: move |e| e.stop_propagation(), // Prevent closing when clicking inside modal
                onkeydown: {
                    let on_close_key = on_close.clone();
                    move |e: KeyboardEvent| {
                        // Handle keyboard navigation for accessibility
                        match e.key().to_string().as_str() {
                            "Escape" => {
                                e.prevent_default();
                                on_close_key.call(());
                            }
                            "Tab" => {
                                // Allow tab to cycle through focusable elements within modal
                                // Browser handles this naturally with proper tabindex values
                            }
                            _ => {}
                        }
                    }
                },
                
                // Header
                div { class: "cheat-sheet-header",
                    h2 { 
                        id: "cheat-sheet-title",
                        class: "cheat-sheet-title", 
                        "Keyboard Shortcuts" 
                    }
                    button {
                        class: "cheat-sheet-close-button",
                        "aria-label": "Close keyboard shortcuts cheat sheet",
                        tabindex: "0",
                        onclick: move |_| on_close.call(()),
                        onkeydown: {
                            let on_close_key = on_close.clone();
                            move |e: KeyboardEvent| {
                                // Handle Enter and Space as click for accessibility
                                match e.key().to_string().as_str() {
                                    "Enter" | " " => {
                                        e.prevent_default();
                                        on_close_key.call(());
                                    }
                                    _ => {}
                                }
                            }
                        },
                        "×"
                    }
                }
                
                // Content with scrollable categories
                div { 
                    id: "cheat-sheet-description",
                    class: "cheat-sheet-content",
                    role: "main",
                    "aria-label": "List of keyboard shortcuts organized by category",
                    for (category, shortcuts) in shortcut_categories.iter() {
                        ShortcutCategory {
                            category_name: category.clone(),
                            shortcuts: shortcuts.clone(),
                        }
                    }
                }
                
                // Footer with help text
                div { 
                    class: "cheat-sheet-footer",
                    role: "contentinfo",
                    span { 
                        class: "cheat-sheet-help-text",
                        "Press F1 to toggle this cheat sheet • Press Escape to close"
                    }
                }
            }
        }
    }
}

/// Individual shortcut category section
#[component]
fn ShortcutCategory(
    /// Name of the category (e.g., "File Operations", "Navigation")
    category_name: String,
    /// List of shortcuts in this category
    shortcuts: Vec<ShortcutInfo>,
) -> Element {
    let category_id = format!("category-{}", category_name.replace(' ', "-").to_lowercase());
    let list_id = format!("shortcuts-{}", category_name.replace(' ', "-").to_lowercase());
    
    rsx! {
        section { 
            class: "shortcut-category",
            "aria-labelledby": "{category_id}",
            h3 { 
                id: "{category_id}",
                class: "category-title", 
                "{category_name}" 
            }
            ul { 
                id: "{list_id}",
                class: "shortcuts-list",
                role: "list",
                "aria-label": "Shortcuts for {category_name}",
                for shortcut in shortcuts {
                    ShortcutRow {
                        shortcut_info: shortcut,
                    }
                }
            }
        }
    }
}

/// Individual shortcut row displaying the key combination and description
#[component]
fn ShortcutRow(
    /// Information about the shortcut to display
    shortcut_info: ShortcutInfo,
) -> Element {
    // Create screen reader friendly description of the shortcut
    let keys_text = shortcut_info.display_keys.join(" plus ");
    let aria_label = format!("Shortcut: {} performs {}", keys_text, shortcut_info.description);
    
    rsx! {
        li { 
            class: "shortcut-row",
            role: "listitem",
            "aria-label": "{aria_label}",
            div { 
                class: "shortcut-keys",
                "aria-label": "Keyboard combination: {keys_text}",
                for (i, key_part) in shortcut_info.display_keys.iter().enumerate() {
                    if i > 0 {
                        span { 
                            class: "key-separator", 
                            " + " 
                        }
                    }
                    kbd { 
                        class: "key",
                        "aria-label": "{key_part} key",
                        "{key_part}" 
                    }
                }
            }
            div { 
                class: "shortcut-description",
                "aria-label": "Action: {shortcut_info.description}",
                "{shortcut_info.description}"
            }
        }
    }
}

/// Information about a keyboard shortcut for display purposes
#[derive(Clone, Debug, PartialEq)]
pub struct ShortcutInfo {
    /// Command ID this shortcut triggers
    pub command_id: String,
    /// Human-readable description of what the shortcut does
    pub description: String,
    /// Platform-aware key combination display (e.g., ["Cmd", "Shift", "P"])
    pub display_keys: Vec<String>,
    /// Raw shortcut string for reference
    pub shortcut_string: String,
}

/// Get all shortcuts organized by category for display
fn get_shortcuts_by_category(
    command_registry: &CommandRegistry,
    shortcut_registry: &ShortcutRegistry,
) -> HashMap<String, Vec<ShortcutInfo>> {
    let mut categories: HashMap<String, Vec<ShortcutInfo>> = HashMap::new();
    
    // Get all shortcuts from the shortcut registry
    let all_shortcuts = shortcut_registry.get_all_shortcuts();
    
    // Process each shortcut and categorize it
    for (key_combo, action) in all_shortcuts {
        let category = get_category_for_action(&action);
        let description = action.description().to_string();
        let display_keys = get_platform_aware_keys(&key_combo);
        let shortcut_string = key_combo.description();
        
        let shortcut_info = ShortcutInfo {
            command_id: format!("{:?}", action),
            description,
            display_keys,
            shortcut_string,
        };
        
        categories.entry(category).or_insert_with(Vec::new).push(shortcut_info);
    }
    
    // Also add commands from command registry that have shortcuts
    for (command_id, command) in &command_registry.commands {
        for shortcut_str in &command.shortcuts {
            if let Some(key_combo) = parse_shortcut_string(shortcut_str) {
                let category = command.category.clone();
                let description = command.title.clone();
                let display_keys = get_platform_aware_keys(&key_combo);
                
                let shortcut_info = ShortcutInfo {
                    command_id: command_id.clone(),
                    description,
                    display_keys,
                    shortcut_string: shortcut_str.clone(),
                };
                
                categories.entry(category).or_insert_with(Vec::new).push(shortcut_info);
            }
        }
    }
    
    // Sort shortcuts within each category by description
    for shortcuts in categories.values_mut() {
        shortcuts.sort_by(|a, b| a.description.cmp(&b.description));
    }
    
    categories
}

/// Get the appropriate category name for a shortcut action
fn get_category_for_action(action: &crate::ui::shortcuts::ShortcutAction) -> String {
    use crate::ui::shortcuts::ShortcutAction;
    
    match action {
        ShortcutAction::Copy | ShortcutAction::Paste | ShortcutAction::Cut 
        | ShortcutAction::Delete | ShortcutAction::Rename => "File Operations".to_string(),
        
        ShortcutAction::SelectAll | ShortcutAction::ClearSelection => "Selection".to_string(),
        
        ShortcutAction::NavigateUp | ShortcutAction::NavigateBack 
        | ShortcutAction::NavigateForward | ShortcutAction::NavigateHome 
        | ShortcutAction::Refresh => "Navigation".to_string(),
        
        ShortcutAction::OpenFile | ShortcutAction::NewFolder => "File Management".to_string(),
        
        ShortcutAction::TogglePreview | ShortcutAction::ToggleSearch 
        | ShortcutAction::ShowProperties | ShortcutAction::ZoomIn 
        | ShortcutAction::ZoomOut | ShortcutAction::ToggleSpace => "View".to_string(),
        
        ShortcutAction::FocusExplorer | ShortcutAction::FocusEditor1 
        | ShortcutAction::FocusEditor2 | ShortcutAction::FocusEditor3 
        | ShortcutAction::CloseTab | ShortcutAction::SwitchTab => "Editor".to_string(),
        
        ShortcutAction::ShowSettings | ShortcutAction::ShowCommandPalette 
        | ShortcutAction::ShowShortcutCheatSheet | ShortcutAction::ToggleHighContrast => "Application".to_string(),
        
        ShortcutAction::Custom(_) => "Custom".to_string(),
    }
}

/// Convert key combination to platform-aware display keys
fn get_platform_aware_keys(key_combo: &KeyCombination) -> Vec<String> {
    let mut keys = Vec::new();
    
    // Add modifier keys in standard order
    if key_combo.ctrl {
        #[cfg(target_os = "macos")]
        keys.push("⌘".to_string()); // Command symbol on macOS
        #[cfg(not(target_os = "macos"))]
        keys.push("Ctrl".to_string());
    }
    
    if key_combo.shift {
        #[cfg(target_os = "macos")]
        keys.push("⇧".to_string()); // Shift symbol on macOS
        #[cfg(not(target_os = "macos"))]
        keys.push("Shift".to_string());
    }
    
    if key_combo.alt {
        #[cfg(target_os = "macos")]
        keys.push("⌥".to_string()); // Option symbol on macOS
        #[cfg(not(target_os = "macos"))]
        keys.push("Alt".to_string());
    }
    
    if key_combo.meta {
        #[cfg(target_os = "macos")]
        keys.push("⌃".to_string()); // Control symbol on macOS (rarely used)
        #[cfg(not(target_os = "macos"))]
        keys.push("Win".to_string());
    }
    
    // Add the main key, with special formatting for certain keys
    let main_key = match key_combo.key.as_str() {
        " " => "Space".to_string(),
        "Enter" => "↵".to_string(),
        "Tab" => "⇥".to_string(),
        "Escape" => "Esc".to_string(),
        "ArrowUp" => "↑".to_string(),
        "ArrowDown" => "↓".to_string(),
        "ArrowLeft" => "←".to_string(),
        "ArrowRight" => "→".to_string(),
        "Delete" => "⌦".to_string(),
        "Backspace" => "⌫".to_string(),
        key => key.to_uppercase(),
    };
    
    keys.push(main_key);
    keys
}

/// Parse a shortcut string into a KeyCombination (simplified parser)
fn parse_shortcut_string(shortcut: &str) -> Option<KeyCombination> {
    let parts: Vec<&str> = shortcut.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }
    
    let mut key_combo = KeyCombination::new(parts.last()?);
    
    for part in &parts[..parts.len().saturating_sub(1)] {
        match part.to_lowercase().as_str() {
            "ctrl" => key_combo.ctrl = true,
            "shift" => key_combo.shift = true,
            "alt" => key_combo.alt = true,
            "meta" | "cmd" | "win" => key_combo.meta = true,
            _ => {}
        }
    }
    
    Some(key_combo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::shortcuts::ShortcutAction;

    #[test]
    fn test_category_assignment() {
        assert_eq!(get_category_for_action(&ShortcutAction::Copy), "File Operations");
        assert_eq!(get_category_for_action(&ShortcutAction::NavigateUp), "Navigation");
        assert_eq!(get_category_for_action(&ShortcutAction::TogglePreview), "View");
        assert_eq!(get_category_for_action(&ShortcutAction::FocusExplorer), "Editor");
        assert_eq!(get_category_for_action(&ShortcutAction::ShowCommandPalette), "Application");
    }

    #[test]
    fn test_platform_aware_keys() {
        let key_combo = KeyCombination::new("c").with_ctrl().with_shift();
        let display_keys = get_platform_aware_keys(&key_combo);
        
        #[cfg(target_os = "macos")]
        assert_eq!(display_keys, vec!["⌘", "⇧", "C"]);
        
        #[cfg(not(target_os = "macos"))]
        assert_eq!(display_keys, vec!["Ctrl", "Shift", "C"]);
    }

    #[test]
    fn test_shortcut_string_parsing() {
        let parsed = parse_shortcut_string("Ctrl+Shift+P").unwrap();
        assert_eq!(parsed.key, "P");
        assert!(parsed.ctrl);
        assert!(parsed.shift);
        assert!(!parsed.alt);
        assert!(!parsed.meta);
    }

    #[test]
    fn test_special_key_formatting() {
        let space_combo = KeyCombination::new(" ");
        let display_keys = get_platform_aware_keys(&space_combo);
        assert_eq!(display_keys, vec!["Space"]);
        
        let enter_combo = KeyCombination::new("Enter");
        let display_keys = get_platform_aware_keys(&enter_combo);
        assert_eq!(display_keys, vec!["↵"]);
    }

    #[test]
    fn test_f1_key_formatting() {
        let f1_combo = KeyCombination::new("F1");
        let display_keys = get_platform_aware_keys(&f1_combo);
        assert_eq!(display_keys, vec!["F1"]);
    }

    #[test]
    fn test_cheat_sheet_categories_comprehensive() {
        // Test that all major shortcut actions have appropriate categories
        use crate::ui::shortcuts::ShortcutAction;
        
        let test_cases = vec![
            (ShortcutAction::Copy, "File Operations"),
            (ShortcutAction::NavigateUp, "Navigation"),
            (ShortcutAction::TogglePreview, "View"),
            (ShortcutAction::FocusExplorer, "Editor"),
            (ShortcutAction::ShowShortcutCheatSheet, "Application"),
            (ShortcutAction::SelectAll, "Selection"),
        ];
        
        for (action, expected_category) in test_cases {
            let actual_category = get_category_for_action(&action);
            assert_eq!(actual_category, expected_category, 
                      "Action {:?} should be in category '{}', but was in '{}'", 
                      action, expected_category, actual_category);
        }
    }

    #[test]
    fn test_shortcut_info_equality() {
        let info1 = ShortcutInfo {
            command_id: "test.command".to_string(),
            description: "Test Command".to_string(),
            display_keys: vec!["Ctrl".to_string(), "T".to_string()],
            shortcut_string: "Ctrl+T".to_string(),
        };
        
        let info2 = ShortcutInfo {
            command_id: "test.command".to_string(),
            description: "Test Command".to_string(),
            display_keys: vec!["Ctrl".to_string(), "T".to_string()],
            shortcut_string: "Ctrl+T".to_string(),
        };
        
        assert_eq!(info1, info2);
    }
}