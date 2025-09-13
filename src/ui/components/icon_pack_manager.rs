use dioxus::prelude::*;
use crate::ui::icon_manager::{use_icon_manager, IconSettings};
use crate::ui::icon_packs::IconPack;

/// Icon Pack Manager page component
#[component]
pub fn IconPackManager() -> Element {
    let icon_manager = use_icon_manager();
    let current_pack = icon_manager.settings.read().current_pack.clone();
    
    rsx! {
        div {
            class: "icon-pack-manager",
            style: "
                padding: 24px;
                background: var(--vscode-editor-background, #1e1e1e);
                color: var(--vscode-foreground, #cccccc);
                font-family: var(--vscode-font-family, 'Segoe UI', sans-serif);
            ",

            // Header
            div {
                class: "header",
                style: "margin-bottom: 32px;",
                
                h1 {
                    style: "
                        margin: 0 0 8px 0;
                        font-size: 28px;
                        font-weight: 600;
                        color: var(--vscode-foreground, #cccccc);
                    ",
                    "File Icon Themes"
                }
                
                p {
                    style: "
                        margin: 0;
                        font-size: 14px;
                        color: var(--vscode-descriptionForeground, #a0a0a0);
                    ",
                    "Choose how file and folder icons appear in the explorer"
                }
            }

            // Available icon packs
            div {
                class: "icon-packs-grid",
                style: "
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
                    gap: 20px;
                    margin-bottom: 32px;
                ",
                
                IconPackCard {
                    name: "VS Code Icons",
                    description: "Default VS Code style icons",
                    pack: IconPack::VSCode,
                    is_current: current_pack == IconPack::VSCode,
                    on_select: {
                        let mut icon_manager = icon_manager.clone();
                        move |pack: IconPack| {
                            icon_manager.change_pack(pack);
                        }
                    }
                }
                
                IconPackCard {
                    name: "Material Theme",
                    description: "Google Material Design icons",
                    pack: IconPack::Material,
                    is_current: current_pack == IconPack::Material,
                    on_select: {
                        let mut icon_manager = icon_manager.clone();
                        move |pack: IconPack| {
                            icon_manager.change_pack(pack);
                        }
                    }
                }
                
                IconPackCard {
                    name: "Seti UI",
                    description: "Seti file type icons",
                    pack: IconPack::Seti,
                    is_current: current_pack == IconPack::Seti,
                    on_select: {
                        let mut icon_manager = icon_manager.clone();
                        move |pack: IconPack| {
                            icon_manager.change_pack(pack);
                        }
                    }
                }
                
                IconPackCard {
                    name: "Atom",
                    description: "Atom editor style icons",
                    pack: IconPack::Atom,
                    is_current: current_pack == IconPack::Atom,
                    on_select: {
                        let mut icon_manager = icon_manager.clone();
                        move |pack: IconPack| {
                            icon_manager.change_pack(pack);
                        }
                    }
                }
                
                IconPackCard {
                    name: "Minimal",
                    description: "Clean minimal icons",
                    pack: IconPack::Minimal,
                    is_current: current_pack == IconPack::Minimal,
                    on_select: {
                        let mut icon_manager = icon_manager.clone();
                        move |pack: IconPack| {
                            icon_manager.change_pack(pack);
                        }
                    }
                }
            }

            // Settings section
            div {
                class: "settings-section",
                style: "margin-top: 40px; padding-top: 20px; border-top: 1px solid var(--vscode-widget-border, #454545);",
                
                h2 {
                    style: "
                        font-size: 18px;
                        font-weight: 600;
                        margin: 0 0 16px 0;
                        color: var(--vscode-foreground, #cccccc);
                    ",
                    "Icon Settings"
                }
                
                div {
                    class: "settings-row",
                    style: "
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 16px;
                    ",
                    
                    div {
                        span {
                            style: "font-size: 14px; color: var(--vscode-foreground, #cccccc);",
                            "Show file icons"
                        }
                    }
                    
                    input {
                        r#type: "checkbox",
                        checked: icon_manager.settings.read().show_file_icons,
                        style: "
                            width: 16px;
                            height: 16px;
                            accent-color: var(--vscode-checkbox-background, #007acc);
                        ",
                        onchange: {
                            let mut icon_manager = icon_manager.clone();
                            move |evt: Event<FormData>| {
                                icon_manager.settings.write().show_file_icons = evt.checked();
                                icon_manager.save_settings();
                            }
                        }
                    }
                }
                
                div {
                    class: "settings-row",
                    style: "
                        display: flex;
                        justify-content: space-between;
                        align-items: center;
                        margin-bottom: 16px;
                    ",
                    
                    div {
                        span {
                            style: "font-size: 14px; color: var(--vscode-foreground, #cccccc);",
                            "Show folder icons"
                        }
                    }
                    
                    input {
                        r#type: "checkbox",
                        checked: icon_manager.settings.read().show_folder_icons,
                        style: "
                            width: 16px;
                            height: 16px;
                            accent-color: var(--vscode-checkbox-background, #007acc);
                        ",
                        onchange: {
                            let mut icon_manager = icon_manager.clone();
                            move |evt: Event<FormData>| {
                                icon_manager.settings.write().show_folder_icons = evt.checked();
                                icon_manager.save_settings();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Simple icon pack card component
#[component]
fn IconPackCard(
    name: String,
    description: String,
    pack: IconPack,
    is_current: bool,
    on_select: EventHandler<IconPack>,
) -> Element {
    let card_border = if is_current { "var(--vscode-button-background, #0e639c)" } else { "var(--vscode-widget-border, #454545)" };
    let card_background = if is_current { "rgba(14, 99, 156, 0.1)" } else { "var(--vscode-input-background, #3c3c3c)" };
    
    rsx! {
        div {
            class: "icon-pack-card",
            style: "
                padding: 20px;
                border: 2px solid {card_border};
                border-radius: 8px;
                background: {card_background};
                cursor: pointer;
                transition: all 0.2s ease;
                position: relative;
            ",
            onclick: move |_| {
                on_select.call(pack.clone());
            },

            // Current indicator
            if is_current {
                div {
                    style: "
                        position: absolute;
                        top: 12px;
                        right: 12px;
                        width: 20px;
                        height: 20px;
                        background: var(--vscode-button-background, #0e639c);
                        border-radius: 50%;
                        display: flex;
                        align-items: center;
                        justify-content: center;
                        font-size: 12px;
                        color: white;
                    ",
                    "✓"
                }
            }

            h3 {
                style: "
                    margin: 0 0 8px 0;
                    font-size: 16px;
                    font-weight: 600;
                    color: var(--vscode-foreground, #cccccc);
                ",
                "{name}"
            }

            p {
                style: "
                    margin: 0;
                    font-size: 14px;
                    color: var(--vscode-descriptionForeground, #a0a0a0);
                    line-height: 1.4;
                ",
                "{description}"
            }

            // Sample icons preview
            div {
                class: "icon-preview",
                style: "
                    margin-top: 16px;
                    display: flex;
                    gap: 8px;
                    flex-wrap: wrap;
                ",
                
                for file_type in ["folder", "js", "ts", "rs", "py", "css"] {
                    div {
                        style: "
                            width: 20px;
                            height: 20px;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            background: var(--vscode-badge-background, #616161);
                            border-radius: 3px;
                            font-size: 10px;
                            color: var(--vscode-badge-foreground, #ffffff);
                        ",
                        "{file_type}"
                    }
                }
            }
        }
    }
}