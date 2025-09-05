use std::path::Path;
use std::time::SystemTime;
use async_trait::async_trait;
use crate::services::preview::{
    PreviewProvider, PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent
};

/// Archive preview provider supporting zip, tar, and other archive formats
pub struct ArchivePreviewProvider {
    _initialized: bool,
}

impl ArchivePreviewProvider {
    pub fn new() -> Result<Self, PreviewError> {
        Ok(Self {
            _initialized: true,
        })
    }

    fn extract_archive_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        use std::fs;

        let mut metadata = FileMetadata::new();
        
        // Get file system metadata
        let fs_metadata = fs::metadata(file_path)?;
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Guess format from extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "zip" => metadata.codec = Some("ZIP Archive".to_string()),
                "tar" => metadata.codec = Some("TAR Archive".to_string()),
                "gz" => metadata.codec = Some("GZIP Archive".to_string()),
                "7z" => metadata.codec = Some("7-Zip Archive".to_string()),
                "rar" => metadata.codec = Some("RAR Archive".to_string()),
                _ => metadata.codec = Some("Unknown Archive".to_string()),
            }
        }
        
        Ok(metadata)
    }

    fn extract_archive_contents(file_path: &Path) -> Result<Vec<String>, PreviewError> {
        // For now, just return basic information
        // Real implementation would use zip, tar-rs, or other archive libraries
        let mut contents = Vec::new();
        
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            match ext.to_lowercase().as_str() {
                "zip" => contents.push("ZIP archive - contents extraction not implemented".to_string()),
                "tar" => contents.push("TAR archive - contents extraction not implemented".to_string()),
                "gz" => contents.push("GZIP archive - contents extraction not implemented".to_string()),
                "7z" => contents.push("7-Zip archive - contents extraction not implemented".to_string()),
                "rar" => contents.push("RAR archive - contents extraction not implemented".to_string()),
                _ => contents.push("Unknown archive format".to_string()),
            }
        }
        
        contents.push(format!("File size: {} bytes", 
            std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0)
        ));
        
        Ok(contents)
    }

    fn create_archive_placeholder_thumbnail() -> Result<Vec<u8>, PreviewError> {
        use image::{RgbImage, DynamicImage, ImageFormat};
        
        let (width, height) = (256, 256);
        let mut img = RgbImage::new(width, height);
        
        // Create a folder-like appearance
        let bg_color = image::Rgb([245, 245, 245]); // Light background
        let folder_color = image::Rgb([255, 206, 84]); // Yellow folder
        let border_color = image::Rgb([200, 150, 50]); // Darker yellow border
        
        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg_color;
        }
        
        // Draw folder shape
        let folder_x = 40;
        let folder_y = 80;
        let folder_width = 176;
        let folder_height = 120;
        
        // Fill folder body
        for x in folder_x..folder_x + folder_width {
            for y in folder_y..folder_y + folder_height {
                if x < width && y < height {
                    img.put_pixel(x, y, folder_color);
                }
            }
        }
        
        // Draw folder tab
        let tab_width = 60;
        let tab_height = 20;
        for x in folder_x..folder_x + tab_width {
            for y in folder_y - tab_height..folder_y {
                if x < width && y < height && y >= 0 {
                    img.put_pixel(x, y as u32, folder_color);
                }
            }
        }
        
        // Draw borders
        for x in folder_x..folder_x + folder_width {
            if x < width {
                if folder_y < height { img.put_pixel(x, folder_y, border_color); }
                if folder_y + folder_height - 1 < height { img.put_pixel(x, folder_y + folder_height - 1, border_color); }
            }
        }
        for y in folder_y..folder_y + folder_height {
            if y < height {
                if folder_x < width { img.put_pixel(folder_x, y, border_color); }
                if folder_x + folder_width - 1 < width { img.put_pixel(folder_x + folder_width - 1, y, border_color); }
            }
        }
        
        // Add "ZIP" text indicator
        let text_x = folder_x + 20;
        let text_y = folder_y + 40;
        let text_color = image::Rgb([100, 70, 20]);
        
        // Simple block letters (very basic)
        for x in text_x..text_x + 80 {
            for y in text_y..text_y + 8 {
                if x < width && y < height {
                    img.put_pixel(x, y, text_color);
                }
            }
        }
        
        // Encode as PNG
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        dynamic_img.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| PreviewError::ArchiveError(format!("Failed to encode archive thumbnail: {}", e)))?;
        
        Ok(buffer)
    }
}

#[async_trait]
impl PreviewProvider for ArchivePreviewProvider {
    fn provider_id(&self) -> &'static str {
        "archive"
    }
    
    fn provider_name(&self) -> &'static str {
        "Archive Preview Provider"
    }
    
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_archive()
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["zip", "tar", "gz", "7z", "rar", "bz2", "xz"]
    }
    
    async fn generate_preview(&self, file_path: &Path, _config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format from extension
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        )
        .filter(|f| f.is_archive())
        .ok_or_else(|| PreviewError::UnsupportedFormat("Not an archive file".to_string()))?;

        // Extract metadata
        let metadata = Self::extract_archive_metadata(file_path)?;
        
        // Extract archive contents list
        let contents = Self::extract_archive_contents(file_path)
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to extract archive contents: {}", e);
                vec!["Unable to extract archive contents".to_string()]
            });
        
        // Generate archive thumbnail
        let thumbnail = Self::create_archive_placeholder_thumbnail()?;
        
        let preview_content = PreviewContent::Archive {
            contents,
            thumbnail,
        };
        
        Ok(PreviewData {
            file_path: file_path.to_path_buf(),
            format,
            thumbnail_path: None,
            metadata,
            preview_content,
            generated_at: SystemTime::now(),
        })
    }
    
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        Self::extract_archive_metadata(file_path)
    }
    
    async fn generate_thumbnail(&self, file_path: &Path, _size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        let _ = file_path; // Suppress unused variable warning
        Self::create_archive_placeholder_thumbnail()
    }
    
    fn supports_background_processing(&self) -> bool {
        true // Archive extraction can benefit from background processing
    }
    
    fn priority(&self) -> u32 {
        260 // Higher priority than generic handlers
    }
}

impl Default for ArchivePreviewProvider {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create ArchivePreviewProvider: {}", e);
            panic!("Archive support not available");
        })
    }
}

// Legacy type alias for backward compatibility
pub type ArchivePreviewHandler = ArchivePreviewProvider;

#[async_trait]
impl PreviewHandler for ArchivePreviewProvider {
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_archive()
    }

    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        <Self as PreviewProvider>::generate_preview(self, file_path, config).await
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        <Self as PreviewProvider>::extract_metadata(self, file_path).await
    }

    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        <Self as PreviewProvider>::generate_thumbnail(self, file_path, size).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_archive_provider_supports_formats() {
        let provider = ArchivePreviewProvider::new().unwrap();
        
        // Test format support (assuming SupportedFormat has archive types)
        // Note: These tests depend on the actual SupportedFormat implementation
        let extensions = provider.supported_extensions();
        assert!(extensions.contains(&"zip"));
        assert!(extensions.contains(&"tar"));
        assert!(extensions.contains(&"gz"));
        
        assert_eq!(provider.provider_id(), "archive");
        assert_eq!(provider.provider_name(), "Archive Preview Provider");
        assert!(provider.supports_background_processing());
        assert_eq!(provider.priority(), 260);
    }

    #[tokio::test]
    async fn test_archive_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.zip");
        
        // Create a fake archive file
        fs::write(&archive_path, b"fake zip data").unwrap();
        
        let metadata = ArchivePreviewProvider::extract_archive_metadata(&archive_path).unwrap();
        assert_eq!(metadata.codec, Some("ZIP Archive".to_string()));
        assert!(metadata.file_size > 0);
    }

    #[tokio::test]
    async fn test_archive_contents_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let archive_path = temp_dir.path().join("test.tar");
        
        // Create a fake archive file
        fs::write(&archive_path, b"fake tar data").unwrap();
        
        let contents = ArchivePreviewProvider::extract_archive_contents(&archive_path).unwrap();
        assert!(!contents.is_empty());
        assert!(contents[0].contains("TAR archive"));
    }

    #[test]
    fn test_archive_placeholder_thumbnail() {
        let result = ArchivePreviewProvider::create_archive_placeholder_thumbnail();
        assert!(result.is_ok());
        
        let thumbnail_data = result.unwrap();
        assert!(!thumbnail_data.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_archive_file() {
        let temp_dir = TempDir::new().unwrap();
        let text_path = temp_dir.path().join("test.txt");
        fs::write(&text_path, "This is not an archive").unwrap();
        
        let provider = ArchivePreviewProvider::new().unwrap();
        let config = PreviewConfig::default();
        let result = provider.generate_preview(&text_path, &config).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PreviewError::UnsupportedFormat(_) => {}, // Expected
            other => panic!("Expected UnsupportedFormat, got {:?}", other),
        }
    }
}