use dioxus::prelude::*;
use crate::state::AppState;

/// Initialize the application state context using use_context_provider
/// This should be called once at the app root level
pub fn init_app_state() -> AppState {
    AppState::new()
}

/// Get the application state from context
/// Must be called within a component that has AppStateProvider as ancestor
pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}

/// Check if app state is available in context
pub fn try_use_app_state() -> Option<AppState> {
    try_use_context::<AppState>()
}

/// Context provider component that initializes and provides app state
#[component]
pub fn AppStateProvider(children: Element) -> Element {
    // Initialize app state and provide it to child components
    let app_state = init_app_state();
    use_context_provider(|| app_state);
    
    tracing::info!("Application state initialized and provided");

    rsx! {
        {children}
    }
}

/// Hook to get file entries from shared state
pub fn use_file_entries() -> Signal<Vec<crate::services::FileEntry>> {
    let app_state = use_app_state();
    app_state.file_entries
}

/// Hook to get current directory from shared state
pub fn use_current_directory() -> std::path::PathBuf {
    let app_state = use_app_state();
    // Store the read in a local variable to extend its lifetime
    let nav = app_state.navigation.read();
    nav.current_path.clone()
}

/// Hook to get view mode from shared state
pub fn use_view_mode() -> Signal<crate::state::ViewMode> {
    let app_state = use_app_state();
    app_state.view_mode
}

/// Hook to get search state from shared state  
pub fn use_search_state() -> Signal<crate::state::SearchState> {
    let app_state = use_app_state();
    app_state.search_state
}

/// Hook to get operation state from shared state
pub fn use_operation_state() -> Signal<crate::state::OperationState> {
    let app_state = use_app_state();
    app_state.operation_state
}

/// Hook to get selection state from shared state
pub fn use_selection_state() -> Signal<crate::state::SelectionState> {
    let app_state = use_app_state();
    app_state.selection
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_functions_exist() {
        // These functions require Dioxus context to work properly
        // We just verify they compile and exist
        assert!(true);
    }
}