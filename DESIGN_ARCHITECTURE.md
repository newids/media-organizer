# MediaOrganizer - System Architecture Design

## Overview

This document outlines the comprehensive system architecture for MediaOrganizer, a cross-platform media/file management application built with Dioxus and Rust.

## Architecture Principles

1. **Separation of Concerns**: Clear boundaries between UI, business logic, and data layers
2. **Performance First**: Virtual scrolling, lazy loading, and background processing
3. **Cross-Platform Consistency**: Unified behavior across Windows, macOS, and Linux
4. **Scalability**: Handle 10,000+ files efficiently
5. **Extensibility**: Plugin-ready architecture for future enhancements

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Presentation Layer                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ FileTree    │  │ ContentView │  │ PreviewPanel        │  │
│  │ Component   │  │ Component   │  │ Component           │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │           Application State Management                  │  │
│  │     (Global State, Selection, Navigation)              │  │
│  └─────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Business Logic Layer                   │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ FileSystem  │  │ Preview     │  │ Operations          │  │
│  │ Service     │  │ Service     │  │ Service             │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Cache       │  │ Search      │  │ Background          │  │
│  │ Service     │  │ Service     │  │ Task Manager        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      Data Access Layer                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ File System │  │ Metadata    │  │ Cache Storage       │  │
│  │ Abstraction │  │ Extractor   │  │ (SQLite/Files)      │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Platform Layer                          │
├─────────────────────────────────────────────────────────────┤
│          OS File System APIs & Native Integrations          │
└─────────────────────────────────────────────────────────────┘
```

## Core Components Architecture

### 1. Application Structure

```rust
// Main application entry point
struct MediaOrganizerApp {
    // Global application state
    state: AppState,
    
    // Service instances
    file_service: Arc<FileSystemService>,
    preview_service: Arc<PreviewService>,
    cache_service: Arc<CacheService>,
    operations_service: Arc<OperationsService>,
    
    // Background task manager
    task_manager: Arc<BackgroundTaskManager>,
}

// Global application state
#[derive(Clone)]
struct AppState {
    // Current navigation state
    current_path: PathBuf,
    navigation_history: VecDeque<PathBuf>,
    
    // Selection state
    selected_files: HashSet<PathBuf>,
    
    // UI state
    layout_config: LayoutConfig,
    view_mode: ViewMode,
    sort_criteria: SortCriteria,
    
    // Performance settings
    virtual_scroll_config: VirtualScrollConfig,
    cache_config: CacheConfig,
}
```

### 2. Component Hierarchy

```rust
// Root component
fn App(cx: Scope) -> Element {
    // Main layout with panels
    render! {
        div { class: "app-container",
            TopBar {}
            div { class: "main-content",
                LeftPanel {}
                RightPanel {}
            }
            BottomBar {}
        }
    }
}

// Left panel - File tree navigator
fn LeftPanel(cx: Scope) -> Element {
    render! {
        div { class: "left-panel",
            FileTree {
                current_path: state.current_path.clone(),
                on_navigate: move |path| { /* navigate to path */ },
                on_select: move |path| { /* select file/folder */ },
            }
        }
    }
}

// Right panel - Content viewer and preview
fn RightPanel(cx: Scope) -> Element {
    render! {
        div { class: "right-panel",
            match state.view_mode {
                ViewMode::Grid => ContentGrid {},
                ViewMode::List => ContentList {},
                ViewMode::Preview => PreviewPanel {},
            }
        }
    }
}
```

## Service Layer Design

### 1. FileSystem Service

```rust
#[async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata, FileSystemError>;
    async fn watch_directory(&self, path: &Path) -> Result<FileWatcher, FileSystemError>;
    
    // File operations
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError>;
}

pub struct NativeFileSystemService {
    // File system watcher
    watcher: Arc<Mutex<RecommendedWatcher>>,
    
    // Event channel for file system changes
    event_sender: UnboundedSender<FileSystemEvent>,
    
    // Configuration
    config: FileSystemConfig,
}
```

### 2. Preview Service

```rust
#[async_trait]
pub trait PreviewService: Send + Sync {
    async fn generate_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Result<Thumbnail, PreviewError>;
    async fn get_preview(&self, path: &Path) -> Result<PreviewContent, PreviewError>;
    async fn extract_metadata(&self, path: &Path) -> Result<FileMetadata, PreviewError>;
}

pub struct MultiFormatPreviewService {
    // Format-specific handlers
    image_handler: Arc<ImagePreviewHandler>,
    video_handler: Arc<VideoPreviewHandler>,
    audio_handler: Arc<AudioPreviewHandler>,
    document_handler: Arc<DocumentPreviewHandler>,
    
    // Background processing pool
    processing_pool: ThreadPool,
}
```

### 3. Cache Service

```rust
pub struct CacheService {
    // SQLite database for metadata cache
    metadata_db: Arc<SqlitePool>,
    
    // File system cache for thumbnails
    thumbnail_cache: Arc<FileCache>,
    
    // In-memory cache for frequently accessed data
    memory_cache: Arc<MemoryCache>,
    
    // Cache configuration
    config: CacheConfig,
}

impl CacheService {
    pub async fn get_cached_metadata(&self, path: &Path) -> Option<FileMetadata>;
    pub async fn cache_metadata(&self, path: &Path, metadata: FileMetadata);
    
    pub async fn get_cached_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Option<Thumbnail>;
    pub async fn cache_thumbnail(&self, path: &Path, size: ThumbnailSize, thumbnail: Thumbnail);
    
    pub async fn invalidate_cache(&self, path: &Path);
    pub async fn cleanup_orphaned_entries(&self);
}
```

## Data Models

### 1. Core Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
    pub size: u64,
    pub modified: SystemTime,
    pub created: SystemTime,
    pub permissions: FilePermissions,
    
    // Optional cached data
    pub thumbnail: Option<Thumbnail>,
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Directory,
    Image(ImageFormat),
    Video(VideoFormat),
    Audio(AudioFormat),
    Document(DocumentFormat),
    Text(TextFormat),
    Other(String),
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
    // Common metadata
    pub basic: BasicMetadata,
    
    // Type-specific metadata
    pub specific: TypeSpecificMetadata,
}

#[derive(Debug, Clone)]
pub enum TypeSpecificMetadata {
    Image(ImageMetadata),
    Video(VideoMetadata),
    Audio(AudioMetadata),
    Document(DocumentMetadata),
}

#[derive(Debug, Clone)]
pub struct ImageMetadata {
    pub dimensions: (u32, u32),
    pub color_space: ColorSpace,
    pub exif_data: Option<ExifData>,
    pub has_transparency: bool,
}
```

### 2. UI State Models

```rust
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub panel_heights: PanelHeights,
    pub is_left_panel_visible: bool,
    pub is_bottom_panel_visible: bool,
}

#[derive(Debug, Clone)]
pub enum ViewMode {
    Grid(GridConfig),
    List(ListConfig),
    Preview(PreviewConfig),
}

#[derive(Debug, Clone)]
pub struct GridConfig {
    pub icon_size: IconSize,
    pub columns: Option<usize>, // None for auto-calculate
    pub show_thumbnails: bool,
    pub show_metadata: bool,
}

#[derive(Debug, Clone)]
pub enum SortCriteria {
    Name(SortOrder),
    Size(SortOrder),
    Modified(SortOrder),
    Created(SortOrder),
    Type(SortOrder),
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}
```

## Virtual Scrolling Architecture

```rust
pub struct VirtualScrollManager {
    // Viewport dimensions
    viewport_height: f32,
    viewport_width: f32,
    
    // Item dimensions
    item_height: f32,
    item_width: f32,
    
    // Calculated properties
    visible_start: usize,
    visible_count: usize,
    total_items: usize,
    
    // Buffer for smooth scrolling
    buffer_size: usize,
}

impl VirtualScrollManager {
    pub fn calculate_visible_range(&self, scroll_top: f32) -> (usize, usize) {
        let start = (scroll_top / self.item_height).floor() as usize;
        let visible_count = (self.viewport_height / self.item_height).ceil() as usize + 1;
        
        let buffered_start = start.saturating_sub(self.buffer_size);
        let buffered_end = (start + visible_count + self.buffer_size).min(self.total_items);
        
        (buffered_start, buffered_end)
    }
    
    pub fn get_scroll_height(&self) -> f32 {
        self.total_items as f32 * self.item_height
    }
}
```

## Background Task Management

```rust
pub struct BackgroundTaskManager {
    // Task queue
    task_queue: Arc<Mutex<VecDeque<BackgroundTask>>>,
    
    // Thread pool for CPU-intensive tasks
    cpu_pool: ThreadPool,
    
    // Async runtime for I/O tasks
    io_runtime: Runtime,
    
    // Task progress tracking
    active_tasks: Arc<Mutex<HashMap<TaskId, TaskProgress>>>,
}

#[derive(Debug, Clone)]
pub enum BackgroundTask {
    GenerateThumbnails {
        paths: Vec<PathBuf>,
        size: ThumbnailSize,
        priority: TaskPriority,
    },
    ExtractMetadata {
        paths: Vec<PathBuf>,
        priority: TaskPriority,
    },
    CalculateDirectorySize {
        path: PathBuf,
        recursive: bool,
        priority: TaskPriority,
    },
    FindDuplicates {
        root_paths: Vec<PathBuf>,
        options: DuplicateSearchOptions,
        priority: TaskPriority,
    },
}

#[derive(Debug, Clone)]
pub struct TaskProgress {
    pub task_id: TaskId,
    pub total_items: usize,
    pub completed_items: usize,
    pub current_item: Option<String>,
    pub status: TaskStatus,
}
```

## Error Handling Strategy

```rust
#[derive(Debug, thiserror::Error)]
pub enum MediaOrganizerError {
    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),
    
    #[error("Preview generation error: {0}")]
    Preview(#[from] PreviewError),
    
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    
    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("Operation cancelled by user")]
    OperationCancelled,
    
    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

// Error recovery strategies
pub struct ErrorRecoveryService {
    pub fn handle_permission_error(&self, path: &Path) -> RecoveryAction;
    pub fn handle_resource_exhaustion(&self, resource: &str) -> RecoveryAction;
    pub fn suggest_user_action(&self, error: &MediaOrganizerError) -> UserActionSuggestion;
}

#[derive(Debug)]
pub enum RecoveryAction {
    Retry { delay: Duration },
    Skip,
    RequestPermission { path: PathBuf },
    FreeResources { target: String },
    UserIntervention { message: String },
}
```

## Performance Optimizations

### 1. Memory Management

```rust
pub struct MemoryManager {
    // Memory usage tracking
    current_usage: Arc<AtomicUsize>,
    max_usage: usize,
    
    // Cache eviction policies
    thumbnail_cache_policy: CachePolicy,
    metadata_cache_policy: CachePolicy,
    
    // Resource limits
    limits: ResourceLimits,
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory: usize,
    pub max_thumbnails_in_memory: usize,
    pub max_concurrent_tasks: usize,
    pub max_cache_entries: usize,
}
```

### 2. Lazy Loading Strategy

```rust
pub struct LazyLoader<T> {
    loader: Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>> + Send>> + Send + Sync>,
    cached_value: Arc<Mutex<Option<T>>>,
    loading_state: Arc<Mutex<LoadingState>>,
}

#[derive(Debug)]
enum LoadingState {
    NotStarted,
    Loading,
    Loaded,
    Error(String),
}

impl<T: Clone + Send + 'static> LazyLoader<T> {
    pub async fn get(&self) -> Result<T, Box<dyn Error>> {
        // Implementation with proper error handling and caching
    }
}
```

This architecture provides a solid foundation for the MediaOrganizer application with clear separation of concerns, performance optimization, and extensibility for future enhancements.