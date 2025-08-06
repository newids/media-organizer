# MediaOrganizer - State Management Design

## Overview

This document defines the state management architecture for MediaOrganizer, focusing on Dioxus state management patterns, data flow, and performance optimization for handling large file collections.

## State Management Principles

1. **Single Source of Truth**: Centralized state with clear ownership
2. **Unidirectional Data Flow**: State flows down, events flow up
3. **Immutable Updates**: State changes through immutable operations
4. **Performance Optimized**: Selective updates to minimize re-renders
5. **Type Safety**: Strong typing for all state structures

## Global State Architecture

```rust
// Main application state container
#[derive(Debug, Clone)]
pub struct AppState {
    // Navigation state
    pub navigation: NavigationState,
    
    // UI state
    pub ui: UiState,
    
    // File operations state
    pub operations: OperationsState,
    
    // Cache and performance state
    pub cache: CacheState,
    
    // Error and notification state
    pub notifications: NotificationState,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            navigation: NavigationState::new(config.initial_path),
            ui: UiState::new(config.ui_config),
            operations: OperationsState::new(),
            cache: CacheState::new(config.cache_config),
            notifications: NotificationState::new(),
        }
    }
    
    // State update methods
    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), NavigationError> {
        self.navigation.navigate_to(path)
    }
    
    pub fn select_files(&mut self, paths: Vec<PathBuf>, mode: SelectionMode) {
        self.navigation.select_files(paths, mode);
    }
    
    pub fn update_view_mode(&mut self, mode: ViewMode) {
        self.ui.view_mode = mode;
    }
    
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.add(notification);
    }
}
```

## Navigation State

```rust
#[derive(Debug, Clone)]
pub struct NavigationState {
    // Current directory
    pub current_path: PathBuf,
    
    // Navigation history for back/forward functionality
    pub history: NavigationHistory,
    
    // File selection state
    pub selection: SelectionState,
    
    // Directory contents cache
    pub directory_contents: HashMap<PathBuf, DirectoryContents>,
    
    // File system watcher state
    pub watch_state: WatchState,
}

impl NavigationState {
    pub fn new(initial_path: Option<PathBuf>) -> Self {
        let current_path = initial_path.unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
        });
        
        Self {
            current_path,
            history: NavigationHistory::new(),
            selection: SelectionState::new(),
            directory_contents: HashMap::new(),
            watch_state: WatchState::new(),
        }
    }
    
    pub fn navigate_to(&mut self, path: PathBuf) -> Result<(), NavigationError> {
        // Validate path exists and is accessible
        if !path.exists() {
            return Err(NavigationError::PathNotFound(path));
        }
        
        if !path.is_dir() {
            return Err(NavigationError::NotADirectory(path));
        }
        
        // Add current path to history
        self.history.push(self.current_path.clone());
        
        // Update current path
        self.current_path = path;
        
        // Clear selection when navigating
        self.selection.clear();
        
        Ok(())
    }
    
    pub fn navigate_back(&mut self) -> Option<PathBuf> {
        if let Some(previous_path) = self.history.back() {
            self.current_path = previous_path.clone();
            self.selection.clear();
            Some(previous_path)
        } else {
            None
        }
    }
    
    pub fn navigate_forward(&mut self) -> Option<PathBuf> {
        if let Some(next_path) = self.history.forward() {
            self.current_path = next_path.clone();
            self.selection.clear();
            Some(next_path)
        } else {
            None
        }
    }
    
    pub fn select_files(&mut self, paths: Vec<PathBuf>, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                self.selection.selected_files = paths.into_iter().collect();
            }
            SelectionMode::Add => {
                for path in paths {
                    self.selection.selected_files.insert(path);
                }
            }
            SelectionMode::Toggle => {
                for path in paths {
                    if self.selection.selected_files.contains(&path) {
                        self.selection.selected_files.remove(&path);
                    } else {
                        self.selection.selected_files.insert(path);
                    }
                }
            }
            SelectionMode::Range => {
                // Implement range selection logic
                self.select_range(paths);
            }
        }
        
        // Update selection metadata
        self.selection.update_metadata();
    }
    
    fn select_range(&mut self, paths: Vec<PathBuf>) {
        // Implementation for range selection (Shift+click)
        if let (Some(start), Some(end)) = (paths.first(), paths.last()) {
            if let Some(contents) = self.directory_contents.get(&self.current_path) {
                let start_idx = contents.files.iter().position(|f| &f.path == start);
                let end_idx = contents.files.iter().position(|f| &f.path == end);
                
                if let (Some(start), Some(end)) = (start_idx, end_idx) {
                    let (start, end) = if start <= end { (start, end) } else { (end, start) };
                    
                    for file in &contents.files[start..=end] {
                        self.selection.selected_files.insert(file.path.clone());
                    }
                }
            }
        }
    }
    
    pub fn get_breadcrumbs(&self) -> Vec<BreadcrumbItem> {
        let mut breadcrumbs = Vec::new();
        let mut current = self.current_path.as_path();
        
        while let Some(parent) = current.parent() {
            if let Some(name) = current.file_name() {
                breadcrumbs.push(BreadcrumbItem {
                    name: name.to_string_lossy().to_string(),
                    path: current.to_path_buf(),
                });
            }
            current = parent;
        }
        
        // Add root
        breadcrumbs.push(BreadcrumbItem {
            name: "Root".to_string(),
            path: current.to_path_buf(),
        });
        
        breadcrumbs.reverse();
        breadcrumbs
    }
}

#[derive(Debug, Clone)]
pub struct NavigationHistory {
    history: Vec<PathBuf>,
    current_index: isize,
    max_size: usize,
}

impl NavigationHistory {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: -1,
            max_size: 100,
        }
    }
    
    pub fn push(&mut self, path: PathBuf) {
        // Remove any forward history when adding new entry
        if self.current_index >= 0 {
            let index = self.current_index as usize;
            self.history.truncate(index + 1);
        }
        
        self.history.push(path);
        self.current_index = self.history.len() as isize - 1;
        
        // Limit history size
        if self.history.len() > self.max_size {
            self.history.remove(0);
            self.current_index -= 1;
        }
    }
    
    pub fn back(&mut self) -> Option<PathBuf> {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.history.get(self.current_index as usize).cloned()
        } else {
            None
        }
    }
    
    pub fn forward(&mut self) -> Option<PathBuf> {
        if self.current_index < self.history.len() as isize - 1 {
            self.current_index += 1;
            self.history.get(self.current_index as usize).cloned()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub selected_files: HashSet<PathBuf>,
    pub last_selected: Option<PathBuf>,
    pub selection_metadata: SelectionMetadata,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            selected_files: HashSet::new(),
            last_selected: None,
            selection_metadata: SelectionMetadata::default(),
        }
    }
    
    pub fn clear(&mut self) {
        self.selected_files.clear();
        self.last_selected = None;
        self.selection_metadata = SelectionMetadata::default();
    }
    
    pub fn update_metadata(&mut self) {
        // Calculate total size and file counts
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

#[derive(Debug, Clone, Default)]
pub struct SelectionMetadata {
    pub total_size: u64,
    pub file_count: usize,
    pub directory_count: usize,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub enum SelectionMode {
    Replace, // Replace current selection
    Add,     // Add to current selection (Ctrl+click)
    Toggle,  // Toggle selection state
    Range,   // Range selection (Shift+click)
}
```

## UI State Management

```rust
#[derive(Debug, Clone)]
pub struct UiState {
    // Layout configuration
    pub layout: LayoutState,
    
    // View configuration
    pub view_mode: ViewMode,
    pub sort_criteria: SortCriteria,
    
    // Panel states
    pub panels: PanelStates,
    
    // Modal and dialog states
    pub modals: ModalStates,
    
    // Theme and appearance
    pub theme: ThemeState,
}

impl UiState {
    pub fn new(config: UiConfig) -> Self {
        Self {
            layout: LayoutState::new(config.layout),
            view_mode: config.default_view_mode,
            sort_criteria: config.default_sort,
            panels: PanelStates::new(),
            modals: ModalStates::new(),
            theme: ThemeState::new(config.theme),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutState {
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_panel_height: f32,
    pub is_left_panel_visible: bool,
    pub is_bottom_panel_visible: bool,
    pub window_size: (f32, f32),
}

impl LayoutState {
    pub fn new(config: LayoutConfig) -> Self {
        Self {
            left_panel_width: config.default_left_width,
            right_panel_width: config.default_right_width,
            bottom_panel_height: config.default_bottom_height,
            is_left_panel_visible: true,
            is_bottom_panel_visible: false,
            window_size: (1200.0, 800.0),
        }
    }
    
    pub fn resize_panel(&mut self, panel: Panel, delta: f32) {
        match panel {
            Panel::Left => {
                self.left_panel_width = (self.left_panel_width + delta).clamp(200.0, 600.0);
                self.right_panel_width = self.window_size.0 - self.left_panel_width;
            }
            Panel::Bottom => {
                self.bottom_panel_height = (self.bottom_panel_height + delta).clamp(100.0, 400.0);
            }
            _ => {}
        }
    }
    
    pub fn toggle_panel(&mut self, panel: Panel) {
        match panel {
            Panel::Left => self.is_left_panel_visible = !self.is_left_panel_visible,
            Panel::Bottom => self.is_bottom_panel_visible = !self.is_bottom_panel_visible,
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum Panel {
    Left,
    Right,
    Bottom,
}

#[derive(Debug, Clone)]
pub struct PanelStates {
    pub file_tree_state: FileTreeState,
    pub content_viewer_state: ContentViewerState,
    pub preview_panel_state: PreviewPanelState,
}

#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub expanded_folders: HashSet<PathBuf>,
    pub scroll_position: f32,
    pub selected_node: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ContentViewerState {
    pub scroll_position: (f32, f32),
    pub virtual_scroll_state: VirtualScrollState,
    pub filter_state: FilterState,
}

#[derive(Debug, Clone)]
pub struct VirtualScrollState {
    pub item_height: f32,
    pub item_width: f32,
    pub visible_start: usize,
    pub visible_count: usize,
    pub buffer_size: usize,
}

impl VirtualScrollState {
    pub fn new() -> Self {
        Self {
            item_height: 100.0,
            item_width: 120.0,
            visible_start: 0,
            visible_count: 0,
            buffer_size: 5,
        }
    }
    
    pub fn update_viewport(&mut self, viewport_height: f32, viewport_width: f32) {
        self.visible_count = (viewport_height / self.item_height).ceil() as usize + 2;
    }
    
    pub fn update_scroll(&mut self, scroll_top: f32) {
        self.visible_start = (scroll_top / self.item_height).floor() as usize;
    }
    
    pub fn get_visible_range(&self, total_items: usize) -> (usize, usize) {
        let start = self.visible_start.saturating_sub(self.buffer_size);
        let end = (self.visible_start + self.visible_count + self.buffer_size).min(total_items);
        (start, end)
    }
}
```

## Operations State Management

```rust
#[derive(Debug, Clone)]
pub struct OperationsState {
    // Current file operations
    pub active_operations: Vec<FileOperation>,
    
    // Operation history for undo/redo
    pub operation_history: OperationHistory,
    
    // Background task state
    pub background_tasks: HashMap<TaskId, BackgroundTaskState>,
    
    // Clipboard state
    pub clipboard: ClipboardState,
}

impl OperationsState {
    pub fn new() -> Self {
        Self {
            active_operations: Vec::new(),
            operation_history: OperationHistory::new(),
            background_tasks: HashMap::new(),
            clipboard: ClipboardState::new(),
        }
    }
    
    pub fn start_operation(&mut self, operation: FileOperation) -> OperationId {
        let id = OperationId::new();
        let mut operation = operation;
        operation.id = id;
        operation.status = OperationStatus::InProgress;
        operation.started_at = Some(Instant::now());
        
        self.active_operations.push(operation);
        id
    }
    
    pub fn update_operation_progress(&mut self, id: OperationId, progress: f32) {
        if let Some(operation) = self.active_operations.iter_mut().find(|op| op.id == id) {
            operation.progress = progress;
        }
    }
    
    pub fn complete_operation(&mut self, id: OperationId, result: OperationResult) {
        if let Some(pos) = self.active_operations.iter().position(|op| op.id == id) {
            let mut operation = self.active_operations.remove(pos);
            operation.status = match result {
                OperationResult::Success => OperationStatus::Completed,
                OperationResult::Error(_) => OperationStatus::Failed,
                OperationResult::Cancelled => OperationStatus::Cancelled,
            };
            operation.completed_at = Some(Instant::now());
            operation.result = Some(result);
            
            // Add to history if it's an undoable operation
            if operation.is_undoable() {
                self.operation_history.add(operation);
            }
        }
    }
    
    pub fn cancel_operation(&mut self, id: OperationId) {
        if let Some(operation) = self.active_operations.iter_mut().find(|op| op.id == id) {
            operation.status = OperationStatus::Cancelling;
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileOperation {
    pub id: OperationId,
    pub operation_type: OperationType,
    pub source_paths: Vec<PathBuf>,
    pub destination_path: Option<PathBuf>,
    pub status: OperationStatus,
    pub progress: f32,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub result: Option<OperationResult>,
}

impl FileOperation {
    pub fn is_undoable(&self) -> bool {
        matches!(self.operation_type, 
            OperationType::Move | 
            OperationType::Copy | 
            OperationType::Delete | 
            OperationType::Rename
        )
    }
}

#[derive(Debug, Clone)]
pub enum OperationType {
    Copy,
    Move,
    Delete,
    Rename,
    CreateFolder,
    ExtractArchive,
    CompressFiles,
}

#[derive(Debug, Clone)]
pub enum OperationStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Cancelling,
}

#[derive(Debug, Clone)]
pub enum OperationResult {
    Success,
    Error(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct OperationHistory {
    operations: Vec<FileOperation>,
    current_index: isize,
    max_size: usize,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            current_index: -1,
            max_size: 50,
        }
    }
    
    pub fn add(&mut self, operation: FileOperation) {
        // Remove any future operations when adding new one
        if self.current_index >= 0 {
            let index = self.current_index as usize;
            self.operations.truncate(index + 1);
        }
        
        self.operations.push(operation);
        self.current_index = self.operations.len() as isize - 1;
        
        // Limit history size
        if self.operations.len() > self.max_size {
            self.operations.remove(0);
            self.current_index -= 1;
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.current_index >= 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.current_index < self.operations.len() as isize - 1
    }
    
    pub fn undo(&mut self) -> Option<&FileOperation> {
        if self.can_undo() {
            let operation = &self.operations[self.current_index as usize];
            self.current_index -= 1;
            Some(operation)
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<&FileOperation> {
        if self.can_redo() {
            self.current_index += 1;
            Some(&self.operations[self.current_index as usize])
        } else {
            None
        }
    }
}
```

## State Synchronization and Updates

```rust
// State update system for coordinating changes
pub struct StateUpdateSystem {
    state: Arc<RwLock<AppState>>,
    event_bus: EventBus,
    persistence: StatePersistence,
}

impl StateUpdateSystem {
    pub fn new(initial_state: AppState) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            event_bus: EventBus::new(),
            persistence: StatePersistence::new(),
        }
    }
    
    pub async fn handle_event(&self, event: StateEvent) -> Result<(), StateError> {
        match event {
            StateEvent::NavigationChanged { path } => {
                let mut state = self.state.write().await;
                state.navigation.navigate_to(path)?;
                self.event_bus.emit(StateEvent::StateUpdated);
            }
            StateEvent::FilesSelected { paths, mode } => {
                let mut state = self.state.write().await;
                state.navigation.select_files(paths, mode);
                self.event_bus.emit(StateEvent::StateUpdated);
            }
            StateEvent::ViewModeChanged { mode } => {
                let mut state = self.state.write().await;
                state.ui.view_mode = mode;
                self.event_bus.emit(StateEvent::StateUpdated);
            }
            StateEvent::OperationStarted { operation } => {
                let mut state = self.state.write().await;
                state.operations.start_operation(operation);
                self.event_bus.emit(StateEvent::StateUpdated);
            }
            // ... handle other events
        }
        
        // Persist state changes
        self.persistence.save_state(&self.state.read().await).await?;
        
        Ok(())
    }
    
    pub fn subscribe_to_updates(&self) -> EventReceiver<StateEvent> {
        self.event_bus.subscribe()
    }
}

#[derive(Debug, Clone)]
pub enum StateEvent {
    NavigationChanged { path: PathBuf },
    FilesSelected { paths: Vec<PathBuf>, mode: SelectionMode },
    ViewModeChanged { mode: ViewMode },
    SortCriteriaChanged { criteria: SortCriteria },
    OperationStarted { operation: FileOperation },
    OperationProgress { id: OperationId, progress: f32 },
    OperationCompleted { id: OperationId, result: OperationResult },
    NotificationAdded { notification: Notification },
    StateUpdated,
}
```

This state management design provides a robust foundation for handling complex application state while maintaining performance and type safety throughout the MediaOrganizer application.