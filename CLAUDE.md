# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MediaOrganizer is a cross-platform media/file management application built with Dioxus (Rust framework), featuring a VS Code-style interface with comprehensive file preview and management capabilities.

**Current Status**: Early Implementation Phase - Basic structure implemented, building successfully

## Technical Stack

- **Framework**: Dioxus (Rust-based cross-platform UI framework)
- **Language**: Rust
- **Target Platforms**: Windows, macOS, Linux
- **Build System**: Cargo (when implemented)

## Project Architecture

### Core Application Structure (Planned)
```
MediaOrganizer/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── app.rs                  # Main app component and state management
│   ├── ui/
│   │   ├── mod.rs              # UI module definitions
│   │   ├── layout.rs           # VS Code-style layout implementation
│   │   ├── file_tree.rs        # Left panel file navigator
│   │   ├── content_viewer.rs   # Right panel content display
│   │   └── components/         # Reusable UI components
│   ├── file_system/
│   │   ├── mod.rs              # File system operations
│   │   ├── navigation.rs       # Directory traversal
│   │   ├── operations.rs       # File operations (copy, move, delete)
│   │   └── metadata.rs         # File metadata extraction
│   ├── preview/
│   │   ├── mod.rs              # Preview system
│   │   ├── image.rs            # Image preview handler
│   │   ├── video.rs            # Video preview handler
│   │   ├── audio.rs            # Audio preview handler
│   │   └── document.rs         # Document preview handler
│   ├── cache/
│   │   ├── mod.rs              # Caching system
│   │   ├── thumbnails.rs       # Thumbnail cache
│   │   └── metadata.rs         # Metadata cache
│   └── utils/
│       ├── mod.rs              # Utility functions
│       ├── async_ops.rs        # Async file operations
│       └── virtual_scroll.rs   # Virtual scrolling implementation
└── assets/                     # Static assets (icons, themes)
```

### Key Design Patterns

#### UI Architecture
- **Split Panel Layout**: VS Code-style with resizable left (file tree) and right (content viewer) panels
- **Component-Based**: Dioxus functional components with hooks for state management
- **Virtual Scrolling**: Essential for handling 10,000+ files efficiently
- **Async Operations**: Non-blocking file operations using Tokio

#### File System Integration
- **Cross-Platform Path Handling**: Proper abstraction for different OS path separators
- **Permission-Aware**: Graceful handling of permission errors and restricted access
- **Caching Strategy**: Multi-level caching (thumbnails, metadata, previews)

#### Performance Optimization
- **Lazy Loading**: Load file previews and metadata on-demand
- **Background Processing**: Thumbnail generation and duplicate detection in background threads
- **Memory Management**: Efficient handling of large file collections

## Development Commands (To Be Implemented)

Once the project is implemented, these commands will be available:

```bash
# Development
cargo run                    # Run the application in development mode
cargo build                  # Build the application
cargo build --release       # Build optimized release version

# Testing
cargo test                   # Run all tests
cargo test --lib            # Run unit tests only
cargo test --bin            # Run integration tests

# Code Quality
cargo clippy                 # Run linter
cargo fmt                    # Format code
cargo check                  # Fast compilation check

# Platform-Specific Builds
cargo build --target x86_64-pc-windows-gnu     # Windows
cargo build --target x86_64-apple-darwin       # macOS
cargo build --target x86_64-unknown-linux-gnu  # Linux
```

## Key Dependencies (Planned)

### Core Framework
- `dioxus` - Cross-platform UI framework
- `dioxus-desktop` - Desktop application support
- `tokio` - Async runtime for non-blocking operations

### File System & Media Processing
- `walkdir` - Recursive directory traversal
- `image` - Image loading and basic manipulation
- `ffmpeg-next` - Video metadata and thumbnail generation (with static build consideration)
- `rodio` or `symphonia` - Audio playback and metadata
- `pdf-rs` or similar - PDF rendering
- `pulldown-cmark` - Markdown rendering

### Utilities
- `serde` - Serialization for configuration and cache
- `dirs` - Cross-platform directory detection
- `notify` - File system change monitoring
- `rayon` - Parallel processing for intensive operations

## Development Phases

### Phase 1: Core Infrastructure
**Focus**: Basic application foundation
- Dioxus application setup with desktop target
- Basic file system navigation using `walkdir`
- Simple file listing with virtual scrolling foundation
- Core file operations (copy, move, delete) with proper error handling

### Phase 2: UI Framework & Essential Features
**Focus**: User interface and search capabilities
- VS Code-style resizable panel layout
- File tree component with expand/collapse functionality
- Grid view with adjustable icon sizes
- Search and filter implementation (prioritized for early user feedback)

### Phase 3: File Preview System
**Focus**: Content display capabilities
- Image preview with zoom controls and EXIF data
- Basic video player with metadata display
- PDF and Markdown viewers
- Text file editor with syntax highlighting
- Audio player with waveform visualization

### Phase 4: Advanced Features
**Focus**: Power user functionality
- Destination management with favorites and shortcuts
- Comprehensive keyboard shortcuts
- Background duplicate detection with progress indication
- Batch operations with queue management

### Phase 5: Performance & Polish
**Focus**: Optimization and refinement
- Thumbnail caching optimization
- Large directory performance tuning
- Comprehensive error handling and user feedback
- Cross-platform testing and UI consistency

## Performance Targets

- **Startup Time**: < 3 seconds
- **Directory Loading**: < 1 second for up to 1,000 files
- **Large Directory**: < 1 second for 10,000+ files (with virtual scrolling)
- **Preview Generation**: < 500ms for common file types
- **Memory Usage**: < 200MB baseline, efficient scaling with content

## Critical Implementation Considerations

### FFmpeg Integration Strategy
- **Current Status**: ⚠️ **Video features temporarily disabled** due to FFmpeg compatibility issues
- **Issue**: `ffmpeg-next` crate v6.1.1 has breaking changes incompatible with current FFmpeg versions
- **Workaround**: Video features disabled in default build configuration
- **Future Resolution**: 
  - Monitor `ffmpeg-next` updates for compatibility fixes
  - Consider alternative video processing crates
  - Implement fallback metadata extraction without FFmpeg

### Microsoft Office File Support
- **Challenge**: Limited Rust ecosystem support for proprietary formats
- **Phased Approach**: 
  - Phase 1: Basic metadata display only
  - Phase 2: WebView-based preview using online services
  - Future: Native rendering (long-term goal)

### Virtual Scrolling Implementation
- **Critical**: Essential for 10,000+ file performance target
- **Implementation**: Custom virtual scrolling component for Dioxus
- **Memory**: Only render visible items to maintain low memory footprint

### Error Handling Strategy
- **User-Friendly**: Provide actionable error messages
- **Graceful Degradation**: Continue functioning when individual features fail
- **Logging**: Comprehensive logging for debugging without exposing sensitive paths

## File Structure Conventions

When implementing:
- Use `mod.rs` files for module organization
- Implement `Display` and `Debug` traits for custom types
- Use `Result<T, E>` types for all fallible operations
- Prefer composition over inheritance for UI components
- Use `async/await` for all I/O operations

## Testing Strategy

- **Unit Tests**: Core logic and file operations
- **Integration Tests**: UI component interactions
- **Performance Tests**: Large directory handling benchmarks
- **Cross-Platform Tests**: Ensure consistent behavior across OS

## Security Considerations

- **Path Traversal**: Validate all file paths to prevent directory traversal attacks
- **Permission Handling**: Respect OS file permissions and provide clear error messages
- **Sandboxing**: Consider OS-specific sandboxing requirements for file access
- **User Data**: Never log or expose sensitive file paths or contents