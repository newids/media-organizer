use tracing::info;

mod models;
mod services;
mod state;
mod ui;

use models::AppConfig;
use ui::Phase2App;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting MediaOrganizer application");

    // Load configuration
    let _config = AppConfig::default();

    // Launch Dioxus desktop application
    dioxus::launch(Phase2App);
}
