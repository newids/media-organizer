use crate::state::{Theme, SettingsState, save_settings_debounced};
use crate::performance::rendering_optimizations::{ThemeOptimizer, RenderingProfiler};
use dioxus::prelude::*;

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

/// Theme management utilities for the MediaOrganizer application
pub struct ThemeManager {
    optimizer: Option<ThemeOptimizer>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

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
                                tracing::debug!("Applied explicit dark theme");
                            }
                            Theme::Light => {
                                let _ = html.set_attribute("data-theme", "light");
                                tracing::debug!("Applied explicit light theme");
                            }
                            Theme::HighContrast => {
                                let _ = html.set_attribute("data-theme", "high-contrast");
                                tracing::debug!("Applied high contrast theme");
                            }
                            Theme::Auto => {
                                // Remove data-theme attribute to let CSS media queries handle it
                                let _ = html.remove_attribute("data-theme");
                                tracing::debug!("Applied auto theme (removed data-theme, using CSS media queries)");
                                
                                // Also log what the system preference actually is
                                let system_theme = Self::detect_system_theme();
                                tracing::info!("Auto theme active, system preference: {}", 
                                             if system_theme { "dark" } else { "light" });
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(not(feature = "web"))]
        {
            // For desktop, we need to simulate the CSS media query behavior
            // since we can't rely on browser media queries
            match theme {
                Theme::Dark => {
                    tracing::debug!("Applied explicit dark theme (desktop)");
                    // In a full desktop implementation, you'd set CSS variables here
                }
                Theme::Light => {
                    tracing::debug!("Applied explicit light theme (desktop)");
                    // In a full desktop implementation, you'd set CSS variables here
                }
                Theme::HighContrast => {
                    tracing::debug!("Applied high contrast theme (desktop)");
                    // In a full desktop implementation, you'd set high contrast CSS variables here
                }
                Theme::Auto => {
                    let system_theme = Self::detect_system_theme();
                    tracing::info!("Auto theme active (desktop), detected system preference: {}", 
                                 if system_theme { "dark" } else { "light" });
                    // In a full implementation, you'd apply the detected theme's CSS variables
                }
            }
        }
    }

    /// Create a new ThemeManager instance with optimizer
    pub fn new() -> Self {
        use std::sync::{Arc, Mutex};
        
        let profiler = Arc::new(Mutex::new(RenderingProfiler::new()));
        Self {
            optimizer: Some(ThemeOptimizer::new(profiler)),
        }
    }

    /// Detect system theme preference (true = dark, false = light)
    /// Uses cached result if available to reduce system calls
    pub fn detect_system_theme() -> bool {
        // Static optimizer for global theme detection
        use std::sync::{Arc, Mutex};
        use std::sync::OnceLock;
        
        static GLOBAL_THEME_OPTIMIZER: OnceLock<Arc<Mutex<ThemeOptimizer>>> = OnceLock::new();
        let optimizer = GLOBAL_THEME_OPTIMIZER.get_or_init(|| {
                let profiler = Arc::new(Mutex::new(RenderingProfiler::new()));
            Arc::new(Mutex::new(ThemeOptimizer::new(profiler)))
        });

        if let Ok(mut opt) = optimizer.lock() {
            if let Some(cached_theme) = opt.get_optimized_theme() {
                tracing::debug!("Using cached theme detection: {}", if cached_theme == "dark" { "dark" } else { "light" });
                return cached_theme == "dark";
            }
        }

        // Fall back to actual detection if cache miss
        Self::detect_system_theme_raw()
    }

    /// Raw system theme detection without caching
    fn detect_system_theme_raw() -> bool {
        #[cfg(feature = "web")]
        {
            if let Some(window) = web_sys::window() {
                // Check if the browser supports matchMedia for prefers-color-scheme
                if let Ok(Some(media_query)) = window.match_media("(prefers-color-scheme: dark)") {
                    tracing::debug!("Browser system theme preference: {}", if media_query.matches() { "dark" } else { "light" });
                    return media_query.matches();
                }
            }
            tracing::warn!("Could not detect system theme preference in browser, defaulting to dark");
        }
        
        #[cfg(not(feature = "web"))]
        {
            // Enhanced desktop system theme detection
            let detected_theme = Self::detect_desktop_system_theme();
            tracing::info!("Desktop system theme detected: {}", if detected_theme { "dark" } else { "light" });
            detected_theme
        }
        
        #[cfg(feature = "web")]
        {
            // Default to dark theme if detection fails on web
            tracing::debug!("System theme detection failed, defaulting to dark");
            true
        }
    }
    
    /// Enhanced desktop system theme detection
    #[cfg(not(feature = "web"))]
    fn detect_desktop_system_theme() -> bool {
        // Try multiple detection methods for desktop platforms
        
        #[cfg(target_os = "macos")]
        {
            // On macOS, check system appearance using command line tool
            if let Ok(output) = std::process::Command::new("defaults")
                .args(&["read", "-g", "AppleInterfaceStyle"])
                .output()
            {
                if output.status.success() {
                    let style = String::from_utf8_lossy(&output.stdout);
                    let is_dark = style.trim().eq_ignore_ascii_case("dark");
                    tracing::debug!("macOS AppleInterfaceStyle: '{}' -> {}", style.trim(), if is_dark { "dark" } else { "light" });
                    return is_dark;
                }
            }
            tracing::debug!("macOS system theme detection failed, defaulting to dark");
            return true; // Default to dark on macOS
        }
        
        #[cfg(target_os = "windows")]
        {
            // On Windows, check the registry for app theme preference
            // This is a simplified approach - production code might use Windows APIs
            use std::process::Command;
            
            if let Ok(output) = Command::new("reg")
                .args(&[
                    "query", 
                    "HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                    "/v", "AppsUseLightTheme"
                ])
                .output()
            {
                if output.status.success() {
                    let registry_output = String::from_utf8_lossy(&output.stdout);
                    // Look for "0x0" (dark) or "0x1" (light)
                    let is_light = registry_output.contains("0x1");
                    tracing::debug!("Windows AppsUseLightTheme registry: {} -> {}", 
                                  if is_light { "0x1" } else { "0x0" }, 
                                  if is_light { "light" } else { "dark" });
                    return !is_light; // Return true for dark theme
                }
            }
            tracing::debug!("Windows system theme detection failed, defaulting to dark");
            return true; // Default to dark on Windows
        }
        
        #[cfg(target_os = "linux")]
        {
            // On Linux, try to detect through various desktop environment settings
            // Check GTK theme first
            if let Ok(gtk_theme) = std::env::var("GTK_THEME") {
                let is_dark = gtk_theme.to_lowercase().contains("dark");
                tracing::debug!("Linux GTK_THEME: '{}' -> {}", gtk_theme, if is_dark { "dark" } else { "light" });
                return is_dark;
            }
            
            // Check for dark themes in gsettings (GNOME)
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                .output()
            {
                if output.status.success() {
                    let theme_name = String::from_utf8_lossy(&output.stdout);
                    let is_dark = theme_name.to_lowercase().contains("dark");
                    tracing::debug!("Linux GNOME theme: '{}' -> {}", theme_name.trim(), if is_dark { "dark" } else { "light" });
                    return is_dark;
                }
            }
            
            tracing::debug!("Linux system theme detection failed, defaulting to dark");
            true // Default to dark on Linux
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", unix)))]
        {
            // Fallback for other platforms
            true
        }
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
            Theme::HighContrast => "High Contrast",
        }
    }

    /// Get all available themes
    pub fn get_available_themes() -> Vec<Theme> {
        vec![Theme::Dark, Theme::Light, Theme::HighContrast, Theme::Auto]
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
            Theme::HighContrast => Theme::Light, // Go to light from high contrast
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
        manual_override_active: false,
        last_detected_system_theme: ThemeManager::detect_system_theme(),
        system_theme_listener: None,
    })
}

/// State for the theme manager hook
#[derive(Clone)]
pub struct ThemeManagerState {
    pub current_theme: Theme,
    pub is_applying: bool,
    pub manual_override_active: bool, // Track if user manually overrode Auto theme
    pub last_detected_system_theme: bool, // Cache last detected system theme (true = dark)
    #[cfg(feature = "web")]
    pub system_theme_listener: Option<js_sys::Function>,
    #[cfg(not(feature = "web"))]
    pub system_theme_listener: Option<()>,
}

impl ThemeManagerState {
    /// Update theme and persist to settings
    pub fn set_theme(&mut self, new_theme: Theme, settings: &mut SettingsState) {
        self.set_theme_with_override(new_theme, settings, false);
    }
    
    /// Update theme with manual override tracking
    pub fn set_theme_with_override(&mut self, new_theme: Theme, settings: &mut SettingsState, is_manual_override: bool) {
        self.is_applying = true;
        let old_theme = self.current_theme.clone();
        self.current_theme = new_theme.clone();
        
        // Track manual override behavior
        if is_manual_override {
            if matches!(old_theme, Theme::Auto) && !matches!(new_theme, Theme::Auto) {
                // User manually switched away from Auto mode
                self.manual_override_active = true;
                tracing::info!("Manual theme override activated: Auto -> {:?}", new_theme);
            } else if matches!(new_theme, Theme::Auto) {
                // User manually switched back to Auto mode
                self.manual_override_active = false;
                tracing::info!("Manual theme override deactivated: returning to Auto mode");
            }
        }
        
        // Update settings
        settings.theme = new_theme.clone();
        
        // Apply theme to UI
        ThemeManager::apply_theme(&new_theme);
        
        // Save to persistence
        save_settings_debounced(settings.clone());
        
        self.is_applying = false;
        
        let effective_theme = ThemeManager::get_effective_theme(&new_theme);
        tracing::info!(
            "Theme changed to: {:?} (effective: {:?}, manual_override: {})", 
            new_theme, 
            effective_theme,
            self.manual_override_active
        );
    }

    /// Toggle theme (with manual override tracking)
    pub fn toggle_theme(&mut self, settings: &mut SettingsState) {
        let new_theme = ThemeManager::toggle_theme(&self.current_theme);
        self.set_theme_with_override(new_theme, settings, true);
    }
    
    /// Cycle through all available themes (Dark → Light → High Contrast → Auto → Dark)
    pub fn cycle_theme(&mut self, settings: &mut SettingsState) {
        let new_theme = match self.current_theme {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::HighContrast,
            Theme::HighContrast => Theme::Auto,
            Theme::Auto => Theme::Dark,
        };
        self.set_theme_with_override(new_theme, settings, true);
    }
    
    /// Force a specific theme (always considered a manual override)
    pub fn force_theme(&mut self, theme: Theme, settings: &mut SettingsState) {
        self.set_theme_with_override(theme, settings, true);
    }
    
    /// Check if system theme has changed and update if in Auto mode
    /// Uses throttled checking to improve performance
    pub fn check_system_theme_change(&mut self, settings: &mut SettingsState) -> bool {
        if !matches!(self.current_theme, Theme::Auto) {
            return false; // Not in Auto mode, no need to check
        }
        
        // Use throttled theme detection with caching
        use std::sync::{Arc, Mutex};
        use std::sync::OnceLock;
        
        static GLOBAL_THEME_OPTIMIZER: OnceLock<Arc<Mutex<ThemeOptimizer>>> = OnceLock::new();
        let optimizer = GLOBAL_THEME_OPTIMIZER.get_or_init(|| {
                let profiler = Arc::new(Mutex::new(RenderingProfiler::new()));
            Arc::new(Mutex::new(ThemeOptimizer::new(profiler)))
        });

        // For now, we'll check every few seconds but use throttling
        let should_check = if let Ok(mut opt) = optimizer.lock() {
            // Check if we have a cached result that's not too old
            opt.get_optimized_theme().is_none()
        } else {
            true // Default to checking if lock fails
        };

        if !should_check {
            tracing::debug!("Theme check throttled, using cached result");
            return false;
        }
        
        let current_system_theme = ThemeManager::detect_system_theme();
        if current_system_theme != self.last_detected_system_theme {
            self.last_detected_system_theme = current_system_theme;
            
            // Re-apply Auto theme to pick up the system change
            ThemeManager::apply_theme(&Theme::Auto);
            
            tracing::info!(
                "System theme changed to: {} (Auto mode active)", 
                if current_system_theme { "dark" } else { "light" }
            );
            
            return true; // Theme changed
        }
        
        false // No change
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
    
    /// Get user-friendly description of current theme status
    pub fn get_theme_status_description(&self) -> String {
        let effective = self.get_effective_theme();
        match (&self.current_theme, self.manual_override_active) {
            (Theme::Auto, false) => {
                format!("Auto (following system: {})", ThemeManager::get_theme_display_name(&effective))
            }
            (Theme::Auto, true) => {
                // This shouldn't happen, but handle gracefully
                format!("Auto (system: {})", ThemeManager::get_theme_display_name(&effective))
            }
            (theme, true) => {
                format!("{} (manual override)", ThemeManager::get_theme_display_name(theme))
            }
            (theme, false) => {
                ThemeManager::get_theme_display_name(theme).to_string()
            }
        }
    }
    
    /// Check if current theme choice is overriding system preference
    pub fn is_overriding_system(&self) -> bool {
        if matches!(self.current_theme, Theme::Auto) {
            return false;
        }
        
        let system_preference = if self.last_detected_system_theme { Theme::Dark } else { Theme::Light };
        self.current_theme != system_preference
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

/// Enhanced theme selector with status information
#[component]
pub fn EnhancedThemeSelector(
    current_theme: Theme,
    theme_manager_state: Signal<ThemeManagerState>,
    on_theme_change: EventHandler<Theme>,
) -> Element {
    let manager_state = theme_manager_state.read();
    let effective_theme = manager_state.get_effective_theme();
    let status_description = manager_state.get_theme_status_description();
    let is_overriding = manager_state.is_overriding_system();
    
    rsx! {
        div {
            class: "enhanced-theme-selector",
            style: "display: flex; flex-direction: column; gap: 4px;",
            
            div {
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
                
                if is_overriding {
                    span {
                        style: "
                            color: var(--vscode-warning-foreground);
                            font-size: var(--vscode-font-size-small);
                            margin-left: 4px;
                        ",
                        title: "Theme is manually overriding system preference",
                        "⚠️"
                    }
                }
            }
            
            div {
                style: "
                    color: var(--vscode-text-secondary);
                    font-size: var(--vscode-font-size-small);
                    font-family: var(--vscode-font-family);
                    font-style: italic;
                ",
                "Current: {status_description}"
            }
        }
    }
}