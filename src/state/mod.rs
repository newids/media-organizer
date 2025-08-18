pub mod app_state;
pub mod navigation;
pub mod persistence;
pub mod context;

// Centralized state management - only export actively used types
pub use app_state::{
    AppState, ViewMode, SearchState, OperationState, Theme, SettingsState
};
pub use navigation::{SelectionState};
pub use context::{
    AppStateProvider, use_app_state, use_file_entries,
    use_selection_state
};

// Panel state persistence - only export actively used functions
pub use persistence::{
    save_panel_state_debounced, load_panel_state,
    save_settings_debounced, load_settings
};