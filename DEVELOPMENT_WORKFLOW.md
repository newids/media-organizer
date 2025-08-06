# MediaOrganizer - Development Workflow

## Overview

This document outlines the complete development workflow for the MediaOrganizer project, including environment setup, coding standards, testing procedures, and release processes.

## 🛠️ Development Environment Setup

### Prerequisites

#### System Requirements
- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 18.04+)
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 5GB free space for development environment
- **Network**: Internet connection for dependency downloads

#### Required Tools

```bash
# 1. Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Install required Rust components
rustup component add clippy rustfmt
rustup target add x86_64-pc-windows-gnu  # For Windows cross-compilation
rustup target add x86_64-apple-darwin    # For macOS cross-compilation

# 3. Install system dependencies (Ubuntu/Debian)
sudo apt update && sudo apt install -y \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libssl-dev \
    libsqlite3-dev \
    libavformat-dev \
    libavcodec-dev \
    libavutil-dev \
    libswscale-dev \
    libpoppler-glib-dev

# 4. Install development tools
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-outdated
cargo install cargo-edit
```

#### macOS Setup
```bash
# Install Xcode command line tools
xcode-select --install

# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install pkg-config ffmpeg poppler
```

#### Windows Setup
```powershell
# Install Visual Studio Build Tools
# Download and install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Install vcpkg for C++ dependencies
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat
.\vcpkg integrate install

# Install required packages
.\vcpkg install sqlite3:x64-windows
.\vcpkg install openssl:x64-windows
```

### Project Setup

```bash
# 1. Clone the repository (when available)
git clone https://github.com/organization/media-organizer.git
cd media-organizer

# 2. Set up development environment
cp .env.example .env  # Copy and modify configuration

# 3. Install dependencies
cargo build

# 4. Run initial setup
cargo run -- --setup  # Initialize database and cache directories

# 5. Verify installation
cargo test
```

## 📋 Coding Standards

### Rust Code Style

#### Formatting
```bash
# Auto-format code (run before every commit)
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check
```

#### Linting
```bash
# Run Clippy with strict settings
cargo clippy -- -D warnings

# Run Clippy for all targets
cargo clippy --all-targets --all-features -- -D warnings
```

#### Code Organization
```rust
// File header template
//! Module documentation
//! 
//! Brief description of the module's purpose and main functionality.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::MediaOrganizerError;
use crate::models::FileEntry;

// Constants at module level
const DEFAULT_CACHE_SIZE: usize = 1000;
const MAX_THUMBNAIL_SIZE: u32 = 512;

// Type aliases for clarity
type FileCache = HashMap<PathBuf, FileEntry>;
type OperationResult<T> = Result<T, MediaOrganizerError>;
```

#### Naming Conventions
- **Modules**: `snake_case` (e.g., `file_system`, `preview_service`)
- **Structs/Enums**: `PascalCase` (e.g., `FileEntry`, `PreviewContent`)
- **Functions/Variables**: `snake_case` (e.g., `list_directory`, `current_path`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_FILE_SIZE`)
- **Traits**: `PascalCase` with descriptive names (e.g., `FileSystemService`)

#### Error Handling
```rust
// Use thiserror for error definitions
#[derive(Debug, thiserror::Error)]
pub enum FileSystemError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Permission denied for path: {path}")]
    PermissionDenied { path: PathBuf },
    
    #[error("Path not found: {path}")]
    PathNotFound { path: PathBuf },
}

// Prefer Result types over panics
pub async fn safe_operation(&self, path: &Path) -> Result<FileEntry, FileSystemError> {
    // Implementation that returns Result
}

// Use anyhow for application-level errors
use anyhow::{Context, Result};

pub async fn complex_operation() -> Result<()> {
    some_operation()
        .await
        .context("Failed to perform complex operation")?;
    Ok(())
}
```

#### Documentation
```rust
/// Service for managing file system operations
/// 
/// Provides cross-platform file operations with proper error handling
/// and performance optimization for large directories.
/// 
/// # Examples
/// 
/// ```rust
/// let service = NativeFileSystemService::new(cache)?;
/// let entries = service.list_directory(&path).await?;
/// ```
pub struct NativeFileSystemService {
    cache: Arc<CacheService>,
}

impl NativeFileSystemService {
    /// Creates a new file system service with the given cache
    /// 
    /// # Arguments
    /// 
    /// * `cache` - Shared cache service for metadata and thumbnails
    /// 
    /// # Errors
    /// 
    /// Returns error if cache initialization fails
    pub fn new(cache: Arc<CacheService>) -> Result<Self, FileSystemError> {
        // Implementation
    }
}
```

### UI Component Standards

#### Dioxus Component Structure
```rust
/// File tree navigator component
/// 
/// Displays hierarchical folder structure with keyboard navigation
/// and drag-and-drop support.
#[derive(Props)]
pub struct FileTreeProps {
    /// Current directory being displayed
    current_path: PathBuf,
    /// Currently selected path (if any)
    selected_path: Option<PathBuf>,
    /// Width of the file tree panel
    width: f32,
    /// Whether the panel is visible
    is_visible: bool,
    /// Callback when user navigates to a new path
    on_navigate: EventHandler<PathBuf>,
    /// Callback when user selects a file/folder
    on_select: EventHandler<PathBuf>,
}

pub fn FileTree(cx: Scope<FileTreeProps>) -> Element {
    // Local state
    let expanded_folders = use_state(cx, HashSet::<PathBuf>::new);
    
    // Effects for data loading
    use_effect(cx, &cx.props.current_path, |current_path| {
        // Load directory contents
    });
    
    // Event handlers
    let handle_keydown = move |event: KeyboardEvent| {
        // Handle keyboard navigation
    };
    
    // Render
    render! {
        div {
            class: "file-tree-container",
            // Component JSX
        }
    }
}
```

#### CSS Class Naming
- Use BEM methodology: `.block__element--modifier`
- Component-specific prefixes: `.file-tree__item`, `.content-viewer__grid`
- State classes: `.is-selected`, `.is-loading`, `.is-error`

## 🧪 Testing Strategy

### Test Structure
```
tests/
├── unit/              # Unit tests
│   ├── services/      # Service layer tests
│   ├── models/        # Data model tests
│   └── utils/         # Utility function tests
├── integration/       # Integration tests
│   ├── file_operations.rs
│   ├── preview_generation.rs
│   └── cache_management.rs
├── ui/               # UI component tests
│   ├── file_tree_test.rs
│   └── content_viewer_test.rs
└── e2e/              # End-to-end tests
    ├── basic_navigation.rs
    └── file_operations.rs
```

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio_test;
    
    #[tokio::test]
    async fn test_list_directory_success() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let service = create_test_service().await;
        create_test_files(&temp_dir).await;
        
        // Act
        let result = service.list_directory(temp_dir.path()).await;
        
        // Assert
        assert!(result.is_ok());
        let entries = result.unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|e| e.name == "test.txt"));
    }
    
    #[tokio::test]
    async fn test_list_directory_permission_denied() {
        // Test error cases
        let service = create_test_service().await;
        let restricted_path = PathBuf::from("/root/restricted");
        
        let result = service.list_directory(&restricted_path).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileSystemError::PermissionDenied { .. }));
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_full_file_operation_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let services = create_test_services().await;
    
    // Create test file
    let source_path = temp_dir.path().join("source.txt");
    tokio::fs::write(&source_path, "test content").await.unwrap();
    
    // Test copy operation
    let dest_path = temp_dir.path().join("destination.txt");
    let operation = FileOperation::copy(vec![source_path.clone()], dest_path.clone());
    
    let result = services.operations.execute_operation(operation).await;
    assert!(result.is_ok());
    
    // Verify file was copied
    assert!(dest_path.exists());
    let content = tokio::fs::read_to_string(&dest_path).await.unwrap();
    assert_eq!(content, "test content");
    
    // Verify original still exists
    assert!(source_path.exists());
}
```

### Test Commands
```bash
# Run all tests
cargo test

# Run specific test module
cargo test file_system

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel
cargo test -- --test-threads=4

# Run tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# Benchmark tests
cargo bench
```

## 🔄 Development Workflow

### Branch Strategy
```
main                    # Production-ready code
├── develop            # Integration branch
├── feature/           # Feature branches
│   ├── file-tree-ui
│   ├── video-preview
│   └── search-filter
├── bugfix/           # Bug fix branches
│   └── cache-memory-leak
└── release/          # Release preparation
    └── v0.1.0
```

### Daily Development Process

#### 1. Start Development Session
```bash
# Update local repository
git checkout develop
git pull origin develop

# Create feature branch
git checkout -b feature/new-feature-name

# Start development server
cargo watch -x run
```

#### 2. Development Loop
```bash
# Make code changes
# ...

# Run tests continuously
cargo watch -x test

# Check code quality
cargo clippy
cargo fmt

# Commit changes
git add .
git commit -m "feat: implement new feature functionality"
```

#### 3. Pre-commit Checks
```bash
#!/bin/bash
# .git/hooks/pre-commit

set -e

echo "Running pre-commit checks..."

# Format code
cargo fmt -- --check || {
    echo "Code not formatted. Run 'cargo fmt' to fix."
    exit 1
}

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "Clippy found issues. Fix them before committing."
    exit 1
}

# Run tests
cargo test || {
    echo "Tests failed. Fix them before committing."
    exit 1
}

echo "All pre-commit checks passed!"
```

### Code Review Process

#### Pull Request Template
```markdown
## Description
Brief description of changes made.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed

## Performance Impact
- [ ] No performance impact
- [ ] Performance improved
- [ ] Performance impact analyzed and documented

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Tests pass locally
```

#### Review Checklist
- [ ] Code follows Rust best practices
- [ ] Error handling is comprehensive
- [ ] Performance implications considered
- [ ] Tests provide adequate coverage
- [ ] Documentation is clear and accurate
- [ ] No security vulnerabilities introduced
- [ ] UI components are accessible
- [ ] Cross-platform compatibility maintained

## 🚀 Build & Release Process

### Development Builds
```bash
# Debug build (default)
cargo build

# Release build with optimizations
cargo build --release

# Build for specific target
cargo build --target x86_64-pc-windows-gnu

# Build with specific features
cargo build --features "video,audio,pdf"
```

### Release Preparation
```bash
# 1. Update version in Cargo.toml
cargo edit set-version 0.1.0

# 2. Update CHANGELOG.md
# Document all changes since last release

# 3. Run full test suite
cargo test --all-features
cargo test --release

# 4. Build for all platforms
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target x86_64-apple-darwin

# 5. Create release tag
git tag -a v0.1.0 -m "Release version 0.1.0"
git push origin v0.1.0
```

### Continuous Integration (GitHub Actions)
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
        
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: ~/.cargo
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        
    - name: Install system dependencies (Ubuntu)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt-get update
        sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev
        
    - name: Check formatting
      run: cargo fmt -- --check
      
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      
    - name: Run tests
      run: cargo test --all-features
      
    - name: Build release
      run: cargo build --release --all-features
```

## 📊 Performance Monitoring

### Benchmarking
```rust
// benches/file_system_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use media_organizer::services::FileSystemService;

fn benchmark_directory_listing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let service = rt.block_on(async { create_test_service().await });
    let test_dir = create_large_test_directory(); // 10,000 files
    
    c.bench_function("list_large_directory", |b| {
        b.iter(|| {
            rt.block_on(async {
                service.list_directory(black_box(&test_dir)).await.unwrap()
            })
        })
    });
}

criterion_group!(benches, benchmark_directory_listing);
criterion_main!(benches);
```

### Memory Profiling
```bash
# Install memory profiler
cargo install cargo-profdata

# Profile memory usage
cargo profdata -- --memory target/release/media-organizer

# Generate memory report
cargo profdata report --memory-usage > memory_report.txt
```

This development workflow ensures consistent code quality, comprehensive testing, and smooth collaboration across the MediaOrganizer project.