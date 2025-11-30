#[cfg(feature = "gui")]
mod archguard;
#[cfg(feature = "gui")]
mod chat;
#[cfg(feature = "gui")]
mod ecs;
#[cfg(feature = "gui")]
mod evolution;
#[cfg(feature = "gui")]
mod integrations;
#[cfg(feature = "gui")]
mod learning;
#[cfg(feature = "gui")]
mod lighting;
#[cfg(feature = "gui")]
mod renderer;
#[cfg(feature = "gui")]
mod resources;
#[cfg(feature = "gui")]
mod ui;
#[cfg(feature = "gui")]
mod voxel;

#[cfg(feature = "gui")]
fn main() -> Result<(), eframe::Error> {
    use eframe::egui;
    use ui::EngineUI;
    
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

#[cfg(not(feature = "gui"))]
fn main() {
    println!("GUI feature not enabled. Run 'cargo run --bin test-components' to test core components.");
}
