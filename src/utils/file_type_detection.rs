use std::path::Path;
use crate::services::file_system::{FileType, ImageFormat, VideoFormat, AudioFormat, DocumentFormat, TextFormat};

/// Result of file type detection for panel switching
#[derive(Debug, Clone, PartialEq)]
pub enum FilePreviewSupport {
    /// File has full preview support and should show in preview panel
    PreviewSupported(PreviewCategory),
    /// File has limited/metadata-only support and should show info panel
    InfoOnly(InfoCategory),
    /// File has no support and should show basic metadata
    Unsupported,
}

/// Categories of files that support full preview
#[derive(Debug, Clone, PartialEq)]
pub enum PreviewCategory {
    /// Images that can be displayed directly
    Image,
    /// Videos that can be played with controls
    Video,
    /// Audio files that can be played with waveform
    Audio,
    /// Text files that can be syntax-highlighted
    Text,
    /// Documents that can be rendered (PDFs, etc.)
    Document,
}

/// Categories of files that show info panels instead of previews
#[derive(Debug, Clone, PartialEq)]
pub enum InfoCategory {
    /// Large documents that are better shown as metadata
    LargeDocument,
    /// Binary files that should show technical details
    Binary,
    /// Compressed archives showing contents list
    Archive,
    /// Unknown file types showing basic properties
    Unknown,
}

/// File type detection utility for dynamic panel switching
pub struct FileTypeDetectionUtil;

impl FileTypeDetectionUtil {
    /// Determines preview support for a given file based on its type and characteristics
    pub fn detect_preview_support(file_type: &FileType, file_path: &Path) -> FilePreviewSupport {
        match file_type {
            FileType::Image(format) => {
                if Self::is_supported_image_format(format) {
                    FilePreviewSupport::PreviewSupported(PreviewCategory::Image)
                } else {
                    FilePreviewSupport::InfoOnly(InfoCategory::Unknown)
                }
            }
            
            FileType::Video(format) => {
                if Self::is_supported_video_format(format) {
                    FilePreviewSupport::PreviewSupported(PreviewCategory::Video)
                } else {
                    FilePreviewSupport::InfoOnly(InfoCategory::Unknown)
                }
            }
            
            FileType::Audio(format) => {
                if Self::is_supported_audio_format(format) {
                    FilePreviewSupport::PreviewSupported(PreviewCategory::Audio)
                } else {
                    FilePreviewSupport::InfoOnly(InfoCategory::Unknown)
                }
            }
            
            FileType::Document(format) => {
                match format {
                    DocumentFormat::Pdf => {
                        // Check file size for PDF - large PDFs might be better as info panels
                        if let Ok(metadata) = std::fs::metadata(file_path) {
                            let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                            if size_mb > 50.0 {
                                FilePreviewSupport::InfoOnly(InfoCategory::LargeDocument)
                            } else {
                                FilePreviewSupport::PreviewSupported(PreviewCategory::Document)
                            }
                        } else {
                            FilePreviewSupport::PreviewSupported(PreviewCategory::Document)
                        }
                    }
                    DocumentFormat::Txt | DocumentFormat::Md => {
                        FilePreviewSupport::PreviewSupported(PreviewCategory::Text)
                    }
                    // Office documents are complex to preview, show info instead
                    DocumentFormat::Docx | DocumentFormat::Doc | 
                    DocumentFormat::Xlsx | DocumentFormat::Xls |
                    DocumentFormat::Pptx | DocumentFormat::Ppt => {
                        FilePreviewSupport::InfoOnly(InfoCategory::LargeDocument)
                    }
                }
            }
            
            FileType::Text(format) => {
                if Self::is_supported_text_format(format) {
                    FilePreviewSupport::PreviewSupported(PreviewCategory::Text)
                } else {
                    FilePreviewSupport::InfoOnly(InfoCategory::Unknown)
                }
            }
            
            FileType::Directory => {
                // Directories always show info panel with contents
                FilePreviewSupport::InfoOnly(InfoCategory::Unknown)
            }
            
            FileType::Other(extension) => {
                // Check for common archive extensions
                if Self::is_archive_extension(extension) {
                    FilePreviewSupport::InfoOnly(InfoCategory::Archive)
                } else if Self::is_binary_extension(extension) {
                    FilePreviewSupport::InfoOnly(InfoCategory::Binary)
                } else {
                    FilePreviewSupport::Unsupported
                }
            }
        }
    }

    /// Quick check if a file path supports preview without detailed analysis
    pub fn supports_preview(file_path: &Path) -> bool {
        let file_type = FileType::from_path(file_path);
        match Self::detect_preview_support(&file_type, file_path) {
            FilePreviewSupport::PreviewSupported(_) => true,
            _ => false,
        }
    }

    /// Get a user-friendly description of what will be shown for this file
    pub fn get_panel_description(support: &FilePreviewSupport) -> &'static str {
        match support {
            FilePreviewSupport::PreviewSupported(category) => {
                match category {
                    PreviewCategory::Image => "Image preview with zoom controls",
                    PreviewCategory::Video => "Video player with controls",
                    PreviewCategory::Audio => "Audio player with waveform",
                    PreviewCategory::Text => "Syntax-highlighted text preview",
                    PreviewCategory::Document => "Document preview",
                }
            }
            FilePreviewSupport::InfoOnly(category) => {
                match category {
                    InfoCategory::LargeDocument => "Document metadata and properties",
                    InfoCategory::Binary => "Binary file technical details",
                    InfoCategory::Archive => "Archive contents listing",
                    InfoCategory::Unknown => "File properties and metadata",
                }
            }
            FilePreviewSupport::Unsupported => "Basic file information",
        }
    }

    // Private helper methods for format support detection
    
    fn is_supported_image_format(format: &ImageFormat) -> bool {
        matches!(format, 
            ImageFormat::Jpeg | 
            ImageFormat::Png | 
            ImageFormat::Gif | 
            ImageFormat::WebP | 
            ImageFormat::Bmp |
            ImageFormat::Svg
        )
    }

    fn is_supported_video_format(format: &VideoFormat) -> bool {
        matches!(format,
            VideoFormat::Mp4 |
            VideoFormat::WebM |
            VideoFormat::Mov
        )
    }

    fn is_supported_audio_format(format: &AudioFormat) -> bool {
        matches!(format,
            AudioFormat::Mp3 |
            AudioFormat::Wav |
            AudioFormat::Ogg
        )
    }

    fn is_supported_text_format(format: &TextFormat) -> bool {
        matches!(format,
            TextFormat::Json |
            TextFormat::Xml |
            TextFormat::Html |
            TextFormat::Css |
            TextFormat::JavaScript |
            TextFormat::Rust |
            TextFormat::Python
        )
    }

    fn is_archive_extension(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(),
            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz"
        )
    }

    fn is_binary_extension(extension: &str) -> bool {
        matches!(extension.to_lowercase().as_str(),
            "exe" | "dll" | "so" | "dylib" | "bin" | "app"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_image_preview_support() {
        let path = PathBuf::from("test.jpg");
        let file_type = FileType::Image(ImageFormat::Jpeg);
        let support = FileTypeDetectionUtil::detect_preview_support(&file_type, &path);
        assert_eq!(support, FilePreviewSupport::PreviewSupported(PreviewCategory::Image));
    }

    #[test]
    fn test_video_preview_support() {
        let path = PathBuf::from("test.mp4");
        let file_type = FileType::Video(VideoFormat::Mp4);
        let support = FileTypeDetectionUtil::detect_preview_support(&file_type, &path);
        assert_eq!(support, FilePreviewSupport::PreviewSupported(PreviewCategory::Video));
    }

    #[test]
    fn test_office_document_info_only() {
        let path = PathBuf::from("document.docx");
        let file_type = FileType::Document(DocumentFormat::Docx);
        let support = FileTypeDetectionUtil::detect_preview_support(&file_type, &path);
        assert_eq!(support, FilePreviewSupport::InfoOnly(InfoCategory::LargeDocument));
    }

    #[test]
    fn test_archive_info_only() {
        let path = PathBuf::from("archive.zip");
        let file_type = FileType::Other("zip".to_string());
        let support = FileTypeDetectionUtil::detect_preview_support(&file_type, &path);
        assert_eq!(support, FilePreviewSupport::InfoOnly(InfoCategory::Archive));
    }

    #[test]
    fn test_supports_preview_quick_check() {
        assert_eq!(FileTypeDetectionUtil::supports_preview(&PathBuf::from("image.jpg")), true);
        assert_eq!(FileTypeDetectionUtil::supports_preview(&PathBuf::from("document.docx")), false);
        assert_eq!(FileTypeDetectionUtil::supports_preview(&PathBuf::from("video.mp4")), true);
    }

    #[test]
    fn test_panel_descriptions() {
        let image_support = FilePreviewSupport::PreviewSupported(PreviewCategory::Image);
        assert_eq!(FileTypeDetectionUtil::get_panel_description(&image_support), 
                  "Image preview with zoom controls");

        let info_support = FilePreviewSupport::InfoOnly(InfoCategory::LargeDocument);
        assert_eq!(FileTypeDetectionUtil::get_panel_description(&info_support), 
                  "Document metadata and properties");
    }
}