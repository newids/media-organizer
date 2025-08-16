pub mod app_state;
pub mod navigation;
pub mod persistence;
pub mod context;

// Centralized state management now active for Task 10.1
pub use app_state::{
    AppState, ViewMode, SearchState, SearchFilters, OperationState, Theme, SettingsState
};
pub use navigation::{NavigationState, SelectionState, SelectionMode, BreadcrumbItem, NavigationError};
pub use context::{
    AppStateProvider, init_app_state, use_app_state, try_use_app_state, use_file_entries,
    use_current_directory, use_view_mode, use_search_state, use_operation_state,
    use_selection_state
};

// Panel state persistence remains active
pub use persistence::{
    PanelState, save_panel_state_debounced, load_panel_state, flush_pending_saves, 
    is_storage_available, save_settings_debounced, load_settings, 
    flush_pending_settings_saves, clear_settings
};