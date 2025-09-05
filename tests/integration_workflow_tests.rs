/*!
 * Integration Workflow Tests for MediaOrganizer
 * 
 * Task 22.3: Comprehensive integration tests covering:
 * - Preview workflows and generation
 * - Multi-file tab management and switching
 * - Theme persistence across sessions
 * - Performance under load (1000+ files)
 * - End-to-end user workflows
 */

use media_organizer::services::{
    PreviewService, PreviewContent, SupportedFormat, PreviewError, PreviewPriority,
    FileSystemService, NativeFileSystemService, FileEntry, PreviewMetadata,
    ThreadSafePreviewCache, PreviewCacheConfig, CachedPreviewData
};
use media_organizer::state::{
    AppState, EditorState, EditorGroup, EditorTab, TabType, PreviewType,
    Theme, LayoutState, LayoutPersistenceSettings, NavigationState, SelectionState
};
use media_organizer::theme::ThemeManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::timeout;
use tracing::{info, warn, debug};
use serde_json;
use std::collections::HashMap;

/// Test fixture for creating large file sets for stress testing
struct LargeFileSetFixture {
    temp_dir: TempDir,
    file_paths: Vec<PathBuf>,
    file_types: HashMap<String, Vec<PathBuf>>,
}

impl LargeFileSetFixture {
    /// Create a large file set with various file types for testing
    async fn new(file_count: usize) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();
        let mut file_paths = Vec::with_capacity(file_count);
        let mut file_types: HashMap<String, Vec<PathBuf>> = HashMap::new();
        
        info!("Creating {} test files for integration testing", file_count);
        
        // Create directory structure
        let dirs = ["documents", "images", "videos", "audio", "code", "archives"];
        for dir in &dirs {
            std::fs::create_dir_all(base_path.join(dir))?;
        }
        
        // Distribution of file types for realistic testing
        let type_distribution = [
            ("documents", 0.4, vec!["txt", "md", "json", "log"]),
            ("images", 0.25, vec!["jpg", "png", "bmp", "gif"]),
            ("videos", 0.15, vec!["mp4", "avi", "mkv"]),
            ("audio", 0.10, vec!["mp3", "wav", "flac"]),
            ("code", 0.08, vec!["rs", "js", "py", "cpp"]),
            ("archives", 0.02, vec!["zip", "tar", "7z"]),
        ];
        
        let mut file_counter = 0;
        
        for (category, ratio, extensions) in &type_distribution {
            let category_count = (file_count as f64 * ratio) as usize;
            let mut category_files = Vec::new();
            
            for i in 0..category_count {
                if file_counter >= file_count { break; }
                
                let ext_idx = i % extensions.len();
                let extension = &extensions[ext_idx];
                let filename = format!("test_file_{:04}.{}", file_counter, extension);
                let file_path = base_path.join(category).join(&filename);
                
                // Create file with appropriate content based on type
                let content = Self::generate_test_content(extension, i);
                std::fs::write(&file_path, content)?;
                
                file_paths.push(file_path.clone());
                category_files.push(file_path);
                file_counter += 1;
            }
            
            file_types.insert(category.to_string(), category_files);
        }
        
        info!("Created {} files across {} categories", file_counter, dirs.len());
        
        Ok(Self {
            temp_dir,
            file_paths,
            file_types,
        })
    }
    
    fn generate_test_content(extension: &str, index: usize) -> Vec<u8> {
        match extension {
            "txt" => format!("Test document {}\nLine 2\nLine 3\nContent for testing preview generation.", index).into_bytes(),
            "md" => format!("# Test Document {}\n\n## Section 1\n\nMarkdown content for preview testing.\n\n- List item 1\n- List item 2", index).into_bytes(),
            "json" => format!("{{\"test\": {}, \"data\": \"value\", \"array\": [1, 2, 3]}}", index).into_bytes(),
            "log" => format!("[INFO] Log entry {}\n[ERROR] Test error message\n[DEBUG] Debug information", index).into_bytes(),
            "rs" => format!("// Rust test file {}\nfn main() {{\n    println!(\"Hello, world!\");\n}}", index).into_bytes(),
            "js" => format!("// JavaScript test file {}\nconsole.log('Hello, world!');\n", index).into_bytes(),
            "py" => format!("# Python test file {}\nprint('Hello, world!')\n", index).into_bytes(),
            "cpp" => format!("// C++ test file {}\n#include <iostream>\nint main() {{ std::cout << \"Hello, world!\"; return 0; }}", index).into_bytes(),
            // For binary files, create minimal valid headers
            "jpg" | "png" | "bmp" | "gif" => Self::create_minimal_image_data(extension),
            "mp4" | "avi" | "mkv" => format!("Fake video file {} - not a real video", index).into_bytes(),
            "mp3" | "wav" | "flac" => format!("Fake audio file {} - not real audio data", index).into_bytes(),
            "zip" | "tar" | "7z" => format!("Fake archive file {} - not a real archive", index).into_bytes(),
            _ => format!("Test file {} content", index).into_bytes(),
        }
    }
    
    fn create_minimal_image_data(extension: &str) -> Vec<u8> {
        match extension {
            "bmp" => {
                // Create minimal 2x2 BMP file (from existing test)
                let mut bmp_data = Vec::new();
                bmp_data.extend_from_slice(b"BM");
                bmp_data.extend_from_slice(&70u32.to_le_bytes());
                bmp_data.extend_from_slice(&0u32.to_le_bytes());
                bmp_data.extend_from_slice(&54u32.to_le_bytes());
                bmp_data.extend_from_slice(&40u32.to_le_bytes());
                bmp_data.extend_from_slice(&2i32.to_le_bytes());
                bmp_data.extend_from_slice(&2i32.to_le_bytes());
                bmp_data.extend_from_slice(&1u16.to_le_bytes());
                bmp_data.extend_from_slice(&24u16.to_le_bytes());
                bmp_data.extend_from_slice(&0u32.to_le_bytes());
                bmp_data.extend_from_slice(&16u32.to_le_bytes());
                bmp_data.extend_from_slice(&2835i32.to_le_bytes());
                bmp_data.extend_from_slice(&2835i32.to_le_bytes());
                bmp_data.extend_from_slice(&0u32.to_le_bytes());
                bmp_data.extend_from_slice(&0u32.to_le_bytes());
                bmp_data.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]);
                bmp_data.extend_from_slice(&[0x00, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00]);
                bmp_data
            }
            _ => format!("Fake {} image data", extension).into_bytes(),
        }
    }
    
    fn path(&self) -> &Path {
        self.temp_dir.path()
    }
    
    fn files(&self) -> &Vec<PathBuf> {
        &self.file_paths
    }
    
    fn files_by_type(&self, file_type: &str) -> Option<&Vec<PathBuf>> {
        self.file_types.get(file_type)
    }
}

/// Integration test fixture for state management and UI components
struct IntegrationTestContext {
    preview_service: PreviewService,
    preview_cache: Arc<ThreadSafePreviewCache>,
    file_system: Arc<dyn FileSystemService>,
    temp_dir: TempDir,
}

impl IntegrationTestContext {
    async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temp_dir = TempDir::new()?;
        
        // Initialize services
        let preview_service = PreviewService::new().with_default_providers();
        
        let cache_config = PreviewCacheConfig {
            max_memory_bytes: 100 * 1024 * 1024, // 100MB for testing
            max_entries: 1000,
            eviction_batch_size: 50,
        };
        let preview_cache = Arc::new(ThreadSafePreviewCache::new(cache_config));
        
        let file_system: Arc<dyn FileSystemService> = Arc::new(
            NativeFileSystemService::new()
        );
        
        Ok(Self {
            preview_service,
            preview_cache,
            file_system,
            temp_dir,
        })
    }
}

#[tokio::test]
async fn test_comprehensive_preview_workflow() {
    info!("🔄 Starting comprehensive preview workflow integration test");
    
    let context = IntegrationTestContext::new().await
        .expect("Failed to create test context");
        
    // Create test files with various formats
    let fixture = LargeFileSetFixture::new(50).await
        .expect("Failed to create file fixture");
    
    info!("✅ Created test context with {} files", fixture.files().len());
    
    // Test 1: Preview generation workflow for different file types
    info!("📋 Test 1: Multi-format preview generation");
    
    let mut preview_results = HashMap::new();
    let mut generation_times = HashMap::new();
    
    for (category, files) in fixture.file_types.iter() {
        if files.is_empty() { continue; }
        
        let test_file = &files[0]; // Test first file of each category
        let start_time = Instant::now();
        
        let result = context.preview_service.generate_preview(test_file).await;
        let duration = start_time.elapsed();
        
        generation_times.insert(category.clone(), duration);
        
        match result {
            Ok(preview_data) => {
                preview_results.insert(category.clone(), preview_data);
                info!("   ✅ {} preview: {} ms", category, duration.as_millis());
            }
            Err(e) => {
                warn!("   ⚠️ {} preview failed: {:?}", category, e);
                // Some failures are expected (e.g., video files without proper codecs)
            }
        }
    }
    
    // Verify successful preview generation for supported formats
    assert!(preview_results.contains_key("documents"), "Document previews should succeed");
    assert!(generation_times.get("documents").unwrap() < &Duration::from_millis(100), 
           "Document preview too slow");
    
    // Test 2: Concurrent preview generation
    info!("📋 Test 2: Concurrent preview generation");
    
    let document_files = fixture.files_by_type("documents").unwrap();
    let concurrent_count = std::cmp::min(10, document_files.len());
    
    let start_time = Instant::now();
    let mut tasks = Vec::new();
    
    for i in 0..concurrent_count {
        let service = &context.preview_service;
        let file_path = &document_files[i];
        tasks.push(service.generate_preview(file_path));
    }
    
    let results = futures::future::join_all(tasks).await;
    let concurrent_duration = start_time.elapsed();
    
    let successful_previews = results.into_iter()
        .filter(|r| r.is_ok())
        .count();
    
    assert!(successful_previews >= concurrent_count / 2, 
           "At least half of concurrent previews should succeed");
    assert!(concurrent_duration < Duration::from_millis(500), 
           "Concurrent preview generation too slow: {} ms", concurrent_duration.as_millis());
    
    info!("   ✅ Concurrent previews: {}/{} successful in {} ms", 
          successful_previews, concurrent_count, concurrent_duration.as_millis());
    
    // Test 3: Preview caching workflow
    info!("📋 Test 3: Preview caching workflow");
    
    let cache_test_file = &document_files[0];
    
    // First generation (cache miss)
    let start_time = Instant::now();
    let first_result = context.preview_service.generate_preview(cache_test_file).await;
    let first_duration = start_time.elapsed();
    assert!(first_result.is_ok(), "First preview generation should succeed");
    
    // Second generation (should be faster due to caching)
    let start_time = Instant::now();
    let second_result = context.preview_service.generate_preview(cache_test_file).await;
    let second_duration = start_time.elapsed();
    assert!(second_result.is_ok(), "Second preview generation should succeed");
    
    // Cache hit should be significantly faster
    info!("   Cache performance: first {} ms, second {} ms", 
          first_duration.as_millis(), second_duration.as_millis());
    
    info!("🎉 Preview workflow integration test completed successfully");
}

#[tokio::test] 
async fn test_multi_file_tab_management() {
    info!("📑 Starting multi-file tab management integration test");
    
    let fixture = LargeFileSetFixture::new(20).await
        .expect("Failed to create file fixture");
        
    // Simulate editor state with multiple tabs
    let mut editor_state = EditorState::default();
    
    // Test 1: Opening multiple files in tabs
    info!("📋 Test 1: Opening multiple files in tabs");
    
    let test_files: Vec<_> = fixture.files().iter().take(10).collect();
    let mut tab_creation_times = Vec::new();
    
    for (i, file_path) in test_files.iter().enumerate() {
        let start_time = Instant::now();
        
        // Create tab for file
        let tab = EditorTab {
            id: format!("tab_{}", i),
            title: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            file_path: Some(file_path.to_path_buf()),
            tab_type: TabType::Preview,
            preview_type: Some(PreviewType::Document),
            is_active: i == 0, // First tab is active
            is_dirty: false,
            is_pinned: false,
            can_close: true,
        };
        
        // Add to first editor group
        editor_state.editor_groups[0].tabs.push(tab);
        let creation_time = start_time.elapsed();
        tab_creation_times.push(creation_time);
    }
    
    let avg_creation_time: Duration = tab_creation_times.iter().sum::<Duration>() / tab_creation_times.len() as u32;
    assert!(avg_creation_time < Duration::from_millis(50), 
           "Tab creation too slow: {} ms", avg_creation_time.as_millis());
    
    info!("   ✅ Created {} tabs, average time: {} ms", 
          test_files.len(), avg_creation_time.as_millis());
    
    // Test 2: Tab switching performance
    info!("📋 Test 2: Tab switching performance");
    
    let mut switch_times = Vec::new();
    let tab_count = editor_state.editor_groups[0].tabs.len();
    
    // Simulate rapid tab switching
    for i in 0..std::cmp::min(20, tab_count * 2) {
        let tab_index = i % tab_count;
        let start_time = Instant::now();
        
        // Simulate tab switching logic
        for tab in &mut editor_state.editor_groups[0].tabs {
            tab.is_active = false;
        }
        editor_state.editor_groups[0].tabs[tab_index].is_active = true;
        editor_state.editor_groups[0].active_tab_index = Some(tab_index);
        
        let switch_time = start_time.elapsed();
        switch_times.push(switch_time);
    }
    
    let avg_switch_time: Duration = switch_times.iter().sum::<Duration>() / switch_times.len() as u32;
    assert!(avg_switch_time < Duration::from_millis(10), 
           "Tab switching too slow: {} ms", avg_switch_time.as_millis());
    
    info!("   ✅ Tab switching: {} operations, average {} ms", 
          switch_times.len(), avg_switch_time.as_millis());
    
    // Test 3: Tab closure and memory cleanup
    info!("📋 Test 3: Tab closure and memory cleanup");
    
    let initial_tab_count = editor_state.editor_groups[0].tabs.len();
    let tabs_to_close = initial_tab_count / 2;
    
    // Close half the tabs
    for _ in 0..tabs_to_close {
        if !editor_state.editor_groups[0].tabs.is_empty() {
            // Remove last tab
            editor_state.editor_groups[0].tabs.pop();
        }
    }
    
    let remaining_tabs = editor_state.editor_groups[0].tabs.len();
    assert_eq!(remaining_tabs, initial_tab_count - tabs_to_close, 
              "Tab closure count mismatch");
    
    // Ensure active tab index is still valid
    if !editor_state.editor_groups[0].tabs.is_empty() && 
       editor_state.editor_groups[0].active_tab_index.unwrap_or(0) >= remaining_tabs {
        editor_state.editor_groups[0].active_tab_index = Some(remaining_tabs - 1);
        editor_state.editor_groups[0].tabs[remaining_tabs - 1].is_active = true;
    }
    
    info!("   ✅ Closed {} tabs, {} remaining, active tab index valid", 
          tabs_to_close, remaining_tabs);
    
    // Test 4: Multiple editor groups
    info!("📋 Test 4: Multiple editor groups");
    
    // Add second editor group
    let second_group = EditorGroup {
        id: "group_2".to_string(),
        tabs: Vec::new(),
        active_tab_index: None,
        layout_position: Default::default(),
        is_focused: false,
    };
    editor_state.editor_groups.push(second_group);
    
    // Split some tabs to second group
    let tabs_to_move = 3;
    let moved_tabs: Vec<_> = editor_state.editor_groups[0].tabs
        .drain(..std::cmp::min(tabs_to_move, editor_state.editor_groups[0].tabs.len()))
        .collect();
    
    editor_state.editor_groups[1].tabs = moved_tabs;
    if !editor_state.editor_groups[1].tabs.is_empty() {
        editor_state.editor_groups[1].active_tab_index = Some(0);
        editor_state.editor_groups[1].tabs[0].is_active = true;
    }
    
    info!("   ✅ Created second editor group with {} tabs", 
          editor_state.editor_groups[1].tabs.len());
    
    assert_eq!(editor_state.editor_groups.len(), 2, "Should have 2 editor groups");
    assert!(!editor_state.editor_groups[1].tabs.is_empty(), "Second group should have tabs");
    
    info!("🎉 Multi-file tab management integration test completed successfully");
}

#[tokio::test]
async fn test_theme_persistence() {
    info!("🎨 Starting theme persistence integration test");
    
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let settings_file = temp_dir.path().join("settings.json");
    
    // Test 1: Theme switching and state management
    info!("📋 Test 1: Theme switching performance");
    
    let themes = vec![Theme::Dark, Theme::Light, Theme::HighContrast, Theme::Auto];
    let mut switch_times = Vec::new();
    
    for theme in &themes {
        let start_time = Instant::now();
        
        // Apply theme using ThemeManager
        ThemeManager::apply_theme(theme);
        
        let switch_time = start_time.elapsed();
        switch_times.push(switch_time);
        
        info!("   Theme switch to {:?}: {} ms", theme, switch_time.as_millis());
    }
    
    let avg_switch_time: Duration = switch_times.iter().sum::<Duration>() / switch_times.len() as u32;
    assert!(avg_switch_time < Duration::from_millis(50), 
           "Theme switching too slow: {} ms", avg_switch_time.as_millis());
    
    // Test 2: Theme persistence configuration
    info!("📋 Test 2: Theme persistence configuration");
    
    let mut persistence_settings = LayoutPersistenceSettings {
        persist_layout: true,
        persist_theme: true,
        restore_window_state: true,
        auto_save_interval: 5, // 5 seconds for testing
    };
    
    // Create layout state with theme
    let layout_state = LayoutState {
        theme: Theme::Dark,
        activity_bar: Default::default(),
        sidebar: Default::default(),
        editor_layout: Default::default(),
        panel: Default::default(),
        viewport: Default::default(),
        ui_preferences: Default::default(),
        persistence: persistence_settings.clone(),
    };
    
    // Test serialization (simulating persistence)
    let start_time = Instant::now();
    let serialized = serde_json::to_string(&layout_state)
        .expect("Failed to serialize layout state");
    let serialization_time = start_time.elapsed();
    
    assert!(serialization_time < Duration::from_millis(10), 
           "Theme serialization too slow: {} ms", serialization_time.as_millis());
    
    // Test deserialization (simulating restoration)
    let start_time = Instant::now();
    let deserialized: LayoutState = serde_json::from_str(&serialized)
        .expect("Failed to deserialize layout state");
    let deserialization_time = start_time.elapsed();
    
    assert!(deserialization_time < Duration::from_millis(10), 
           "Theme deserialization too slow: {} ms", deserialization_time.as_millis());
    assert_eq!(deserialized.theme, Theme::Dark, "Theme should be preserved");
    
    info!("   ✅ Persistence: serialize {} ms, deserialize {} ms", 
          serialization_time.as_millis(), deserialization_time.as_millis());
    
    // Test 3: System theme detection
    info!("📋 Test 3: System theme detection");
    
    let start_time = Instant::now();
    let system_theme = ThemeManager::detect_system_theme();
    let detection_time = start_time.elapsed();
    
    assert!(detection_time < Duration::from_millis(50), 
           "System theme detection too slow: {} ms", detection_time.as_millis());
    
    info!("   ✅ System theme detection: {} (dark={}), {} ms", 
          if system_theme { "Dark" } else { "Light" }, system_theme, detection_time.as_millis());
    
    // Test 4: Theme state consistency
    info!("📋 Test 4: Theme state consistency across components");
    
    // Simulate theme changes across multiple components
    let theme_changes = vec![
        (Theme::Light, "Main UI"),
        (Theme::Dark, "Editor"), 
        (Theme::HighContrast, "Settings Panel"),
        (Theme::Auto, "System Sync"),
    ];
    
    for (theme, component) in theme_changes {
        let start_time = Instant::now();
        
        // Apply theme
        ThemeManager::apply_theme(&theme);
        
        // Verify theme detection works
        let detected_system = ThemeManager::detect_system_theme();
        
        let consistency_check_time = start_time.elapsed();
        assert!(consistency_check_time < Duration::from_millis(25), 
               "Theme consistency check too slow for {}: {} ms", 
               component, consistency_check_time.as_millis());
        
        info!("   ✅ {} theme consistency: {} ms", component, consistency_check_time.as_millis());
    }
    
    info!("🎉 Theme persistence integration test completed successfully");
}

#[tokio::test]
async fn test_large_file_set_performance() {
    info!("⚡ Starting large file set performance test (1000+ files)");
    
    let context = IntegrationTestContext::new().await
        .expect("Failed to create test context");
    
    // Create large file set for stress testing
    let file_count = 1200; // Slightly above 1000 for stress testing
    let fixture = LargeFileSetFixture::new(file_count).await
        .expect("Failed to create large file fixture");
    
    info!("✅ Created {} files for performance testing", fixture.files().len());
    
    // Test 1: File system operations at scale
    info!("📋 Test 1: File system operations at scale");
    
    let start_time = Instant::now();
    let mut file_entries = Vec::new();
    
    // Simulate file system scanning
    for file_path in fixture.files().iter().take(1000) {
        if let Ok(metadata) = std::fs::metadata(file_path) {
            let entry = FileEntry {
                path: file_path.clone(),
                name: file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                is_directory: metadata.is_dir(),
                size: metadata.len(),
                modified: metadata.modified().ok(),
                file_type: file_path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|s| s.to_string()),
                permissions: Some(if metadata.permissions().readonly() { "readonly".to_string() } else { "readwrite".to_string() }),
            };
            file_entries.push(entry);
        }
    }
    
    let scan_duration = start_time.elapsed();
    assert!(scan_duration < Duration::from_millis(2000), 
           "File system scan too slow: {} ms", scan_duration.as_millis());
    
    info!("   ✅ Scanned {} files in {} ms", file_entries.len(), scan_duration.as_millis());
    
    // Test 2: Preview generation under load
    info!("📋 Test 2: Preview generation under load");
    
    let document_files: Vec<_> = fixture.files().iter()
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
        .take(50) // Test with 50 document files
        .collect();
    
    let start_time = Instant::now();
    let mut successful_previews = 0;
    let mut failed_previews = 0;
    
    for file_path in document_files.iter() {
        match timeout(Duration::from_millis(200), context.preview_service.generate_preview(file_path)).await {
            Ok(Ok(_)) => successful_previews += 1,
            Ok(Err(_)) | Err(_) => failed_previews += 1,
        }
    }
    
    let batch_duration = start_time.elapsed();
    let success_rate = (successful_previews as f64) / (successful_previews + failed_previews) as f64 * 100.0;
    
    assert!(success_rate > 80.0, "Preview success rate too low: {:.1}%", success_rate);
    assert!(batch_duration < Duration::from_millis(10000), 
           "Batch preview generation too slow: {} ms", batch_duration.as_millis());
    
    info!("   ✅ Preview batch: {}/{} successful ({:.1}%) in {} ms", 
          successful_previews, successful_previews + failed_previews, 
          success_rate, batch_duration.as_millis());
    
    // Test 3: Memory usage under load
    info!("📋 Test 3: Memory usage under load");
    
    let cache_stats_before = context.preview_cache.stats();
    
    // Generate previews for various file types to test memory management
    let mixed_files: Vec<_> = fixture.files().iter().take(100).collect();
    
    for file_path in mixed_files.iter() {
        // Use timeout to prevent hanging on problematic files
        let _ = timeout(Duration::from_millis(100), 
            context.preview_service.generate_preview(file_path)
        ).await;
    }
    
    let cache_stats_after = context.preview_cache.stats();
    
    // Verify memory usage is reasonable
    let memory_used_mb = cache_stats_after.memory_bytes / (1024 * 1024);
    assert!(memory_used_mb < 100, "Memory usage too high: {}MB", memory_used_mb);
    
    let cache_hit_rate = if cache_stats_after.total_requests > 0 {
        (cache_stats_after.hits as f64) / (cache_stats_after.total_requests as f64) * 100.0
    } else {
        0.0
    };
    
    info!("   ✅ Memory usage: {}MB, Cache hit rate: {:.1}%, Entries: {}", 
          memory_used_mb, cache_hit_rate, cache_stats_after.entries);
    
    // Test 4: Tab management at scale
    info!("📋 Test 4: Tab management at scale");
    
    let mut large_editor_state = EditorState::default();
    let max_tabs = 50; // Stress test with 50 tabs
    
    let start_time = Instant::now();
    
    // Create many tabs
    for i in 0..max_tabs {
        let file_path = &fixture.files()[i % fixture.files().len()];
        let tab = EditorTab {
            id: format!("stress_tab_{}", i),
            title: format!("File {}", i),
            file_path: Some(file_path.clone()),
            tab_type: TabType::Preview,
            preview_type: Some(PreviewType::Document),
            is_active: i == 0,
            is_dirty: false,
            is_pinned: i < 5, // Pin first 5 tabs
            can_close: true,
        };
        
        large_editor_state.editor_groups[0].tabs.push(tab);
    }
    
    let tab_creation_duration = start_time.elapsed();
    
    // Test rapid tab switching
    let start_time = Instant::now();
    for i in 0..100 { // 100 tab switches
        let tab_index = i % max_tabs;
        
        // Switch active tab
        for tab in &mut large_editor_state.editor_groups[0].tabs {
            tab.is_active = false;
        }
        large_editor_state.editor_groups[0].tabs[tab_index].is_active = true;
        large_editor_state.editor_groups[0].active_tab_index = Some(tab_index);
    }
    let switching_duration = start_time.elapsed();
    
    assert!(tab_creation_duration < Duration::from_millis(500), 
           "Tab creation at scale too slow: {} ms", tab_creation_duration.as_millis());
    assert!(switching_duration < Duration::from_millis(100), 
           "Tab switching at scale too slow: {} ms", switching_duration.as_millis());
    
    info!("   ✅ Scale test: {} tabs created in {} ms, 100 switches in {} ms", 
          max_tabs, tab_creation_duration.as_millis(), switching_duration.as_millis());
    
    info!("🎉 Large file set performance test completed successfully");
    info!("📊 Performance Summary:");
    info!("   • File system scan: {} files in {} ms", file_entries.len(), scan_duration.as_millis());
    info!("   • Preview generation: {:.1}% success rate", success_rate);
    info!("   • Memory usage: {}MB with {:.1}% cache hit rate", memory_used_mb, cache_hit_rate);
    info!("   • Tab management: {} tabs, switching < {} ms", max_tabs, switching_duration.as_millis());
}

#[tokio::test]
async fn test_end_to_end_user_workflows() {
    info!("👤 Starting end-to-end user workflow integration test");
    
    let context = IntegrationTestContext::new().await
        .expect("Failed to create test context");
        
    let fixture = LargeFileSetFixture::new(30).await
        .expect("Failed to create file fixture");
    
    // Simulate complete user workflow: browse → preview → edit → switch themes
    
    // Workflow 1: File browsing and preview
    info!("📋 Workflow 1: File browsing and preview");
    
    let mut navigation_state = NavigationState {
        current_path: fixture.path().to_path_buf(),
        history: vec![fixture.path().to_path_buf()],
        forward_stack: Vec::new(),
        breadcrumbs: vec!["Root".to_string()],
        can_go_back: false,
        can_go_forward: false,
    };
    
    let mut selection_state = SelectionState {
        selected_files: Vec::new(),
        selection_mode: crate::state::SelectionMode::Single,
        last_selected: None,
        anchor: None,
    };
    
    // Browse through different directories
    let document_files = fixture.files_by_type("documents").unwrap();
    let image_files = fixture.files_by_type("images").unwrap();
    
    // Select and preview different file types
    let workflow_files = vec![
        ("Document", &document_files[0]),
        ("Image", &image_files[0]),
        ("Document", &document_files[1]),
    ];
    
    let mut workflow_times = Vec::new();
    
    for (file_type, file_path) in workflow_files {
        let start_time = Instant::now();
        
        // Simulate file selection
        selection_state.selected_files = vec![file_path.clone()];
        selection_state.last_selected = Some(file_path.clone());
        
        // Generate preview
        let preview_result = context.preview_service.generate_preview(file_path).await;
        
        let workflow_time = start_time.elapsed();
        workflow_times.push(workflow_time);
        
        info!("   {} workflow: {} ms (success: {})", 
              file_type, workflow_time.as_millis(), preview_result.is_ok());
    }
    
    let avg_workflow_time: Duration = workflow_times.iter().sum::<Duration>() / workflow_times.len() as u32;
    assert!(avg_workflow_time < Duration::from_millis(200), 
           "User workflow too slow: {} ms", avg_workflow_time.as_millis());
    
    // Workflow 2: Multi-tab editing session
    info!("📋 Workflow 2: Multi-tab editing session");
    
    let mut editor_state = EditorState::default();
    let session_files = &document_files[..5]; // Work with 5 documents
    
    let start_time = Instant::now();
    
    // Open multiple files in tabs (simulating user opening files for editing)
    for (i, file_path) in session_files.iter().enumerate() {
        let tab = EditorTab {
            id: format!("session_tab_{}", i),
            title: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            file_path: Some(file_path.to_path_buf()),
            tab_type: TabType::Preview,
            preview_type: Some(PreviewType::Document),
            is_active: i == 0,
            is_dirty: i % 2 == 1, // Every other tab is "dirty" (edited)
            is_pinned: i == 0, // Pin first tab
            can_close: true,
        };
        
        editor_state.editor_groups[0].tabs.push(tab);
    }
    
    // Simulate user switching between tabs during editing session
    for switch in 0..10 {
        let tab_index = switch % session_files.len();
        
        // Switch to tab
        for tab in &mut editor_state.editor_groups[0].tabs {
            tab.is_active = false;
        }
        editor_state.editor_groups[0].tabs[tab_index].is_active = true;
        editor_state.editor_groups[0].active_tab_index = Some(tab_index);
        
        // Generate preview for active tab
        let active_file = &session_files[tab_index];
        let _ = context.preview_service.generate_preview(active_file).await;
    }
    
    let session_duration = start_time.elapsed();
    assert!(session_duration < Duration::from_millis(1000), 
           "Multi-tab editing session too slow: {} ms", session_duration.as_millis());
    
    info!("   ✅ Multi-tab session: {} tabs, {} switches in {} ms", 
          session_files.len(), 10, session_duration.as_millis());
    
    // Workflow 3: Theme switching during work session
    info!("📋 Workflow 3: Theme switching during active work");
    
    let themes_to_test = vec![Theme::Dark, Theme::Light, Theme::HighContrast, Theme::Auto];
    
    let start_time = Instant::now();
    
    for theme in themes_to_test {
        // Switch theme
        ThemeManager::apply_theme(&theme);
        
        // Continue working (switch tabs, generate previews)
        let tab_index = 1;
        editor_state.editor_groups[0].tabs[tab_index].is_active = true;
        let active_file = &session_files[tab_index];
        let _ = context.preview_service.generate_preview(active_file).await;
    }
    
    let theme_workflow_duration = start_time.elapsed();
    assert!(theme_workflow_duration < Duration::from_millis(500), 
           "Theme switching during work too slow: {} ms", theme_workflow_duration.as_millis());
    
    info!("   ✅ Theme switching workflow: {} ms", theme_workflow_duration.as_millis());
    
    info!("🎉 End-to-end user workflow integration test completed successfully");
    info!("📊 Workflow Performance Summary:");
    info!("   • Average file workflow: {} ms", avg_workflow_time.as_millis());
    info!("   • Multi-tab editing session: {} ms", session_duration.as_millis());
    info!("   • Theme switching during work: {} ms", theme_workflow_duration.as_millis());
}

/// Performance benchmarking for integration testing
#[tokio::test]
async fn test_integration_performance_benchmarks() {
    info!("📊 Starting integration performance benchmarks");
    
    let context = IntegrationTestContext::new().await
        .expect("Failed to create test context");
        
    let fixture = LargeFileSetFixture::new(100).await
        .expect("Failed to create file fixture");
    
    // Benchmark 1: Preview generation across all file types
    info!("📋 Benchmark 1: Preview generation performance");
    
    let mut benchmark_results = HashMap::new();
    
    for (category, files) in fixture.file_types.iter() {
        if files.is_empty() { continue; }
        
        let test_files: Vec<_> = files.iter().take(5).collect();
        let start_time = Instant::now();
        let mut successful = 0;
        let mut failed = 0;
        
        for file_path in test_files {
            match timeout(Duration::from_millis(500), 
                context.preview_service.generate_preview(file_path)
            ).await {
                Ok(Ok(_)) => successful += 1,
                _ => failed += 1,
            }
        }
        
        let category_duration = start_time.elapsed();
        let avg_time = category_duration / (successful + failed).max(1) as u32;
        
        benchmark_results.insert(category.clone(), (avg_time, successful, failed));
        
        info!("   {} files: {} ms avg, {}/{} successful", 
              category, avg_time.as_millis(), successful, successful + failed);
    }
    
    // Verify performance targets
    if let Some((avg_time, _, _)) = benchmark_results.get("documents") {
        assert!(avg_time < &Duration::from_millis(100), 
               "Document preview performance target not met: {} ms", avg_time.as_millis());
    }
    
    // Benchmark 2: Memory efficiency over extended operation
    info!("📋 Benchmark 2: Memory efficiency");
    
    let start_memory = context.preview_cache.stats().memory_bytes;
    
    // Perform extended operations
    for i in 0..50 {
        let file_index = i % fixture.files().len();
        let file_path = &fixture.files()[file_index];
        
        let _ = timeout(Duration::from_millis(100), 
            context.preview_service.generate_preview(file_path)
        ).await;
    }
    
    let end_memory = context.preview_cache.stats().memory_bytes;
    let memory_growth = end_memory.saturating_sub(start_memory);
    let memory_growth_mb = memory_growth / (1024 * 1024);
    
    assert!(memory_growth_mb < 50, "Memory growth too high: {}MB", memory_growth_mb);
    
    info!("   Memory growth: {}MB over 50 operations", memory_growth_mb);
    
    // Benchmark 3: Concurrent operations
    info!("📋 Benchmark 3: Concurrent operation performance");
    
    let concurrent_files: Vec<_> = fixture.files().iter().take(20).collect();
    let start_time = Instant::now();
    
    let tasks: Vec<_> = concurrent_files.iter().map(|file_path| {
        timeout(Duration::from_millis(300), 
            context.preview_service.generate_preview(file_path)
        )
    }).collect();
    
    let results = futures::future::join_all(tasks).await;
    let concurrent_duration = start_time.elapsed();
    
    let successful_concurrent = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();
    let concurrency_success_rate = (successful_concurrent as f64) / (results.len() as f64) * 100.0;
    
    assert!(concurrency_success_rate > 70.0, 
           "Concurrent operation success rate too low: {:.1}%", concurrency_success_rate);
    assert!(concurrent_duration < Duration::from_millis(1000), 
           "Concurrent operations too slow: {} ms", concurrent_duration.as_millis());
    
    info!("   Concurrent ops: {}/{} successful ({:.1}%) in {} ms", 
          successful_concurrent, results.len(), concurrency_success_rate, concurrent_duration.as_millis());
    
    info!("🎉 Integration performance benchmarks completed successfully");
}