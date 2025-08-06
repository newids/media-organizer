# MediaOrganizer - API Reference

## Overview

This document provides a comprehensive API reference for the MediaOrganizer application, covering all major services, components, and data structures.

## 🧩 Core Services API

### FileSystemService

```rust
#[async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn get_metadata(&self, path: &Path) -> Result<FileMetadata, FileSystemError>;
    async fn watch_directory(&self, path: &Path) -> Result<FileWatcher, FileSystemError>;
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError>;
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError>;
    async fn rename_file(&self, old_path: &Path, new_name: &str) -> Result<PathBuf, FileSystemError>;
    async fn get_file_permissions(&self, path: &Path) -> Result<FilePermissions, FileSystemError>;
    async fn set_file_permissions(&self, path: &Path, permissions: FilePermissions) -> Result<(), FileSystemError>;
}
```

**Key Methods:**
- `list_directory()`: Returns sorted file entries for a directory
- `watch_directory()`: Sets up file system monitoring for changes
- `copy_files()` / `move_files()`: Batch file operations with progress reporting
- `delete_files()`: Safe deletion with trash/recycle bin support

**Error Handling:** All methods return `Result<T, FileSystemError>` with specific error types for different failure scenarios.

### PreviewService

```rust
#[async_trait]
pub trait PreviewService: Send + Sync {
    async fn generate_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Result<Thumbnail, PreviewError>;
    async fn get_preview(&self, path: &Path) -> Result<PreviewContent, PreviewError>;
    async fn extract_metadata(&self, path: &Path) -> Result<FileMetadata, PreviewError>;
    async fn get_supported_formats(&self) -> Vec<FileFormat>;
    async fn is_format_supported(&self, path: &Path) -> bool;
}
```

**Key Methods:**
- `generate_thumbnail()`: Creates cached thumbnails for various file types
- `get_preview()`: Returns renderable preview content
- `extract_metadata()`: Extracts type-specific metadata (EXIF, video info, etc.)

**Supported Formats:**
- **Images**: JPEG, PNG, GIF, WebP, TIFF, BMP, SVG
- **Videos**: MP4, AVI, MOV, WMV, MKV, WebM (with FFmpeg)
- **Audio**: MP3, WAV, FLAC, AAC, OGG
- **Documents**: PDF, Markdown, Text files

### CacheService

```rust
pub struct CacheService {
    pub async fn get_cached_metadata(&self, path: &Path) -> Option<FileMetadata>;
    pub async fn cache_metadata(&self, path: &Path, metadata: FileMetadata);
    pub async fn get_cached_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Option<Thumbnail>;
    pub async fn cache_thumbnail(&self, path: &Path, size: ThumbnailSize, thumbnail: Thumbnail);
    pub async fn invalidate_cache(&self, path: &Path);
    pub async fn cleanup_orphaned_entries(&self);
    pub async fn get_cache_stats(&self) -> CacheStatistics;
    pub async fn clear_cache(&self, cache_type: CacheType) -> Result<(), CacheError>;
}
```

**Cache Types:**
- **Metadata Cache**: SQLite database for file metadata
- **Thumbnail Cache**: File system cache for generated thumbnails
- **Memory Cache**: In-memory LRU cache for frequently accessed data

### OperationsService

```rust
pub struct OperationsService {
    pub async fn execute_operation(&self, operation: FileOperation) -> Result<OperationResult, OperationError>;
    pub async fn cancel_operation(&self, operation_id: OperationId) -> Result<(), OperationError>;
    pub async fn get_operation_progress(&self, operation_id: OperationId) -> Option<OperationProgress>;
    pub async fn get_active_operations(&self) -> Vec<FileOperation>;
    pub async fn undo_operation(&self, operation_id: OperationId) -> Result<(), OperationError>;
    pub async fn redo_operation(&self, operation_id: OperationId) -> Result<(), OperationError>;
    pub async fn get_operation_history(&self) -> Vec<FileOperation>;
}
```

**Operation Types:**
- `Copy`: Copy files to destination
- `Move`: Move files to destination  
- `Delete`: Delete files (with trash support)
- `Rename`: Rename single file/folder
- `CreateFolder`: Create new directory
- `ExtractArchive`: Extract compressed files
- `CompressFiles`: Create archives

## 📊 Data Models

### FileEntry

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
    pub is_directory: bool,
    pub is_hidden: bool,
    pub thumbnail: Option<Thumbnail>,
    pub metadata: Option<FileMetadata>,
}
```

### FileType

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    Directory,
    Image(ImageFormat),
    Video(VideoFormat),
    Audio(AudioFormat),
    Document(DocumentFormat),
    Text(TextFormat),
    Archive(ArchiveFormat),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Jpeg, Png, Gif, WebP, Tiff, Bmp, Svg
}

#[derive(Debug, Clone, PartialEq)]
pub enum VideoFormat {
    Mp4, Avi, Mov, Wmv, Mkv, WebM
}
```

### FileMetadata

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub basic: BasicMetadata,
    pub specific: TypeSpecificMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicMetadata {
    pub file_size: u64,
    pub created: SystemTime,
    pub modified: SystemTime,
    pub accessed: SystemTime,
    pub permissions: FilePermissions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeSpecificMetadata {
    Image(ImageMetadata),
    Video(VideoMetadata),
    Audio(AudioMetadata),
    Document(DocumentMetadata),
}
```

### State Models

```rust
#[derive(Debug, Clone)]
pub struct AppState {
    pub navigation: NavigationState,
    pub ui: UiState,
    pub operations: OperationsState,
    pub cache: CacheState,
    pub notifications: NotificationState,
}

#[derive(Debug, Clone)]
pub struct NavigationState {
    pub current_path: PathBuf,
    pub history: NavigationHistory,
    pub selection: SelectionState,
    pub directory_contents: HashMap<PathBuf, DirectoryContents>,
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub selected_files: HashSet<PathBuf>,
    pub last_selected: Option<PathBuf>,
    pub selection_metadata: SelectionMetadata,
}
```

## 🎨 UI Components API

### App Component

```rust
#[derive(Props)]
pub struct AppProps {
    initial_path: Option<PathBuf>,
    config: AppConfig,
}

pub fn App(cx: Scope<AppProps>) -> Element
```

### FileTree Component

```rust
#[derive(Props)]
pub struct FileTreeProps {
    current_path: PathBuf,
    selected_path: Option<PathBuf>,
    width: f32,
    is_visible: bool,
    on_navigate: EventHandler<PathBuf>,
    on_select: EventHandler<PathBuf>,
    on_resize: EventHandler<f32>,
}

pub fn FileTree(cx: Scope<FileTreeProps>) -> Element
```

**Events:**
- `on_navigate`: Called when user navigates to a new directory
- `on_select`: Called when user selects a file/folder
- `on_resize`: Called when panel is resized

### ContentViewer Component

```rust
#[derive(Props)]
pub struct ContentViewerProps {
    current_path: PathBuf,
    view_mode: ViewMode,
    sort_criteria: SortCriteria,
    selected_files: HashSet<PathBuf>,
    on_file_select: EventHandler<(PathBuf, bool)>,
    on_file_action: EventHandler<FileAction>,
    on_view_mode_change: EventHandler<ViewMode>,
    on_sort_change: EventHandler<SortCriteria>,
}

pub fn ContentViewer(cx: Scope<ContentViewerProps>) -> Element
```

### VirtualScroll Component

```rust
#[derive(Props)]
pub struct VirtualScrollProps<T: Clone + PartialEq + 'static> {
    items: Vec<T>,
    item_height: f32,
    container_height: f32,
    render_item: Rc<dyn Fn(&T, usize) -> Element>,
}

pub fn VirtualScroll<T: Clone + PartialEq + 'static>(cx: Scope<VirtualScrollProps<T>>) -> Element
```

## ⚙️ Configuration API

### AppConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub cache: CacheConfig,
    pub file_system: FileSystemConfig,
    pub preview: PreviewConfig,
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: Theme,
    pub layout: LayoutConfig,
    pub default_view_mode: ViewMode,
    pub default_sort: SortCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_memory_usage: usize,
    pub thumbnail_cache_size: usize,
    pub metadata_cache_size: usize,
    pub virtual_scroll_buffer: usize,
    pub background_thread_count: usize,
}
```

## 🔧 Event System API

### StateEvent

```rust
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
    CacheUpdated { path: PathBuf },
    FileSystemChanged { event: FileSystemEvent },
}
```

### FileAction

```rust
#[derive(Debug, Clone)]
pub enum FileAction {
    Open(PathBuf),
    OpenWith(PathBuf, String),
    Copy(Vec<PathBuf>),
    Cut(Vec<PathBuf>),
    Paste(PathBuf),
    Delete(Vec<PathBuf>),
    Rename(PathBuf, String),
    CreateFolder(PathBuf, String),
    ShowProperties(PathBuf),
    AddToFavorites(PathBuf),
}
```

## 🚦 Error Handling API

### Error Types

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
}

#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: PathBuf },
}
```

## 🧪 Testing API

### Test Utilities

```rust
pub mod test_utils {
    pub fn create_temp_directory() -> TempDir;
    pub fn create_test_files(dir: &Path, files: &[(&str, &[u8])]) -> Result<(), std::io::Error>;
    pub fn create_mock_file_system_service() -> MockFileSystemService;
    pub fn create_test_app_state() -> AppState;
    pub fn simulate_file_selection(state: &mut AppState, paths: Vec<PathBuf>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    
    #[tokio::test]
    async fn test_file_listing() {
        let temp_dir = create_temp_directory();
        let service = NativeFileSystemService::new(Arc::new(CacheService::new().unwrap())).unwrap();
        
        create_test_files(temp_dir.path(), &[
            ("test.txt", b"content"),
            ("image.jpg", &[]),
        ]);
        
        let entries = service.list_directory(temp_dir.path()).await.unwrap();
        assert_eq!(entries.len(), 2);
    }
}
```

## 📈 Performance Monitoring API

### Metrics Collection

```rust
pub struct PerformanceMetrics {
    pub fn record_directory_load_time(&self, path: &Path, duration: Duration);
    pub fn record_thumbnail_generation_time(&self, file_type: FileType, duration: Duration);
    pub fn record_memory_usage(&self, component: &str, bytes: usize);
    pub fn get_performance_report(&self) -> PerformanceReport;
}

#[derive(Debug, Serialize)]
pub struct PerformanceReport {
    pub avg_directory_load_time: Duration,
    pub avg_thumbnail_generation_time: HashMap<FileType, Duration>,
    pub memory_usage_by_component: HashMap<String, usize>,
    pub cache_hit_rates: CacheHitRates,
}
```

## 🔌 Plugin API (Future)

```rust
pub trait MediaOrganizerPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn supported_file_types(&self) -> Vec<FileType>;
    
    fn generate_preview(&self, path: &Path) -> Result<PreviewContent, PluginError>;
    fn extract_metadata(&self, path: &Path) -> Result<FileMetadata, PluginError>;
    fn provide_actions(&self, file_type: FileType) -> Vec<PluginAction>;
}
```

This API reference provides comprehensive coverage of all major interfaces and data structures in the MediaOrganizer application, enabling developers to understand and extend the system effectively.