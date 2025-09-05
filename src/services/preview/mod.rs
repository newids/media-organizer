pub mod core;
pub mod image;
pub mod video;
pub mod audio;
pub mod pdf;
pub mod text;
pub mod archive;
pub mod fallback;
pub mod thumbnail_service;
pub mod metadata_display;
pub mod integration_tests;

pub use core::*;
pub use image::{ImagePreviewProvider, ImagePreviewHandler};
pub use video::{VideoPreviewProvider, VideoPreviewHandler};
pub use audio::{AudioPreviewProvider, AudioPreviewHandler};
pub use pdf::{PdfPreviewProvider, PdfPreviewHandler};
pub use text::{TextPreviewProvider, TextPreviewHandler};
pub use archive::{ArchivePreviewProvider, ArchivePreviewHandler};
pub use fallback::{FallbackPreviewProvider, FallbackPreviewHandler};
// pub use thumbnail_service::{
//     ThumbnailService, ThumbnailPriority, ThumbnailJobStatus, ThumbnailJobConfig, 
//     ThumbnailJob, ThumbnailServiceStats
// };
// pub use metadata_display::{
//     MetadataDisplay, BasicInfoSection, TechnicalInfoSection, ContentInfoSection,
//     ExifInfoSection, TimestampInfoSection
// };