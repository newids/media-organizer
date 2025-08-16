use std::path::PathBuf;
use std::sync::Arc;
use crate::services::{FileSystemService, NativeFileSystemService, FileEntry};
use crate::state::navigation::{NavigationState, SelectionState};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// Centralized application state with shared contexts for all UI components
#[derive(Clone)]
pub struct AppState {
    /// Navigation state (current directory, history, breadcrumbs)
    pub navigation: Signal<NavigationState>,
    /// File selection state (selected files, selection mode)
    pub selection: Signal<SelectionState>,
    /// Current directory file entries
    pub file_entries: Signal<Vec<FileEntry>>,
    /// Current view mode (grid, list, preview)
    pub view_mode: Signal<ViewMode>,
    /// Search and filter state
    pub search_state: Signal<SearchState>,
    /// Operation progress and status
    pub operation_state: Signal<OperationState>,
    /// Application settings and preferences
    pub settings: Signal<SettingsState>,
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
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Theme::Light,
            "auto" => Theme::Auto,
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

impl AppState {
    /// Create new AppState with initial signals
    /// Note: This must be called within a Dioxus component context
    pub fn new() -> Self {
        // Use default home directory for normal operation
        let initial_path = dirs::home_dir();
        
        Self {
            navigation: use_signal(|| NavigationState::new(initial_path)),
            selection: use_signal(|| SelectionState::new()),
            file_entries: use_signal(Vec::new),
            view_mode: use_signal(ViewMode::default),
            search_state: use_signal(SearchState::default),
            operation_state: use_signal(OperationState::default),
            settings: use_signal(SettingsState::default),
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
}

// Note: Default implementation would need a scope, so we'll remove it
// and create AppState directly in the component

// Note: AppState tests require Dioxus ScopeState which is not available in unit tests.
// Integration tests with actual Dioxus components should be used instead.
// 
// The core functionality is tested through the individual NavigationState and SelectionState
// components, and the FileSystemService has its own comprehensive test suite.