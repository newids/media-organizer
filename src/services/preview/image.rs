use std::path::Path;
use std::time::SystemTime;
use async_trait::async_trait;
use image::{ImageFormat, DynamicImage, GenericImageView};
use crate::services::preview::{
    PreviewProvider, PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent, ExifData
};

/// Image preview provider supporting multiple formats using the image crate v0.24
pub struct ImagePreviewProvider;

impl ImagePreviewProvider {
    pub fn new() -> Self {
        Self
    }

    /// Convert image crate format to our internal format
    fn map_image_format(format: ImageFormat) -> Option<SupportedFormat> {
        match format {
            ImageFormat::Jpeg => Some(SupportedFormat::Jpeg),
            ImageFormat::Png => Some(SupportedFormat::Png),
            ImageFormat::Gif => Some(SupportedFormat::Gif),
            ImageFormat::WebP => Some(SupportedFormat::WebP),
            ImageFormat::Tiff => Some(SupportedFormat::Tiff),
            ImageFormat::Bmp => Some(SupportedFormat::Bmp),
            _ => None,
        }
    }

    /// Generate thumbnail from loaded image
    fn create_thumbnail(img: &DynamicImage, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        use image::imageops::FilterType;

        // Calculate aspect ratio preserving dimensions
        let (orig_width, orig_height) = img.dimensions();
        let (target_width, target_height) = size;
        
        let aspect_ratio = orig_width as f32 / orig_height as f32;
        let target_aspect = target_width as f32 / target_height as f32;
        
        let (new_width, new_height) = if aspect_ratio > target_aspect {
            // Image is wider, fit to width
            (target_width, (target_width as f32 / aspect_ratio) as u32)
        } else {
            // Image is taller, fit to height
            ((target_height as f32 * aspect_ratio) as u32, target_height)
        };

        // Resize image
        let thumbnail = img.resize(new_width, new_height, FilterType::Lanczos3);
        
        // Encode as PNG for consistent format
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        thumbnail.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| PreviewError::ImageError(format!("Failed to encode thumbnail: {}", e)))?;
        
        Ok(buffer)
    }

    /// Extract basic image metadata
    fn extract_image_metadata(img: &DynamicImage, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        let (width, height) = img.dimensions();
        
        // Get file system metadata
        let fs_metadata = std::fs::metadata(file_path)
            .map_err(|e| PreviewError::IoError(e))?;
        
        let file_size = fs_metadata.len();
        let created = fs_metadata.created().ok();
        let modified = fs_metadata.modified().ok();

        // Determine color space
        let color_space = match img.color() {
            image::ColorType::L8 => Some("Grayscale".to_string()),
            image::ColorType::La8 => Some("Grayscale with Alpha".to_string()),
            image::ColorType::Rgb8 => Some("RGB".to_string()),
            image::ColorType::Rgba8 => Some("RGBA".to_string()),
            image::ColorType::L16 => Some("16-bit Grayscale".to_string()),
            image::ColorType::La16 => Some("16-bit Grayscale with Alpha".to_string()),
            image::ColorType::Rgb16 => Some("16-bit RGB".to_string()),
            image::ColorType::Rgba16 => Some("16-bit RGBA".to_string()),
            image::ColorType::Rgb32F => Some("32-bit Float RGB".to_string()),
            image::ColorType::Rgba32F => Some("32-bit Float RGBA".to_string()),
            _ => None,
        };

        Ok(FileMetadata {
            file_size,
            created,
            modified,
            width: Some(width),
            height: Some(height),
            duration: None,
            bit_rate: None,
            sample_rate: None,
            codec: None,
            title: None,
            artist: None,
            album: None,
            year: None,
            page_count: None,
            color_space,
            compression: None,
            exif_data: None, // Will be extracted separately
        })
    }

    /// Extract EXIF data from image file
    /// TODO: Implement EXIF extraction when a suitable library is available
    fn extract_exif_data(_file_path: &Path) -> Option<ExifData> {
        // EXIF extraction disabled until we find a compatible library
        // For now, return None to indicate no EXIF data available
        None
    }

    /// Detect image format from file content
    async fn detect_format_from_content(file_path: &Path) -> Result<SupportedFormat, PreviewError> {
        // Try to guess from extension first (simpler approach)
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            SupportedFormat::from_extension(ext)
                .filter(|f| f.is_image())
                .ok_or_else(|| PreviewError::UnsupportedFormat(ext.to_string()))
        } else {
            Err(PreviewError::UnsupportedFormat("Unknown".to_string()))
        }
    }
}

#[async_trait]
impl PreviewProvider for ImagePreviewProvider {
    fn provider_id(&self) -> &'static str {
        "image"
    }
    
    fn provider_name(&self) -> &'static str {
        "Image Preview Provider"
    }
    
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_image()
    }
    
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["jpg", "jpeg", "png", "gif", "webp", "tiff", "tif", "bmp", "svg"]
    }
    
    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format
        let format = Self::detect_format_from_content(file_path).await?;
        
        // Load image
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        // Extract metadata
        let mut metadata = Self::extract_image_metadata(&img, file_path)?;
        
        // Extract EXIF data
        metadata.exif_data = Self::extract_exif_data(file_path);

        // Generate thumbnail
        let thumbnail_data = Self::create_thumbnail(&img, config.thumbnail_size)?;

        // Determine original format string
        let original_format = match format {
            SupportedFormat::Jpeg => "JPEG",
            SupportedFormat::Png => "PNG", 
            SupportedFormat::Gif => "GIF",
            SupportedFormat::WebP => "WebP",
            SupportedFormat::Tiff => "TIFF",
            SupportedFormat::Bmp => "BMP",
            SupportedFormat::Svg => "SVG",
            _ => "Unknown",
        }.to_string();

        let preview_content = PreviewContent::Image {
            thumbnail_data,
            original_format,
        };

        Ok(PreviewData {
            file_path: file_path.to_path_buf(),
            format,
            thumbnail_path: None, // Will be set by service if saved to disk
            metadata,
            preview_content,
            generated_at: SystemTime::now(),
        })
    }
    
    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        // For images, we need to load the image to get dimensions
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        let mut metadata = Self::extract_image_metadata(&img, file_path)?;
        metadata.exif_data = Self::extract_exif_data(file_path);

        Ok(metadata)
    }
    
    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // Load image
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        // Generate thumbnail
        Self::create_thumbnail(&img, size)
    }
    
    fn supports_background_processing(&self) -> bool {
        true // Large images can benefit from background processing
    }
    
    fn priority(&self) -> u32 {
        200 // Higher priority than generic handlers
    }
}

// Legacy PreviewHandler implementation for backward compatibility
#[async_trait]
impl PreviewHandler for ImagePreviewProvider {
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_image()
    }

    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format
        let format = Self::detect_format_from_content(file_path).await?;
        
        // Load image
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        // Extract metadata
        let mut metadata = Self::extract_image_metadata(&img, file_path)?;
        
        // Extract EXIF data
        metadata.exif_data = Self::extract_exif_data(file_path);

        // Generate thumbnail
        let thumbnail_data = Self::create_thumbnail(&img, config.thumbnail_size)?;

        // Determine original format string
        let original_format = match format {
            SupportedFormat::Jpeg => "JPEG",
            SupportedFormat::Png => "PNG", 
            SupportedFormat::Gif => "GIF",
            SupportedFormat::WebP => "WebP",
            SupportedFormat::Tiff => "TIFF",
            SupportedFormat::Bmp => "BMP",
            SupportedFormat::Svg => "SVG",
            _ => "Unknown",
        }.to_string();

        let preview_content = PreviewContent::Image {
            thumbnail_data,
            original_format,
        };

        Ok(PreviewData {
            file_path: file_path.to_path_buf(),
            format,
            thumbnail_path: None, // Will be set by service if saved to disk
            metadata,
            preview_content,
            generated_at: SystemTime::now(),
        })
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        // For images, we need to load the image to get dimensions
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        let mut metadata = Self::extract_image_metadata(&img, file_path)?;
        metadata.exif_data = Self::extract_exif_data(file_path);

        Ok(metadata)
    }

    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // Load image
        let img = image::open(file_path)
            .map_err(|e| PreviewError::ImageError(format!("Failed to load image: {}", e)))?;

        // Generate thumbnail
        Self::create_thumbnail(&img, size)
    }
}

impl Default for ImagePreviewProvider {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy type alias for backward compatibility
pub type ImagePreviewHandler = ImagePreviewProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_image_provider_supports_formats() {
        let provider = ImagePreviewProvider::new();
        
        assert!(provider.supports_format(SupportedFormat::Jpeg));
        assert!(provider.supports_format(SupportedFormat::Png));
        assert!(provider.supports_format(SupportedFormat::Gif));
        assert!(!provider.supports_format(SupportedFormat::Mp4));
        assert!(!provider.supports_format(SupportedFormat::Pdf));
        
        // Test provider metadata
        assert_eq!(provider.provider_id(), "image");
        assert_eq!(provider.provider_name(), "Image Preview Provider");
        assert!(provider.supports_background_processing());
        assert_eq!(provider.priority(), 200);
        
        // Test supported extensions
        let extensions = provider.supported_extensions();
        assert!(extensions.contains(&"jpg"));
        assert!(extensions.contains(&"png"));
        assert!(extensions.contains(&"gif"));
    }

    #[tokio::test] 
    async fn test_create_test_image() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.png");
        
        // Create a simple test image
        let img = image::RgbImage::new(100, 100);
        let dynamic_img = image::DynamicImage::ImageRgb8(img);
        
        dynamic_img.save(&image_path).unwrap();
        
        let provider = ImagePreviewProvider::new();
        let config = PreviewConfig::default();
        
        // Test preview generation
        let result = provider.generate_preview(&image_path, &config).await;
        assert!(result.is_ok());
        
        let preview = result.unwrap();
        assert_eq!(preview.format, SupportedFormat::Png);
        assert_eq!(preview.metadata.width, Some(100));
        assert_eq!(preview.metadata.height, Some(100));
        
        // Test thumbnail generation
        let thumbnail_result = provider.generate_thumbnail(&image_path, (64, 64)).await;
        assert!(thumbnail_result.is_ok());
        let thumbnail_data = thumbnail_result.unwrap();
        assert!(!thumbnail_data.is_empty());
    }

    #[tokio::test]
    async fn test_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let image_path = temp_dir.path().join("test.png");
        
        // Create a test image
        let img = image::RgbImage::new(200, 150);
        let dynamic_img = image::DynamicImage::ImageRgb8(img);
        dynamic_img.save(&image_path).unwrap();
        
        let provider = ImagePreviewProvider::new();
        
        let metadata_result = provider.extract_metadata(&image_path).await;
        assert!(metadata_result.is_ok());
        
        let metadata = metadata_result.unwrap();
        assert_eq!(metadata.width, Some(200));
        assert_eq!(metadata.height, Some(150));
        assert!(metadata.file_size > 0);
        assert!(metadata.color_space.is_some());
    }

    #[test]
    fn test_thumbnail_creation() {
        let img = image::RgbImage::new(400, 300);
        let dynamic_img = image::DynamicImage::ImageRgb8(img);
        
        let thumbnail_result = ImagePreviewProvider::create_thumbnail(&dynamic_img, (100, 100));
        assert!(thumbnail_result.is_ok());
        
        let thumbnail_data = thumbnail_result.unwrap();
        assert!(!thumbnail_data.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_file() {
        let temp_dir = TempDir::new().unwrap();
        let text_path = temp_dir.path().join("test.txt");
        fs::write(&text_path, "This is not an image").unwrap();
        
        let provider = ImagePreviewProvider::new();
        let config = PreviewConfig::default();
        
        let result = provider.generate_preview(&text_path, &config).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PreviewError::UnsupportedFormat(_) => {}, // Expected for non-image files
            other => panic!("Expected UnsupportedFormat, got {:?}", other),
        }
    }
}