use dioxus::prelude::*;
use crate::state::use_app_state;
use crate::ui::vscode_layout::VSCodeLayout;

/// New VS Code-like application component
/// This is the main entry point for the redesigned VS Code-style interface
#[component]
pub fn VSCodeApp() -> Element {
    // Get the existing app state from context
    let app_state = use_app_state();
    
    // Convert AppState to Signal for VSCodeLayout
    let app_state_signal = use_signal(|| app_state.clone());
    
    rsx! {
        style { {include_str!("../../assets/styles.css")} }
        
        // VS Code Layout with CSS Grid
        VSCodeLayout {
            app_state: app_state_signal
        }
    }
}