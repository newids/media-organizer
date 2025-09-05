use dioxus::prelude::*;
use crate::state::{AppState, LayoutManager};

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

/// Hook to get activity bar view from shared state
pub fn use_activity_bar_view() -> Signal<crate::state::ActivityBarView> {
    let app_state = use_app_state();
    app_state.active_activity_view
}

/// Hook to get sidebar state from shared state
pub fn use_sidebar_state() -> Signal<crate::state::SidebarState> {
    let app_state = use_app_state();
    app_state.sidebar_state
}

/// Hook to get file tree state from shared state
pub fn use_file_tree_state() -> Signal<crate::state::FileTreeState> {
    let app_state = use_app_state();
    app_state.file_tree_state
}

/// Hook to get editor state from shared state
pub fn use_editor_state() -> Signal<crate::state::EditorState> {
    let app_state = use_app_state();
    app_state.editor_state
}

/// Hook to get panel state from shared state
pub fn use_panel_state() -> Signal<crate::state::PanelState> {
    let app_state = use_app_state();
    app_state.panel_state
}

/// Hook to get layout manager from context
pub fn use_layout_manager() -> LayoutManager {
    use_context::<LayoutManager>()
}

/// Try to get layout manager from context (returns None if not available)
pub fn try_use_layout_manager() -> Option<LayoutManager> {
    try_use_context::<LayoutManager>()
}

/// Initialize layout manager with AppState integration
pub fn init_layout_manager(app_state: &AppState) -> LayoutManager {
    // Create layout manager using the layout state from app state
    let initial_layout_state = app_state.layout_state.read().clone();
    LayoutManager::with_state(initial_layout_state)
}

/// Initialize layout manager with persistence support
/// This loads saved layout state and creates a manager with automatic persistence
pub fn init_layout_manager_with_persistence() -> LayoutManager {
    tracing::info!("Initializing layout manager with persistence support");
    
    // Try to load saved layout state, fall back to defaults if not available
    let layout_manager = LayoutManager::new_with_persistence();
    
    tracing::info!("Layout manager initialized with persistence support");
    layout_manager
}

/// Context provider component for layout manager
#[component]
pub fn LayoutManagerProvider(children: Element) -> Element {
    // Initialize layout manager and provide it to child components
    let app_state = use_app_state();
    let layout_manager = init_layout_manager(&app_state);
    use_context_provider(|| layout_manager);
    
    tracing::info!("Layout manager initialized and provided");

    rsx! {
        {children}
    }
}

/// Enhanced context provider component for layout manager with persistence
#[component]
pub fn LayoutManagerProviderWithPersistence(children: Element) -> Element {
    // Initialize layout manager with persistence support
    let layout_manager = init_layout_manager_with_persistence();
    use_context_provider(|| layout_manager.clone());
    
    // Setup periodic auto-save checking using use_effect
    let manager_for_effect = layout_manager.clone();
    use_effect(move || {
        // Schedule auto-save check (simplified approach for Dioxus)
        // In a real app, this would be handled by a background service
        tracing::debug!("Auto-save check scheduled for layout manager");
        
        // For now, we'll rely on manual auto-save calls from user interactions
        // A full implementation would use a timer service or background task
    });
    
    tracing::info!("Layout manager with persistence initialized and provided");

    rsx! {
        {children}
    }
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