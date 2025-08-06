# MediaOrganizer

[![Build Status](https://github.com/organization/media-organizer/workflows/CI/badge.svg)](https://github.com/organization/media-organizer/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-%3E%3D1.70-orange.svg)](https://www.rust-lang.org/)

A high-performance, cross-platform media and file management application built with Rust and Dioxus, featuring a Visual Studio Code-style interface with comprehensive preview capabilities.

![MediaOrganizer Preview](docs/images/app-preview.png)

## ✨ Features

### 🎨 Modern Interface

- **VS Code-Style Layout**: Familiar split-panel design with resizable panes
- **Multiple View Modes**: Grid, list, and preview layouts
- **Dark/Light Themes**: System-aware theme switching
- **Keyboard Navigation**: Comprehensive shortcuts and accessibility support

### ⚡ High Performance

- **Virtual Scrolling**: Handle 10,000+ files smoothly
- **Background Processing**: Non-blocking thumbnail generation and metadata extraction
- **Smart Caching**: Persistent thumbnail and metadata cache with automatic cleanup
- **Memory Efficient**: Lazy loading with configurable resource limits

### 📁 File Management

- **Cross-Platform Operations**: Copy, move, delete, rename with proper error handling
- **Batch Operations**: Multi-file selection and processing with progress tracking
- **Undo/Redo**: Operation history with rollback capability
- **Drag & Drop**: Intuitive file operations between panels

### 🔍 Advanced Preview

- **Images**: JPEG, PNG, GIF, WebP, TIFF, BMP, SVG with EXIF data
- **Videos**: MP4, AVI, MOV, WMV, MKV, WebM with metadata display
- **Audio**: MP3, WAV, FLAC, AAC, OGG with waveform visualization
- **Documents**: PDF viewer, Markdown rendering, text files with syntax highlighting

### 🔎 Search & Organization

- **Real-time Search**: Instant filename filtering with regex support
- **Advanced Filters**: Filter by type, size, date range
- **Duplicate Detection**: Content-based duplicate finding with background processing
- **Favorite Destinations**: Quick-access folder shortcuts

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.70 or higher
- **System Dependencies**:
  - Linux: `libgtk-3-dev`, `libwebkit2gtk-4.0-dev`, `libssl-dev`
  - macOS: Xcode Command Line Tools
  - Windows: Visual Studio Build Tools

### Installation

```bash
# Clone the repository
git clone https://github.com/organization/media-organizer.git
cd media-organizer

# Install dependencies and build
cargo build --release

# Run the application
cargo run --release
```

### Quick Development Setup

```bash
# Install development tools
cargo install cargo-watch cargo-audit

# Run in development mode with auto-reload
cargo watch -x run

# Run tests
cargo test
```

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [**Project Index**](PROJECT_INDEX.md) | Complete documentation navigation |
| [**API Reference**](API_REFERENCE.md) | Comprehensive API documentation |
| [**Implementation Guide**](IMPLEMENTATION_GUIDE.md) | Step-by-step development guide |
| [**Development Workflow**](DEVELOPMENT_WORKFLOW.md) | Development process and standards |
| [**Architecture Design**](DESIGN_ARCHITECTURE.md) | System architecture overview |

## 🏗️ Architecture

MediaOrganizer follows a layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────┐
│         Presentation Layer          │  ← Dioxus UI Components
├─────────────────────────────────────┤
│        Business Logic Layer        │  ← Services & State Management
├─────────────────────────────────────┤
│         Data Access Layer          │  ← File System & Cache
├─────────────────────────────────────┤
│          Platform Layer            │  ← OS-Specific APIs
└─────────────────────────────────────┘
```

### Key Components

- **FileSystemService**: Cross-platform file operations
- **PreviewService**: Multi-format preview generation
- **CacheService**: SQLite metadata and file-based thumbnail cache
- **Virtual Scrolling**: Efficient rendering for large directories
- **State Management**: Centralized state with event-driven updates

## 🛠️ Development

### Project Structure

```
src/
├── main.rs                 # Application entry point
├── ui/                     # Dioxus UI components
│   ├── file_tree.rs       # File navigator
│   ├── content_viewer.rs  # File content display
│   └── components/        # Reusable components
├── services/              # Business logic services
│   ├── file_system.rs     # File operations
│   ├── preview.rs         # Preview generation
│   └── cache.rs           # Caching system
├── state/                 # State management
└── models/                # Data structures
```

### Development Commands

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run tests with coverage
cargo test

# Build for release
cargo build --release

# Cross-compile for Windows
cargo build --target x86_64-pc-windows-gnu
```

### Performance Targets

- **Startup Time**: < 3 seconds
- **Directory Loading**: < 1 second for 1,000 files, < 1 second for 10,000+ files (virtual)
- **Preview Generation**: < 500ms for common file types
- **Memory Usage**: < 200MB baseline, efficient scaling

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test file_system

# Run integration tests
cargo test --test integration

# Benchmark performance
cargo bench
```

## 📦 Building & Distribution

### Feature Flags

```toml
[features]
default = ["video", "audio", "pdf"]
video = ["ffmpeg-next"]          # Video preview support
audio = ["rodio"]                # Audio playback support
pdf = ["pdf", "poppler"]         # PDF rendering support
```

### Build Commands

```bash
# Full feature build
cargo build --release --all-features

# Minimal build (images and text only)
cargo build --release --no-default-features

# Platform-specific builds
cargo build --release --target x86_64-pc-windows-gnu    # Windows
cargo build --release --target x86_64-apple-darwin      # macOS
cargo build --release --target x86_64-unknown-linux-gnu # Linux
```

## 🤝 Contributing

We welcome contributions! Please see our [Development Workflow](DEVELOPMENT_WORKFLOW.md) for detailed guidelines.

### Quick Contribution Guide

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Follow** our coding standards (see [Development Workflow](DEVELOPMENT_WORKFLOW.md))
4. **Test** your changes: `cargo test`
5. **Commit** with clear messages: `git commit -m "feat: add amazing feature"`
6. **Push** to your branch: `git push origin feature/amazing-feature`
7. **Submit** a pull request

### Development Environment

```bash
# Set up pre-commit hooks
cp .git/hooks/pre-commit.sample .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Install additional tools
cargo install cargo-watch cargo-audit cargo-outdated
```

## 📋 Roadmap

### Phase 1: Core Infrastructure ✅

- [x] Basic application setup
- [x] File system navigation
- [x] Virtual scrolling foundation
- [x] System architecture design

### Phase 2: UI Framework (In Progress)

- [ ] VS Code-style layout implementation
- [ ] File tree component
- [ ] Content viewer with grid/list modes
- [ ] Search and filter functionality

### Phase 3: Preview System (Planned)

- [ ] Image preview with EXIF data
- [ ] Video player integration
- [ ] Audio player with waveform
- [ ] PDF and document viewers

### Phase 4: Advanced Features (Planned)

- [ ] Background duplicate detection
- [ ] Batch operations with progress
- [ ] Destination management
- [ ] Comprehensive error handling

### Future Enhancements

- Cloud storage integration (Google Drive, Dropbox, OneDrive)
- Plugin system for custom file handlers
- Advanced image editing capabilities
- AI-powered file organization suggestions

## 🔧 Configuration

MediaOrganizer uses a configuration file located at:

- **Linux**: `~/.config/media-organizer/config.toml`
- **macOS**: `~/Library/Application Support/media-organizer/config.toml`
- **Windows**: `%APPDATA%\media-organizer\config.toml`

Example configuration:

```toml
[ui]
theme = "system"  # "light", "dark", or "system"
default_view_mode = "grid"
icon_size = "medium"

[performance]
max_memory_usage = 536870912  # 512MB
thumbnail_cache_size = 1000
virtual_scroll_buffer = 10

[cache]
thumbnail_cache_dir = "~/.cache/media-organizer/thumbnails"
metadata_db_path = "~/.cache/media-organizer/metadata.db"
cleanup_interval_days = 30
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Dioxus](https://dioxuslabs.com/) - Modern Rust UI framework
- [Tauri](https://tauri.app/) - Inspiration for cross-platform desktop apps
- [VS Code](https://code.visualstudio.com/) - UI design inspiration
- [Rust Community](https://www.rust-lang.org/community) - Amazing ecosystem and support

## 📞 Support

- **Documentation**: [Project Index](PROJECT_INDEX.md)
- **Issues**: [GitHub Issues](https://github.com/organization/media-organizer/issues)
- **Discussions**: [GitHub Discussions](https://github.com/organization/media-organizer/discussions)
- **Wiki**: [Project Wiki](https://github.com/organization/media-organizer/wiki)

---

<p align="center">
  Made with ❤️ and 🦀 Rust
</p>
