use dioxus::prelude::*;

/// Empty state component for file tree when no folder is selected
#[component]
pub fn EmptyFileTree(on_folder_select: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "empty-file-tree",
            style: "
                padding: 40px 20px;
                text-align: center;
                color: var(--vscode-text-secondary, #999999);
                height: 100%;
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: 20px;
            ",
            
            // Icon
            div {
                style: "font-size: 48px; opacity: 0.6;",
                "📁"
            }
            
            // Title
            h3 {
                style: "margin: 0; font-size: 16px; color: var(--vscode-foreground, #cccccc);",
                "No Folder Selected"
            }
            
            // Description
            p {
                style: "margin: 0; font-size: 14px; max-width: 280px; line-height: 1.4;",
                "To start exploring files, please select a folder to open."
            }
            
            // Select Folder Button
            button {
                r#type: "button",
                onclick: move |_| on_folder_select.call(()),
                style: "
                    padding: 12px 24px;
                    background: var(--vscode-button-background, #0e639c);
                    color: var(--vscode-button-foreground, #ffffff);
                    border: none;
                    border-radius: 3px;
                    font-size: 13px;
                    cursor: pointer;
                ",
                
                "📂 Select Folder"
            }
        }
    }
}