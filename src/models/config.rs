use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ui: UiConfig,
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub left_panel_width: f32,
    pub show_hidden_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig {
                theme: "system".to_string(),
                left_panel_width: 300.0,
                show_hidden_files: false,
            },
            window: WindowConfig {
                width: 1200,
                height: 800,
                maximized: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();

        assert_eq!(config.ui.theme, "system");
        assert_eq!(config.ui.left_panel_width, 300.0);
        assert_eq!(config.ui.show_hidden_files, false);

        assert_eq!(config.window.width, 1200);
        assert_eq!(config.window.height, 800);
        assert_eq!(config.window.maximized, false);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.ui.theme, deserialized.ui.theme);
        assert_eq!(config.window.width, deserialized.window.width);
    }
}
