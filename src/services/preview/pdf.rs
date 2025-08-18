use std::path::Path;
use std::time::SystemTime;
use async_trait::async_trait;
use crate::services::preview::{
    PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent
};

/// PDF preview handler supporting basic document metadata extraction
pub struct PdfPreviewHandler {
    #[cfg(feature = "pdf")]
    _initialized: bool,
}

impl PdfPreviewHandler {
    pub fn new() -> Result<Self, PreviewError> {
        #[cfg(feature = "pdf")]
        {
            Ok(Self {
                _initialized: true,
            })
        }
        
        #[cfg(not(feature = "pdf"))]
        {
            Err(PreviewError::PdfError("PDF support not enabled. Enable the 'pdf' feature.".to_string()))
        }
    }

    #[cfg(feature = "pdf")]
    fn extract_pdf_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        use std::fs;
        
        let mut metadata = FileMetadata::new();
        
        // Get file system metadata
        let fs_metadata = fs::metadata(file_path)?;
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Read the first part of the PDF to verify it's valid and extract basic info
        let file_content = fs::read(file_path)
            .map_err(|e| PreviewError::PdfError(format!("Failed to read PDF file: {}", e)))?;
            
        if file_content.len() < 4 || &file_content[0..4] != b"%PDF" {
            return Err(PreviewError::PdfError("Not a valid PDF file".to_string()));
        }
        
        // Extract PDF version from header
        if file_content.len() > 8 {
            let header = String::from_utf8_lossy(&file_content[0..8]);
            if let Some(version) = header.strip_prefix("%PDF-") {
                metadata.codec = Some(format!("PDF {}", version));
            } else {
                metadata.codec = Some("PDF".to_string());
            }
        } else {
            metadata.codec = Some("PDF".to_string());
        }
        
        // Basic page count estimation by counting page objects
        let content_str = String::from_utf8_lossy(&file_content);
        let page_objects = content_str.matches("/Type/Page").count() + content_str.matches("/Type /Page").count();
        metadata.page_count = Some(if page_objects > 0 { page_objects as u32 } else { 1 });
        
        // Try to extract document metadata from PDF metadata dictionary
        if let Some(title_match) = Self::extract_pdf_string(&content_str, "/Title") {
            metadata.title = Some(title_match);
        }
        
        // Try to extract author from PDF metadata dictionary
        if let Some(author_match) = Self::extract_pdf_string(&content_str, "/Author") {
            metadata.artist = Some(author_match);
        }
        
        // Try to extract subject as album (since PDF doesn't have direct album concept)
        if let Some(subject_match) = Self::extract_pdf_string(&content_str, "/Subject") {
            metadata.album = Some(subject_match);
        }
        
        // Try to extract creation date
        if let Some(creation_date) = Self::extract_pdf_string(&content_str, "/CreationDate") {
            if let Some(parsed_date) = Self::parse_pdf_date(&creation_date) {
                metadata.created = Some(parsed_date);
            }
        }
        
        // Try to extract modification date
        if let Some(mod_date) = Self::extract_pdf_string(&content_str, "/ModDate") {
            if let Some(parsed_date) = Self::parse_pdf_date(&mod_date) {
                metadata.modified = Some(parsed_date);
            }
        }
        
        // Try to extract producer/creator information
        if let Some(producer) = Self::extract_pdf_string(&content_str, "/Producer") {
            if metadata.codec.is_some() {
                metadata.codec = Some(format!("{} ({})", metadata.codec.unwrap(), producer));
            } else {
                metadata.codec = Some(format!("PDF ({})", producer));
            }
        }
        
        Ok(metadata)
    }

    #[cfg(feature = "pdf")]
    fn extract_pdf_string(content: &str, key: &str) -> Option<String> {
        // Look for patterns like "/Title (some title)" or "/Title<some hex>"
        if let Some(start_pos) = content.find(key) {
            let remaining = &content[start_pos + key.len()..];
            
            // Handle parentheses format: /Title (some title)
            if let Some(paren_start) = remaining.find('(') {
                let after_paren = &remaining[paren_start + 1..];
                if let Some(paren_end) = after_paren.find(')') {
                    let title = &after_paren[0..paren_end];
                    if !title.trim().is_empty() {
                        return Some(title.trim().to_string());
                    }
                }
            }
            
            // Handle hex string format: /Title<48656C6C6F>
            if let Some(hex_start) = remaining.find('<') {
                let after_hex = &remaining[hex_start + 1..];
                if let Some(hex_end) = after_hex.find('>') {
                    let hex_str = &after_hex[0..hex_end];
                    if let Ok(decoded) = Self::decode_hex_string(hex_str) {
                        if !decoded.trim().is_empty() {
                            return Some(decoded.trim().to_string());
                        }
                    }
                }
            }
        }
        None
    }

    #[cfg(feature = "pdf")]
    fn parse_pdf_date(date_str: &str) -> Option<SystemTime> {
        // PDF date format: D:YYYYMMDDHHmmSSOHH'mm'
        // Example: D:20230615142030+02'00'
        if date_str.len() < 14 || !date_str.starts_with('D') {
            return None;
        }
        
        let date_part = &date_str[2..]; // Skip "D:"
        
        // Extract year, month, day, hour, minute, second
        if date_part.len() >= 14 {
            if let (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute), Ok(second)) = (
                date_part[0..4].parse::<i32>(),
                date_part[4..6].parse::<u32>(),
                date_part[6..8].parse::<u32>(),
                date_part[8..10].parse::<u32>(),
                date_part[10..12].parse::<u32>(),
                date_part[12..14].parse::<u32>(),
            ) {
                use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
                
                if let Some(naive_date) = NaiveDate::from_ymd_opt(year, month, day) {
                    if let Some(naive_time) = NaiveTime::from_hms_opt(hour, minute, second) {
                        let naive_datetime = NaiveDateTime::new(naive_date, naive_time);
                        let timestamp = naive_datetime.and_utc().timestamp();
                        
                        if timestamp >= 0 {
                            return Some(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(timestamp as u64));
                        }
                    }
                }
            }
        }
        
        None
    }

    #[cfg(feature = "pdf")]
    fn decode_hex_string(hex: &str) -> Result<String, PreviewError> {
        let mut result = Vec::new();
        let clean_hex = hex.trim();
        
        for chunk in clean_hex.as_bytes().chunks(2) {
            if chunk.len() == 2 {
                let hex_byte = std::str::from_utf8(chunk)
                    .map_err(|_| PreviewError::PdfError("Invalid hex string".to_string()))?;
                
                if let Ok(byte_val) = u8::from_str_radix(hex_byte, 16) {
                    result.push(byte_val);
                } else {
                    return Err(PreviewError::PdfError("Invalid hex value".to_string()));
                }
            }
        }
        
        String::from_utf8(result).map_err(|_| PreviewError::PdfError("Invalid UTF-8 in hex string".to_string()))
    }

    #[cfg(feature = "pdf")]
    fn extract_document_outline(file_path: &Path) -> Result<Vec<String>, PreviewError> {
        use std::fs;
        
        let file_content = fs::read(file_path)?;
        let content_str = String::from_utf8_lossy(&file_content);
        
        let mut outline = Vec::new();
        
        // Count pages for basic information
        let page_objects = content_str.matches("/Type/Page").count() + content_str.matches("/Type /Page").count();
        let page_count = if page_objects > 0 { page_objects } else { 1 };
        
        // Look for outline/bookmarks structure
        if content_str.contains("/Outlines") || content_str.contains("/Outline") {
            outline.push("Document contains bookmarks/outline".to_string());
        }
        
        // Basic document information
        outline.push(format!("Document contains {} page{}", page_count, if page_count == 1 { "" } else { "s" }));
        
        // Try to find text content as a basic outline
        let text_objects = content_str.matches("BT").count(); // Begin Text operator
        if text_objects > 0 {
            outline.push(format!("Contains {} text object{}", text_objects, if text_objects == 1 { "" } else { "s" }));
        }
        
        // Check for images
        let image_objects = content_str.matches("/Subtype/Image").count() + content_str.matches("/Subtype /Image").count();
        if image_objects > 0 {
            outline.push(format!("Contains {} image{}", image_objects, if image_objects == 1 { "" } else { "s" }));
        }
        
        Ok(outline)
    }

    fn create_pdf_placeholder_thumbnail() -> Result<Vec<u8>, PreviewError> {
        use image::{RgbImage, DynamicImage, ImageFormat};
        
        let (width, height) = (256, 256);
        let mut img = RgbImage::new(width, height);
        
        // Create a document-like appearance
        let bg_color = image::Rgb([240, 240, 240]); // Light gray background
        let border_color = image::Rgb([200, 200, 200]); // Gray border
        let text_color = image::Rgb([60, 60, 60]); // Dark gray for text lines
        
        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg_color;
        }
        
        // Draw document border
        let border_width = 2;
        for x in 0..width {
            for y in 0..border_width {
                if y < height { img.put_pixel(x, y, border_color); }
                if height - y - 1 < height { img.put_pixel(x, height - y - 1, border_color); }
            }
        }
        for y in 0..height {
            for x in 0..border_width {
                if x < width { img.put_pixel(x, y, border_color); }
                if width - x - 1 < width { img.put_pixel(width - x - 1, y, border_color); }
            }
        }
        
        // Draw text lines to simulate document content
        let line_height = 20;
        let margin = 20;
        let line_width = width - margin * 2;
        
        for line in 0..8 {
            let y_pos = margin + line * line_height;
            if y_pos + 2 < height {
                let line_length = if line == 7 { line_width / 2 } else { line_width }; // Last line shorter
                for x in margin..(margin + line_length).min(width - margin) {
                    if x < width && y_pos < height && y_pos + 1 < height {
                        img.put_pixel(x, y_pos, text_color);
                        img.put_pixel(x, y_pos + 1, text_color);
                    }
                }
            }
        }
        
        // Add PDF icon in corner
        let icon_size = 24;
        let icon_x = width - icon_size - 10;
        let icon_y = 10;
        
        // Simple PDF text indicator
        if icon_x < width && icon_y < height {
            // Red background for "PDF" indicator
            for x in icon_x..icon_x + icon_size {
                for y in icon_y..icon_y + icon_size / 3 {
                    if x < width && y < height {
                        img.put_pixel(x, y, image::Rgb([200, 50, 50]));
                    }
                }
            }
        }
        
        // Encode as PNG
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        dynamic_img.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| PreviewError::PdfError(format!("Failed to encode PDF thumbnail: {}", e)))?;
        
        Ok(buffer)
    }

    #[cfg(not(feature = "pdf"))]
    fn extract_pdf_metadata_fallback(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        // Fallback implementation without PDF processing
        let fs_metadata = std::fs::metadata(file_path)?;
        
        let mut metadata = FileMetadata::new();
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Basic PDF assumptions for fallback
        metadata.codec = Some("PDF".to_string());
        metadata.page_count = Some(1); // Assume at least 1 page
        
        Ok(metadata)
    }

    #[cfg(not(feature = "pdf"))]
    fn extract_document_outline_fallback() -> Vec<String> {
        vec!["PDF preview not available - enable 'pdf' feature".to_string()]
    }
}

#[async_trait]
impl PreviewHandler for PdfPreviewHandler {
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_document()
    }

    async fn generate_preview(&self, file_path: &Path, _config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format from extension
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        )
        .filter(|f| f.is_document())
        .ok_or_else(|| PreviewError::UnsupportedFormat("Not a PDF file".to_string()))?;

        #[cfg(feature = "pdf")]
        {            
            // Extract metadata
            let metadata = Self::extract_pdf_metadata(file_path)?;
                
            // Extract document outline
            let outline = Self::extract_document_outline(file_path)
                .unwrap_or_else(|e| {
                    tracing::warn!("Failed to extract PDF outline: {}", e);
                    vec!["Unable to extract document outline".to_string()]
                });
            
            // Generate first page thumbnail
            let first_page_image = Self::create_pdf_placeholder_thumbnail()?;
            
            let preview_content = PreviewContent::Document {
                first_page_image,
                outline,
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
        
        #[cfg(not(feature = "pdf"))]
        {
            // Fallback implementation without PDF processing
            let metadata = Self::extract_pdf_metadata_fallback(file_path)?;
            let outline = Self::extract_document_outline_fallback();
            let first_page_image = Self::create_pdf_placeholder_thumbnail()?;
            
            let preview_content = PreviewContent::Document {
                first_page_image,
                outline,
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
    }

    async fn extract_metadata(&self, file_path: &Path) -> Result<FileMetadata, PreviewError> {
        #[cfg(feature = "pdf")]
        {
            Self::extract_pdf_metadata(file_path)
        }
        
        #[cfg(not(feature = "pdf"))]
        {
            Self::extract_pdf_metadata_fallback(file_path)
        }
    }

    async fn generate_thumbnail(&self, file_path: &Path, _size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // For now, always generate placeholder thumbnail
        // Real PDF page rendering would require additional dependencies
        let _ = file_path; // Suppress unused variable warning
        Self::create_pdf_placeholder_thumbnail()
    }
}

impl Default for PdfPreviewHandler {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create PdfPreviewHandler: {}", e);
            panic!("PDF support not available");
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_pdf_handler_supports_formats() {
        let result = PdfPreviewHandler::new();
        
        #[cfg(feature = "pdf")]
        {
            let handler = result.unwrap();
            assert!(handler.supports_format(SupportedFormat::Pdf));
            assert!(!handler.supports_format(SupportedFormat::Jpeg));
            assert!(!handler.supports_format(SupportedFormat::Mp4));
        }
        
        #[cfg(not(feature = "pdf"))]
        {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_pdf_metadata_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        
        // Create a fake PDF file
        fs::write(&pdf_path, b"%PDF-1.4\nfake pdf data").unwrap();
        
        #[cfg(not(feature = "pdf"))]
        {
            let metadata = PdfPreviewHandler::extract_pdf_metadata_fallback(&pdf_path).unwrap();
            assert_eq!(metadata.codec, Some("PDF".to_string()));
            assert_eq!(metadata.page_count, Some(1));
            assert!(metadata.file_size > 0);
        }
    }

    #[test]
    fn test_pdf_placeholder_thumbnail() {
        let result = PdfPreviewHandler::create_pdf_placeholder_thumbnail();
        assert!(result.is_ok());
        
        let thumbnail_data = result.unwrap();
        assert!(!thumbnail_data.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_pdf_file() {
        let temp_dir = TempDir::new().unwrap();
        let text_path = temp_dir.path().join("test.txt");
        fs::write(&text_path, "This is not a PDF").unwrap();
        
        if let Ok(handler) = PdfPreviewHandler::new() {
            let config = PreviewConfig::default();
            let result = handler.generate_preview(&text_path, &config).await;
            assert!(result.is_err());
            
            match result.unwrap_err() {
                PreviewError::UnsupportedFormat(_) => {}, // Expected
                other => panic!("Expected UnsupportedFormat, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_pdf_outline_fallback() {
        #[cfg(not(feature = "pdf"))]
        {
            let outline = PdfPreviewHandler::extract_document_outline_fallback();
            assert_eq!(outline.len(), 1);
            assert!(outline[0].contains("PDF preview not available"));
        }
    }

    #[cfg(feature = "pdf")]
    #[tokio::test]
    async fn test_pdf_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let pdf_path = temp_dir.path().join("test.pdf");
        
        // Create a minimal valid PDF with basic structure
        let pdf_content = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n3 0 obj<</Type/Page/Parent 2 0 R>>endobj\nxref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000050 00000 n \n0000000100 00000 n \ntrailer<</Size 4/Root 1 0 R>>\nstartxref\n140\n%%EOF";
        fs::write(&pdf_path, pdf_content).unwrap();
        
        let metadata = PdfPreviewHandler::extract_pdf_metadata(&pdf_path).unwrap();
        assert!(metadata.codec.is_some());
        assert!(metadata.codec.unwrap().contains("PDF"));
        assert!(metadata.file_size > 0);
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_hex_string_decoding() {
        let result = PdfPreviewHandler::decode_hex_string("48656C6C6F");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
        
        let invalid_result = PdfPreviewHandler::decode_hex_string("ZZ");
        assert!(invalid_result.is_err());
    }

    #[cfg(feature = "pdf")]
    #[test]
    fn test_pdf_string_extraction() {
        let content = "/Title (My Document) /Author (John Doe)";
        
        let title = PdfPreviewHandler::extract_pdf_string(content, "/Title");
        assert_eq!(title, Some("My Document".to_string()));
        
        let author = PdfPreviewHandler::extract_pdf_string(content, "/Author");
        assert_eq!(author, Some("John Doe".to_string()));
        
        let nonexistent = PdfPreviewHandler::extract_pdf_string(content, "/Subject");
        assert_eq!(nonexistent, None);
    }
}