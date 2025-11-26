use crate::archguard::ArchGuard;
use crate::chat::{ChatSystem, MessageSender};
use crate::evolution::EvolutionEngine;
use crate::integrations::IntegrationManager;
use crate::learning::LearningMode;
use crate::lighting::LightingSystem;
use crate::voxel::VoxelWorld;
use eframe::egui;
use std::sync::atomic::Ordering;
use std::time::Instant;

#[derive(PartialEq)]
enum ViewMode {
    Engine,
    Chat,
    Learning,
    Integrations,
}

pub struct EngineUI {
    world: VoxelWorld,
    evolution: EvolutionEngine,
    lighting: LightingSystem,
    archguard: ArchGuard,
    chat: ChatSystem,
    learning: LearningMode,
    integrations: IntegrationManager,
    start_time: Instant,
    trauma_mode: bool,
    show_debug: bool,
    point_cloud_data: Vec<([f32; 3], [f32; 3])>,
    current_view: ViewMode,
    integration_api_key_input: String,
    selected_integration: Option<String>,
}

impl EngineUI {
    pub fn new() -> Self {
        let mut chat = ChatSystem::new();
        chat.add_message(
            "Добро пожаловать в Adaptive Entity Engine! Я готов помочь вам с работой в системе.".to_string(),
            MessageSender::System,
        );

        Self {
            world: VoxelWorld::new(),
            evolution: EvolutionEngine::new(),
            lighting: LightingSystem::new(),
            archguard: ArchGuard::new(),
            chat,
            learning: LearningMode::new(),
            integrations: IntegrationManager::new(),
            start_time: Instant::now(),
            trauma_mode: false,
            show_debug: true,
            point_cloud_data: Vec::new(),
            current_view: ViewMode::Chat,
            integration_api_key_input: String::new(),
            selected_integration: None,
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
        
        // Боковая панель навигации
        egui::SidePanel::left("navigation")
            .resizable(true)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("🎯 Навигация");
                ui.separator();
                
                if ui.selectable_label(self.current_view == ViewMode::Chat, "💬 Чат").clicked() {
                    self.current_view = ViewMode::Chat;
                }
                if ui.selectable_label(self.current_view == ViewMode::Learning, "📚 Обучение").clicked() {
                    self.current_view = ViewMode::Learning;
                }
                if ui.selectable_label(self.current_view == ViewMode::Integrations, "🔌 Интеграции").clicked() {
                    self.current_view = ViewMode::Integrations;
                }
                if ui.selectable_label(self.current_view == ViewMode::Engine, "⚙️ Движок").clicked() {
                    self.current_view = ViewMode::Engine;
                }
                
                ui.separator();
                ui.label("⚙️ Настройки");
                ui.checkbox(&mut self.trauma_mode, "Trauma Mode");
                ui.checkbox(&mut self.show_debug, "Показать Debug");
            });
        
        // Основная панель контента
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(egui::Color32::from_rgb(255, 255, 255)))
            .show(ctx, |ui| {
                match self.current_view {
                    ViewMode::Chat => {
                        self.chat.show_ui(ui);
                    }
                    ViewMode::Learning => {
                        self.show_learning_ui(ui);
                    }
                    ViewMode::Integrations => {
                        self.show_integrations_ui(ui);
                    }
                    ViewMode::Engine => {
                        self.show_engine_ui(ui);
                    }
                }
            });
        
        // Request repaint
        ctx.request_repaint();
    }
}

impl EngineUI {
    fn show_engine_ui(&mut self, ui: &mut egui::Ui) {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let delta_time = ui.input(|i| i.stable_dt);
        
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
    }

    fn show_learning_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("📚 Режим обучения");
        ui.separator();

        // Загрузка файлов
        ui.horizontal(|ui| {
            if ui.button("📁 Загрузить текстовый файл").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Текстовые файлы", &["txt", "md", "pdf", "doc", "docx", "rtf"])
                    .pick_file()
                {
                    match self.learning.upload_file(&path) {
                        Ok(file_id) => {
                            self.chat.add_message(
                                format!("Файл '{}' успешно загружен", path.file_name().unwrap_or_default().to_string_lossy()),
                                MessageSender::System,
                            );
                        }
                        Err(e) => {
                            self.chat.add_message(
                                format!("Ошибка загрузки файла: {}", e),
                                MessageSender::System,
                            );
                        }
                    }
                }
            }

            if ui.button("🎬 Загрузить видео").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Видео файлы", &["mp4", "avi", "mov", "mkv", "webm", "flv"])
                    .pick_file()
                {
                    match self.learning.upload_file(&path) {
                        Ok(file_id) => {
                            self.chat.add_message(
                                format!("Видео '{}' успешно загружено", path.file_name().unwrap_or_default().to_string_lossy()),
                                MessageSender::System,
                            );
                        }
                        Err(e) => {
                            self.chat.add_message(
                                format!("Ошибка загрузки видео: {}", e),
                                MessageSender::System,
                            );
                        }
                    }
                }
            }

            if ui.button("📄 Загрузить любой файл").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    match self.learning.upload_file(&path) {
                        Ok(_) => {
                            self.chat.add_message(
                                format!("Файл '{}' успешно загружен", path.file_name().unwrap_or_default().to_string_lossy()),
                                MessageSender::System,
                            );
                        }
                        Err(e) => {
                            self.chat.add_message(
                                format!("Ошибка загрузки: {}", e),
                                MessageSender::System,
                            );
                        }
                    }
                }
            }
        });

        ui.separator();

        // Список загруженных файлов
        ui.heading("Загруженные файлы");
        
        if self.learning.list_files().is_empty() {
            ui.label(egui::RichText::new("Нет загруженных файлов").color(egui::Color32::GRAY));
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for file in self.learning.list_files() {
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(250, 250, 255))
                            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 230, 230)))
                            .inner_margin(egui::Margin::same(10.0))
                            .rounding(egui::Rounding::same(5.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Иконка типа файла
                                    let icon = match file.file_type {
                                        crate::learning::FileType::Text => "📄",
                                        crate::learning::FileType::Video => "🎬",
                                        crate::learning::FileType::Image => "🖼️",
                                        crate::learning::FileType::Audio => "🎵",
                                        _ => "📎",
                                    };
                                    ui.label(icon);

                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(&file.name).strong());
                                        ui.label(format!("Размер: {:.2} МБ", file.size as f64 / (1024.0 * 1024.0)));
                                        
                                        let datetime = std::time::UNIX_EPOCH
                                            + std::time::Duration::from_secs_f64(file.uploaded_at);
                                        let dt = chrono::DateTime::<chrono::Utc>::from(datetime);
                                        ui.label(format!("Загружено: {}", dt.format("%Y-%m-%d %H:%M:%S")));
                                    });

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("🗑️").clicked() {
                                            if let Err(e) = self.learning.remove_file(&file.id) {
                                                self.chat.add_message(
                                                    format!("Ошибка удаления: {}", e),
                                                    MessageSender::System,
                                                );
                                            } else {
                                                self.chat.add_message(
                                                    format!("Файл '{}' удален", file.name),
                                                    MessageSender::System,
                                                );
                                            }
                                        }

                                        if matches!(file.file_type, crate::learning::FileType::Text) {
                                            if ui.button("👁️ Просмотр").clicked() {
                                                match self.learning.read_text_file(&file.id) {
                                                    Ok(content) => {
                                                        // Показываем содержимое в отдельном окне или в чате
                                                        let preview = if content.len() > 500 {
                                                            format!("{}...", &content[..500])
                                                        } else {
                                                            content
                                                        };
                                                        self.chat.add_message(
                                                            format!("Содержимое файла '{}':\n\n{}", file.name, preview),
                                                            MessageSender::System,
                                                        );
                                                    }
                                                    Err(e) => {
                                                        self.chat.add_message(
                                                            format!("Ошибка чтения: {}", e),
                                                            MessageSender::System,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            });
                            
                            ui.add_space(5.0);
                        }
                    }
                });
        }

        ui.separator();
        ui.label(format!("Директория загрузок: {}", self.learning.get_upload_directory().display()));
    }

    fn show_integrations_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔌 Интеграции с внешними сервисами");
        ui.separator();

        ui.label(egui::RichText::new("Подключите AI-сервисы для расширенной функциональности чата и обработки данных.")
            .color(egui::Color32::from_rgb(100, 100, 100)));

        ui.separator();

        // Список доступных интеграций
        ui.heading("Доступные интеграции");

        for (id, config) in self.integrations.list_integrations() {
            egui::Frame::none()
                .fill(if config.enabled {
                    egui::Color32::from_rgb(240, 255, 240)
                } else {
                    egui::Color32::from_rgb(250, 250, 250)
                })
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(230, 230, 230)))
                .inner_margin(egui::Margin::same(15.0))
                .rounding(egui::Rounding::same(8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut enabled = config.enabled;
                        ui.checkbox(&mut enabled, &config.name);
                        if enabled != config.enabled {
                            if enabled {
                                let _ = self.integrations.enable_integration(id, config.api_key.clone());
                            } else {
                                self.integrations.disable_integration(id);
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⚙️ Настроить").clicked() {
                                self.selected_integration = Some(id.clone());
                                if let Some(key) = &config.api_key {
                                    self.integration_api_key_input = key.clone();
                                } else {
                                    self.integration_api_key_input.clear();
                                }
                            }
                        });
                    });

                    if config.enabled {
                        ui.add_space(5.0);
                        if let Some(endpoint) = &config.endpoint {
                            ui.label(egui::RichText::new(format!("Endpoint: {}", endpoint))
                                .color(egui::Color32::from_rgb(150, 150, 150))
                                .small());
                        }
                        if config.api_key.is_some() {
                            ui.label(egui::RichText::new("API ключ: ✓ установлен")
                                .color(egui::Color32::from_rgb(0, 150, 0))
                                .small());
                        } else {
                            ui.label(egui::RichText::new("API ключ: не установлен")
                                .color(egui::Color32::from_rgb(200, 0, 0))
                                .small());
                        }
                    }
                });

            ui.add_space(5.0);
        }

        // Диалог настройки интеграции
        if let Some(ref integration_id) = self.selected_integration {
            egui::Window::new("Настройка интеграции")
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    if let Some(config) = self.integrations.get_integration(integration_id) {
                        ui.label(format!("Настройка: {}", config.name));
                        ui.separator();

                        ui.label("API ключ:");
                        ui.text_edit_singleline(&mut self.integration_api_key_input);
                        ui.label(egui::RichText::new("Введите ваш API ключ для доступа к сервису")
                            .color(egui::Color32::from_rgb(150, 150, 150))
                            .small());

                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("💾 Сохранить").clicked() {
                                let api_key = if self.integration_api_key_input.trim().is_empty() {
                                    None
                                } else {
                                    Some(self.integration_api_key_input.clone())
                                };

                                if let Err(e) = self.integrations.enable_integration(integration_id, api_key) {
                                    self.chat.add_message(
                                        format!("Ошибка настройки интеграции: {}", e),
                                        MessageSender::System,
                                    );
                                } else {
                                    self.chat.add_message(
                                        format!("Интеграция '{}' успешно настроена", config.name),
                                        MessageSender::System,
                                    );
                                    self.selected_integration = None;
                                    self.integration_api_key_input.clear();
                                }
                            }

                            if ui.button("❌ Отмена").clicked() {
                                self.selected_integration = None;
                                self.integration_api_key_input.clear();
                            }
                        });
                    }
                });
        }

        ui.separator();
        ui.label(egui::RichText::new("💡 Совет: После настройки интеграций вы сможете использовать их в чате для получения ответов от AI-сервисов.")
            .color(egui::Color32::from_rgb(100, 100, 200)));
    }
}
