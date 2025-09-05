use std::path::Path;
use std::time::SystemTime;
use async_trait::async_trait;
use crate::services::preview::{
    PreviewProvider, PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent
};

/// Fallback preview provider for unsupported file types
/// Provides generic file information and placeholder thumbnails
pub struct FallbackPreviewProvider {
    _initialized: bool,
}

impl FallbackPreviewProvider {
    pub fn new() -> Self {
        Self {
            _initialized: true,
        }
    }

    /// Generate a generic file thumbnail based on extension or file type
    fn generate_generic_thumbnail(file_path: &Path) -> Result<Vec<u8>, PreviewError> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Create a simple colored rectangle as placeholder
        // In a real implementation, this would generate an icon based on file type
        let color = match extension.as_str() {
            // Documents
            "doc" | "docx" | "rtf" | "odt" => (70, 130, 180),   // Steel Blue
            "xls" | "xlsx" | "ods" | "csv" => (34, 139, 34),    // Forest Green
            "ppt" | "pptx" | "odp" => (255, 140, 0),            // Dark Orange
            
            // Programming files
            "py" | "js" | "ts" | "rs" | "go" | "java" | "c" | "cpp" | "h" => (128, 0, 128), // Purple
            "html" | "css" | "xml" | "json" | "yaml" | "yml" => (220, 20, 60), // Crimson
            
            // Data files
            "sql" | "db" | "sqlite" => (25, 25, 112),           // Midnight Blue
            "log" | "txt" => (105, 105, 105),                   // Dim Gray
            
            // Archive files (if not handled by ArchivePreviewProvider)
            "rar" | "7z" | "tar.gz" | "bz2" | "xz" => (139, 69, 19), // Saddle Brown
            
            // Unknown files
            _ => (169, 169, 169), // Dark Gray for unknown
        };

        // Create a simple 256x256 RGB image with the specified color
        let width = 256u32;
        let height = 256u32;
        let mut image_data = Vec::with_capacity((width * height * 3) as usize);
        
        // Fill with solid color
        for _ in 0..(width * height) {
            image_data.push(color.0); // R
            image_data.push(color.1); // G
            image_data.push(color.2); // B
        }

        // Create a simple PPM format image (portable pixmap)
        let ppm_header = format!("P6\n{} {}\n255\n", width, height);
        let mut ppm_data = ppm_header.into_bytes();
        ppm_data.extend(image_data);

        Ok(ppm_data)
    }

    /// Extract basic file information for unsupported types
    async fn extract_basic_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        let metadata = std::fs::metadata(file_path)?;
        
        Ok(FileMetadata {
            file_size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            width: Some(256), // Generic thumbnail size
            height: Some(256),
            duration: None,
            bit_rate: None,
            sample_rate: None,
            codec: None,
            title: None,
            artist: None,
            album: None,
            year: None,
            page_count: None,
            color_space: Some("RGB".to_string()),
            compression: None,
            exif_data: None,
        })
    }

    /// Create fallback content based on file characteristics
    fn create_fallback_content(file_path: &Path, thumbnail: Vec<u8>) -> PreviewContent {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase();

        // Try to categorize the file type for appropriate fallback content
        match extension.as_str() {
            // Text-like files - provide basic text preview
            "log" | "txt" | "cfg" | "conf" | "ini" | "env" => {
                // Try to read first few lines as text preview
                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        let preview_text = content
                            .lines()
                            .take(10)
                            .collect::<Vec<_>>()
                            .join("\n");
                        PreviewContent::Text {
                            content: preview_text,
                            language: None,
                            line_count: content.lines().count(),
                        }
                    }
                    Err(_) => PreviewContent::Unsupported {
                        file_type: extension,
                        reason: "Unable to read file content".to_string(),
                        suggested_action: Some("File may be binary or corrupted".to_string()),
                    }
                }
            }
            
            // Known but unsupported office files
            "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => {
                PreviewContent::Unsupported {
                    file_type: extension,
                    reason: "Microsoft Office files require specialized handling".to_string(),
                    suggested_action: Some("Consider opening with appropriate application".to_string()),
                }
            }
            
            // Unknown files
            _ => PreviewContent::Unsupported {
                file_type: extension,
                reason: "File type not recognized or supported".to_string(),
                suggested_action: Some("Check file extension and content".to_string()),
            }
        }
    }
}

#[async_trait]
impl PreviewProvider for FallbackPreviewProvider {
    fn provider_id(&self) -> &'static str {
        "fallback"
    }

    fn provider_name(&self) -> &'static str {
        "Fallback Preview Provider"
    }

    fn supports_format(&self, _format: SupportedFormat) -> bool {
        // The fallback provider supports any format (lowest priority)
        true
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        // Return empty - this indicates it's a catch-all provider
        vec![]
    }

    fn priority(&self) -> u32 {
        // Lowest possible priority - only used when nothing else works
        0
    }

    async fn generate_preview(&self, file_path: &Path, _config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        tracing::info!("Generating fallback preview for: {:?}", file_path);

        // Generate generic thumbnail
        let thumbnail = Self::generate_generic_thumbnail(file_path)?;
        
        // Extract basic metadata
        let file_metadata = Self::extract_basic_metadata(file_path).await?;
        
        // Create fallback content
        let content = Self::create_fallback_content(file_path, thumbnail);
        
        Ok(PreviewData {
            file_path: file_path.to_path_buf(),
            format: SupportedFormat::Text, // Generic format for fallback
            preview_content: content,
            thumbnail_path: None, // Thumbnail is embedded in memory
            metadata: file_metadata,
            generated_at: SystemTime::now(),
        })
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        Self::extract_basic_metadata(file_path).await
    }

    async fn generate_thumbnail(&self, file_path: &Path, _size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        Self::generate_generic_thumbnail(file_path)
    }
}

/// Legacy fallback handler for backward compatibility
pub struct FallbackPreviewHandler {
    provider: FallbackPreviewProvider,
}

impl FallbackPreviewHandler {
    pub fn new() -> Self {
        Self {
            provider: FallbackPreviewProvider::new(),
        }
    }
}

#[async_trait]
impl PreviewHandler for FallbackPreviewHandler {
    fn supports_format(&self, _format: SupportedFormat) -> bool {
        // Fallback handler supports any format as last resort
        true
    }

    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        self.provider.generate_preview(file_path, config).await
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        self.provider.extract_metadata(file_path).await
    }

    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        self.provider.generate_thumbnail(file_path, size).await
    }
}

// Type aliases for consistency
pub type FallbackHandler = FallbackPreviewHandler;

impl Default for FallbackPreviewProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FallbackPreviewHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fallback_provider_basic() {
        let provider = FallbackPreviewProvider::new();
        
        // Test with a fake unsupported file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.unknown");
        std::fs::write(&test_file, b"some binary content").unwrap();
        
        let config = PreviewConfig::default();
        let result = provider.generate_preview(&test_file, &config).await;
        
        assert!(result.is_ok());
        let preview_data = result.unwrap();
        
        // Check that we got fallback content
        match preview_data.preview_content {
            PreviewContent::Unsupported { file_type, .. } => {
                assert_eq!(file_type, "unknown");
            }
            _ => panic!("Expected unsupported content"),
        }
        
        assert_eq!(preview_data.metadata.file_size, 19);
    }

    #[tokio::test]
    async fn test_fallback_text_file() {
        let provider = FallbackPreviewProvider::new();
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.log");
        std::fs::write(&test_file, "Line 1\nLine 2\nLine 3").unwrap();
        
        let config = PreviewConfig::default();
        let result = provider.generate_preview(&test_file, &config).await;
        
        assert!(result.is_ok());
        let preview_data = result.unwrap();
        
        match preview_data.preview_content {
            PreviewContent::Text { content, line_count, .. } => {
                assert!(content.contains("Line 1"));
                assert_eq!(line_count, 3);
            }
            _ => panic!("Expected text content for .log file"),
        }
    }

    #[tokio::test]
    async fn test_fallback_thumbnail_generation() {
        let provider = FallbackPreviewProvider::new();
        
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.doc");
        std::fs::write(&test_file, b"fake word document").unwrap();
        
        let thumbnail = provider.generate_thumbnail(&test_file, (256, 256)).await;
        assert!(thumbnail.is_ok());
        
        let thumbnail_data = thumbnail.unwrap();
        assert!(!thumbnail_data.is_empty());
        
        // Should start with PPM header
        let header = String::from_utf8_lossy(&thumbnail_data[0..20]);
        assert!(header.starts_with("P6"));
    }

    #[tokio::test]
    async fn test_fallback_provider_priority() {
        let provider = FallbackPreviewProvider::new();
        assert_eq!(provider.priority(), 0);
        assert_eq!(provider.provider_id(), "fallback");
        assert!(provider.supports_format(SupportedFormat::Jpeg)); // Supports everything
    }
}