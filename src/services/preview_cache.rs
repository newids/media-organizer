use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, Duration};
use std::sync::{Arc, Mutex};
// use chrono::{DateTime, Utc};
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Errors that can occur in the preview cache
#[derive(Error, Debug)]
pub enum PreviewCacheError {
    #[error("Cache is full and eviction failed")]
    CacheFull,
    #[error("Preview data too large: {size} bytes (max: {max_size} bytes)")]
    DataTooLarge { size: usize, max_size: usize },
    #[error("Preview not found: {path}")]
    NotFound { path: String },
    #[error("Cache corruption: {reason}")]
    Corruption { reason: String },
    #[error("Memory limit exceeded: current {current} bytes, limit {limit} bytes")]
    MemoryLimitExceeded { current: usize, limit: usize },
}

/// Cache key for preview data - combines path and modification time for invalidation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PreviewCacheKey {
    pub path: PathBuf,
    pub modified_time: SystemTime,
}

impl PreviewCacheKey {
    pub fn new(path: PathBuf, modified_time: SystemTime) -> Self {
        Self { path, modified_time }
    }

    /// Create key from path, automatically getting modification time from filesystem
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        let modified_time = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        Ok(Self {
            path: path.to_path_buf(),
            modified_time,
        })
    }

    /// Check if this key is still valid (file hasn't been modified)
    pub fn is_valid(&self) -> bool {
        match std::fs::metadata(&self.path) {
            Ok(metadata) => {
                let current_modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                current_modified <= self.modified_time
            }
            Err(_) => false, // File doesn't exist, key is invalid
        }
    }
}

/// Preview data stored in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPreviewData {
    /// Preview content (thumbnail data, waveform, etc.)
    pub data: Vec<u8>,
    /// Content type for proper handling
    pub content_type: String,
    /// Preview format identifier
    pub format: String,
    /// Size of original file for context
    pub original_size: u64,
    /// When this preview was generated
    pub generated_at: SystemTime,
    /// Metadata associated with the preview
    pub metadata: PreviewDataMetadata,
}

impl CachedPreviewData {
    pub fn new(
        data: Vec<u8>,
        content_type: String,
        format: String,
        original_size: u64,
        metadata: PreviewDataMetadata,
    ) -> Self {
        Self {
            data,
            content_type,
            format,
            original_size,
            generated_at: SystemTime::now(),
            metadata,
        }
    }

    /// Get the memory size of this cached data
    pub fn memory_size(&self) -> usize {
        self.data.len() + 
        self.content_type.len() + 
        self.format.len() + 
        self.metadata.memory_size() +
        64 // Estimated overhead for other fields
    }

    /// Check if this preview data is still fresh (not too old)
    pub fn is_fresh(&self, max_age: Duration) -> bool {
        self.generated_at.elapsed().unwrap_or(Duration::MAX) < max_age
    }
}

/// Additional metadata for preview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewDataMetadata {
    /// Image/video dimensions
    pub width: Option<u32>,
    pub height: Option<u32>,
    /// Media duration for video/audio
    pub duration: Option<f64>,
    /// Quality level (e.g., thumbnail size, compression level)
    pub quality_level: u8,
}

impl PreviewDataMetadata {
    pub fn new() -> Self {
        Self {
            width: None,
            height: None,
            duration: None,
            quality_level: 80, // Default quality
        }
    }

    fn memory_size(&self) -> usize {
        32 // Estimated size for all fields
    }
}

/// LRU cache node for doubly-linked list
#[derive(Debug, Clone)]
struct CacheNode {
    key: PreviewCacheKey,
    data: CachedPreviewData,
    /// Previous node in LRU order (more recently used)
    prev: Option<usize>,
    /// Next node in LRU order (less recently used)
    next: Option<usize>,
}

impl CacheNode {
    fn new(key: PreviewCacheKey, data: CachedPreviewData) -> Self {
        Self {
            key,
            data,
            prev: None,
            next: None,
        }
    }
}

/// Configuration for the preview LRU cache
#[derive(Debug, Clone)]
pub struct PreviewCacheConfig {
    /// Maximum number of previews to cache
    pub max_entries: usize,
    /// Maximum total memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum age for cached previews before they're considered stale
    pub max_age: Duration,
    /// Maximum size for individual preview data
    pub max_single_item_bytes: usize,
}

impl Default for PreviewCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 50,                           // 50 previews as per requirements
            max_memory_bytes: 500 * 1024 * 1024,      // 500MB as per requirements
            max_age: Duration::from_secs(24 * 3600),  // 24 hours default
            max_single_item_bytes: 50 * 1024 * 1024,  // 50MB max per item
        }
    }
}

/// Statistics about the preview cache
#[derive(Debug, Clone)]
pub struct PreviewCacheStats {
    /// Number of entries currently in cache
    pub entries: usize,
    /// Total memory usage in bytes
    pub memory_bytes: usize,
    /// Cache hit rate (hits / (hits + misses))
    pub hit_rate: f64,
    /// Total cache hits
    pub hits: u64,
    /// Total cache misses
    pub misses: u64,
    /// Number of evictions performed
    pub evictions: u64,
    /// Most recently used entry (for debugging)
    pub most_recent_key: Option<PreviewCacheKey>,
    /// Least recently used entry (for debugging)
    pub least_recent_key: Option<PreviewCacheKey>,
}

/// In-memory LRU cache for preview data
/// Uses a HashMap for O(1) access and doubly-linked list for O(1) LRU operations
pub struct PreviewLRUCache {
    config: PreviewCacheConfig,
    /// HashMap for O(1) key lookup - maps to node index
    map: HashMap<PreviewCacheKey, usize>,
    /// Node storage - uses indices instead of pointers for safety
    nodes: Vec<Option<CacheNode>>,
    /// Free node indices for reuse
    free_indices: Vec<usize>,
    /// Head of LRU list (most recently used)
    head: Option<usize>,
    /// Tail of LRU list (least recently used)
    tail: Option<usize>,
    /// Current memory usage
    current_memory: usize,
    /// Statistics
    hits: u64,
    misses: u64,
    evictions: u64,
}

impl PreviewLRUCache {
    /// Create a new preview LRU cache with the given configuration
    pub fn new(config: PreviewCacheConfig) -> Self {
        Self {
            config,
            map: HashMap::new(),
            nodes: Vec::new(),
            free_indices: Vec::new(),
            head: None,
            tail: None,
            current_memory: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
        }
    }

    /// Get preview data from cache
    pub fn get(&mut self, key: &PreviewCacheKey) -> Option<&CachedPreviewData> {
        if let Some(&node_index) = self.map.get(key) {
            // Check if the key is still valid (file hasn't been modified)
            if !key.is_valid() {
                // File has been modified, remove stale entry
                self.remove_by_index(node_index);
                self.misses += 1;
                return None;
            }

            // Move to front (most recently used)
            self.move_to_front(node_index);
            self.hits += 1;
            
            // Return the data
            self.nodes[node_index].as_ref().map(|node| &node.data)
        } else {
            self.misses += 1;
            None
        }
    }

    /// Put preview data into cache
    pub fn put(
        &mut self, 
        key: PreviewCacheKey, 
        data: CachedPreviewData
    ) -> Result<(), PreviewCacheError> {
        let data_size = data.memory_size();
        
        // Check if single item is too large
        if data_size > self.config.max_single_item_bytes {
            return Err(PreviewCacheError::DataTooLarge {
                size: data_size,
                max_size: self.config.max_single_item_bytes,
            });
        }

        // If key already exists, update it
        if let Some(&existing_index) = self.map.get(&key) {
            self.update_existing(existing_index, data, data_size)?;
            return Ok(());
        }

        // Ensure we have capacity (memory and count)
        self.ensure_capacity(data_size)?;

        // Add new node
        let node_index = self.allocate_node_index();
        let node = CacheNode::new(key.clone(), data);
        
        if node_index >= self.nodes.len() {
            self.nodes.resize(node_index + 1, None);
        }
        
        self.nodes[node_index] = Some(node);
        self.map.insert(key, node_index);
        self.current_memory += data_size;
        
        // Add to front of list
        self.add_to_front(node_index);
        
        Ok(())
    }

    /// Remove entry from cache by key
    pub fn remove(&mut self, key: &PreviewCacheKey) -> bool {
        if let Some(&node_index) = self.map.get(key) {
            self.remove_by_index(node_index);
            true
        } else {
            false
        }
    }

    /// Clear all entries from cache
    pub fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.free_indices.clear();
        self.head = None;
        self.tail = None;
        self.current_memory = 0;
        // Don't reset stats - they're cumulative
    }

    /// Get cache statistics
    pub fn stats(&self) -> PreviewCacheStats {
        let total_requests = self.hits + self.misses;
        let hit_rate = if total_requests > 0 {
            self.hits as f64 / total_requests as f64
        } else {
            0.0
        };

        PreviewCacheStats {
            entries: self.map.len(),
            memory_bytes: self.current_memory,
            hit_rate,
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            most_recent_key: self.head.and_then(|idx| {
                self.nodes.get(idx)?.as_ref().map(|node| node.key.clone())
            }),
            least_recent_key: self.tail.and_then(|idx| {
                self.nodes.get(idx)?.as_ref().map(|node| node.key.clone())
            }),
        }
    }

    /// Clean up stale entries (files that no longer exist or have been modified)
    pub fn cleanup_stale(&mut self) -> usize {
        let mut stale_keys = Vec::new();
        
        for key in self.map.keys() {
            if !key.is_valid() {
                stale_keys.push(key.clone());
            }
        }
        
        let count = stale_keys.len();
        for key in stale_keys {
            self.remove(&key);
        }
        
        count
    }

    /// Clean up entries older than max_age
    pub fn cleanup_old(&mut self) -> usize {
        let mut old_keys = Vec::new();
        let max_age = self.config.max_age;
        
        for (key, &node_index) in &self.map {
            if let Some(node) = self.nodes[node_index].as_ref() {
                if !node.data.is_fresh(max_age) {
                    old_keys.push(key.clone());
                }
            }
        }
        
        let count = old_keys.len();
        for key in old_keys {
            self.remove(&key);
        }
        
        count
    }

    /// Get current memory usage as percentage of limit
    pub fn memory_usage_percent(&self) -> f64 {
        if self.config.max_memory_bytes > 0 {
            (self.current_memory as f64 / self.config.max_memory_bytes as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if cache needs cleanup
    pub fn needs_cleanup(&self) -> bool {
        self.current_memory > self.config.max_memory_bytes * 90 / 100 || // >90% memory
        self.map.len() > self.config.max_entries * 90 / 100 // >90% entries
    }

    // Private helper methods

    fn update_existing(
        &mut self, 
        node_index: usize, 
        new_data: CachedPreviewData,
        new_size: usize
    ) -> Result<(), PreviewCacheError> {
        if let Some(ref mut node) = self.nodes[node_index] {
            let old_size = node.data.memory_size();
            
            // Check if we have enough memory for the update
            let memory_delta = new_size.saturating_sub(old_size);
            if self.current_memory + memory_delta > self.config.max_memory_bytes {
                return Err(PreviewCacheError::MemoryLimitExceeded {
                    current: self.current_memory + memory_delta,
                    limit: self.config.max_memory_bytes,
                });
            }
            
            // Update the data and memory tracking
            node.data = new_data;
            self.current_memory = self.current_memory.saturating_sub(old_size) + new_size;
            
            // Move to front
            self.move_to_front(node_index);
        }
        
        Ok(())
    }

    fn ensure_capacity(&mut self, needed_bytes: usize) -> Result<(), PreviewCacheError> {
        // First, try to free up memory by removing LRU entries
        while (self.current_memory + needed_bytes > self.config.max_memory_bytes) ||
              (self.map.len() >= self.config.max_entries) {
            if !self.evict_lru() {
                return Err(PreviewCacheError::CacheFull);
            }
        }
        
        Ok(())
    }

    fn evict_lru(&mut self) -> bool {
        if let Some(tail_index) = self.tail {
            self.remove_by_index(tail_index);
            self.evictions += 1;
            true
        } else {
            false
        }
    }

    fn remove_by_index(&mut self, node_index: usize) {
        if let Some(node) = self.nodes[node_index].take() {
            // Remove from hash map
            self.map.remove(&node.key);
            
            // Update memory tracking
            self.current_memory = self.current_memory.saturating_sub(node.data.memory_size());
            
            // Remove from linked list
            self.remove_from_list(node_index);
            
            // Add index to free list for reuse
            self.free_indices.push(node_index);
        }
    }

    fn remove_from_list(&mut self, node_index: usize) {
        let (prev_idx, next_idx) = if let Some(node) = &self.nodes[node_index] {
            (node.prev, node.next)
        } else {
            return;
        };

        // Update previous node's next pointer
        if let Some(prev_idx) = prev_idx {
            if let Some(ref mut prev_node) = self.nodes[prev_idx] {
                prev_node.next = next_idx;
            }
        } else {
            // This was the head
            self.head = next_idx;
        }

        // Update next node's prev pointer
        if let Some(next_idx) = next_idx {
            if let Some(ref mut next_node) = self.nodes[next_idx] {
                next_node.prev = prev_idx;
            }
        } else {
            // This was the tail
            self.tail = prev_idx;
        }
    }

    fn move_to_front(&mut self, node_index: usize) {
        // Remove from current position
        self.remove_from_list(node_index);
        // Add to front
        self.add_to_front(node_index);
    }

    fn add_to_front(&mut self, node_index: usize) {
        if let Some(ref mut node) = self.nodes[node_index] {
            node.prev = None;
            node.next = self.head;
        }

        if let Some(old_head) = self.head {
            if let Some(ref mut old_head_node) = self.nodes[old_head] {
                old_head_node.prev = Some(node_index);
            }
        }

        self.head = Some(node_index);

        if self.tail.is_none() {
            self.tail = Some(node_index);
        }
    }

    fn allocate_node_index(&mut self) -> usize {
        if let Some(index) = self.free_indices.pop() {
            index
        } else {
            self.nodes.len()
        }
    }
}

/// Thread-safe wrapper around PreviewLRUCache
pub struct ThreadSafePreviewCache {
    cache: Arc<Mutex<PreviewLRUCache>>,
}

impl ThreadSafePreviewCache {
    pub fn new(config: PreviewCacheConfig) -> Self {
        Self {
            cache: Arc::new(Mutex::new(PreviewLRUCache::new(config))),
        }
    }

    pub fn get(&self, key: &PreviewCacheKey) -> Option<CachedPreviewData> {
        self.cache.lock().unwrap().get(key).cloned()
    }

    pub fn put(&self, key: PreviewCacheKey, data: CachedPreviewData) -> Result<(), PreviewCacheError> {
        self.cache.lock().unwrap().put(key, data)
    }

    pub fn remove(&self, key: &PreviewCacheKey) -> bool {
        self.cache.lock().unwrap().remove(key)
    }

    pub fn clear(&self) {
        self.cache.lock().unwrap().clear();
    }

    pub fn stats(&self) -> PreviewCacheStats {
        self.cache.lock().unwrap().stats()
    }

    pub fn cleanup_stale(&self) -> usize {
        self.cache.lock().unwrap().cleanup_stale()
    }

    pub fn cleanup_old(&self) -> usize {
        self.cache.lock().unwrap().cleanup_old()
    }

    pub fn memory_usage_percent(&self) -> f64 {
        self.cache.lock().unwrap().memory_usage_percent()
    }

    pub fn needs_cleanup(&self) -> bool {
        self.cache.lock().unwrap().needs_cleanup()
    }
}

impl Clone for ThreadSafePreviewCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    fn create_test_key(path: &str) -> PreviewCacheKey {
        PreviewCacheKey::new(
            PathBuf::from(path),
            SystemTime::now(),
        )
    }

    fn create_test_data(size: usize, content_type: &str) -> CachedPreviewData {
        CachedPreviewData::new(
            vec![0u8; size],
            content_type.to_string(),
            "test".to_string(),
            size as u64,
            PreviewDataMetadata::new(),
        )
    }

    #[test]
    fn test_preview_cache_key() {
        let path = PathBuf::from("/test/file.jpg");
        let time = SystemTime::now();
        let key = PreviewCacheKey::new(path.clone(), time);
        
        assert_eq!(key.path, path);
        assert_eq!(key.modified_time, time);
    }

    #[test]
    fn test_cached_preview_data_memory_size() {
        let data = create_test_data(1000, "image/jpeg");
        let size = data.memory_size();
        
        // Should be data size + strings + metadata + overhead
        assert!(size > 1000); // At least the data size
        assert!(size < 1200); // Not too much overhead
    }

    #[test]
    fn test_lru_cache_basic_operations() {
        let config = PreviewCacheConfig::default();
        let mut cache = PreviewLRUCache::new(config);
        
        let key1 = create_test_key("/test/file1.jpg");
        let data1 = create_test_data(1000, "image/jpeg");
        
        // Test put and get
        cache.put(key1.clone(), data1.clone()).unwrap();
        let retrieved = cache.get(&key1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data, data1.data);
        
        // Test stats
        let stats = cache.stats();
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut config = PreviewCacheConfig::default();
        config.max_entries = 2; // Small cache for testing
        config.max_memory_bytes = 10000; // Generous memory limit
        
        let mut cache = PreviewLRUCache::new(config);
        
        let key1 = create_test_key("/test/file1.jpg");
        let key2 = create_test_key("/test/file2.jpg");
        let key3 = create_test_key("/test/file3.jpg");
        
        let data1 = create_test_data(1000, "image/jpeg");
        let data2 = create_test_data(1000, "image/jpeg");
        let data3 = create_test_data(1000, "image/jpeg");
        
        // Fill cache to capacity
        cache.put(key1.clone(), data1).unwrap();
        cache.put(key2.clone(), data2).unwrap();
        
        // Access key1 to make it more recently used
        cache.get(&key1);
        
        // Add third item - should evict key2 (least recently used)
        cache.put(key3.clone(), data3).unwrap();
        
        assert!(cache.get(&key1).is_some()); // Still there
        assert!(cache.get(&key2).is_none());  // Evicted
        assert!(cache.get(&key3).is_some()); // Newly added
        
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.evictions, 1);
    }

    #[test]
    fn test_memory_limit_enforcement() {
        let mut config = PreviewCacheConfig::default();
        config.max_entries = 10; // High entry limit
        config.max_memory_bytes = 2000; // Low memory limit
        
        let mut cache = PreviewLRUCache::new(config);
        
        let key1 = create_test_key("/test/file1.jpg");
        let key2 = create_test_key("/test/file2.jpg");
        let key3 = create_test_key("/test/file3.jpg");
        
        let data1 = create_test_data(800, "image/jpeg");
        let data2 = create_test_data(800, "image/jpeg");
        let data3 = create_test_data(800, "image/jpeg");
        
        // Add items that fit within memory
        cache.put(key1.clone(), data1).unwrap();
        cache.put(key2.clone(), data2).unwrap();
        
        // Third item should trigger eviction due to memory limit
        cache.put(key3.clone(), data3).unwrap();
        
        let stats = cache.stats();
        assert!(stats.memory_bytes <= 2000);
        assert!(stats.evictions > 0);
    }

    #[test]
    fn test_data_too_large_error() {
        let mut config = PreviewCacheConfig::default();
        config.max_single_item_bytes = 1000;
        
        let mut cache = PreviewLRUCache::new(config);
        
        let key = create_test_key("/test/large_file.jpg");
        let large_data = create_test_data(2000, "image/jpeg");
        
        let result = cache.put(key, large_data);
        assert!(matches!(result, Err(PreviewCacheError::DataTooLarge { .. })));
    }

    #[test]
    fn test_cache_cleanup() {
        let config = PreviewCacheConfig::default();
        let mut cache = PreviewLRUCache::new(config);
        
        let key1 = create_test_key("/test/file1.jpg");
        let key2 = create_test_key("/test/file2.jpg");
        
        let data1 = create_test_data(1000, "image/jpeg");
        let mut data2 = create_test_data(1000, "image/jpeg");
        
        // Make data2 old
        data2.generated_at = SystemTime::now() - Duration::from_secs(48 * 3600); // 48 hours ago
        
        cache.put(key1.clone(), data1).unwrap();
        cache.put(key2.clone(), data2).unwrap();
        
        // Clean up old entries (older than 24 hours)
        let cleaned = cache.cleanup_old();
        assert_eq!(cleaned, 1);
        
        // Only fresh entry should remain
        assert!(cache.get(&key1).is_some());
        assert!(cache.get(&key2).is_none());
    }

    #[test]
    fn test_thread_safe_cache() {
        let cache = ThreadSafePreviewCache::new(PreviewCacheConfig::default());
        let cache_clone = cache.clone();
        
        let key = create_test_key("/test/thread_test.jpg");
        let key_clone = key.clone();
        let data = create_test_data(1000, "image/jpeg");
        
        // Test from different thread
        let handle = thread::spawn(move || {
            cache_clone.put(key_clone, data).unwrap();
        });
        
        handle.join().unwrap();
        
        // Should be able to retrieve from main thread
        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_cache_statistics() {
        let config = PreviewCacheConfig::default();
        let mut cache = PreviewLRUCache::new(config);
        
        let key1 = create_test_key("/test/stats1.jpg");
        let key2 = create_test_key("/test/stats2.jpg");
        
        let data1 = create_test_data(1000, "image/jpeg");
        let data2 = create_test_data(1000, "image/jpeg");
        
        // Add some data and test stats
        cache.put(key1.clone(), data1).unwrap();
        cache.put(key2.clone(), data2).unwrap();
        
        // Generate some hits and misses
        cache.get(&key1); // hit
        cache.get(&key2); // hit
        let _ = cache.get(&create_test_key("/test/missing.jpg")); // miss
        
        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 2.0 / 3.0);
        assert!(stats.memory_bytes > 2000); // Should include data + overhead
        
        // Test memory usage percentage
        let usage_percent = cache.memory_usage_percent();
        assert!(usage_percent > 0.0);
        assert!(usage_percent < 100.0);
    }

    #[test]
    fn test_cache_needs_cleanup() {
        let mut config = PreviewCacheConfig::default();
        config.max_entries = 10;
        config.max_memory_bytes = 1000;
        
        let mut cache = PreviewLRUCache::new(config);
        
        // Initially should not need cleanup
        assert!(!cache.needs_cleanup());
        
        // Fill cache to >90% capacity
        for i in 0..9 {
            let key = create_test_key(&format!("/test/file{}.jpg", i));
            let data = create_test_data(100, "image/jpeg");
            cache.put(key, data).unwrap();
        }
        
        // Should now need cleanup
        assert!(cache.needs_cleanup());
    }
}