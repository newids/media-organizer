# MediaOrganizer - Implementation Guide

## Overview

This guide provides detailed implementation specifications and step-by-step instructions for building the MediaOrganizer application, including dependency setup, project structure, and development workflow.

## Project Setup

### 1. Initialize Rust Project

```bash
# Create new Dioxus project
cargo new media-organizer --bin
cd media-organizer

# Add to Cargo.toml
```

### 2. Core Dependencies

```toml
[package]
name = "media-organizer"
version = "0.1.0"
edition = "2021"

[dependencies]
# UI Framework
dioxus = "0.4"
dioxus-desktop = "0.4"
dioxus-hooks = "0.4"

# Async Runtime
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# File System Operations
walkdir = "2.4"
notify = "6.1"
dirs = "5.0"

# Media Processing
image = { version = "0.24", features = ["jpeg", "png", "gif", "webp", "tiff", "bmp"] }
ffmpeg-next = { version = "6.0", optional = true }
rodio = { version = "0.17", optional = true }

# PDF Processing
pdf = { version = "0.9", optional = true }
poppler = { version = "0.23", optional = true }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls", "chrono", "uuid"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# Cross-platform
winapi = { version = "0.3", features = ["winuser", "shellapi"], target_os = "windows" }
cocoa = { version = "0.24", target_os = "macos" }

[features]
default = ["video", "audio", "pdf"]
video = ["ffmpeg-next"]
audio = ["rodio"]
pdf = ["pdf", "poppler"]

[dev-dependencies]
tempfile = "3.8"
```

### 3. Project Structure

```
src/
├── main.rs                 # Application entry point
├── app.rs                  # Main app component
├── config.rs               # Configuration management
├── error.rs               # Error types and handling
├── events.rs              # Event system
│
├── ui/                    # User Interface Components
│   ├── mod.rs             # UI module exports
│   ├── app.rs             # Root app component
│   ├── layout.rs          # Layout components
│   ├── file_tree.rs       # File tree navigator
│   ├── content_viewer.rs  # Content viewer components
│   ├── preview_panel.rs   # File preview components
│   ├── toolbar.rs         # Toolbar components
│   ├── modal.rs           # Modal dialogs
│   └── components/        # Reusable UI components
│       ├── mod.rs
│       ├── virtual_scroll.rs
│       ├── file_icon.rs
│       ├── context_menu.rs
│       └── progress_bar.rs
│
├── services/              # Business Logic Services
│   ├── mod.rs            # Service module exports
│   ├── file_system.rs    # File system operations
│   ├── preview.rs        # File preview generation
│   ├── cache.rs          # Caching system
│   ├── operations.rs     # File operations
│   ├── search.rs         # Search functionality
│   └── background.rs     # Background task management
│
├── state/                # State Management
│   ├── mod.rs            # State module exports
│   ├── app_state.rs      # Global application state
│   ├── navigation.rs     # Navigation state
│   ├── ui_state.rs       # UI state management
│   ├── operations.rs     # Operations state
│   └── persistence.rs    # State persistence
│
├── models/               # Data Models
│   ├── mod.rs           # Models module exports
│   ├── file_entry.rs    # File system entry models
│   ├── metadata.rs      # File metadata models
│   ├── config.rs        # Configuration models
│   └── events.rs        # Event models
│
├── utils/               # Utility Functions
│   ├── mod.rs          # Utils module exports
│   ├── path.rs         # Path utilities
│   ├── format.rs       # Formatting utilities
│   ├── async_utils.rs  # Async utilities
│   └── platform.rs     # Platform-specific utilities
│
└── platform/           # Platform-specific code
    ├── mod.rs          # Platform module exports
    ├── windows.rs      # Windows-specific functionality
    ├── macos.rs        # macOS-specific functionality
    └── linux.rs        # Linux-specific functionality
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

#### 1.1 Basic Application Setup

```rust
// src/main.rs
use dioxus::prelude::*;
use media_organizer::{App, AppConfig, Services};

fn main() {
    tracing_subscriber::init();
    
    // Initialize services
    let services = Services::new().expect("Failed to initialize services");
    
    // Load configuration
    let config = AppConfig::load().unwrap_or_default();
    
    // Launch application
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_window(
                dioxus_desktop::WindowBuilder::new()
                    .with_title("MediaOrganizer")
                    .with_inner_size(dioxus_desktop::LogicalSize::new(1200, 800))
                    .with_min_inner_size(dioxus_desktop::LogicalSize::new(800, 600))
            )
    );
}
```

#### 1.2 Service Architecture Implementation

```rust
// src/services/mod.rs
use std::sync::Arc;
use crate::services::{FileSystemService, CacheService, PreviewService};

#[derive(Clone)]
pub struct Services {
    pub file_system: Arc<dyn FileSystemService>,
    pub cache: Arc<CacheService>,
    pub preview: Arc<dyn PreviewService>,
}

impl Services {
    pub fn new() -> anyhow::Result<Self> {
        let cache = Arc::new(CacheService::new()?);
        let file_system = Arc::new(NativeFileSystemService::new(cache.clone())?);
        let preview = Arc::new(MultiFormatPreviewService::new(cache.clone())?);
        
        Ok(Self {
            file_system,
            cache,
            preview,
        })
    }
}
```

#### 1.3 Basic File System Service

```rust
// src/services/file_system.rs
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::models::{FileEntry, FileType};
use crate::error::FileSystemError;

#[async_trait]
pub trait FileSystemService: Send + Sync {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError>;
    async fn get_metadata(&self, path: &Path) -> Result<std::fs::Metadata, FileSystemError>;
    async fn create_directory(&self, path: &Path) -> Result<(), FileSystemError>;
    async fn copy_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn move_files(&self, sources: &[PathBuf], dest: &Path) -> Result<(), FileSystemError>;
    async fn delete_files(&self, paths: &[PathBuf]) -> Result<(), FileSystemError>;
}

pub struct NativeFileSystemService {
    cache: Arc<CacheService>,
}

impl NativeFileSystemService {
    pub fn new(cache: Arc<CacheService>) -> Result<Self, FileSystemError> {
        Ok(Self { cache })
    }
}

#[async_trait]
impl FileSystemService for NativeFileSystemService {
    async fn list_directory(&self, path: &Path) -> Result<Vec<FileEntry>, FileSystemError> {
        let path = path.to_path_buf();
        
        tokio::task::spawn_blocking(move || {
            let mut entries = Vec::new();
            
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                let path = entry.path();
                let metadata = entry.metadata()?;
                
                let file_entry = FileEntry {
                    path: path.clone(),
                    name: path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    file_type: FileType::from_path(&path),
                    size: metadata.len(),
                    modified: metadata.modified().unwrap_or(std::time::UNIX_EPOCH),
                    created: metadata.created().unwrap_or(std::time::UNIX_EPOCH),
                    is_directory: metadata.is_dir(),
                    is_hidden: is_hidden(&path),
                    permissions: get_permissions(&metadata),
                    thumbnail: None,
                    metadata: None,
                };
                
                entries.push(file_entry);
            }
            
            // Sort entries: directories first, then by name
            entries.sort_by(|a, b| {
                match (a.is_directory, b.is_directory) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.cmp(&b.name),
                }
            });
            
            Ok(entries)
        }).await
        .map_err(|e| FileSystemError::TaskJoinError(e))?
    }
    
    // Implement other methods...
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(windows)]
fn get_permissions(metadata: &std::fs::Metadata) -> FilePermissions {
    use std::os::windows::fs::MetadataExt;
    // Windows-specific permission handling
    FilePermissions::default()
}

#[cfg(unix)]
fn get_permissions(metadata: &std::fs::Metadata) -> FilePermissions {
    use std::os::unix::fs::PermissionsExt;
    let mode = metadata.permissions().mode();
    
    FilePermissions {
        readable: mode & 0o400 != 0,
        writable: mode & 0o200 != 0,
        executable: mode & 0o100 != 0,
    }
}
```

### Phase 2: UI Framework (Week 3-4)

#### 2.1 Layout Components

```rust
// src/ui/layout.rs
use dioxus::prelude::*;
use crate::state::AppState;

#[derive(Props)]
pub struct MainLayoutProps {
    children: Element,
}

pub fn MainLayout(cx: Scope<MainLayoutProps>) -> Element {
    let app_state = use_shared_state::<AppState>(cx).unwrap();
    let layout = &app_state.read().ui.layout;
    
    render! {
        div {
            class: "main-layout",
            style: "
                display: flex;
                flex-direction: column;
                height: 100vh;
                overflow: hidden;
            ",
            
            TopBar {}
            
            div {
                class: "content-area",
                style: "
                    display: flex;
                    flex: 1;
                    overflow: hidden;
                ",
                
                if layout.is_left_panel_visible {
                    render! {
                        LeftPanel {
                            width: layout.left_panel_width,
                        }
                        ResizeHandle {
                            orientation: ResizeOrientation::Vertical,
                        }
                    }
                }
                
                RightPanel {
                    width: if layout.is_left_panel_visible {
                        layout.right_panel_width
                    } else {
                        layout.left_panel_width + layout.right_panel_width
                    },
                }
            }
            
            if layout.is_bottom_panel_visible {
                render! {
                    ResizeHandle {
                        orientation: ResizeOrientation::Horizontal,
                    }
                    BottomPanel {
                        height: layout.bottom_panel_height,
                    }
                }
            }
        }
    }
}

pub fn TopBar(cx: Scope) -> Element {
    render! {
        div {
            class: "top-bar",
            style: "
                height: 40px;
                background: #2d2d30;
                border-bottom: 1px solid #3e3e42;
                display: flex;
                align-items: center;
                padding: 0 12px;
            ",
            
            MenuBar {}
            div { class: "spacer", style: "flex: 1;" }
            WindowControls {}
        }
    }
}
```

#### 2.2 Virtual Scrolling Implementation

```rust
// src/ui/components/virtual_scroll.rs
use dioxus::prelude::*;
use std::rc::Rc;

#[derive(Props)]
pub struct VirtualScrollProps<T: Clone + PartialEq + 'static> {
    items: Vec<T>,
    item_height: f32,
    container_height: f32,
    render_item: Rc<dyn Fn(&T, usize) -> Element>,
}

pub fn VirtualScroll<T: Clone + PartialEq + 'static>(cx: Scope<VirtualScrollProps<T>>) -> Element {
    let scroll_top = use_state(cx, || 0.0);
    let container_ref = use_ref(cx, || None::<web_sys::Element>);
    
    // Calculate visible range
    let item_height = cx.props.item_height;
    let container_height = cx.props.container_height;
    let total_items = cx.props.items.len();
    
    let visible_count = (container_height / item_height).ceil() as usize + 2; // +2 for buffer
    let start_index = (*scroll_top.get() / item_height).floor() as usize;
    let end_index = (start_index + visible_count).min(total_items);
    
    let visible_items: Vec<(usize, &T)> = cx.props.items
        .iter()
        .enumerate()
        .skip(start_index)
        .take(end_index - start_index)
        .collect();
    
    let total_height = total_items as f32 * item_height;
    let offset_y = start_index as f32 * item_height;
    
    render! {
        div {
            class: "virtual-scroll-container",
            style: "
                height: {container_height}px;
                overflow-y: auto;
                position: relative;
            ",
            onscroll: move |event| {
                if let Some(target) = event.target_dyn_into::<web_sys::Element>() {
                    scroll_top.set(target.scroll_top() as f32);
                }
            },
            
            // Spacer to maintain scroll height
            div {
                style: "height: {total_height}px; position: relative;",
                
                // Visible items container
                div {
                    style: "
                        position: absolute;
                        top: {offset_y}px;
                        width: 100%;
                    ",
                    
                    for (index, item) in visible_items {
                        div {
                            key: "{index}",
                            style: "height: {item_height}px;",
                            (cx.props.render_item)(item, index)
                        }
                    }
                }
            }
        }
    }
}
```

### Phase 3: File Preview System (Week 5-6)

#### 3.1 Preview Service Implementation

```rust
// src/services/preview.rs
use async_trait::async_trait;
use image::DynamicImage;
use std::path::Path;
use crate::models::{Thumbnail, PreviewContent, FileMetadata};
use crate::error::PreviewError;

#[async_trait]
pub trait PreviewService: Send + Sync {
    async fn generate_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Result<Thumbnail, PreviewError>;
    async fn get_preview(&self, path: &Path) -> Result<PreviewContent, PreviewError>;
    async fn extract_metadata(&self, path: &Path) -> Result<FileMetadata, PreviewError>;
}

pub struct MultiFormatPreviewService {
    image_handler: ImagePreviewHandler,
    #[cfg(feature = "video")]
    video_handler: VideoPreviewHandler,
    #[cfg(feature = "audio")]
    audio_handler: AudioPreviewHandler,
    #[cfg(feature = "pdf")]
    pdf_handler: PdfPreviewHandler,
    cache: Arc<CacheService>,
}

impl MultiFormatPreviewService {
    pub fn new(cache: Arc<CacheService>) -> Result<Self, PreviewError> {
        Ok(Self {
            image_handler: ImagePreviewHandler::new(),
            #[cfg(feature = "video")]
            video_handler: VideoPreviewHandler::new()?,
            #[cfg(feature = "audio")]
            audio_handler: AudioPreviewHandler::new()?,
            #[cfg(feature = "pdf")]
            pdf_handler: PdfPreviewHandler::new()?,
            cache,
        })
    }
}

#[async_trait]
impl PreviewService for MultiFormatPreviewService {
    async fn generate_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Result<Thumbnail, PreviewError> {
        // Check cache first
        if let Some(cached) = self.cache.get_thumbnail(path, size).await {
            return Ok(cached);
        }
        
        let file_type = FileType::from_path(path);
        let thumbnail = match file_type {
            FileType::Image(_) => {
                self.image_handler.generate_thumbnail(path, size).await?
            }
            #[cfg(feature = "video")]
            FileType::Video(_) => {
                self.video_handler.generate_thumbnail(path, size).await?
            }
            #[cfg(feature = "pdf")]
            FileType::Document(DocumentFormat::Pdf) => {
                self.pdf_handler.generate_thumbnail(path, size).await?
            }
            _ => {
                // Generate generic file icon
                self.generate_generic_thumbnail(&file_type, size)?
            }
        };
        
        // Cache the result
        self.cache.cache_thumbnail(path, size, thumbnail.clone()).await;
        
        Ok(thumbnail)
    }
    
    // Implement other methods...
}

pub struct ImagePreviewHandler;

impl ImagePreviewHandler {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn generate_thumbnail(&self, path: &Path, size: ThumbnailSize) -> Result<Thumbnail, PreviewError> {
        let path = path.to_path_buf();
        let (width, height) = size.dimensions();
        
        tokio::task::spawn_blocking(move || {
            let img = image::open(&path)?;
            let thumbnail = img.thumbnail(width, height);
            
            // Convert to base64 data URL
            let mut buffer = Vec::new();
            thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)?;
            let data_url = format!("data:image/png;base64,{}", base64::encode(&buffer));
            
            Ok(Thumbnail {
                data_url,
                width: thumbnail.width(),
                height: thumbnail.height(),
                format: ThumbnailFormat::Png,
            })
        }).await
        .map_err(|e| PreviewError::TaskJoinError(e))?
    }
}
```

### Phase 4: Advanced Features (Week 7-8)

#### 4.1 Search Implementation

```rust
// src/services/search.rs
use std::path::{Path, PathBuf};
use regex::Regex;
use crate::models::{FileEntry, SearchCriteria, SearchResult};
use crate::error::SearchError;

pub struct SearchService {
    file_system: Arc<dyn FileSystemService>,
}

impl SearchService {
    pub fn new(file_system: Arc<dyn FileSystemService>) -> Self {
        Self { file_system }
    }
    
    pub async fn search(&self, criteria: SearchCriteria) -> Result<Vec<SearchResult>, SearchError> {
        let root_paths = criteria.root_paths.clone();
        let criteria = criteria.clone();
        
        tokio::task::spawn_blocking(move || {
            let mut results = Vec::new();
            
            for root_path in root_paths {
                let walker = walkdir::WalkDir::new(&root_path)
                    .max_depth(criteria.max_depth.unwrap_or(100))
                    .follow_links(criteria.follow_links);
                
                for entry in walker {
                    let entry = entry?;
                    let path = entry.path();
                    
                    if self.matches_criteria(path, &criteria)? {
                        let metadata = entry.metadata()?;
                        let file_entry = FileEntry::from_path_and_metadata(path, &metadata)?;
                        
                        let result = SearchResult {
                            file_entry,
                            relevance_score: self.calculate_relevance(path, &criteria),
                            matched_content: None, // TODO: Implement content matching
                        };
                        
                        results.push(result);
                    }
                }
            }
            
            // Sort by relevance
            results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());
            
            Ok(results)
        }).await
        .map_err(|e| SearchError::TaskJoinError(e))?
    }
    
    fn matches_criteria(&self, path: &Path, criteria: &SearchCriteria) -> Result<bool, SearchError> {
        // Name pattern matching
        if let Some(ref pattern) = criteria.name_pattern {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if criteria.use_regex {
                let regex = Regex::new(pattern)?;
                if !regex.is_match(name) {
                    return Ok(false);
                }
            } else {
                if !name.to_lowercase().contains(&pattern.to_lowercase()) {
                    return Ok(false);
                }
            }
        }
        
        // File type filtering
        if !criteria.file_types.is_empty() {
            let file_type = FileType::from_path(path);
            if !criteria.file_types.contains(&file_type) {
                return Ok(false);
            }
        }
        
        // Size filtering
        if let (Some(min_size), Some(max_size)) = (criteria.min_size, criteria.max_size) {
            if let Ok(metadata) = std::fs::metadata(path) {
                let size = metadata.len();
                if size < min_size || size > max_size {
                    return Ok(false);
                }
            }
        }
        
        // Date filtering
        if let (Some(after), Some(before)) = (criteria.modified_after, criteria.modified_before) {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    if modified < after || modified > before {
                        return Ok(false);
                    }
                }
            }
        }
        
        Ok(true)
    }
    
    fn calculate_relevance(&self, path: &Path, criteria: &SearchCriteria) -> f32 {
        let mut score = 1.0;
        
        // Boost score for exact name matches
        if let Some(ref pattern) = criteria.name_pattern {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            if name.to_lowercase() == pattern.to_lowercase() {
                score += 10.0;
            } else if name.to_lowercase().starts_with(&pattern.to_lowercase()) {
                score += 5.0;
            }
        }
        
        // Boost score for preferred file types
        if criteria.preferred_types.contains(&FileType::from_path(path)) {
            score += 2.0;
        }
        
        score
    }
}
```

## Development Workflow

### 1. Environment Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install system dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libssl-dev \
    libsqlite3-dev

# For video support (optional)
sudo apt install -y \
    libavformat-dev \
    libavcodec-dev \
    libavutil-dev \
    libswscale-dev

# For PDF support (optional)
sudo apt install -y \
    libpoppler-glib-dev
```

### 2. Development Commands

```bash
# Run in development mode
cargo run

# Run with specific features
cargo run --features "video,audio,pdf"

# Build for release
cargo build --release

# Run tests
cargo test

# Run clippy for linting
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check for security vulnerabilities
cargo audit
```

### 3. Testing Strategy

```rust
// tests/integration_tests.rs
use tempfile::TempDir;
use media_organizer::services::FileSystemService;

#[tokio::test]
async fn test_list_directory() {
    let temp_dir = TempDir::new().unwrap();
    let service = NativeFileSystemService::new(Arc::new(CacheService::new().unwrap())).unwrap();
    
    // Create test files
    std::fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();
    std::fs::create_dir(temp_dir.path().join("subdir")).unwrap();
    
    let entries = service.list_directory(temp_dir.path()).await.unwrap();
    
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|e| e.name == "test.txt"));
    assert!(entries.iter().any(|e| e.name == "subdir"));
}
```

This implementation guide provides a solid foundation for building the MediaOrganizer application with proper architecture, error handling, and extensibility for future enhancements.