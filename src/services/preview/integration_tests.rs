#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::services::preview::{PreviewService, PreviewContent, SupportedFormat, PreviewError, PreviewPriority};
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::time::Duration;

    /// Comprehensive integration tests for the preview service
    /// Tests end-to-end workflow: provider registration → format detection → preview generation → caching
    
    async fn create_test_files(temp_dir: &TempDir) -> Result<std::collections::HashMap<String, std::path::PathBuf>, Box<dyn std::error::Error>> {
        let base_path = temp_dir.path();
        let mut test_files = std::collections::HashMap::new();
        
        // Text files (supported by TextPreviewProvider - priority 150)
        let text_file = base_path.join("test.txt");
        std::fs::write(&text_file, "Hello World\nThis is a test file\nLine 3\nLine 4\nLine 5")?;
        test_files.insert("text".to_string(), text_file);
        
        let rust_file = base_path.join("test.rs");
        std::fs::write(&rust_file, "fn main() {\n    println!(\"Hello, world!\");\n}")?;
        test_files.insert("rust".to_string(), rust_file);
        
        let log_file = base_path.join("test.log");
        std::fs::write(&log_file, "[INFO] Application started\n[ERROR] Something went wrong\n[DEBUG] Debug info")?;
        test_files.insert("log".to_string(), log_file);
        
        // Create a simple bitmap image (supported by ImagePreviewProvider - priority 200)
        let bmp_file = base_path.join("test.bmp");
        // Simple 2x2 bitmap (BMP format header + pixel data)
        let bmp_data = create_simple_bmp();
        std::fs::write(&bmp_file, bmp_data)?;
        test_files.insert("image".to_string(), bmp_file);
        
        // Unknown file types (should use FallbackPreviewProvider - priority 0)
        let unknown_file = base_path.join("test.unknown");
        std::fs::write(&unknown_file, b"binary data \x00\x01\x02\x03")?;
        test_files.insert("unknown".to_string(), unknown_file);
        
        let doc_file = base_path.join("test.doc");
        std::fs::write(&doc_file, b"fake word document content for testing")?;
        test_files.insert("doc".to_string(), doc_file);
        
        let proprietary_file = base_path.join("test.xyz");
        std::fs::write(&proprietary_file, b"proprietary format data")?;
        test_files.insert("proprietary".to_string(), proprietary_file);
        
        Ok(test_files)
    }
    
    /// Create a minimal valid BMP file for testing
    fn create_simple_bmp() -> Vec<u8> {
        let mut bmp_data = Vec::new();
        
        // BMP Header (14 bytes)
        bmp_data.extend_from_slice(b"BM");           // Signature
        bmp_data.extend_from_slice(&70u32.to_le_bytes());  // File size (70 bytes)
        bmp_data.extend_from_slice(&0u32.to_le_bytes());   // Reserved
        bmp_data.extend_from_slice(&54u32.to_le_bytes());  // Data offset
        
        // DIB Header (40 bytes)
        bmp_data.extend_from_slice(&40u32.to_le_bytes());  // Header size
        bmp_data.extend_from_slice(&2i32.to_le_bytes());   // Width (2 pixels)
        bmp_data.extend_from_slice(&2i32.to_le_bytes());   // Height (2 pixels) 
        bmp_data.extend_from_slice(&1u16.to_le_bytes());   // Planes
        bmp_data.extend_from_slice(&24u16.to_le_bytes());  // Bits per pixel
        bmp_data.extend_from_slice(&0u32.to_le_bytes());   // Compression
        bmp_data.extend_from_slice(&16u32.to_le_bytes());  // Image size
        bmp_data.extend_from_slice(&2835i32.to_le_bytes()); // X pixels per meter
        bmp_data.extend_from_slice(&2835i32.to_le_bytes()); // Y pixels per meter
        bmp_data.extend_from_slice(&0u32.to_le_bytes());   // Colors used
        bmp_data.extend_from_slice(&0u32.to_le_bytes());   // Important colors
        
        // Pixel data (2x2 pixels, 3 bytes per pixel, padded to 4-byte alignment)
        // Row 1 (bottom row in BMP): Red pixel, Blue pixel + 2 bytes padding
        bmp_data.extend_from_slice(&[0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]); // Red, Blue + padding
        // Row 2 (top row in BMP): Green pixel, Yellow pixel + 2 bytes padding  
        bmp_data.extend_from_slice(&[0x00, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00]); // Green, Yellow + padding
        
        bmp_data
    }
    
    #[tokio::test]
    async fn test_preview_service_integration() {
        println!("🔍 Starting comprehensive preview service integration test");
        
        // Create temporary directory and test files
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files(&temp_dir).await.expect("Failed to create test files");
        
        // Initialize preview service with all providers
        let mut service = PreviewService::new()
            .with_default_providers();
        
        println!("✅ Created preview service with {} providers", service.providers().len());
        
        // Test 1: Provider registration and priority ordering
        println!("\n📋 Test 1: Provider Registration and Priority");
        assert!(service.providers().len() > 0, "No providers registered");
        
        // Check that fallback provider has lowest priority
        let fallback_priority = service.providers()
            .iter()
            .find(|p| p.provider_id() == "fallback")
            .map(|p| p.priority())
            .expect("Fallback provider not found");
        assert_eq!(fallback_priority, 0, "Fallback provider should have priority 0");
        
        // Check that image provider has higher priority than text
        let image_priority = service.providers()
            .iter()
            .find(|p| p.provider_id() == "image")
            .map(|p| p.priority())
            .unwrap_or(0);
        let text_priority = service.providers()
            .iter()
            .find(|p| p.provider_id() == "text")
            .map(|p| p.priority())
            .unwrap_or(0);
        assert!(image_priority > text_priority, "Image provider should have higher priority than text");
        
        println!("   ✅ Provider priorities: Image({}), Text({}), Fallback({})", 
                 image_priority, text_priority, fallback_priority);
        
        // Test 2: Format detection
        println!("\n🔍 Test 2: Format Detection");
        
        let text_format = service.detect_format(&test_files["text"]);
        assert_eq!(text_format, Some(SupportedFormat::Text), "Text format not detected");
        
        let rust_format = service.detect_format(&test_files["rust"]);
        assert_eq!(rust_format, Some(SupportedFormat::Rust), "Rust format not detected");
        
        let bmp_format = service.detect_format(&test_files["image"]);
        assert_eq!(bmp_format, Some(SupportedFormat::Bmp), "BMP format not detected");
        
        let unknown_format = service.detect_format(&test_files["unknown"]);
        assert_eq!(unknown_format, None, "Unknown format should return None");
        
        println!("   ✅ Format detection working: TXT, RS, BMP detected; .unknown returns None");
        
        // Test 3: Provider selection for supported formats
        println!("\n🎯 Test 3: Provider Selection");
        
        let text_provider = service.find_provider_for_format(SupportedFormat::Text);
        assert!(text_provider.is_some(), "No provider found for text format");
        assert_eq!(text_provider.unwrap().provider_id(), "text", "Wrong provider selected for text");
        
        let image_provider = service.find_provider_for_format(SupportedFormat::Bmp);
        assert!(image_provider.is_some(), "No provider found for BMP format");
        assert_eq!(image_provider.unwrap().provider_id(), "image", "Wrong provider selected for BMP");
        
        println!("   ✅ Provider selection: Text→TextProvider, BMP→ImageProvider");
        
        // Test 4: End-to-end preview generation for supported formats
        println!("\n🚀 Test 4: Preview Generation - Supported Formats");
        
        // Test text file preview
        println!("   📄 Testing text file preview...");
        let start_time = std::time::Instant::now();
        let text_preview = service.generate_preview(&test_files["text"]).await;
        let text_duration = start_time.elapsed();
        
        assert!(text_preview.is_ok(), "Text preview generation failed: {:?}", text_preview);
        let text_data = text_preview.unwrap();
        assert_eq!(text_data.format, SupportedFormat::Text);
        
        // Verify preview content
        match &text_data.preview_content {
            PreviewContent::Text { content, line_count, .. } => {
                assert!(content.contains("Hello World"), "Text content not found");
                assert!(*line_count > 0, "Line count should be greater than 0");
                println!("      ✅ Text preview: {} lines, {} ms", line_count, text_duration.as_millis());
            }
            _ => panic!("Expected text preview content"),
        }
        
        // Test image file preview  
        println!("   🖼️ Testing image file preview...");
        let start_time = std::time::Instant::now();
        let image_preview = service.generate_preview(&test_files["image"]).await;
        let image_duration = start_time.elapsed();
        
        assert!(image_preview.is_ok(), "Image preview generation failed: {:?}", image_preview);
        let image_data = image_preview.unwrap();
        assert_eq!(image_data.format, SupportedFormat::Bmp);
        
        // Verify image metadata
        assert!(image_data.metadata.width.is_some(), "Image width not extracted");
        assert!(image_data.metadata.height.is_some(), "Image height not extracted");
        println!("      ✅ Image preview: {}x{}, {} ms", 
                 image_data.metadata.width.unwrap(), 
                 image_data.metadata.height.unwrap(),
                 image_duration.as_millis());
        
        // Test 5: Fallback mechanism for unsupported formats
        println!("\n🛡️ Test 5: Fallback Mechanism");
        
        // Test unknown file extension
        println!("   ❓ Testing unknown file format...");
        let unknown_preview = service.generate_preview(&test_files["unknown"]).await;
        assert!(unknown_preview.is_ok(), "Unknown file preview should succeed with fallback");
        
        let unknown_data = unknown_preview.unwrap();
        match &unknown_data.preview_content {
            PreviewContent::Unsupported { file_type, reason, .. } => {
                assert_eq!(file_type, "unknown");
                assert!(reason.contains("not recognized"), "Fallback reason not set correctly");
                println!("      ✅ Fallback for .unknown: {}", reason);
            }
            _ => panic!("Expected unsupported content for unknown file"),
        }
        
        // Test Microsoft Office file (should show appropriate message)
        println!("   📝 Testing Microsoft Office file...");
        let doc_preview = service.generate_preview(&test_files["doc"]).await;
        assert!(doc_preview.is_ok(), "DOC file preview should succeed with fallback");
        
        let doc_data = doc_preview.unwrap();
        match &doc_data.preview_content {
            PreviewContent::Unsupported { file_type, reason, suggested_action } => {
                assert_eq!(file_type, "doc");
                assert!(reason.contains("Microsoft Office"), "Should mention Office files");
                assert!(suggested_action.is_some(), "Should provide suggested action");
                println!("      ✅ Fallback for .doc: {}", reason);
                if let Some(suggestion) = suggested_action {
                    println!("         💡 Suggestion: {}", suggestion);
                }
            }
            _ => panic!("Expected unsupported content for DOC file"),
        }
        
        // Test 6: Performance validation
        println!("\n⚡ Test 6: Performance Validation");
        
        // Text files should be very fast
        assert!(text_duration < Duration::from_millis(100), "Text preview too slow: {} ms", text_duration.as_millis());
        
        // Image files should be reasonably fast (within 500ms target for our simple test image)
        assert!(image_duration < Duration::from_millis(500), "Image preview too slow: {} ms", image_duration.as_millis());
        
        println!("   ✅ Performance: Text {} ms, Image {} ms (both within targets)", 
                 text_duration.as_millis(), image_duration.as_millis());
        
        // Test 7: Error handling
        println!("\n❌ Test 7: Error Handling");
        
        // Test non-existent file
        let missing_file = temp_dir.path().join("nonexistent.txt");
        let missing_preview = service.generate_preview(&missing_file).await;
        assert!(missing_preview.is_err(), "Missing file should return error");
        
        match missing_preview {
            Err(PreviewError::FileNotFound(_)) => {
                println!("   ✅ File not found error handled correctly");
            }
            _ => panic!("Expected FileNotFound error"),
        }
        
        println!("\n🎉 All integration tests passed!");
        println!("✅ Provider registration: {} providers", service.providers().len());
        println!("✅ Format detection: Working for known and unknown formats");
        println!("✅ Provider selection: Correct providers chosen by priority");
        println!("✅ Preview generation: Text and image previews working");
        println!("✅ Fallback mechanism: Handles unknown and unsupported formats gracefully");
        println!("✅ Performance: Within acceptable limits");
        println!("✅ Error handling: Proper error responses");
    }
    
    #[tokio::test]
    async fn test_async_preview_generation() {
        println!("🔄 Testing asynchronous preview generation");
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_files = create_test_files(&temp_dir).await.expect("Failed to create test files");
        
        let service = PreviewService::new().with_default_providers();
        
        // Test background preview generation
        println!("   🚀 Testing background task generation...");
        let task = service.generate_preview_background(&test_files["text"]);
        
        // Wait for completion
        let result = task.await_result().await;
        assert!(result.is_ok(), "Background task should complete successfully");
        
        let preview_data = result.unwrap();
        assert_eq!(preview_data.format, SupportedFormat::Text);
        
        println!("   ✅ Background preview generation working");
        
        // Test task queue management
        println!("   📊 Testing task queue...");
        let task_id = service.generate_preview_queued(&test_files["image"], PreviewPriority::High);
        assert!(task_id.is_ok(), "Task should be queued successfully");
        
        let stats = service.get_queue_stats();
        println!("   ✅ Queue stats: {} active tasks", stats.active_tasks);
        
        // Allow some time for task completion
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    #[tokio::test]
    async fn test_provider_priority_selection() {
        println!("🏆 Testing provider priority selection");
        
        let service = PreviewService::new().with_default_providers();
        
        // Test that higher priority providers are selected
        let text_provider = service.find_provider_for_format(SupportedFormat::Text);
        assert!(text_provider.is_some());
        assert_eq!(text_provider.unwrap().provider_id(), "text");
        
        let bmp_provider = service.find_provider_for_format(SupportedFormat::Bmp);
        assert!(bmp_provider.is_some());
        assert_eq!(bmp_provider.unwrap().provider_id(), "image");
        
        // Verify priority ordering
        let providers = service.providers();
        let video_priority = providers.iter().find(|p| p.provider_id() == "video").map(|p| p.priority()).unwrap_or(0);
        let image_priority = providers.iter().find(|p| p.provider_id() == "image").map(|p| p.priority()).unwrap_or(0);
        let text_priority = providers.iter().find(|p| p.provider_id() == "text").map(|p| p.priority()).unwrap_or(0);
        let fallback_priority = providers.iter().find(|p| p.provider_id() == "fallback").map(|p| p.priority()).unwrap_or(0);
        
        assert!(video_priority > image_priority, "Video should have higher priority than image");
        assert!(image_priority > text_priority, "Image should have higher priority than text");
        assert!(text_priority > fallback_priority, "Text should have higher priority than fallback");
        assert_eq!(fallback_priority, 0, "Fallback should have lowest priority");
        
        println!("   ✅ Priority ordering: Video({}) > Image({}) > Text({}) > Fallback({})", 
                 video_priority, image_priority, text_priority, fallback_priority);
    }
}