// UI components module
// Contains reusable UI components for the MediaOrganizer application

pub mod virtual_scroll;
// pub mod virtual_file_tree; // Has type annotation issues - will fix later
// pub mod file_tree; // Temporarily disabled due to syntax error
// pub mod file_tree_simple; // Temporarily disabled due to syntax error
pub mod working_file_tree;
pub mod dialogs;
pub mod context_menu;
pub mod drag_drop;
pub mod settings_panel;
pub mod vscode_settings_dialog;
pub mod duplicate_manager;
pub mod command_palette;
pub mod shortcut_cheat_sheet;
pub mod preview_panel;
pub mod info_panel;
pub mod dynamic_content_panel;
pub mod empty_file_tree;
pub mod settings_dialog;
pub mod icon_pack_manager;
pub mod file_tree;

// Re-export only actively used types to reduce warnings
// pub use file_tree::{FileTree, FileTreeNode}; // Temporarily disabled
pub use working_file_tree::WorkingFileTree;
pub use dialogs::{
    ConfirmationDialog, ConfirmationResult,
    ProgressDialog
};
pub use context_menu::{
    ContextMenu,
    use_context_menu
};
pub use drag_drop::{
    DragPreview, DropZone,
    DragOperation,
    use_drag_drop, use_drop_zone
};
pub use settings_panel::{SettingsPanel};
pub use vscode_settings_dialog::{VsCodeSettingsDialog};
pub use command_palette::{CommandPalette};
pub use shortcut_cheat_sheet::{ShortcutCheatSheet};
pub use preview_panel::{PreviewPanel, PreviewHeader, PreviewMetadata, LazyPreviewContentArea};
pub use info_panel::{InfoPanel};
pub use dynamic_content_panel::{DynamicContentPanel, PanelTypeIndicator};
pub use empty_file_tree::{EmptyFileTree};
pub use settings_dialog::{SettingsDialog};
pub use icon_pack_manager::{IconPackManager};
pub use file_tree::{FileTree, FileTreeNode};
// Note: duplicate_manager exports are only used internally by phase2_app
// pub use duplicate_manager::{DuplicateManager};

// Note: Individual component files (simple_split_panel, simple_file_tree, simple_content_viewer)
// have been removed as Phase2App uses an integrated layout approach.
// This provides better performance and simpler maintenance.