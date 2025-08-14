pub mod app_state;
pub mod navigation;
pub mod persistence;

// Keeping modules available but imports commented out for clean Phase 2A build
// These will be uncommented when implementing Phase 2B interactive features
// pub use app_state::AppState;
// pub use navigation::{NavigationState, SelectionState, SelectionMode, BreadcrumbItem, NavigationError};

// Panel state persistence is active for Phase 2A
pub use persistence::{PanelState, save_panel_state_debounced, load_panel_state, flush_pending_saves, is_storage_available};