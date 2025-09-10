use dioxus::prelude::*;
use crate::theme::{ThemeManager};
use crate::state::{use_app_state, save_settings_debounced, Theme, FontFamily, FontSize};

/// VSCode-style settings dialog with enhanced theme management
#[component]
pub fn VsCodeSettingsDialog(
    is_open: Signal<bool>,
) -> Element {
    let mut app_state = use_app_state();
    
    if !is_open.read().clone() {
        return rsx! { div {} };
    }
    
    let current_settings = app_state.settings.read();
    let current_theme = current_settings.theme.clone();
    let current_font_family = current_settings.font_family.clone();
    let current_font_size = current_settings.font_size.clone();
    drop(current_settings);
    
    rsx! {
        // Modal overlay
        div {
            class: "vscode-settings-overlay",
            style: "
                position: fixed;
                top: 0;
                left: 0;
                width: 100vw;
                height: 100vh;
                background-color: rgba(0, 0, 0, 0.5);
                display: flex;
                justify-content: center;
                align-items: center;
                z-index: 10000;
                backdrop-filter: blur(2px);
            ",
            onclick: move |_| {
                is_open.set(false);
            },
            
            // Settings dialog
            div {
                class: "vscode-settings-dialog",
                style: "
                    width: 600px;
                    max-height: 80vh;
                    background-color: var(--vscode-secondary-background);
                    border: 1px solid var(--vscode-border);
                    border-radius: 8px;
                    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
                    display: flex;
                    flex-direction: column;
                    overflow: hidden;
                    font-family: var(--vscode-font-family);
                ",
                // Prevent event propagation to avoid closing when clicking inside
                onclick: move |evt| {
                    evt.stop_propagation();
                },
                
                // Header
                div {
                    class: "settings-header",
                    style: "
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        padding: 16px 24px;
                        border-bottom: 1px solid var(--vscode-border);
                        background-color: var(--vscode-title-bar-active-background);
                    ",
                    
                    h2 {
                        style: "
                            margin: 0;
                            color: var(--vscode-title-bar-active-foreground);
                            font-size: 18px;
                            font-weight: 600;
                        ",
                        "Settings"
                    }
                    
                    button {
                        style: "
                            background: none;
                            border: none;
                            color: var(--vscode-title-bar-active-foreground);
                            font-size: 20px;
                            cursor: pointer;
                            padding: 4px 8px;
                            border-radius: 4px;
                            transition: background-color 0.2s ease;
                        ",
                        onmouseover: |_| {},
                        onmouseout: |_| {},
                        onclick: move |_| {
                            is_open.set(false);
                        },
                        "✕"
                    }
                }
                
                // Content
                div {
                    class: "settings-content",
                    style: "
                        flex: 1;
                        padding: 24px;
                        overflow-y: auto;
                        background-color: var(--vscode-background);
                    ",
                    
                    // Appearance section
                    div {
                        class: "settings-section",
                        style: "margin-bottom: 32px;",
                        
                        h3 {
                            style: "
                                margin: 0 0 16px 0;
                                color: var(--vscode-settings-header-foreground);
                                font-size: 16px;
                                font-weight: 600;
                                border-bottom: 1px solid var(--vscode-border);
                                padding-bottom: 8px;
                            ",
                            "🎨 Appearance"
                        }
                        
                        // Theme dropdown
                        div {
                            style: "margin-bottom: 16px;",
                            
                            label {
                                style: "
                                    display: block;
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                    margin-bottom: 6px;
                                ",
                                "Color Theme:"
                            }
                            
                            select {
                                value: "{current_theme.as_str()}",
                                style: "
                                    width: 100%;
                                    background-color: var(--vscode-input-background);
                                    color: var(--vscode-input-foreground);
                                    border: 1px solid var(--vscode-input-border);
                                    border-radius: 4px;
                                    padding: 8px 12px;
                                    font-size: 13px;
                                    font-family: var(--vscode-font-family);
                                    outline: none;
                                    cursor: pointer;
                                ",
                                onchange: move |evt| {
                                    let theme = Theme::from_str(&evt.value());
                                    tracing::info!("Theme changed to: {:?}", theme);
                                    
                                    // Update app state settings
                                    let mut settings = app_state.settings.write();
                                    settings.theme = theme.clone();
                                    
                                    // Apply theme immediately
                                    ThemeManager::apply_theme(&theme);
                                    
                                    // Save settings
                                    save_settings_debounced(settings.clone());
                                },
                                
                                option { value: "dark", selected: current_theme == Theme::Dark, "Dark" }
                                option { value: "light", selected: current_theme == Theme::Light, "Light" }
                                option { value: "high-contrast", selected: current_theme == Theme::HighContrast, "High Contrast" }
                                option { value: "auto", selected: current_theme == Theme::Auto, "Auto (Follow System)" }
                            }
                        }
                        
                        // Font family dropdown
                        div {
                            style: "margin-bottom: 16px;",
                            
                            label {
                                style: "
                                    display: block;
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                    margin-bottom: 6px;
                                ",
                                "Font Family:"
                            }
                            
                            select {
                                value: "{current_font_family.as_str()}",
                                style: "
                                    width: 100%;
                                    background-color: var(--vscode-input-background);
                                    color: var(--vscode-input-foreground);
                                    border: 1px solid var(--vscode-input-border);
                                    border-radius: 4px;
                                    padding: 8px 12px;
                                    font-size: 13px;
                                    font-family: var(--vscode-font-family);
                                    outline: none;
                                    cursor: pointer;
                                ",
                                onchange: move |evt| {
                                    let font_family = FontFamily::from_str(&evt.value());
                                    tracing::info!("Font family changed to: {:?}", font_family);
                                    
                                    // Update app state settings
                                    let mut settings = app_state.settings.write();
                                    settings.font_family = font_family.clone();
                                    
                                    // Apply font family immediately via CSS variable
                                    let mut css_vars = std::collections::HashMap::new();
                                    css_vars.insert("--vscode-font-family".to_string(), font_family.css_value().to_string());
                                    ThemeManager::apply_custom_css_variables(&css_vars);
                                    
                                    // Save settings
                                    save_settings_debounced(settings.clone());
                                },
                                
                                for font in FontFamily::get_all() {
                                    option {
                                        value: "{font.as_str()}",
                                        selected: current_font_family == font,
                                        "{font.display_name()}"
                                    }
                                }
                            }
                        }
                        
                        // Font size dropdown
                        div {
                            style: "margin-bottom: 16px;",
                            
                            label {
                                style: "
                                    display: block;
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                    margin-bottom: 6px;
                                ",
                                "Font Size:"
                            }
                            
                            select {
                                value: "{current_font_size.as_str()}",
                                style: "
                                    width: 100%;
                                    background-color: var(--vscode-input-background);
                                    color: var(--vscode-input-foreground);
                                    border: 1px solid var(--vscode-input-border);
                                    border-radius: 4px;
                                    padding: 8px 12px;
                                    font-size: 13px;
                                    font-family: var(--vscode-font-family);
                                    outline: none;
                                    cursor: pointer;
                                ",
                                onchange: move |evt| {
                                    let font_size = FontSize::from_str(&evt.value());
                                    tracing::info!("Font size changed to: {:?}", font_size);
                                    
                                    // Update app state settings
                                    let mut settings = app_state.settings.write();
                                    settings.font_size = font_size.clone();
                                    
                                    // Apply font size immediately via CSS variable
                                    let mut css_vars = std::collections::HashMap::new();
                                    css_vars.insert("--vscode-font-size-normal".to_string(), font_size.css_value().to_string());
                                    ThemeManager::apply_custom_css_variables(&css_vars);
                                    
                                    // Save settings
                                    save_settings_debounced(settings.clone());
                                },
                                
                                for size in FontSize::get_all() {
                                    option {
                                        value: "{size.as_str()}",
                                        selected: current_font_size == size,
                                        "{size.display_name()}"
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Footer
                div {
                    class: "settings-footer",
                    style: "
                        display: flex;
                        justify-content: flex-end;
                        gap: 12px;
                        padding: 16px 24px;
                        border-top: 1px solid var(--vscode-border);
                        background-color: var(--vscode-secondary-background);
                    ",
                    
                    button {
                        style: "
                            background-color: var(--vscode-secondary-background);
                            color: var(--vscode-text-primary);
                            border: 1px solid var(--vscode-border);
                            border-radius: 4px;
                            padding: 8px 16px;
                            font-size: 13px;
                            font-family: var(--vscode-font-family);
                            cursor: pointer;
                            transition: background-color 0.2s ease;
                        ",
                        onclick: move |_| {
                            is_open.set(false);
                        },
                        "Cancel"
                    }
                    
                    button {
                        style: "
                            background-color: var(--vscode-button-background);
                            color: var(--vscode-button-foreground);
                            border: none;
                            border-radius: 4px;
                            padding: 8px 16px;
                            font-size: 13px;
                            font-family: var(--vscode-font-family);
                            cursor: pointer;
                            transition: background-color 0.2s ease;
                        ",
                        onclick: move |_| {
                            // Save settings and close
                            let settings = app_state.settings.read().clone();
                            save_settings_debounced(settings);
                            is_open.set(false);
                            tracing::info!("Settings saved successfully");
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

