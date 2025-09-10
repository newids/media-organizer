use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// VSCode-style color theme definition with all semantic color tokens
/// This provides a comprehensive color system similar to VSCode's theme JSON format
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColorTheme {
    /// Theme metadata
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub is_dark: bool,
    
    /// Core color definitions - these map to VSCode's color tokens
    pub colors: ThemeColors,
    
    /// Syntax highlighting colors for code editors
    pub token_colors: Vec<TokenColor>,
    
    /// Semantic highlighting colors for language features
    pub semantic_highlighting: SemanticColors,
}

/// Comprehensive color definitions matching VSCode's color tokens
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeColors {
    // Base colors
    pub foreground: String,
    pub foreground_secondary: String,
    pub error_foreground: String,
    pub warning_foreground: String,
    pub info_foreground: String,
    pub success_foreground: String,
    
    // Background colors
    pub background: String,
    pub secondary_background: String,
    pub tertiary_background: String,
    
    // Border and separator colors
    pub border: String,
    pub border_secondary: String,
    pub focus_border: String,
    pub drop_shadow: String,
    
    // Button colors
    pub button_background: String,
    pub button_foreground: String,
    pub button_hover_background: String,
    pub button_disabled_background: String,
    pub button_disabled_foreground: String,
    
    // Input colors
    pub input_background: String,
    pub input_foreground: String,
    pub input_border: String,
    pub input_placeholder_foreground: String,
    pub input_validation_info_background: String,
    pub input_validation_info_foreground: String,
    pub input_validation_warning_background: String,
    pub input_validation_warning_foreground: String,
    pub input_validation_error_background: String,
    pub input_validation_error_foreground: String,
    
    // Dropdown colors
    pub dropdown_background: String,
    pub dropdown_foreground: String,
    pub dropdown_border: String,
    pub dropdown_list_background: String,
    
    // List colors
    pub list_background: String,
    pub list_foreground: String,
    pub list_active_selection_background: String,
    pub list_active_selection_foreground: String,
    pub list_inactive_selection_background: String,
    pub list_inactive_selection_foreground: String,
    pub list_hover_background: String,
    pub list_hover_foreground: String,
    pub list_focus_background: String,
    pub list_focus_foreground: String,
    pub list_focus_outline: String,
    pub list_drop_background: String,
    pub list_highlight_foreground: String,
    pub list_invalid_item_foreground: String,
    pub list_error_foreground: String,
    pub list_warning_foreground: String,
    pub list_filter_widget_background: String,
    pub list_filter_widget_outline: String,
    pub list_filter_widget_no_matches_outline: String,
    
    // Activity Bar colors
    pub activity_bar_background: String,
    pub activity_bar_foreground: String,
    pub activity_bar_inactive_foreground: String,
    pub activity_bar_border: String,
    pub activity_bar_drag_and_drop_border: String,
    pub activity_bar_active_background: String,
    pub activity_bar_active_foreground: String,
    pub activity_bar_active_focus_border: String,
    pub activity_bar_badge_background: String,
    pub activity_bar_badge_foreground: String,
    
    // Side Bar colors
    pub side_bar_background: String,
    pub side_bar_foreground: String,
    pub side_bar_border: String,
    pub side_bar_title_foreground: String,
    pub side_bar_drag_and_drop_background: String,
    pub side_bar_section_header_background: String,
    pub side_bar_section_header_foreground: String,
    pub side_bar_section_header_border: String,
    
    // Minimap colors
    pub minimap_background: String,
    pub minimap_selection_highlight: String,
    pub minimap_find_match_highlight: String,
    pub minimap_selection_occurrence_highlight: String,
    
    // Editor Group & Tab colors
    pub editor_group_border: String,
    pub editor_group_drag_and_drop_background: String,
    pub editor_group_empty_background: String,
    pub editor_group_focused_empty_border: String,
    pub editor_group_header_tabs_background: String,
    pub editor_group_header_tabs_border: String,
    pub editor_group_header_no_tabs_background: String,
    
    // Tab colors
    pub tab_active_background: String,
    pub tab_active_foreground: String,
    pub tab_active_border: String,
    pub tab_active_border_top: String,
    pub tab_inactive_background: String,
    pub tab_inactive_foreground: String,
    pub tab_inactive_border: String,
    pub tab_inactive_border_top: String,
    pub tab_hover_background: String,
    pub tab_hover_foreground: String,
    pub tab_hover_border: String,
    pub tab_unfocused_active_background: String,
    pub tab_unfocused_active_foreground: String,
    pub tab_unfocused_active_border: String,
    pub tab_unfocused_active_border_top: String,
    pub tab_unfocused_inactive_background: String,
    pub tab_unfocused_inactive_foreground: String,
    pub tab_unfocused_inactive_border: String,
    pub tab_unfocused_inactive_border_top: String,
    pub tab_unfocused_hover_background: String,
    pub tab_unfocused_hover_foreground: String,
    pub tab_unfocused_hover_border: String,
    pub tab_last_pinned_border: String,
    
    // Editor colors
    pub editor_background: String,
    pub editor_foreground: String,
    pub editor_line_highlight_background: String,
    pub editor_line_highlight_border: String,
    pub editor_range_highlight_background: String,
    pub editor_range_highlight_border: String,
    pub editor_cursor_background: String,
    pub editor_cursor_foreground: String,
    pub editor_white_space_foreground: String,
    pub editor_indent_guide_background: String,
    pub editor_indent_guide_active_background: String,
    pub editor_line_number_foreground: String,
    pub editor_line_number_active_foreground: String,
    pub editor_ruler_foreground: String,
    pub editor_code_lens_foreground: String,
    pub editor_bracket_match_background: String,
    pub editor_bracket_match_border: String,
    pub editor_overview_ruler_border: String,
    pub editor_overview_ruler_find_match_foreground: String,
    pub editor_overview_ruler_range_highlight_foreground: String,
    pub editor_overview_ruler_selection_highlight_foreground: String,
    pub editor_overview_ruler_word_highlight_foreground: String,
    pub editor_overview_ruler_word_highlight_strong_foreground: String,
    pub editor_overview_ruler_modified_foreground: String,
    pub editor_overview_ruler_added_foreground: String,
    pub editor_overview_ruler_deleted_foreground: String,
    pub editor_overview_ruler_error_foreground: String,
    pub editor_overview_ruler_warning_foreground: String,
    pub editor_overview_ruler_info_foreground: String,
    pub editor_overview_ruler_bracket_match_foreground: String,
    
    // Find Widget colors
    pub editor_find_match_background: String,
    pub editor_find_match_highlight_background: String,
    pub editor_find_range_highlight_background: String,
    pub editor_find_match_border: String,
    pub editor_find_range_highlight_border: String,
    pub editor_hovered_highlight_background: String,
    pub editor_hovered_word_highlight_background: String,
    pub editor_hovered_word_highlight_border: String,
    pub editor_link_active_foreground: String,
    
    // Selection colors
    pub editor_selection_background: String,
    pub editor_selection_foreground: String,
    pub editor_inactive_selection_background: String,
    pub editor_selection_highlight_background: String,
    pub editor_selection_highlight_border: String,
    
    // Word highlight colors
    pub editor_word_highlight_background: String,
    pub editor_word_highlight_border: String,
    pub editor_word_highlight_strong_background: String,
    pub editor_word_highlight_strong_border: String,
    
    // Panel colors
    pub panel_background: String,
    pub panel_border: String,
    pub panel_drag_and_drop_border: String,
    pub panel_section_border: String,
    pub panel_section_drag_and_drop_background: String,
    pub panel_section_header_background: String,
    pub panel_section_header_foreground: String,
    pub panel_section_header_border: String,
    
    // Status Bar colors
    pub status_bar_background: String,
    pub status_bar_foreground: String,
    pub status_bar_border: String,
    pub status_bar_debugging_background: String,
    pub status_bar_debugging_foreground: String,
    pub status_bar_debugging_border: String,
    pub status_bar_no_folder_background: String,
    pub status_bar_no_folder_foreground: String,
    pub status_bar_no_folder_border: String,
    pub status_bar_item_active_background: String,
    pub status_bar_item_hover_background: String,
    pub status_bar_item_prominent_background: String,
    pub status_bar_item_prominent_hover_background: String,
    pub status_bar_item_remote_background: String,
    pub status_bar_item_remote_foreground: String,
    
    // Title Bar colors
    pub title_bar_active_background: String,
    pub title_bar_active_foreground: String,
    pub title_bar_inactive_background: String,
    pub title_bar_inactive_foreground: String,
    pub title_bar_border: String,
    
    // Menu colors
    pub menu_foreground: String,
    pub menu_background: String,
    pub menu_selection_foreground: String,
    pub menu_selection_background: String,
    pub menu_selection_border: String,
    pub menu_separator_background: String,
    pub menu_border: String,
    
    // Notification colors
    pub notifications_center_border: String,
    pub notifications_toast_border: String,
    pub notifications_foreground: String,
    pub notifications_background: String,
    pub notifications_border: String,
    pub notification_center_header_foreground: String,
    pub notification_center_header_background: String,
    pub notification_toast_background: String,
    pub notifications_links: String,
    pub notifications_center_header_border: String,
    
    // Extensions colors
    pub extension_button_prominent_foreground: String,
    pub extension_button_prominent_background: String,
    pub extension_button_prominent_hover_background: String,
    
    // Picklist colors
    pub picklist_group_foreground: String,
    pub picklist_group_border: String,
    
    // Terminal colors
    pub terminal_background: String,
    pub terminal_foreground: String,
    pub terminal_cursor_background: String,
    pub terminal_cursor_foreground: String,
    pub terminal_selection_background: String,
    pub terminal_inactive_selection_background: String,
    
    // Git decoration colors
    pub git_decoration_modified_resource_foreground: String,
    pub git_decoration_deleted_resource_foreground: String,
    pub git_decoration_untracked_resource_foreground: String,
    pub git_decoration_ignored_resource_foreground: String,
    pub git_decoration_conflicting_resource_foreground: String,
    pub git_decoration_staged_modified_resource_foreground: String,
    pub git_decoration_staged_deleted_resource_foreground: String,
    
    // Settings colors
    pub settings_header_foreground: String,
    pub settings_modified_item_indicator: String,
    pub settings_dropdown_background: String,
    pub settings_dropdown_foreground: String,
    pub settings_dropdown_border: String,
    pub settings_dropdown_list_border: String,
    pub settings_checkbox_background: String,
    pub settings_checkbox_foreground: String,
    pub settings_checkbox_border: String,
    pub settings_row_hover_background: String,
    pub settings_text_input_background: String,
    pub settings_text_input_foreground: String,
    pub settings_text_input_border: String,
    pub settings_number_input_background: String,
    pub settings_number_input_foreground: String,
    pub settings_number_input_border: String,
    
    // Progress bar colors
    pub progress_bar_background: String,
    
    // Breadcrumb colors
    pub breadcrumb_foreground: String,
    pub breadcrumb_background: String,
    pub breadcrumb_focus_foreground: String,
    pub breadcrumb_active_selection_foreground: String,
    pub breadcrumb_picker_background: String,
    
    // Scrollbar colors
    pub scrollbar_shadow: String,
    pub scrollbar_slider_background: String,
    pub scrollbar_slider_hover_background: String,
    pub scrollbar_slider_active_background: String,
    
    // Badge colors
    pub badge_background: String,
    pub badge_foreground: String,
}

/// Token color definition for syntax highlighting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenColor {
    pub name: Option<String>,
    pub scope: Vec<String>,
    pub settings: TokenSettings,
}

/// Settings for a token color
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>, // "bold", "italic", "underline", "strikethrough"
}

/// Semantic colors for language-specific highlighting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticColors {
    /// Whether semantic highlighting is enabled for this theme
    pub enabled: bool,
    
    /// Color rules for semantic tokens
    pub rules: HashMap<String, SemanticColorRule>,
}

/// Rule for semantic color highlighting
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SemanticColorRule {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub font_style: Option<String>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
}

impl ColorTheme {
    /// Create a new dark theme with VSCode Dark+ colors
    pub fn dark_plus() -> Self {
        Self {
            name: "dark-plus".to_string(),
            display_name: "Dark+ (default dark)".to_string(),
            description: Some("Dark theme based on VSCode's default dark theme".to_string()),
            is_dark: true,
            colors: ThemeColors::dark_plus(),
            token_colors: TokenColor::dark_plus_tokens(),
            semantic_highlighting: SemanticColors::default_dark(),
        }
    }
    
    /// Create a new light theme with VSCode Light+ colors
    pub fn light_plus() -> Self {
        Self {
            name: "light-plus".to_string(),
            display_name: "Light+ (default light)".to_string(),
            description: Some("Light theme based on VSCode's default light theme".to_string()),
            is_dark: false,
            colors: ThemeColors::light_plus(),
            token_colors: TokenColor::light_plus_tokens(),
            semantic_highlighting: SemanticColors::default_light(),
        }
    }
    
    /// Create a high contrast dark theme
    pub fn high_contrast_dark() -> Self {
        Self {
            name: "high-contrast-dark".to_string(),
            display_name: "Dark High Contrast".to_string(),
            description: Some("High contrast dark theme for better accessibility".to_string()),
            is_dark: true,
            colors: ThemeColors::high_contrast_dark(),
            token_colors: TokenColor::high_contrast_tokens(),
            semantic_highlighting: SemanticColors::high_contrast_dark(),
        }
    }
    
    /// Create a high contrast light theme
    pub fn high_contrast_light() -> Self {
        Self {
            name: "high-contrast-light".to_string(),
            display_name: "Light High Contrast".to_string(),
            description: Some("High contrast light theme for better accessibility".to_string()),
            is_dark: false,
            colors: ThemeColors::high_contrast_light(),
            token_colors: TokenColor::high_contrast_tokens(),
            semantic_highlighting: SemanticColors::high_contrast_light(),
        }
    }
    
    /// Get CSS custom properties from this theme
    pub fn to_css_variables(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        let colors = &self.colors;
        
        // Base variables
        vars.insert("--vscode-foreground".to_string(), colors.foreground.clone());
        vars.insert("--vscode-foreground-secondary".to_string(), colors.foreground_secondary.clone());
        vars.insert("--vscode-background".to_string(), colors.background.clone());
        vars.insert("--vscode-secondary-background".to_string(), colors.secondary_background.clone());
        vars.insert("--vscode-tertiary-background".to_string(), colors.tertiary_background.clone());
        vars.insert("--vscode-border".to_string(), colors.border.clone());
        vars.insert("--vscode-border-secondary".to_string(), colors.border_secondary.clone());
        vars.insert("--vscode-focus-border".to_string(), colors.focus_border.clone());
        
        // Text colors
        vars.insert("--vscode-text-primary".to_string(), colors.foreground.clone());
        vars.insert("--vscode-text-secondary".to_string(), colors.foreground_secondary.clone());
        vars.insert("--vscode-text-error".to_string(), colors.error_foreground.clone());
        vars.insert("--vscode-text-warning".to_string(), colors.warning_foreground.clone());
        vars.insert("--vscode-text-info".to_string(), colors.info_foreground.clone());
        vars.insert("--vscode-text-success".to_string(), colors.success_foreground.clone());
        
        // Button colors
        vars.insert("--vscode-button-background".to_string(), colors.button_background.clone());
        vars.insert("--vscode-button-foreground".to_string(), colors.button_foreground.clone());
        vars.insert("--vscode-button-hover-background".to_string(), colors.button_hover_background.clone());
        
        // Input colors
        vars.insert("--vscode-input-background".to_string(), colors.input_background.clone());
        vars.insert("--vscode-input-foreground".to_string(), colors.input_foreground.clone());
        vars.insert("--vscode-input-border".to_string(), colors.input_border.clone());
        vars.insert("--vscode-input-placeholder-foreground".to_string(), colors.input_placeholder_foreground.clone());
        
        // List colors
        vars.insert("--vscode-list-background".to_string(), colors.list_background.clone());
        vars.insert("--vscode-list-foreground".to_string(), colors.list_foreground.clone());
        vars.insert("--vscode-list-active-selection-background".to_string(), colors.list_active_selection_background.clone());
        vars.insert("--vscode-list-active-selection-foreground".to_string(), colors.list_active_selection_foreground.clone());
        vars.insert("--vscode-list-hover-background".to_string(), colors.list_hover_background.clone());
        vars.insert("--vscode-list-hover-foreground".to_string(), colors.list_hover_foreground.clone());
        
        // Activity Bar colors
        vars.insert("--vscode-activity-bar-background".to_string(), colors.activity_bar_background.clone());
        vars.insert("--vscode-activity-bar-foreground".to_string(), colors.activity_bar_foreground.clone());
        vars.insert("--vscode-activity-bar-inactive-foreground".to_string(), colors.activity_bar_inactive_foreground.clone());
        vars.insert("--vscode-activity-bar-active-background".to_string(), colors.activity_bar_active_background.clone());
        vars.insert("--vscode-activity-bar-active-foreground".to_string(), colors.activity_bar_active_foreground.clone());
        vars.insert("--vscode-activity-bar-badge-background".to_string(), colors.activity_bar_badge_background.clone());
        vars.insert("--vscode-activity-bar-badge-foreground".to_string(), colors.activity_bar_badge_foreground.clone());
        
        // Side Bar colors
        vars.insert("--vscode-side-bar-background".to_string(), colors.side_bar_background.clone());
        vars.insert("--vscode-side-bar-foreground".to_string(), colors.side_bar_foreground.clone());
        vars.insert("--vscode-side-bar-border".to_string(), colors.side_bar_border.clone());
        vars.insert("--vscode-side-bar-title-foreground".to_string(), colors.side_bar_title_foreground.clone());
        
        // Editor Group & Tab colors
        vars.insert("--vscode-editor-group-border".to_string(), colors.editor_group_border.clone());
        vars.insert("--vscode-tab-active-background".to_string(), colors.tab_active_background.clone());
        vars.insert("--vscode-tab-active-foreground".to_string(), colors.tab_active_foreground.clone());
        vars.insert("--vscode-tab-active-border".to_string(), colors.tab_active_border.clone());
        vars.insert("--vscode-tab-inactive-background".to_string(), colors.tab_inactive_background.clone());
        vars.insert("--vscode-tab-inactive-foreground".to_string(), colors.tab_inactive_foreground.clone());
        vars.insert("--vscode-tab-hover-background".to_string(), colors.tab_hover_background.clone());
        
        // Editor colors
        vars.insert("--vscode-editor-background".to_string(), colors.editor_background.clone());
        vars.insert("--vscode-editor-foreground".to_string(), colors.editor_foreground.clone());
        vars.insert("--vscode-editor-line-highlight-background".to_string(), colors.editor_line_highlight_background.clone());
        vars.insert("--vscode-editor-selection-background".to_string(), colors.editor_selection_background.clone());
        vars.insert("--vscode-editor-cursor-foreground".to_string(), colors.editor_cursor_foreground.clone());
        
        // Panel colors
        vars.insert("--vscode-panel-background".to_string(), colors.panel_background.clone());
        vars.insert("--vscode-panel-border".to_string(), colors.panel_border.clone());
        
        // Status Bar colors
        vars.insert("--vscode-status-bar-background".to_string(), colors.status_bar_background.clone());
        vars.insert("--vscode-status-bar-foreground".to_string(), colors.status_bar_foreground.clone());
        vars.insert("--vscode-status-bar-border".to_string(), colors.status_bar_border.clone());
        
        // Menu colors
        vars.insert("--vscode-menu-background".to_string(), colors.menu_background.clone());
        vars.insert("--vscode-menu-foreground".to_string(), colors.menu_foreground.clone());
        vars.insert("--vscode-menu-selection-background".to_string(), colors.menu_selection_background.clone());
        vars.insert("--vscode-menu-selection-foreground".to_string(), colors.menu_selection_foreground.clone());
        
        // Typography
        vars.insert("--vscode-font-family".to_string(), "system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif".to_string());
        vars.insert("--vscode-font-size-normal".to_string(), "13px".to_string());
        vars.insert("--vscode-font-size-small".to_string(), "11px".to_string());
        vars.insert("--vscode-font-size-large".to_string(), "16px".to_string());
        vars.insert("--vscode-line-height".to_string(), "1.4".to_string());
        
        // Code font
        vars.insert("--vscode-editor-font-family".to_string(), "'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace".to_string());
        vars.insert("--vscode-editor-font-size".to_string(), "12px".to_string());
        vars.insert("--vscode-editor-line-height".to_string(), "1.5".to_string());
        
        // Scrollbar colors
        vars.insert("--vscode-scrollbar-slider-background".to_string(), colors.scrollbar_slider_background.clone());
        vars.insert("--vscode-scrollbar-slider-hover-background".to_string(), colors.scrollbar_slider_hover_background.clone());
        vars.insert("--vscode-scrollbar-slider-active-background".to_string(), colors.scrollbar_slider_active_background.clone());
        
        vars
    }
    
    /// Get a color by semantic name
    pub fn get_color(&self, name: &str) -> Option<&String> {
        match name {
            "foreground" => Some(&self.colors.foreground),
            "background" => Some(&self.colors.background),
            "border" => Some(&self.colors.border),
            "button.background" => Some(&self.colors.button_background),
            "button.foreground" => Some(&self.colors.button_foreground),
            "input.background" => Some(&self.colors.input_background),
            "list.activeSelectionBackground" => Some(&self.colors.list_active_selection_background),
            "activityBar.background" => Some(&self.colors.activity_bar_background),
            "sideBar.background" => Some(&self.colors.side_bar_background),
            "editor.background" => Some(&self.colors.editor_background),
            "panel.background" => Some(&self.colors.panel_background),
            "statusBar.background" => Some(&self.colors.status_bar_background),
            _ => None,
        }
    }
}