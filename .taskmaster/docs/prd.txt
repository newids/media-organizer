# Product Requirements Document (PRD): MediaOrganizer UI Redesign & Preview Enhancement

## 1. Executive Summary

### Project Overview
This PRD outlines the redesign of MediaOrganizer's user interface to match Visual Studio Code's design language, implementation of a comprehensive preview feature, and cleanup of unnecessary UI components to improve user experience and maintain consistency with modern development tools.

### Objectives
- **Primary**: Redesign UI to match Visual Studio Code's interface patterns
- **Secondary**: Implement comprehensive file preview functionality
- **Tertiary**: Remove redundant label components and streamline UI

### Success Metrics
- UI consistency score of 90%+ with VS Code design patterns
- Preview feature supports 8+ file types with <500ms load times
- 25% reduction in UI component count through label removal
- User satisfaction score improvement of 20%+

## 2. Problem Statement

### Current Challenges
1. **Inconsistent UI Design**: Current interface lacks the polished, professional appearance of modern development tools
2. **Missing Preview Capability**: Users cannot preview file contents without external applications
3. **UI Clutter**: Unnecessary label components reduce usability and visual clarity
4. **Poor Developer Experience**: Interface doesn't align with familiar VS Code patterns used by target audience

### User Pain Points
- Cognitive load from unfamiliar interface patterns
- Workflow interruption when needing to open external applications for file preview
- Visual noise from redundant UI elements
- Lack of keyboard shortcuts and accessibility features common in VS Code

## 3. Solution Overview

### Core Features

#### 3.1 Visual Studio Code UI Redesign
**Description**: Complete interface redesign to match VS Code's design language, layout patterns, and interaction models.

**Key Components**:
- **Activity Bar**: Left-side vertical navigation bar with icons for Explorer, Search, Extensions, etc.
- **Primary Sidebar**: Collapsible file explorer with tree view
- **Editor Groups**: Tabbed interface for viewing multiple files
- **Panel**: Bottom panel for terminal, problems, output, etc.
- **Status Bar**: Bottom status information bar
- **Command Palette**: Searchable command interface (Ctrl+Shift+P)

**Design Specifications**:
- Color scheme: VS Code Dark/Light themes with auto-detection
- Typography: Consolas/Monaco monospace fonts with size hierarchy
- Spacing: 8px grid system matching VS Code
- Icons: Codicons icon set or equivalent
- Layout: CSS Grid with VS Code-like dimensions and proportions

#### 3.2 Comprehensive Preview Feature
**Description**: In-place file preview system supporting multiple file types without external applications.

**Supported File Types**:
1. **Images**: JPG, PNG, GIF, WebP, SVG, BMP, TIFF
2. **Videos**: MP4, AVI, MOV, WMV, MKV (with basic controls)
3. **Audio**: MP3, WAV, FLAC, AAC, OGG (with waveform visualization)
4. **Documents**: PDF, TXT, MD, JSON, XML, CSV
5. **Code**: JS, TS, RS, PY, HTML, CSS (with syntax highlighting)
6. **Archives**: ZIP, RAR, 7Z (content listing)
7. **3D Models**: OBJ, STL, PLY (basic viewer)
8. **Office**: DOCX, XLSX, PPTX (basic preview)

**Preview Panel Features**:
- **Zoom Controls**: Pan, zoom, fit-to-window for images and documents
- **Metadata Display**: File properties, EXIF data, creation/modification dates
- **Quick Actions**: Rotate, copy path, open with external app
- **Keyboard Navigation**: Arrow keys for file navigation, space for preview toggle
- **Performance**: Lazy loading, thumbnail caching, progressive rendering

#### 3.3 UI Component Cleanup
**Description**: Systematic removal of unnecessary label components and UI elements that contribute to visual clutter.

**Removal Targets**:
- Redundant file type labels (replace with icons)
- Excessive descriptive text (replace with tooltips)
- Duplicate navigation elements
- Outdated progress indicators
- Unnecessary separator lines and borders

**Retention Criteria**:
- Essential for accessibility (screen readers)
- Required for user understanding (first-time users)
- Part of core functionality (cannot be replaced with icons/tooltips)

## 4. Technical Requirements

### 4.1 Architecture Changes

#### Frontend Framework Updates
- **Dioxus Component Restructuring**: Reorganize components to match VS Code layout hierarchy
- **CSS System Overhaul**: Implement CSS custom properties for theming
- **State Management**: Centralized layout state for panels, sidebars, and editor groups
- **Event Handling**: Keyboard shortcuts system matching VS Code patterns

#### New Service Layer Components
```rust
// Preview Service Architecture
pub trait PreviewProvider {
    fn can_preview(&self, file_type: &FileType) -> bool;
    async fn generate_preview(&self, file_path: &Path) -> Result<PreviewContent, PreviewError>;
    fn get_thumbnail(&self, file_path: &Path) -> Option<ThumbnailData>;
}

// UI Layout Service
pub struct LayoutManager {
    pub activity_bar: ActivityBarState,
    pub sidebar: SidebarState,
    pub editor_groups: EditorGroupState,
    pub panel: PanelState,
    pub status_bar: StatusBarState,
}
```

### 4.2 Implementation Specifications

#### UI Framework Requirements
- **CSS Grid Layout**: Responsive grid system matching VS Code proportions
- **Theme System**: Dark/light mode with CSS custom properties
- **Icon System**: SVG icon set with 16px, 20px, 24px variants
- **Typography**: Font hierarchy with consistent sizing (12px, 14px, 16px, 18px)
- **Accessibility**: ARIA labels, keyboard navigation, high contrast support

#### Performance Requirements
- **Preview Generation**: <500ms for images, <1s for documents, <2s for videos
- **UI Rendering**: <100ms for layout changes, <50ms for theme switches
- **Memory Usage**: <200MB baseline, <500MB with active previews
- **File Loading**: Support for files up to 100MB with progressive loading

### 4.3 Data Models

#### Preview Content Model
```rust
#[derive(Debug, Clone)]
pub enum PreviewContent {
    Image { data: Vec<u8>, format: ImageFormat, dimensions: (u32, u32) },
    Video { thumbnail: Vec<u8>, duration: Duration, format: VideoFormat },
    Audio { waveform: Vec<f32>, duration: Duration, metadata: AudioMetadata },
    Document { pages: Vec<PageContent>, total_pages: usize },
    Text { content: String, syntax: Option<SyntaxHighlight> },
    Archive { entries: Vec<ArchiveEntry>, total_size: u64 },
    Unsupported { reason: String, fallback_icon: String },
}
```

#### Layout State Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub activity_bar_visible: bool,
    pub sidebar_width: f64,
    pub sidebar_visible: bool,
    pub panel_height: f64,
    pub panel_visible: bool,
    pub editor_groups: Vec<EditorGroup>,
    pub current_theme: Theme,
}
```

## 5. User Experience Design

### 5.1 Navigation Flow

#### Primary Workflow
1. **File Discovery**: Activity Bar → Explorer → Browse directories
2. **File Selection**: Click file in tree view → Preview appears in editor area
3. **Preview Interaction**: Use mouse/keyboard to navigate, zoom, or perform actions
4. **Multi-file Workflow**: Open multiple files in tabs, switch between previews
5. **Search & Filter**: Command palette or search panel for quick file access

#### Keyboard Shortcuts (VS Code Compatible)
- `Ctrl+Shift+E`: Focus Explorer
- `Ctrl+Shift+P`: Command Palette
- `Ctrl+1/2/3`: Focus Editor Group
- `Ctrl+W`: Close Current Tab
- `Ctrl+Tab`: Switch Between Tabs
- `F2`: Rename File
- `Delete`: Move to Trash
- `Space`: Toggle Preview
- `Ctrl+Plus/Minus`: Zoom In/Out

### 5.2 Visual Design Specification

#### Color Palette (Dark Theme)
```css
:root {
  --vscode-background: #1e1e1e;
  --vscode-foreground: #cccccc;
  --vscode-sidebar-background: #252526;
  --vscode-activity-bar-background: #333333;
  --vscode-tab-active-background: #1e1e1e;
  --vscode-tab-inactive-background: #2d2d30;
  --vscode-border: #464647;
  --vscode-accent: #007acc;
  --vscode-error: #f14c4c;
  --vscode-warning: #ffcc02;
}
```

#### Typography Scale
- **Interface**: Segoe UI, Tahoma, sans-serif
- **Code**: Consolas, Monaco, 'Courier New', monospace
- **Sizes**: 11px (small), 13px (body), 14px (headings), 16px (large)

#### Layout Dimensions
- **Activity Bar**: 48px width
- **Sidebar**: 240px default width (200px-400px range)
- **Panel**: 200px default height (150px-50% range)
- **Tab Height**: 35px
- **Status Bar**: 22px height

## 6. Implementation Plan

### 6.1 Development Phases

#### Phase 1: UI Framework Foundation (Weeks 1-3)
**Objectives**: Establish VS Code-like layout structure
- Set up CSS grid system and custom properties
- Implement basic activity bar and sidebar
- Create tab system for editor groups
- Add theme switching infrastructure

**Deliverables**:
- Basic VS Code layout structure
- Theme system with dark/light modes
- Activity bar with navigation icons
- Collapsible sidebar with file tree
- Tab-based editor area

#### Phase 2: Preview System Core (Weeks 4-6)
**Objectives**: Implement fundamental preview capabilities
- Create preview service architecture
- Implement image preview with zoom/pan
- Add text file preview with syntax highlighting
- Build metadata display system

**Deliverables**:
- Preview service with plugin architecture
- Image preview with full controls
- Text file preview with syntax highlighting
- Metadata panel for file properties
- Thumbnail generation system

#### Phase 3: Enhanced Preview Features (Weeks 7-9)
**Objectives**: Extend preview support to multimedia and documents
- Video preview with playback controls
- Audio preview with waveform visualization
- PDF document preview
- Archive content listing

**Deliverables**:
- Video player with timeline controls
- Audio player with waveform display
- PDF viewer with page navigation
- Archive explorer with file listing
- Preview loading states and error handling

#### Phase 4: UI Polish & Component Cleanup (Weeks 10-12)
**Objectives**: Remove unnecessary components and polish interface
- Audit and remove redundant labels
- Implement consistent iconography
- Add keyboard shortcuts and accessibility
- Performance optimization and testing

**Deliverables**:
- Cleaned UI with minimal visual noise
- Complete icon system implementation
- Full keyboard navigation support
- Accessibility compliance (WCAG 2.1 AA)
- Performance benchmarks and optimizations

### 6.2 Technical Dependencies

#### Required Libraries
```toml
[dependencies]
# UI Framework (existing)
dioxus = "0.6.3"
dioxus-desktop = "0.6.3"

# Preview Generation
image = "0.24"
ffmpeg-next = "7.1"
rodio = "0.17"
pdf = "0.9"
syntect = "5.0"  # Syntax highlighting
tree-sitter = "0.20"  # Code parsing

# UI Enhancement
winit = "0.28"  # Window management
wgpu = "0.17"  # GPU acceleration for previews
fontdb = "0.15"  # Font management
```

#### Integration Points
- **File System Service**: Extended to support preview metadata
- **Cache Service**: Thumbnail and preview caching
- **Theme Service**: CSS custom properties management
- **Settings Service**: Layout preferences persistence

### 6.3 Testing Strategy

#### Unit Testing (80% Coverage Target)
- Preview service components
- Layout state management
- Theme switching logic
- File type detection
- Keyboard shortcut handling

#### Integration Testing
- End-to-end preview workflows
- Multi-file tab management
- Responsive layout behavior
- Theme persistence across sessions
- Performance under load (1000+ files)

#### User Acceptance Testing
- VS Code user familiarity assessment
- Preview functionality validation
- Accessibility compliance verification
- Performance benchmarking
- Cross-platform compatibility testing

## 7. Success Criteria & Metrics

### 7.1 Functional Requirements
✅ **Complete VS Code UI Parity**: Layout, colors, typography, interactions
✅ **Multi-format Preview Support**: 8+ file types with appropriate viewers
✅ **Performance Targets**: <500ms preview load, <100ms UI transitions
✅ **Keyboard Navigation**: Full VS Code-compatible shortcut system
✅ **Accessibility Compliance**: WCAG 2.1 AA standards
✅ **Theme System**: Dark/light modes with system preference detection

### 7.2 Quality Gates
- **Code Coverage**: >80% unit test coverage
- **Performance**: Preview generation <500ms for 95% of files <10MB
- **Memory Usage**: <500MB with 50 active previews
- **Accessibility**: 100% keyboard navigable, screen reader compatible
- **Cross-platform**: Identical behavior on Windows, macOS, Linux

### 7.3 User Experience Metrics
- **Learning Curve**: VS Code users achieve 90% feature discovery within 5 minutes
- **Task Completion**: File preview workflow completion in <10 seconds
- **Error Rate**: <5% user errors in navigation and preview operations
- **Satisfaction**: 90%+ positive feedback on interface familiarity

## 8. Risk Assessment & Mitigation

### 8.1 Technical Risks

#### High Risk: Preview Performance
**Risk**: Large files or complex formats cause UI freezing
**Mitigation**: 
- Implement progressive loading and streaming
- Add file size limits with user override options
- Use background threads for preview generation
- Provide cancel functionality for long operations

#### Medium Risk: VS Code UI Complexity
**Risk**: Perfect VS Code replication proves technically challenging
**Mitigation**:
- Prioritize core layout patterns over pixel-perfect matching
- Focus on familiar interaction patterns rather than visual exactness
- Implement incrementally with user feedback loops
- Maintain fallback to current UI during development

### 8.2 User Experience Risks

#### Medium Risk: Feature Discoverability
**Risk**: Users may not find or understand new preview capabilities
**Mitigation**:
- Add contextual tooltips and help text
- Implement first-run tour or tutorial
- Provide keyboard shortcut cheat sheet
- Include preview examples in onboarding

#### Low Risk: Performance Perception
**Risk**: Users expect instant preview for all file types
**Mitigation**:
- Show loading indicators with progress percentages
- Implement smart caching for frequently accessed files
- Provide preview quality settings (speed vs. quality trade-off)
- Clear communication about file size limitations

## 9. Future Considerations

### 9.1 Extension Points
- **Plugin Architecture**: Third-party preview providers
- **Custom Themes**: User-created VS Code theme imports
- **Advanced Features**: Split view, diff view, integrated terminal
- **Cloud Integration**: Preview for cloud-stored files
- **Collaboration**: Real-time preview sharing

### 9.2 Scalability Considerations
- **Performance Optimization**: GPU acceleration for complex previews
- **Memory Management**: Intelligent preview cache eviction
- **File Size Limits**: Streaming for large file handling
- **Network Previews**: Remote file preview capabilities

## 10. Appendices

### 10.1 VS Code UI Reference
- Activity Bar: File explorer, search, source control, run/debug, extensions
- Primary Sidebar: Contextual content based on activity selection
- Editor Groups: Tabbed interface with split view support
- Panel: Integrated terminal, problems, output, debug console
- Status Bar: Current file info, git branch, errors/warnings

### 10.2 Preview Format Priority Matrix
| Format | Priority | Complexity | User Impact |
|--------|----------|------------|-------------|
| Images | High | Low | High |
| Text/Code | High | Medium | High |
| PDF | High | Medium | High |
| Video | Medium | High | Medium |
| Audio | Medium | Medium | Medium |
| Archives | Low | Low | Low |
| 3D Models | Low | High | Low |

This PRD provides comprehensive guidance for transforming MediaOrganizer into a VS Code-like professional file management tool with powerful preview capabilities and a clean, intuitive interface.