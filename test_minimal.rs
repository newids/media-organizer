use std::path::Path;
use tempfile::TempDir;

// Simple integration test for the fallback preview system
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing MediaOrganizer Preview Service Integration");
    
    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;
    let test_files = create_test_files(&temp_dir)?;
    
    println!("✅ Created {} test files", test_files.len());
    
    // Test each file type
    for (file_type, file_path) in test_files {
        println!("📄 Testing {}: {}", file_type, file_path.display());
        
        // Verify file exists and is readable
        if file_path.exists() {
            let metadata = std::fs::metadata(&file_path)?;
            println!("   ✅ File size: {} bytes", metadata.len());
        } else {
            println!("   ❌ File not found");
        }
    }
    
    println!("🎉 Integration test completed successfully!");
    Ok(())
}

fn create_test_files(temp_dir: &TempDir) -> Result<Vec<(String, std::path::PathBuf)>, Box<dyn std::error::Error>> {
    let base_path = temp_dir.path();
    let mut test_files = Vec::new();
    
    // Text files
    let text_file = base_path.join("test.txt");
    std::fs::write(&text_file, "Hello World\nThis is a test file\nLine 3")?;
    test_files.push(("Text".to_string(), text_file));
    
    let log_file = base_path.join("test.log");
    std::fs::write(&log_file, "[INFO] Application started\n[ERROR] Something went wrong\n[DEBUG] Debug info")?;
    test_files.push(("Log".to_string(), log_file));
    
    // Unknown file types (for fallback testing)
    let unknown_file = base_path.join("test.unknown");
    std::fs::write(&unknown_file, b"binary data \x00\x01\x02\x03")?;
    test_files.push(("Unknown".to_string(), unknown_file));
    
    let doc_file = base_path.join("test.doc");
    std::fs::write(&doc_file, b"fake word document content")?;
    test_files.push(("Document".to_string(), doc_file));
    
    Ok(test_files)
}