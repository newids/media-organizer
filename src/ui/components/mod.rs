// UI components module
// Contains reusable UI components for the MediaOrganizer application

pub mod virtual_scroll;
// pub mod virtual_file_tree;
pub mod dialogs;
pub mod context_menu;
pub mod drag_drop;
pub mod settings_panel;
pub mod duplicate_manager;

// Re-export commonly used types for easier imports
pub use virtual_scroll::{VirtualScrollCalculator, VisibleRange, ScrollAlignment, PerformanceMetrics};
// pub use virtual_file_tree::{VirtualFileTree, VirtualFileTreeProps};
pub use dialogs::{
    ConfirmationDialog, ConfirmationDialogProps, ConfirmationAction, ConfirmationResult,
    ProgressDialog, ProgressDialogProps, ProgressDialogState,
    ToastContainer, ToastContainerProps, ToastNotification, ToastType, ToastManager,
    OperationSummaryDialog, OperationSummaryDialogProps, OperationSummary
};
pub use context_menu::{
    ContextMenu, ContextMenuProps, ContextMenuAction, ContextMenuState, MenuPosition,
    use_context_menu
};
pub use drag_drop::{
    DragPreview, DragPreviewProps, DropZone, DropZoneProps,
    DragState, DragOperation, DropZoneState,
    use_drag_drop, use_drop_zone, is_valid_drop_target
};
pub use settings_panel::{SettingsPanel, SettingsTab};
pub use duplicate_manager::{
    DuplicateManager, DuplicateManagerProps, DuplicateManagerState,
    DetectionSettings, DuplicateAction
};

// Note: Individual component files (simple_split_panel, simple_file_tree, simple_content_viewer)
// have been removed as Phase2App uses an integrated layout approach.
// This provides better performance and simpler maintenance.