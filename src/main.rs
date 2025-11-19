mod archguard;
mod ecs;
mod evolution;
mod lighting;
mod renderer;
mod ui;
mod voxel;

use eframe::egui;
use ui::EngineUI;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1920.0, 1080.0])
            .with_title("Adaptive Entity Engine v1.0"),
        ..Default::default()
    };

    eframe::run_native(
        "Adaptive Entity Engine v1.0",
        options,
        Box::new(|_cc| {
            Ok(Box::new(EngineUI::new()))
        }),
    )
}
