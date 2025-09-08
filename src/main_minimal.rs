use dioxus::prelude::*;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    tracing::info!("Starting MediaOrganizer minimal version");
    
    // Launch minimal Dioxus app
    dioxus::launch(minimal_app);
}

#[component]
fn minimal_app() -> Element {
    let mut file_path = use_signal(|| String::from("No file selected"));
    
    rsx! {
        div {
            style: "
                width: 100vw;
                height: 100vh;
                display: flex;
                background: #1e1e1e;
                color: #ffffff;
                font-family: system-ui, -apple-system, 'Segoe UI', sans-serif;
            ",
            
            // Left panel - File tree placeholder
            div {
                style: "
                    width: 300px;
                    height: 100%;
                    background: #252526;
                    border-right: 1px solid #3e3e3e;
                    padding: 16px;
                ",
                h2 {
                    style: "
                        margin: 0 0 16px 0;
                        font-size: 16px;
                        font-weight: 600;
                    ",
                    "📁 File Explorer"
                }
                div {
                    style: "
                        padding: 8px;
                        background: #2d2d30;
                        border-radius: 4px;
                        cursor: pointer;
                        margin-bottom: 4px;
                    ",
                    onclick: move |_| {
                        file_path.set("Documents/example.txt".to_string());
                    },
                    "📄 example.txt"
                }
                div {
                    style: "
                        padding: 8px;
                        background: #2d2d30;
                        border-radius: 4px;
                        cursor: pointer;
                        margin-bottom: 4px;
                    ",
                    onclick: move |_| {
                        file_path.set("Images/photo.jpg".to_string());
                    },
                    "🖼️ photo.jpg"
                }
                div {
                    style: "
                        padding: 8px;
                        background: #2d2d30;
                        border-radius: 4px;
                        cursor: pointer;
                        margin-bottom: 4px;
                    ",
                    onclick: move |_| {
                        file_path.set("Videos/movie.mp4".to_string());
                    },
                    "🎥 movie.mp4"
                }
            }
            
            // Right panel - Preview area
            div {
                style: "
                    flex: 1;
                    height: 100%;
                    padding: 16px;
                    display: flex;
                    flex-direction: column;
                    align-items: center;
                    justify-content: center;
                ",
                h1 {
                    style: "
                        font-size: 32px;
                        font-weight: 700;
                        margin-bottom: 16px;
                        background: linear-gradient(135deg, #007acc, #00c9ff);
                        background-clip: text;
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                    ",
                    "🎯 MediaOrganizer"
                }
                p {
                    style: "
                        font-size: 18px;
                        margin-bottom: 8px;
                        color: #cccccc;
                    ",
                    "Cross-platform media management with Dioxus & Rust"
                }
                div {
                    style: "
                        padding: 16px;
                        background: #2d2d30;
                        border-radius: 8px;
                        border: 2px solid #007acc;
                        margin-top: 24px;
                    ",
                    p {
                        style: "margin: 0 0 8px 0; font-weight: 600;",
                        "Selected File:"
                    }
                    p {
                        style: "
                            margin: 0;
                            font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Courier New', monospace;
                            color: #4ec9b0;
                        ",
                        "{file_path}"
                    }
                }
                div {
                    style: "
                        margin-top: 24px;
                        padding: 16px;
                        background: #0e3a0e;
                        border-radius: 8px;
                        border: 1px solid #00aa00;
                    ",
                    "✅ Successfully running Dioxus 0.6 with updated signal patterns!"
                }
            }
        }
    }
}