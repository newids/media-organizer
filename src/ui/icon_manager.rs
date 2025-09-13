use serde::{Serialize, Deserialize};
use crate::ui::icon_packs::IconPack;
use dioxus::prelude::*;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSettings {
    pub current_pack: IconPack,
    pub custom_icon_path: Option<PathBuf>,
    pub show_file_extensions: bool,
    pub icon_size: IconSize,
    pub show_file_icons: bool,
    pub show_folder_icons: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IconSize {
    Small,   // 12px
    Medium,  // 16px
    Large,   // 20px
}

impl IconSize {
    pub fn to_pixels(&self) -> u32 {
        match self {
            IconSize::Small => 12,
            IconSize::Medium => 16,
            IconSize::Large => 20,
        }
    }
}

impl Default for IconSettings {
    fn default() -> Self {
        Self {
            current_pack: IconPack::VSCode,
            custom_icon_path: None,
            show_file_extensions: true,
            icon_size: IconSize::Medium,
            show_file_icons: true,
            show_folder_icons: true,
        }
    }
}

impl IconSettings {
    const SETTINGS_FILE: &'static str = "icon_settings.json";
    
    pub fn load() -> Self {
        let settings_path = Self::get_settings_path();
        
        if let Ok(content) = fs::read_to_string(&settings_path) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
        
        Self::default()
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path();
        
        // Create parent directories if they don't exist
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, content)?;
        
        Ok(())
    }
    
    fn get_settings_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("MediaOrganizer")
            .join(Self::SETTINGS_FILE)
    }
}

// Global icon manager state
#[derive(Clone)]
pub struct IconManager {
    pub settings: Signal<IconSettings>,
}

impl IconManager {
    pub fn new() -> Self {
        Self {
            settings: Signal::new(IconSettings::load()),
        }
    }
    
    pub fn change_pack(&mut self, pack: IconPack) {
        let mut settings = self.settings.write();
        settings.current_pack = pack;

        // Save settings
        if let Err(e) = settings.save() {
            tracing::warn!("Failed to save icon settings: {}", e);
        }
    }
    
    pub fn change_size(&mut self, size: IconSize) {
        let mut settings = self.settings.write();
        settings.icon_size = size;
        
        if let Err(e) = settings.save() {
            tracing::warn!("Failed to save icon settings: {}", e);
        }
    }
    
    pub fn toggle_extensions(&mut self) {
        let mut settings = self.settings.write();
        settings.show_file_extensions = !settings.show_file_extensions;
        
        if let Err(e) = settings.save() {
            tracing::warn!("Failed to save icon settings: {}", e);
        }
    }
    
    pub fn save_settings(&self) {
        let settings = self.settings.read();
        if let Err(e) = settings.save() {
            tracing::warn!("Failed to save icon settings: {}", e);
        }
    }
}

// Hook to use icon manager
pub fn use_icon_manager() -> IconManager {
    let icon_manager = use_context::<IconManager>();
    icon_manager
}

// Simplified function to get icon settings info
pub fn get_icon_settings_info() -> String {
    let settings = IconSettings::load();
    format!("Current icon pack: {}", settings.current_pack.name())
}

// Provider component for icon manager
#[component]
pub fn IconManagerProvider(children: Element) -> Element {
    use_context_provider(|| IconManager::new());
    
    rsx! {
        {children}
    }
}