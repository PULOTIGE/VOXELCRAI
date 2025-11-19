use crate::archguard::ArchGuard;
use crate::evolution::EvolutionEngine;
use crate::lighting::LightingSystem;
use crate::voxel::VoxelWorld;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::time::Instant;

pub struct EngineUI {
    world: VoxelWorld,
    evolution: EvolutionEngine,
    lighting: LightingSystem,
    archguard: ArchGuard,
    start_time: Instant,
    trauma_mode: bool,
    show_debug: bool,
    point_cloud_data: Vec<([f32; 3], [f32; 3])>,
}

impl EngineUI {
    pub fn new() -> Self {
        Self {
            world: VoxelWorld::new(),
            evolution: EvolutionEngine::new(),
            lighting: LightingSystem::new(),
            archguard: ArchGuard::new(),
            start_time: Instant::now(),
            trauma_mode: false,
            show_debug: true,
            point_cloud_data: Vec::new(),
        }
    }
}

impl eframe::App for EngineUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let delta_time = ctx.input(|i| i.stable_dt);
        let elapsed = self.start_time.elapsed().as_secs_f64();
        
        // Update world
        self.world.trauma_mode = self.trauma_mode;
        self.world.update(delta_time);
        
        // Update lighting
        self.lighting.update_lighting(elapsed as f32);
        
        // Update rhythm detector
        self.archguard.update_rhythm(elapsed);
        
        // Get point cloud data
        self.point_cloud_data = self.world.get_point_cloud_data();
        
        // UI
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Adaptive Entity Engine v1.0");
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.trauma_mode, "Trauma Mode");
                ui.checkbox(&mut self.show_debug, "Show Debug");
            });
            
            ui.separator();
            
            // Stats
            ui.label(format!("Voxels: {}", self.world.voxels.len()));
            ui.label(format!("Points: {}", self.point_cloud_data.len()));
            ui.label(format!("FPS: {:.1}", 1.0 / delta_time));
            ui.label(format!("Time: {:.2}s", elapsed));
            
            // ArchGuard stats
            ui.separator();
            ui.heading("ArchGuard Enterprise");
            ui.label(format!("Circuit Open: {}", 
                self.archguard.circuit_open.load(Ordering::Acquire)));
            
            let empathy = pollster::block_on(self.archguard.get_empathy_ratio());
            ui.label(format!("Empathy Ratio: {:.3}", empathy));
            
            let rhythm_phase = self.archguard.get_rhythm_phase();
            ui.label(format!("Rhythm Phase (0.038 Hz): {:.3}", rhythm_phase));
            
            // Evolution controls
            ui.separator();
            ui.heading("Evolution");
            ui.label(format!("Mutation Rate: {:.2}", self.evolution.mutation_rate));
            ui.label(format!("Crossover Rate: {:.2}", self.evolution.crossover_rate));
            
            if ui.button("Evolve Population").clicked() {
                // Evolve voxels (would need mutable access to voxel data)
            }
            
            // Lighting controls
            ui.separator();
            ui.heading("Lighting");
            ui.label(format!("Light Patterns: {}", self.lighting.patterns.len()));
            
            if ui.button("Add Light Pattern").clicked() {
                self.lighting.add_pattern(Default::default());
            }
            
            // Point cloud visualization (simplified - would use custom rendering in real implementation)
            ui.separator();
            ui.heading("Point Cloud Visualization");
            if !self.point_cloud_data.is_empty() {
                let max_points_display = 1000.min(self.point_cloud_data.len());
                ui.label(format!("Displaying first {} points", max_points_display));
                
                // Simple 2D projection visualization
                let (rect, _) = ui.allocate_exact_size(
                    egui::Vec2::new(800.0, 600.0),
                    egui::Sense::hover()
                );
                
                let painter = ui.painter();
                for (pos, color) in self.point_cloud_data.iter().take(max_points_display) {
                    // Simple 2D projection
                    let x = rect.min.x + (pos[0] * 100.0 + 400.0);
                    let y = rect.min.y + (pos[1] * 100.0 + 300.0);
                    let point = egui::Pos2::new(x, y);
                    let egui_color = egui::Color32::from_rgb(
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    );
                    painter.circle_filled(point, 1.0, egui_color);
                }
            }
            
            // Debug info
            if self.show_debug {
                ui.separator();
                ui.heading("Debug Info");
                ui.label("Renderer: wgpu (Vulkan) via eframe");
                ui.label(format!("Max Points: {}", self.world.max_points));
                ui.label(format!("Voxel Size: ~{} bytes", 
                    if !self.world.voxels.is_empty() {
                        // Estimate
                        "9-13 KB"
                    } else {
                        "N/A"
                    }));
            }
        });
        
        // Request repaint
        ctx.request_repaint();
    }
}
