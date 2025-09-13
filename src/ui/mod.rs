pub mod phase2_app;
pub mod components;
pub mod shortcuts;
pub mod shortcut_handler;
pub mod vscode_layout;
pub mod vscode_app;
pub mod icons;
pub mod icon_packs;
pub mod icon_manager;

pub use phase2_app::phase2_app;
pub use shortcut_handler::{use_shortcut_handler};
pub use vscode_layout::{VSCodeLayout, ActivityBar, Sidebar, EditorGroups, Panel, StatusBar};
pub use vscode_app::VSCodeApp;
pub use icons::{FileIconComponent, get_icon_for_file, IconType};
pub use icon_packs::{IconPack, get_icon_for_file as get_icon_for_file_with_pack};
pub use icon_manager::{IconManager, IconSettings, IconManagerProvider, use_icon_manager};
