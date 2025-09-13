use dioxus::prelude::*;
use crate::state::{SettingsState, Theme, FontFamily, FontSize};
use crate::theme::{ThemeManager, EnhancedThemeSelector};
use crate::ui::components::IconPackManager;

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

    // Work directly with the current settings signal - no local copy needed
    let on_theme_change = move |new_theme: Theme| {
        props.on_settings_change.call({
            let mut settings = props.current_settings.read().clone();
            settings.theme = new_theme;
            settings
        });
    };

    let on_close = move |_| {
        props.on_close.call(());
    };

    rsx! {
        div {
            class: "dialog-overlay",
            onclick: move |_| on_close(()),
            
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
                        onclick: move |_| on_close(()),
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
                                    current_theme: props.current_settings.read().theme.clone(),
                                    theme_manager_state: use_signal(|| crate::theme::ThemeManagerState {
                                        current_theme: props.current_settings.read().theme.clone(),
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
                                value: "{props.current_settings.read().font_family.as_str()}",
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
                                    
                                    props.on_settings_change.call({
                                        let mut settings = props.current_settings.read().clone();
                                        settings.font_family = font_family.clone();
                                        
                                        // Update the custom CSS variables in settings
                                        settings.custom_css_variables.insert("--vscode-font-family".to_string(), font_family.css_value().to_string());
                                        
                                        tracing::info!("Font family applied immediately: {:?}", font_family);
                                        settings
                                    });
                                },
                                
                                for font in FontFamily::get_all() {
                                    option {
                                        value: "{font.as_str()}",
                                        selected: props.current_settings.read().font_family == font,
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
                                value: "{props.current_settings.read().font_size.as_str()}",
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
                                    
                                    props.on_settings_change.call({
                                        let mut settings = props.current_settings.read().clone();
                                        settings.font_size = font_size.clone();
                                        
                                        // Update the custom CSS variables in settings
                                        settings.custom_css_variables.insert("--vscode-font-size-normal".to_string(), font_size.css_value().to_string());
                                        settings.custom_css_variables.insert("--vscode-font-size".to_string(), font_size.css_value().to_string());
                                        
                                        tracing::info!("Font size applied immediately: {:?}", font_size);
                                        settings
                                    });
                                },
                                
                                for size in FontSize::get_all() {
                                    option {
                                        value: "{size.as_str()}",
                                        selected: props.current_settings.read().font_size == size,
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
                    
                    // Icon Packs Section
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
                            span { style: "font-size: 14px;", "📦" }
                            "Icon Packs"
                        }
                        
                        div {
                            style: "
                                max-height: 300px;
                                overflow-y: auto;
                                margin-top: 8px;
                            ",
                            IconPackManager {}
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
                                checked: props.current_settings.read().remember_last_directory,
                                style: "
                                    accent-color: var(--vscode-accent);
                                    transform: scale(1.2);
                                ",
                                onchange: move |evt| {
                                    props.on_settings_change.call({
                                        let mut settings = props.current_settings.read().clone();
                                        settings.remember_last_directory = evt.checked();
                                        tracing::info!("Remember last directory changed to: {}", evt.checked());
                                        settings
                                    });
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
                                "Settings are automatically applied when changed. Use Ctrl+, to quickly open settings."
                            }
                        }
                    }
                }
            }
        }
    }
}