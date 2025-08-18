# MediaOrganizer System Architecture Design
*Based on PRD.md - VS Code UI Redesign & Preview Enhancement*

## 1. System Overview

### 1.1 Architecture Principles
- **Layered Architecture**: Clear separation of presentation, business logic, and data layers
- **Component-Based Design**: Modular, reusable UI components following VS Code patterns
- **Service-Oriented Architecture**: Dedicated services for preview, layout, theme management
- **Event-Driven Communication**: Reactive state management with signal-based updates
- **Plugin Architecture**: Extensible preview providers for different file types

### 1.2 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                       │
├─────────────────────────────────────────────────────────────┤
│  Activity Bar  │  Sidebar     │  Editor Groups  │  Panel   │
│  - Explorer    │  - File Tree │  - Tabs        │  - Output │
│  - Search      │  - Preview   │  - Preview     │  - Problems│
│  - Extensions  │  - Metadata  │  - Editor      │  - Terminal│
├─────────────────────────────────────────────────────────────┤
│                    APPLICATION LAYER                        │
├─────────────────────────────────────────────────────────────┤
│  Layout Manager │ Preview Engine │ Theme Manager │ Command  │
│  - Panels       │ - Providers    │ - CSS Props   │ Palette  │
│  - Sizing       │ - Cache        │ - Themes      │ - Actions │
│  - State        │ - Metadata     │ - Detection   │ - Shortcuts│
├─────────────────────────────────────────────────────────────┤
│                     SERVICE LAYER                           │
├─────────────────────────────────────────────────────────────┤
│  File System   │ Cache Service  │ Settings      │ Background│
│  - Operations   │ - Thumbnails   │ - Persistence │ - Tasks   │
│  - Watching     │ - Previews     │ - Validation  │ - Queue   │
│  - Metadata     │ - Cleanup      │ - Defaults    │ - Progress│
├─────────────────────────────────────────────────────────────┤
│                      DATA LAYER                             │
├─────────────────────────────────────────────────────────────┤
│  App State     │ Layout State   │ Preview Cache │ Settings  │
│  - Navigation  │ - Dimensions   │ - Content     │ - User    │
│  - Selection   │ - Visibility   │ - Metadata    │ - Theme   │
│  - Operations  │ - Positions    │ - Thumbnails  │ - Layout  │
└─────────────────────────────────────────────────────────────┘
```

## 2. VS Code UI Architecture

### 2.1 Layout System Design

#### Main Layout Container
```rust
#[derive(Debug, Clone)]
pub struct VSCodeLayout {
    /// Overall layout state and dimensions
    pub layout_state: Signal<LayoutState>,
    /// Window management and sizing
    pub window_manager: Arc<WindowManager>,
    /// Theme system for consistent styling
    pub theme_manager: Arc<ThemeManager>,
    /// Keyboard shortcut handling
    pub shortcut_manager: Arc<ShortcutManager>,
}

impl VSCodeLayout {
    pub fn new() -> Self {
        Self {
            layout_state: use_signal(LayoutState::default),
            window_manager: Arc::new(WindowManager::new()),
            theme_manager: Arc::new(ThemeManager::new()),
            shortcut_manager: Arc::new(ShortcutManager::new()),
        }
    }
    
    pub fn render(&self) -> Element {
        rsx! {
            div {
                class: "vscode-layout",
                style: self.get_layout_styles(),
                
                // Activity Bar (leftmost)
                ActivityBar {
                    state: self.layout_state.read().activity_bar,
                    on_action: self.handle_activity_action
                }
                
                // Primary Sidebar
                Sidebar {
                    state: self.layout_state.read().sidebar,
                    visible: self.layout_state.read().sidebar_visible,
                    width: self.layout_state.read().sidebar_width,
                    on_resize: self.handle_sidebar_resize
                }
                
                // Main Editor Area
                EditorGroups {
                    groups: self.layout_state.read().editor_groups,
                    on_tab_change: self.handle_tab_change,
                    on_tab_close: self.handle_tab_close
                }
                
                // Bottom Panel
                Panel {
                    state: self.layout_state.read().panel,
                    visible: self.layout_state.read().panel_visible,
                    height: self.layout_state.read().panel_height,
                    on_resize: self.handle_panel_resize
                }
                
                // Status Bar (bottom)
                StatusBar {
                    state: self.layout_state.read().status_bar
                }
                
                // Command Palette (overlay)
                CommandPalette {
                    visible: self.layout_state.read().command_palette_visible,
                    on_command: self.handle_command,
                    on_close: self.hide_command_palette
                }
            }
        }
    }
}
```

### 2.2 Component Hierarchy

#### Activity Bar Component
```rust
#[derive(Props, PartialEq)]
pub struct ActivityBarProps {
    pub state: ActivityBarState,
    pub on_action: EventHandler<ActivityAction>,
}

pub fn ActivityBar(props: ActivityBarProps) -> Element {
    rsx! {
        div {
            class: "activity-bar",
            role: "navigation",
            "aria-label": "Primary navigation",
            
            // Activity items
            for item in props.state.items {
                ActivityBarItem {
                    item: item,
                    active: props.state.active_item == item.id,
                    on_click: move |_| props.on_action.call(ActivityAction::Select(item.id))
                }
            }
            
            // Settings at bottom
            div {
                class: "activity-bar-footer",
                ActivityBarItem {
                    item: ActivityItem::settings(),
                    on_click: move |_| props.on_action.call(ActivityAction::Settings)
                }
            }
        }
    }
}
```

#### Editor Groups Component
```rust
#[derive(Props, PartialEq)]
pub struct EditorGroupsProps {
    pub groups: Vec<EditorGroup>,
    pub on_tab_change: EventHandler<(GroupId, TabId)>,
    pub on_tab_close: EventHandler<(GroupId, TabId)>,
}

pub fn EditorGroups(props: EditorGroupsProps) -> Element {
    rsx! {
        div {
            class: "editor-groups",
            
            for group in props.groups {
                EditorGroup {
                    group: group,
                    on_tab_change: move |tab_id| props.on_tab_change.call((group.id, tab_id)),
                    on_tab_close: move |tab_id| props.on_tab_close.call((group.id, tab_id))
                }
            }
        }
    }
}

pub fn EditorGroup(props: EditorGroupProps) -> Element {
    rsx! {
        div {
            class: "editor-group",
            
            // Tab bar
            div {
                class: "tab-bar",
                role: "tablist",
                
                for tab in props.group.tabs {
                    Tab {
                        tab: tab,
                        active: props.group.active_tab == tab.id,
                        on_select: move |_| props.on_tab_change.call(tab.id),
                        on_close: move |_| props.on_tab_close.call(tab.id)
                    }
                }
            }
            
            // Content area
            div {
                class: "editor-content",
                role: "tabpanel",
                
                if let Some(active_tab) = props.group.get_active_tab() {
                    PreviewContainer {
                        file_path: active_tab.file_path,
                        preview_type: active_tab.preview_type
                    }
                }
            }
        }
    }
}
```

## 3. Preview Service Architecture

### 3.1 Preview Provider System

```rust
/// Core preview service trait
#[async_trait]
pub trait PreviewProvider: Send + Sync {
    /// Check if this provider can handle the given file type
    fn can_preview(&self, file_type: &FileType) -> bool;
    
    /// Generate preview content for the file
    async fn generate_preview(&self, file_path: &Path) -> Result<PreviewContent, PreviewError>;
    
    /// Get a thumbnail for the file (optional, for performance)
    fn get_thumbnail(&self, file_path: &Path) -> Option<ThumbnailData>;
    
    /// Get supported operations for this file type
    fn get_operations(&self) -> Vec<PreviewOperation>;
    
    /// Provider metadata
    fn metadata(&self) -> PreviewProviderMetadata;
}

/// Registry for managing preview providers
pub struct PreviewRegistry {
    providers: Vec<Arc<dyn PreviewProvider>>,
    cache: Arc<PreviewCache>,
}

impl PreviewRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
            cache: Arc::new(PreviewCache::new()),
        };
        
        // Register built-in providers
        registry.register(Arc::new(ImagePreviewProvider::new()));
        registry.register(Arc::new(TextPreviewProvider::new()));
        registry.register(Arc::new(VideoPreviewProvider::new()));
        registry.register(Arc::new(AudioPreviewProvider::new()));
        registry.register(Arc::new(PdfPreviewProvider::new()));
        registry.register(Arc::new(ArchivePreviewProvider::new()));
        
        registry
    }
    
    pub fn register(&mut self, provider: Arc<dyn PreviewProvider>) {
        self.providers.push(provider);
    }
    
    pub async fn generate_preview(&self, file_path: &Path, file_type: &FileType) -> Result<PreviewContent, PreviewError> {
        // Check cache first
        if let Some(cached) = self.cache.get(file_path).await {
            return Ok(cached);
        }
        
        // Find appropriate provider
        let provider = self.providers
            .iter()
            .find(|p| p.can_preview(file_type))
            .ok_or(PreviewError::UnsupportedFileType)?;
        
        // Generate preview
        let content = provider.generate_preview(file_path).await?;
        
        // Cache result
        self.cache.set(file_path, content.clone()).await;
        
        Ok(content)
    }
}
```

### 3.2 Specific Preview Providers

#### Image Preview Provider
```rust
pub struct ImagePreviewProvider {
    supported_formats: HashSet<ImageFormat>,
}

#[async_trait]
impl PreviewProvider for ImagePreviewProvider {
    fn can_preview(&self, file_type: &FileType) -> bool {
        matches!(file_type, FileType::Image(format) if self.supported_formats.contains(format))
    }
    
    async fn generate_preview(&self, file_path: &Path) -> Result<PreviewContent, PreviewError> {
        let image_data = tokio::fs::read(file_path).await?;
        let image = image::load_from_memory(&image_data)?;
        
        let dimensions = (image.width(), image.height());
        let format = ImageFormat::from_path(file_path)?;
        
        Ok(PreviewContent::Image {
            data: image_data,
            format,
            dimensions,
            metadata: self.extract_metadata(file_path).await?,
        })
    }
    
    fn get_thumbnail(&self, file_path: &Path) -> Option<ThumbnailData> {
        // Generate or retrieve cached thumbnail
        self.thumbnail_cache.get(file_path)
    }
    
    fn get_operations(&self) -> Vec<PreviewOperation> {
        vec![
            PreviewOperation::ZoomIn,
            PreviewOperation::ZoomOut,
            PreviewOperation::FitToWindow,
            PreviewOperation::ActualSize,
            PreviewOperation::RotateLeft,
            PreviewOperation::RotateRight,
        ]
    }
}
```

#### Text/Code Preview Provider
```rust
pub struct TextPreviewProvider {
    syntax_highlighter: SyntaxHighlighter,
}

#[async_trait]
impl PreviewProvider for TextPreviewProvider {
    fn can_preview(&self, file_type: &FileType) -> bool {
        matches!(file_type, 
            FileType::Text(_) | 
            FileType::Code(_) |
            FileType::Document(DocumentFormat::Json | DocumentFormat::Xml | DocumentFormat::Csv)
        )
    }
    
    async fn generate_preview(&self, file_path: &Path) -> Result<PreviewContent, PreviewError> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let syntax = self.detect_syntax(file_path, &content);
        
        Ok(PreviewContent::Text {
            content,
            syntax: Some(syntax),
            line_count: content.lines().count(),
            encoding: self.detect_encoding(file_path).await?,
        })
    }
    
    fn get_operations(&self) -> Vec<PreviewOperation> {
        vec![
            PreviewOperation::Search,
            PreviewOperation::GoToLine,
            PreviewOperation::ToggleWordWrap,
            PreviewOperation::CopyContent,
        ]
    }
}
```

## 4. State Management Design

### 4.1 Centralized State Architecture

```rust
/// Main application state container
#[derive(Debug, Clone)]
pub struct AppState {
    /// Layout and UI state
    pub layout: Signal<LayoutState>,
    /// File system and navigation state
    pub navigation: Signal<NavigationState>,
    /// Preview and content state
    pub preview: Signal<PreviewState>,
    /// Theme and appearance state
    pub theme: Signal<ThemeState>,
    /// Command and shortcut state
    pub commands: Signal<CommandState>,
}

/// Layout state for VS Code-like interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    // Activity Bar
    pub activity_bar_visible: bool,
    pub active_activity: ActivityType,
    
    // Sidebar
    pub sidebar_visible: bool,
    pub sidebar_width: f64,
    pub sidebar_content: SidebarContent,
    
    // Editor Groups
    pub editor_groups: Vec<EditorGroup>,
    pub active_group: GroupId,
    
    // Panel
    pub panel_visible: bool,
    pub panel_height: f64,
    pub active_panel: PanelType,
    
    // Status Bar
    pub status_bar_visible: bool,
    
    // Command Palette
    pub command_palette_visible: bool,
    
    // Window state
    pub window_maximized: bool,
    pub window_dimensions: (f64, f64),
}

/// Preview state management
#[derive(Debug, Clone)]
pub struct PreviewState {
    /// Currently active previews by tab
    pub active_previews: HashMap<TabId, PreviewSession>,
    /// Preview cache status
    pub cache_status: CacheStatus,
    /// Loading states
    pub loading_previews: HashSet<TabId>,
    /// Error states
    pub preview_errors: HashMap<TabId, PreviewError>,
}

#[derive(Debug, Clone)]
pub struct PreviewSession {
    pub file_path: PathBuf,
    pub content: PreviewContent,
    pub view_state: ViewState,
    pub operations_history: Vec<PreviewOperation>,
    pub metadata: FileMetadata,
}

#[derive(Debug, Clone)]
pub struct ViewState {
    pub zoom_level: f64,
    pub scroll_position: (f64, f64),
    pub selection: Option<TextSelection>,
    pub current_page: usize, // For documents
    pub playback_position: f64, // For media
}
```

### 4.2 State Management Patterns

#### Signal-Based Reactivity
```rust
/// State manager for coordinating updates
pub struct StateManager {
    app_state: AppState,
    event_bus: EventBus,
    persistence: PersistenceService,
}

impl StateManager {
    pub fn new() -> Self {
        let app_state = AppState {
            layout: use_signal(LayoutState::default),
            navigation: use_signal(NavigationState::default),
            preview: use_signal(PreviewState::default),
            theme: use_signal(ThemeState::default),
            commands: use_signal(CommandState::default),
        };
        
        Self {
            app_state,
            event_bus: EventBus::new(),
            persistence: PersistenceService::new(),
        }
    }
    
    /// Handle layout changes
    pub fn update_layout(&mut self, update: LayoutUpdate) {
        let mut layout = self.app_state.layout.write();
        
        match update {
            LayoutUpdate::ToggleSidebar => {
                layout.sidebar_visible = !layout.sidebar_visible;
            }
            LayoutUpdate::ResizeSidebar(width) => {
                layout.sidebar_width = width.clamp(200.0, 400.0);
            }
            LayoutUpdate::SwitchActivity(activity) => {
                layout.active_activity = activity;
            }
            LayoutUpdate::OpenTab(file_path) => {
                let group = layout.get_active_group_mut();
                group.open_tab(file_path);
            }
            LayoutUpdate::CloseTab(group_id, tab_id) => {
                if let Some(group) = layout.get_group_mut(group_id) {
                    group.close_tab(tab_id);
                }
            }
        }
        
        // Persist changes
        self.persistence.save_layout_state(&layout);
        
        // Emit events
        self.event_bus.emit(AppEvent::LayoutChanged(update));
    }
    
    /// Handle preview updates
    pub async fn update_preview(&mut self, tab_id: TabId, file_path: PathBuf) {
        // Set loading state
        self.app_state.preview.write().loading_previews.insert(tab_id);
        
        // Generate preview asynchronously
        let registry = PreviewRegistry::new();
        let file_type = FileType::from_path(&file_path);
        
        match registry.generate_preview(&file_path, &file_type).await {
            Ok(content) => {
                let mut preview_state = self.app_state.preview.write();
                preview_state.loading_previews.remove(&tab_id);
                preview_state.active_previews.insert(tab_id, PreviewSession {
                    file_path,
                    content,
                    view_state: ViewState::default(),
                    operations_history: Vec::new(),
                    metadata: FileMetadata::default(),
                });
            }
            Err(error) => {
                let mut preview_state = self.app_state.preview.write();
                preview_state.loading_previews.remove(&tab_id);
                preview_state.preview_errors.insert(tab_id, error);
            }
        }
    }
}
```

## 5. Service Layer Design

### 5.1 Layout Manager Service

```rust
/// Service for managing VS Code-like layout
pub struct LayoutManager {
    state: Signal<LayoutState>,
    constraints: LayoutConstraints,
    animations: AnimationManager,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            state: use_signal(LayoutState::default),
            constraints: LayoutConstraints::vscode_defaults(),
            animations: AnimationManager::new(),
        }
    }
    
    /// Calculate layout dimensions based on constraints
    pub fn calculate_layout(&self, window_size: (f64, f64)) -> LayoutDimensions {
        let (window_width, window_height) = window_size;
        let state = self.state.read();
        
        let activity_bar_width = if state.activity_bar_visible { 48.0 } else { 0.0 };
        let sidebar_width = if state.sidebar_visible { state.sidebar_width } else { 0.0 };
        let panel_height = if state.panel_visible { state.panel_height } else { 0.0 };
        let status_bar_height = if state.status_bar_visible { 22.0 } else { 0.0 };
        
        let editor_width = window_width - activity_bar_width - sidebar_width;
        let editor_height = window_height - panel_height - status_bar_height;
        
        LayoutDimensions {
            activity_bar: Rect::new(0.0, 0.0, activity_bar_width, window_height),
            sidebar: Rect::new(activity_bar_width, 0.0, sidebar_width, window_height - status_bar_height),
            editor: Rect::new(activity_bar_width + sidebar_width, 0.0, editor_width, editor_height),
            panel: Rect::new(activity_bar_width + sidebar_width, editor_height, editor_width, panel_height),
            status_bar: Rect::new(0.0, window_height - status_bar_height, window_width, status_bar_height),
        }
    }
    
    /// Handle panel resizing with constraints
    pub fn resize_panel(&mut self, new_height: f64, window_height: f64) {
        let min_height = self.constraints.panel_min_height;
        let max_height = window_height * self.constraints.panel_max_height_ratio;
        
        let constrained_height = new_height.clamp(min_height, max_height);
        
        self.state.write().panel_height = constrained_height;
        
        // Animate the resize if enabled
        if self.animations.enabled {
            self.animations.animate_panel_resize(constrained_height);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub activity_bar_width: f64,
    pub sidebar_min_width: f64,
    pub sidebar_max_width: f64,
    pub panel_min_height: f64,
    pub panel_max_height_ratio: f64,
    pub tab_min_width: f64,
    pub tab_max_width: f64,
}

impl LayoutConstraints {
    pub fn vscode_defaults() -> Self {
        Self {
            activity_bar_width: 48.0,
            sidebar_min_width: 200.0,
            sidebar_max_width: 400.0,
            panel_min_height: 150.0,
            panel_max_height_ratio: 0.5,
            tab_min_width: 120.0,
            tab_max_width: 240.0,
        }
    }
}
```

### 5.2 Theme Manager Service

```rust
/// Service for managing VS Code-compatible themes
pub struct ThemeManager {
    current_theme: Signal<Theme>,
    available_themes: Vec<ThemeDefinition>,
    css_manager: CSSCustomPropertiesManager,
    system_theme_detector: SystemThemeDetector,
}

impl ThemeManager {
    pub fn new() -> Self {
        Self {
            current_theme: use_signal(|| Theme::Auto),
            available_themes: Self::load_default_themes(),
            css_manager: CSSCustomPropertiesManager::new(),
            system_theme_detector: SystemThemeDetector::new(),
        }
    }
    
    /// Apply theme by updating CSS custom properties
    pub fn apply_theme(&self, theme: &ThemeDefinition) {
        let properties = theme.to_css_properties();
        self.css_manager.update_properties(properties);
        
        // Update document class for theme-specific styles
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = document.body() {
                let _ = body.set_class_name(&format!("theme-{}", theme.name));
            }
        }
    }
    
    /// Detect and apply system theme if using auto mode
    pub async fn update_system_theme(&self) {
        if matches!(*self.current_theme.read(), Theme::Auto) {
            let system_theme = self.system_theme_detector.detect().await;
            let theme_def = self.get_theme_definition(&system_theme);
            self.apply_theme(&theme_def);
        }
    }
    
    fn load_default_themes() -> Vec<ThemeDefinition> {
        vec![
            ThemeDefinition::vscode_dark(),
            ThemeDefinition::vscode_light(),
            ThemeDefinition::vscode_high_contrast(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub name: String,
    pub display_name: String,
    pub base_theme: BaseTheme,
    pub colors: ThemeColors,
    pub token_colors: Vec<TokenColor>,
}

impl ThemeDefinition {
    pub fn vscode_dark() -> Self {
        Self {
            name: "vscode-dark".to_string(),
            display_name: "Dark (Visual Studio Code)".to_string(),
            base_theme: BaseTheme::Dark,
            colors: ThemeColors {
                background: "#1e1e1e".to_string(),
                foreground: "#cccccc".to_string(),
                sidebar_background: "#252526".to_string(),
                activity_bar_background: "#333333".to_string(),
                tab_active_background: "#1e1e1e".to_string(),
                tab_inactive_background: "#2d2d30".to_string(),
                border: "#464647".to_string(),
                accent: "#007acc".to_string(),
                error: "#f14c4c".to_string(),
                warning: "#ffcc02".to_string(),
            },
            token_colors: vec![
                TokenColor { scope: "keyword".to_string(), foreground: "#569cd6".to_string() },
                TokenColor { scope: "string".to_string(), foreground: "#ce9178".to_string() },
                TokenColor { scope: "comment".to_string(), foreground: "#6a9955".to_string() },
            ],
        }
    }
    
    pub fn to_css_properties(&self) -> HashMap<String, String> {
        let mut properties = HashMap::new();
        
        properties.insert("--vscode-background".to_string(), self.colors.background.clone());
        properties.insert("--vscode-foreground".to_string(), self.colors.foreground.clone());
        properties.insert("--vscode-sidebar-background".to_string(), self.colors.sidebar_background.clone());
        properties.insert("--vscode-activity-bar-background".to_string(), self.colors.activity_bar_background.clone());
        properties.insert("--vscode-tab-active-background".to_string(), self.colors.tab_active_background.clone());
        properties.insert("--vscode-tab-inactive-background".to_string(), self.colors.tab_inactive_background.clone());
        properties.insert("--vscode-border".to_string(), self.colors.border.clone());
        properties.insert("--vscode-accent".to_string(), self.colors.accent.clone());
        properties.insert("--vscode-error".to_string(), self.colors.error.clone());
        properties.insert("--vscode-warning".to_string(), self.colors.warning.clone());
        
        properties
    }
}
```

## 6. Performance & Optimization

### 6.1 Preview Caching Strategy

```rust
/// Multi-level cache for preview content
pub struct PreviewCache {
    /// In-memory cache for recent previews
    memory_cache: Arc<RwLock<LruCache<PathBuf, PreviewContent>>>,
    /// Disk cache for thumbnails and metadata
    disk_cache: Arc<DiskCache>,
    /// Cache statistics and metrics
    stats: Arc<RwLock<CacheStats>>,
}

impl PreviewCache {
    pub fn new() -> Self {
        Self {
            memory_cache: Arc::new(RwLock::new(LruCache::new(100))),
            disk_cache: Arc::new(DiskCache::new()),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    pub async fn get(&self, file_path: &Path) -> Option<PreviewContent> {
        // Check memory cache first
        if let Some(content) = self.memory_cache.read().await.get(file_path) {
            self.stats.write().await.memory_hits += 1;
            return Some(content.clone());
        }
        
        // Check disk cache
        if let Some(content) = self.disk_cache.get(file_path).await {
            // Promote to memory cache
            self.memory_cache.write().await.insert(file_path.to_path_buf(), content.clone());
            self.stats.write().await.disk_hits += 1;
            return Some(content);
        }
        
        self.stats.write().await.misses += 1;
        None
    }
    
    pub async fn set(&self, file_path: &Path, content: PreviewContent) {
        // Store in memory cache
        self.memory_cache.write().await.insert(file_path.to_path_buf(), content.clone());
        
        // Store in disk cache if cacheable
        if content.is_cacheable() {
            self.disk_cache.set(file_path, content).await;
        }
        
        self.stats.write().await.stores += 1;
    }
}
```

### 6.2 Progressive Loading

```rust
/// Progressive loading for large files
pub struct ProgressiveLoader {
    chunk_size: usize,
    max_initial_load: usize,
}

impl ProgressiveLoader {
    pub async fn load_progressively<T>(&self, file_path: &Path, processor: impl Fn(&[u8]) -> T) -> Result<Stream<T>, LoadError> {
        let file_size = tokio::fs::metadata(file_path).await?.len() as usize;
        
        if file_size <= self.max_initial_load {
            // Load entire file at once for small files
            let data = tokio::fs::read(file_path).await?;
            let result = processor(&data);
            Ok(stream::once(async { result }))
        } else {
            // Progressive loading for large files
            let mut file = tokio::fs::File::open(file_path).await?;
            let stream = async_stream::stream! {
                let mut buffer = vec![0; self.chunk_size];
                loop {
                    match file.read(&mut buffer).await {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let chunk_result = processor(&buffer[..n]);
                            yield chunk_result;
                        }
                        Err(e) => {
                            // Handle error
                            break;
                        }
                    }
                }
            };
            Ok(Box::pin(stream))
        }
    }
}
```

This comprehensive system architecture design provides a solid foundation for implementing the VS Code-like UI redesign and preview features outlined in the PRD. The design emphasizes modularity, performance, and maintainability while staying true to VS Code's interaction patterns and visual design.