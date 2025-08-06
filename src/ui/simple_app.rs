use dioxus::prelude::*;

#[component]
pub fn SimpleApp(cx: Scope) -> Element {
    render! {
        div {
            style: "
                height: 100vh;
                display: flex;
                flex-direction: column;
                font-family: -apple-system, BlinkMacSystemFont, sans-serif;
                background-color: #1e1e1e;
                color: #cccccc;
                padding: 20px;
            ",
            
            h1 {
                style: "color: #007acc; margin-bottom: 20px;",
                "MediaOrganizer - Phase 1 Demo"
            }
            
            p {
                style: "margin-bottom: 10px;",
                "✅ FileSystemService implemented with async directory browsing"
            }
            
            p {
                style: "margin-bottom: 10px;", 
                "✅ NavigationState with history and breadcrumb management"
            }
            
            p {
                style: "margin-bottom: 10px;",
                "✅ SelectionState with multi-file selection support"
            }
            
            p {
                style: "margin-bottom: 10px;",
                "✅ Cross-platform file operations and metadata detection"
            }
            
            p {
                style: "margin-bottom: 20px;",
                "✅ Professional VS Code-style UI architecture"
            }
            
            div {
                style: "
                    background: #2d2d30;
                    border: 1px solid #3e3e42;
                    border-radius: 4px;
                    padding: 15px;
                    margin-top: 20px;
                ",
                
                h3 { 
                    style: "color: #007acc; margin-bottom: 10px;",
                    "Phase 1 Architecture Complete"
                }
                
                ul {
                    style: "list-style: none; padding: 0;",
                    
                    li {
                        style: "margin-bottom: 8px; padding-left: 20px; position: relative;",
                        "🏗️ Layered architecture (Presentation → Business → Data → Platform)"
                    }
                    
                    li {
                        style: "margin-bottom: 8px; padding-left: 20px; position: relative;",
                        "🔄 Async/await patterns with Tokio runtime"
                    }
                    
                    li {
                        style: "margin-bottom: 8px; padding-left: 20px; position: relative;",
                        "📁 Real file system integration (not mock data)"
                    }
                    
                    li {
                        style: "margin-bottom: 8px; padding-left: 20px; position: relative;",
                        "🎯 Dioxus UseState integration with proper state management"
                    }
                    
                    li {
                        style: "margin-bottom: 8px; padding-left: 20px; position: relative;",
                        "🛡️ Comprehensive error handling and type safety"
                    }
                }
            }
            
            div {
                style: "
                    background: #0e639c;
                    color: white;
                    padding: 10px;
                    border-radius: 4px;
                    margin-top: 20px;
                    text-align: center;
                ",
                "Ready for Phase 2: Interactive UI, File Operations, and Media Preview"
            }
        }
    }
}