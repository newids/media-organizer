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

    // Launch Dioxus desktop application
    // Note: Native system menu integration is not currently implemented
    // due to API limitations in Dioxus 0.6.3. The current approach with
    // muda/tao integration requires deeper integration that may need
    // a custom desktop backend or waiting for future Dioxus releases.
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
