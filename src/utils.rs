//! Utility functions for MediaOrganizer
//! 
//! This module contains various utility functions used throughout the application,
//! including path normalization, string processing, and cross-platform helpers.

pub mod path_utils;

// Re-export commonly used utilities
pub use path_utils::{normalize_path_display, normalize_path_string, path_to_element_id};