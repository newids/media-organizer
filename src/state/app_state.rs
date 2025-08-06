use std::path::PathBuf;
use std::sync::Arc;
use crate::services::{FileSystemService, NativeFileSystemService, FileEntry};
use crate::state::{NavigationState, SelectionState};
use dioxus::prelude::*;

#[derive(Clone)]
pub struct AppState {
    pub navigation: UseState<NavigationState>,
    pub selection: UseState<SelectionState>,
    pub file_service: Arc<dyn FileSystemService>,
}

impl AppState {
    pub fn new(cx: &ScopeState) -> Self {
        let initial_path = dirs::home_dir();
        
        Self {
            navigation: use_state(cx, || NavigationState::new(initial_path)).clone(),
            selection: use_state(cx, || SelectionState::new()).clone(),
            file_service: Arc::new(NativeFileSystemService::new()),
        }
    }
    
    pub async fn navigate_to(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Set loading state
        self.navigation.with_mut(|nav| {
            nav.set_loading(path.clone(), true);
        });
        
        // Load directory contents
        match self.file_service.list_directory(&path).await {
            Ok(contents) => {
                // Update navigation state
                self.navigation.with_mut(|nav| {
                    if let Err(e) = nav.navigate_to(path.clone()) {
                        tracing::warn!("Navigation error: {}", e);
                    }
                    nav.set_directory_contents(path.clone(), contents);
                });
                
                // Clear selection when navigating
                self.selection.with_mut(|sel| {
                    sel.clear_selection();
                });
                
                Ok(())
            }
            Err(e) => {
                self.navigation.with_mut(|nav| {
                    nav.set_loading(path, false);
                });
                Err(Box::new(e))
            }
        }
    }
    
    pub async fn refresh_current_directory(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_path = self.navigation.get().current_path.clone();
        self.load_directory_contents(current_path).await
    }
    
    pub async fn load_directory_contents(&self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.navigation.with_mut(|nav| {
            nav.set_loading(path.clone(), true);
        });
        
        match self.file_service.list_directory(&path).await {
            Ok(contents) => {
                self.navigation.with_mut(|nav| {
                    nav.set_directory_contents(path, contents);
                });
                Ok(())
            }
            Err(e) => {
                self.navigation.with_mut(|nav| {
                    nav.set_loading(path, false);
                });
                Err(Box::new(e))
            }
        }
    }
    
    pub async fn navigate_back(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path_opt = None;
        self.navigation.with_mut(|nav| {
            path_opt = nav.navigate_back();
        });
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.get().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.with_mut(|sel| {
                sel.clear_selection();
            });
        }
        Ok(())
    }
    
    pub async fn navigate_forward(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path_opt = None;
        self.navigation.with_mut(|nav| {
            path_opt = nav.navigate_forward();
        });
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.get().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.with_mut(|sel| {
                sel.clear_selection();
            });
        }
        Ok(())
    }
    
    pub async fn navigate_up(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path_opt = None;
        self.navigation.with_mut(|nav| {
            path_opt = nav.navigate_up();
        });
        
        if let Some(path) = path_opt {
            // Check if we already have contents cached
            if self.navigation.get().get_directory_contents(&path).is_none() {
                self.load_directory_contents(path).await?;
            }
            
            // Clear selection when navigating
            self.selection.with_mut(|sel| {
                sel.clear_selection();
            });
        }
        Ok(())
    }
    
    pub fn get_current_directory_contents(&self) -> Option<Vec<FileEntry>> {
        let nav = self.navigation.get();
        nav.get_directory_contents(&nav.current_path).cloned()
    }
    
    pub fn is_current_directory_loading(&self) -> bool {
        let nav = self.navigation.get();
        nav.is_loading(&nav.current_path)
    }
    
    pub fn get_current_path(&self) -> PathBuf {
        self.navigation.get().current_path.clone()
    }
    
    pub fn get_breadcrumbs(&self) -> Vec<crate::state::navigation::BreadcrumbItem> {
        self.navigation.get().breadcrumbs.clone()
    }
    
    pub fn can_navigate_back(&self) -> bool {
        self.navigation.get().can_navigate_back()
    }
    
    pub fn can_navigate_forward(&self) -> bool {
        self.navigation.get().can_navigate_forward()
    }
    
    pub fn can_navigate_up(&self) -> bool {
        self.navigation.get().can_navigate_up()
    }
    
    pub fn select_files(&self, paths: Vec<PathBuf>, mode: crate::state::navigation::SelectionMode) {
        self.selection.with_mut(|sel| {
            sel.select_files(paths, mode);
        });
    }
    
    pub fn clear_selection(&self) {
        self.selection.with_mut(|sel| {
            sel.clear_selection();
        });
    }
    
    pub fn is_selected(&self, path: &PathBuf) -> bool {
        self.selection.get().is_selected(path)
    }
    
    pub fn get_selected_files(&self) -> Vec<PathBuf> {
        self.selection.get().get_selected_paths()
    }
    
    pub fn get_selection_count(&self) -> usize {
        self.selection.get().selection_count()
    }
    
    pub fn get_selection_metadata(&self) -> crate::state::navigation::SelectionMetadata {
        self.selection.get().selection_metadata.clone()
    }
}

// Note: Default implementation would need a scope, so we'll remove it
// and create AppState directly in the component

// Note: AppState tests require Dioxus ScopeState which is not available in unit tests.
// Integration tests with actual Dioxus components should be used instead.
// 
// The core functionality is tested through the individual NavigationState and SelectionState
// components, and the FileSystemService has its own comprehensive test suite.