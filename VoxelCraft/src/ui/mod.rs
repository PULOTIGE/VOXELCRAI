// VoxelCraft - Mobile UI System

use crate::GameState;
use glam::Vec2;
use winit::event::{Touch, TouchPhase};

/// Touch input handler for mobile controls
pub struct TouchInput {
    // Touch tracking
    touches: Vec<ActiveTouch>,
    
    // Virtual joystick
    joystick_center: Option<Vec2>,
    joystick_current: Option<Vec2>,
    joystick_touch_id: Option<u64>,
    
    // Look area
    look_start: Option<Vec2>,
    look_current: Option<Vec2>,
    look_touch_id: Option<u64>,
    look_delta: Vec2,
    
    // Buttons
    jump_pressed: bool,
    action_pressed: bool,
    break_mode: bool,
    
    screen_size: Vec2,
}

#[derive(Clone)]
struct ActiveTouch {
    id: u64,
    position: Vec2,
    start_position: Vec2,
    phase: TouchPhase,
}

impl TouchInput {
    pub fn new() -> Self {
        Self {
            touches: Vec::new(),
            joystick_center: None,
            joystick_current: None,
            joystick_touch_id: None,
            look_start: None,
            look_current: None,
            look_touch_id: None,
            look_delta: Vec2::ZERO,
            jump_pressed: false,
            action_pressed: false,
            break_mode: true,
            screen_size: Vec2::new(1280.0, 720.0),
        }
    }

    pub fn handle_touch(&mut self, touch: Touch, screen_size: Vec2) {
        self.screen_size = screen_size;
        let pos = Vec2::new(touch.location.x as f32, touch.location.y as f32);
        let normalized = pos / screen_size;

        match touch.phase {
            TouchPhase::Started => self.on_touch_start(touch.id, pos, normalized),
            TouchPhase::Moved => self.on_touch_move(touch.id, pos, normalized),
            TouchPhase::Ended | TouchPhase::Cancelled => self.on_touch_end(touch.id),
        }
    }

    fn on_touch_start(&mut self, id: u64, pos: Vec2, normalized: Vec2) {
        self.touches.push(ActiveTouch {
            id,
            position: pos,
            start_position: pos,
            phase: TouchPhase::Started,
        });

        // Left side = joystick
        if normalized.x < 0.3 && normalized.y > 0.5 {
            self.joystick_center = Some(pos);
            self.joystick_current = Some(pos);
            self.joystick_touch_id = Some(id);
        }
        // Right side = look/camera
        else if normalized.x > 0.4 {
            self.look_start = Some(pos);
            self.look_current = Some(pos);
            self.look_touch_id = Some(id);
        }

        // Jump button (bottom right)
        if normalized.x > 0.85 && normalized.y > 0.7 {
            self.jump_pressed = true;
        }

        // Action button (center right)
        if normalized.x > 0.85 && normalized.y > 0.4 && normalized.y < 0.7 {
            self.action_pressed = true;
        }

        // Toggle break/place (top right)
        if normalized.x > 0.85 && normalized.y < 0.2 {
            self.break_mode = !self.break_mode;
        }
    }

    fn on_touch_move(&mut self, id: u64, pos: Vec2, _normalized: Vec2) {
        // Update touch position
        for touch in &mut self.touches {
            if touch.id == id {
                touch.position = pos;
                touch.phase = TouchPhase::Moved;
            }
        }

        // Update joystick
        if Some(id) == self.joystick_touch_id {
            self.joystick_current = Some(pos);
        }

        // Update look
        if Some(id) == self.look_touch_id {
            if let (Some(start), Some(current)) = (self.look_start, self.look_current) {
                self.look_delta = pos - current;
            }
            self.look_current = Some(pos);
        }
    }

    fn on_touch_end(&mut self, id: u64) {
        self.touches.retain(|t| t.id != id);

        if Some(id) == self.joystick_touch_id {
            self.joystick_center = None;
            self.joystick_current = None;
            self.joystick_touch_id = None;
        }

        if Some(id) == self.look_touch_id {
            self.look_start = None;
            self.look_current = None;
            self.look_touch_id = None;
            self.look_delta = Vec2::ZERO;
        }

        self.jump_pressed = false;
        self.action_pressed = false;
    }

    pub fn get_movement(&self) -> Option<glam::Vec3> {
        if let (Some(center), Some(current)) = (self.joystick_center, self.joystick_current) {
            let delta = current - center;
            let max_radius = 80.0;
            let normalized = delta / max_radius;
            
            if normalized.length() > 0.1 {
                return Some(glam::Vec3::new(
                    normalized.x.clamp(-1.0, 1.0),
                    0.0,
                    normalized.y.clamp(-1.0, 1.0),
                ));
            }
        }
        None
    }

    pub fn get_look_delta(&self) -> Option<Vec2> {
        if self.look_delta.length() > 0.1 {
            Some(self.look_delta)
        } else {
            None
        }
    }

    pub fn is_jump_pressed(&self) -> bool {
        self.jump_pressed
    }

    pub fn is_action_pressed(&self) -> bool {
        self.action_pressed
    }

    pub fn is_break_mode(&self) -> bool {
        self.break_mode
    }

    pub fn clear_frame(&mut self) {
        self.look_delta = Vec2::ZERO;
        self.action_pressed = false;
    }
}

impl Default for TouchInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Game UI elements
pub struct GameUI {
    screen_width: u32,
    screen_height: u32,
    pub elements: Vec<UIElement>,
    pub show_inventory: bool,
    pub show_crafting: bool,
    pub show_pause: bool,
}

impl GameUI {
    pub fn new() -> Self {
        Self {
            screen_width: 1280,
            screen_height: 720,
            elements: Vec::new(),
            show_inventory: false,
            show_crafting: false,
            show_pause: false,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_width = width;
        self.screen_height = height;
        self.rebuild_elements();
    }

    pub fn get_screen_size(&self) -> Vec2 {
        Vec2::new(self.screen_width as f32, self.screen_height as f32)
    }

    fn rebuild_elements(&mut self) {
        self.elements.clear();
        
        let w = self.screen_width as f32;
        let h = self.screen_height as f32;
        
        // Crosshair
        self.elements.push(UIElement::Crosshair {
            x: w / 2.0,
            y: h / 2.0,
            size: 20.0,
        });

        // Hotbar
        self.elements.push(UIElement::Hotbar {
            x: w / 2.0 - 180.0,
            y: h - 50.0,
            slot_size: 40.0,
        });

        // Health bar
        self.elements.push(UIElement::HealthBar {
            x: 10.0,
            y: h - 30.0,
            width: 200.0,
            height: 20.0,
        });

        // Hunger bar
        self.elements.push(UIElement::HungerBar {
            x: w - 210.0,
            y: h - 30.0,
            width: 200.0,
            height: 20.0,
        });

        // Virtual joystick area
        self.elements.push(UIElement::Joystick {
            x: 100.0,
            y: h - 150.0,
            radius: 80.0,
        });

        // Jump button
        self.elements.push(UIElement::Button {
            x: w - 80.0,
            y: h - 150.0,
            width: 60.0,
            height: 60.0,
            label: "↑".to_string(),
            action: ButtonAction::Jump,
        });

        // Action button
        self.elements.push(UIElement::Button {
            x: w - 80.0,
            y: h - 250.0,
            width: 60.0,
            height: 60.0,
            label: "⛏".to_string(),
            action: ButtonAction::Action,
        });

        // Inventory button
        self.elements.push(UIElement::Button {
            x: w - 80.0,
            y: 20.0,
            width: 60.0,
            height: 40.0,
            label: "≡".to_string(),
            action: ButtonAction::Inventory,
        });
    }

    pub fn update(&mut self, state: &GameState, input: &TouchInput) {
        // Update UI based on game state
    }

    pub fn handle_tap(&mut self, x: f32, y: f32) -> Option<ButtonAction> {
        for element in &self.elements {
            if let UIElement::Button { x: bx, y: by, width, height, action, .. } = element {
                if x >= *bx && x <= bx + width && y >= *by && y <= by + height {
                    return Some(*action);
                }
            }
        }
        None
    }
}

impl Default for GameUI {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub enum UIElement {
    Crosshair { x: f32, y: f32, size: f32 },
    Hotbar { x: f32, y: f32, slot_size: f32 },
    HealthBar { x: f32, y: f32, width: f32, height: f32 },
    HungerBar { x: f32, y: f32, width: f32, height: f32 },
    Joystick { x: f32, y: f32, radius: f32 },
    Button { x: f32, y: f32, width: f32, height: f32, label: String, action: ButtonAction },
    Text { x: f32, y: f32, text: String, size: f32 },
    Panel { x: f32, y: f32, width: f32, height: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonAction {
    Jump,
    Action,
    Inventory,
    Pause,
    Craft,
    ToggleMode,
}
