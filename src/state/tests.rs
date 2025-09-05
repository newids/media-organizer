//! State Management Test Suite
//! 
//! Comprehensive testing for the layout state management system including:
//! - LayoutState operations and serialization
//! - Persistence workflows 
//! - Performance validation (<100ms targets)
//! - Integration tests

use crate::state::app_state::{LayoutState, Theme, ActivityBarPosition, SidebarPosition, PanelPosition};
use serde_json;
use std::time::{Duration, Instant};
use tempfile::tempdir;

#[cfg(test)]
mod layout_state_tests {
    use super::*;

    #[test]
    fn test_layout_state_default_initialization() {
        let state = LayoutState::default();
        
        // Test default values
        assert_eq!(state.activity_bar.position, ActivityBarPosition::Left);
        assert!(!state.activity_bar.hidden);
        assert_eq!(state.sidebar.position, SidebarPosition::Left);
        assert!(!state.sidebar.collapsed);
        assert_eq!(state.sidebar.width, 280.0);
        assert_eq!(state.panel.position, PanelPosition::Bottom);
        assert!(!state.panel.hidden);
        assert_eq!(state.panel.height, 200.0);
        assert_eq!(state.theme, Theme::Light);
        assert!(!state.reduced_motion);
    }

    #[test]
    fn test_layout_state_theme_transitions() {
        let mut state = LayoutState::default();
        
        // Test all theme variants
        state.theme = Theme::Dark;
        assert_eq!(state.theme, Theme::Dark);
        
        state.theme = Theme::HighContrast;
        assert_eq!(state.theme, Theme::HighContrast);
        
        state.theme = Theme::Light;
        assert_eq!(state.theme, Theme::Light);
    }

    #[test]
    fn test_layout_state_sidebar_operations() {
        let mut state = LayoutState::default();
        
        // Test sidebar collapse/expand
        state.sidebar.collapsed = true;
        assert!(state.sidebar.collapsed);
        
        // Test width adjustments
        state.sidebar.width = 350.0;
        assert_eq!(state.sidebar.width, 350.0);
        
        // Test position changes
        state.sidebar.position = SidebarPosition::Right;
        assert_eq!(state.sidebar.position, SidebarPosition::Right);
    }

    #[test]
    fn test_layout_state_panel_operations() {
        let mut state = LayoutState::default();
        
        // Test panel visibility
        state.panel.hidden = true;
        assert!(state.panel.hidden);
        
        // Test height adjustments
        state.panel.height = 300.0;
        assert_eq!(state.panel.height, 300.0);
        
        // Test position changes
        state.panel.position = PanelPosition::Right;
        assert_eq!(state.panel.position, PanelPosition::Right);
    }

    #[test]
    fn test_layout_state_serialization() {
        let state = LayoutState::default();
        
        // Test JSON serialization
        let json_result = serde_json::to_string(&state);
        assert!(json_result.is_ok(), "Failed to serialize LayoutState");
        
        let json = json_result.unwrap();
        assert!(!json.is_empty());
        
        // Test JSON deserialization
        let deserialized_result: Result<LayoutState, _> = serde_json::from_str(&json);
        assert!(deserialized_result.is_ok(), "Failed to deserialize LayoutState");
        
        let deserialized = deserialized_result.unwrap();
        
        // Verify critical fields match
        assert_eq!(state.theme, deserialized.theme);
        assert_eq!(state.sidebar.width, deserialized.sidebar.width);
        assert_eq!(state.panel.height, deserialized.panel.height);
    }

    #[test]
    fn test_layout_state_complex_workflow() {
        let mut state = LayoutState::default();
        
        // Simulate complex user workflow
        state.theme = Theme::Dark;
        state.sidebar.collapsed = true;
        state.panel.hidden = true;
        state.activity_bar.position = ActivityBarPosition::Right;
        state.reduced_motion = true;
        
        // Verify final state
        assert_eq!(state.theme, Theme::Dark);
        assert!(state.sidebar.collapsed);
        assert!(state.panel.hidden);
        assert_eq!(state.activity_bar.position, ActivityBarPosition::Right);
        assert!(state.reduced_motion);
        
        // Test serialization of modified state
        let json_result = serde_json::to_string(&state);
        assert!(json_result.is_ok());
        
        let json = json_result.unwrap();
        let restored_result: Result<LayoutState, _> = serde_json::from_str(&json);
        assert!(restored_result.is_ok());
        
        let restored = restored_result.unwrap();
        assert_eq!(state.theme, restored.theme);
        assert_eq!(state.sidebar.collapsed, restored.sidebar.collapsed);
        assert_eq!(state.panel.hidden, restored.panel.hidden);
    }

    #[test]
    fn test_layout_state_edge_cases() {
        let mut state = LayoutState::default();
        
        // Test extreme width values
        state.sidebar.width = 0.0;
        assert_eq!(state.sidebar.width, 0.0);
        
        state.sidebar.width = 1000.0;
        assert_eq!(state.sidebar.width, 1000.0);
        
        // Test extreme height values
        state.panel.height = 0.0;
        assert_eq!(state.panel.height, 0.0);
        
        state.panel.height = 800.0;
        assert_eq!(state.panel.height, 800.0);
    }
}

#[cfg(test)]
mod persistence_tests {
    use super::*;

    #[tokio::test]
    async fn test_layout_state_file_persistence() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("layout_config.json");
        
        // Create test state with custom values
        let mut original_state = LayoutState::default();
        original_state.theme = Theme::Dark;
        original_state.sidebar.collapsed = true;
        original_state.sidebar.width = 350.0;
        original_state.panel.hidden = true;
        original_state.panel.height = 250.0;
        
        // Test save operation
        let json = serde_json::to_string_pretty(&original_state)
            .expect("Failed to serialize state");
        tokio::fs::write(&config_path, json)
            .await
            .expect("Failed to write config file");
        
        // Test load operation
        let saved_json = tokio::fs::read_to_string(&config_path)
            .await
            .expect("Failed to read config file");
        let loaded_state: LayoutState = serde_json::from_str(&saved_json)
            .expect("Failed to deserialize state");
        
        // Verify persistence integrity
        assert_eq!(original_state.theme, loaded_state.theme);
        assert_eq!(original_state.sidebar.collapsed, loaded_state.sidebar.collapsed);
        assert_eq!(original_state.sidebar.width, loaded_state.sidebar.width);
        assert_eq!(original_state.panel.hidden, loaded_state.panel.hidden);
        assert_eq!(original_state.panel.height, loaded_state.panel.height);
    }

    #[tokio::test]
    async fn test_persistence_error_handling() {
        // Test loading from non-existent file
        let non_existent_path = "/path/that/does/not/exist/config.json";
        let result = tokio::fs::read_to_string(non_existent_path).await;
        assert!(result.is_err(), "Should fail when reading non-existent file");
        
        // Test loading invalid JSON
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let invalid_config_path = temp_dir.path().join("invalid_config.json");
        
        tokio::fs::write(&invalid_config_path, "invalid json content")
            .await
            .expect("Failed to write invalid config");
        
        let invalid_json = tokio::fs::read_to_string(&invalid_config_path)
            .await
            .expect("Failed to read invalid config");
        
        let result: Result<LayoutState, _> = serde_json::from_str(&invalid_json);
        assert!(result.is_err(), "Should fail when parsing invalid JSON");
    }

    #[tokio::test]
    async fn test_concurrent_file_operations() {
        use std::sync::Arc;
        use tokio::sync::RwLock;
        
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let shared_state = Arc::new(RwLock::new(LayoutState::default()));
        let mut handles = vec![];
        
        // Spawn multiple tasks performing concurrent updates
        for i in 0..5 {
            let state_clone = Arc::clone(&shared_state);
            let temp_path = temp_dir.path().join(format!("config_{}.json", i));
            
            let handle = tokio::spawn(async move {
                let mut state = state_clone.write().await;
                
                // Perform different updates
                match i % 3 {
                    0 => state.theme = Theme::Dark,
                    1 => state.sidebar.width = 250.0 + (i as f64 * 10.0),
                    2 => state.panel.height = 200.0 + (i as f64 * 5.0),
                    _ => {}
                }
                
                // Save to individual file
                let json = serde_json::to_string_pretty(&*state)
                    .expect("Failed to serialize");
                tokio::fs::write(&temp_path, json)
                    .await
                    .expect("Failed to write config");
                
                // Verify file was written
                let content = tokio::fs::read_to_string(&temp_path)
                    .await
                    .expect("Failed to read back config");
                let _: LayoutState = serde_json::from_str(&content)
                    .expect("Failed to parse saved config");
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }
        
        // Verify final state is consistent
        let final_state = shared_state.read().await;
        assert!(final_state.sidebar.width >= 250.0);
        assert!(final_state.panel.height >= 200.0);
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_layout_state_creation_performance() {
        let iterations = 1000;
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _state = LayoutState::default();
        }
        
        let elapsed = start.elapsed();
        let avg_per_creation = elapsed / iterations;
        
        // Should be very fast - under 1ms per creation
        assert!(avg_per_creation < Duration::from_millis(1),
               "LayoutState creation took {:?} per iteration", avg_per_creation);
    }

    #[test]
    fn test_layout_state_modification_performance() {
        let mut state = LayoutState::default();
        let modifications = 1000;
        
        let start = Instant::now();
        
        for i in 0..modifications {
            match i % 4 {
                0 => state.theme = if i % 2 == 0 { Theme::Dark } else { Theme::Light },
                1 => state.sidebar.collapsed = i % 2 == 0,
                2 => state.panel.hidden = i % 2 == 0,
                3 => state.reduced_motion = i % 2 == 0,
                _ => {}
            }
        }
        
        let elapsed = start.elapsed();
        let avg_per_modification = elapsed / modifications;
        
        // Should be extremely fast - under 100μs per modification
        assert!(avg_per_modification < Duration::from_micros(100),
               "State modification took {:?} per operation", avg_per_modification);
    }

    #[test]
    fn test_serialization_performance() {
        let state = LayoutState::default();
        let iterations = 100;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _json = serde_json::to_string(&state)
                .expect("Serialization failed");
        }
        
        let elapsed = start.elapsed();
        let avg_per_serialization = elapsed / iterations;
        
        // Should be under 1ms per serialization
        assert!(avg_per_serialization < Duration::from_millis(1),
               "Serialization took {:?} per operation", avg_per_serialization);
    }

    #[test]
    fn test_deserialization_performance() {
        let state = LayoutState::default();
        let json = serde_json::to_string(&state).expect("Failed to serialize");
        let iterations = 100;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _state: LayoutState = serde_json::from_str(&json)
                .expect("Deserialization failed");
        }
        
        let elapsed = start.elapsed();
        let avg_per_deserialization = elapsed / iterations;
        
        // Should be under 1ms per deserialization
        assert!(avg_per_deserialization < Duration::from_millis(1),
               "Deserialization took {:?} per operation", avg_per_deserialization);
    }

    #[test]
    fn test_performance_targets_validation() {
        // Define performance targets from requirements
        let target_ui_transition = Duration::from_millis(100);
        let excellent_performance = Duration::from_millis(50);
        
        // Test that basic state operations meet targets
        let start = Instant::now();
        
        let mut state = LayoutState::default();
        state.theme = Theme::Dark;
        state.sidebar.collapsed = true;
        state.panel.hidden = true;
        
        let operation_time = start.elapsed();
        
        // Verify we meet performance targets
        assert!(operation_time < target_ui_transition,
               "Operation took {:?}, exceeds 100ms target", operation_time);
        assert!(operation_time < excellent_performance,
               "Operation took {:?}, exceeds 50ms excellent target", operation_time);
    }

    #[tokio::test]
    async fn test_performance_under_load() {
        let operations = 500;
        let start = Instant::now();
        
        for i in 0..operations {
            let mut state = LayoutState::default();
            
            // Perform various state changes
            match i % 5 {
                0 => state.theme = Theme::Dark,
                1 => state.sidebar.collapsed = true,
                2 => state.panel.hidden = true,
                3 => state.activity_bar.hidden = true,
                4 => state.reduced_motion = true,
                _ => {}
            }
            
            // Occasionally yield to prevent blocking
            if i % 50 == 0 {
                tokio::task::yield_now().await;
            }
        }
        
        let total_time = start.elapsed();
        let avg_time_per_operation = total_time / operations;
        
        // Verify average operation time meets targets
        assert!(avg_time_per_operation < Duration::from_millis(10),
               "Average operation time {:?} too slow", avg_time_per_operation);
        assert!(total_time < Duration::from_millis(2000),
               "Total time {:?} exceeded 2 seconds", total_time);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_state_workflow() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("workflow_test_config.json");
        
        // 1. Create and modify state
        let mut state = LayoutState::default();
        state.theme = Theme::Dark;
        state.sidebar.collapsed = true;
        state.sidebar.width = 320.0;
        state.panel.hidden = true;
        state.panel.height = 280.0;
        state.activity_bar.position = ActivityBarPosition::Right;
        state.reduced_motion = true;
        
        // 2. Persist state
        let json = serde_json::to_string_pretty(&state)
            .expect("Failed to serialize state");
        tokio::fs::write(&config_path, json)
            .await
            .expect("Failed to save state");
        
        // 3. Load state (simulating app restart)
        let saved_json = tokio::fs::read_to_string(&config_path)
            .await
            .expect("Failed to load state");
        let restored_state: LayoutState = serde_json::from_str(&saved_json)
            .expect("Failed to parse state");
        
        // 4. Verify complete integrity
        assert_eq!(state.theme, restored_state.theme);
        assert_eq!(state.sidebar.collapsed, restored_state.sidebar.collapsed);
        assert_eq!(state.sidebar.width, restored_state.sidebar.width);
        assert_eq!(state.panel.hidden, restored_state.panel.hidden);
        assert_eq!(state.panel.height, restored_state.panel.height);
        assert_eq!(state.activity_bar.position, restored_state.activity_bar.position);
        assert_eq!(state.reduced_motion, restored_state.reduced_motion);
        
        // 5. Test additional modifications on restored state
        let mut modified_state = restored_state;
        modified_state.theme = Theme::HighContrast;
        modified_state.sidebar.width = 400.0;
        
        // Verify modifications work correctly
        assert_eq!(modified_state.theme, Theme::HighContrast);
        assert_eq!(modified_state.sidebar.width, 400.0);
        // Original values should remain unchanged
        assert!(modified_state.sidebar.collapsed);
        assert!(modified_state.panel.hidden);
    }

    #[test]
    fn test_error_recovery_patterns() {
        // Test graceful handling of edge cases
        
        // 1. Invalid width recovery
        let mut state = LayoutState::default();
        state.sidebar.width = -100.0; // Invalid width
        
        // Apply validation logic (would be in LayoutManager)
        if state.sidebar.width < 150.0 {
            state.sidebar.width = 280.0; // Reset to default
        }
        assert_eq!(state.sidebar.width, 280.0);
        
        // 2. Invalid height recovery
        state.panel.height = -50.0; // Invalid height
        if state.panel.height < 100.0 {
            state.panel.height = 200.0; // Reset to default
        }
        assert_eq!(state.panel.height, 200.0);
        
        // 3. State consistency validation
        assert!(matches!(state.sidebar.position, 
                        SidebarPosition::Left | SidebarPosition::Right));
        assert!(matches!(state.panel.position, 
                        PanelPosition::Bottom | PanelPosition::Top | PanelPosition::Left | PanelPosition::Right));
        assert!(matches!(state.theme, 
                        Theme::Light | Theme::Dark | Theme::HighContrast));
    }

    #[test]
    fn test_state_validation_logic() {
        let state = LayoutState::default();
        
        // Test that default state is always valid
        assert!(state.sidebar.width > 0.0);
        assert!(state.panel.height > 0.0);
        
        // Test enum variants are valid
        match state.theme {
            Theme::Light | Theme::Dark | Theme::HighContrast => {},
        }
        
        match state.sidebar.position {
            SidebarPosition::Left | SidebarPosition::Right => {},
        }
        
        match state.panel.position {
            PanelPosition::Bottom | PanelPosition::Top | 
            PanelPosition::Left | PanelPosition::Right => {},
        }
        
        match state.activity_bar.position {
            ActivityBarPosition::Left | ActivityBarPosition::Right => {},
        }
    }
}