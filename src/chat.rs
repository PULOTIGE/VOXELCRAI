use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub text: String,
    pub sender: MessageSender,
    pub timestamp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSender {
    User,
    Assistant,
    System,
}

pub struct ChatSystem {
    messages: VecDeque<ChatMessage>,
    input_text: String,
    max_messages: usize,
    scroll_to_bottom: bool,
}

impl ChatSystem {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            input_text: String::new(),
            max_messages: 1000,
            scroll_to_bottom: true,
        }
    }

    pub fn add_message(&mut self, text: String, sender: MessageSender) {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            sender,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        };

        self.messages.push_back(message);
        
        // Ограничиваем количество сообщений
        while self.messages.len() > self.max_messages {
            self.messages.pop_front();
        }
        
        self.scroll_to_bottom = true;
    }

    pub fn show_ui(&mut self, ui: &mut egui::Ui) {
        // Стиль в белых тонах
        let chat_bg = egui::Color32::from_rgb(255, 255, 255);
        let user_msg_bg = egui::Color32::from_rgb(240, 240, 250);
        let assistant_msg_bg = egui::Color32::from_rgb(250, 250, 255);
        let border_color = egui::Color32::from_rgb(230, 230, 230);
        let text_color = egui::Color32::from_rgb(30, 30, 30);

        // Область чата
        let chat_area = egui::Frame::none()
            .fill(chat_bg)
            .stroke(egui::Stroke::new(1.0, border_color))
            .inner_margin(egui::Margin::same(10.0));

        chat_area.show(ui, |ui| {
            // Заголовок чата
            ui.horizontal(|ui| {
                ui.heading("💬 Чат с системой");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Очистить").clicked() {
                        self.messages.clear();
                    }
                });
            });
            
            ui.separator();

            // Область сообщений с прокруткой
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    
                    for message in &self.messages {
                        ui.vertical(|ui| {
                            ui.add_space(5.0);
                            
                            let (bg_color, align) = match message.sender {
                                MessageSender::User => (user_msg_bg, egui::Align::RIGHT),
                                MessageSender::Assistant => (assistant_msg_bg, egui::Align::LEFT),
                                MessageSender::System => (egui::Color32::from_rgb(245, 245, 245), egui::Align::CENTER),
                            };

                            ui.with_layout(
                                egui::Layout::top_down(align),
                                |ui| {
                                    let max_width = ui.available_width() * 0.7;
                                    
                                    egui::Frame::none()
                                        .fill(bg_color)
                                        .stroke(egui::Stroke::new(1.0, border_color))
                                        .rounding(egui::Rounding::same(8.0))
                                        .inner_margin(egui::Margin::same(12.0))
                                        .show(ui, |ui| {
                                            ui.set_max_width(max_width);
                                            
                                            // Имя отправителя
                                            let sender_name = match message.sender {
                                                MessageSender::User => "Вы",
                                                MessageSender::Assistant => "Ассистент",
                                                MessageSender::System => "Система",
                                            };
                                            
                                            ui.label(
                                                egui::RichText::new(sender_name)
                                                    .color(text_color)
                                                    .strong()
                                            );
                                            
                                            ui.add_space(4.0);
                                            
                                            // Текст сообщения
                                            ui.label(
                                                egui::RichText::new(&message.text)
                                                    .color(text_color)
                                            );
                                            
                                            // Время
                                            let time = std::time::UNIX_EPOCH
                                                + std::time::Duration::from_secs_f64(message.timestamp);
                                            #[cfg(feature = "gui")]
                                            let time_str = {
                                                let datetime = chrono::DateTime::<chrono::Utc>::from(time);
                                                datetime.format("%H:%M:%S").to_string()
                                            };
                                            #[cfg(not(feature = "gui"))]
                                            let time_str = format!("{:.0}", message.timestamp);
                                            
                                            ui.add_space(4.0);
                                            ui.label(
                                                egui::RichText::new(time_str)
                                                    .color(egui::Color32::from_rgb(150, 150, 150))
                                                    .small()
                                            );
                                        });
                                }
                            );
                            
                            ui.add_space(5.0);
                        });
                    }
                    
                    // Автопрокрутка вниз
                    if self.scroll_to_bottom {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.scroll_to_bottom = false;
                    }
                });

            ui.separator();

            // Поле ввода
            ui.horizontal(|ui| {
                let text_edit = egui::TextEdit::multiline(&mut self.input_text)
                    .desired_width(ui.available_width() - 100.0)
                    .desired_rows(3)
                    .hint_text("Введите сообщение...");

                if ui.add(text_edit).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !self.input_text.trim().is_empty() {
                        let text = self.input_text.trim().to_string();
                        self.input_text.clear();
                        self.add_message(text.clone(), MessageSender::User);
                        
                        // Симуляция ответа ассистента (замените на реальную логику)
                        self.add_message(
                            format!("Получено: {}", text),
                            MessageSender::Assistant,
                        );
                    }
                }

                if ui.button("Отправить").clicked() {
                    if !self.input_text.trim().is_empty() {
                        let text = self.input_text.trim().to_string();
                        self.input_text.clear();
                        self.add_message(text.clone(), MessageSender::User);
                        
                        // Симуляция ответа ассистента
                        self.add_message(
                            format!("Получено: {}", text),
                            MessageSender::Assistant,
                        );
                    }
                }
            });
        });
    }

    pub fn get_messages(&self) -> &VecDeque<ChatMessage> {
        &self.messages
    }
}

impl Default for ChatSystem {
    fn default() -> Self {
        Self::new()
    }
}
