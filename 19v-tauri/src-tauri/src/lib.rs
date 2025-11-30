mod app;
mod duckduckgo;
mod light_pattern;
mod simulation;

use app::spawn_ui;
use duckduckgo::spawn_ingestion;
use simulation::{LogLevel, SharedSimulation};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared = SharedSimulation::new();
    spawn_ingestion(shared.clone());
    spawn_ui(shared.clone());

    let log_builder = tauri_plugin_log::Builder::default().level(log::LevelFilter::Info);
    let shared_for_setup = shared.clone();

    tauri::Builder::default()
        .plugin(log_builder.build())
        .manage(shared.clone())
        .setup(move |_app| {
            shared_for_setup.log(LogLevel::Info, \"19V организм активирован\");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect(\"error while running tauri application\");
}
