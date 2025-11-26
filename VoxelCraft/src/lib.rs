// VoxelCraft - Minecraft-like game for Android
// Main library module

pub mod world;
pub mod player;
pub mod crafting;
pub mod entities;
pub mod ui;
pub mod rendering;
pub mod save;

use std::sync::Arc;
use winit::event::{Event, WindowEvent, Touch};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub use world::{World, Chunk, BlockType, Biome};
pub use player::{Player, Inventory, ItemStack};
pub use crafting::{Recipe, CraftingSystem};
pub use entities::{Entity, EntityType, Mob, MobAI};
pub use ui::{GameUI, UIElement, TouchInput};
pub use rendering::{Renderer, Camera};
pub use save::{SaveSystem, WorldSave};

/// Game state
pub struct GameState {
    pub world: World,
    pub player: Player,
    pub crafting: CraftingSystem,
    pub entities: Vec<Box<dyn Entity>>,
    pub time_of_day: f32,
    pub day_count: u32,
    pub paused: bool,
}

impl GameState {
    pub fn new(seed: u64) -> Self {
        Self {
            world: World::new(seed),
            player: Player::new(),
            crafting: CraftingSystem::new(),
            entities: Vec::new(),
            time_of_day: 0.25, // Start at morning
            day_count: 1,
            paused: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        // Update time
        self.time_of_day += dt / 600.0; // 10 minute day cycle
        if self.time_of_day >= 1.0 {
            self.time_of_day = 0.0;
            self.day_count += 1;
        }

        // Update world (generate chunks around player)
        let player_chunk = self.world.world_to_chunk(self.player.position);
        self.world.update_around(player_chunk, 4);

        // Update player
        self.player.update(dt, &self.world);

        // Update entities
        for entity in &mut self.entities {
            entity.update(dt, &self.world, &self.player);
        }

        // Spawn mobs at night
        if self.is_night() {
            self.spawn_night_mobs();
        }

        // Remove dead entities
        self.entities.retain(|e| e.is_alive());
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day > 0.75 || self.time_of_day < 0.25
    }

    pub fn get_sun_direction(&self) -> glam::Vec3 {
        let angle = self.time_of_day * std::f32::consts::PI * 2.0;
        glam::Vec3::new(angle.cos(), angle.sin().abs().max(0.1), 0.3).normalize()
    }

    pub fn get_ambient_light(&self) -> f32 {
        if self.is_night() {
            0.1
        } else {
            let noon_factor = (self.time_of_day - 0.5).abs() * 2.0;
            0.3 + (1.0 - noon_factor) * 0.7
        }
    }

    fn spawn_night_mobs(&mut self) {
        // Limit mob count
        if self.entities.len() >= 50 {
            return;
        }

        // Random spawn chance
        if rand::random::<f32>() > 0.01 {
            return;
        }

        // Spawn zombie near player
        let offset = glam::Vec3::new(
            (rand::random::<f32>() - 0.5) * 40.0,
            0.0,
            (rand::random::<f32>() - 0.5) * 40.0,
        );

        let spawn_pos = self.player.position + offset;
        let ground_y = self.world.get_height_at(spawn_pos.x as i32, spawn_pos.z as i32) as f32 + 1.0;

        if ground_y > 0.0 {
            let zombie = Box::new(Mob::new(
                EntityType::Zombie,
                glam::Vec3::new(spawn_pos.x, ground_y, spawn_pos.z),
            ));
            self.entities.push(zombie);
        }
    }
}

/// Main game entry point
pub struct Game {
    pub state: GameState,
    pub renderer: Option<Renderer>,
    pub ui: GameUI,
    pub touch_input: TouchInput,
    pub save_system: SaveSystem,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        Self {
            state: GameState::new(seed),
            renderer: None,
            ui: GameUI::new(),
            touch_input: TouchInput::new(),
            save_system: SaveSystem::new(),
        }
    }

    pub fn init_renderer(&mut self, window: Arc<Window>) {
        self.renderer = Some(pollster::block_on(Renderer::new(window)));
    }

    pub fn update(&mut self, dt: f32) {
        // Process touch input
        self.process_input(dt);

        // Update game state
        self.state.update(dt);

        // Update UI
        self.ui.update(&self.state, &self.touch_input);
    }

    pub fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.state, &self.ui);
        }
    }

    fn process_input(&mut self, dt: f32) {
        // Movement from joystick
        if let Some(move_dir) = self.touch_input.get_movement() {
            self.state.player.move_direction(move_dir, dt);
        }

        // Camera from look area
        if let Some(look_delta) = self.touch_input.get_look_delta() {
            self.state.player.rotate(look_delta.x * 0.01, look_delta.y * 0.01);
        }

        // Jump button
        if self.touch_input.is_jump_pressed() {
            self.state.player.jump();
        }

        // Break/Place block
        if self.touch_input.is_action_pressed() {
            self.handle_block_action();
        }
    }

    fn handle_block_action(&mut self) {
        let (hit, pos, face) = self.state.world.raycast(
            self.state.player.get_eye_position(),
            self.state.player.get_look_direction(),
            5.0,
        );

        if hit {
            if self.touch_input.is_break_mode() {
                // Break block
                if let Some(block) = self.state.world.get_block(pos.x, pos.y, pos.z) {
                    // Add to inventory
                    self.state.player.inventory.add_item(ItemStack::from_block(block));
                    self.state.world.set_block(pos.x, pos.y, pos.z, BlockType::Air);
                }
            } else {
                // Place block
                let place_pos = pos + face;
                if let Some(item) = self.state.player.inventory.get_selected() {
                    if let Some(block_type) = item.as_block() {
                        self.state.world.set_block(place_pos.x, place_pos.y, place_pos.z, block_type);
                        self.state.player.inventory.consume_selected();
                    }
                }
            }
        }
    }

    pub fn handle_touch(&mut self, touch: Touch) {
        self.touch_input.handle_touch(touch, self.ui.get_screen_size());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
        self.ui.resize(width, height);
    }

    pub fn save(&self) {
        let _ = self.save_system.save(&self.state);
    }

    pub fn load(&mut self) -> bool {
        if let Some(state) = self.save_system.load() {
            self.state = state;
            true
        } else {
            false
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(_app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info),
    );
    
    log::info!("VoxelCraft starting...");
    
    // Run game
    run_game();
}

pub fn run_game() {
    let event_loop = EventLoop::new().unwrap();
    
    #[cfg(target_os = "android")]
    let window = WindowBuilder::new()
        .with_title("VoxelCraft")
        .build(&event_loop)
        .unwrap();
    
    #[cfg(not(target_os = "android"))]
    let window = WindowBuilder::new()
        .with_title("VoxelCraft")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);
    
    let mut game = Game::new(rand::random());
    game.init_renderer(window.clone());
    
    // Try to load saved game
    game.load();

    let mut last_time = std::time::Instant::now();

    let _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                
                WindowEvent::Resized(size) => {
                    game.resize(size.width, size.height);
                }
                
                WindowEvent::Touch(touch) => {
                    game.handle_touch(touch);
                }
                
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    game.update(dt.min(0.1)); // Cap dt to avoid physics issues
                    game.render();
                    
                    window.request_redraw();
                }
                
                _ => {}
            },
            
            Event::Suspended => {
                game.save();
            }
            
            Event::Resumed => {
                window.request_redraw();
            }
            
            _ => {}
        }
    });
}
