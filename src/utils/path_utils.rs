//! Path normalization utilities for cross-platform path display
//! 
//! This module provides utilities to clean up path displays by removing redundant
//! slashes and ensuring consistent path formatting across different platforms.

use std::path::{Path, PathBuf};

/// Normalize a path string by removing redundant slashes while preserving important path semantics
///
/// This function handles:
/// - Double forward slashes (//) -> single slash (/)
/// - Double backslashes (\\) -> single backslash (\)
/// - Mixed slash types (preserves the predominant type)
/// - Network paths (preserves UNC paths like //server/share)
/// - Root directories (preserves leading slashes)
///
/// # Examples
/// ```
/// use media_organizer::utils::normalize_path_string;
/// 
/// assert_eq!(normalize_path_string("folder//file.txt"), "folder/file.txt");
/// assert_eq!(normalize_path_string("folder\\\\file.txt"), "folder\\file.txt");
/// assert_eq!(normalize_path_string("//server/share"), "//server/share"); // UNC preserved
/// assert_eq!(normalize_path_string("/root//nested/file"), "/root/nested/file");
/// ```
pub fn normalize_path_string(path: &str) -> String {
    if path.is_empty() {
        return path.to_string();
    }
    
    // Handle UNC paths (network paths starting with //) - preserve the double slash at start
    let is_unc = path.starts_with("//") && !path.starts_with("///");
    
    // Determine predominant slash type to maintain consistency
    let forward_count = path.matches('/').count();
    let back_count = path.matches('\\').count();
    let use_forward_slash = forward_count >= back_count;
    
    let mut normalized = path.to_string();
    
    if use_forward_slash {
        // Normalize forward slashes - replace multiple with single
        normalized = regex::Regex::new(r"/+").unwrap().replace_all(&normalized, "/").to_string();
        
        // Also normalize any backslashes to forward slashes for consistency
        normalized = normalized.replace('\\', "/");
        
        // Handle UNC path case - restore the leading double slash
        if is_unc && !normalized.starts_with("//") {
            normalized = format!("/{}", normalized);
        }
    } else {
        // Normalize backslashes - replace multiple with single
        normalized = regex::Regex::new(r"\\+").unwrap().replace_all(&normalized, "\\").to_string();
        
        // Also normalize any forward slashes to backslashes for consistency
        normalized = normalized.replace('/', "\\");
    }
    
    normalized
}

/// Normalize a Path for display purposes
///
/// This function takes a Path reference and returns a normalized string representation
/// suitable for display in UI components. It handles cross-platform path differences
/// and removes redundant path separators.
///
/// # Examples
/// ```
/// use std::path::Path;
/// use media_organizer::utils::normalize_path_display;
/// 
/// let path = Path::new("folder//subfolder//file.txt");
/// assert_eq!(normalize_path_display(path), "folder/subfolder/file.txt");
/// ```
pub fn normalize_path_display(path: &Path) -> String {
    let path_str = path.display().to_string();
    normalize_path_string(&path_str)
}

/// Normalize a PathBuf for display purposes
///
/// This function takes a PathBuf reference and returns a normalized string representation.
/// It's a convenience wrapper around normalize_path_display for PathBuf types.
///
/// # Examples
/// ```
/// use std::path::PathBuf;
/// use media_organizer::utils::normalize_pathbuf_display;
/// 
/// let pathbuf = PathBuf::from("folder//subfolder//file.txt");
/// assert_eq!(normalize_pathbuf_display(&pathbuf), "folder/subfolder/file.txt");
/// ```
pub fn normalize_pathbuf_display(pathbuf: &PathBuf) -> String {
    normalize_path_display(pathbuf.as_path())
}

/// Create a normalized string suitable for HTML element IDs from a path
///
/// This function creates a string that can be safely used as HTML element IDs
/// by replacing path separators and spaces with safe characters.
///
/// # Examples
/// ```
/// use std::path::Path;
/// use media_organizer::utils::path_to_element_id;
/// 
/// let path = Path::new("folder/subfolder/file name.txt");
/// assert_eq!(path_to_element_id(path), "folder-subfolder-file_name_txt");
/// ```
pub fn path_to_element_id(path: &Path) -> String {
    let normalized = normalize_path_display(path);
    normalized
        .replace(['/', '\\'], "-")
        .replace(' ', "_")
        .replace('.', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_normalize_path_string_forward_slashes() {
        assert_eq!(normalize_path_string("folder//file.txt"), "folder/file.txt");
        assert_eq!(normalize_path_string("folder///file.txt"), "folder/file.txt");
        assert_eq!(normalize_path_string("/root//nested/file"), "/root/nested/file");
        assert_eq!(normalize_path_string("folder/file.txt"), "folder/file.txt"); // Already normalized
    }

    #[test]
    fn test_normalize_path_string_back_slashes() {
        assert_eq!(normalize_path_string("folder\\\\file.txt"), "folder\\file.txt");
        assert_eq!(normalize_path_string("folder\\\\\\file.txt"), "folder\\file.txt");
        assert_eq!(normalize_path_string("C:\\\\Documents\\\\file"), "C:\\Documents\\file");
    }

    #[test]
    fn test_normalize_path_string_mixed_slashes() {
        // Should normalize to predominant type (forward slashes win)
        assert_eq!(normalize_path_string("folder/sub\\\\folder//file"), "folder/sub/folder/file");
        // Backslashes win in this case
        assert_eq!(normalize_path_string("folder\\sub//folder\\\\file"), "folder\\sub\\folder\\file");
    }

    #[test]
    fn test_normalize_path_string_unc_paths() {
        // UNC paths should preserve double slash at start
        assert_eq!(normalize_path_string("//server/share"), "//server/share");
        assert_eq!(normalize_path_string("//server//share//folder"), "//server/share/folder");
        
        // Triple slash should be normalized to double
        assert_eq!(normalize_path_string("///server/share"), "/server/share");
    }

    #[test]
    fn test_normalize_path_string_edge_cases() {
        assert_eq!(normalize_path_string(""), "");
        assert_eq!(normalize_path_string("/"), "/");
        assert_eq!(normalize_path_string("\\"), "\\");
        assert_eq!(normalize_path_string("//"), "//");
        assert_eq!(normalize_path_string("filename"), "filename");
    }

    #[test]
    fn test_normalize_path_display() {
        let path = Path::new("folder//subfolder//file.txt");
        assert_eq!(normalize_path_display(path), "folder/subfolder/file.txt");
        
        let path = Path::new("C:\\\\Documents\\\\file.txt");
        let result = normalize_path_display(path);
        // Should be normalized but exact format depends on platform
        assert!(!result.contains("//"));
        assert!(!result.contains("\\\\"));
    }

    #[test]
    fn test_normalize_pathbuf_display() {
        let pathbuf = PathBuf::from("folder//subfolder//file.txt");
        assert_eq!(normalize_pathbuf_display(&pathbuf), "folder/subfolder/file.txt");
    }

    #[test]
    fn test_path_to_element_id() {
        let path = Path::new("folder/subfolder/file name.txt");
        assert_eq!(path_to_element_id(path), "folder-subfolder-file_name_txt");
        
        let path = Path::new("folder\\subfolder\\file.doc");
        let result = path_to_element_id(path);
        assert!(result.contains("folder-subfolder"));
        assert!(!result.contains(' '));
        assert!(!result.contains('.'));
    }

    #[test]
    fn test_path_to_element_id_double_slashes() {
        let path = Path::new("folder//subfolder//file name.txt");
        let result = path_to_element_id(path);
        assert_eq!(result, "folder-subfolder-file_name_txt");
        // Ensure no double separators in the ID
        assert!(!result.contains("--"));
    }
}