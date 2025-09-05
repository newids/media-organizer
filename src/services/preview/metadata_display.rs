use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use crate::services::preview::{
    FileMetadata, ExifData, SupportedFormat, PreviewContent
};

/// Unified metadata display interface for all file types
/// Provides formatted and categorized metadata information for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDisplay {
    /// Basic file information
    pub basic_info: BasicInfoSection,
    /// Technical metadata
    pub technical_info: TechnicalInfoSection,
    /// Content-specific metadata
    pub content_info: ContentInfoSection,
    /// EXIF data for images
    pub exif_info: Option<ExifInfoSection>,
    /// Creation and modification timestamps
    pub timestamp_info: TimestampInfoSection,
    /// Custom key-value pairs for extensibility
    pub additional_info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicInfoSection {
    pub file_name: String,
    pub file_path: String,
    pub file_size: String, // Human-readable format (e.g., "1.5 MB")
    pub file_type: String, // Human-readable format (e.g., "JPEG Image")
    pub format: SupportedFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalInfoSection {
    pub codec: Option<String>,
    pub dimensions: Option<String>, // "1920x1080" or "N/A"
    pub duration: Option<String>, // "2m 30s" or "N/A"
    pub sample_rate: Option<String>, // "44.1 kHz" or "N/A"
    pub bit_rate: Option<String>, // "128 kbps" or "N/A"
    pub color_space: Option<String>,
    pub compression: Option<String>,
    pub page_count: Option<String>, // "5 pages" or "150 lines"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInfoSection {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub language: Option<String>, // For text files
    pub encoding: Option<String>, // For text files
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifInfoSection {
    pub camera_info: Option<String>, // "Canon EOS 5D Mark IV"
    pub lens_info: Option<String>, // "Canon EF 24-70mm f/2.8L II USM"
    pub photo_settings: Vec<(String, String)>, // [("Aperture", "f/2.8"), ("Shutter Speed", "1/250s")]
    pub location: Option<String>, // "37.7749, -122.4194" or address if available
    pub date_taken: Option<String>, // "June 15, 2023 2:30 PM"
    pub orientation: Option<String>, // "Landscape" or "Portrait"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampInfoSection {
    pub created: Option<String>, // "June 15, 2023 2:30 PM"
    pub modified: Option<String>, // "June 16, 2023 10:45 AM"
    pub accessed: Option<String>, // If available from filesystem
}

impl MetadataDisplay {
    /// Create a unified metadata display from file metadata and preview content
    pub fn from_metadata(
        file_path: &std::path::Path,
        metadata: &FileMetadata,
        format: SupportedFormat,
        preview_content: Option<&PreviewContent>,
    ) -> Self {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let file_path_str = file_path.display().to_string();

        Self {
            basic_info: BasicInfoSection {
                file_name,
                file_path: file_path_str,
                file_size: Self::format_file_size(metadata.file_size),
                file_type: Self::format_file_type(format),
                format,
            },
            technical_info: TechnicalInfoSection {
                codec: metadata.codec.clone(),
                dimensions: Self::format_dimensions(metadata.width, metadata.height),
                duration: Self::format_duration(metadata.duration),
                sample_rate: Self::format_sample_rate(metadata.sample_rate),
                bit_rate: Self::format_bit_rate(metadata.bit_rate),
                color_space: metadata.color_space.clone(),
                compression: metadata.compression.clone(),
                page_count: Self::format_page_count(metadata.page_count, format),
            },
            content_info: ContentInfoSection {
                title: metadata.title.clone(),
                artist: metadata.artist.clone(),
                album: metadata.album.clone(),
                year: metadata.year.map(|y| y.to_string()),
                language: Self::extract_language_from_content(preview_content),
                encoding: Self::extract_encoding_from_content(preview_content),
            },
            exif_info: Self::format_exif_info(&metadata.exif_data),
            timestamp_info: TimestampInfoSection {
                created: Self::format_timestamp(metadata.created),
                modified: Self::format_timestamp(metadata.modified),
                accessed: None, // Not commonly available
            },
            additional_info: HashMap::new(),
        }
    }

    /// Add custom metadata information
    pub fn add_custom_info(&mut self, key: String, value: String) {
        self.additional_info.insert(key, value);
    }

    /// Get a flat list of all metadata for simple display
    pub fn to_flat_list(&self) -> Vec<(String, String)> {
        let mut items = Vec::new();

        // Basic Info
        items.push(("File Name".to_string(), self.basic_info.file_name.clone()));
        items.push(("File Size".to_string(), self.basic_info.file_size.clone()));
        items.push(("File Type".to_string(), self.basic_info.file_type.clone()));

        // Technical Info
        if let Some(codec) = &self.technical_info.codec {
            items.push(("Codec".to_string(), codec.clone()));
        }
        if let Some(dimensions) = &self.technical_info.dimensions {
            items.push(("Dimensions".to_string(), dimensions.clone()));
        }
        if let Some(duration) = &self.technical_info.duration {
            items.push(("Duration".to_string(), duration.clone()));
        }
        if let Some(sample_rate) = &self.technical_info.sample_rate {
            items.push(("Sample Rate".to_string(), sample_rate.clone()));
        }
        if let Some(bit_rate) = &self.technical_info.bit_rate {
            items.push(("Bit Rate".to_string(), bit_rate.clone()));
        }
        if let Some(color_space) = &self.technical_info.color_space {
            items.push(("Color Space".to_string(), color_space.clone()));
        }
        if let Some(page_count) = &self.technical_info.page_count {
            items.push(("Pages/Lines".to_string(), page_count.clone()));
        }

        // Content Info
        if let Some(title) = &self.content_info.title {
            items.push(("Title".to_string(), title.clone()));
        }
        if let Some(artist) = &self.content_info.artist {
            items.push(("Artist/Author".to_string(), artist.clone()));
        }
        if let Some(album) = &self.content_info.album {
            items.push(("Album/Collection".to_string(), album.clone()));
        }
        if let Some(year) = &self.content_info.year {
            items.push(("Year".to_string(), year.clone()));
        }
        if let Some(language) = &self.content_info.language {
            items.push(("Language".to_string(), language.clone()));
        }
        if let Some(encoding) = &self.content_info.encoding {
            items.push(("Encoding".to_string(), encoding.clone()));
        }

        // EXIF Info
        if let Some(exif) = &self.exif_info {
            if let Some(camera) = &exif.camera_info {
                items.push(("Camera".to_string(), camera.clone()));
            }
            if let Some(lens) = &exif.lens_info {
                items.push(("Lens".to_string(), lens.clone()));
            }
            for (key, value) in &exif.photo_settings {
                items.push((key.clone(), value.clone()));
            }
            if let Some(location) = &exif.location {
                items.push(("GPS Location".to_string(), location.clone()));
            }
            if let Some(date_taken) = &exif.date_taken {
                items.push(("Date Taken".to_string(), date_taken.clone()));
            }
        }

        // Timestamps
        if let Some(created) = &self.timestamp_info.created {
            items.push(("Created".to_string(), created.clone()));
        }
        if let Some(modified) = &self.timestamp_info.modified {
            items.push(("Modified".to_string(), modified.clone()));
        }

        // Additional custom info
        for (key, value) in &self.additional_info {
            items.push((key.clone(), value.clone()));
        }

        items
    }

    /// Format file size in human-readable format
    fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;

        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size_f, UNITS[unit_index])
        }
    }

    /// Format file type description
    fn format_file_type(format: SupportedFormat) -> String {
        match format {
            SupportedFormat::Jpeg => "JPEG Image".to_string(),
            SupportedFormat::Png => "PNG Image".to_string(),
            SupportedFormat::Gif => "GIF Image".to_string(),
            SupportedFormat::WebP => "WebP Image".to_string(),
            SupportedFormat::Tiff => "TIFF Image".to_string(),
            SupportedFormat::Bmp => "BMP Image".to_string(),
            SupportedFormat::Svg => "SVG Image".to_string(),
            SupportedFormat::Mp4 => "MP4 Video".to_string(),
            SupportedFormat::Avi => "AVI Video".to_string(),
            SupportedFormat::Mkv => "MKV Video".to_string(),
            SupportedFormat::Mov => "QuickTime Video".to_string(),
            SupportedFormat::Wmv => "WMV Video".to_string(),
            SupportedFormat::Flv => "FLV Video".to_string(),
            SupportedFormat::WebM => "WebM Video".to_string(),
            SupportedFormat::Mp3 => "MP3 Audio".to_string(),
            SupportedFormat::Wav => "WAV Audio".to_string(),
            SupportedFormat::Flac => "FLAC Audio".to_string(),
            SupportedFormat::Aac => "AAC Audio".to_string(),
            SupportedFormat::Ogg => "OGG Audio".to_string(),
            SupportedFormat::M4a => "M4A Audio".to_string(),
            SupportedFormat::Pdf => "PDF Document".to_string(),
            SupportedFormat::Text => "Plain Text".to_string(),
            SupportedFormat::Markdown => "Markdown Document".to_string(),
            SupportedFormat::Json => "JSON Data".to_string(),
            SupportedFormat::Xml => "XML Document".to_string(),
            SupportedFormat::Html => "HTML Document".to_string(),
            SupportedFormat::Css => "CSS Stylesheet".to_string(),
            SupportedFormat::Javascript => "JavaScript Code".to_string(),
            SupportedFormat::Rust => "Rust Source Code".to_string(),
            SupportedFormat::Python => "Python Source Code".to_string(),
            SupportedFormat::Cpp => "C++ Source Code".to_string(),
            SupportedFormat::Java => "Java Source Code".to_string(),
            // Archive formats
            SupportedFormat::Zip => "ZIP Archive".to_string(),
            SupportedFormat::Tar => "TAR Archive".to_string(),
            SupportedFormat::Gz => "GZIP Archive".to_string(),
            SupportedFormat::SevenZip => "7-Zip Archive".to_string(),
            SupportedFormat::Rar => "RAR Archive".to_string(),
        }
    }

    /// Format dimensions as a readable string
    fn format_dimensions(width: Option<u32>, height: Option<u32>) -> Option<String> {
        match (width, height) {
            (Some(w), Some(h)) => Some(format!("{} × {}", w, h)),
            (Some(w), None) => Some(format!("{} px wide", w)),
            (None, Some(h)) => Some(format!("{} px tall", h)),
            (None, None) => None,
        }
    }

    /// Format duration as a readable string
    fn format_duration(duration: Option<f64>) -> Option<String> {
        duration.map(|dur| {
            if dur < 60.0 {
                format!("{:.1}s", dur)
            } else if dur < 3600.0 {
                let minutes = dur as u32 / 60;
                let seconds = dur as u32 % 60;
                format!("{}m {}s", minutes, seconds)
            } else {
                let hours = dur as u32 / 3600;
                let minutes = (dur as u32 % 3600) / 60;
                let seconds = dur as u32 % 60;
                format!("{}h {}m {}s", hours, minutes, seconds)
            }
        })
    }

    /// Format sample rate as a readable string
    fn format_sample_rate(sample_rate: Option<u32>) -> Option<String> {
        sample_rate.map(|rate| {
            if rate % 1000 == 0 {
                format!("{} kHz", rate / 1000)
            } else {
                format!("{} Hz", rate)
            }
        })
    }

    /// Format bit rate as a readable string
    fn format_bit_rate(bit_rate: Option<u32>) -> Option<String> {
        bit_rate.map(|rate| {
            if rate >= 1_000_000 {
                format!("{:.1} Mbps", rate as f64 / 1_000_000.0)
            } else if rate >= 1_000 {
                format!("{} kbps", rate / 1_000)
            } else {
                format!("{} bps", rate)
            }
        })
    }

    /// Format page count based on file type
    fn format_page_count(page_count: Option<u32>, format: SupportedFormat) -> Option<String> {
        page_count.map(|count| {
            match format {
                _ if format.is_document() => {
                    if count == 1 {
                        "1 page".to_string()
                    } else {
                        format!("{} pages", count)
                    }
                }
                _ if format.is_text() => {
                    if count == 1 {
                        "1 line".to_string()
                    } else {
                        format!("{} lines", count)
                    }
                }
                _ => count.to_string(),
            }
        })
    }

    /// Format timestamp as a readable string
    fn format_timestamp(timestamp: Option<SystemTime>) -> Option<String> {
        timestamp.map(|time| {
            use chrono::{DateTime, Utc};
            
            let datetime: DateTime<Utc> = time.into();
            datetime.format("%B %d, %Y at %I:%M %p").to_string()
        })
    }

    /// Extract language information from preview content
    fn extract_language_from_content(content: Option<&PreviewContent>) -> Option<String> {
        match content {
            Some(PreviewContent::Text { language, .. }) => language.clone(),
            _ => None,
        }
    }

    /// Extract encoding information from preview content
    fn extract_encoding_from_content(content: Option<&PreviewContent>) -> Option<String> {
        match content {
            Some(PreviewContent::Text { .. }) => Some("UTF-8".to_string()),
            _ => None,
        }
    }

    /// Format EXIF information for display
    fn format_exif_info(exif_data: &Option<ExifData>) -> Option<ExifInfoSection> {
        exif_data.as_ref().map(|exif| {
            let camera_info = match (&exif.camera_make, &exif.camera_model) {
                (Some(make), Some(model)) => Some(format!("{} {}", make, model)),
                (Some(make), None) => Some(make.clone()),
                (None, Some(model)) => Some(model.clone()),
                (None, None) => None,
            };

            let mut photo_settings = Vec::new();
            
            if let Some(aperture) = exif.aperture {
                photo_settings.push(("Aperture".to_string(), format!("f/{:.1}", aperture)));
            }
            
            if let Some(shutter_speed) = &exif.shutter_speed {
                photo_settings.push(("Shutter Speed".to_string(), shutter_speed.clone()));
            }
            
            if let Some(iso) = exif.iso {
                photo_settings.push(("ISO".to_string(), format!("ISO {}", iso)));
            }
            
            if let Some(focal_length) = exif.focal_length {
                photo_settings.push(("Focal Length".to_string(), format!("{:.0}mm", focal_length)));
            }
            
            if let Some(flash) = exif.flash {
                photo_settings.push((
                    "Flash".to_string(),
                    if flash { "Fired".to_string() } else { "Did not fire".to_string() }
                ));
            }

            let location = match (exif.gps_latitude, exif.gps_longitude) {
                (Some(lat), Some(lon)) => Some(format!("{:.6}, {:.6}", lat, lon)),
                _ => None,
            };

            let date_taken = Self::format_timestamp(exif.date_taken);

            let orientation = exif.orientation.map(|o| {
                match o {
                    1 => "Normal".to_string(),
                    2 => "Flipped horizontally".to_string(),
                    3 => "Rotated 180°".to_string(),
                    4 => "Flipped vertically".to_string(),
                    5 => "Rotated 90° CCW and flipped".to_string(),
                    6 => "Rotated 90° CW".to_string(),
                    7 => "Rotated 90° CW and flipped".to_string(),
                    8 => "Rotated 90° CCW".to_string(),
                    _ => format!("Orientation {}", o),
                }
            });

            ExifInfoSection {
                camera_info,
                lens_info: exif.lens_model.clone(),
                photo_settings,
                location,
                date_taken,
                orientation,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(MetadataDisplay::format_file_size(512), "512 B");
        assert_eq!(MetadataDisplay::format_file_size(1024), "1.0 KB");
        assert_eq!(MetadataDisplay::format_file_size(1536), "1.5 KB");
        assert_eq!(MetadataDisplay::format_file_size(1048576), "1.0 MB");
        assert_eq!(MetadataDisplay::format_file_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_duration_formatting() {
        assert_eq!(MetadataDisplay::format_duration(Some(30.5)), Some("30.5s".to_string()));
        assert_eq!(MetadataDisplay::format_duration(Some(90.0)), Some("1m 30s".to_string()));
        assert_eq!(MetadataDisplay::format_duration(Some(3661.0)), Some("1h 1m 1s".to_string()));
        assert_eq!(MetadataDisplay::format_duration(None), None);
    }

    #[test]
    fn test_dimensions_formatting() {
        assert_eq!(
            MetadataDisplay::format_dimensions(Some(1920), Some(1080)),
            Some("1920 × 1080".to_string())
        );
        assert_eq!(
            MetadataDisplay::format_dimensions(Some(800), None),
            Some("800 px wide".to_string())
        );
        assert_eq!(
            MetadataDisplay::format_dimensions(None, Some(600)),
            Some("600 px tall".to_string())
        );
        assert_eq!(MetadataDisplay::format_dimensions(None, None), None);
    }

    #[test]
    fn test_bit_rate_formatting() {
        assert_eq!(MetadataDisplay::format_bit_rate(Some(128000)), Some("128 kbps".to_string()));
        assert_eq!(MetadataDisplay::format_bit_rate(Some(1500000)), Some("1.5 Mbps".to_string()));
        assert_eq!(MetadataDisplay::format_bit_rate(Some(64000)), Some("64 kbps".to_string()));
        assert_eq!(MetadataDisplay::format_bit_rate(None), None);
    }

    #[test]
    fn test_sample_rate_formatting() {
        assert_eq!(MetadataDisplay::format_sample_rate(Some(44100)), Some("44100 Hz".to_string()));
        assert_eq!(MetadataDisplay::format_sample_rate(Some(48000)), Some("48 kHz".to_string()));
        assert_eq!(MetadataDisplay::format_sample_rate(Some(22050)), Some("22050 Hz".to_string()));
        assert_eq!(MetadataDisplay::format_sample_rate(None), None);
    }

    #[test]
    fn test_page_count_formatting() {
        assert_eq!(
            MetadataDisplay::format_page_count(Some(1), SupportedFormat::Pdf),
            Some("1 page".to_string())
        );
        assert_eq!(
            MetadataDisplay::format_page_count(Some(5), SupportedFormat::Pdf),
            Some("5 pages".to_string())
        );
        assert_eq!(
            MetadataDisplay::format_page_count(Some(1), SupportedFormat::Text),
            Some("1 line".to_string())
        );
        assert_eq!(
            MetadataDisplay::format_page_count(Some(150), SupportedFormat::Text),
            Some("150 lines".to_string())
        );
    }

    #[test]
    fn test_file_type_formatting() {
        assert_eq!(MetadataDisplay::format_file_type(SupportedFormat::Jpeg), "JPEG Image");
        assert_eq!(MetadataDisplay::format_file_type(SupportedFormat::Mp4), "MP4 Video");
        assert_eq!(MetadataDisplay::format_file_type(SupportedFormat::Mp3), "MP3 Audio");
        assert_eq!(MetadataDisplay::format_file_type(SupportedFormat::Pdf), "PDF Document");
        assert_eq!(MetadataDisplay::format_file_type(SupportedFormat::Rust), "Rust Source Code");
    }

    #[test]
    fn test_metadata_display_creation() {
        let mut metadata = FileMetadata::new();
        metadata.file_size = 1024;
        metadata.width = Some(1920);
        metadata.height = Some(1080);
        metadata.title = Some("Test Image".to_string());
        metadata.codec = Some("JPEG".to_string());

        let file_path = PathBuf::from("/test/image.jpg");
        let display = MetadataDisplay::from_metadata(
            &file_path,
            &metadata,
            SupportedFormat::Jpeg,
            None,
        );

        assert_eq!(display.basic_info.file_name, "image.jpg");
        assert_eq!(display.basic_info.file_size, "1.0 KB");
        assert_eq!(display.basic_info.file_type, "JPEG Image");
        assert_eq!(display.technical_info.dimensions, Some("1920 × 1080".to_string()));
        assert_eq!(display.content_info.title, Some("Test Image".to_string()));
    }

    #[test]
    fn test_flat_list_generation() {
        let mut metadata = FileMetadata::new();
        metadata.file_size = 2048;
        metadata.title = Some("Test Document".to_string());
        metadata.artist = Some("Test Author".to_string());

        let file_path = PathBuf::from("/test/document.pdf");
        let display = MetadataDisplay::from_metadata(
            &file_path,
            &metadata,
            SupportedFormat::Pdf,
            None,
        );

        let flat_list = display.to_flat_list();
        
        // Should contain basic file info
        assert!(flat_list.iter().any(|(k, v)| k == "File Name" && v == "document.pdf"));
        assert!(flat_list.iter().any(|(k, v)| k == "File Size" && v == "2.0 KB"));
        assert!(flat_list.iter().any(|(k, v)| k == "File Type" && v == "PDF Document"));
        
        // Should contain content info
        assert!(flat_list.iter().any(|(k, v)| k == "Title" && v == "Test Document"));
        assert!(flat_list.iter().any(|(k, v)| k == "Artist/Author" && v == "Test Author"));
    }
}