use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Represents a keyboard shortcut key combination
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyCombination {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
        }
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }

    /// Check if this key combination matches a keyboard event
    pub fn matches(&self, key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> bool {
        self.key.to_lowercase() == key.to_lowercase()
            && self.ctrl == ctrl
            && self.shift == shift
            && self.alt == alt
            && self.meta == meta
    }

    /// Create a human-readable description of the shortcut
    pub fn description(&self) -> String {
        let mut parts = Vec::new();
        
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.meta {
            #[cfg(target_os = "macos")]
            parts.push("Cmd");
            #[cfg(not(target_os = "macos"))]
            parts.push("Win");
        }
        
        parts.push(&self.key);
        parts.join(" + ")
    }
}

/// Action that can be triggered by a keyboard shortcut
#[derive(Debug, Clone)]
pub enum ShortcutAction {
    Copy,
    Paste,
    Cut,
    Delete,
    SelectAll,
    ClearSelection,
    Rename,
    NavigateUp,
    NavigateBack,
    NavigateForward,
    NavigateHome,
    Refresh,
    OpenFile,
    ShowProperties,
    TogglePreview,
    ToggleSearch,
    NewFolder,
    ShowSettings,
    Custom(String),
}

impl ShortcutAction {
    pub fn description(&self) -> &'static str {
        match self {
            ShortcutAction::Copy => "Copy selected items",
            ShortcutAction::Paste => "Paste from clipboard",
            ShortcutAction::Cut => "Cut selected items",
            ShortcutAction::Delete => "Delete selected items",
            ShortcutAction::SelectAll => "Select all items",
            ShortcutAction::ClearSelection => "Clear selection",
            ShortcutAction::Rename => "Rename selected item",
            ShortcutAction::NavigateUp => "Navigate to parent directory",
            ShortcutAction::NavigateBack => "Navigate back",
            ShortcutAction::NavigateForward => "Navigate forward",
            ShortcutAction::NavigateHome => "Navigate to home directory",
            ShortcutAction::Refresh => "Refresh current directory",
            ShortcutAction::OpenFile => "Open selected file",
            ShortcutAction::ShowProperties => "Show file properties",
            ShortcutAction::TogglePreview => "Toggle preview panel",
            ShortcutAction::ToggleSearch => "Toggle search",
            ShortcutAction::NewFolder => "Create new folder",
            ShortcutAction::ShowSettings => "Open settings panel",
            ShortcutAction::Custom(_) => "Custom action",
        }
    }
}

/// Keyboard shortcut registry for managing shortcuts and preventing conflicts
#[derive(Clone)]
pub struct ShortcutRegistry {
    shortcuts: Arc<Mutex<HashMap<KeyCombination, ShortcutAction>>>,
    disabled: Arc<Mutex<bool>>,
}

impl ShortcutRegistry {
    pub fn new() -> Self {
        let registry = Self {
            shortcuts: Arc::new(Mutex::new(HashMap::new())),
            disabled: Arc::new(Mutex::new(false)),
        };
        
        // Register default shortcuts
        registry.register_defaults();
        registry
    }

    /// Register default keyboard shortcuts
    fn register_defaults(&self) {
        let shortcuts = [
            // File operations
            (KeyCombination::new("c").with_ctrl(), ShortcutAction::Copy),
            (KeyCombination::new("v").with_ctrl(), ShortcutAction::Paste),
            (KeyCombination::new("x").with_ctrl(), ShortcutAction::Cut),
            (KeyCombination::new("Delete"), ShortcutAction::Delete),
            (KeyCombination::new("a").with_ctrl(), ShortcutAction::SelectAll),
            (KeyCombination::new("Escape"), ShortcutAction::ClearSelection),
            (KeyCombination::new("F2"), ShortcutAction::Rename),
            
            // Navigation
            (KeyCombination::new("ArrowUp").with_alt(), ShortcutAction::NavigateUp),
            (KeyCombination::new("ArrowLeft").with_alt(), ShortcutAction::NavigateBack),
            (KeyCombination::new("ArrowRight").with_alt(), ShortcutAction::NavigateForward),
            (KeyCombination::new("h").with_ctrl(), ShortcutAction::NavigateHome),
            (KeyCombination::new("F5"), ShortcutAction::Refresh),
            (KeyCombination::new("r").with_ctrl(), ShortcutAction::Refresh),
            (KeyCombination::new("Enter"), ShortcutAction::OpenFile),
            
            // View operations
            (KeyCombination::new("p").with_ctrl(), ShortcutAction::TogglePreview),
            (KeyCombination::new("f").with_ctrl(), ShortcutAction::ToggleSearch),
            (KeyCombination::new("n").with_ctrl().with_shift(), ShortcutAction::NewFolder),
            
            // Properties
            (KeyCombination::new("i").with_alt().with_ctrl(), ShortcutAction::ShowProperties),
            
            // Settings
            (KeyCombination::new(",").with_ctrl(), ShortcutAction::ShowSettings),
        ];

        if let Ok(mut map) = self.shortcuts.lock() {
            for (key_combo, action) in shortcuts {
                map.insert(key_combo, action);
            }
        }
    }

    /// Register a new keyboard shortcut
    pub fn register(&self, key_combo: KeyCombination, action: ShortcutAction) -> Result<(), String> {
        if let Ok(mut shortcuts) = self.shortcuts.lock() {
            if shortcuts.contains_key(&key_combo) {
                return Err(format!("Shortcut {} is already registered", key_combo.description()));
            }
            shortcuts.insert(key_combo, action);
            Ok(())
        } else {
            Err("Failed to acquire shortcuts lock".to_string())
        }
    }

    /// Unregister a keyboard shortcut
    pub fn unregister(&self, key_combo: &KeyCombination) -> bool {
        if let Ok(mut shortcuts) = self.shortcuts.lock() {
            shortcuts.remove(key_combo).is_some()
        } else {
            false
        }
    }

    /// Check if a key combination is registered
    pub fn is_registered(&self, key_combo: &KeyCombination) -> bool {
        if let Ok(shortcuts) = self.shortcuts.lock() {
            shortcuts.contains_key(key_combo)
        } else {
            false
        }
    }

    /// Get the action for a key combination
    pub fn get_action(&self, key_combo: &KeyCombination) -> Option<ShortcutAction> {
        if let Ok(shortcuts) = self.shortcuts.lock() {
            shortcuts.get(key_combo).cloned()
        } else {
            None
        }
    }

    /// Try to trigger a shortcut from keyboard event parameters
    pub fn try_trigger(&self, key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> Option<ShortcutAction> {
        // Check if shortcuts are disabled
        if let Ok(disabled) = self.disabled.lock() {
            if *disabled {
                return None;
            }
        }

        if let Ok(shortcuts) = self.shortcuts.lock() {
            for (key_combo, action) in shortcuts.iter() {
                if key_combo.matches(key, ctrl, shift, alt, meta) {
                    return Some(action.clone());
                }
            }
        }
        None
    }

    /// Get all registered shortcuts
    pub fn get_all_shortcuts(&self) -> Vec<(KeyCombination, ShortcutAction)> {
        if let Ok(shortcuts) = self.shortcuts.lock() {
            shortcuts.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            Vec::new()
        }
    }

    /// Enable or disable all shortcuts
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut disabled) = self.disabled.lock() {
            *disabled = !enabled;
        }
    }

    /// Check if shortcuts are enabled
    pub fn is_enabled(&self) -> bool {
        if let Ok(disabled) = self.disabled.lock() {
            !*disabled
        } else {
            true
        }
    }
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_combination_creation() {
        let combo = KeyCombination::new("c").with_ctrl();
        assert_eq!(combo.key, "c");
        assert!(combo.ctrl);
        assert!(!combo.shift);
        assert!(!combo.alt);
        assert!(!combo.meta);
    }

    #[test]
    fn test_key_combination_matching() {
        let combo = KeyCombination::new("c").with_ctrl();
        assert!(combo.matches("c", true, false, false, false));
        assert!(combo.matches("C", true, false, false, false)); // Case insensitive
        assert!(!combo.matches("c", false, false, false, false));
        assert!(!combo.matches("d", true, false, false, false));
    }

    #[test]
    fn test_shortcut_registry() {
        let registry = ShortcutRegistry::new();
        
        // Test default shortcuts are registered
        let copy_combo = KeyCombination::new("c").with_ctrl();
        assert!(registry.is_registered(&copy_combo));
        
        // Test getting action
        let action = registry.get_action(&copy_combo);
        assert!(matches!(action, Some(ShortcutAction::Copy)));
        
        // Test triggering shortcut
        let triggered = registry.try_trigger("c", true, false, false, false);
        assert!(matches!(triggered, Some(ShortcutAction::Copy)));
    }

    #[test]
    fn test_shortcut_registry_conflict_detection() {
        let registry = ShortcutRegistry::new();
        let copy_combo = KeyCombination::new("c").with_ctrl();
        
        // Try to register conflicting shortcut
        let result = registry.register(copy_combo, ShortcutAction::Cut);
        assert!(result.is_err());
    }

    #[test]
    fn test_shortcut_enable_disable() {
        let registry = ShortcutRegistry::new();
        
        // Shortcuts should be enabled by default
        assert!(registry.is_enabled());
        
        // Test triggering when enabled
        let triggered = registry.try_trigger("c", true, false, false, false);
        assert!(triggered.is_some());
        
        // Disable shortcuts
        registry.set_enabled(false);
        assert!(!registry.is_enabled());
        
        // Test triggering when disabled
        let triggered = registry.try_trigger("c", true, false, false, false);
        assert!(triggered.is_none());
        
        // Re-enable shortcuts
        registry.set_enabled(true);
        assert!(registry.is_enabled());
        
        // Test triggering when re-enabled
        let triggered = registry.try_trigger("c", true, false, false, false);
        assert!(triggered.is_some());
    }
}