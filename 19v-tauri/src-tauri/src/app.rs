use std::{fs, path::PathBuf, time::Duration};

use anyhow::Result;
use eframe::{egui, App};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sha2::{Digest, Sha256};

use crate::{
    light_pattern::{LightPatternCompute, LightPatternSnapshot},
    simulation::{CameraState, ConceptOrigin, LogLevel, SharedSimulation, SimulationSnapshot},
};

const FRAME_TIME: Duration = Duration::from_millis(16);
const MAX_FILE_BYTES: usize = 1_048_576; // 1 MiB safety limit

pub fn spawn_ui(shared: SharedSimulation) {
    std::thread::spawn(move || {
        if let Err(err) = launch_eframe(shared) {
            eprintln!("Failed to start egui shell: {err:?}");
        }
    });
}

fn launch_eframe(shared: SharedSimulation) -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("19V v3.0"),
        ..Default::default()
    };

    eframe::run_native(
        "19V v3.0",
        native_options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 240));
            cc.egui_ctx.set_style(style);
            Ok(Box::new(NineteenVApp::new(cc, shared.clone())))
        }),
    )?;
    Ok(())
}

pub struct NineteenVApp {
    shared: SharedSimulation,
    light_pattern: LightPatternCompute,
    last_pattern: Option<LightPatternSnapshot>,
    snapshot: Option<SimulationSnapshot>,
    camera: CameraState,
    rng: StdRng,
    energy_bias: f32,
    stochasticity: f32,
}

impl NineteenVApp {
    fn new(_cc: &eframe::CreationContext<'_>, shared: SharedSimulation) -> Self {
        Self {
            shared,
            light_pattern: LightPatternCompute::default(),
            last_pattern: None,
            snapshot: None,
            camera: CameraState::default(),
            rng: StdRng::seed_from_u64(19),
            energy_bias: 0.1,
            stochasticity: 0.35,
        }
    }

    fn simulate_frame(&mut self) {
        let jitter = self.rng.gen_range(0.0..1.0) * self.stochasticity;
        match self.light_pattern.step(self.energy_bias, jitter) {
            Ok(pattern) => {
                self.last_pattern = Some(pattern.clone());
                let snapshot = self.shared.tick(&pattern);
                self.snapshot = Some(snapshot);
            }
            Err(err) => {
                self.shared
                    .log(LogLevel::Error, format!("GPU LightPattern error: {err}"));
            }
        }
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) {
        if let Some(snapshot) = &self.snapshot {
            ui.horizontal(|ui| {
                ui.heading("19В Организм v3.0");
                ui.separator();
                ui.label(format!(
                    "Σ обновлений: {:.1} M",
                    snapshot.updates_processed as f64 / 1_000_000.0
                ));
                ui.label(format!(
                    "Энергия: {:.2}",
                    snapshot.average_energy
                ));
                ui.label(format!(
                    "Концепты: {}",
                    snapshot.concept_count
                ));
            });
        }

        ui.horizontal(|ui| {
            ui.label("Энергия");
            ui.add(egui::Slider::new(&mut self.energy_bias, -0.5..=0.8).text("bias"));
            ui.label("Стохастика");
            ui.add(egui::Slider::new(&mut self.stochasticity, 0.0..=1.0));
            if ui.button("19В").clicked() {
                self.energy_bias = self.rng.gen_range(-0.3..0.8);
                self.stochasticity = self.rng.gen_range(0.2..0.85);
                self.shared.log(LogLevel::Info, "19В импульс синхронизации");
            }
        });

        if let Some(snapshot) = &self.snapshot {
            let tick_ms = snapshot.tick_time.as_secs_f32() * 1000.0;
            ui.label(format!("Тик: {:.2} мс (цель 16 мс)", tick_ms));
            if let Some(pattern) = &self.last_pattern {
                let blend = pattern.values.iter().copied().sum::<f32>()
                    / pattern.values.len().max(1) as f32;
                ui.label(format!("LightPattern: {:.2}", blend));
            }
        }
    }

    fn draw_point_cloud(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available, egui::Sense::drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(230));

        if response.dragged() {
            let delta = response.drag_delta();
            self.camera.orbit(egui::Vec2::new(delta.x, delta.y));
        }
        if response.hovered() {
            let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > f32::EPSILON {
                self.camera.zoom(scroll.signum());
            }
        }

        if let Some(snapshot) = &self.snapshot {
            for particle in &snapshot.particles {
                if let Some((pos, depth)) = self.camera.project(particle.position, rect) {
                    let energy = particle.energy.clamp(0.0, 1.0);
                    let color = egui::Color32::from_rgb(
                        (energy * 255.0) as u8,
                        ((particle.emotion.valence.clamp(-1.0, 1.0) + 1.0) * 127.0) as u8,
                        ((1.0 - energy) * 255.0) as u8,
                    );
                    let radius = (3.0 / depth).clamp(0.5, 4.0);
                    painter.circle_filled(pos, radius, color);
                }
            }
        } else {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Загрузка организму...",
                egui::FontId::proportional(24.0),
                egui::Color32::LIGHT_GRAY,
            );
        }
    }

    fn draw_log(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("log-grid").striped(true).show(ui, |ui| {
            if let Some(snapshot) = &self.snapshot {
                for entry in snapshot.log.iter().rev().take(16) {
                    let color = match entry.level {
                        LogLevel::Info => egui::Color32::LIGHT_GREEN,
                        LogLevel::Warn => egui::Color32::YELLOW,
                        LogLevel::Error => egui::Color32::RED,
                    };
                    ui.label(entry.timestamp.format("%H:%M:%S").to_string());
                    ui.colored_label(color, &entry.message);
                    ui.end_row();
                }
            }
        });
    }

    fn handle_file(&mut self, path: &PathBuf) {
        match fs::read(path) {
            Ok(mut bytes) => {
                if bytes.len() > MAX_FILE_BYTES {
                    bytes.truncate(MAX_FILE_BYTES);
                }
                let digest = Sha256::digest(&bytes);
                let label = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("anonymous_file")
                    .to_string();
                self.shared
                    .ingest_concepts(&label, ConceptOrigin::FileDrop, &bytes);
                self.shared.log(
                    LogLevel::Info,
                    format!("Файл {label} внедрён ({} байт)", bytes.len()),
                );
                self.shared.log(
                    LogLevel::Info,
                    format!("SHA256: {:x}", digest),
                );
            }
            Err(err) => {
                self.shared.log(
                    LogLevel::Error,
                    format!("Не удалось прочитать файл {:?}: {err}", path),
                );
            }
        }
    }
}

impl App for NineteenVApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(FRAME_TIME);
        self.simulate_frame();

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            self.draw_header(ui);
        });

        egui::TopBottomPanel::bottom("log").default_height(200.0).show(ctx, |ui| {
            self.draw_log(ui);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(5, 5, 5)))
            .show(ctx, |ui| {
                self.draw_point_cloud(ui);
            });
    }

    fn on_event(
        &mut self,
        _ctx: &egui::Context,
        event: &egui::Event,
        _frame: &mut eframe::Frame,
    ) -> eframe::EventResponse {
        match event {
            egui::Event::FileDropped(file) => {
                if let Some(path) = file.path.clone() {
                    self.handle_file(&path);
                } else if let Some(bytes) = &file.bytes {
                    let mut data = bytes.clone();
                    if data.len() > MAX_FILE_BYTES {
                        data.truncate(MAX_FILE_BYTES);
                    }
                    let label = file.name.clone().unwrap_or_else(|| "drop.bin".into());
                    self.shared
                        .ingest_concepts(&label, ConceptOrigin::FileDrop, &data);
                    self.shared.log(
                        LogLevel::Info,
                        format!("Буфер {label} внедрён ({} байт)", data.len()),
                    );
                }
                eframe::EventResponse::Consumed
            }
            egui::Event::Scroll(delta) => {
                self.camera.zoom(delta.y);
                eframe::EventResponse::Consumed
            }
            _ => eframe::EventResponse::Ignored,
        }
    }
}
