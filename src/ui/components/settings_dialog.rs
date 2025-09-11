use dioxus::prelude::*;
use crate::state::{SettingsState, Theme, FontFamily, FontSize};
use crate::theme::{ThemeManager, EnhancedThemeSelector};

/// Props for settings dialog
#[derive(Props, Clone, PartialEq)]
pub struct SettingsDialogProps {
    pub visible: bool,
    pub on_close: EventHandler<()>,
    pub current_settings: Signal<SettingsState>,
    pub on_settings_change: EventHandler<SettingsState>,
}

/// Settings dialog component for application preferences
#[component]
pub fn SettingsDialog(props: SettingsDialogProps) -> Element {
    if !props.visible {
        return rsx! { div {} };
    }

    // Create local copy of settings for editing
    let mut local_settings = use_signal(|| props.current_settings.read().clone());
    
    // Sync with current settings when dialog opens
    use_effect(move || {
        if props.visible {
            local_settings.set(props.current_settings.read().clone());
        }
    });

    let on_theme_change = move |new_theme: Theme| {
        let mut settings = local_settings.write();
        settings.theme = new_theme;
        // Apply changes immediately for preview
        props.on_settings_change.call(settings.clone());
    };

    let on_save = move |_| {
        // Save current local settings
        props.on_settings_change.call(local_settings.read().clone());
        props.on_close.call(());
    };

    let on_cancel = move |_| {
        // Revert to original settings
        props.on_settings_change.call(props.current_settings.read().clone());
        props.on_close.call(());
    };

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| on_cancel(()),
            
            div {
                class: "settings-dialog",
                onclick: |evt| evt.stop_propagation(),
                style: "
                    background: var(--vscode-background);
                    border: 1px solid var(--vscode-border);
                    border-radius: 8px;
                    padding: 0;
                    max-width: 600px;
                    width: 90vw;
                    max-height: 80vh;
                    overflow: hidden;
                    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
                ",
                
                // Dialog Header
                div {
                    class: "dialog-header",
                    style: "
                        display: flex;
                        align-items: center;
                        justify-content: space-between;
                        padding: 16px 20px;
                        border-bottom: 1px solid var(--vscode-border);
                        background: var(--vscode-secondary-background);
                    ",
                    
                    div {
                        style: "display: flex; align-items: center; gap: 8px;",
                        span { 
                            class: "dialog-icon",
                            style: "font-size: 18px;",
                            "⚙️" 
                        }
                        h2 { 
                            style: "
                                margin: 0;
                                font-size: 18px;
                                font-weight: 600;
                                color: var(--vscode-text-primary);
                            ",
                            "Settings" 
                        }
                    }
                    
                    button {
                        class: "icon-button",
                        style: "
                            background: transparent;
                            border: none;
                            color: var(--vscode-text-secondary);
                            cursor: pointer;
                            padding: 4px;
                            font-size: 16px;
                            border-radius: 4px;
                        ",
                        onclick: move |_| on_cancel(()),
                        title: "Close settings",
                        "×"
                    }
                }
                
                // Dialog Content
                div {
                    class: "dialog-content",
                    style: "
                        padding: 20px;
                        overflow-y: auto;
                        max-height: 60vh;
                    ",
                    
                    // Appearance Section
                    div {
                        class: "settings-section",
                        style: "
                            margin-bottom: 24px;
                            padding-bottom: 20px;
                            border-bottom: 1px solid var(--vscode-border);
                        ",
                        
                        h3 {
                            style: "
                                margin: 0 0 16px 0;
                                font-size: 16px;
                                font-weight: 600;
                                color: var(--vscode-text-primary);
                                display: flex;
                                align-items: center;
                                gap: 8px;
                            ",
                            span { style: "font-size: 14px;", "🎨" }
                            "Appearance"
                        }
                        
                        div {
                            class: "setting-item",
                            style: "
                                display: flex;
                                flex-direction: column;
                                gap: 8px;
                                padding: 12px 0;
                            ",
                            
                            label {
                                style: "
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                ",
                                "Theme"
                            }
                            
                            div {
                                style: "padding-left: 8px;",
                                EnhancedThemeSelector {
                                    current_theme: local_settings.read().theme.clone(),
                                    theme_manager_state: use_signal(|| crate::theme::ThemeManagerState {
                                        current_theme: local_settings.read().theme.clone(),
                                        is_applying: false,
                                        manual_override_active: false,
                                        last_detected_system_theme: ThemeManager::detect_system_theme(),
                                        system_theme_listener: None,
                                    }),
                                    on_theme_change: on_theme_change,
                                }
                            }
                            
                            p {
                                style: "
                                    margin: 4px 0 0 8px;
                                    color: var(--vscode-text-secondary);
                                    font-size: 12px;
                                    line-height: 1.4;
                                ",
                                "Select your preferred color theme. Auto mode follows your system preference."
                            }
                        }

                        // Font Family Setting
                        div {
                            class: "setting-item",
                            style: "
                                display: flex;
                                flex-direction: column;
                                gap: 8px;
                                padding: 12px 0;
                                border-top: 1px solid var(--vscode-border-light);
                                margin-top: 16px;
                                padding-top: 16px;
                            ",
                            
                            label {
                                style: "
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                ",
                                "Font Family"
                            }
                            
                            select {
                                value: "{local_settings.read().font_family.as_str()}",
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
                                    
                                    let mut settings = local_settings.write();
                                    settings.font_family = font_family.clone();
                                    
                                    // Update the custom CSS variables in settings
                                    settings.custom_css_variables.insert("--vscode-font-family".to_string(), font_family.css_value().to_string());
                                    
                                    // Apply font family changes immediately by updating CSS variables
                                    tracing::info!("Font family will be applied on next theme refresh");
                                    
                                    // Apply changes immediately for preview
                                    props.on_settings_change.call(settings.clone());
                                },
                                
                                for font in FontFamily::get_all() {
                                    option {
                                        value: "{font.as_str()}",
                                        selected: local_settings.read().font_family == font,
                                        "{font.display_name()}"
                                    }
                                }
                            }
                            
                            p {
                                style: "
                                    margin: 4px 0 0 0;
                                    color: var(--vscode-text-secondary);
                                    font-size: 12px;
                                    line-height: 1.4;
                                ",
                                "Choose the font family used throughout the interface."
                            }
                        }

                        // Font Size Setting
                        div {
                            class: "setting-item",
                            style: "
                                display: flex;
                                flex-direction: column;
                                gap: 8px;
                                padding: 12px 0;
                                border-top: 1px solid var(--vscode-border-light);
                                margin-top: 16px;
                                padding-top: 16px;
                            ",
                            
                            label {
                                style: "
                                    color: var(--vscode-text-primary);
                                    font-size: 14px;
                                    font-weight: 500;
                                ",
                                "Font Size"
                            }
                            
                            select {
                                value: "{local_settings.read().font_size.as_str()}",
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
                                    
                                    let mut settings = local_settings.write();
                                    settings.font_size = font_size.clone();
                                    
                                    // Update the custom CSS variables in settings
                                    settings.custom_css_variables.insert("--vscode-font-size-normal".to_string(), font_size.css_value().to_string());
                                    settings.custom_css_variables.insert("--vscode-font-size".to_string(), font_size.css_value().to_string());
                                    
                                    // Apply font size changes immediately by updating CSS variables  
                                    tracing::info!("Font size will be applied on next theme refresh");
                                    
                                    // Apply changes immediately for preview
                                    props.on_settings_change.call(settings.clone());
                                },
                                
                                for size in FontSize::get_all() {
                                    option {
                                        value: "{size.as_str()}",
                                        selected: local_settings.read().font_size == size,
                                        "{size.display_name()}"
                                    }
                                }
                            }
                            
                            p {
                                style: "
                                    margin: 4px 0 0 0;
                                    color: var(--vscode-text-secondary);
                                    font-size: 12px;
                                    line-height: 1.4;
                                ",
                                "Adjust the size of text displayed in the interface."
                            }
                        }
                    }
                    
                    // File Management Section
                    div {
                        class: "settings-section",
                        style: "
                            margin-bottom: 24px;
                            padding-bottom: 20px;
                            border-bottom: 1px solid var(--vscode-border);
                        ",
                        
                        h3 {
                            style: "
                                margin: 0 0 16px 0;
                                font-size: 16px;
                                font-weight: 600;
                                color: var(--vscode-text-primary);
                                display: flex;
                                align-items: center;
                                gap: 8px;
                            ",
                            span { style: "font-size: 14px;", "📁" }
                            "File Management"
                        }
                        
                        div {
                            class: "setting-item",
                            style: "
                                display: flex;
                                align-items: center;
                                justify-content: space-between;
                                padding: 8px 0;
                            ",
                            
                            div {
                                label {
                                    style: "
                                        color: var(--vscode-text-primary);
                                        font-size: 14px;
                                        font-weight: 500;
                                        display: block;
                                        margin-bottom: 4px;
                                    ",
                                    "Remember last folder"
                                }
                                p {
                                    style: "
                                        margin: 0;
                                        color: var(--vscode-text-secondary);
                                        font-size: 12px;
                                        line-height: 1.4;
                                    ",
                                    "Automatically reopen the last viewed folder on startup"
                                }
                            }
                            
                            input {
                                r#type: "checkbox",
                                checked: local_settings.read().remember_last_directory,
                                style: "
                                    accent-color: var(--vscode-accent);
                                    transform: scale(1.2);
                                ",
                                onchange: move |evt| {
                                    let mut settings = local_settings.write();
                                    settings.remember_last_directory = evt.checked();
                                }
                            }
                        }
                    }
                    
                    // Advanced Section
                    div {
                        class: "settings-section",
                        
                        h3 {
                            style: "
                                margin: 0 0 16px 0;
                                font-size: 16px;
                                font-weight: 600;
                                color: var(--vscode-text-primary);
                                display: flex;
                                align-items: center;
                                gap: 8px;
                            ",
                            span { style: "font-size: 14px;", "⚙️" }
                            "Advanced"
                        }
                        
                        div {
                            class: "setting-item",
                            style: "padding: 8px 0;",
                            
                            p {
                                style: "
                                    margin: 0;
                                    color: var(--vscode-text-secondary);
                                    font-size: 12px;
                                    line-height: 1.4;
                                ",
                                "Settings are automatically saved when changed. Use Ctrl+, to quickly open settings."
                            }
                        }
                    }
                }
                
                // Dialog Actions
                div {
                    class: "dialog-actions",
                    style: "
                        display: flex;
                        justify-content: flex-end;
                        gap: 8px;
                        padding: 16px 20px;
                        border-top: 1px solid var(--vscode-border);
                        background: var(--vscode-secondary-background);
                    ",
                    
                    button {
                        class: "button secondary",
                        style: "
                            background: transparent;
                            color: var(--vscode-text-secondary);
                            border: 1px solid var(--vscode-border);
                            padding: 8px 16px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-size: 14px;
                        ",
                        onclick: move |_| on_cancel(()),
                        "Cancel"
                    }
                    
                    button {
                        class: "button primary",
                        style: "
                            background: var(--vscode-accent);
                            color: white;
                            border: none;
                            padding: 8px 16px;
                            border-radius: 4px;
                            cursor: pointer;
                            font-size: 14px;
                        ",
                        onclick: move |_| on_save(()),
                        "Save"
                    }
                }
            }
        }
    }
}