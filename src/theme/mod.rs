use crate::state::{Theme, SettingsState, save_settings_debounced};
use dioxus::prelude::*;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

/// Theme management utilities for the MediaOrganizer application
pub struct ThemeManager;

impl ThemeManager {
    /// Apply a theme to the document root element
    pub fn apply_theme(theme: &Theme) {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        match theme {
                            Theme::Dark => {
                                let _ = html.set_attribute("data-theme", "dark");
                            }
                            Theme::Light => {
                                let _ = html.set_attribute("data-theme", "light");
                            }
                            Theme::Auto => {
                                // Detect system preference and apply appropriate theme
                                let system_theme = Self::detect_system_theme();
                                let theme_value = if system_theme { "dark" } else { "light" };
                                let _ = html.set_attribute("data-theme", theme_value);
                            }
                        }
                        tracing::debug!("Applied theme: {:?}", theme);
                    }
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            // For desktop, we'll handle theme application through CSS custom properties
            // The styling is already in the CSS with :root and [data-theme] selectors
            tracing::debug!("Applied theme: {:?} (desktop mode)", theme);
        }
    }

    /// Detect system theme preference (true = dark, false = light)
    pub fn detect_system_theme() -> bool {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                // Check if the browser supports matchMedia
                if let Ok(Some(media_query)) = window.match_media("(prefers-color-scheme: dark)") {
                    return media_query.matches();
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            // For desktop, try to detect system theme preference
            // This is a simplified approach - in a full implementation you'd
            // use platform-specific APIs to detect the system theme
            if cfg!(target_os = "macos") {
                // On macOS, we could use NSUserDefaults or similar
                // For now, default to dark
                return true;
            } else if cfg!(target_os = "windows") {
                // On Windows, we could check the registry
                // For now, default to dark
                return true;
            } else {
                // On Linux, we could check various desktop environment settings
                // For now, default to dark
                return true;
            }
        }
        
        // Default to dark theme if we can't detect
        true
    }

    /// Get the effective theme (resolving Auto to the actual theme)
    pub fn get_effective_theme(theme: &Theme) -> Theme {
        match theme {
            Theme::Auto => {
                if Self::detect_system_theme() {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            other => other.clone(),
        }
    }

    /// Get theme display name for UI
    pub fn get_theme_display_name(theme: &Theme) -> &'static str {
        match theme {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::Auto => "Auto",
        }
    }

    /// Get all available themes
    pub fn get_available_themes() -> Vec<Theme> {
        vec![Theme::Dark, Theme::Light, Theme::Auto]
    }

    /// Toggle between themes (Dark ↔ Light, preserving Auto preference)
    pub fn toggle_theme(current_theme: &Theme) -> Theme {
        match current_theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
            Theme::Auto => {
                // When in Auto mode, toggle to the opposite of system preference
                if Self::detect_system_theme() {
                    Theme::Light
                } else {
                    Theme::Dark
                }
            }
        }
    }

    /// Apply CSS custom properties dynamically (for advanced theming)
    pub fn apply_custom_css_variables(variables: &std::collections::HashMap<String, String>) {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(html) = document.document_element() {
                        if let Ok(html_element) = html.dyn_into::<web_sys::HtmlElement>() {
                            let style = html_element.style();
                            
                            for (key, value) in variables {
                                if key.starts_with("--") {
                                    if let Err(e) = style.set_property(key, value) {
                                        tracing::warn!("Failed to set CSS property {}: {:?}", key, e);
                                    }
                                }
                            }
                            
                            tracing::debug!("Applied {} custom CSS variables", variables.len());
                        }
                    }
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            // For desktop, we can't directly apply CSS variables via web APIs
            // In a production app, you might want to generate a CSS string and inject it
            tracing::debug!("Custom CSS variables not directly supported in desktop mode: {} variables", variables.len());
        }
    }

    /// Initialize theme system with saved settings
    pub fn initialize_with_settings(settings: &SettingsState) {
        Self::apply_theme(&settings.theme);
        if !settings.custom_css_variables.is_empty() {
            Self::apply_custom_css_variables(&settings.custom_css_variables);
        }
    }
}

/// Hook for theme management in Dioxus components
pub fn use_theme_manager() -> Signal<ThemeManagerState> {
    use_signal(|| ThemeManagerState {
        current_theme: Theme::default(),
        is_applying: false,
        system_theme_listener: None,
    })
}

/// State for the theme manager hook
#[derive(Clone)]
pub struct ThemeManagerState {
    pub current_theme: Theme,
    pub is_applying: bool,
    #[cfg(feature = "web")]
    pub system_theme_listener: Option<js_sys::Function>,
    #[cfg(not(feature = "web"))]
    pub system_theme_listener: Option<()>,
}

impl ThemeManagerState {
    /// Update theme and persist to settings
    pub fn set_theme(&mut self, new_theme: Theme, settings: &mut SettingsState) {
        self.is_applying = true;
        self.current_theme = new_theme.clone();
        
        // Update settings
        settings.theme = new_theme.clone();
        
        // Apply theme to UI
        ThemeManager::apply_theme(&new_theme);
        
        // Save to persistence
        save_settings_debounced(settings.clone());
        
        self.is_applying = false;
        
        tracing::info!("Theme changed to: {:?}", new_theme);
    }

    /// Toggle theme
    pub fn toggle_theme(&mut self, settings: &mut SettingsState) {
        let new_theme = ThemeManager::toggle_theme(&self.current_theme);
        self.set_theme(new_theme, settings);
    }

    /// Get effective theme (resolving Auto to actual theme)
    pub fn get_effective_theme(&self) -> Theme {
        ThemeManager::get_effective_theme(&self.current_theme)
    }

    /// Setup system theme preference listener for Auto mode
    pub fn setup_system_theme_listener(&mut self) {
        // This would be implemented for production to listen for system theme changes
        // For now, we'll just detect on initialization
        tracing::debug!("System theme listener setup (placeholder)");
    }
}

/// Component for theme selection UI
#[component]
pub fn ThemeSelector(
    current_theme: Theme,
    on_theme_change: EventHandler<Theme>,
) -> Element {
    rsx! {
        div {
            class: "theme-selector",
            style: "display: flex; gap: 8px; align-items: center;",
            
            label {
                style: "color: var(--vscode-text-primary); font-size: var(--vscode-font-size-normal);",
                "Theme:"
            }
            
            select {
                value: "{current_theme.as_str()}",
                style: "
                    background-color: var(--vscode-secondary-background);
                    color: var(--vscode-text-primary);
                    border: 1px solid var(--vscode-border);
                    border-radius: 3px;
                    padding: 4px 8px;
                    font-size: var(--vscode-font-size-normal);
                    font-family: var(--vscode-font-family);
                ",
                onchange: move |evt| {
                    let theme = Theme::from_str(&evt.value());
                    on_theme_change.call(theme);
                },
                
                for theme in ThemeManager::get_available_themes() {
                    option {
                        value: "{theme.as_str()}",
                        selected: theme == current_theme,
                        "{ThemeManager::get_theme_display_name(&theme)}"
                    }
                }
            }
        }
    }
}