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
pub use icons::{IconType};
pub use icon_packs::{IconPack, FileIconComponent};
pub use icon_manager::{IconManager, IconSettings, IconManagerProvider, use_icon_manager};
