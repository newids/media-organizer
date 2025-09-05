pub mod app_state;
pub mod navigation;
pub mod persistence;
pub mod context;
pub mod layout_manager;
pub mod performance;
pub mod signal_optimization;
pub mod batch_optimizer;
pub mod benchmarks;

#[cfg(test)]
pub mod tests;

// Centralized state management - only export actively used types
pub use app_state::{
    AppState, ViewMode, ActivityBarView, SidebarState, FileTreeState, SearchState, OperationState, Theme, SettingsState,
    EditorState, EditorGroup, EditorTab, TabType, PreviewType, EditorLayoutConfig, EditorGroupPosition, TabDragOperation,
    TabContextMenu, PanelTab, PanelState,
    // New unified layout state types
    LayoutState, ActivityBarPosition, SidebarPosition, SidebarContent, PanelPosition,
    // Command system types
    Command, CommandPaletteState, CommandHandler, SystemCommand, EditorCommand, FileCommand, NavigationCommand, ViewCommand
};
pub use layout_manager::LayoutManager;
pub use navigation::{SelectionState};
pub use context::{
    AppStateProvider, use_app_state, use_file_entries,
    use_selection_state, use_activity_bar_view, use_sidebar_state, use_file_tree_state, use_editor_state, use_panel_state
};

// Panel state persistence - only export actively used functions
pub use persistence::{
    save_panel_state_debounced, load_panel_state,
    save_settings_debounced, load_settings
};