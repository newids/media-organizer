// UI components module
// Contains reusable UI components for the MediaOrganizer application

pub mod virtual_scroll;
// pub mod virtual_file_tree;
pub mod dialogs;

// Re-export commonly used types for easier imports
pub use virtual_scroll::{VirtualScrollCalculator, VisibleRange, ScrollAlignment, PerformanceMetrics};
// pub use virtual_file_tree::{VirtualFileTree, VirtualFileTreeProps};
pub use dialogs::{
    ConfirmationDialog, ConfirmationDialogProps, ConfirmationAction, ConfirmationResult,
    ProgressDialog, ProgressDialogProps, ProgressDialogState,
    ToastContainer, ToastContainerProps, ToastNotification, ToastType, ToastManager,
    OperationSummaryDialog, OperationSummaryDialogProps, OperationSummary
};

// Note: Individual component files (simple_split_panel, simple_file_tree, simple_content_viewer)
// have been removed as Phase2App uses an integrated layout approach.
// This provides better performance and simpler maintenance.