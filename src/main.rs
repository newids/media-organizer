use tracing::info;

mod models;
mod services;
mod state;
mod theme;
mod ui;

use models::AppConfig;
use state::AppStateProvider;
use ui::phase2_app;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting MediaOrganizer application");

    // Load configuration
    let _config = AppConfig::default();

    // Launch Dioxus desktop application with state provider
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
