# MediaOrganizer - Requirements Document

## Project Overview

**MediaOrganizer**는 Dioxus 프레임워크를 사용하여 개발되는 크로스플랫폼 미디어/파일 관리 애플리케이션입니다. Visual Studio Code와 유사한 인터페이스를 제공하며, 다양한 파일 형식에 대한 미리보기와 관리 기능을 제공합니다.

## Technical Stack

- **Framework**: Dioxus (Rust-based cross-platform UI framework)
- **Target Platforms**: Windows, macOS, Linux
- **Language**: Rust
- **UI Pattern**: Desktop application with native file system access

## Core Features

### 1. User Interface Layout

#### Main Layout (VS Code Style)
- **Left Panel**: File tree navigator (30% width, resizable)
- **Right Panel**: Content viewer/file grid (70% width, resizable)
- **Top Bar**: Application menu and toolbar
- **Bottom Bar**: Status information (selected files count, total size, etc.)

#### Left Panel - File Tree Navigator
- Hierarchical folder structure display
- Expandable/collapsible folders
- File and folder icons based on type
- Context menu support (right-click)
- Drag and drop support
- Keyboard navigation (arrow keys, Enter, Space)

#### Right Panel - Content Viewer
- **Folder View**: macOS Finder-style grid layout
  - Adjustable icon sizes (small, medium, large, extra large)
  - List view option with detailed information
  - Sorting options (name, date, size, type)
  - File thumbnails where applicable
- **File Preview**: Content display based on file type
- **File Operations Toolbar**: Action buttons for selected files

### 2. File Type Support & Preview

#### Image Files
- **Supported Formats**: JPEG, PNG, GIF, WebP, TIFF, BMP, SVG
- **Features**:
  - High-quality image preview with zoom controls
  - Image metadata display (dimensions, file size, date taken, EXIF data)
  - Slideshow mode for multiple images
  - Basic image operations (rotate, flip)

#### Video Files
- **Supported Formats**: MP4, AVI, MOV, WMV, MKV, WebM
- **Features**:
  - Built-in video player with standard controls
  - Video metadata display (duration, resolution, codec, bitrate)
  - Thumbnail generation and preview
  - Frame-by-frame navigation

#### Document Files
- **Markdown (.md)**: Rendered preview with syntax highlighting
- **PDF**: Built-in PDF viewer with page navigation
- **Microsoft Office**: 
  - PowerPoint (.ppt, .pptx): Slide preview and navigation
  - Word (.doc, .docx): Document preview (read-only)
  - Excel (.xls, .xlsx): Spreadsheet preview
- **Apple Keynote**: Slide preview (if supported by platform)
- **Text Files**: Syntax-highlighted text editor view

#### Audio Files
- **Supported Formats**: MP3, WAV, FLAC, AAC, OGG
- **Features**:
  - Built-in audio player
  - Waveform visualization
  - Metadata display (artist, album, duration, bitrate)

### 3. File Operations

#### Core Operations
- **Move**: Move files/folders to different locations
- **Copy**: Create copies of files/folders
- **Delete**: Move to trash/recycle bin with undo capability
- **Duplicate**: Create copies in same directory with naming convention
- **Share**: Platform-specific sharing options (email, cloud services)

#### Destination Management
- **Favorite Destinations**: Quick-access list of frequently used folders
- **Custom Shortcuts**: User-defined keyboard shortcuts for each destination
- **Recent Destinations**: History of recently used move/copy targets
- **Drag & Drop**: Visual drag and drop between panels

#### Batch Operations
- Multi-file selection support (Ctrl+click, Shift+click, Ctrl+A)
- Batch operations on selected files
- Progress indicators for long-running operations
- Operation queue with cancel/pause capability

### 4. Advanced Features

#### File Information Panel
- **General Info**: File size, creation date, modification date, permissions
- **Type-Specific Info**:
  - Images: Dimensions, color profile, camera info
  - Videos: Duration, resolution, frame rate, codec
  - Audio: Duration, bitrate, sample rate, artist/album
  - Documents: Page count, word count, author
- **Storage Info**: Disk usage, file path, symbolic link targets

#### Search & Filter
- **Quick Search**: Real-time filename filtering
- **Advanced Search**: Content-based search for supported file types
- **Filter Options**: By file type, size range, date range
- **Saved Searches**: Bookmark frequently used search criteria

#### Folder Analysis
- **Size Calculation**: Recursive folder size calculation with caching
- **File Type Distribution**: Breakdown of file types within folders
- **Duplicate Detection**: Find duplicate files based on content hash
- **Large File Detection**: Identify files above specified size thresholds

### 5. Performance Requirements

#### File System Operations
- **Large Directory Handling**: Efficient loading of folders with 10,000+ files
- **Thumbnail Generation**: Background thumbnail creation with caching
- **Memory Management**: Lazy loading of file previews and metadata
- **Responsive UI**: Non-blocking operations with progress feedback

#### Caching Strategy
- **Thumbnail Cache**: Persistent thumbnail storage for faster loading
- **Metadata Cache**: Cache file information to reduce filesystem calls
- **Preview Cache**: Cache rendered previews for documents and media

### 6. User Experience

#### Keyboard Shortcuts
- **Navigation**: Arrow keys, Page Up/Down, Home/End
- **Selection**: Ctrl+A (select all), Ctrl+click (multi-select)
- **Operations**: Ctrl+C (copy), Ctrl+X (cut), Ctrl+V (paste), Delete
- **Custom Shortcuts**: User-defined shortcuts for destination folders
- **Quick Actions**: Space (preview), Enter (open), F2 (rename)

#### Accessibility
- **Screen Reader Support**: Proper ARIA labels and descriptions
- **High Contrast Mode**: Support for system high contrast themes
- **Keyboard Navigation**: Full keyboard accessibility
- **Font Scaling**: Respect system font size preferences

### 7. Platform Integration

#### Native OS Features
- **File Associations**: Respect system default applications
- **Context Menus**: Integration with OS context menu items
- **Notifications**: System notifications for completed operations
- **Clipboard Integration**: Standard copy/paste operations

#### Cross-Platform Considerations
- **Path Handling**: Proper handling of different path separators
- **File Permissions**: Platform-appropriate permission handling
- **System Integration**: Platform-specific features where available

## Development Phases

### Phase 1: Core Infrastructure
- Basic Dioxus application setup
- File system navigation
- Basic file operations (copy, move, delete)
- Simple file listing

### Phase 2: UI Framework
- VS Code-style layout implementation
- Resizable panels
- File tree component
- Basic grid view

### Phase 3: File Preview System
- Image preview implementation
- Video player integration
- Document viewer foundation
- Metadata extraction

### Phase 4: Advanced Features
- Destination management
- Keyboard shortcuts
- Search and filter
- Batch operations

### Phase 5: Performance & Polish
- Thumbnail caching
- Performance optimization
- Error handling
- User testing and refinement

## Technical Considerations

### Dependencies
- **File System**: Use Rust's `std::fs` and `walkdir` for file operations
- **Media Processing**: 
  - Images: `image` crate for loading and basic manipulation
  - Video: `ffmpeg` bindings for metadata and thumbnails
  - Audio: `rodio` or `symphonia` for audio playback
- **UI Components**: Custom Dioxus components for file grids and trees
- **Async Operations**: `tokio` for non-blocking file operations

### Performance Targets
- **Startup Time**: < 3 seconds for application launch
- **Directory Loading**: < 1 second for folders with up to 1000 files
- **Preview Generation**: < 500ms for common file types
- **Memory Usage**: < 200MB baseline, scalable with content

### Error Handling
- Graceful handling of permission errors
- Network drive timeout handling
- Corrupted file detection and reporting
- User-friendly error messages with suggested actions

## Success Criteria

1. **Functionality**: All core file operations work reliably across platforms
2. **Performance**: Smooth interaction with large file collections
3. **Usability**: Intuitive interface comparable to professional file managers
4. **Stability**: No crashes during normal usage patterns
5. **Cross-Platform**: Consistent behavior and appearance across Windows, macOS, and Linux

## Future Enhancements

- Cloud storage integration (Google Drive, Dropbox, OneDrive)
- Advanced image editing capabilities
- Plugin system for custom file type handlers
- Network file sharing capabilities
- Advanced duplicate file management
- File organization suggestions using AI/ML