use tracing::info;

mod models;
mod performance;
mod services;
mod state;
mod theme;
mod ui;
mod utils;

use models::AppConfig;
use state::AppStateProvider;
use ui::phase2_app;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting File Manager application");

    // Load configuration
    let _config = AppConfig::default();

    // Launch Dioxus desktop application with state provider
    // Note: Custom MenuBar component has been removed as per UPGRADE-003
    // Native menu bar integration is planned but requires direct tao/wry integration
    // which is not currently implemented due to API limitations in Dioxus 0.6.3
    dioxus::launch(app);
}

// Root app component with state provider
fn app() -> dioxus::prelude::Element {
    use dioxus::prelude::*;
    
    rsx! {
        AppStateProvider {
            phase2_app {}
        }
    }
}
