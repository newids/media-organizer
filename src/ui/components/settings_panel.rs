use dioxus::prelude::*;
use crate::state::{SettingsState, Theme, ViewMode, save_settings_debounced};
use crate::theme::{ThemeManager, ThemeSelector, EnhancedThemeSelector};
use crate::ui::shortcuts::ShortcutRegistry;
use std::collections::HashMap;

/// Settings panel component with tabbed interface for different settings categories
#[component]
pub fn SettingsPanel(
    is_visible: Signal<bool>,
    mut settings: Signal<SettingsState>,
    on_close: EventHandler<()>,
) -> Element {
    // Local state for the settings panel
    let active_tab = use_signal(|| SettingsTab::General);
    let temp_settings = use_signal(|| settings.read().clone());
    let is_modified = use_signal(|| false);
    let import_export_message = use_signal(|| String::new());

    // Update temp settings when main settings change
    use_effect({
        let mut temp_settings = temp_settings;
        let settings = settings;
        move || {
            let current_settings = settings.read();
            temp_settings.set(current_settings.clone());
        }
    });

    if !*is_visible.read() {
        return rsx! { div { display: "none" } };
    }

    // Apply settings changes
    let mut apply_settings = {
        let mut settings = settings;
        let mut is_modified = is_modified;
        let temp_settings = temp_settings;
        
        move || {
            let new_settings = temp_settings.read().clone();
            settings.set(new_settings.clone());
            save_settings_debounced(new_settings);
            is_modified.set(false);
            tracing::info!("Settings applied and saved");
        }
    };

    // Reset settings to current saved state
    let mut reset_settings = {
        let mut temp_settings = temp_settings;
        let settings = settings;
        let mut is_modified = is_modified;
        
        move || {
            temp_settings.set(settings.read().clone());
            is_modified.set(false);
        }
    };

    // Check if current temp settings differ from saved
    let mut check_modifications = {
        let temp_settings = temp_settings;
        let settings = settings;
        let mut is_modified = is_modified;
        
        move || {
            let current = settings.read();
            let temp = temp_settings.read();
            
            let modified = current.theme != temp.theme ||
                          current.default_panel_width != temp.default_panel_width ||
                          current.default_view_mode != temp.default_view_mode ||
                          current.show_hidden_files != temp.show_hidden_files ||
                          current.remember_last_directory != temp.remember_last_directory ||
                          current.auto_save_interval != temp.auto_save_interval ||
                          current.enable_animations != temp.enable_animations ||
                          current.custom_css_variables != temp.custom_css_variables;
            
            is_modified.set(modified);
        }
    };

    rsx! {
        div {
            class: "settings-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "settings-title",
            "aria-describedby": "settings-description",
            style: "
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: var(--vscode-overlay-background, rgba(0, 0, 0, 0.5));
                z-index: 10000;
                display: flex;
                align-items: center;
                justify-content: center;
                animation: fadeIn 0.2s ease-out;
            ",
            
            onclick: move |_evt| {
                // Close when clicking overlay background
                on_close.call(());
            },
            
            div {
                class: "settings-panel",
                style: "
                    background: var(--vscode-panel-background, #1e1e1e);
                    border: 1px solid var(--vscode-panel-border, #333);
                    border-radius: 8px;
                    width: 800px;
                    height: 600px;
                    display: flex;
                    flex-direction: column;
                    box-shadow: var(--vscode-shadow-large, 0 8px 16px rgba(0, 0, 0, 0.4));
                    animation: slideIn 0.2s ease-out;
                ",
                onclick: move |evt| evt.stop_propagation(), // Prevent closing when clicking panel
                
                // Header
                div {
                    class: "settings-header",
                    style: "
                        padding: 16px 20px;
                        border-bottom: 1px solid var(--vscode-panel-border, #333);
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                    ",
                    
                    h2 {
                        id: "settings-title",
                        style: "
                            color: var(--vscode-text-primary, #cccccc);
                            font-size: 18px;
                            font-weight: 600;
                            margin: 0;
                            font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                        ",
                        "Settings"
                    }
                    
                    button {
                        class: "close-button",
                        "aria-label": "Close settings panel",
                        title: "Close settings (Escape)",
                        style: "
                            background: transparent;
                            border: none;
                            color: var(--vscode-text-secondary, #999);
                            font-size: 20px;
                            cursor: pointer;
                            padding: 4px 8px;
                            border-radius: 4px;
                            transition: all 0.2s ease;
                        ",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }
                
                // Content area
                div {
                    class: "settings-content",
                    style: "
                        display: flex;
                        flex: 1;
                        overflow: hidden;
                    ",
                    
                    // Sidebar with tabs
                    div {
                        class: "settings-sidebar",
                        role: "tablist",
                        "aria-orientation": "vertical",
                        "aria-label": "Settings categories",
                        style: "
                            width: 200px;
                            background: var(--vscode-secondary-background, #2d2d30);
                            border-right: 1px solid var(--vscode-panel-border, #333);
                            padding: 16px 0;
                            overflow-y: auto;
                        ",
                        
                        for tab in [SettingsTab::General, SettingsTab::Appearance, SettingsTab::Keyboard, SettingsTab::Advanced] {
                            SettingsTabButton {
                                tab: tab,
                                is_active: tab == *active_tab.read(),
                                on_click: {
                                    let mut active_tab = active_tab;
                                    move |new_tab: SettingsTab| {
                                        active_tab.set(new_tab);
                                    }
                                },
                            }
                        }
                    }
                    
                    // Main content area
                    div {
                        class: "settings-main",
                        role: "tabpanel",
                        id: format!("{}-panel", active_tab.read().id()),
                        "aria-labelledby": format!("{}-tab", active_tab.read().id()),
                        style: "
                            flex: 1;
                            padding: 20px;
                            overflow-y: auto;
                        ",
                        
                        // Hidden description for screen readers
                        div {
                            id: "settings-description",
                            class: "sr-only",
                            style: "position: absolute; left: -10000px; width: 1px; height: 1px; overflow: hidden;",
                            "Configure application preferences and customize your MediaOrganizer experience"
                        }
                        
                        match *active_tab.read() {
                            SettingsTab::General => rsx! {
                                GeneralSettingsTab {
                                    settings: temp_settings,
                                    on_change: move |_| check_modifications(),
                                }
                            },
                            SettingsTab::Appearance => rsx! {
                                AppearanceSettingsTab {
                                    settings: temp_settings,
                                    on_change: move |_| check_modifications(),
                                }
                            },
                            SettingsTab::Keyboard => rsx! {
                                KeyboardSettingsTab {}
                            },
                            SettingsTab::Advanced => rsx! {
                                AdvancedSettingsTab {
                                    settings: temp_settings,
                                    on_change: move |_| check_modifications(),
                                    import_export_message: import_export_message,
                                }
                            },
                        }
                    }
                }
                
                // Footer with action buttons
                div {
                    class: "settings-footer",
                    style: "
                        padding: 16px 20px;
                        border-top: 1px solid var(--vscode-panel-border, #333);
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        background: var(--vscode-secondary-background, #2d2d30);
                    ",
                    
                    div {
                        if *is_modified.read() {
                            span {
                                style: "
                                    color: var(--vscode-warning-foreground, #ffcc02);
                                    font-size: 14px;
                                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                ",
                                "● Unsaved changes"
                            }
                        }
                    }
                    
                    div {
                        style: "display: flex; gap: 12px;",
                        
                        button {
                            class: "settings-button secondary",
                            style: "
                                background: transparent;
                                border: 1px solid var(--vscode-button-border, #666);
                                color: var(--vscode-text-primary, #cccccc);
                                padding: 8px 16px;
                                border-radius: 4px;
                                cursor: pointer;
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                                transition: all 0.2s ease;
                            ",
                            disabled: !*is_modified.read(),
                            onclick: move |_| reset_settings(),
                            "Reset"
                        }
                        
                        button {
                            class: "settings-button primary",
                            style: "
                                background: var(--vscode-button-background, #0e639c);
                                border: 1px solid var(--vscode-button-border, #0e639c);
                                color: var(--vscode-button-foreground, #ffffff);
                                padding: 8px 16px;
                                border-radius: 4px;
                                cursor: pointer;
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                                font-weight: 500;
                                transition: all 0.2s ease;
                            ",
                            disabled: !*is_modified.read(),
                            onclick: move |_| apply_settings(),
                            "Apply"
                        }
                        
                        button {
                            class: "settings-button secondary",
                            style: "
                                background: transparent;
                                border: 1px solid var(--vscode-button-border, #666);
                                color: var(--vscode-text-primary, #cccccc);
                                padding: 8px 16px;
                                border-radius: 4px;
                                cursor: pointer;
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                                transition: all 0.2s ease;
                            ",
                            onclick: move |_| on_close.call(()),
                            "Close"
                        }
                    }
                }
            }
        }
    }
}

/// Tab identifier for settings categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Appearance,
    Keyboard,
    Advanced,
}

impl SettingsTab {
    pub fn title(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Appearance => "Appearance",
            SettingsTab::Keyboard => "Keyboard",
            SettingsTab::Advanced => "Advanced",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SettingsTab::General => "⚙️",
            SettingsTab::Appearance => "🎨",
            SettingsTab::Keyboard => "⌨️",
            SettingsTab::Advanced => "🔧",
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            SettingsTab::General => "general",
            SettingsTab::Appearance => "appearance",
            SettingsTab::Keyboard => "keyboard",
            SettingsTab::Advanced => "advanced",
        }
    }
}

/// Settings tab button component
#[component]
fn SettingsTabButton(
    tab: SettingsTab,
    is_active: bool,
    on_click: EventHandler<SettingsTab>,
) -> Element {
    rsx! {
        button {
            class: "settings-tab-button",
            role: "tab",
            "aria-selected": is_active.to_string(),
            "aria-controls": format!("{}-panel", tab.id()),
            id: format!("{}-tab", tab.id()),
            tabindex: if is_active { "0" } else { "-1" },
            style: format!("
                display: flex;
                align-items: center;
                width: 100%;
                padding: 12px 16px;
                border: none;
                background: {};
                color: {};
                text-align: left;
                cursor: pointer;
                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                font-size: 14px;
                transition: all 0.2s ease;
                border-left: 3px solid {};
            ",
                if is_active { "var(--vscode-list-active-background, #094771)" } else { "transparent" },
                if is_active { "var(--vscode-list-active-foreground, #ffffff)" } else { "var(--vscode-text-primary, #cccccc)" },
                if is_active { "var(--vscode-accent-foreground, #007acc)" } else { "transparent" }
            ),
            onclick: move |_| on_click.call(tab),
            
            span {
                style: "margin-right: 8px; font-size: 16px;",
                "{tab.icon()}"
            }
            span { "{tab.title()}" }
        }
    }
}

/// General settings tab content
#[component]
fn GeneralSettingsTab(
    mut settings: Signal<SettingsState>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "settings-section",
            
            h3 {
                style: "
                    color: var(--vscode-text-primary, #cccccc);
                    font-size: 16px;
                    font-weight: 600;
                    margin: 0 0 16px 0;
                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                ",
                "General Preferences"
            }
            
            div {
                class: "settings-group",
                style: "margin-bottom: 24px;",
                
                SettingsRow {
                    label: "Default View Mode",
                    description: "Choose the default view mode for new directories",
                    content: rsx! {
                        select {
                            value: "{settings.read().default_view_mode.as_str()}",
                            style: "
                                background: var(--vscode-input-background, #3c3c3c);
                                color: var(--vscode-input-foreground, #cccccc);
                                border: 1px solid var(--vscode-input-border, #666);
                                border-radius: 3px;
                                padding: 6px 8px;
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                            ",
                            onchange: move |evt| {
                                let view_mode = match evt.value().as_str() {
                                    "list" => ViewMode::List,
                                    "preview" => ViewMode::Preview,
                                    _ => ViewMode::Grid,
                                };
                                settings.write().default_view_mode = view_mode;
                                on_change.call(());
                            },
                            
                            option { value: "grid", "Grid View" }
                            option { value: "list", "List View" }
                            option { value: "preview", "Preview Mode" }
                        }
                    }
                }
                
                SettingsRow {
                    label: "Show Hidden Files",
                    description: "Display files and directories that start with a dot",
                    content: rsx! {
                        input {
                            r#type: "checkbox",
                            checked: settings.read().show_hidden_files,
                            style: "
                                width: 16px;
                                height: 16px;
                                accent-color: var(--vscode-checkbox-background, #007acc);
                            ",
                            onchange: move |evt| {
                                settings.write().show_hidden_files = evt.value() == "true";
                                on_change.call(());
                            }
                        }
                    }
                }
                
                SettingsRow {
                    label: "Remember Last Directory",
                    description: "Open the last visited directory when starting the application",
                    content: rsx! {
                        input {
                            r#type: "checkbox",
                            checked: settings.read().remember_last_directory,
                            style: "
                                width: 16px;
                                height: 16px;
                                accent-color: var(--vscode-checkbox-background, #007acc);
                            ",
                            onchange: move |evt| {
                                settings.write().remember_last_directory = evt.value() == "true";
                                on_change.call(());
                            }
                        }
                    }
                }
                
                SettingsRow {
                    label: "Auto-save Interval",
                    description: "How often to automatically save settings (in seconds)",
                    content: rsx! {
                        input {
                            r#type: "number",
                            value: "{settings.read().auto_save_interval}",
                            min: 10,
                            max: 3600,
                            step: 10,
                            style: "
                                background: var(--vscode-input-background, #3c3c3c);
                                color: var(--vscode-input-foreground, #cccccc);
                                border: 1px solid var(--vscode-input-border, #666);
                                border-radius: 3px;
                                padding: 6px 8px;
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                                width: 80px;
                            ",
                            onchange: move |evt| {
                                if let Ok(interval) = evt.value().parse::<u64>() {
                                    settings.write().auto_save_interval = interval.max(10).min(3600);
                                    on_change.call(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Appearance settings tab content
#[component]
fn AppearanceSettingsTab(
    mut settings: Signal<SettingsState>,
    on_change: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "settings-section",
            
            h3 {
                style: "
                    color: var(--vscode-text-primary, #cccccc);
                    font-size: 16px;
                    font-weight: 600;
                    margin: 0 0 16px 0;
                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                ",
                "Theme & Appearance"
            }
            
            div {
                class: "settings-group",
                style: "margin-bottom: 24px;",
                
                SettingsRow {
                    label: "Color Theme",
                    description: "Select the color theme for the application",
                    content: rsx! {
                        ThemeSelector {
                            current_theme: settings.read().theme.clone(),
                            on_theme_change: move |new_theme: Theme| {
                                settings.write().theme = new_theme.clone();
                                ThemeManager::apply_theme(&new_theme);
                                on_change.call(());
                            }
                        }
                    }
                }
                
                SettingsRow {
                    label: "Default Panel Width",
                    description: "Default width of the file tree panel (200-600 pixels)",
                    content: rsx! {
                        input {
                            r#type: "range",
                            min: 200,
                            max: 600,
                            step: 10,
                            value: "{settings.read().default_panel_width}",
                            style: "
                                width: 200px;
                                accent-color: var(--vscode-accent-foreground, #007acc);
                            ",
                            onchange: move |evt| {
                                if let Ok(width) = evt.value().parse::<f64>() {
                                    settings.write().default_panel_width = width.max(200.0).min(600.0);
                                    on_change.call(());
                                }
                            }
                        }
                        span {
                            style: "
                                margin-left: 12px;
                                color: var(--vscode-text-secondary, #999);
                                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                font-size: 14px;
                            ",
                            "{settings.read().default_panel_width as i32}px"
                        }
                    }
                }
                
                SettingsRow {
                    label: "Enable Animations",
                    description: "Enable smooth transitions and animations throughout the interface",
                    content: rsx! {
                        input {
                            r#type: "checkbox",
                            checked: settings.read().enable_animations,
                            style: "
                                width: 16px;
                                height: 16px;
                                accent-color: var(--vscode-checkbox-background, #007acc);
                            ",
                            onchange: move |evt| {
                                settings.write().enable_animations = evt.value() == "true";
                                on_change.call(());
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Keyboard shortcuts tab content
#[component]
fn KeyboardSettingsTab() -> Element {
    let shortcuts = ShortcutRegistry::new().get_all_shortcuts();

    rsx! {
        div {
            class: "settings-section",
            
            h3 {
                style: "
                    color: var(--vscode-text-primary, #cccccc);
                    font-size: 16px;
                    font-weight: 600;
                    margin: 0 0 16px 0;
                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                ",
                "Keyboard Shortcuts"
            }
            
            div {
                style: "
                    border: 1px solid var(--vscode-panel-border, #333);
                    border-radius: 4px;
                    overflow: hidden;
                ",
                
                div {
                    style: "
                        background: var(--vscode-secondary-background, #2d2d30);
                        padding: 12px 16px;
                        font-weight: 600;
                        font-size: 14px;
                        color: var(--vscode-text-primary, #cccccc);
                        border-bottom: 1px solid var(--vscode-panel-border, #333);
                    ",
                    "Current Shortcuts"
                }
                
                div {
                    style: "max-height: 400px; overflow-y: auto;",
                    
                    for (index, (key_combo, action)) in shortcuts.into_iter().enumerate() {
                        div {
                            key: "shortcut-{index}",
                            style: format!("
                                display: flex;
                                justify-content: space-between;
                                align-items: center;
                                padding: 12px 16px;
                                border-bottom: {};
                                background: {};
                            ",
                                if index % 2 == 0 { "1px solid var(--vscode-panel-border, #333)" } else { "none" },
                                if index % 2 == 0 { "transparent" } else { "var(--vscode-list-hover-background, rgba(255, 255, 255, 0.1))" }
                            ),
                            
                            div {
                                style: "
                                    color: var(--vscode-text-primary, #cccccc);
                                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                    font-size: 14px;
                                ",
                                "{action.description()}"
                            }
                            
                            div {
                                style: "
                                    background: var(--vscode-badge-background, #616161);
                                    color: var(--vscode-badge-foreground, #ffffff);
                                    padding: 4px 8px;
                                    border-radius: 3px;
                                    font-family: var(--vscode-font-family-monospace, 'Cascadia Code', monospace);
                                    font-size: 12px;
                                    font-weight: 500;
                                ",
                                "{key_combo.description()}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Advanced settings tab content
#[component]
fn AdvancedSettingsTab(
    mut settings: Signal<SettingsState>,
    on_change: EventHandler<()>,
    mut import_export_message: Signal<String>,
) -> Element {
    // Import/export functionality
    let mut export_settings = {
        let settings = settings;
        let mut import_export_message = import_export_message;
        
        move || {
            match serde_json::to_string_pretty(&*settings.read()) {
                Ok(json) => {
                    // In a real app, you'd trigger a file download here
                    // For now, we'll just log it and show a message
                    tracing::info!("Settings exported: {}", json);
                    import_export_message.set("Settings exported to console (in a real app, this would download a file)".to_string());
                    
                    // Clear message after 3 seconds
                    let mut message_copy = import_export_message;
                    spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                        message_copy.set(String::new());
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to export settings: {}", e);
                    import_export_message.set(format!("Export failed: {}", e));
                }
            }
        }
    };

    let mut reset_to_defaults = {
        let mut settings = settings;
        let on_change = on_change;
        let mut import_export_message = import_export_message;
        
        move || {
            settings.set(SettingsState::default());
            on_change.call(());
            import_export_message.set("Settings reset to defaults".to_string());
            
            // Clear message after 3 seconds
            let mut message_copy = import_export_message;
            spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                message_copy.set(String::new());
            });
        }
    };

    rsx! {
        div {
            class: "settings-section",
            
            h3 {
                style: "
                    color: var(--vscode-text-primary, #cccccc);
                    font-size: 16px;
                    font-weight: 600;
                    margin: 0 0 16px 0;
                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                ",
                "Advanced Settings"
            }
            
            div {
                class: "settings-group",
                style: "margin-bottom: 24px;",
                
                SettingsRow {
                    label: "Custom CSS Variables",
                    description: "Advanced theme customization using CSS custom properties",
                    content: rsx! {
                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            
                            textarea {
                                placeholder: "Enter custom CSS variables in JSON format, e.g.:\n{{\n  \"--custom-color\": \"#ff0000\",\n  \"--custom-font-size\": \"14px\"\n}}",
                                value: "{serde_json::to_string_pretty(&settings.read().custom_css_variables).unwrap_or_default()}",
                                style: "
                                    background: var(--vscode-input-background, #3c3c3c);
                                    color: var(--vscode-input-foreground, #cccccc);
                                    border: 1px solid var(--vscode-input-border, #666);
                                    border-radius: 3px;
                                    padding: 8px;
                                    font-family: var(--vscode-font-family-monospace, 'Cascadia Code', monospace);
                                    font-size: 12px;
                                    width: 100%;
                                    height: 120px;
                                    resize: vertical;
                                ",
                                onchange: move |evt| {
                                    // Try to parse as JSON
                                    match serde_json::from_str::<HashMap<String, String>>(&evt.value()) {
                                        Ok(variables) => {
                                            settings.write().custom_css_variables = variables;
                                            on_change.call(());
                                        }
                                        Err(_) => {
                                            // Invalid JSON, but don't prevent typing
                                            tracing::warn!("Invalid JSON in custom CSS variables");
                                        }
                                    }
                                }
                            }
                            
                            div {
                                style: "
                                    color: var(--vscode-text-secondary, #999);
                                    font-size: 12px;
                                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                                ",
                                "⚠️ Advanced users only. Invalid JSON will be ignored."
                            }
                        }
                    }
                }
            }
            
            h3 {
                style: "
                    color: var(--vscode-text-primary, #cccccc);
                    font-size: 16px;
                    font-weight: 600;
                    margin: 32px 0 16px 0;
                    font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                ",
                "Import/Export Settings"
            }
            
            div {
                class: "settings-group",
                style: "margin-bottom: 24px;",
                
                div {
                    style: "display: flex; gap: 12px; margin-bottom: 16px;",
                    
                    button {
                        style: "
                            background: var(--vscode-button-background, #0e639c);
                            border: 1px solid var(--vscode-button-border, #0e639c);
                            color: var(--vscode-button-foreground, #ffffff);
                            padding: 8px 16px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                            font-size: 14px;
                            font-weight: 500;
                        ",
                        onclick: move |_| export_settings(),
                        "Export Settings"
                    }
                    
                    button {
                        style: "
                            background: var(--vscode-button-secondary-background, #5a5d5e);
                            border: 1px solid var(--vscode-button-border, #666);
                            color: var(--vscode-button-secondary-foreground, #cccccc);
                            padding: 8px 16px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                            font-size: 14px;
                            font-weight: 500;
                        ",
                        "Import Settings"
                    }
                    
                    button {
                        style: "
                            background: var(--vscode-button-secondary-background, #5a5d5e);
                            border: 1px solid var(--vscode-button-border, #666);
                            color: var(--vscode-button-secondary-foreground, #cccccc);
                            padding: 8px 16px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                            font-size: 14px;
                            font-weight: 500;
                        ",
                        onclick: move |_| reset_to_defaults(),
                        "Reset to Defaults"
                    }
                }
                
                if !import_export_message.read().is_empty() {
                    div {
                        style: "
                            padding: 8px 12px;
                            background: var(--vscode-notifications-background, #2d2d30);
                            border: 1px solid var(--vscode-notifications-border, #555);
                            border-radius: 4px;
                            color: var(--vscode-text-primary, #cccccc);
                            font-size: 14px;
                            font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                        ",
                        "{import_export_message.read()}"
                    }
                }
            }
        }
    }
}

/// Reusable settings row component for consistent layout
#[component]
fn SettingsRow(
    label: String,
    description: String,
    content: Element,
) -> Element {
    rsx! {
        div {
            style: "
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                padding: 16px 0;
                border-bottom: 1px solid var(--vscode-panel-border, #333);
            ",
            
            div {
                style: "flex: 1; margin-right: 20px;",
                
                div {
                    style: "
                        color: var(--vscode-text-primary, #cccccc);
                        font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                        font-size: 14px;
                        font-weight: 500;
                        margin-bottom: 4px;
                    ",
                    "{label}"
                }
                
                div {
                    style: "
                        color: var(--vscode-text-secondary, #999);
                        font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
                        font-size: 13px;
                        line-height: 1.4;
                    ",
                    "{description}"
                }
            }
            
            div {
                style: "flex-shrink: 0; display: flex; align-items: center;",
                {content}
            }
        }
    }
}

// Add ViewMode string conversion
impl ViewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ViewMode::Grid => "grid",
            ViewMode::List => "list",
            ViewMode::Preview => "preview",
        }
    }
}