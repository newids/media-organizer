use super::color_theme::ColorTheme;
use crate::state::{Theme, SettingsState, save_settings_debounced};
use crate::performance::rendering_optimizations::{ThemeOptimizer, RenderingProfiler};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

/// Enhanced theme manager that provides VSCode-style color theme functionality
pub struct VsCodeThemeManager {
    /// Available color themes
    available_themes: HashMap<String, ColorTheme>,
    /// Currently active theme
    current_theme: Option<ColorTheme>,
    /// Theme optimizer for performance
    optimizer: Option<ThemeOptimizer>,
    /// Current simple theme selection (for backwards compatibility)
    current_simple_theme: Theme,
}

impl Default for VsCodeThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VsCodeThemeManager {
    /// Create a new theme manager with built-in themes
    pub fn new() -> Self {
        let mut manager = Self {
            available_themes: HashMap::new(),
            current_theme: None,
            optimizer: None,
            current_simple_theme: Theme::Dark,
        };
        
        // Initialize optimizer
        let profiler = Arc::new(Mutex::new(RenderingProfiler::new()));
        manager.optimizer = Some(ThemeOptimizer::new(profiler));
        
        // Register built-in themes
        manager.register_builtin_themes();
        
        // Set default theme
        manager.set_theme_by_name("dark-plus").unwrap_or_else(|_| {
            tracing::warn!("Failed to set default dark-plus theme");
        });
        
        manager
    }
    
    /// Register all built-in VSCode-style themes
    fn register_builtin_themes(&mut self) {
        let themes = vec![
            ColorTheme::dark_plus(),
            ColorTheme::light_plus(),
            ColorTheme::high_contrast_dark(),
            ColorTheme::high_contrast_light(),
        ];
        
        for theme in themes {
            self.available_themes.insert(theme.name.clone(), theme);
        }
        
        tracing::info!("Registered {} built-in themes", self.available_themes.len());
    }
    
    /// Get all available theme names
    pub fn get_available_theme_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.available_themes.keys().cloned().collect();
        names.sort();
        names
    }
    
    /// Get theme information by name
    pub fn get_theme_info(&self, name: &str) -> Option<(&str, &str, bool)> {
        self.available_themes.get(name).map(|theme| {
            (theme.display_name.as_str(), theme.description.as_deref().unwrap_or(""), theme.is_dark)
        })
    }
    
    /// Set theme by name
    pub fn set_theme_by_name(&mut self, name: &str) -> Result<(), String> {
        if let Some(theme) = self.available_themes.get(name).cloned() {
            self.current_theme = Some(theme.clone());
            
            // Update simple theme for backwards compatibility
            self.current_simple_theme = if theme.is_dark {
                if name.contains("high-contrast") {
                    Theme::HighContrast
                } else {
                    Theme::Dark
                }
            } else {
                if name.contains("high-contrast") {
                    Theme::HighContrast
                } else {
                    Theme::Light
                }
            };
            
            // Apply theme to the UI
            self.apply_current_theme();
            
            tracing::info!("Switched to theme: {} ({})", theme.display_name, name);
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }
    
    /// Set theme from simple Theme enum (backwards compatibility)
    pub fn set_theme_from_simple(&mut self, simple_theme: &Theme) -> Result<(), String> {
        let theme_name = match simple_theme {
            Theme::Dark => "dark-plus",
            Theme::Light => "light-plus", 
            Theme::HighContrast => {
                // Choose high contrast based on system preference or current theme
                if self.current_theme.as_ref().map_or(true, |t| t.is_dark) {
                    "high-contrast-dark"
                } else {
                    "high-contrast-light"
                }
            },
            Theme::Auto => {
                // Detect system preference and choose appropriate theme
                let is_dark = Self::detect_system_theme();
                if is_dark { "dark-plus" } else { "light-plus" }
            }
        };
        
        self.current_simple_theme = simple_theme.clone();
        self.set_theme_by_name(theme_name)
    }
    
    /// Get current theme
    pub fn get_current_theme(&self) -> Option<&ColorTheme> {
        self.current_theme.as_ref()
    }
    
    /// Get current simple theme (backwards compatibility)
    pub fn get_current_simple_theme(&self) -> &Theme {
        &self.current_simple_theme
    }
    
    /// Apply the current theme to the UI
    pub fn apply_current_theme(&self) {
        if let Some(theme) = &self.current_theme {
            let css_vars = theme.to_css_variables();
            self.apply_css_variables(&css_vars);
            
            // Also apply to the legacy theme system
            self.apply_legacy_theme_attributes(&self.current_simple_theme);
            
            tracing::debug!("Applied theme: {} with {} CSS variables", theme.display_name, css_vars.len());
        }
    }
    
    /// Apply CSS variables to the document
    fn apply_css_variables(&self, variables: &HashMap<String, String>) {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        if let Ok(html_element) = html.dyn_into::<web_sys::HtmlElement>() {
                            let style = html_element.style();
                            
                            for (key, value) in variables {
                                if let Err(e) = style.set_property(key, value) {
                                    tracing::warn!("Failed to set CSS property {}: {:?}", key, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            // For desktop, log the variables that would be applied
            tracing::debug!("Would apply {} CSS variables in desktop mode", variables.len());
            for (key, value) in variables.iter().take(5) { // Log first 5 for debugging
                tracing::debug!("CSS Var: {} = {}", key, value);
            }
        }
    }
    
    /// Apply legacy theme attributes for backwards compatibility
    fn apply_legacy_theme_attributes(&self, theme: &Theme) {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        let theme_attr = match theme {
                            Theme::Dark => "dark",
                            Theme::Light => "light", 
                            Theme::HighContrast => "high-contrast",
                            Theme::Auto => {
                                // Remove attribute to let CSS media queries handle it
                                let _ = html.remove_attribute("data-theme");
                                return;
                            }
                        };
                        
                        let _ = html.set_attribute("data-theme", theme_attr);
                    }
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            tracing::debug!("Would set legacy theme attribute: {:?}", theme);
        }
    }
    
    /// Register a custom theme
    pub fn register_custom_theme(&mut self, theme: ColorTheme) {
        let name = theme.name.clone();
        self.available_themes.insert(name.clone(), theme);
        tracing::info!("Registered custom theme: {}", name);
    }
    
    /// Load theme from JSON string
    pub fn load_theme_from_json(&mut self, json: &str) -> Result<String, String> {
        let theme: ColorTheme = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse theme JSON: {}", e))?;
        
        let name = theme.name.clone();
        self.register_custom_theme(theme);
        Ok(name)
    }
    
    /// Export current theme to JSON string
    pub fn export_current_theme_to_json(&self) -> Result<String, String> {
        if let Some(theme) = &self.current_theme {
            serde_json::to_string_pretty(theme)
                .map_err(|e| format!("Failed to serialize theme: {}", e))
        } else {
            Err("No current theme to export".to_string())
        }
    }
    
    /// Get a specific color from the current theme
    pub fn get_current_color(&self, color_name: &str) -> Option<String> {
        self.current_theme.as_ref()?.get_color(color_name).cloned()
    }
    
    /// Get all CSS variables for the current theme
    pub fn get_current_css_variables(&self) -> HashMap<String, String> {
        if let Some(theme) = &self.current_theme {
            theme.to_css_variables()
        } else {
            HashMap::new()
        }
    }
    
    /// Toggle between dark and light themes
    pub fn toggle_dark_light(&mut self) -> Result<(), String> {
        let current_is_dark = self.current_theme.as_ref().map_or(true, |t| t.is_dark);
        
        if current_is_dark {
            self.set_theme_by_name("light-plus")
        } else {
            self.set_theme_by_name("dark-plus")
        }
    }
    
    /// Cycle through main themes (Dark+ → Light+ → High Contrast Dark → High Contrast Light → Dark+)
    pub fn cycle_themes(&mut self) -> Result<(), String> {
        let next_theme = if let Some(current) = &self.current_theme {
            match current.name.as_str() {
                "dark-plus" => "light-plus",
                "light-plus" => "high-contrast-dark",
                "high-contrast-dark" => "high-contrast-light",
                "high-contrast-light" => "dark-plus",
                _ => "dark-plus", // Default fallback
            }
        } else {
            "dark-plus"
        };
        
        self.set_theme_by_name(next_theme)
    }
    
    /// Detect system theme preference (true = dark, false = light)
    pub fn detect_system_theme() -> bool {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(media_query)) = window.match_media("(prefers-color-scheme: dark)") {
                    return media_query.matches();
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            return Self::detect_desktop_system_theme();
        }
        
        #[cfg(feature = "web")]
        {
            true // Default to dark if detection fails
        }
    }
    
    /// Desktop system theme detection
    #[cfg(not(feature = "web"))]
    fn detect_desktop_system_theme() -> bool {
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = std::process::Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                if output.status.success() {
                    let style = String::from_utf8_lossy(&output.stdout);
                    return style.trim().eq_ignore_ascii_case("dark");
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("reg")
                .args(&[
                    "query", 
                    "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                    "/v", "AppsUseLightTheme"
                ])
                .output()
            {
                if output.status.success() {
                    let registry_output = String::from_utf8_lossy(&output.stdout);
                    let is_light = registry_output.contains("0x1");
                    return !is_light;
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Check GTK theme
            if let Ok(gtk_theme) = std::env::var("GTK_THEME") {
                return gtk_theme.to_lowercase().contains("dark");
            }
            
            // Check gsettings for GNOME
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
            {
                if output.status.success() {
                    let theme_name = String::from_utf8_lossy(&output.stdout);
                    return theme_name.to_lowercase().contains("dark");
                }
            }
        }
        
        true // Default to dark
    }
    
    /// Update theme manager with settings (backwards compatibility)
    pub fn sync_with_settings(&mut self, settings: &SettingsState) -> Result<(), String> {
        // Apply custom CSS variables if any
        if !settings.custom_css_variables.is_empty() {
            self.apply_css_variables(&settings.custom_css_variables);
        }
        
        // Sync with the simple theme setting
        self.set_theme_from_simple(&settings.theme)
    }
    
    /// Apply auto theme based on system preference
    pub fn apply_auto_theme(&mut self) -> Result<(), String> {
        let is_dark = Self::detect_system_theme();
        let theme_name = if is_dark { "dark-plus" } else { "light-plus" };
        self.current_simple_theme = Theme::Auto;
        self.set_theme_by_name(theme_name)
    }
    
    /// Get theme suggestions based on current theme
    pub fn get_theme_suggestions(&self) -> Vec<(String, String)> {
        let mut suggestions = Vec::new();
        
        if let Some(current) = &self.current_theme {
            // Suggest related themes
            if current.is_dark {
                suggestions.push(("light-plus".to_string(), "Switch to Light theme".to_string()));
                suggestions.push(("high-contrast-dark".to_string(), "High contrast dark".to_string()));
            } else {
                suggestions.push(("dark-plus".to_string(), "Switch to Dark theme".to_string()));
                suggestions.push(("high-contrast-light".to_string(), "High contrast light".to_string()));
            }
        }
        
        suggestions
    }
    
    /// Check if a theme supports semantic highlighting
    pub fn current_theme_supports_semantic_highlighting(&self) -> bool {
        self.current_theme.as_ref()
            .map_or(false, |theme| theme.semantic_highlighting.enabled)
    }
    
    /// Get semantic color rules for current theme
    pub fn get_current_semantic_colors(&self) -> HashMap<String, String> {
        let mut colors = HashMap::new();
        
        if let Some(theme) = &self.current_theme {
            for (token, rule) in &theme.semantic_highlighting.rules {
                if let Some(color) = &rule.foreground {
                    colors.insert(format!("--vscode-semantic-{}", token), color.clone());
                }
            }
        }
        
        colors
    }
    
    /// Validate theme integrity
    pub fn validate_current_theme(&self) -> Vec<String> {
        let mut issues = Vec::new();
        
        if let Some(theme) = &self.current_theme {
            // Check for required colors
            let required_colors = [
                "foreground", "background", "border", "button_background", 
                "input_background", "list_background", "editor_background"
            ];
            
            for &required in &required_colors {
                if theme.get_color(required).is_none() {
                    issues.push(format!("Missing required color: {}", required));
                }
            }
            
            // Validate color format (basic hex check)
            for (key, value) in theme.to_css_variables() {
                if value.starts_with('#') && (value.len() != 7 && value.len() != 9) {
                    issues.push(format!("Invalid color format for {}: {}", key, value));
                }
            }
        } else {
            issues.push("No current theme set".to_string());
        }
        
        issues
    }
}

/// Dioxus hook for VSCode-style theme management
pub fn use_vscode_theme_manager() -> Signal<VsCodeThemeManager> {
    use_signal(|| VsCodeThemeManager::new())
}

/// Enhanced theme selector component with VSCode-style theming
#[component] 
pub fn VsCodeThemeSelector(
    theme_manager: Signal<VsCodeThemeManager>,
    on_theme_change: EventHandler<String>,
) -> Element {
    let manager = theme_manager.read();
    let available_themes = manager.get_available_theme_names();
    let current_theme_name = manager.get_current_theme()
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "dark-plus".to_string());
    
    rsx! {
        div {
            class: "vscode-theme-selector",
            style: "
                display: flex;
                flex-direction: column;
                gap: 12px;
                padding: 16px;
                background-color: var(--vscode-secondary-background);
                border: 1px solid var(--vscode-border);
                border-radius: 6px;
                font-family: var(--vscode-font-family);
            ",
            
            div {
                style: "display: flex; gap: 8px; align-items: center;",
                
                label {
                    style: "
                        color: var(--vscode-text-primary);
                        font-size: var(--vscode-font-size-normal);
                        font-weight: 500;
                        min-width: 80px;
                    ",
                    "Color Theme:"
                }
                
                select {
                    value: "{current_theme_name}",
                    style: "
                        flex: 1;
                        background-color: var(--vscode-input-background);
                        color: var(--vscode-input-foreground);
                        border: 1px solid var(--vscode-input-border);
                        border-radius: 4px;
                        padding: 8px 12px;
                        font-size: var(--vscode-font-size-normal);
                        font-family: var(--vscode-font-family);
                        outline: none;
                        cursor: pointer;
                        transition: border-color 0.2s ease;
                    ",
                    onchange: move |evt| {
                        on_theme_change.call(evt.value());
                    },
                    onfocus: |_| {},  // Could add focus styling
                    
                    for theme_name in available_themes {
                        option {
                            value: "{theme_name}",
                            selected: theme_name == current_theme_name,
                            
                            {
                                if let Some((display_name, _, is_dark)) = manager.get_theme_info(&theme_name) {
                                    let icon = if is_dark { "🌙" } else { "☀️" };
                                    format!("{} {}", icon, display_name)
                                } else {
                                    theme_name.clone()
                                }
                            }
                        }
                    }
                }
            }
            
            // Theme preview
            if let Some(current_theme) = manager.get_current_theme() {
                div {
                    style: "
                        background-color: var(--vscode-editor-background);
                        border: 1px solid var(--vscode-border);
                        border-radius: 4px;
                        padding: 12px;
                        font-size: var(--vscode-font-size-small);
                    ",
                    
                    div {
                        style: "
                            color: var(--vscode-text-secondary);
                            margin-bottom: 8px;
                            font-weight: 500;
                        ",
                        "{current_theme.display_name}"
                    }
                    
                    if let Some(description) = &current_theme.description {
                        div {
                            style: "
                                color: var(--vscode-text-secondary);
                                font-style: italic;
                                margin-bottom: 8px;
                            ",
                            "{description}"
                        }
                    }
                    
                    div {
                        style: "display: flex; gap: 8px; flex-wrap: wrap;",
                        
                        div {
                            style: "
                                background-color: var(--vscode-activity-bar-background);
                                color: var(--vscode-activity-bar-foreground);
                                padding: 4px 8px;
                                border-radius: 3px;
                                font-size: 10px;
                                font-weight: 500;
                            ",
                            "Activity Bar"
                        }
                        
                        div {
                            style: "
                                background-color: var(--vscode-side-bar-background);
                                color: var(--vscode-side-bar-foreground);
                                padding: 4px 8px;
                                border-radius: 3px;
                                font-size: 10px;
                                font-weight: 500;
                            ",
                            "Sidebar"
                        }
                        
                        div {
                            style: "
                                background-color: var(--vscode-status-bar-background);
                                color: var(--vscode-status-bar-foreground);
                                padding: 4px 8px;
                                border-radius: 3px;
                                font-size: 10px;
                                font-weight: 500;
                            ",
                            "Status Bar"
                        }
                    }
                }
            }
            
            // Quick actions
            div {
                style: "display: flex; gap: 8px; margin-top: 8px;",
                
                button {
                    style: "
                        background-color: var(--vscode-button-background);
                        color: var(--vscode-button-foreground);
                        border: none;
                        border-radius: 4px;
                        padding: 6px 12px;
                        font-size: var(--vscode-font-size-small);
                        font-family: var(--vscode-font-family);
                        cursor: pointer;
                        transition: background-color 0.2s ease;
                    ",
                    onclick: move |_| {
                        // Toggle between dark and light
                        let mut manager = theme_manager.write();
                        if let Ok(()) = manager.toggle_dark_light() {
                            if let Some(new_theme) = manager.get_current_theme() {
                                on_theme_change.call(new_theme.name.clone());
                            }
                        }
                    },
                    "Toggle Dark/Light"
                }
                
                button {
                    style: "
                        background-color: var(--vscode-secondary-background);
                        color: var(--vscode-text-primary);
                        border: 1px solid var(--vscode-border);
                        border-radius: 4px;
                        padding: 6px 12px;
                        font-size: var(--vscode-font-size-small);
                        font-family: var(--vscode-font-family);
                        cursor: pointer;
                        transition: background-color 0.2s ease;
                    ",
                    onclick: move |_| {
                        // Cycle through all themes
                        let mut manager = theme_manager.write();
                        if let Ok(()) = manager.cycle_themes() {
                            if let Some(new_theme) = manager.get_current_theme() {
                                on_theme_change.call(new_theme.name.clone());
                            }
                        }
                    },
                    "Cycle Themes"
                }
            }
        }
    }
}