use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::FaMagnifyingGlass;
use dioxus_free_icons::Icon;
use std::collections::HashMap;

use crate::state::{
    use_app_state, Command, CommandPaletteState, 
    SystemCommand, EditorCommand, FileCommand, NavigationCommand, ViewCommand
};

/// Command palette component for VS Code-style command searching and execution
#[component]
pub fn CommandPalette() -> Element {
    let mut app_state = use_app_state();
    let palette_state = &app_state.command_registry.read().palette_state;
    
    // Only render if the palette is visible
    if !palette_state.is_visible {
        return rsx! {};
    }

    // Get filtered commands based on search query
    let filtered_commands = get_filtered_commands(&app_state.command_registry.read().commands, &palette_state.search_query);
    
    rsx! {
        div {
            class: "command-palette-overlay",
            onclick: move |_| {
                app_state.command_registry.write().palette_state.is_visible = false;
            },
            div {
                class: "command-palette",
                role: "dialog",
                "aria-modal": "true",
                "aria-labelledby": "command-palette-label",
                "aria-describedby": "command-palette-description",
                onclick: move |e| e.stop_propagation(),
                
                // Hidden labels for screen readers
                h1 {
                    id: "command-palette-label",
                    class: "sr-only",
                    "Command Palette"
                }
                p {
                    id: "command-palette-description", 
                    class: "sr-only",
                    "Search and execute commands. Use arrow keys to navigate, Enter to execute, Escape to close."
                }
                
                // Search input
                div {
                    class: "command-palette-search",
                    Icon {
                        width: 16,
                        height: 16,
                        icon: FaMagnifyingGlass,
                        class: "search-icon"
                    }
                    input {
                        r#type: "text",
                        placeholder: "Type a command to search...",
                        value: "{palette_state.search_query}",
                        class: "search-input",
                        "aria-label": "Search commands",
                        "aria-describedby": "search-results-count",
                        autofocus: true,
                        oninput: move |e| {
                            app_state.command_registry.write().palette_state.search_query = e.value();
                            // Reset selection when search changes
                            app_state.command_registry.write().palette_state.selected_index = 0;
                        },
                        onkeydown: {
                            let mut app_state_clone = app_state.clone();
                            let command_count = filtered_commands.len();
                            move |e| {
                                handle_keyboard_navigation(e, &mut app_state_clone, command_count);
                            }
                        }
                    }
                }
                
                // Results count for screen readers
                div {
                    id: "search-results-count",
                    class: "sr-only",
                    "aria-live": "polite",
                    "aria-atomic": "true",
                    if filtered_commands.is_empty() {
                        "No matching commands found"
                    } else {
                        "{filtered_commands.len()} commands found"
                    }
                }
                
                // Command list
                div {
                    class: "command-palette-list",
                    role: "listbox",
                    "aria-label": "Available commands",
                    "aria-activedescendant": if !filtered_commands.is_empty() && palette_state.selected_index < filtered_commands.len() {
                        "command-item-{palette_state.selected_index}"
                    } else {
                        ""
                    },
                    if filtered_commands.is_empty() {
                        div {
                            class: "no-commands",
                            role: "status",
                            "aria-live": "polite",
                            "No matching commands found"
                        }
                    } else {
                        for (index, command) in filtered_commands.iter().enumerate() {
                            CommandItem {
                                command: command.clone(),
                                is_selected: index == palette_state.selected_index,
                                index: index,
                            }
                        }
                    }
                }
                
                // Footer with hint
                div {
                    class: "command-palette-footer",
                    role: "contentinfo",
                    span {
                        class: "hint",
                        "aria-live": "polite",
                        "↑↓ to navigate • Enter to execute • Escape to close"
                    }
                }
            }
        }
    }
}

/// Individual command item in the palette
#[component]
fn CommandItem(command: Command, is_selected: bool, index: usize) -> Element {
    let mut app_state = use_app_state();
    
    // Create accessible description
    let shortcuts_text = if !command.shortcuts.is_empty() {
        format!(" Keyboard shortcut: {}", command.shortcuts.join(", "))
    } else {
        String::new()
    };
    
    let aria_label = format!("{}.{}{}", 
        command.title,
        command.description.as_ref().map(|d| format!(" {}", d)).unwrap_or_default(),
        shortcuts_text
    );
    
    rsx! {
        div {
            id: "command-item-{index}",
            class: if is_selected { "command-item selected" } else { "command-item" },
            role: "option",
            "aria-selected": "{is_selected}",
            "aria-label": "{aria_label}",
            tabindex: if is_selected { "0" } else { "-1" },
            onclick: {
                let mut app_state_click = app_state.clone();
                let command_click = command.clone();
                move |_| {
                    execute_command(&command_click, &mut app_state_click);
                }
            },
            onmouseenter: {
                let mut app_state_mouse = app_state.clone();
                move |_| {
                    app_state_mouse.command_registry.write().palette_state.selected_index = index;
                }
            },
            
            div {
                class: "command-content",
                div {
                    class: "command-title",
                    "{command.title}"
                }
                if let Some(description) = &command.description {
                    if !description.is_empty() {
                        div {
                            class: "command-description",
                            "{description}"
                        }
                    }
                }
            }
            
            if !command.shortcuts.is_empty() {
                div {
                    class: "command-shortcuts",
                    for shortcut in &command.shortcuts {
                        span {
                            class: "shortcut-key",
                            "{shortcut}"
                        }
                    }
                }
            }
        }
    }
}

/// Handle keyboard navigation within the command palette
fn handle_keyboard_navigation(e: KeyboardEvent, app_state: &mut crate::state::AppState, command_count: usize) {
    if command_count == 0 {
        return;
    }
    
    match e.key().to_string().as_str() {
        "ArrowUp" => {
            e.prevent_default();
            let current = app_state.command_registry.read().palette_state.selected_index;
            let new_index = if current == 0 { command_count - 1 } else { current - 1 };
            app_state.command_registry.write().palette_state.selected_index = new_index;
        }
        "ArrowDown" => {
            e.prevent_default();
            let current = app_state.command_registry.read().palette_state.selected_index;
            let new_index = if current >= command_count - 1 { 0 } else { current + 1 };
            app_state.command_registry.write().palette_state.selected_index = new_index;
        }
        "Enter" => {
            e.prevent_default();
            let commands = get_filtered_commands(
                &app_state.command_registry.read().commands, 
                &app_state.command_registry.read().palette_state.search_query
            );
            let selected_index = app_state.command_registry.read().palette_state.selected_index;
            if let Some(command) = commands.get(selected_index) {
                execute_command(command, app_state);
            }
        }
        "Escape" => {
            e.prevent_default();
            app_state.command_registry.write().palette_state.is_visible = false;
        }
        _ => {}
    }
}

/// Execute a command based on its handler type
fn execute_command(command: &Command, app_state: &mut crate::state::AppState) {
    use crate::state::CommandHandler;
    
    // Close the palette first
    app_state.command_registry.write().palette_state.is_visible = false;
    
    match &command.handler {
        CommandHandler::System(system_cmd) => {
            execute_system_command(system_cmd, app_state);
        }
        CommandHandler::Editor(editor_cmd) => {
            execute_editor_command(editor_cmd, app_state);
        }
        CommandHandler::File(file_cmd) => {
            execute_file_command(file_cmd, app_state);
        }
        CommandHandler::Navigation(nav_cmd) => {
            execute_navigation_command(nav_cmd, app_state);
        }
        CommandHandler::View(view_cmd) => {
            execute_view_command(view_cmd, app_state);
        }
    }
}

/// Execute system commands
fn execute_system_command(command: &SystemCommand, app_state: &mut crate::state::AppState) {
    match command {
        SystemCommand::ShowCommandPalette => {
            app_state.command_registry.write().palette_state.is_visible = true;
            app_state.command_registry.write().palette_state.search_query.clear();
            app_state.command_registry.write().palette_state.selected_index = 0;
        }
        SystemCommand::ToggleSettings => {
            // TODO: Implement settings panel toggle
            println!("Toggle settings panel");
        }
        SystemCommand::ShowKeyboardShortcuts => {
            // TODO: Implement keyboard shortcuts help dialog
            println!("Show keyboard shortcuts help");
        }
        SystemCommand::Quit => {
            // TODO: Implement graceful application quit
            println!("Quit application");
        }
    }
}

/// Execute editor commands
fn execute_editor_command(command: &EditorCommand, app_state: &mut crate::state::AppState) {
    match command {
        EditorCommand::CloseTab => {
            // TODO: Implement tab closing based on actual EditorGroup structure
            println!("Close tab");
        }
        EditorCommand::CloseAllTabs => {
            // TODO: Implement close all tabs based on actual EditorGroup structure
            println!("Close all tabs");
        }
        EditorCommand::FocusNextGroup => {
            // TODO: Implement group navigation
            println!("Navigate to next group");
        }
        EditorCommand::FocusPreviousGroup => {
            // TODO: Implement group navigation
            println!("Navigate to previous group");
        }
        EditorCommand::CloseOtherTabs => {
            // TODO: Implement close other tabs
            println!("Close other tabs");
        }
        EditorCommand::CloseTabsToRight => {
            // TODO: Implement close tabs to right
            println!("Close tabs to right");
        }
        EditorCommand::ToggleTabPin => {
            // TODO: Implement toggle tab pin
            println!("Toggle tab pin");
        }
        EditorCommand::SplitHorizontal => {
            // TODO: Implement horizontal split
            println!("Split horizontal");
        }
        EditorCommand::SplitVertical => {
            // TODO: Implement vertical split
            println!("Split vertical");
        }
    }
}

/// Execute file commands
fn execute_file_command(command: &FileCommand, app_state: &mut crate::state::AppState) {
    match command {
        FileCommand::NewFile => {
            // TODO: Implement new file creation
            println!("Create new file");
        }
        FileCommand::OpenFile => {
            // TODO: Implement file opening dialog
            println!("Open file dialog");
        }
        FileCommand::SaveFile => {
            // TODO: Implement file saving
            println!("Save current file");
        }
        FileCommand::SaveAllFiles => {
            // TODO: Implement save all files
            println!("Save all files");
        }
        FileCommand::RenameFile => {
            // TODO: Implement rename file
            println!("Rename file");
        }
        FileCommand::DeleteFile => {
            // TODO: Implement delete file
            println!("Delete file");
        }
        FileCommand::CopyFile => {
            // TODO: Implement copy file
            println!("Copy file");
        }
        FileCommand::CutFile => {
            // TODO: Implement cut file
            println!("Cut file");
        }
        FileCommand::PasteFile => {
            // TODO: Implement paste file
            println!("Paste file");
        }
    }
}

/// Execute navigation commands
fn execute_navigation_command(command: &NavigationCommand, app_state: &mut crate::state::AppState) {
    match command {
        NavigationCommand::GoToHome => {
            // TODO: Implement go to home functionality
            println!("Go to home");
        }
        NavigationCommand::ToggleFileTree => {
            // TODO: Implement file tree toggle
            println!("Toggle file tree");
        }
        NavigationCommand::NavigateBack => {
            // TODO: Implement navigation history back
            println!("Navigate back");
        }
        NavigationCommand::NavigateForward => {
            // TODO: Implement navigation history forward
            println!("Navigate forward");
        }
        NavigationCommand::NavigateUp => {
            // TODO: Implement navigate up
            println!("Navigate up");
        }
    }
}

/// Execute view commands
fn execute_view_command(command: &ViewCommand, app_state: &mut crate::state::AppState) {
    match command {
        ViewCommand::ToggleSidebar => {
            let is_collapsed = app_state.sidebar_state.read().is_collapsed;
            app_state.sidebar_state.write().is_collapsed = !is_collapsed;
        }
        ViewCommand::TogglePanel => {
            let is_visible = app_state.panel_state.read().is_visible;
            app_state.panel_state.write().is_visible = !is_visible;
        }
        ViewCommand::ToggleViewMode => {
            // TODO: Implement view mode toggle
            println!("Toggle view mode");
        }
        ViewCommand::ZoomIn => {
            // TODO: Implement zoom functionality
            println!("Zoom in");
        }
        ViewCommand::ZoomOut => {
            // TODO: Implement zoom functionality
            println!("Zoom out");
        }
        ViewCommand::ResetZoom => {
            // TODO: Implement zoom reset
            println!("Reset zoom");
        }
        ViewCommand::GridView => {
            // TODO: Implement grid view
            println!("Switch to grid view");
        }
        ViewCommand::ListView => {
            // TODO: Implement list view
            println!("Switch to list view");
        }
        ViewCommand::PreviewView => {
            // TODO: Implement preview view
            println!("Switch to preview view");
        }
    }
}

/// Get filtered commands based on search query using fuzzy search
fn get_filtered_commands(commands: &HashMap<String, Command>, search_query: &str) -> Vec<Command> {
    if search_query.is_empty() {
        // Return all commands sorted by category and title
        let mut all_commands: Vec<Command> = commands.values().cloned().collect();
        all_commands.sort_by(|a, b| {
            a.category.cmp(&b.category).then_with(|| a.title.cmp(&b.title))
        });
        return all_commands;
    }

    let query_lower = search_query.to_lowercase();
    let mut scored_commands: Vec<(Command, i32)> = Vec::new();

    for command in commands.values() {
        let mut score = calculate_fuzzy_score(&command.title.to_lowercase(), &query_lower);
        
        // Add description score if description exists
        if let Some(description) = &command.description {
            score += calculate_fuzzy_score(&description.to_lowercase(), &query_lower) / 2;
        }
        
        if score > 0 {
            scored_commands.push((command.clone(), score));
        }
    }

    // Sort by score (descending) then by title (ascending)
    scored_commands.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title.cmp(&b.0.title)));
    
    scored_commands.into_iter().map(|(command, _)| command).collect()
}

/// Calculate fuzzy matching score for a string against a query
/// Returns higher scores for better matches, 0 for no match
fn calculate_fuzzy_score(text: &str, query: &str) -> i32 {
    if query.is_empty() {
        return 1;
    }
    
    if text.is_empty() {
        return 0;
    }

    // Exact match gets highest score
    if text == query {
        return 1000;
    }
    
    // Exact prefix match gets high score
    if text.starts_with(query) {
        return 500;
    }
    
    // Contains match gets medium score
    if text.contains(query) {
        return 250;
    }
    
    // Fuzzy character matching
    let mut score = 0;
    let mut query_chars = query.chars().peekable();
    let mut consecutive_matches = 0;
    let mut last_match_index = 0;

    for (i, text_char) in text.chars().enumerate() {
        if let Some(&query_char) = query_chars.peek() {
            if text_char == query_char {
                query_chars.next();
                
                // Bonus for consecutive matches
                if i == last_match_index + 1 {
                    consecutive_matches += 1;
                    score += consecutive_matches * 10;
                } else {
                    consecutive_matches = 1;
                    score += 5;
                }
                
                last_match_index = i;
                
                // All query characters matched
                if query_chars.peek().is_none() {
                    score += 50; // Bonus for complete fuzzy match
                    break;
                }
            }
        }
    }
    
    // Return 0 if not all query characters were found
    if query_chars.peek().is_some() {
        return 0;
    }
    
    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Command, CommandHandler, SystemCommand};

    #[test]
    fn test_fuzzy_score_exact_match() {
        assert_eq!(calculate_fuzzy_score("test", "test"), 1000);
    }

    #[test]
    fn test_fuzzy_score_prefix_match() {
        assert_eq!(calculate_fuzzy_score("testing", "test"), 500);
    }

    #[test]
    fn test_fuzzy_score_contains_match() {
        assert!(calculate_fuzzy_score("my test file", "test") >= 250);
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert_eq!(calculate_fuzzy_score("hello", "xyz"), 0);
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        assert_eq!(calculate_fuzzy_score("hello", ""), 1);
    }

    #[test]
    fn test_fuzzy_score_fuzzy_match() {
        // "goto" should match "GoToFile" with some score
        let score = calculate_fuzzy_score("gotofile", "goto");
        assert!(score > 0);
        assert!(score < 250); // Less than contains match
    }

    #[test]
    fn test_get_filtered_commands_empty_query() {
        let mut commands = HashMap::new();
        commands.insert("test1".to_string(), Command {
            id: "test1".to_string(),
            title: "Test Command 1".to_string(),
            description: "First test".to_string(),
            category: "Test".to_string(),
            shortcuts: vec![],
            handler: CommandHandler::System(SystemCommand::ShowCommandPalette),
        });
        commands.insert("test2".to_string(), Command {
            id: "test2".to_string(),
            title: "Another Command".to_string(),
            description: "Second test".to_string(),
            category: "Test".to_string(),
            shortcuts: vec![],
            handler: CommandHandler::System(SystemCommand::ToggleSettings),
        });

        let filtered = get_filtered_commands(&commands, "");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_filtered_commands_with_query() {
        let mut commands = HashMap::new();
        commands.insert("test1".to_string(), Command {
            id: "test1".to_string(),
            title: "Test Command".to_string(),
            description: "First test".to_string(),
            category: "Test".to_string(),
            shortcuts: vec![],
            handler: CommandHandler::System(SystemCommand::ShowCommandPalette),
        });
        commands.insert("other".to_string(), Command {
            id: "other".to_string(),
            title: "Other Command".to_string(),
            description: "Different".to_string(),
            category: "Other".to_string(),
            shortcuts: vec![],
            handler: CommandHandler::System(SystemCommand::ToggleSettings),
        });

        let filtered = get_filtered_commands(&commands, "test");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "Test Command");
    }
}