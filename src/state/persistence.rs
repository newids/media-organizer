use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs;
use tracing::{debug, warn, error};

const STORAGE_KEY: &str = "media_organizer_panel_state";
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
    config_file_path: PathBuf,
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
        Self {
            last_save: None,
            pending_state: None,
            config_file_path,
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
}

/// Global persistence service instance
static mut PERSISTENCE_SERVICE: Option<PersistenceService> = None;
static mut INIT_ONCE: std::sync::Once = std::sync::Once::new();

/// Get the global persistence service instance
pub fn get_persistence_service() -> &'static mut PersistenceService {
    unsafe {
        INIT_ONCE.call_once(|| {
            PERSISTENCE_SERVICE = Some(PersistenceService::new());
        });
        PERSISTENCE_SERVICE.as_mut().unwrap()
    }
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
}