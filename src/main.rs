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
    dioxus_desktop::launch_cfg(
        Phase2App,
        dioxus_desktop::Config::new().with_window(
            dioxus_desktop::WindowBuilder::new()
                .with_title("MediaOrganizer")
                .with_inner_size(dioxus_desktop::LogicalSize::new(1200, 800))
                .with_min_inner_size(dioxus_desktop::LogicalSize::new(800, 600))
                .with_resizable(true),
        ),
    );
}
