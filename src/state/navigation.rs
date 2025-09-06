use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use crate::services::FileEntry;

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_path: PathBuf,
    pub history: NavigationHistory,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub directory_contents: HashMap<PathBuf, Vec<FileEntry>>,
    pub loading_paths: HashSet<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    history: VecDeque<PathBuf>,
    current_index: Option<usize>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub selected_files: HashSet<PathBuf>,
    pub last_selected: Option<PathBuf>,
    pub selection_metadata: SelectionMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct SelectionMetadata {
    pub total_size: u64,
    pub file_count: usize,
    pub directory_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub enum SelectionMode {
    Replace,
    Add,
    Toggle,
    Range,
}

impl NavigationState {
    pub fn new(initial_path: Option<PathBuf>) -> Self {
        let current_path = initial_path.unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        });
        
        let breadcrumbs = Self::create_breadcrumbs(&current_path);
        
        Self {
            current_path,
            history: NavigationHistory::new(),
            breadcrumbs,
            directory_contents: HashMap::new(),
            loading_paths: HashSet::new(),
        }
    }
    
    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), NavigationError> {
        if !path.exists() {
            return Err(NavigationError::PathNotFound(path));
        }
        
        if !path.is_dir() {
            return Err(NavigationError::NotADirectory(path));
        }
        
        // Add current path to history before navigating
        self.history.push(self.current_path.clone());
        
        // Update current path and breadcrumbs
        self.current_path = path;
        self.breadcrumbs = Self::create_breadcrumbs(&self.current_path);
        
        Ok(())
    }
    
    pub fn navigate_back(&mut self) -> Option<PathBuf> {
        if let Some(previous_path) = self.history.back() {
            self.current_path = previous_path.clone();
            self.breadcrumbs = Self::create_breadcrumbs(&self.current_path);
            Some(previous_path)
        } else {
            None
        }
    }
    
    pub fn navigate_forward(&mut self) -> Option<PathBuf> {
        if let Some(next_path) = self.history.forward() {
            self.current_path = next_path.clone();
            self.breadcrumbs = Self::create_breadcrumbs(&self.current_path);
            Some(next_path)
        } else {
            None
        }
    }
    
    pub fn navigate_up(&mut self) -> Option<PathBuf> {
        if let Some(parent) = self.current_path.parent() {
            let parent_path = parent.to_path_buf();
            self.history.push(self.current_path.clone());
            self.current_path = parent_path.clone();
            self.breadcrumbs = Self::create_breadcrumbs(&self.current_path);
            Some(parent_path)
        } else {
            None
        }
    }
    
    pub fn set_directory_contents(&mut self, path: PathBuf, contents: Vec<FileEntry>) {
        self.directory_contents.insert(path.clone(), contents);
        self.loading_paths.remove(&path);
    }
    
    pub fn get_directory_contents(&self, path: &PathBuf) -> Option<&Vec<FileEntry>> {
        self.directory_contents.get(path)
    }
    
    pub fn set_loading(&mut self, path: PathBuf, loading: bool) {
        if loading {
            self.loading_paths.insert(path);
        } else {
            self.loading_paths.remove(&path);
        }
    }
    
    pub fn is_loading(&self, path: &PathBuf) -> bool {
        self.loading_paths.contains(path)
    }
    
    pub fn can_navigate_back(&self) -> bool {
        self.history.can_back()
    }
    
    pub fn can_navigate_forward(&self) -> bool {
        self.history.can_forward()
    }
    
    pub fn can_navigate_up(&self) -> bool {
        self.current_path.parent().is_some()
    }
    
    /// Clear navigation history (used when changing root contexts)
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    fn create_breadcrumbs(path: &PathBuf) -> Vec<BreadcrumbItem> {
        let mut breadcrumbs = Vec::new();
        let mut current = path.as_path();
        
        // Build breadcrumbs from root to current path
        let mut components = Vec::new();
        while let Some(parent) = current.parent() {
            if let Some(name) = current.file_name() {
                components.push((name.to_string_lossy().to_string(), current.to_path_buf()));
            }
            current = parent;
        }
        
        // Add root component
        components.push(("Root".to_string(), current.to_path_buf()));
        components.reverse();
        
        for (name, path) in components {
            breadcrumbs.push(BreadcrumbItem { name, path });
        }
        
        breadcrumbs
    }
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self {
            history: VecDeque::new(),
            current_index: None,
            max_size: 100,
        }
    }
    
    pub fn push(&mut self, path: PathBuf) {
        // Remove any forward history when adding new entry
        if let Some(index) = self.current_index {
            self.history.truncate(index + 1);
        }
        
        self.history.push_back(path);
        self.current_index = Some(self.history.len() - 1);
        
        // Limit history size
        if self.history.len() > self.max_size {
            self.history.pop_front();
            if let Some(ref mut index) = self.current_index {
                *index = index.saturating_sub(1);
            }
        }
    }
    
    pub fn back(&mut self) -> Option<PathBuf> {
        if let Some(current_index) = self.current_index {
            if current_index > 0 {
                self.current_index = Some(current_index - 1);
                return self.history.get(current_index - 1).cloned();
            }
        }
        None
    }
    
    pub fn forward(&mut self) -> Option<PathBuf> {
        if let Some(current_index) = self.current_index {
            if current_index < self.history.len() - 1 {
                self.current_index = Some(current_index + 1);
                return self.history.get(current_index + 1).cloned();
            }
        }
        None
    }
    
    pub fn can_back(&self) -> bool {
        self.current_index.map_or(false, |index| index > 0)
    }
    
    pub fn can_forward(&self) -> bool {
        self.current_index.map_or(false, |index| index < self.history.len() - 1)
    }
    
    /// Clear all navigation history
    pub fn clear(&mut self) {
        self.history.clear();
        self.current_index = None;
    }
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            selected_files: HashSet::new(),
            last_selected: None,
            selection_metadata: SelectionMetadata::default(),
        }
    }
    
    pub fn select_files(&mut self, paths: Vec<PathBuf>, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.selected_files.clear();
                for path in paths {
                    self.selected_files.insert(path.clone());
                    self.last_selected = Some(path);
                }
            }
            SelectionMode::Add => {
                for path in paths {
                    self.selected_files.insert(path.clone());
                    self.last_selected = Some(path);
                }
            }
            SelectionMode::Toggle => {
                for path in paths {
                    if self.selected_files.contains(&path) {
                        self.selected_files.remove(&path);
                    } else {
                        self.selected_files.insert(path.clone());
                        self.last_selected = Some(path);
                    }
                }
            }
            SelectionMode::Range => {
                // Range selection would need additional context (e.g., file list)
                // For now, treat as add
                for path in paths {
                    self.selected_files.insert(path.clone());
                    self.last_selected = Some(path);
                }
            }
        }
        
        self.update_metadata();
    }
    
    pub fn clear_selection(&mut self) {
        self.selected_files.clear();
        self.last_selected = None;
        self.selection_metadata = SelectionMetadata::default();
    }
    
    pub fn is_selected(&self, path: &PathBuf) -> bool {
        self.selected_files.contains(path)
    }
    
    pub fn selection_count(&self) -> usize {
        self.selected_files.len()
    }
    
    pub fn get_selected_paths(&self) -> Vec<PathBuf> {
        self.selected_files.iter().cloned().collect()
    }
    
    fn update_metadata(&mut self) {
        let mut total_size = 0u64;
        let mut file_count = 0;
        let mut directory_count = 0;
        
        for path in &self.selected_files {
            if path.is_dir() {
                directory_count += 1;
            } else {
                file_count += 1;
                if let Ok(metadata) = std::fs::metadata(path) {
                    total_size += metadata.len();
                }
            }
        }
        
        self.selection_metadata = SelectionMetadata {
            total_size,
            file_count,
            directory_count,
            total_count: self.selected_files.len(),
        };
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NavigationError {
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_navigation_history() {
        let mut history = NavigationHistory::new();
        
        let path1 = PathBuf::from("/home/user");
        let path2 = PathBuf::from("/home/user/documents");
        let path3 = PathBuf::from("/home/user/pictures");
        
        // Test initial state
        assert!(!history.can_back());
        assert!(!history.can_forward());
        
        // Add paths
        history.push(path1.clone());
        history.push(path2.clone());
        history.push(path3.clone());
        
        // Test back navigation
        assert!(history.can_back());
        assert_eq!(history.back(), Some(path2.clone()));
        assert_eq!(history.back(), Some(path1.clone()));
        assert!(!history.can_back());
        
        // Test forward navigation
        assert!(history.can_forward());
        assert_eq!(history.forward(), Some(path2.clone()));
        assert_eq!(history.forward(), Some(path3.clone()));
        assert!(!history.can_forward());
    }
    
    #[test]
    fn test_selection_state() {
        let mut selection = SelectionState::new();
        
        let file1 = PathBuf::from("test1.txt");
        let file2 = PathBuf::from("test2.txt");
        
        // Test initial state
        assert_eq!(selection.selection_count(), 0);
        assert!(!selection.is_selected(&file1));
        
        // Test replace selection
        selection.select_files(vec![file1.clone()], SelectionMode::Replace);
        assert_eq!(selection.selection_count(), 1);
        assert!(selection.is_selected(&file1));
        assert!(!selection.is_selected(&file2));
        
        // Test add selection
        selection.select_files(vec![file2.clone()], SelectionMode::Add);
        assert_eq!(selection.selection_count(), 2);
        assert!(selection.is_selected(&file1));
        assert!(selection.is_selected(&file2));
        
        // Test toggle selection
        selection.select_files(vec![file1.clone()], SelectionMode::Toggle);
        assert_eq!(selection.selection_count(), 1);
        assert!(!selection.is_selected(&file1));
        assert!(selection.is_selected(&file2));
        
        // Test clear selection
        selection.clear_selection();
        assert_eq!(selection.selection_count(), 0);
    }
    
    #[test]
    fn test_breadcrumbs() {
        let path = PathBuf::from("/home/user/documents/projects");
        let breadcrumbs = NavigationState::create_breadcrumbs(&path);
        
        assert!(!breadcrumbs.is_empty());
        assert_eq!(breadcrumbs[0].name, "Root");
        
        // Check that breadcrumbs represent the path hierarchy
        for (i, breadcrumb) in breadcrumbs.iter().enumerate() {
            if i > 0 {
                assert!(breadcrumb.path.starts_with(&breadcrumbs[i-1].path));
            }
        }
    }
}