use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use crate::services::{FileEntry};
use crate::services::file_system::{FileSystemService, NativeFileSystemService};
use crate::state::navigation::{NavigationState, SelectionState};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unified layout state model that centralizes all UI layout and theme configuration
/// This struct consolidates panel states, sidebars, editor groups, and theme settings
/// for consistent state management and performance optimization
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutState {
    /// Theme and visual preferences
    pub theme: Theme,
    /// Activity bar configuration and active view
    pub activity_bar: ActivityBarLayout,
    /// Sidebar state and dimensions
    pub sidebar: SidebarLayout,
    /// Editor groups layout and configuration
    pub editor_layout: EditorLayoutState,
    /// Bottom panel state and configuration
    pub panel: PanelLayout,
    /// Window and viewport dimensions
    pub viewport: ViewportState,
    /// UI animation and transition preferences
    pub ui_preferences: UIPreferences,
    /// Layout persistence settings
    pub persistence: LayoutPersistenceSettings,
}

/// Activity bar layout configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActivityBarLayout {
    /// Currently active view in the activity bar
    pub active_view: ActivityBarView,
    /// Whether the activity bar is visible
    pub is_visible: bool,
    /// Position of the activity bar (left, right)
    pub position: ActivityBarPosition,
    /// Width of the activity bar in pixels
    pub width: f64,
}

/// Activity bar position options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActivityBarPosition {
    Left,
    Right,
}

/// Sidebar layout configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarLayout {
    /// Whether the sidebar is collapsed
    pub is_collapsed: bool,
    /// Current sidebar width in pixels
    pub width: f64,
    /// Minimum allowed width
    pub min_width: f64,
    /// Maximum allowed width
    pub max_width: f64,
    /// Whether the sidebar can be resized
    pub is_resizable: bool,
    /// Position of the sidebar (left, right)
    pub position: SidebarPosition,
    /// Active sidebar content
    pub active_content: SidebarContent,
}

/// Sidebar position options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SidebarPosition {
    Left,
    Right,
}

/// Sidebar content types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SidebarContent {
    FileTree,
    Search,
    Extensions,
    Settings,
}

/// Editor layout state with enhanced configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EditorLayoutState {
    /// Layout configuration for editor groups
    pub layout_config: EditorLayoutConfig,
    /// Currently active editor group index
    pub active_group: usize,
    /// Split ratios for multi-group layouts (0.0 to 1.0)
    pub split_ratios: Vec<f64>,
    /// Whether tabs are visible
    pub show_tabs: bool,
    /// Tab bar height in pixels
    pub tab_height: f64,
    /// Maximum number of visible tabs before scrolling
    pub max_visible_tabs: usize,
}

/// Bottom panel layout configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelLayout {
    /// Whether the panel is visible
    pub is_visible: bool,
    /// Height of the panel in pixels
    pub height: f64,
    /// Minimum allowed height
    pub min_height: f64,
    /// Maximum allowed height (as a fraction of viewport)
    pub max_height_fraction: f64,
    /// Currently active panel tab
    pub active_tab: PanelTab,
    /// Whether the panel is resizable
    pub is_resizable: bool,
    /// Position of the panel (bottom, top)
    pub position: PanelPosition,
}

/// Panel position options
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PanelPosition {
    Bottom,
    Top,
}

/// Viewport state for responsive layout calculations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViewportState {
    /// Window width in pixels
    pub width: f64,
    /// Window height in pixels
    pub height: f64,
    /// Device pixel ratio
    pub pixel_ratio: f64,
    /// Whether the window is maximized
    pub is_maximized: bool,
    /// Whether the window is in fullscreen mode
    pub is_fullscreen: bool,
}

/// UI preferences for animations and visual effects
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPreferences {
    /// Enable animations and transitions
    pub enable_animations: bool,
    /// Animation duration in milliseconds
    pub animation_duration: u32,
    /// Enable smooth scrolling
    pub smooth_scrolling: bool,
    /// Enable visual feedback for interactions
    pub visual_feedback: bool,
    /// Reduced motion preference for accessibility
    pub reduced_motion: bool,
}

/// Layout persistence configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutPersistenceSettings {
    /// Whether to persist layout state between sessions
    pub persist_layout: bool,
    /// Whether to persist theme selection
    pub persist_theme: bool,
    /// Whether to restore window size and position
    pub restore_window_state: bool,
    /// Auto-save interval in seconds (0 = disabled)
    pub auto_save_interval: u32,
}

/// Centralized application state with shared contexts for all UI components
#[derive(Clone)]
pub struct AppState {
    /// Unified layout state (NEW - consolidates panel, sidebar, editor, theme state)
    pub layout_state: Signal<LayoutState>,
    /// Navigation state (current directory, history, breadcrumbs)
    pub navigation: Signal<NavigationState>,
    /// File selection state (selected files, selection mode)
    pub selection: Signal<SelectionState>,
    /// Current directory file entries
    pub file_entries: Signal<Vec<FileEntry>>,
    /// Current view mode (grid, list, preview)
    pub view_mode: Signal<ViewMode>,
    /// Active Activity Bar view (Explorer, Search, etc.) - DEPRECATED: Use layout_state.activity_bar
    pub active_activity_view: Signal<ActivityBarView>,
    /// Sidebar state (collapsed, width, etc.) - DEPRECATED: Use layout_state.sidebar
    pub sidebar_state: Signal<SidebarState>,
    /// File tree state (expansion, children, etc.)
    pub file_tree_state: Signal<FileTreeState>,
    /// Search and filter state
    pub search_state: Signal<SearchState>,
    /// Operation progress and status
    pub operation_state: Signal<OperationState>,
    /// Editor groups and tabs management
    pub editor_state: Signal<EditorState>,
    /// Bottom panel state (problems, output, terminal) - DEPRECATED: Use layout_state.panel
    pub panel_state: Signal<PanelState>,
    /// Application settings and preferences - DEPRECATED: Use layout_state for layout-related settings
    pub settings: Signal<SettingsState>,
    /// Command registry and keyboard shortcuts
    pub command_registry: Signal<CommandRegistry>,
    /// Shortcut cheat sheet state (visibility)
    pub cheat_sheet_visible: Signal<bool>,
    /// File system service for operations
    pub file_service: Arc<dyn FileSystemService>,
}

/// View mode options for file display
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ViewMode {
    Grid,
    List,
    Preview,
}

/// Activity Bar navigation views
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActivityBarView {
    Explorer,
    Search,
    SourceControl,
    Debug,
    Extensions,
    Settings,
}

/// Sidebar state and configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarState {
    /// Whether the sidebar is collapsed
    pub is_collapsed: bool,
    /// Current sidebar width in pixels
    pub width: f64,
    /// Minimum allowed width
    pub min_width: f64,
    /// Maximum allowed width
    pub max_width: f64,
    /// Whether the sidebar can be resized
    pub is_resizable: bool,
}

/// File tree state and directory expansion tracking
#[derive(Clone, Debug, Default)]
pub struct FileTreeState {
    /// Map of directory paths to their expanded state
    pub expanded_directories: std::collections::HashMap<PathBuf, bool>,
    /// Map of directory paths to their loaded children
    pub directory_children: std::collections::HashMap<PathBuf, Vec<FileEntry>>,
    /// Currently loading directories
    pub loading_directories: std::collections::HashSet<PathBuf>,
    /// Directories that failed to load with error messages
    pub error_directories: std::collections::HashMap<PathBuf, String>,
    /// Root directory for the file tree
    pub root_directory: Option<PathBuf>,
    /// Currently selected file/directory
    pub selected_path: Option<PathBuf>,
}

impl Default for ActivityBarView {
    fn default() -> Self {
        ActivityBarView::Explorer
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            activity_bar: ActivityBarLayout::default(),
            sidebar: SidebarLayout::default(),
            editor_layout: EditorLayoutState::default(),
            panel: PanelLayout::default(),
            viewport: ViewportState::default(),
            ui_preferences: UIPreferences::default(),
            persistence: LayoutPersistenceSettings::default(),
        }
    }
}

impl Default for ActivityBarLayout {
    fn default() -> Self {
        Self {
            active_view: ActivityBarView::default(),
            is_visible: true,
            position: ActivityBarPosition::Left,
            width: 50.0,
        }
    }
}

impl Default for SidebarLayout {
    fn default() -> Self {
        Self {
            is_collapsed: false,
            width: 240.0,
            min_width: 200.0,
            max_width: 400.0,
            is_resizable: true,
            position: SidebarPosition::Left,
            active_content: SidebarContent::FileTree,
        }
    }
}

impl Default for EditorLayoutState {
    fn default() -> Self {
        Self {
            layout_config: EditorLayoutConfig::Single,
            active_group: 0,
            split_ratios: vec![1.0],
            show_tabs: true,
            tab_height: 35.0,
            max_visible_tabs: 20,
        }
    }
}

impl Default for PanelLayout {
    fn default() -> Self {
        Self {
            is_visible: true,
            height: 200.0,
            min_height: 150.0,
            max_height_fraction: 0.5,
            active_tab: PanelTab::Problems,
            is_resizable: true,
            position: PanelPosition::Bottom,
        }
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            pixel_ratio: 1.0,
            is_maximized: false,
            is_fullscreen: false,
        }
    }
}

impl Default for UIPreferences {
    fn default() -> Self {
        Self {
            enable_animations: true,
            animation_duration: 250,
            smooth_scrolling: true,
            visual_feedback: true,
            reduced_motion: false,
        }
    }
}

impl Default for LayoutPersistenceSettings {
    fn default() -> Self {
        Self {
            persist_layout: true,
            persist_theme: true,
            restore_window_state: true,
            auto_save_interval: 300, // 5 minutes
        }
    }
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            is_collapsed: false,
            width: 240.0,
            min_width: 200.0,
            max_width: 400.0,
            is_resizable: true,
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::Grid
    }
}

/// Theme configuration for the application
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    Auto, // Follows system preference
    HighContrast, // High contrast mode for accessibility
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Auto => "auto",
            Theme::HighContrast => "high-contrast",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            "auto" => Theme::Auto,
            "high-contrast" => Theme::HighContrast,
            _ => Theme::Dark,
        }
    }
}

/// Application settings and preferences
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SettingsState {
    /// Current theme selection
    pub theme: Theme,
    /// Panel width preferences
    pub default_panel_width: f64,
    /// Default view mode for new directories
    pub default_view_mode: ViewMode,
    /// Whether to show hidden files by default
    pub show_hidden_files: bool,
    /// Whether to remember last directory on startup
    pub remember_last_directory: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Enable animations and transitions
    pub enable_animations: bool,
    /// Custom CSS variables override (advanced users)
    pub custom_css_variables: std::collections::HashMap<String, String>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            default_panel_width: 300.0,
            default_view_mode: ViewMode::default(),
            show_hidden_files: false,
            remember_last_directory: true,
            auto_save_interval: 300, // 5 minutes
            enable_animations: true,
            custom_css_variables: std::collections::HashMap::new(),
        }
    }
}

/// Search and filter state
#[derive(Clone, Debug, Default)]
pub struct SearchState {
    pub query: String,
    pub is_active: bool,
    pub results: Vec<FileEntry>,
    pub filters: SearchFilters,
}

/// Search filters configuration
#[derive(Clone, Debug, Default)]
pub struct SearchFilters {
    pub file_types: Vec<String>,
    pub size_range: Option<(u64, u64)>,
    pub modified_range: Option<(std::time::SystemTime, std::time::SystemTime)>,
    pub include_hidden: bool,
}

/// Operation progress and status tracking
#[derive(Clone, Debug, Default)]
pub struct OperationState {
    pub is_active: bool,
    pub operation_type: Option<String>,
    pub progress: f32,
    pub status_message: String,
    pub can_cancel: bool,
}

/// Bottom panel state for terminal, problems, output, etc.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PanelState {
    /// Whether the panel is visible
    pub is_visible: bool,
    /// Height of the panel in pixels
    pub height: f64,
    /// Minimum allowed height
    pub min_height: f64,
    /// Maximum allowed height (as a fraction of viewport)
    pub max_height_fraction: f64,
    /// Currently active panel tab
    pub active_tab: PanelTab,
    /// Whether the panel is resizable
    pub is_resizable: bool,
}

/// Available panel tabs
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PanelTab {
    Problems,
    Output,
    Terminal,
    Debug,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            is_visible: true,
            height: 200.0,
            min_height: 150.0,
            max_height_fraction: 0.5, // 50% of viewport height
            active_tab: PanelTab::Problems,
            is_resizable: true,
        }
    }
}

impl Default for PanelTab {
    fn default() -> Self {
        PanelTab::Problems
    }
}

/// Editor state for managing tabbed editor groups
#[derive(Clone, Debug)]
pub struct EditorState {
    /// All editor groups in the interface
    pub editor_groups: Vec<EditorGroup>,
    /// Currently active editor group index
    pub active_group: usize,
    /// Layout configuration for split view
    pub layout_config: EditorLayoutConfig,
    /// Next unique ID for tabs
    pub next_tab_id: usize,
    /// Current drag operation state
    pub drag_operation: Option<TabDragOperation>,
    /// Context menu state
    pub context_menu: Option<TabContextMenu>,
}

/// Information about a tab being dragged
#[derive(Clone, Debug, PartialEq)]
pub struct TabDragOperation {
    /// ID of the tab being dragged
    pub tab_id: usize,
    /// Index of the tab within its group
    pub tab_index: usize,
    /// ID of the source editor group
    pub source_group_id: usize,
}

/// Context menu state for tabs
#[derive(Clone, Debug, PartialEq)]
pub struct TabContextMenu {
    /// Tab ID that triggered the context menu
    pub tab_id: usize,
    /// Tab index within its group
    pub tab_index: usize,
    /// Group index containing the tab
    pub group_index: usize,
    /// X coordinate of the menu position
    pub x: f64,
    /// Y coordinate of the menu position
    pub y: f64,
    /// Whether the menu is visible
    pub is_visible: bool,
}

/// Command registry for managing application commands and keyboard shortcuts
#[derive(Clone, Debug, PartialEq)]
pub struct CommandRegistry {
    /// Map of command ID to command definition
    pub commands: HashMap<String, Command>,
    /// Map of keyboard shortcut to command ID
    pub shortcuts: HashMap<String, String>,
    /// Map of category name to list of command IDs
    pub categories: HashMap<String, Vec<String>>,
    /// Command palette visibility and search state
    pub palette_state: CommandPaletteState,
}

/// Individual command definition
#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    /// Unique identifier for the command
    pub id: String,
    /// Display title for the command
    pub title: String,
    /// Optional description for the command
    pub description: Option<String>,
    /// Category this command belongs to
    pub category: String,
    /// Keyboard shortcuts for this command
    pub shortcuts: Vec<String>,
    /// Whether the command is currently enabled
    pub enabled: bool,
    /// Command execution handler type
    pub handler: CommandHandler,
}

/// Types of command handlers supported
#[derive(Clone, Debug, PartialEq)]
pub enum CommandHandler {
    /// Built-in system command
    System(SystemCommand),
    /// Editor operation command
    Editor(EditorCommand),
    /// File operation command
    File(FileCommand),
    /// Navigation command
    Navigation(NavigationCommand),
    /// View manipulation command
    View(ViewCommand),
}

/// System-level commands
#[derive(Clone, Debug, PartialEq)]
pub enum SystemCommand {
    /// Show command palette
    ShowCommandPalette,
    /// Show keyboard shortcuts
    ShowKeyboardShortcuts,
    /// Toggle settings panel
    ToggleSettings,
    /// Quit application
    Quit,
}

/// Editor-related commands
#[derive(Clone, Debug, PartialEq)]
pub enum EditorCommand {
    /// Close current tab
    CloseTab,
    /// Close all tabs
    CloseAllTabs,
    /// Close other tabs
    CloseOtherTabs,
    /// Close tabs to the right
    CloseTabsToRight,
    /// Pin/unpin current tab
    ToggleTabPin,
    /// Split editor horizontally
    SplitHorizontal,
    /// Split editor vertically
    SplitVertical,
    /// Focus next editor group
    FocusNextGroup,
    /// Focus previous editor group
    FocusPreviousGroup,
}

/// File operation commands
#[derive(Clone, Debug, PartialEq)]
pub enum FileCommand {
    /// Open file
    OpenFile,
    /// New file
    NewFile,
    /// Save file
    SaveFile,
    /// Save all files
    SaveAllFiles,
    /// Rename file
    RenameFile,
    /// Delete file
    DeleteFile,
    /// Copy file
    CopyFile,
    /// Cut file
    CutFile,
    /// Paste file
    PasteFile,
}

/// Navigation commands
#[derive(Clone, Debug, PartialEq)]
pub enum NavigationCommand {
    /// Navigate to parent directory
    NavigateUp,
    /// Navigate back in history
    NavigateBack,
    /// Navigate forward in history
    NavigateForward,
    /// Go to home directory
    GoToHome,
    /// Toggle file tree
    ToggleFileTree,
}

/// View manipulation commands
#[derive(Clone, Debug, PartialEq)]
pub enum ViewCommand {
    /// Toggle between view modes
    ToggleViewMode,
    /// Switch to grid view
    GridView,
    /// Switch to list view
    ListView,
    /// Switch to preview view
    PreviewView,
    /// Toggle sidebar
    ToggleSidebar,
    /// Toggle bottom panel
    TogglePanel,
    /// Zoom in
    ZoomIn,
    /// Zoom out
    ZoomOut,
    /// Reset zoom
    ResetZoom,
}

/// Command palette state and configuration
#[derive(Clone, Debug, PartialEq)]
pub struct CommandPaletteState {
    /// Whether the command palette is visible
    pub is_visible: bool,
    /// Current search query in the palette
    pub search_query: String,
    /// Currently selected command index
    pub selected_index: usize,
    /// Filtered commands based on search
    pub filtered_commands: Vec<String>,
    /// Maximum number of commands to display
    pub max_results: usize,
}

/// Individual editor group containing multiple tabs
#[derive(Clone, Debug, PartialEq)]
pub struct EditorGroup {
    /// Unique identifier for this group
    pub id: usize,
    /// All tabs in this group
    pub tabs: Vec<EditorTab>,
    /// Currently active tab index within this group
    pub active_tab: usize,
    /// Position and size within the layout
    pub layout_position: EditorGroupPosition,
}

/// Individual editor tab
#[derive(Clone, Debug, PartialEq)]
pub struct EditorTab {
    /// Unique identifier for this tab
    pub id: usize,
    /// Display title of the tab
    pub title: String,
    /// File path associated with this tab (if any)
    pub file_path: Option<PathBuf>,
    /// Tab type and content
    pub tab_type: TabType,
    /// Whether the tab has unsaved changes
    pub is_dirty: bool,
    /// Whether the tab is pinned
    pub is_pinned: bool,
    /// Whether the tab is currently active
    pub is_active: bool,
}

/// Types of tabs that can be opened
#[derive(Clone, Debug, PartialEq)]
pub enum TabType {
    /// Welcome screen
    Welcome,
    /// File editor for text/code files
    FileEditor { content: String },
    /// Preview tab for media files
    Preview { preview_type: PreviewType },
    /// Settings panel
    Settings,
    /// Search results
    SearchResults,
}

/// Types of previews that can be displayed
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewType {
    Image,
    Video,
    Audio,
    Pdf,
    Archive,
    Unknown,
}

/// Position and layout configuration for editor groups
#[derive(Clone, Debug, PartialEq)]
pub struct EditorGroupPosition {
    /// X position as a fraction of available space (0.0 to 1.0)
    pub x: f32,
    /// Y position as a fraction of available space (0.0 to 1.0)
    pub y: f32,
    /// Width as a fraction of available space (0.0 to 1.0)
    pub width: f32,
    /// Height as a fraction of available space (0.0 to 1.0)
    pub height: f32,
}

/// Layout configuration for editor groups
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorLayoutConfig {
    /// Single editor group taking full space
    Single,
    /// Two groups split horizontally
    SplitHorizontal,
    /// Two groups split vertically
    SplitVertical,
    /// Grid layout with multiple groups
    Grid { rows: usize, cols: usize },
}

impl Default for EditorState {
    fn default() -> Self {
        // Create initial welcome tab
        let welcome_tab = EditorTab {
            id: 1,
            title: "Welcome".to_string(),
            file_path: None,
            tab_type: TabType::Welcome,
            is_dirty: false,
            is_pinned: false,
            is_active: true,
        };

        // Create initial editor group
        let initial_group = EditorGroup {
            id: 1,
            tabs: vec![welcome_tab],
            active_tab: 0,
            layout_position: EditorGroupPosition {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        };

        Self {
            editor_groups: vec![initial_group],
            active_group: 0,
            layout_config: EditorLayoutConfig::Single,
            next_tab_id: 2, // Start from 2 since welcome tab uses 1
            drag_operation: None,
            context_menu: None,
        }
    }
}

impl Default for EditorGroupPosition {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        let mut registry = Self {
            commands: HashMap::new(),
            shortcuts: HashMap::new(),
            categories: HashMap::new(),
            palette_state: CommandPaletteState::default(),
        };
        
        // Register default commands
        registry.register_default_commands();
        registry
    }
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self {
            is_visible: false,
            search_query: String::new(),
            selected_index: 0,
            filtered_commands: Vec::new(),
            max_results: 10,
        }
    }
}

impl AppState {
    /// Create new AppState with initial signals
    /// Note: This must be called within a Dioxus component context
    pub fn new() -> Self {
        // Use default home directory for normal operation
        let initial_path = dirs::home_dir();
        
        Self {
            layout_state: use_signal(LayoutState::default),
            navigation: use_signal(|| NavigationState::new(initial_path)),
            selection: use_signal(|| SelectionState::new()),
            file_entries: use_signal(Vec::new),
            view_mode: use_signal(ViewMode::default),
            active_activity_view: use_signal(ActivityBarView::default),
            sidebar_state: use_signal(SidebarState::default),
            file_tree_state: use_signal(FileTreeState::default),
            search_state: use_signal(SearchState::default),
            operation_state: use_signal(OperationState::default),
            editor_state: use_signal(EditorState::default),
            panel_state: use_signal(PanelState::default),
            settings: use_signal(SettingsState::default),
            command_registry: use_signal(CommandRegistry::default),
            cheat_sheet_visible: use_signal(|| false),
            file_service: Arc::new(NativeFileSystemService::new()),
        }
    }
    
    pub async fn navigate_to(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Set loading state
        self.navigation.write().set_loading(path.clone(), true);
        
        // Load directory contents
        match self.file_service.list_directory(&path).await {
            Ok(contents) => {
                // Update navigation state
                {
                    let mut nav = self.navigation.write();
                    if let Err(e) = nav.navigate_to(path.clone()) {
                        tracing::warn!("Navigation error: {}", e);
                    }
                    nav.set_directory_contents(path.clone(), contents.clone());
                }
                
                // Update file entries in shared state
                self.file_entries.set(contents);
                
                // Clear selection when navigating
                self.selection.write().clear_selection();
                
                // Clear search when navigating
                self.search_state.write().query.clear();
                self.search_state.write().is_active = false;
                
                Ok(())
            }
            Err(e) => {
                self.navigation.write().set_loading(path, false);
                Err(Box::new(e))
            }
        }
    }
    
    pub async fn refresh_current_directory(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let current_path = self.navigation.read().current_path.clone();
        self.load_directory_contents(current_path).await
    }
    
    pub async fn load_directory_contents(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.navigation.write().set_loading(path.clone(), true);
        
        match self.file_service.list_directory(&path).await {
            Ok(contents) => {
                self.navigation.write().set_directory_contents(path.clone(), contents.clone());
                // Update shared file entries if this is the current directory
                let current_path = self.navigation.read().current_path.clone();
                if path == current_path {
                    self.file_entries.set(contents);
                }
                Ok(())
            }
            Err(e) => {
                self.navigation.write().set_loading(path, false);
                Err(Box::new(e))
            }
        }
    }
    
    pub async fn navigate_back(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path_opt = self.navigation.write().navigate_back();
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.read().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.write().clear_selection();
        }
        Ok(())
    }
    
    pub async fn navigate_forward(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path_opt = self.navigation.write().navigate_forward();
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.read().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.write().clear_selection();
        }
        Ok(())
    }
    
    pub async fn navigate_up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let path_opt = self.navigation.write().navigate_up();
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.read().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.write().clear_selection();
        }
        Ok(())
    }
    
    pub fn get_current_directory_contents(&self) -> Option<Vec<FileEntry>> {
        let nav = self.navigation.read();
        nav.get_directory_contents(&nav.current_path).cloned()
    }
    
    pub fn is_current_directory_loading(&self) -> bool {
        let nav = self.navigation.read();
        nav.is_loading(&nav.current_path)
    }
    
    pub fn get_current_path(&self) -> PathBuf {
        self.navigation.read().current_path.clone()
    }
    
    pub fn get_breadcrumbs(&self) -> Vec<crate::state::navigation::BreadcrumbItem> {
        self.navigation.read().breadcrumbs.clone()
    }
    
    pub fn can_navigate_back(&self) -> bool {
        self.navigation.read().can_navigate_back()
    }
    
    pub fn can_navigate_forward(&self) -> bool {
        self.navigation.read().can_navigate_forward()
    }
    
    pub fn can_navigate_up(&self) -> bool {
        self.navigation.read().can_navigate_up()
    }
    
    pub fn select_files(&mut self, paths: Vec<PathBuf>, mode: crate::state::navigation::SelectionMode) {
        self.selection.write().select_files(paths, mode);
    }
    
    pub fn clear_selection(&mut self) {
        self.selection.write().clear_selection();
    }
    
    pub fn is_selected(&self, path: &PathBuf) -> bool {
        self.selection.read().is_selected(path)
    }
    
    pub fn get_selected_files(&self) -> Vec<PathBuf> {
        self.selection.read().get_selected_paths()
    }
    
    pub fn get_selection_count(&self) -> usize {
        self.selection.read().selection_count()
    }
    
    pub fn get_selection_metadata(&self) -> crate::state::navigation::SelectionMetadata {
        self.selection.read().selection_metadata.clone()
    }
    
    // Direct signal access for expanded functionality
    // Components can access signals directly for reading and writing
    
    /// Get current file entries from shared state
    pub fn get_file_entries(&self) -> Vec<FileEntry> {
        self.file_entries.read().clone()
    }
    
    /// Get current view mode
    pub fn get_view_mode(&self) -> ViewMode {
        self.view_mode.read().clone()
    }
    
    /// Get search state
    pub fn get_search_state(&self) -> SearchState {
        self.search_state.read().clone()
    }
    
    /// Get operation state
    pub fn get_operation_state(&self) -> OperationState {
        self.operation_state.read().clone()
    }
    
    /// Check if any operation is active
    pub fn is_operation_active(&self) -> bool {
        self.operation_state.read().is_active
    }
    
    // Settings management methods
    
    /// Get current settings state
    pub fn get_settings(&self) -> SettingsState {
        self.settings.read().clone()
    }
    
    /// Get current theme
    pub fn get_current_theme(&self) -> Theme {
        self.settings.read().theme.clone()
    }
    
    /// Update theme setting
    pub fn set_theme(&mut self, theme: Theme) {
        self.settings.write().theme = theme;
    }
    
    /// Update panel width preference
    pub fn set_default_panel_width(&mut self, width: f64) {
        self.settings.write().default_panel_width = width;
    }
    
    /// Update view mode preference
    pub fn set_default_view_mode(&mut self, view_mode: ViewMode) {
        self.settings.write().default_view_mode = view_mode;
    }
    
    /// Toggle hidden files visibility
    pub fn toggle_show_hidden_files(&mut self) {
        let current_value = self.settings.read().show_hidden_files;
        self.settings.write().show_hidden_files = !current_value;
    }
    
    /// Update settings from external configuration
    pub fn update_settings(&mut self, new_settings: SettingsState) {
        self.settings.set(new_settings);
    }
    
    /// Check if animations are enabled
    pub fn are_animations_enabled(&self) -> bool {
        self.settings.read().enable_animations
    }
    
    /// Update animations preference
    pub fn set_animations_enabled(&mut self, enabled: bool) {
        self.settings.write().enable_animations = enabled;
    }
    
    // Layout state management methods
    
    /// Get current layout state
    pub fn get_layout_state(&self) -> LayoutState {
        self.layout_state.read().clone()
    }
    
    /// Get current theme from layout state
    pub fn get_current_theme_from_layout(&self) -> Theme {
        self.layout_state.read().theme.clone()
    }
    
    /// Update theme in layout state
    pub fn set_theme_in_layout(&mut self, theme: Theme) {
        self.layout_state.write().theme = theme;
    }
    
    /// Toggle sidebar collapse state
    pub fn toggle_sidebar_collapse(&mut self) {
        let current_state = self.layout_state.read().sidebar.is_collapsed;
        self.layout_state.write().sidebar.is_collapsed = !current_state;
    }
    
    /// Update sidebar width
    pub fn set_sidebar_width(&mut self, width: f64) {
        let mut layout = self.layout_state.write();
        layout.sidebar.width = width.max(layout.sidebar.min_width).min(layout.sidebar.max_width);
    }
    
    /// Toggle panel visibility
    pub fn toggle_panel_visibility(&mut self) {
        let current_state = self.layout_state.read().panel.is_visible;
        self.layout_state.write().panel.is_visible = !current_state;
    }
    
    /// Update panel height
    pub fn set_panel_height(&mut self, height: f64) {
        let mut layout = self.layout_state.write();
        layout.panel.height = height.max(layout.panel.min_height);
    }
    
    /// Update viewport dimensions
    pub fn set_viewport_dimensions(&mut self, width: f64, height: f64) {
        let mut layout = self.layout_state.write();
        layout.viewport.width = width;
        layout.viewport.height = height;
    }
    
    /// Toggle animations in UI preferences
    pub fn toggle_animations(&mut self) {
        let current_state = self.layout_state.read().ui_preferences.enable_animations;
        self.layout_state.write().ui_preferences.enable_animations = !current_state;
    }
    
    /// Check if animations are enabled from layout state
    pub fn are_animations_enabled_from_layout(&self) -> bool {
        self.layout_state.read().ui_preferences.enable_animations
    }
}

impl CommandRegistry {
    /// Register a new command in the registry
    pub fn register_command(&mut self, command: Command) {
        // Add to category
        self.categories.entry(command.category.clone())
            .or_insert_with(Vec::new)
            .push(command.id.clone());
        
        // Register keyboard shortcuts
        for shortcut in &command.shortcuts {
            self.shortcuts.insert(shortcut.clone(), command.id.clone());
        }
        
        // Store the command
        self.commands.insert(command.id.clone(), command);
    }
    
    /// Get a command by its ID
    pub fn get_command(&self, command_id: &str) -> Option<&Command> {
        self.commands.get(command_id)
    }
    
    /// Get command ID by keyboard shortcut
    pub fn get_command_by_shortcut(&self, shortcut: &str) -> Option<&Command> {
        self.shortcuts.get(shortcut)
            .and_then(|id| self.commands.get(id))
    }
    
    /// Get all commands in a category
    pub fn get_commands_by_category(&self, category: &str) -> Vec<&Command> {
        self.categories.get(category)
            .map(|ids| ids.iter().filter_map(|id| self.commands.get(id)).collect())
            .unwrap_or_default()
    }
    
    /// Search commands by title or description
    pub fn search_commands(&self, query: &str) -> Vec<&Command> {
        let query_lower = query.to_lowercase();
        self.commands.values()
            .filter(|cmd| {
                cmd.enabled && (
                    cmd.title.to_lowercase().contains(&query_lower) ||
                    cmd.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower)) ||
                    cmd.category.to_lowercase().contains(&query_lower)
                )
            })
            .collect()
    }
    
    /// Update command palette state
    pub fn update_palette_search(&mut self, query: String) {
        self.palette_state.search_query = query.clone();
        self.palette_state.selected_index = 0;
        
        // Filter commands based on search
        if query.is_empty() {
            // Show all enabled commands when no search query
            self.palette_state.filtered_commands = self.commands.keys()
                .filter(|id| self.commands.get(*id).map_or(false, |cmd| cmd.enabled))
                .cloned()
                .collect();
        } else {
            // Filter by search query
            self.palette_state.filtered_commands = self.search_commands(&query)
                .into_iter()
                .map(|cmd| cmd.id.clone())
                .take(self.palette_state.max_results)
                .collect();
        }
    }
    
    /// Toggle command palette visibility
    pub fn toggle_command_palette(&mut self) {
        self.palette_state.is_visible = !self.palette_state.is_visible;
        if self.palette_state.is_visible {
            self.update_palette_search(String::new());
        }
    }
    
    /// Select next command in palette
    pub fn select_next_command(&mut self) {
        if !self.palette_state.filtered_commands.is_empty() {
            self.palette_state.selected_index = 
                (self.palette_state.selected_index + 1) % self.palette_state.filtered_commands.len();
        }
    }
    
    /// Select previous command in palette
    pub fn select_previous_command(&mut self) {
        if !self.palette_state.filtered_commands.is_empty() {
            if self.palette_state.selected_index > 0 {
                self.palette_state.selected_index -= 1;
            } else {
                self.palette_state.selected_index = self.palette_state.filtered_commands.len() - 1;
            }
        }
    }
    
    /// Get currently selected command in palette
    pub fn get_selected_command(&self) -> Option<&Command> {
        self.palette_state.filtered_commands
            .get(self.palette_state.selected_index)
            .and_then(|id| self.commands.get(id))
    }
    
    /// Register all default commands
    pub fn register_default_commands(&mut self) {
        // System commands
        self.register_command(Command {
            id: "system.show_command_palette".to_string(),
            title: "Show Command Palette".to_string(),
            description: Some("Open the command palette to search and execute commands".to_string()),
            category: "System".to_string(),
            shortcuts: vec!["Ctrl+Shift+P".to_string(), "Cmd+Shift+P".to_string()],
            enabled: true,
            handler: CommandHandler::System(SystemCommand::ShowCommandPalette),
        });
        
        self.register_command(Command {
            id: "system.show_keyboard_shortcuts".to_string(),
            title: "Show Keyboard Shortcuts".to_string(),
            description: Some("Display a list of all keyboard shortcuts".to_string()),
            category: "System".to_string(),
            shortcuts: vec!["Ctrl+K Ctrl+S".to_string(), "Cmd+K Cmd+S".to_string()],
            enabled: true,
            handler: CommandHandler::System(SystemCommand::ShowKeyboardShortcuts),
        });
        
        // Editor commands
        self.register_command(Command {
            id: "editor.close_tab".to_string(),
            title: "Close Tab".to_string(),
            description: Some("Close the currently active tab".to_string()),
            category: "Editor".to_string(),
            shortcuts: vec!["Ctrl+W".to_string(), "Cmd+W".to_string()],
            enabled: true,
            handler: CommandHandler::Editor(EditorCommand::CloseTab),
        });
        
        self.register_command(Command {
            id: "editor.toggle_tab_pin".to_string(),
            title: "Toggle Tab Pin".to_string(),
            description: Some("Pin or unpin the current tab".to_string()),
            category: "Editor".to_string(),
            shortcuts: vec!["Ctrl+P".to_string(), "Cmd+P".to_string()],
            enabled: true,
            handler: CommandHandler::Editor(EditorCommand::ToggleTabPin),
        });
        
        // Navigation commands
        self.register_command(Command {
            id: "navigation.navigate_up".to_string(),
            title: "Navigate Up".to_string(),
            description: Some("Go to parent directory".to_string()),
            category: "Navigation".to_string(),
            shortcuts: vec!["Alt+Up".to_string(), "Cmd+Up".to_string()],
            enabled: true,
            handler: CommandHandler::Navigation(NavigationCommand::NavigateUp),
        });
        
        self.register_command(Command {
            id: "navigation.navigate_back".to_string(),
            title: "Navigate Back".to_string(),
            description: Some("Go back in navigation history".to_string()),
            category: "Navigation".to_string(),
            shortcuts: vec!["Alt+Left".to_string(), "Cmd+Left".to_string()],
            enabled: true,
            handler: CommandHandler::Navigation(NavigationCommand::NavigateBack),
        });
        
        self.register_command(Command {
            id: "navigation.toggle_file_tree".to_string(),
            title: "Toggle File Tree".to_string(),
            description: Some("Show or hide the file tree sidebar".to_string()),
            category: "Navigation".to_string(),
            shortcuts: vec!["Ctrl+Shift+E".to_string(), "Cmd+Shift+E".to_string()],
            enabled: true,
            handler: CommandHandler::Navigation(NavigationCommand::ToggleFileTree),
        });
        
        // File operations
        self.register_command(Command {
            id: "file.rename".to_string(),
            title: "Rename File".to_string(),
            description: Some("Rename the selected file or directory".to_string()),
            category: "File".to_string(),
            shortcuts: vec!["F2".to_string()],
            enabled: true,
            handler: CommandHandler::File(FileCommand::RenameFile),
        });
        
        // View commands
        self.register_command(Command {
            id: "view.toggle_sidebar".to_string(),
            title: "Toggle Sidebar".to_string(),
            description: Some("Show or hide the sidebar".to_string()),
            category: "View".to_string(),
            shortcuts: vec!["Ctrl+B".to_string(), "Cmd+B".to_string()],
            enabled: true,
            handler: CommandHandler::View(ViewCommand::ToggleSidebar),
        });
    }
}

impl fmt::Display for CommandHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandHandler::System(cmd) => write!(f, "System: {:?}", cmd),
            CommandHandler::Editor(cmd) => write!(f, "Editor: {:?}", cmd),
            CommandHandler::File(cmd) => write!(f, "File: {:?}", cmd),
            CommandHandler::Navigation(cmd) => write!(f, "Navigation: {:?}", cmd),
            CommandHandler::View(cmd) => write!(f, "View: {:?}", cmd),
        }
    }
}

// Note: Default implementation would need a scope, so we'll remove it
// and create AppState directly in the component

// Note: AppState tests require Dioxus ScopeState which is not available in unit tests.
// Integration tests with actual Dioxus components should be used instead.
// 
// The core functionality is tested through the individual NavigationState and SelectionState
// components, and the FileSystemService has its own comprehensive test suite.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_registry_creation() {
        let registry = CommandRegistry::default();
        
        // Should have default commands registered
        assert!(!registry.commands.is_empty());
        assert!(!registry.shortcuts.is_empty());
        assert!(!registry.categories.is_empty());
        
        // Verify some key commands exist
        assert!(registry.get_command("system.show_command_palette").is_some());
        assert!(registry.get_command("editor.close_tab").is_some());
        assert!(registry.get_command("navigation.navigate_up").is_some());
        
        // Verify shortcuts are mapped correctly
        assert!(registry.get_command_by_shortcut("Ctrl+Shift+P").is_some());
        assert!(registry.get_command_by_shortcut("Ctrl+W").is_some());
        assert!(registry.get_command_by_shortcut("Alt+Up").is_some());
    }
    
    #[test]
    fn test_command_search() {
        let registry = CommandRegistry::default();
        
        // Search for "close" should find close-related commands
        let close_commands = registry.search_commands("close");
        assert!(!close_commands.is_empty());
        assert!(close_commands.iter().any(|cmd| cmd.id == "editor.close_tab"));
        
        // Search for "navigate" should find navigation commands
        let nav_commands = registry.search_commands("navigate");
        assert!(!nav_commands.is_empty());
        assert!(nav_commands.iter().any(|cmd| cmd.id == "navigation.navigate_up"));
        
        // Empty search should return all enabled commands
        let all_commands = registry.search_commands("");
        assert!(!all_commands.is_empty());
    }
    
    #[test]
    fn test_command_palette_state() {
        let mut registry = CommandRegistry::default();
        
        // Initially palette should be hidden
        assert!(!registry.palette_state.is_visible);
        
        // Toggle should make it visible and populate filtered commands
        registry.toggle_command_palette();
        assert!(registry.palette_state.is_visible);
        assert!(!registry.palette_state.filtered_commands.is_empty());
        
        // Update search should filter commands
        registry.update_palette_search("close".to_string());
        assert!(registry.palette_state.search_query == "close");
        let filtered_count = registry.palette_state.filtered_commands.len();
        
        // Should have fewer commands after filtering
        assert!(filtered_count > 0);
        assert!(filtered_count <= registry.commands.len());
    }
    
    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::default();
        let initial_count = registry.commands.len();
        
        // Register a custom command
        let custom_command = Command {
            id: "test.custom_command".to_string(),
            title: "Test Custom Command".to_string(),
            description: Some("A test command".to_string()),
            category: "Test".to_string(),
            shortcuts: vec!["Ctrl+T".to_string()],
            enabled: true,
            handler: CommandHandler::System(SystemCommand::ShowKeyboardShortcuts),
        };
        
        registry.register_command(custom_command);
        
        // Should have one more command
        assert_eq!(registry.commands.len(), initial_count + 1);
        
        // Should be able to find the custom command
        assert!(registry.get_command("test.custom_command").is_some());
        assert!(registry.get_command_by_shortcut("Ctrl+T").is_some());
        
        // Should be in the Test category
        let test_commands = registry.get_commands_by_category("Test");
        assert_eq!(test_commands.len(), 1);
        assert_eq!(test_commands[0].id, "test.custom_command");
    }
}