use dioxus::prelude::*;

/// Menu bar component with File menu and other application menus
#[component]
pub fn MenuBar(on_open_folder: EventHandler<()>) -> Element {
    let mut file_menu_open = use_signal(|| false);

    rsx! {
        div {
            class: "menu-bar",
            style: "
                display: flex;
                background-color: var(--vscode-secondary-background);
                border-bottom: 1px solid var(--vscode-border);
                padding: 0;
                font-size: var(--vscode-font-size-small);
                height: 24px;
                user-select: none;
                z-index: 1000;
                position: relative;
            ",
            
            // File Menu
            div {
                class: "menu-item",
                style: "
                    position: relative;
                    display: inline-block;
                ",
                
                // File menu button
                button {
                    class: "menu-button",
                    style: "
                        background: transparent;
                        border: none;
                        color: var(--vscode-text-primary);
                        padding: 4px 8px;
                        cursor: pointer;
                        font-size: inherit;
                        font-family: inherit;
                        height: 24px;
                        display: flex;
                        align-items: center;
                    ",
                    onmousedown: move |_| {
                        let current_state = *file_menu_open.read();
                        file_menu_open.set(!current_state);
                    },
                    onmouseover: move |_| {
                        // Auto-open when hovering if any menu is already open
                        // This mimics native menu behavior
                    },
                    "File"
                }
                
                // File menu dropdown
                if file_menu_open.read().clone() {
                    div {
                        class: "menu-dropdown",
                        style: "
                            position: absolute;
                            top: 24px;
                            left: 0;
                            background-color: var(--vscode-secondary-background);
                            border: 1px solid var(--vscode-border);
                            box-shadow: var(--vscode-shadow-medium);
                            min-width: 160px;
                            z-index: 1001;
                        ",
                        
                        // Open Folder menu item
                        div {
                            class: "menu-dropdown-item",
                            style: "
                                padding: 8px 16px;
                                cursor: pointer;
                                color: var(--vscode-text-primary);
                                background-color: transparent;
                                transition: background-color var(--vscode-transition-fast);
                            ",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                on_open_folder.call(());
                            },
                            
                            span {
                                style: "display: flex; justify-content: space-between; align-items: center;",
                                span { "Open Folder..." }
                                span {
                                    style: "color: var(--vscode-text-secondary); font-size: var(--vscode-font-size-small); margin-left: 24px;",
                                    "Ctrl+O"
                                }
                            }
                        }
                        
                        // Separator
                        div {
                            style: "
                                height: 1px;
                                background-color: var(--vscode-border);
                                margin: 4px 0;
                            "
                        }
                        
                        // Additional menu items can be added here
                        div {
                            class: "menu-dropdown-item",
                            style: "
                                padding: 8px 16px;
                                cursor: pointer;
                                color: var(--vscode-text-secondary);
                                background-color: transparent;
                                transition: background-color var(--vscode-transition-fast);
                            ",
                            
                            span { "Recent Folders" }
                        }
                    }
                }
            }
        }
        
        // Click outside to close menus
        if file_menu_open.read().clone() {
            div {
                style: "
                    position: fixed;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    z-index: 999;
                ",
                onclick: move |_| {
                    file_menu_open.set(false);
                },
            }
        }
    }
}