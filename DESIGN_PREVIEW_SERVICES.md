# Preview Services Design Document
*Detailed service architecture for file preview capabilities*

## 1. Preview Service Architecture Overview

### 1.1 Core Components Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    PREVIEW ENGINE                           │
├─────────────────────────────────────────────────────────────┤
│  Preview Registry │ Content Manager │ Cache Manager        │
│  - Provider Reg   │ - Content Gen   │ - Memory Cache       │
│  - Type Detection │ - Metadata Ext  │ - Disk Cache         │
│  - Fallbacks      │ - Transformation│ - Cache Policies     │
├─────────────────────────────────────────────────────────────┤
│                 PREVIEW PROVIDERS                           │
├─────────────────────────────────────────────────────────────┤
│ Image Provider │ Text Provider  │ Video Provider │ Audio    │
│ - JPEG/PNG     │ - Syntax HL    │ - MP4/AVI     │ - MP3    │
│ - SVG/WebP     │ - Code Format  │ - Thumbnails  │ - Waveform│
│ - EXIF Data    │ - Search       │ - Controls    │ - Metadata│
├─────────────────────────────────────────────────────────────┤
│ PDF Provider   │ Archive Provider│ 3D Provider  │ Office   │
│ - Page Nav     │ - File List    │ - OBJ/STL     │ - DOCX   │
│ - Text Extract │ - Compression  │ - Viewport    │ - XLSX   │
│ - Annotations  │ - Preview      │ - Materials   │ - Preview│
└─────────────────────────────────────────────────────────────┘
```

## 2. Core Service Interfaces

### 2.1 Preview Provider Trait

```rust
use async_trait::async_trait;
use std::path::Path;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Core trait for all preview providers
#[async_trait]
pub trait PreviewProvider: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this provider
    fn id(&self) -> &'static str;
    
    /// Human-readable name for this provider
    fn name(&self) -> &'static str;
    
    /// Check if this provider can handle the given file type
    fn can_preview(&self, file_type: &FileType, file_path: &Path) -> bool;
    
    /// Get the priority of this provider for the given file type (higher = preferred)
    fn priority(&self, file_type: &FileType) -> u8;
    
    /// Generate preview content for the file
    async fn generate_preview(&self, request: PreviewRequest) -> Result<PreviewContent, PreviewError>;
    
    /// Generate a thumbnail for the file (optional, for performance)
    async fn generate_thumbnail(&self, request: ThumbnailRequest) -> Result<ThumbnailData, PreviewError>;
    
    /// Get metadata for the file
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError>;
    
    /// Get supported operations for this file type
    fn supported_operations(&self) -> Vec<PreviewOperation>;
    
    /// Get configuration options for this provider
    fn configuration_schema(&self) -> Option<ConfigurationSchema>;
    
    /// Validate the provider configuration
    fn validate_configuration(&self, config: &HashMap<String, serde_json::Value>) -> Result<(), String>;
}

/// Request structure for preview generation
#[derive(Debug, Clone)]
pub struct PreviewRequest {
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub file_size: u64,
    pub options: PreviewOptions,
    pub context: PreviewContext,
}

#[derive(Debug, Clone)]
pub struct PreviewOptions {
    /// Maximum dimensions for image/video previews
    pub max_dimensions: Option<(u32, u32)>,
    /// Quality setting (0.0 to 1.0)
    pub quality: f32,
    /// Whether to include metadata
    pub include_metadata: bool,
    /// Custom provider-specific options
    pub provider_options: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct PreviewContext {
    /// Available memory for preview generation
    pub available_memory: Option<usize>,
    /// Maximum processing time allowed
    pub timeout: Option<std::time::Duration>,
    /// Cache availability
    pub cache_available: bool,
    /// User preferences
    pub user_preferences: UserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub prefer_quality_over_speed: bool,
    pub max_file_size_for_preview: u64,
    pub auto_play_media: bool,
    pub show_hidden_metadata: bool,
}
```

### 2.2 Preview Content Model

```rust
/// Union type for all preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewContent {
    Image(ImagePreview),
    Video(VideoPreview),
    Audio(AudioPreview),
    Text(TextPreview),
    Document(DocumentPreview),
    Archive(ArchivePreview),
    Model3D(Model3DPreview),
    Unsupported(UnsupportedPreview),
}

impl PreviewContent {
    /// Get the type of preview content
    pub fn content_type(&self) -> PreviewContentType {
        match self {
            PreviewContent::Image(_) => PreviewContentType::Image,
            PreviewContent::Video(_) => PreviewContentType::Video,
            PreviewContent::Audio(_) => PreviewContentType::Audio,
            PreviewContent::Text(_) => PreviewContentType::Text,
            PreviewContent::Document(_) => PreviewContentType::Document,
            PreviewContent::Archive(_) => PreviewContentType::Archive,
            PreviewContent::Model3D(_) => PreviewContentType::Model3D,
            PreviewContent::Unsupported(_) => PreviewContentType::Unsupported,
        }
    }
    
    /// Check if this content can be cached
    pub fn is_cacheable(&self) -> bool {
        match self {
            PreviewContent::Unsupported(_) => false,
            _ => true,
        }
    }
    
    /// Get estimated memory usage in bytes
    pub fn memory_usage(&self) -> usize {
        match self {
            PreviewContent::Image(img) => img.data.len(),
            PreviewContent::Video(vid) => vid.thumbnail.len(),
            PreviewContent::Audio(aud) => aud.waveform.len() * 4, // f32 = 4 bytes
            PreviewContent::Text(txt) => txt.content.len(),
            PreviewContent::Document(doc) => doc.pages.iter().map(|p| p.content.len()).sum(),
            PreviewContent::Archive(arc) => arc.entries.len() * 100, // estimate
            PreviewContent::Model3D(model) => model.vertices.len() * 12, // 3 f32s per vertex
            PreviewContent::Unsupported(_) => 0,
        }
    }
}

/// Image preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePreview {
    /// Image data in optimized format
    pub data: Vec<u8>,
    /// Image format
    pub format: ImageFormat,
    /// Image dimensions (width, height)
    pub dimensions: (u32, u32),
    /// Color profile information
    pub color_profile: Option<ColorProfile>,
    /// EXIF and other metadata
    pub metadata: ImageMetadata,
    /// Available operations
    pub operations: Vec<ImageOperation>,
}

/// Video preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPreview {
    /// Video thumbnail
    pub thumbnail: Vec<u8>,
    /// Video format information
    pub format: VideoFormat,
    /// Video dimensions
    pub dimensions: (u32, u32),
    /// Video duration in seconds
    pub duration: f64,
    /// Frame rate
    pub frame_rate: f64,
    /// Bitrate information
    pub bitrate: Option<u64>,
    /// Audio tracks information
    pub audio_tracks: Vec<AudioTrackInfo>,
    /// Video metadata
    pub metadata: VideoMetadata,
    /// Streaming URL for playback (if supported)
    pub stream_url: Option<String>,
}

/// Audio preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPreview {
    /// Waveform data for visualization
    pub waveform: Vec<f32>,
    /// Audio format information
    pub format: AudioFormat,
    /// Duration in seconds
    pub duration: f64,
    /// Sample rate
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Bitrate
    pub bitrate: Option<u32>,
    /// Audio metadata (ID3, etc.)
    pub metadata: AudioMetadata,
    /// Streaming URL for playback
    pub stream_url: Option<String>,
}

/// Text preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPreview {
    /// Text content (possibly truncated)
    pub content: String,
    /// Detected or specified syntax highlighting
    pub syntax: Option<SyntaxHighlighting>,
    /// Encoding information
    pub encoding: String,
    /// Line count
    pub line_count: usize,
    /// Character count
    pub char_count: usize,
    /// Language detection result
    pub detected_language: Option<String>,
    /// Whether content is truncated
    pub is_truncated: bool,
    /// File statistics
    pub statistics: TextStatistics,
}

/// Document preview content (PDF, Office, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentPreview {
    /// Document pages
    pub pages: Vec<DocumentPage>,
    /// Total page count
    pub total_pages: usize,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Table of contents if available
    pub table_of_contents: Option<Vec<TocEntry>>,
    /// Text extraction capability
    pub text_extractable: bool,
}

/// Archive preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivePreview {
    /// Archive entries
    pub entries: Vec<ArchiveEntry>,
    /// Total number of files
    pub total_files: usize,
    /// Total uncompressed size
    pub total_size: u64,
    /// Compression ratio
    pub compression_ratio: f32,
    /// Archive format
    pub format: ArchiveFormat,
    /// Archive metadata
    pub metadata: ArchiveMetadata,
}

/// 3D model preview content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model3DPreview {
    /// Simplified mesh vertices
    pub vertices: Vec<[f32; 3]>,
    /// Face indices
    pub faces: Vec<[u32; 3]>,
    /// Material information
    pub materials: Vec<Material3D>,
    /// Bounding box
    pub bounding_box: BoundingBox3D,
    /// Model metadata
    pub metadata: Model3DMetadata,
    /// Thumbnail image
    pub thumbnail: Option<Vec<u8>>,
}

/// Unsupported file preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsupportedPreview {
    /// Reason why preview is not supported
    pub reason: String,
    /// Suggested external applications
    pub suggested_apps: Vec<String>,
    /// Fallback icon name
    pub fallback_icon: String,
    /// Basic file metadata
    pub metadata: BasicFileMetadata,
}
```

## 3. Specific Provider Implementations

### 3.1 Image Preview Provider

```rust
use image::{ImageFormat as ImgFormat, DynamicImage, imageops::FilterType};
use kamadak_exif::Reader as ExifReader;

#[derive(Debug)]
pub struct ImagePreviewProvider {
    supported_formats: HashSet<ImageFormat>,
    max_dimensions: (u32, u32),
    quality_settings: ImageQualitySettings,
}

impl ImagePreviewProvider {
    pub fn new() -> Self {
        Self {
            supported_formats: HashSet::from([
                ImageFormat::Jpeg,
                ImageFormat::Png,
                ImageFormat::Gif,
                ImageFormat::Webp,
                ImageFormat::Svg,
                ImageFormat::Bmp,
                ImageFormat::Tiff,
            ]),
            max_dimensions: (2048, 2048),
            quality_settings: ImageQualitySettings::default(),
        }
    }
    
    async fn load_and_optimize_image(&self, file_path: &Path, options: &PreviewOptions) -> Result<DynamicImage, PreviewError> {
        let data = tokio::fs::read(file_path).await?;
        let image = image::load_from_memory(&data)
            .map_err(|e| PreviewError::ProcessingFailed(format!("Failed to load image: {}", e)))?;
        
        // Resize if needed
        let (width, height) = image.dimensions();
        let max_dims = options.max_dimensions.unwrap_or(self.max_dimensions);
        
        if width > max_dims.0 || height > max_dims.1 {
            let aspect_ratio = width as f32 / height as f32;
            let (new_width, new_height) = if aspect_ratio > 1.0 {
                (max_dims.0, (max_dims.0 as f32 / aspect_ratio) as u32)
            } else {
                ((max_dims.1 as f32 * aspect_ratio) as u32, max_dims.1)
            };
            
            Ok(image.resize(new_width, new_height, FilterType::Lanczos3))
        } else {
            Ok(image)
        }
    }
    
    async fn extract_exif_metadata(&self, file_path: &Path) -> Option<ExifData> {
        if let Ok(file) = std::fs::File::open(file_path) {
            if let Ok(mut bufreader) = std::io::BufReader::new(file).try_into() {
                if let Ok(exif) = ExifReader::new().read_from_container(&mut bufreader) {
                    return Some(ExifData::from_exif(exif));
                }
            }
        }
        None
    }
}

#[async_trait]
impl PreviewProvider for ImagePreviewProvider {
    fn id(&self) -> &'static str { "image_preview" }
    fn name(&self) -> &'static str { "Image Preview Provider" }
    
    fn can_preview(&self, file_type: &FileType, _file_path: &Path) -> bool {
        matches!(file_type, FileType::Image(format) if self.supported_formats.contains(format))
    }
    
    fn priority(&self, file_type: &FileType) -> u8 {
        match file_type {
            FileType::Image(ImageFormat::Jpeg | ImageFormat::Png) => 100,
            FileType::Image(ImageFormat::Webp | ImageFormat::Gif) => 90,
            FileType::Image(_) => 80,
            _ => 0,
        }
    }
    
    async fn generate_preview(&self, request: PreviewRequest) -> Result<PreviewContent, PreviewError> {
        let image = self.load_and_optimize_image(&request.file_path, &request.options).await?;
        
        // Encode optimized image
        let mut buffer = Vec::new();
        let format = ImgFormat::WebP; // Use WebP for efficient storage
        image.write_to(&mut std::io::Cursor::new(&mut buffer), format)?;
        
        // Extract metadata if requested
        let metadata = if request.options.include_metadata {
            let exif = self.extract_exif_metadata(&request.file_path).await;
            ImageMetadata {
                exif,
                color_space: None, // TODO: Implement color space detection
                camera_info: None, // Extract from EXIF
                location: None,    // Extract GPS from EXIF
            }
        } else {
            ImageMetadata::default()
        };
        
        Ok(PreviewContent::Image(ImagePreview {
            data: buffer,
            format: ImageFormat::Webp,
            dimensions: image.dimensions(),
            color_profile: None,
            metadata,
            operations: vec![
                ImageOperation::ZoomIn,
                ImageOperation::ZoomOut,
                ImageOperation::FitToWindow,
                ImageOperation::ActualSize,
                ImageOperation::RotateLeft,
                ImageOperation::RotateRight,
                ImageOperation::FlipHorizontal,
                ImageOperation::FlipVertical,
            ],
        }))
    }
    
    async fn generate_thumbnail(&self, request: ThumbnailRequest) -> Result<ThumbnailData, PreviewError> {
        let thumbnail_size = request.size.unwrap_or((128, 128));
        let data = tokio::fs::read(&request.file_path).await?;
        let image = image::load_from_memory(&data)?;
        
        let thumbnail = image.thumbnail(thumbnail_size.0, thumbnail_size.1);
        let mut buffer = Vec::new();
        thumbnail.write_to(&mut std::io::Cursor::new(&mut buffer), ImgFormat::WebP)?;
        
        Ok(ThumbnailData {
            data: buffer,
            format: ThumbnailFormat::WebP,
            dimensions: thumbnail.dimensions(),
        })
    }
    
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        let data = tokio::fs::read(file_path).await?;
        let image = image::load_from_memory(&data)?;
        let exif = self.extract_exif_metadata(file_path).await;
        
        Ok(FileMetadata::Image {
            dimensions: image.dimensions(),
            color_mode: image.color().into(),
            bit_depth: 8, // TODO: Proper bit depth detection
            exif,
        })
    }
    
    fn supported_operations(&self) -> Vec<PreviewOperation> {
        vec![
            PreviewOperation::Zoom,
            PreviewOperation::Pan,
            PreviewOperation::Rotate,
            PreviewOperation::Flip,
            PreviewOperation::FitToWindow,
            PreviewOperation::ActualSize,
            PreviewOperation::Fullscreen,
        ]
    }
    
    fn configuration_schema(&self) -> Option<ConfigurationSchema> {
        Some(ConfigurationSchema {
            properties: vec![
                ConfigProperty {
                    key: "max_dimensions".to_string(),
                    property_type: ConfigPropertyType::Array,
                    default_value: Some(serde_json::json!([2048, 2048])),
                    description: "Maximum dimensions for image preview".to_string(),
                },
                ConfigProperty {
                    key: "quality".to_string(),
                    property_type: ConfigPropertyType::Number,
                    default_value: Some(serde_json::json!(0.85)),
                    description: "Image quality for preview generation".to_string(),
                },
            ],
        })
    }
    
    fn validate_configuration(&self, config: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        if let Some(quality) = config.get("quality") {
            if let Some(q) = quality.as_f64() {
                if q < 0.0 || q > 1.0 {
                    return Err("Quality must be between 0.0 and 1.0".to_string());
                }
            }
        }
        Ok(())
    }
}
```

### 3.2 Text/Code Preview Provider

```rust
use syntect::parsing::SyntaxSet;
use syntect::highlighting::{ThemeSet, Style};
use syntect::html::highlighted_html_for_string;
use tree_sitter::{Parser, Language};
use chardet::{charset2encoding, detect};

#[derive(Debug)]
pub struct TextPreviewProvider {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    parsers: HashMap<String, Language>,
    max_file_size: usize,
    max_lines: usize,
}

impl TextPreviewProvider {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            parsers: Self::load_tree_sitter_parsers(),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_lines: 10000,
        }
    }
    
    fn load_tree_sitter_parsers() -> HashMap<String, Language> {
        let mut parsers = HashMap::new();
        
        // Load tree-sitter languages
        if let Ok(rust_lang) = tree_sitter_rust::language() {
            parsers.insert("rust".to_string(), rust_lang);
        }
        if let Ok(js_lang) = tree_sitter_javascript::language() {
            parsers.insert("javascript".to_string(), js_lang);
        }
        if let Ok(py_lang) = tree_sitter_python::language() {
            parsers.insert("python".to_string(), py_lang);
        }
        
        parsers
    }
    
    async fn detect_encoding(&self, file_path: &Path) -> Result<String, PreviewError> {
        let sample = tokio::fs::read(file_path.clone()).await?;
        let (encoding, _confidence, _language) = detect(&sample);
        Ok(charset2encoding(&encoding).to_string())
    }
    
    async fn read_with_encoding(&self, file_path: &Path, encoding: &str) -> Result<String, PreviewError> {
        let bytes = tokio::fs::read(file_path).await?;
        
        match encoding {
            "UTF-8" => String::from_utf8(bytes)
                .map_err(|e| PreviewError::EncodingError(format!("UTF-8 decode error: {}", e))),
            _ => {
                // Use encoding_rs for other encodings
                let (cow, _encoding_used, had_errors) = encoding_rs::UTF_8.decode(&bytes);
                if had_errors {
                    Err(PreviewError::EncodingError("Encoding conversion had errors".to_string()))
                } else {
                    Ok(cow.into_owned())
                }
            }
        }
    }
    
    fn detect_language(&self, file_path: &Path, content: &str) -> Option<String> {
        // First try by file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if let Some(syntax) = self.syntax_set.find_syntax_by_extension(ext) {
                return Some(syntax.name.clone());
            }
        }
        
        // Try by first line (shebang)
        if let Some(first_line) = content.lines().next() {
            if let Some(syntax) = self.syntax_set.find_syntax_by_first_line(first_line) {
                return Some(syntax.name.clone());
            }
        }
        
        None
    }
    
    fn generate_syntax_highlighting(&self, content: &str, language: &str, theme: &str) -> Result<SyntaxHighlighting, PreviewError> {
        let syntax = self.syntax_set.find_syntax_by_name(language)
            .or_else(|| self.syntax_set.find_syntax_by_extension(language))
            .ok_or_else(|| PreviewError::UnsupportedLanguage(language.to_string()))?;
        
        let theme = self.theme_set.themes.get(theme)
            .ok_or_else(|| PreviewError::UnsupportedTheme(theme.to_string()))?;
        
        let highlighted = highlighted_html_for_string(content, &self.syntax_set, syntax, theme)?;
        
        Ok(SyntaxHighlighting {
            language: language.to_string(),
            theme: theme.to_string(),
            highlighted_html: highlighted,
            tokens: Vec::new(), // TODO: Extract tokens for custom rendering
        })
    }
    
    fn calculate_statistics(&self, content: &str) -> TextStatistics {
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len();
        let char_count = content.chars().count();
        let word_count = content.split_whitespace().count();
        let byte_count = content.len();
        
        let avg_line_length = if line_count > 0 {
            char_count as f32 / line_count as f32
        } else {
            0.0
        };
        
        let max_line_length = lines.iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        
        let blank_lines = lines.iter().filter(|line| line.trim().is_empty()).count();
        
        TextStatistics {
            line_count,
            char_count,
            word_count,
            byte_count,
            avg_line_length,
            max_line_length,
            blank_lines,
        }
    }
}

#[async_trait]
impl PreviewProvider for TextPreviewProvider {
    fn id(&self) -> &'static str { "text_preview" }
    fn name(&self) -> &'static str { "Text/Code Preview Provider" }
    
    fn can_preview(&self, file_type: &FileType, file_path: &Path) -> bool {
        match file_type {
            FileType::Text(_) | FileType::Code(_) => true,
            FileType::Document(DocumentFormat::Json | DocumentFormat::Xml | DocumentFormat::Csv) => true,
            _ => {
                // Check by extension as fallback
                if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                    self.syntax_set.find_syntax_by_extension(ext).is_some()
                } else {
                    false
                }
            }
        }
    }
    
    fn priority(&self, file_type: &FileType) -> u8 {
        match file_type {
            FileType::Code(_) => 100,
            FileType::Text(TextFormat::Plain) => 90,
            FileType::Text(_) => 85,
            FileType::Document(DocumentFormat::Json | DocumentFormat::Xml) => 80,
            _ => 50,
        }
    }
    
    async fn generate_preview(&self, request: PreviewRequest) -> Result<PreviewContent, PreviewError> {
        // Check file size
        if request.file_size > self.max_file_size as u64 {
            return Err(PreviewError::FileTooLarge {
                size: request.file_size,
                max_size: self.max_file_size as u64,
            });
        }
        
        // Detect encoding
        let encoding = self.detect_encoding(&request.file_path).await?;
        
        // Read file content
        let content = self.read_with_encoding(&request.file_path, &encoding).await?;
        
        // Truncate if too many lines
        let lines: Vec<&str> = content.lines().collect();
        let is_truncated = lines.len() > self.max_lines;
        let display_content = if is_truncated {
            lines[..self.max_lines].join("\n")
        } else {
            content.clone()
        };
        
        // Detect language
        let detected_language = self.detect_language(&request.file_path, &content);
        
        // Generate syntax highlighting if language detected
        let syntax = if let Some(ref lang) = detected_language {
            let theme = request.context.user_preferences.theme_preference
                .unwrap_or_else(|| "base16-ocean.dark".to_string());
            
            match self.generate_syntax_highlighting(&display_content, lang, &theme) {
                Ok(highlighting) => Some(highlighting),
                Err(_) => None, // Fall back to plain text
            }
        } else {
            None
        };
        
        // Calculate statistics
        let statistics = self.calculate_statistics(&content);
        
        Ok(PreviewContent::Text(TextPreview {
            content: display_content,
            syntax,
            encoding,
            line_count: lines.len(),
            char_count: content.chars().count(),
            detected_language,
            is_truncated,
            statistics,
        }))
    }
    
    async fn generate_thumbnail(&self, request: ThumbnailRequest) -> Result<ThumbnailData, PreviewError> {
        // For text files, generate a thumbnail showing the first few lines
        let content = self.read_with_encoding(&request.file_path, "UTF-8").await?;
        let preview_lines: Vec<&str> = content.lines().take(10).collect();
        let preview_text = preview_lines.join("\n");
        
        // Generate a simple text-based thumbnail
        // This would typically involve rendering text to an image
        // For now, return the text data
        Ok(ThumbnailData {
            data: preview_text.into_bytes(),
            format: ThumbnailFormat::Text,
            dimensions: (200, 150), // Fixed size for text thumbnails
        })
    }
    
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        let encoding = self.detect_encoding(file_path).await?;
        let content = self.read_with_encoding(file_path, &encoding).await?;
        let statistics = self.calculate_statistics(&content);
        let detected_language = self.detect_language(file_path, &content);
        
        Ok(FileMetadata::Text {
            encoding,
            line_ending: self.detect_line_ending(&content),
            detected_language,
            statistics,
        })
    }
    
    fn supported_operations(&self) -> Vec<PreviewOperation> {
        vec![
            PreviewOperation::Search,
            PreviewOperation::GoToLine,
            PreviewOperation::ToggleWordWrap,
            PreviewOperation::ToggleLineNumbers,
            PreviewOperation::ChangeTheme,
            PreviewOperation::CopyContent,
            PreviewOperation::PrintPreview,
        ]
    }
    
    fn configuration_schema(&self) -> Option<ConfigurationSchema> {
        Some(ConfigurationSchema {
            properties: vec![
                ConfigProperty {
                    key: "max_file_size".to_string(),
                    property_type: ConfigPropertyType::Number,
                    default_value: Some(serde_json::json!(10485760)), // 10MB
                    description: "Maximum file size for text preview in bytes".to_string(),
                },
                ConfigProperty {
                    key: "max_lines".to_string(),
                    property_type: ConfigPropertyType::Number,
                    default_value: Some(serde_json::json!(10000)),
                    description: "Maximum number of lines to display".to_string(),
                },
                ConfigProperty {
                    key: "default_theme".to_string(),
                    property_type: ConfigPropertyType::String,
                    default_value: Some(serde_json::json!("base16-ocean.dark")),
                    description: "Default syntax highlighting theme".to_string(),
                },
            ],
        })
    }
    
    fn validate_configuration(&self, config: &HashMap<String, serde_json::Value>) -> Result<(), String> {
        if let Some(max_size) = config.get("max_file_size") {
            if let Some(size) = max_size.as_u64() {
                if size > 100 * 1024 * 1024 { // 100MB limit
                    return Err("Maximum file size cannot exceed 100MB".to_string());
                }
            }
        }
        
        if let Some(max_lines) = config.get("max_lines") {
            if let Some(lines) = max_lines.as_u64() {
                if lines > 100000 {
                    return Err("Maximum lines cannot exceed 100,000".to_string());
                }
            }
        }
        
        Ok(())
    }
}
```

## 4. Preview Registry Implementation

```rust
/// Central registry for managing preview providers
pub struct PreviewRegistry {
    providers: Vec<Arc<dyn PreviewProvider>>,
    cache: Arc<PreviewCache>,
    config: PreviewRegistryConfig,
    metrics: Arc<RwLock<PreviewMetrics>>,
}

impl PreviewRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            providers: Vec::new(),
            cache: Arc::new(PreviewCache::new()),
            config: PreviewRegistryConfig::default(),
            metrics: Arc::new(RwLock::new(PreviewMetrics::default())),
        };
        
        // Register built-in providers
        registry.register_builtin_providers();
        
        registry
    }
    
    fn register_builtin_providers(&mut self) {
        self.register(Arc::new(ImagePreviewProvider::new()));
        self.register(Arc::new(TextPreviewProvider::new()));
        self.register(Arc::new(VideoPreviewProvider::new()));
        self.register(Arc::new(AudioPreviewProvider::new()));
        self.register(Arc::new(PdfPreviewProvider::new()));
        self.register(Arc::new(ArchivePreviewProvider::new()));
        self.register(Arc::new(Model3DPreviewProvider::new()));
    }
    
    pub fn register(&mut self, provider: Arc<dyn PreviewProvider>) {
        // Validate provider configuration
        if let Some(schema) = provider.configuration_schema() {
            if let Err(e) = provider.validate_configuration(&HashMap::new()) {
                tracing::warn!("Provider {} has invalid default configuration: {}", provider.id(), e);
            }
        }
        
        self.providers.push(provider);
        
        // Sort providers by priority
        self.providers.sort_by(|a, b| {
            // This is a simplified sort - in practice you'd need a more complex priority system
            a.name().cmp(b.name())
        });
    }
    
    pub async fn generate_preview(&self, file_path: &Path) -> Result<PreviewContent, PreviewError> {
        let start_time = std::time::Instant::now();
        
        // Detect file type
        let file_type = FileType::from_path(file_path);
        let file_size = tokio::fs::metadata(file_path).await?.len();
        
        // Check cache first
        if let Some(cached) = self.cache.get(file_path).await {
            self.metrics.write().await.cache_hits += 1;
            return Ok(cached);
        }
        
        // Find best provider
        let provider = self.find_best_provider(&file_type, file_path)?;
        
        // Create preview request
        let request = PreviewRequest {
            file_path: file_path.to_path_buf(),
            file_type,
            file_size,
            options: PreviewOptions {
                max_dimensions: Some(self.config.max_preview_dimensions),
                quality: self.config.default_quality,
                include_metadata: true,
                provider_options: HashMap::new(),
            },
            context: PreviewContext {
                available_memory: Some(self.config.max_memory_usage),
                timeout: Some(self.config.generation_timeout),
                cache_available: true,
                user_preferences: UserPreferences::default(),
            },
        };
        
        // Generate preview
        let content = provider.generate_preview(request).await?;
        
        // Cache result
        self.cache.set(file_path, content.clone()).await;
        
        // Update metrics
        let duration = start_time.elapsed();
        let mut metrics = self.metrics.write().await;
        metrics.total_previews += 1;
        metrics.total_generation_time += duration;
        metrics.provider_usage.entry(provider.id().to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        
        Ok(content)
    }
    
    fn find_best_provider(&self, file_type: &FileType, file_path: &Path) -> Result<Arc<dyn PreviewProvider>, PreviewError> {
        let mut candidates: Vec<_> = self.providers
            .iter()
            .filter(|provider| provider.can_preview(file_type, file_path))
            .collect();
        
        if candidates.is_empty() {
            return Err(PreviewError::NoProviderFound {
                file_type: file_type.clone(),
                file_path: file_path.to_path_buf(),
            });
        }
        
        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.priority(file_type).cmp(&a.priority(file_type)));
        
        Ok(candidates[0].clone())
    }
    
    pub async fn get_metrics(&self) -> PreviewMetrics {
        self.metrics.read().await.clone()
    }
    
    pub fn get_providers(&self) -> Vec<Arc<dyn PreviewProvider>> {
        self.providers.clone()
    }
}

#[derive(Debug, Clone)]
pub struct PreviewRegistryConfig {
    pub max_preview_dimensions: (u32, u32),
    pub default_quality: f32,
    pub max_memory_usage: usize,
    pub generation_timeout: std::time::Duration,
    pub cache_enabled: bool,
    pub max_cache_size: usize,
}

impl Default for PreviewRegistryConfig {
    fn default() -> Self {
        Self {
            max_preview_dimensions: (2048, 2048),
            default_quality: 0.85,
            max_memory_usage: 512 * 1024 * 1024, // 512MB
            generation_timeout: std::time::Duration::from_secs(30),
            cache_enabled: true,
            max_cache_size: 1024 * 1024 * 1024, // 1GB
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PreviewMetrics {
    pub total_previews: u64,
    pub cache_hits: u64,
    pub total_generation_time: std::time::Duration,
    pub provider_usage: HashMap<String, u64>,
    pub error_counts: HashMap<String, u64>,
}
```

This comprehensive preview services design provides a robust, extensible foundation for implementing file preview capabilities across multiple file types while maintaining performance and user experience standards defined in the PRD.