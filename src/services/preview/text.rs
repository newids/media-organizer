use std::path::Path;
use std::time::SystemTime;
use std::fs;
use async_trait::async_trait;
use crate::services::preview::{
    PreviewHandler, PreviewData, PreviewConfig, PreviewError, 
    SupportedFormat, FileMetadata, PreviewContent
};

/// Text preview handler supporting various text formats with syntax highlighting detection
pub struct TextPreviewHandler {
    _initialized: bool,
}

impl TextPreviewHandler {
    pub fn new() -> Result<Self, PreviewError> {
        Ok(Self {
            _initialized: true,
        })
    }

    /// Detect text encoding from file bytes
    fn detect_encoding(bytes: &[u8]) -> String {
        // Check for BOM (Byte Order Mark) first
        if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
            return "UTF-8 with BOM".to_string();
        }
        
        if bytes.len() >= 2 {
            match &bytes[0..2] {
                b"\xFF\xFE" => return "UTF-16 LE".to_string(),
                b"\xFE\xFF" => return "UTF-16 BE".to_string(),
                _ => {}
            }
        }
        
        if bytes.len() >= 4 && &bytes[0..4] == b"\xFF\xFE\x00\x00" {
            return "UTF-32 LE".to_string();
        }
        
        if bytes.len() >= 4 && &bytes[0..4] == b"\x00\x00\xFE\xFF" {
            return "UTF-32 BE".to_string();
        }
        
        // Use chardet for better encoding detection
        if bytes.len() > 0 {
            let (encoding, confidence, _language) = chardet::detect(bytes);
            if confidence > 0.7 {
                return format!("{} ({:.0}% confidence)", encoding, confidence * 100.0);
            }
        }
        
        // Fallback heuristics
        if std::str::from_utf8(bytes).is_ok() {
            "UTF-8".to_string()
        } else {
            // Check for ASCII
            if bytes.iter().all(|&b| b <= 127) {
                "ASCII".to_string()
            } else {
                // More sophisticated heuristics for common encodings
                if Self::is_likely_latin1(bytes) {
                    "ISO-8859-1 (Latin-1)".to_string()
                } else if Self::is_likely_windows1252(bytes) {
                    "Windows-1252".to_string()
                } else {
                    "Unknown".to_string()
                }
            }
        }
    }
    
    /// Simple heuristic to detect Latin-1 encoding
    fn is_likely_latin1(bytes: &[u8]) -> bool {
        // Latin-1 has printable characters in the 0xA0-0xFF range
        let high_chars = bytes.iter().filter(|&&b| b >= 0xA0).count();
        let total_high_chars = bytes.iter().filter(|&&b| b >= 0x80).count();
        
        // If most high-bit characters are in the Latin-1 printable range, likely Latin-1
        total_high_chars > 0 && (high_chars as f32 / total_high_chars as f32) > 0.8
    }
    
    /// Simple heuristic to detect Windows-1252 encoding
    fn is_likely_windows1252(bytes: &[u8]) -> bool {
        // Windows-1252 has specific characters in 0x80-0x9F range
        let windows_chars = bytes.iter().filter(|&&b| {
            matches!(b, 0x80..=0x9F)
        }).count();
        
        windows_chars > 0
    }

    /// Detect programming language from file extension and content
    fn detect_language(file_path: &Path, content: &str) -> Option<String> {
        // First try file extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let language = match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "py" => "python", 
                "js" => "javascript",
                "ts" => "typescript",
                "cpp" | "cc" | "cxx" | "c++" => "cpp",
                "c" => "c",
                "h" | "hpp" => "c",
                "java" => "java",
                "go" => "go",
                "php" => "php",
                "rb" => "ruby",
                "swift" => "swift",
                "kt" => "kotlin",
                "scala" => "scala",
                "cs" => "csharp",
                "fs" => "fsharp",
                "vb" => "vbnet",
                "sh" | "bash" => "bash",
                "ps1" => "powershell",
                "sql" => "sql",
                "html" | "htm" => "html",
                "css" => "css",
                "scss" | "sass" => "scss",
                "less" => "less",
                "xml" => "xml",
                "json" => "json",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                "ini" | "cfg" | "conf" => "ini",
                "md" | "markdown" => "markdown",
                "tex" => "latex",
                "r" => "r",
                "m" => "matlab",
                "jl" => "julia",
                "dart" => "dart",
                "lua" => "lua",
                "vim" => "vim",
                "dockerfile" => "dockerfile",
                _ => return None,
            };
            return Some(language.to_string());
        }
        
        // Try filename patterns
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            match filename.to_lowercase().as_str() {
                "dockerfile" | "dockerfile.dev" | "dockerfile.prod" => return Some("dockerfile".to_string()),
                "makefile" | "gnumakefile" => return Some("makefile".to_string()),
                "cmakelists.txt" => return Some("cmake".to_string()),
                "requirements.txt" | "setup.py" | "pyproject.toml" => return Some("python".to_string()),
                "package.json" | "tsconfig.json" => return Some("json".to_string()),
                "cargo.toml" | "cargo.lock" => return Some("toml".to_string()),
                ".gitignore" | ".gitattributes" => return Some("gitignore".to_string()),
                _ => {}
            }
        }
        
        // Content-based detection as fallback
        let first_line = content.lines().next().unwrap_or("").trim();
        
        // Shebang detection
        if first_line.starts_with("#!") {
            if first_line.contains("python") {
                return Some("python".to_string());
            } else if first_line.contains("bash") || first_line.contains("sh") {
                return Some("bash".to_string());
            } else if first_line.contains("node") {
                return Some("javascript".to_string());
            }
        }
        
        // XML declaration
        if first_line.starts_with("<?xml") {
            return Some("xml".to_string());
        }
        
        // HTML doctype
        if first_line.to_lowercase().starts_with("<!doctype html") {
            return Some("html".to_string());
        }
        
        // JSON detection
        if (content.trim_start().starts_with('{') && content.trim_end().ends_with('}')) ||
           (content.trim_start().starts_with('[') && content.trim_end().ends_with(']')) {
            // Simple JSON validation
            if content.contains("\"") && (content.contains(':') || content.contains('[')) {
                return Some("json".to_string());
            }
        }
        
        None
    }

    /// Extract preview text with intelligent truncation
    fn extract_preview_text(content: &str, max_length: usize) -> String {
        if content.len() <= max_length {
            return content.to_string();
        }
        
        // Try to break at a natural point (line ending)
        if let Some(pos) = content[..max_length].rfind('\n') {
            if pos > max_length / 2 { // Don't break too early
                return format!("{}...", content[..pos].trim_end());
            }
        }
        
        // Try to break at word boundary
        if let Some(pos) = content[..max_length].rfind(' ') {
            if pos > max_length / 2 {
                return format!("{}...", content[..pos].trim_end());
            }
        }
        
        // Hard truncation as last resort
        format!("{}...", &content[..max_length.saturating_sub(3)])
    }

    /// Extract text metadata
    fn extract_text_metadata(file_path: &Path) -> Result<FileMetadata, PreviewError> {
        let mut metadata = FileMetadata::new();
        
        // Get file system metadata
        let fs_metadata = fs::metadata(file_path)?;
        metadata.file_size = fs_metadata.len();
        metadata.created = fs_metadata.created().ok();
        metadata.modified = fs_metadata.modified().ok();
        
        // Read file to analyze content
        let file_content = fs::read(file_path)
            .map_err(|e| PreviewError::TextError(format!("Failed to read text file: {}", e)))?;
        
        // Detect encoding
        let encoding = Self::detect_encoding(&file_content);
        
        // Convert to string for analysis
        let content = String::from_utf8_lossy(&file_content);
        
        // Count lines
        let line_count = content.lines().count() as u32;
        metadata.page_count = Some(line_count); // Reusing page_count for line count
        
        // Set format info
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            metadata.codec = Some(format!("Text ({})", ext.to_uppercase()));
        } else {
            metadata.codec = Some("Text".to_string());
        }
        
        // Try to extract title from content (first non-empty line for markdown, etc.)
        let first_line = content.lines()
            .find(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string());
            
        if let Some(first_line) = first_line {
            // For markdown, check if first line is a header
            if first_line.starts_with('#') {
                metadata.title = Some(first_line.trim_start_matches('#').trim().to_string());
            } else if first_line.len() < 100 { // Reasonable title length
                metadata.title = Some(first_line);
            }
        }
        
        Ok(metadata)
    }
}

#[async_trait]
impl PreviewHandler for TextPreviewHandler {
    fn supports_format(&self, format: SupportedFormat) -> bool {
        format.is_text()
    }

    async fn generate_preview(&self, file_path: &Path, config: &PreviewConfig) -> Result<PreviewData, PreviewError> {
        // Detect format from extension
        let format = SupportedFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .ok_or_else(|| PreviewError::UnsupportedFormat("No extension".to_string()))?
        )
        .filter(|f| f.is_text())
        .ok_or_else(|| PreviewError::UnsupportedFormat("Not a text file".to_string()))?;

        // Extract metadata
        let metadata = Self::extract_text_metadata(file_path)?;
        
        // Read file content
        let file_content = fs::read(file_path)
            .map_err(|e| PreviewError::TextError(format!("Failed to read text file: {}", e)))?;
        
        // Detect encoding
        let encoding = Self::detect_encoding(&file_content);
        
        // Convert to string
        let content = String::from_utf8_lossy(&file_content);
        
        // Extract preview text
        let preview_text = Self::extract_preview_text(&content, config.max_preview_text_length);
        
        // Detect language for syntax highlighting
        let language = Self::detect_language(file_path, &content);
        
        let preview_content = PreviewContent::Text {
            preview_text,
            language,
            encoding,
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
        Self::extract_text_metadata(file_path)
    }

    async fn generate_thumbnail(&self, file_path: &Path, size: (u32, u32)) -> Result<Vec<u8>, PreviewError> {
        // Generate a text document thumbnail
        use image::{RgbImage, DynamicImage, ImageFormat};
        
        let (width, height) = size;
        let mut img = RgbImage::new(width, height);
        
        // Create a document-like appearance
        let bg_color = image::Rgb([255, 255, 255]); // White background
        let border_color = image::Rgb([200, 200, 200]); // Gray border
        let text_color = image::Rgb([30, 30, 30]); // Dark text
        let highlight_color = image::Rgb([100, 150, 255]); // Blue for syntax highlighting
        
        // Fill background
        for pixel in img.pixels_mut() {
            *pixel = bg_color;
        }
        
        // Draw document border
        let border_width = 1;
        for x in 0..width {
            for y in 0..border_width {
                if y < height { 
                    img.put_pixel(x, y, border_color); 
                }
                if height.saturating_sub(y + 1) < height { 
                    img.put_pixel(x, height.saturating_sub(y + 1), border_color); 
                }
            }
        }
        for y in 0..height {
            for x in 0..border_width {
                if x < width { 
                    img.put_pixel(x, y, border_color); 
                }
                if width.saturating_sub(x + 1) < width { 
                    img.put_pixel(width.saturating_sub(x + 1), y, border_color); 
                }
            }
        }
        
        // Draw text lines
        let line_height = height / 16; // More lines for text documents
        let margin = width / 20;
        
        for line in 0..12 {
            let y_pos = margin + line * line_height;
            if y_pos + 1 < height {
                // Vary line lengths to simulate code/text
                let line_length = match line {
                    0 => width / 3,      // Short first line (like a title)
                    1 => 0,              // Empty line
                    2 => width * 2 / 3,  // Medium line
                    3 => width * 3 / 4,  // Long line
                    4 => width / 2,      // Medium line
                    5 => width * 4 / 5,  // Long line
                    _ => (width * 2 / 3) + ((line * 17) % (width / 6)), // Varied lengths
                };
                
                let line_end = (margin + line_length).min(width.saturating_sub(margin));
                
                // Color some lines differently to simulate syntax highlighting
                let color = if line == 0 || line % 4 == 2 { highlight_color } else { text_color };
                
                for x in margin..line_end {
                    if x < width && y_pos < height {
                        img.put_pixel(x, y_pos, color);
                        if y_pos + 1 < height {
                            img.put_pixel(x, y_pos + 1, color); // Make lines slightly thicker
                        }
                    }
                }
            }
        }
        
        // Add file type indicator based on extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let indicator_color = match ext.to_lowercase().as_str() {
                "rs" => image::Rgb([255, 100, 50]),      // Rust orange
                "py" => image::Rgb([50, 150, 255]),      // Python blue  
                "js" | "ts" => image::Rgb([255, 200, 50]), // JavaScript yellow
                "html" => image::Rgb([255, 100, 100]),   // HTML red
                "css" => image::Rgb([100, 200, 255]),    // CSS blue
                "json" => image::Rgb([150, 255, 150]),   // JSON green
                "md" => image::Rgb([100, 100, 100]),     // Markdown gray
                _ => image::Rgb([100, 100, 100]),        // Default gray
            };
            
            // Draw a small colored square in the top-right corner
            let indicator_size = width / 16;
            let start_x = width.saturating_sub(indicator_size + margin);
            let start_y = margin;
            
            for x in start_x..start_x + indicator_size {
                for y in start_y..start_y + indicator_size {
                    if x < width && y < height {
                        img.put_pixel(x, y, indicator_color);
                    }
                }
            }
        }
        
        // Encode as PNG
        let dynamic_img = DynamicImage::ImageRgb8(img);
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        dynamic_img.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| PreviewError::TextError(format!("Failed to encode text thumbnail: {}", e)))?;
        
        Ok(buffer)
    }
}

impl Default for TextPreviewHandler {
    fn default() -> Self {
        Self::new().expect("TextPreviewHandler should always initialize successfully")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_text_handler_supports_formats() {
        let handler = TextPreviewHandler::new().unwrap();
        assert!(handler.supports_format(SupportedFormat::Text));
        assert!(handler.supports_format(SupportedFormat::Markdown));
        assert!(handler.supports_format(SupportedFormat::Json));
        assert!(handler.supports_format(SupportedFormat::Rust));
        assert!(!handler.supports_format(SupportedFormat::Jpeg));
        assert!(!handler.supports_format(SupportedFormat::Mp4));
        assert!(!handler.supports_format(SupportedFormat::Pdf));
    }

    #[test]
    fn test_encoding_detection() {
        // UTF-8 with BOM
        let utf8_bom = b"\xEF\xBB\xBFHello";
        assert_eq!(TextPreviewHandler::detect_encoding(utf8_bom), "UTF-8 with BOM");
        
        // UTF-16 LE
        let utf16_le = b"\xFF\xFEH\x00e\x00l\x00l\x00o\x00";
        assert_eq!(TextPreviewHandler::detect_encoding(utf16_le), "UTF-16 LE");
        
        // Plain ASCII
        let ascii = b"Hello World";
        assert_eq!(TextPreviewHandler::detect_encoding(ascii), "UTF-8");
        
        // Invalid UTF-8
        let invalid = b"\xFF\xFF\xFF";
        assert_eq!(TextPreviewHandler::detect_encoding(invalid), "Unknown");
    }

    #[test]
    fn test_language_detection() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test Rust file
        let rust_file = temp_dir.path().join("test.rs");
        let rust_content = "fn main() {\n    println!(\"Hello, world!\");\n}";
        
        let lang = TextPreviewHandler::detect_language(&rust_file, rust_content);
        assert_eq!(lang, Some("rust".to_string()));
        
        // Test Python file with shebang
        let python_file = temp_dir.path().join("test.py");
        let python_content = "#!/usr/bin/env python3\nprint('Hello, world!')";
        
        let lang = TextPreviewHandler::detect_language(&python_file, python_content);
        assert_eq!(lang, Some("python".to_string()));
        
        // Test JSON content
        let json_file = temp_dir.path().join("test.json");
        let json_content = r#"{"name": "test", "value": 42}"#;
        
        let lang = TextPreviewHandler::detect_language(&json_file, json_content);
        assert_eq!(lang, Some("json".to_string()));
        
        // Test Dockerfile
        let dockerfile = temp_dir.path().join("Dockerfile");
        let docker_content = "FROM ubuntu:20.04\nRUN apt-get update";
        
        let lang = TextPreviewHandler::detect_language(&dockerfile, docker_content);
        assert_eq!(lang, Some("dockerfile".to_string()));
    }

    #[test]
    fn test_preview_text_extraction() {
        let short_text = "Hello World";
        let result = TextPreviewHandler::extract_preview_text(short_text, 1000);
        assert_eq!(result, "Hello World");
        
        let long_text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let result = TextPreviewHandler::extract_preview_text(long_text, 15);
        assert_eq!(result, "Line 1\nLine 2...");
        
        let no_newlines = "This is a very long line without any newlines to break on";
        let result = TextPreviewHandler::extract_preview_text(no_newlines, 20);
        assert_eq!(result, "This is a very long...");
    }

    #[tokio::test]
    async fn test_text_metadata_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let text_file = temp_dir.path().join("test.txt");
        
        let content = "Title Line\n\nThis is the content\nLine 2\nLine 3";
        fs::write(&text_file, content).unwrap();
        
        let metadata = TextPreviewHandler::extract_text_metadata(&text_file).unwrap();
        assert!(metadata.file_size > 0);
        assert_eq!(metadata.page_count, Some(5)); // 5 lines including empty line
        assert_eq!(metadata.codec, Some("Text (TXT)".to_string()));
        assert_eq!(metadata.title, Some("Title Line".to_string()));
    }

    #[tokio::test]
    async fn test_markdown_title_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let md_file = temp_dir.path().join("test.md");
        
        let content = "# Main Title\n\nThis is markdown content\n## Subtitle";
        fs::write(&md_file, content).unwrap();
        
        let metadata = TextPreviewHandler::extract_text_metadata(&md_file).unwrap();
        assert_eq!(metadata.title, Some("Main Title".to_string()));
        assert_eq!(metadata.codec, Some("Text (MD)".to_string()));
    }

    #[tokio::test]
    async fn test_text_preview_generation() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");
        
        let content = "// This is a Rust file\nfn main() {\n    println!(\"Hello, world!\");\n}";
        fs::write(&rust_file, content).unwrap();
        
        let handler = TextPreviewHandler::new().unwrap();
        let config = PreviewConfig::default();
        
        let preview = handler.generate_preview(&rust_file, &config).await.unwrap();
        
        assert_eq!(preview.format, SupportedFormat::Rust);
        assert!(preview.file_path.ends_with("test.rs"));
        
        match preview.preview_content {
            PreviewContent::Text { preview_text, language, encoding } => {
                assert!(preview_text.contains("This is a Rust file"));
                assert!(preview_text.contains("fn main()"));
                assert_eq!(language, Some("rust".to_string()));
                assert_eq!(encoding, "UTF-8");
            }
            _ => panic!("Expected Text preview content"),
        }
    }

    #[tokio::test]
    async fn test_thumbnail_generation() {
        let temp_dir = TempDir::new().unwrap();
        let text_file = temp_dir.path().join("test.txt");
        fs::write(&text_file, "Sample text content").unwrap();
        
        let handler = TextPreviewHandler::new().unwrap();
        let result = handler.generate_thumbnail(&text_file, (256, 256)).await;
        
        assert!(result.is_ok());
        let thumbnail_data = result.unwrap();
        assert!(!thumbnail_data.is_empty());
    }

    #[tokio::test]
    async fn test_unsupported_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let image_file = temp_dir.path().join("test.jpg");
        fs::write(&image_file, b"fake image data").unwrap();
        
        let handler = TextPreviewHandler::new().unwrap();
        let config = PreviewConfig::default();
        let result = handler.generate_preview(&image_file, &config).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            PreviewError::UnsupportedFormat(_) => {}, // Expected
            other => panic!("Expected UnsupportedFormat, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_json_content_detection() {
        let temp_dir = TempDir::new().unwrap();
        let json_file = temp_dir.path().join("data.json");
        
        let json_content = r#"{
    "name": "MediaOrganizer",
    "version": "0.1.0",
    "features": ["preview", "cache"]
}"#;
        fs::write(&json_file, json_content).unwrap();
        
        let handler = TextPreviewHandler::new().unwrap();
        let config = PreviewConfig::default();
        let preview = handler.generate_preview(&json_file, &config).await.unwrap();
        
        match preview.preview_content {
            PreviewContent::Text { language, .. } => {
                assert_eq!(language, Some("json".to_string()));
            }
            _ => panic!("Expected Text preview content"),
        }
    }
}