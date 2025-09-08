//! Utility functions for MediaOrganizer
//! 
//! This module contains various utility functions used throughout the application,
//! including path normalization, string processing, file type detection, and cross-platform helpers.

pub mod path_utils;
pub mod file_type_detection;

// Re-export commonly used utilities
pub use path_utils::{normalize_path_display, normalize_path_string, path_to_element_id};
pub use file_type_detection::{FileTypeDetectionUtil, FilePreviewSupport, PreviewCategory, InfoCategory};