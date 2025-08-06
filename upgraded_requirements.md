# MediaOrganizer - Upgraded Requirements Document

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
- **Folder View**: macOS Finder-style grid layout with virtualization
  - Adjustable icon sizes (small, medium, large, extra large)
  - Virtual scrolling for 10,000+ files (windowing technique)
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
- **Text Files**: Syntax-highlighted text editor view
- **Microsoft Office** (제한된 지원):
  - **Phase 1**: 기본 메타데이터 표시만 지원
  - **Phase 2**: WebView 기반 미리보기 (온라인 서비스 활용)
  - **Future**: 네이티브 렌더링 (복잡도 고려하여 장기 계획)

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
  - **Enhanced Error Handling**: 끊어진 심볼릭 링크나 접근 권한 부족 시 사용자 친화적 오류 메시지 표시

#### Search & Filter (우선순위 상향 조정)
- **Quick Search**: Real-time filename filtering
- **Advanced Search**: Content-based search for supported file types
- **Filter Options**: By file type, size range, date range
- **Saved Searches**: Bookmark frequently used search criteria

#### Folder Analysis
- **Size Calculation**: Recursive folder size calculation with caching
- **File Type Distribution**: Breakdown of file types within folders
- **Duplicate Detection**: Find duplicate files based on content hash
  - **Enhanced Implementation**:
    - 백그라운드 스레드에서 비동기 처리
    - 진행 상태 표시 (프로그레스바)
    - 특정 폴더 대상 선택 옵션
    - 사용자 중단 가능한 인터페이스
- **Large File Detection**: Identify files above specified size thresholds

### 5. Performance Requirements

#### File System Operations
- **Large Directory Handling**: 
  - **Target**: 10,000+ files 폴더 로딩 시간 1초 미만
  - **Implementation**: Virtual scrolling과 lazy loading 적용
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

## Development Phases (수정된 우선순위)

### Phase 1: Core Infrastructure
- Basic Dioxus application setup
- File system navigation
- Basic file operations (copy, move, delete)
- Simple file listing with virtual scrolling foundation

### Phase 2: UI Framework & Essential Features
- VS Code-style layout implementation
- Resizable panels
- File tree component
- Basic grid view with virtualization
- **Search & Filter implementation** (우선순위 상향)

### Phase 3: File Preview System
- Image preview implementation
- Basic video player integration
- Document viewer foundation (PDF, Markdown, Text)
- Metadata extraction
- **Note**: Microsoft Office 지원은 기본 메타데이터만

### Phase 4: Advanced Features
- Destination management
- Keyboard shortcuts
- Enhanced duplicate detection (백그라운드 처리)
- Batch operations

### Phase 5: Performance & Polish
- Thumbnail caching optimization
- Performance optimization for large directories
- Comprehensive error handling
- User testing and refinement

## Technical Considerations

### Dependencies (개선된 권장사항)
- **File System**: Use Rust's `std::fs` and `walkdir` for file operations
- **Media Processing**: 
  - Images: `image` crate for loading and basic manipulation
  - Video: `ffmpeg-next` (정적 빌드 포함) 또는 동적 로딩 방식 선택
  - Audio: `rodio` or `symphonia` for audio playback
- **UI Components**: Custom Dioxus components with virtual scrolling
- **Async Operations**: `tokio` for non-blocking file operations

### FFmpeg Integration Strategy
- **Option 1**: 정적 빌드 - 배포 간소화, 크기 증가
- **Option 2**: 동적 로딩 - 런타임 의존성, 플랫폼별 관리 필요
- **권장**: 초기에는 기본 메타데이터만 지원, 추후 사용자 피드백에 따라 확장

### Performance Targets (구체화된 지표)
- **Startup Time**: < 3 seconds for application launch
- **Directory Loading**: < 1 second for folders with up to 1000 files
- **Large Directory**: < 1 second for 10,000+ files (virtual scrolling)
- **Preview Generation**: < 500ms for common file types
- **Memory Usage**: < 200MB baseline, scalable with content
- **Duplicate Detection**: Progress feedback every 100 files processed

### Error Handling
- Graceful handling of permission errors
- Network drive timeout handling
- Corrupted file detection and reporting
- User-friendly error messages with suggested actions
- **Enhanced**: 심볼릭 링크 오류 처리 및 사용자 안내

## Success Criteria (측정 가능한 지표 추가)

1. **Functionality**: All core file operations work reliably across platforms
2. **Performance**: 
   - 10,000+ 파일 폴더 로딩 1초 미만
   - UI 응답성 유지 (60 FPS 목표)
3. **Usability**: Intuitive interface comparable to professional file managers
4. **Stability**: No crashes during normal usage patterns
5. **Cross-Platform**: Consistent behavior and appearance across Windows, macOS, and Linux
6. **Memory Efficiency**: 대용량 폴더 처리 시 메모리 사용량 최적화

## Future Enhancements (장기 로드맵)

### Phase 6+: Extended Features
- **Microsoft Office 네이티브 지원**: 안정적인 파싱 라이브러리 확보 후
- Cloud storage integration (Google Drive, Dropbox, OneDrive)
- Advanced image editing capabilities
- Plugin system for custom file type handlers
- Network file sharing capabilities
- Advanced duplicate file management with smart suggestions
- File organization suggestions using AI/ML

### Technology Evolution
- WebView 기반 Office 파일 미리보기 구현
- 플러그인 아키텍처를 통한 확장성 확보
- 클라우드 서비스 통합을 위한 OAuth 인증 시스템

## Risk Mitigation

### Technical Risks
- **FFmpeg 의존성**: 정적 빌드 옵션 준비 및 fallback 메커니즘
- **Office 파일 지원**: 단계적 구현으로 리스크 분산
- **성능 문제**: 초기부터 가상화 및 캐싱 전략 적용

### Development Risks
- **복잡도 관리**: 핵심 기능 우선 개발, 고급 기능은 점진적 추가
- **플랫폼 호환성**: 지속적인 크로스플랫폼 테스트 환경 구축

## Conclusion

이 업그레이드된 요구사항 문서는 원본의 핵심 비전을 유지하면서도 실제 구현 시 발생할 수 있는 기술적 문제점들을 사전에 고려하여 보다 현실적이고 단계적인 개발 계획을 제시합니다. 특히 성능 최적화와 사용자 경험 개선에 중점을 두어 안정적이고 효율적인 미디어 관리 애플리케이션 개발을 목표로 합니다.