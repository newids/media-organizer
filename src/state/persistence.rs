use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs;
use tracing::{debug, warn, error};
use crate::state::app_state::{SettingsState, LayoutState};

const STORAGE_KEY: &str = "media_organizer_panel_state";
const SETTINGS_STORAGE_KEY: &str = "media_organizer_settings";
const LAYOUT_STORAGE_KEY: &str = "media_organizer_layout_state";
const DEBOUNCE_DELAY: Duration = Duration::from_millis(300);

/// Panel state configuration that can be persisted to localStorage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelState {
    /// Width of the left file tree panel in pixels
    pub panel_width: f64,
    /// Whether the file tree panel is visible
    pub panel_visible: bool,
    /// Last modified timestamp for versioning
    pub last_modified: u64,
    /// Version of the state format for future migrations
    pub version: u32,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            panel_width: 300.0,
            panel_visible: true,
            last_modified: 0,
            version: 1,
        }
    }
}

impl PanelState {
    /// Create a new panel state with current timestamp
    pub fn new(panel_width: f64, panel_visible: bool) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            panel_width,
            panel_visible,
            last_modified: now,
            version: 1,
        }
    }

    /// Validate the panel state values
    pub fn validate(&mut self) {
        // Ensure panel width is within reasonable bounds
        self.panel_width = self.panel_width.max(200.0).min(600.0);
        
        // Update timestamp
        self.last_modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
}

/// Service for persisting and restoring panel state using file system
pub struct PersistenceService {
    last_save: Option<Instant>,
    pending_state: Option<PanelState>,
    pending_settings: Option<SettingsState>,
    last_settings_save: Option<Instant>,
    pending_layout: Option<LayoutState>,
    last_layout_save: Option<Instant>,
    config_file_path: PathBuf,
    settings_file_path: PathBuf,
    layout_file_path: PathBuf,
}

impl Default for PersistenceService {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceService {
    /// Create a new persistence service
    pub fn new() -> Self {
        let config_file_path = Self::get_config_file_path();
        let settings_file_path = Self::get_settings_file_path();
        let layout_file_path = Self::get_layout_file_path();
        Self {
            last_save: None,
            pending_state: None,
            pending_settings: None,
            last_settings_save: None,
            pending_layout: None,
            last_layout_save: None,
            config_file_path,
            settings_file_path,
            layout_file_path,
        }
    }
    
    /// Get the path to the configuration file
    fn get_config_file_path() -> PathBuf {
        // Use application data directory for persistence
        if let Some(data_dir) = dirs::data_dir() {
            let app_dir = data_dir.join("MediaOrganizer");
            app_dir.join("panel_state.json")
        } else {
            // Fallback to current directory
            PathBuf::from("panel_state.json")
        }
    }
    
    /// Get the path to the settings file
    fn get_settings_file_path() -> PathBuf {
        // Use application data directory for persistence
        if let Some(data_dir) = dirs::data_dir() {
            let app_dir = data_dir.join("MediaOrganizer");
            app_dir.join("settings.json")
        } else {
            // Fallback to current directory
            PathBuf::from("settings.json")
        }
    }
    
    /// Get the path to the layout state file
    fn get_layout_file_path() -> PathBuf {
        // Use application data directory for persistence
        if let Some(data_dir) = dirs::data_dir() {
            let app_dir = data_dir.join("MediaOrganizer");
            app_dir.join("layout_state.json")
        } else {
            // Fallback to current directory
            PathBuf::from("layout_state.json")
        }
    }
    
    /// Ensure the config directory exists
    fn ensure_config_dir(&self) -> Result<(), String> {
        if let Some(parent) = self.config_file_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        Ok(())
    }

    /// Save panel state to localStorage with debouncing
    pub fn save_state_debounced(&mut self, state: PanelState) {
        self.pending_state = Some(state);
        
        let now = Instant::now();
        let should_save = match self.last_save {
            Some(last) => now.duration_since(last) >= DEBOUNCE_DELAY,
            None => true,
        };

        if should_save {
            if let Some(state) = self.pending_state.take() {
                self.save_state_immediate(state);
                self.last_save = Some(now);
            }
        }
    }

    /// Force save the pending state immediately
    pub fn flush_pending_save(&mut self) {
        if let Some(state) = self.pending_state.take() {
            self.save_state_immediate(state);
            self.last_save = Some(Instant::now());
        }
    }

    /// Save panel state to localStorage immediately
    fn save_state_immediate(&self, mut state: PanelState) {
        state.validate();
        
        match self.serialize_state(&state) {
            Ok(json) => {
                if let Err(e) = self.write_to_storage(&json) {
                    error!("Failed to write panel state to localStorage: {}", e);
                } else {
                    debug!("Panel state saved successfully: width={}", state.panel_width);
                }
            }
            Err(e) => {
                error!("Failed to serialize panel state: {}", e);
            }
        }
    }

    /// Load panel state from localStorage
    pub fn load_state(&self) -> PanelState {
        match self.read_from_storage() {
            Ok(Some(json)) => {
                match self.deserialize_state(&json) {
                    Ok(mut state) => {
                        state.validate();
                        debug!("Panel state loaded successfully: width={}", state.panel_width);
                        state
                    }
                    Err(e) => {
                        warn!("Failed to deserialize panel state, using defaults: {}", e);
                        PanelState::default()
                    }
                }
            }
            Ok(None) => {
                debug!("No saved panel state found, using defaults");
                PanelState::default()
            }
            Err(e) => {
                warn!("Failed to read panel state from localStorage, using defaults: {}", e);
                PanelState::default()
            }
        }
    }

    /// Clear saved state from localStorage
    pub fn clear_state(&self) -> Result<(), String> {
        self.remove_from_storage()
    }

    /// Check if file storage is available
    pub fn is_storage_available(&self) -> bool {
        // Check if we can write to the config directory
        self.ensure_config_dir().is_ok()
    }
    
    // Settings persistence methods
    
    /// Save settings state with debouncing
    pub fn save_settings_debounced(&mut self, settings: SettingsState) {
        self.pending_settings = Some(settings);
        
        let now = Instant::now();
        let should_save = match self.last_settings_save {
            Some(last) => now.duration_since(last) >= DEBOUNCE_DELAY,
            None => true,
        };

        if should_save {
            if let Some(settings) = self.pending_settings.take() {
                self.save_settings_immediate(settings);
                self.last_settings_save = Some(now);
            }
        }
    }

    /// Force save the pending settings immediately
    pub fn flush_pending_settings_save(&mut self) {
        if let Some(settings) = self.pending_settings.take() {
            self.save_settings_immediate(settings);
            self.last_settings_save = Some(Instant::now());
        }
    }

    /// Save settings state immediately
    fn save_settings_immediate(&self, settings: SettingsState) {
        match self.serialize_settings(&settings) {
            Ok(json) => {
                if let Err(e) = self.write_settings_to_storage(&json) {
                    error!("Failed to write settings to storage: {}", e);
                } else {
                    debug!("Settings saved successfully: theme={:?}", settings.theme);
                }
            }
            Err(e) => {
                error!("Failed to serialize settings: {}", e);
            }
        }
    }

    /// Load settings state from storage
    pub fn load_settings(&self) -> SettingsState {
        match self.read_settings_from_storage() {
            Ok(Some(json)) => {
                match self.deserialize_settings(&json) {
                    Ok(settings) => {
                        debug!("Settings loaded successfully: theme={:?}", settings.theme);
                        settings
                    }
                    Err(e) => {
                        warn!("Failed to deserialize settings, using defaults: {}", e);
                        SettingsState::default()
                    }
                }
            }
            Ok(None) => {
                debug!("No saved settings found, using defaults");
                SettingsState::default()
            }
            Err(e) => {
                warn!("Failed to read settings from storage, using defaults: {}", e);
                SettingsState::default()
            }
        }
    }

    /// Clear saved settings from storage
    pub fn clear_settings(&self) -> Result<(), String> {
        self.remove_settings_from_storage()
    }

    // Layout state persistence methods
    
    /// Save layout state with debouncing
    pub fn save_layout_debounced(&mut self, layout: LayoutState) {
        self.pending_layout = Some(layout);
        
        let now = Instant::now();
        let should_save = match self.last_layout_save {
            Some(last) => now.duration_since(last) >= DEBOUNCE_DELAY,
            None => true,
        };

        if should_save {
            if let Some(layout) = self.pending_layout.take() {
                self.save_layout_immediate(layout);
                self.last_layout_save = Some(now);
            }
        }
    }

    /// Force save the pending layout immediately
    pub fn flush_pending_layout_save(&mut self) {
        if let Some(layout) = self.pending_layout.take() {
            self.save_layout_immediate(layout);
            self.last_layout_save = Some(Instant::now());
        }
    }

    /// Save layout state immediately
    fn save_layout_immediate(&self, layout: LayoutState) {
        match self.serialize_layout(&layout) {
            Ok(json) => {
                if let Err(e) = self.write_layout_to_storage(&json) {
                    error!("Failed to write layout state to storage: {}", e);
                } else {
                    debug!("Layout state saved successfully: theme={:?}", layout.theme);
                }
            }
            Err(e) => {
                error!("Failed to serialize layout state: {}", e);
            }
        }
    }

    /// Load layout state from storage
    pub fn load_layout(&self) -> Option<LayoutState> {
        match self.read_layout_from_storage() {
            Ok(Some(json)) => {
                match self.deserialize_layout(&json) {
                    Ok(layout) => {
                        debug!("Layout state loaded successfully: theme={:?}", layout.theme);
                        Some(layout)
                    }
                    Err(e) => {
                        warn!("Failed to deserialize layout state, using defaults: {}", e);
                        None
                    }
                }
            }
            Ok(None) => {
                debug!("No saved layout state found");
                None
            }
            Err(e) => {
                warn!("Failed to read layout state from storage: {}", e);
                None
            }
        }
    }

    /// Clear saved layout state from storage
    pub fn clear_layout(&self) -> Result<(), String> {
        self.remove_layout_from_storage()
    }

    // Private helper methods

    fn serialize_state(&self, state: &PanelState) -> Result<String, String> {
        serde_json::to_string(state).map_err(|e| format!("Serialization error: {}", e))
    }

    fn deserialize_state(&self, json: &str) -> Result<PanelState, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))
    }

    fn write_to_storage(&self, json: &str) -> Result<(), String> {
        self.ensure_config_dir()?;
        fs::write(&self.config_file_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))
    }

    fn read_from_storage(&self) -> Result<Option<String>, String> {
        match fs::read_to_string(&self.config_file_path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("Failed to read config file: {}", e)),
        }
    }

    fn remove_from_storage(&self) -> Result<(), String> {
        match fs::remove_file(&self.config_file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => Err(format!("Failed to remove config file: {}", e)),
        }
    }
    
    // Settings-specific helper methods
    
    fn serialize_settings(&self, settings: &SettingsState) -> Result<String, String> {
        serde_json::to_string_pretty(settings).map_err(|e| format!("Settings serialization error: {}", e))
    }

    fn deserialize_settings(&self, json: &str) -> Result<SettingsState, String> {
        serde_json::from_str(json).map_err(|e| format!("Settings deserialization error: {}", e))
    }

    fn write_settings_to_storage(&self, json: &str) -> Result<(), String> {
        self.ensure_config_dir()?;
        fs::write(&self.settings_file_path, json)
            .map_err(|e| format!("Failed to write settings file: {}", e))
    }

    fn read_settings_from_storage(&self) -> Result<Option<String>, String> {
        match fs::read_to_string(&self.settings_file_path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("Failed to read settings file: {}", e)),
        }
    }

    fn remove_settings_from_storage(&self) -> Result<(), String> {
        match fs::remove_file(&self.settings_file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => Err(format!("Failed to remove settings file: {}", e)),
        }
    }
    
    // Layout-specific helper methods
    
    fn serialize_layout(&self, layout: &LayoutState) -> Result<String, String> {
        serde_json::to_string_pretty(layout).map_err(|e| format!("Layout serialization error: {}", e))
    }

    fn deserialize_layout(&self, json: &str) -> Result<LayoutState, String> {
        serde_json::from_str(json).map_err(|e| format!("Layout deserialization error: {}", e))
    }

    fn write_layout_to_storage(&self, json: &str) -> Result<(), String> {
        self.ensure_config_dir()?;
        fs::write(&self.layout_file_path, json)
            .map_err(|e| format!("Failed to write layout file: {}", e))
    }

    fn read_layout_from_storage(&self) -> Result<Option<String>, String> {
        match fs::read_to_string(&self.layout_file_path) {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("Failed to read layout file: {}", e)),
        }
    }

    fn remove_layout_from_storage(&self) -> Result<(), String> {
        match fs::remove_file(&self.layout_file_path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Already deleted
            Err(e) => Err(format!("Failed to remove layout file: {}", e)),
        }
    }
}

use std::sync::{Mutex, OnceLock};

/// Global persistence service instance using thread-safe OnceLock
static PERSISTENCE_SERVICE: OnceLock<Mutex<PersistenceService>> = OnceLock::new();

/// Get the global persistence service instance
pub fn get_persistence_service() -> std::sync::MutexGuard<'static, PersistenceService> {
    PERSISTENCE_SERVICE
        .get_or_init(|| Mutex::new(PersistenceService::new()))
        .lock()
        .unwrap()
}

/// Convenience function to save panel state with debouncing
pub fn save_panel_state_debounced(panel_width: f64, panel_visible: bool) {
    let state = PanelState::new(panel_width, panel_visible);
    get_persistence_service().save_state_debounced(state);
}

/// Convenience function to load panel state
pub fn load_panel_state() -> PanelState {
    get_persistence_service().load_state()
}

/// Convenience function to flush any pending saves
pub fn flush_pending_saves() {
    get_persistence_service().flush_pending_save();
}

/// Convenience function to check if storage is available
pub fn is_storage_available() -> bool {
    get_persistence_service().is_storage_available()
}

// Settings convenience functions

/// Convenience function to save settings with debouncing
pub fn save_settings_debounced(settings: SettingsState) {
    get_persistence_service().save_settings_debounced(settings);
}

/// Convenience function to load settings
pub fn load_settings() -> SettingsState {
    get_persistence_service().load_settings()
}

/// Convenience function to flush pending settings saves
pub fn flush_pending_settings_saves() {
    get_persistence_service().flush_pending_settings_save();
}

/// Convenience function to clear saved settings
pub fn clear_settings() -> Result<(), String> {
    get_persistence_service().clear_settings()
}

// Layout state convenience functions

/// Convenience function to save layout state with debouncing
pub fn save_layout_state_debounced(layout: LayoutState) {
    get_persistence_service().save_layout_debounced(layout);
}

/// Convenience function to load layout state
pub fn load_layout_state() -> Option<LayoutState> {
    get_persistence_service().load_layout()
}

/// Convenience function to flush pending layout saves
pub fn flush_pending_layout_saves() {
    get_persistence_service().flush_pending_layout_save();
}

/// Convenience function to clear saved layout state
pub fn clear_layout_state() -> Result<(), String> {
    get_persistence_service().clear_layout()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_state_validation() {
        let mut state = PanelState::new(50.0, true); // Too small
        state.validate();
        assert_eq!(state.panel_width, 200.0);

        let mut state = PanelState::new(1000.0, true); // Too large
        state.validate();
        assert_eq!(state.panel_width, 600.0);

        let mut state = PanelState::new(350.0, true); // Valid
        state.validate();
        assert_eq!(state.panel_width, 350.0);
    }

    #[test]
    fn test_panel_state_serialization() {
        let service = PersistenceService::new();
        let state = PanelState::new(350.0, true);
        
        let json = service.serialize_state(&state).unwrap();
        let deserialized = service.deserialize_state(&json).unwrap();
        
        assert_eq!(state.panel_width, deserialized.panel_width);
        assert_eq!(state.panel_visible, deserialized.panel_visible);
        assert_eq!(state.version, deserialized.version);
    }

    #[test]
    fn test_default_panel_state() {
        let state = PanelState::default();
        assert_eq!(state.panel_width, 300.0);
        assert_eq!(state.panel_visible, true);
        assert_eq!(state.version, 1);
    }

    // Layout state persistence integration tests
    
    #[test]
    fn test_layout_state_serialization() {
        let service = PersistenceService::new();
        let layout_state = LayoutState::default();
        
        let json = service.serialize_layout(&layout_state).unwrap();
        let deserialized = service.deserialize_layout(&json).unwrap();
        
        assert_eq!(layout_state.theme, deserialized.theme);
        assert_eq!(layout_state.activity_bar.active_view, deserialized.activity_bar.active_view);
        assert_eq!(layout_state.sidebar.width, deserialized.sidebar.width);
        assert_eq!(layout_state.panel.height, deserialized.panel.height);
        assert_eq!(layout_state.persistence.persist_layout, deserialized.persistence.persist_layout);
    }
    
    #[test]
    fn test_layout_state_persistence_cycle() {
        let mut service = PersistenceService::new();
        let mut layout_state = LayoutState::default();
        
        // Modify some values to test persistence
        layout_state.theme = crate::state::Theme::Light;
        layout_state.sidebar.width = 350.0;
        layout_state.panel.height = 250.0;
        layout_state.activity_bar.is_visible = false;
        
        // Save the layout state
        service.save_layout_debounced(layout_state.clone());
        service.flush_pending_layout_save();
        
        // Load it back and verify
        let loaded_state = service.load_layout().unwrap();
        assert_eq!(layout_state.theme, loaded_state.theme);
        assert_eq!(layout_state.sidebar.width, loaded_state.sidebar.width);
        assert_eq!(layout_state.panel.height, loaded_state.panel.height);
        assert_eq!(layout_state.activity_bar.is_visible, loaded_state.activity_bar.is_visible);
    }
    
    #[test]
    fn test_layout_state_convenience_functions() {
        let mut layout_state = LayoutState::default();
        layout_state.theme = crate::state::Theme::Dark;
        layout_state.sidebar.width = 400.0;
        
        // Test convenience function saving
        save_layout_state_debounced(layout_state.clone());
        flush_pending_layout_saves();
        
        // Test convenience function loading
        let loaded_state = load_layout_state().unwrap();
        assert_eq!(layout_state.theme, loaded_state.theme);
        assert_eq!(layout_state.sidebar.width, loaded_state.sidebar.width);
        
        // Test clearing
        assert!(clear_layout_state().is_ok());
        
        // Verify it was cleared (should return None)
        assert!(load_layout_state().is_none());
    }
    
    #[test]
    fn test_layout_persistence_settings() {
        let mut layout_state = LayoutState::default();
        
        // Test with persistence disabled
        layout_state.persistence.persist_layout = false;
        save_layout_state_debounced(layout_state.clone());
        flush_pending_layout_saves();
        
        // Should still save (the settings control auto-save behavior, not manual saves)
        let loaded_state = load_layout_state().unwrap();
        assert_eq!(layout_state.persistence.persist_layout, loaded_state.persistence.persist_layout);
        
        // Test auto-save interval setting
        layout_state.persistence.auto_save_interval = 600; // 10 minutes
        save_layout_state_debounced(layout_state.clone());
        flush_pending_layout_saves();
        
        let loaded_state = load_layout_state().unwrap();
        assert_eq!(layout_state.persistence.auto_save_interval, loaded_state.persistence.auto_save_interval);
    }
    
    #[test]
    fn test_multiple_persistence_services() {
        // Test that multiple persistence operations don't interfere
        let mut service1 = PersistenceService::new();
        let mut service2 = PersistenceService::new();
        
        let layout1 = LayoutState::default();
        let mut settings1 = SettingsState::default();
        settings1.theme = crate::state::Theme::Light;
        
        // Save different types through different services
        service1.save_layout_debounced(layout1.clone());
        service2.save_settings_debounced(settings1.clone());
        
        service1.flush_pending_layout_save();
        service2.flush_pending_settings_save();
        
        // Verify both were saved correctly
        let loaded_layout = service1.load_layout().unwrap();
        let loaded_settings = service2.load_settings();
        
        assert_eq!(layout1.theme, loaded_layout.theme);
        assert_eq!(settings1.theme, loaded_settings.theme);
    }
    
    #[test]
    fn test_debounced_layout_saving() {
        let mut service = PersistenceService::new();
        let mut layout_state = LayoutState::default();
        
        // Make rapid changes
        for i in 0..5 {
            layout_state.sidebar.width = 300.0 + (i as f64 * 10.0);
            service.save_layout_debounced(layout_state.clone());
        }
        
        // Only the last change should be saved after flush
        service.flush_pending_layout_save();
        
        let loaded_state = service.load_layout().unwrap();
        assert_eq!(loaded_state.sidebar.width, 340.0); // 300 + 4*10
    }
    
    #[test] 
    fn test_layout_persistence_error_handling() {
        let service = PersistenceService::new();
        
        // Test invalid JSON deserialization
        let invalid_json = "{invalid json}";
        assert!(service.deserialize_layout(invalid_json).is_err());
        
        // Test loading from non-existent file (should return None, not error)
        let loaded_state = service.load_layout();
        assert!(loaded_state.is_none());
        
        // Test clearing non-existent file (should succeed)
        assert!(service.clear_layout().is_ok());
    }
    
    #[test]
    fn test_layout_state_json_format() {
        let service = PersistenceService::new();
        let layout_state = LayoutState::default();
        
        let json = service.serialize_layout(&layout_state).unwrap();
        
        // Verify JSON contains expected fields
        assert!(json.contains("\"theme\":"));
        assert!(json.contains("\"activity_bar\":"));
        assert!(json.contains("\"sidebar\":"));
        assert!(json.contains("\"editor_layout\":"));
        assert!(json.contains("\"panel\":"));
        assert!(json.contains("\"viewport\":"));
        assert!(json.contains("\"ui_preferences\":"));
        assert!(json.contains("\"persistence\":"));
        
        // Verify it's pretty-printed JSON
        assert!(json.contains("\n"));
        assert!(json.contains("  "));
    }
    
    #[test]
    fn test_concurrent_persistence_operations() {
        use std::thread;
        use std::sync::Arc;
        
        let layout_state = Arc::new(LayoutState::default());
        let settings_state = Arc::new(SettingsState::default());
        
        let handles: Vec<_> = (0..5).map(|i| {
            let layout = Arc::clone(&layout_state);
            let settings = Arc::clone(&settings_state);
            
            thread::spawn(move || {
                // Save both layout and settings concurrently
                save_layout_state_debounced((*layout).clone());
                save_settings_debounced((*settings).clone());
                
                if i == 4 {
                    // Last thread flushes
                    flush_pending_layout_saves();
                    flush_pending_settings_saves();
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final state was saved
        let loaded_layout = load_layout_state().unwrap();
        let loaded_settings = load_settings();
        
        assert_eq!(layout_state.theme, loaded_layout.theme);
        assert_eq!(settings_state.theme, loaded_settings.theme);
    }
}