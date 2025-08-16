use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn, error};

use crate::services::{HashingService, FileHash, BackgroundProcessor, HashingTask, FileEntry};

/// Errors that can occur during duplicate detection
#[derive(Debug, Error)]
pub enum DuplicateDetectionError {
    #[error("Hashing service error: {0}")]
    HashingError(String),
    
    #[error("Background processing error: {0}")]
    BackgroundError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid file path: {path}")]
    InvalidPath { path: PathBuf },
    
    #[error("No files provided for detection")]
    NoFiles,
}

/// Result type for duplicate detection operations
pub type DuplicateDetectionResult<T> = Result<T, DuplicateDetectionError>;

/// Methods for comparing files to determine duplicates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonMethod {
    /// Compare files by content hash (SHA-256)
    Content,
    /// Compare files by size only
    Size,
    /// Compare files by name only
    Name,
    /// Compare by size and name combined
    SizeAndName,
    /// Compare by content hash and size (most reliable)
    ContentAndSize,
}

impl ComparisonMethod {
    /// Get human-readable name for the comparison method
    pub fn display_name(&self) -> &'static str {
        match self {
            ComparisonMethod::Content => "Content Hash",
            ComparisonMethod::Size => "File Size",
            ComparisonMethod::Name => "File Name",
            ComparisonMethod::SizeAndName => "Size + Name",
            ComparisonMethod::ContentAndSize => "Content + Size",
        }
    }
    
    /// Check if this method requires content hashing
    pub fn requires_hashing(&self) -> bool {
        matches!(self, ComparisonMethod::Content | ComparisonMethod::ContentAndSize)
    }
    
    /// Get the comparison key for a file based on this method
    pub fn get_key(&self, file_entry: &FileEntry, file_hash: Option<&FileHash>) -> Option<String> {
        match self {
            ComparisonMethod::Content => {
                file_hash.map(|h| h.hash.clone())
            }
            ComparisonMethod::Size => {
                Some(file_entry.size.to_string())
            }
            ComparisonMethod::Name => {
                Some(file_entry.name.clone())
            }
            ComparisonMethod::SizeAndName => {
                Some(format!("{}_{}", file_entry.size, file_entry.name))
            }
            ComparisonMethod::ContentAndSize => {
                file_hash.map(|h| format!("{}_{}", h.hash, file_entry.size))
            }
        }
    }
}

impl Default for ComparisonMethod {
    fn default() -> Self {
        ComparisonMethod::Content
    }
}

/// A single file within a duplicate group
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateFile {
    /// File entry with metadata
    pub file_entry: FileEntry,
    /// Content hash if available
    pub hash: Option<FileHash>,
    /// Whether this is selected as the primary/keep file
    pub is_primary: bool,
    /// Whether this file is selected for action (delete, move, etc.)
    pub is_selected: bool,
}

impl DuplicateFile {
    /// Create a new duplicate file entry
    pub fn new(file_entry: FileEntry, hash: Option<FileHash>) -> Self {
        Self {
            file_entry,
            hash,
            is_primary: false,
            is_selected: false,
        }
    }
    
    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.file_entry.path
    }
    
    /// Get the file size
    pub fn size(&self) -> u64 {
        self.file_entry.size
    }
    
    /// Get the modification time
    pub fn modified(&self) -> SystemTime {
        self.file_entry.modified
    }
    
    /// Check if this file has content hash
    pub fn has_hash(&self) -> bool {
        self.hash.is_some()
    }
    
    /// Get content hash if available
    pub fn content_hash(&self) -> Option<&str> {
        self.hash.as_ref().map(|h| h.hash.as_str())
    }
}

/// A group of duplicate files
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateGroup {
    /// Unique identifier for this group
    pub id: String,
    /// The comparison key that groups these files
    pub group_key: String,
    /// Method used to detect these duplicates
    pub comparison_method: ComparisonMethod,
    /// Files in this duplicate group
    pub files: Vec<DuplicateFile>,
    /// Total size of all files in the group
    pub total_size: u64,
    /// Size that would be saved by keeping only one file
    pub potential_savings: u64,
}

impl DuplicateGroup {
    /// Create a new duplicate group
    pub fn new(group_key: String, comparison_method: ComparisonMethod) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            group_key,
            comparison_method,
            files: Vec::new(),
            total_size: 0,
            potential_savings: 0,
        }
    }
    
    /// Add a file to this group
    pub fn add_file(&mut self, file: DuplicateFile) {
        self.total_size += file.size();
        self.files.push(file);
        self.update_potential_savings();
    }
    
    /// Get the number of files in this group
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
    
    /// Check if this group has actual duplicates (more than one file)
    pub fn has_duplicates(&self) -> bool {
        self.files.len() > 1
    }
    
    /// Get the primary file (the one to keep)
    pub fn primary_file(&self) -> Option<&DuplicateFile> {
        self.files.iter().find(|f| f.is_primary)
    }
    
    /// Get selected files (for deletion/moving)
    pub fn selected_files(&self) -> Vec<&DuplicateFile> {
        self.files.iter().filter(|f| f.is_selected).collect()
    }
    
    /// Set the primary file based on selection logic
    pub fn set_primary_file(&mut self, strategy: PrimarySelectionStrategy) {
        if self.files.is_empty() {
            return;
        }
        
        // Reset current primary status
        for file in &mut self.files {
            file.is_primary = false;
        }
        
        // Select primary based on strategy
        let primary_index = match strategy {
            PrimarySelectionStrategy::Oldest => {
                self.files.iter()
                    .enumerate()
                    .min_by_key(|(_, f)| f.modified())
                    .map(|(i, _)| i)
            }
            PrimarySelectionStrategy::Newest => {
                self.files.iter()
                    .enumerate()
                    .max_by_key(|(_, f)| f.modified())
                    .map(|(i, _)| i)
            }
            PrimarySelectionStrategy::ShortestPath => {
                self.files.iter()
                    .enumerate()
                    .min_by_key(|(_, f)| f.path().to_string_lossy().len())
                    .map(|(i, _)| i)
            }
            PrimarySelectionStrategy::LongestPath => {
                self.files.iter()
                    .enumerate()
                    .max_by_key(|(_, f)| f.path().to_string_lossy().len())
                    .map(|(i, _)| i)
            }
            PrimarySelectionStrategy::First => Some(0),
        };
        
        if let Some(index) = primary_index {
            if let Some(file) = self.files.get_mut(index) {
                file.is_primary = true;
            }
        }
    }
    
    /// Update potential savings calculation
    fn update_potential_savings(&mut self) {
        if self.files.len() <= 1 {
            self.potential_savings = 0;
            return;
        }
        
        // Find the largest file (assuming we keep the largest)
        let max_size = self.files.iter().map(|f| f.size()).max().unwrap_or(0);
        self.potential_savings = self.total_size.saturating_sub(max_size);
    }
    
    /// Sort files in the group by various criteria
    pub fn sort_files(&mut self, sort_by: FileSortCriteria) {
        match sort_by {
            FileSortCriteria::Size => {
                self.files.sort_by(|a, b| b.size().cmp(&a.size()));
            }
            FileSortCriteria::Name => {
                self.files.sort_by(|a, b| a.file_entry.name.cmp(&b.file_entry.name));
            }
            FileSortCriteria::Path => {
                self.files.sort_by(|a, b| a.path().cmp(b.path()));
            }
            FileSortCriteria::Modified => {
                self.files.sort_by(|a, b| b.modified().cmp(&a.modified()));
            }
        }
    }
}

/// Strategy for selecting the primary file to keep in a duplicate group
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimarySelectionStrategy {
    /// Keep the oldest file
    Oldest,
    /// Keep the newest file
    Newest,
    /// Keep the file with the shortest path
    ShortestPath,
    /// Keep the file with the longest path
    LongestPath,
    /// Keep the first file found
    First,
}

impl Default for PrimarySelectionStrategy {
    fn default() -> Self {
        PrimarySelectionStrategy::Oldest
    }
}

/// Criteria for sorting files within duplicate groups
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileSortCriteria {
    Size,
    Name,
    Path,
    Modified,
}

/// Configuration for duplicate detection
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateDetectionConfig {
    /// Method used for comparison
    pub comparison_method: ComparisonMethod,
    /// Minimum file size to consider (bytes)
    pub min_file_size: u64,
    /// Maximum file size to consider (bytes)
    pub max_file_size: u64,
    /// File extensions to include (empty = all)
    pub include_extensions: Vec<String>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
    /// Whether to include hidden files
    pub include_hidden: bool,
    /// Strategy for selecting primary file
    pub primary_selection: PrimarySelectionStrategy,
    /// Maximum number of files to process
    pub max_files: Option<usize>,
}

impl Default for DuplicateDetectionConfig {
    fn default() -> Self {
        Self {
            comparison_method: ComparisonMethod::default(),
            min_file_size: 1, // At least 1 byte
            max_file_size: u64::MAX,
            include_extensions: Vec::new(),
            exclude_extensions: vec![
                "tmp".to_string(),
                "temp".to_string(),
                "log".to_string(),
                "cache".to_string(),
            ],
            include_hidden: false,
            primary_selection: PrimarySelectionStrategy::default(),
            max_files: None,
        }
    }
}

impl DuplicateDetectionConfig {
    /// Check if a file should be included based on this configuration
    pub fn should_include_file(&self, file_entry: &FileEntry) -> bool {
        // Check file size
        if file_entry.size < self.min_file_size || file_entry.size > self.max_file_size {
            return false;
        }
        
        // Check hidden files
        if !self.include_hidden && file_entry.is_hidden {
            return false;
        }
        
        // Check extensions
        if let Some(extension) = file_entry.path.extension().and_then(|e| e.to_str()) {
            let ext = extension.to_lowercase();
            
            // If include list is specified, file must be in it
            if !self.include_extensions.is_empty() && !self.include_extensions.contains(&ext) {
                return false;
            }
            
            // File must not be in exclude list
            if self.exclude_extensions.contains(&ext) {
                return false;
            }
        }
        
        true
    }
}

/// Progress information for duplicate detection
#[derive(Debug, Clone, PartialEq)]
pub struct DetectionProgress {
    /// Current phase of detection
    pub phase: DetectionPhase,
    /// Number of files processed
    pub files_processed: usize,
    /// Total number of files to process
    pub total_files: usize,
    /// Number of duplicate groups found
    pub groups_found: usize,
    /// Current file being processed
    pub current_file: Option<PathBuf>,
    /// Estimated progress percentage (0.0 to 1.0)
    pub progress_percentage: f64,
}

/// Phases of duplicate detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionPhase {
    /// Scanning directories for files
    Scanning,
    /// Filtering files based on configuration
    Filtering,
    /// Computing file hashes
    Hashing,
    /// Grouping files by comparison criteria
    Grouping,
    /// Analyzing results and selecting primary files
    Analyzing,
    /// Completed
    Completed,
}

/// Callback function type for progress updates
pub type DetectionProgressCallback = std::sync::Arc<dyn Fn(DetectionProgress) + Send + Sync>;

/// Results from duplicate detection
#[derive(Debug, Clone, PartialEq)]
pub struct DuplicateDetectionResults {
    /// All duplicate groups found
    pub groups: Vec<DuplicateGroup>,
    /// Total number of files analyzed
    pub total_files_analyzed: usize,
    /// Total number of duplicate files found
    pub total_duplicates: usize,
    /// Total potential space savings in bytes
    pub total_potential_savings: u64,
    /// Time taken for detection
    pub detection_time_ms: u64,
    /// Configuration used for detection
    pub config: DuplicateDetectionConfig,
}

impl DuplicateDetectionResults {
    /// Get only groups that have actual duplicates
    pub fn duplicate_groups(&self) -> Vec<&DuplicateGroup> {
        self.groups.iter().filter(|g| g.has_duplicates()).collect()
    }
    
    /// Get the number of duplicate groups
    pub fn duplicate_group_count(&self) -> usize {
        self.groups.iter().filter(|g| g.has_duplicates()).count()
    }
    
    /// Get total space savings in a human-readable format
    pub fn format_savings(&self) -> String {
        format_file_size(self.total_potential_savings)
    }
}

/// Main service for duplicate detection
#[derive(Debug)]
pub struct DuplicateDetector {
    /// Hashing service for content comparison
    hashing_service: HashingService,
    /// Background processor for async operations
    background_processor: BackgroundProcessor,
    /// Current detection configuration
    config: DuplicateDetectionConfig,
}

impl DuplicateDetector {
    /// Create a new duplicate detector with default configuration
    pub fn new() -> Self {
        Self {
            hashing_service: HashingService::new(),
            background_processor: BackgroundProcessor::default(),
            config: DuplicateDetectionConfig::default(),
        }
    }
    
    /// Create a new duplicate detector with custom configuration
    pub fn with_config(config: DuplicateDetectionConfig) -> Self {
        Self {
            hashing_service: HashingService::new(),
            background_processor: BackgroundProcessor::default(),
            config,
        }
    }
    
    /// Update the detection configuration
    pub fn set_config(&mut self, config: DuplicateDetectionConfig) {
        self.config = config;
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &DuplicateDetectionConfig {
        &self.config
    }
    
    /// Detect duplicates in the given files
    pub async fn detect_duplicates(
        &self,
        files: Vec<FileEntry>,
        progress_callback: Option<DetectionProgressCallback>,
    ) -> DuplicateDetectionResult<DuplicateDetectionResults> {
        let start_time = std::time::Instant::now();
        
        if files.is_empty() {
            return Err(DuplicateDetectionError::NoFiles);
        }
        
        info!("Starting duplicate detection for {} files", files.len());
        
        // Phase 1: Filter files based on configuration
        let mut progress = DetectionProgress {
            phase: DetectionPhase::Filtering,
            files_processed: 0,
            total_files: files.len(),
            groups_found: 0,
            current_file: None,
            progress_percentage: 0.0,
        };
        
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        let filtered_files: Vec<FileEntry> = files
            .into_iter()
            .filter(|file| self.config.should_include_file(file))
            .take(self.config.max_files.unwrap_or(usize::MAX))
            .collect();
        
        info!("Filtered to {} files for processing", filtered_files.len());
        
        progress.total_files = filtered_files.len();
        progress.progress_percentage = 10.0;
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        // Phase 2: Hash files if needed for content comparison
        let file_hashes = if self.config.comparison_method.requires_hashing() {
            progress.phase = DetectionPhase::Hashing;
            progress.progress_percentage = 20.0;
            if let Some(ref callback) = progress_callback {
                callback(progress.clone());
            }
            
            self.hash_files(&filtered_files, progress_callback.clone()).await?
        } else {
            HashMap::new()
        };
        
        // Phase 3: Group files by comparison criteria
        progress.phase = DetectionPhase::Grouping;
        progress.progress_percentage = 70.0;
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        let filtered_files_count = filtered_files.len();
        let groups = self.group_files(filtered_files, file_hashes).await?;
        
        // Phase 4: Analyze results and set primary files
        progress.phase = DetectionPhase::Analyzing;
        progress.progress_percentage = 90.0;
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        let analyzed_groups = self.analyze_groups(groups).await?;
        
        // Calculate final statistics
        let total_duplicates = analyzed_groups.iter()
            .filter(|g| g.has_duplicates())
            .map(|g| g.file_count().saturating_sub(1))
            .sum();
        
        let total_potential_savings = analyzed_groups.iter()
            .map(|g| g.potential_savings)
            .sum();
        
        let detection_time = start_time.elapsed().as_millis() as u64;
        
        progress.phase = DetectionPhase::Completed;
        progress.progress_percentage = 100.0;
        progress.groups_found = analyzed_groups.iter().filter(|g| g.has_duplicates()).count();
        if let Some(ref callback) = progress_callback {
            callback(progress.clone());
        }
        
        info!(
            "Duplicate detection completed in {}ms: {} groups, {} duplicates, {} potential savings",
            detection_time,
            analyzed_groups.len(),
            total_duplicates,
            format_file_size(total_potential_savings)
        );
        
        Ok(DuplicateDetectionResults {
            groups: analyzed_groups,
            total_files_analyzed: filtered_files_count,
            total_duplicates,
            total_potential_savings,
            detection_time_ms: detection_time,
            config: self.config.clone(),
        })
    }
    
    /// Hash files using the background processor
    async fn hash_files(
        &self,
        files: &[FileEntry],
        progress_callback: Option<DetectionProgressCallback>,
    ) -> DuplicateDetectionResult<HashMap<PathBuf, FileHash>> {
        let file_paths: Vec<PathBuf> = files.iter().map(|f| f.path.clone()).collect();
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        
        let mut file_hashes = HashMap::new();
        
        // Create progress callback for hashing
        let hash_progress_callback: crate::services::BackgroundProgressCallback = 
            if let Some(detection_callback) = progress_callback {
                std::sync::Arc::new(move |hash_progress: crate::services::BackgroundProgressInfo| {
                    let detection_progress = DetectionProgress {
                        phase: DetectionPhase::Hashing,
                        files_processed: hash_progress.files_processed,
                        total_files: hash_progress.total_files,
                        groups_found: 0,
                        current_file: hash_progress.current_file.clone(),
                        progress_percentage: 20.0 + (hash_progress.completion_percentage() * 50.0), // 20-70%
                    };
                    detection_callback(detection_progress);
                })
            } else {
                std::sync::Arc::new(|_| {})
            };
        
        // Create hashing task
        let hashing_task = HashingTask::new(
            file_paths,
            total_size,
            hash_progress_callback,
        );
        
        let task_id = self.background_processor
            .start_hashing_task(hashing_task)
            .await
            .map_err(|e| DuplicateDetectionError::BackgroundError(e.to_string()))?;
        
        // Wait for completion
        loop {
            if !self.background_processor.is_task_running(task_id).await {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        
        // Get results
        if let Some(result) = self.background_processor.get_task_result(task_id).await {
            for file_hash in result.successful_hashes {
                file_hashes.insert(file_hash.path.clone(), file_hash);
            }
            
            if !result.failed_files.is_empty() {
                warn!("Failed to hash {} files", result.failed_files.len());
                for (path, error) in &result.failed_files {
                    debug!("Failed to hash {}: {}", path.display(), error);
                }
            }
        }
        
        Ok(file_hashes)
    }
    
    /// Group files based on the comparison method
    async fn group_files(
        &self,
        files: Vec<FileEntry>,
        file_hashes: HashMap<PathBuf, FileHash>,
    ) -> DuplicateDetectionResult<Vec<DuplicateGroup>> {
        let mut groups_map: HashMap<String, DuplicateGroup> = HashMap::new();
        
        for file_entry in files {
            let file_hash = file_hashes.get(&file_entry.path);
            
            // Get comparison key
            if let Some(key) = self.config.comparison_method.get_key(&file_entry, file_hash) {
                let group = groups_map.entry(key.clone()).or_insert_with(|| {
                    DuplicateGroup::new(key, self.config.comparison_method)
                });
                
                let duplicate_file = DuplicateFile::new(file_entry, file_hash.cloned());
                group.add_file(duplicate_file);
            }
        }
        
        Ok(groups_map.into_values().collect())
    }
    
    /// Analyze groups and set primary files
    async fn analyze_groups(
        &self,
        mut groups: Vec<DuplicateGroup>,
    ) -> DuplicateDetectionResult<Vec<DuplicateGroup>> {
        for group in &mut groups {
            if group.has_duplicates() {
                // Sort files for consistent ordering
                group.sort_files(FileSortCriteria::Size);
                
                // Set primary file based on strategy
                group.set_primary_file(self.config.primary_selection);
            }
        }
        
        // Sort groups by potential savings (highest first)
        groups.sort_by(|a, b| b.potential_savings.cmp(&a.potential_savings));
        
        Ok(groups)
    }
}

impl Default for DuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Format file size in human-readable format
fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use crate::services::file_system::{FileType, TextFormat, FilePermissions};
    
    /// Helper function to create a test file entry
    fn create_test_file_entry(name: &str, size: u64, content: Option<&str>) -> (FileEntry, Option<NamedTempFile>) {
        let temp_file = if content.is_some() {
            Some(NamedTempFile::new().unwrap())
        } else {
            None
        };
        
        let path = if let Some(ref temp) = temp_file {
            temp.path().to_path_buf()
        } else {
            PathBuf::from(format!("/test/{}", name))
        };
        
        // Write content if provided
        if let (Some(content), Some(ref temp)) = (content, &temp_file) {
            std::fs::write(temp.path(), content.as_bytes()).unwrap();
        }
        
        let file_entry = FileEntry {
            path,
            name: name.to_string(),
            file_type: FileType::Text(TextFormat::Plain),
            size,
            modified: SystemTime::now(),
            created: SystemTime::now(),
            is_directory: false,
            is_hidden: false,
            permissions: FilePermissions::read_write(),
        };
        
        (file_entry, temp_file)
    }
    
    #[test]
    fn test_comparison_method_display_names() {
        assert_eq!(ComparisonMethod::Content.display_name(), "Content Hash");
        assert_eq!(ComparisonMethod::Size.display_name(), "File Size");
        assert_eq!(ComparisonMethod::Name.display_name(), "File Name");
        assert_eq!(ComparisonMethod::SizeAndName.display_name(), "Size + Name");
        assert_eq!(ComparisonMethod::ContentAndSize.display_name(), "Content + Size");
    }
    
    #[test]
    fn test_comparison_method_requires_hashing() {
        assert!(ComparisonMethod::Content.requires_hashing());
        assert!(!ComparisonMethod::Size.requires_hashing());
        assert!(!ComparisonMethod::Name.requires_hashing());
        assert!(!ComparisonMethod::SizeAndName.requires_hashing());
        assert!(ComparisonMethod::ContentAndSize.requires_hashing());
    }
    
    #[test]
    fn test_comparison_method_get_key() {
        let (file_entry, _temp) = create_test_file_entry("test.txt", 100, None);
        
        // Test size key
        let size_key = ComparisonMethod::Size.get_key(&file_entry, None);
        assert_eq!(size_key, Some("100".to_string()));
        
        // Test name key
        let name_key = ComparisonMethod::Name.get_key(&file_entry, None);
        assert_eq!(name_key, Some("test.txt".to_string()));
        
        // Test size and name key
        let size_name_key = ComparisonMethod::SizeAndName.get_key(&file_entry, None);
        assert_eq!(size_name_key, Some("100_test.txt".to_string()));
        
        // Test content key (requires hash)
        let file_hash = FileHash::new("abc123".to_string(), file_entry.path.clone(), 100, 50);
        let content_key = ComparisonMethod::Content.get_key(&file_entry, Some(&file_hash));
        assert_eq!(content_key, Some("abc123".to_string()));
        
        // Test content and size key
        let content_size_key = ComparisonMethod::ContentAndSize.get_key(&file_entry, Some(&file_hash));
        assert_eq!(content_size_key, Some("abc123_100".to_string()));
    }
    
    #[test]
    fn test_duplicate_file_creation() {
        let (file_entry, _temp) = create_test_file_entry("test.txt", 100, None);
        let file_hash = FileHash::new("abc123".to_string(), file_entry.path.clone(), 100, 50);
        
        let dup_file = DuplicateFile::new(file_entry.clone(), Some(file_hash.clone()));
        
        assert_eq!(dup_file.path(), &file_entry.path);
        assert_eq!(dup_file.size(), 100);
        assert!(dup_file.has_hash());
        assert_eq!(dup_file.content_hash(), Some("abc123"));
        assert!(!dup_file.is_primary);
        assert!(!dup_file.is_selected);
    }
    
    #[test]
    fn test_duplicate_group_creation() {
        let group = DuplicateGroup::new("test_key".to_string(), ComparisonMethod::Content);
        
        assert_eq!(group.group_key, "test_key");
        assert_eq!(group.comparison_method, ComparisonMethod::Content);
        assert_eq!(group.file_count(), 0);
        assert!(!group.has_duplicates());
        assert_eq!(group.total_size, 0);
        assert_eq!(group.potential_savings, 0);
    }
    
    #[test]
    fn test_duplicate_group_add_files() {
        let mut group = DuplicateGroup::new("test_key".to_string(), ComparisonMethod::Content);
        
        let (file1, _temp1) = create_test_file_entry("test1.txt", 100, None);
        let (file2, _temp2) = create_test_file_entry("test2.txt", 150, None);
        
        let dup_file1 = DuplicateFile::new(file1, None);
        let dup_file2 = DuplicateFile::new(file2, None);
        
        group.add_file(dup_file1);
        assert_eq!(group.file_count(), 1);
        assert!(!group.has_duplicates());
        assert_eq!(group.total_size, 100);
        
        group.add_file(dup_file2);
        assert_eq!(group.file_count(), 2);
        assert!(group.has_duplicates());
        assert_eq!(group.total_size, 250);
        assert_eq!(group.potential_savings, 100); // 250 - 150 (largest file)
    }
    
    #[test]
    fn test_primary_selection_strategies() {
        let mut group = DuplicateGroup::new("test_key".to_string(), ComparisonMethod::Content);
        
        let (file1, _temp1) = create_test_file_entry("short.txt", 100, None);
        let (file2, _temp2) = create_test_file_entry("very_long_filename.txt", 150, None);
        
        let dup_file1 = DuplicateFile::new(file1, None);
        let dup_file2 = DuplicateFile::new(file2, None);
        
        group.add_file(dup_file1);
        group.add_file(dup_file2);
        
        // Test shortest path selection
        group.set_primary_file(PrimarySelectionStrategy::ShortestPath);
        let primary = group.primary_file().unwrap();
        assert_eq!(primary.file_entry.name, "short.txt");
        
        // Test longest path selection
        group.set_primary_file(PrimarySelectionStrategy::LongestPath);
        let primary = group.primary_file().unwrap();
        assert_eq!(primary.file_entry.name, "very_long_filename.txt");
        
        // Test first selection
        group.set_primary_file(PrimarySelectionStrategy::First);
        let primary = group.primary_file().unwrap();
        assert_eq!(primary.file_entry.name, "short.txt"); // First added
    }
    
    #[test]
    fn test_duplicate_detection_config_filtering() {
        let config = DuplicateDetectionConfig {
            min_file_size: 50,
            max_file_size: 200,
            include_extensions: vec!["txt".to_string()],
            exclude_extensions: vec!["log".to_string()],
            include_hidden: false,
            ..Default::default()
        };
        
        // Test file within size range and correct extension
        let (good_file, _temp1) = create_test_file_entry("test.txt", 100, None);
        assert!(config.should_include_file(&good_file));
        
        // Test file too small
        let (small_file, _temp2) = create_test_file_entry("small.txt", 30, None);
        assert!(!config.should_include_file(&small_file));
        
        // Test file too large
        let (large_file, _temp3) = create_test_file_entry("large.txt", 300, None);
        assert!(!config.should_include_file(&large_file));
        
        // Test excluded extension
        let (log_file, _temp4) = create_test_file_entry("test.log", 100, None);
        assert!(!config.should_include_file(&log_file));
        
        // Test hidden file
        let mut hidden_file = good_file.clone();
        hidden_file.is_hidden = true;
        assert!(!config.should_include_file(&hidden_file));
    }
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_file_size(1024_u64.pow(4)), "1.0 TB");
    }
    
    #[tokio::test]
    async fn test_duplicate_detector_creation() {
        let detector = DuplicateDetector::new();
        assert_eq!(detector.config().comparison_method, ComparisonMethod::Content);
        assert_eq!(detector.config().min_file_size, 1);
        assert!(!detector.config().include_hidden);
    }
    
    #[tokio::test]
    async fn test_duplicate_detector_with_config() {
        let config = DuplicateDetectionConfig {
            comparison_method: ComparisonMethod::Size,
            min_file_size: 1000,
            ..Default::default()
        };
        
        let detector = DuplicateDetector::with_config(config.clone());
        assert_eq!(detector.config().comparison_method, ComparisonMethod::Size);
        assert_eq!(detector.config().min_file_size, 1000);
    }
    
    #[tokio::test]
    async fn test_duplicate_detection_no_files() {
        let detector = DuplicateDetector::new();
        let result = detector.detect_duplicates(vec![], None).await;
        
        assert!(matches!(result, Err(DuplicateDetectionError::NoFiles)));
    }
    
    #[tokio::test]
    async fn test_duplicate_detection_size_comparison() {
        let detector = DuplicateDetector::with_config(DuplicateDetectionConfig {
            comparison_method: ComparisonMethod::Size,
            ..Default::default()
        });
        
        // Create test files with same size
        let (file1, _temp1) = create_test_file_entry("file1.txt", 100, None);
        let (file2, _temp2) = create_test_file_entry("file2.txt", 100, None);
        let (file3, _temp3) = create_test_file_entry("file3.txt", 200, None);
        
        let files = vec![file1, file2, file3];
        
        // Track progress updates
        let progress_count = std::sync::Arc::new(AtomicUsize::new(0));
        let progress_count_clone = progress_count.clone();
        
        let callback = std::sync::Arc::new(move |progress: DetectionProgress| {
            progress_count_clone.fetch_add(1, Ordering::SeqCst);
            println!("Progress: {:?} - {:.1}%", progress.phase, progress.progress_percentage);
        });
        
        let result = detector.detect_duplicates(files, Some(callback)).await.unwrap();
        
        // Should find one group with 2 duplicates (size 100)
        let duplicate_groups = result.duplicate_groups();
        assert_eq!(duplicate_groups.len(), 1);
        assert_eq!(duplicate_groups[0].file_count(), 2);
        assert_eq!(duplicate_groups[0].total_size, 200);
        assert_eq!(duplicate_groups[0].potential_savings, 100);
        
        // Check that we got progress updates
        assert!(progress_count.load(Ordering::SeqCst) > 0);
    }
    
    #[tokio::test]
    async fn test_duplicate_detection_name_comparison() {
        let detector = DuplicateDetector::with_config(DuplicateDetectionConfig {
            comparison_method: ComparisonMethod::Name,
            ..Default::default()
        });
        
        // Create test files with same name
        let (file1, _temp1) = create_test_file_entry("duplicate.txt", 100, None);
        let (file2, _temp2) = create_test_file_entry("duplicate.txt", 150, None);
        let (file3, _temp3) = create_test_file_entry("unique.txt", 200, None);
        
        let files = vec![file1, file2, file3];
        
        let result = detector.detect_duplicates(files, None).await.unwrap();
        
        // Should find one group with 2 duplicates (same name)
        let duplicate_groups = result.duplicate_groups();
        assert_eq!(duplicate_groups.len(), 1);
        assert_eq!(duplicate_groups[0].file_count(), 2);
        assert_eq!(duplicate_groups[0].comparison_method, ComparisonMethod::Name);
    }
    
    #[tokio::test]
    async fn test_duplicate_detection_results() {
        let detector = DuplicateDetector::with_config(DuplicateDetectionConfig {
            comparison_method: ComparisonMethod::Size,
            primary_selection: PrimarySelectionStrategy::First,
            ..Default::default()
        });
        
        // Create test files
        let (file1, _temp1) = create_test_file_entry("file1.txt", 100, None);
        let (file2, _temp2) = create_test_file_entry("file2.txt", 100, None);
        let (file3, _temp3) = create_test_file_entry("file3.txt", 200, None);
        
        let files = vec![file1, file2, file3];
        
        let result = detector.detect_duplicates(files, None).await.unwrap();
        
        assert_eq!(result.total_files_analyzed, 3);
        assert_eq!(result.total_duplicates, 1); // One duplicate file (keeping one of the two size-100 files)
        assert_eq!(result.duplicate_group_count(), 1);
        assert!(result.detection_time_ms >= 0); // Detection time might be 0 for very fast operations
        assert_eq!(result.config.comparison_method, ComparisonMethod::Size);
        
        // Test format savings
        assert!(result.format_savings().contains("B"));
    }
    
    #[test]
    fn test_detection_progress_creation() {
        let progress = DetectionProgress {
            phase: DetectionPhase::Filtering,
            files_processed: 5,
            total_files: 10,
            groups_found: 2,
            current_file: Some(PathBuf::from("test.txt")),
            progress_percentage: 50.0,
        };
        
        assert_eq!(progress.phase, DetectionPhase::Filtering);
        assert_eq!(progress.files_processed, 5);
        assert_eq!(progress.total_files, 10);
        assert_eq!(progress.groups_found, 2);
        assert_eq!(progress.progress_percentage, 50.0);
        assert!(progress.current_file.is_some());
    }
}