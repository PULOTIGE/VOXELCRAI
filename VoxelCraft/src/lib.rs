// VoxelCraft - Minecraft-like game for Android
// Simplified version for debugging

pub mod world;
pub mod player;
pub mod crafting;
pub mod entities;
pub mod ui;
pub mod rendering;
pub mod save;

use std::sync::Arc;
use winit::event::{Event, WindowEvent, Touch};
use winit::event_loop::ControlFlow;
use winit::window::{Window, WindowBuilder};

pub use world::{World, Chunk, BlockType, Biome};
pub use player::{Player, Inventory, ItemStack};
pub use crafting::{Recipe, CraftingSystem};
pub use entities::{Entity, EntityType, Mob, MobAI};
pub use ui::{GameUI, UIElement, TouchInput};
pub use rendering::Renderer;
pub use save::{SaveSystem, WorldSave};

// Re-export Camera if it exists
pub use rendering::Camera;

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
            time_of_day: 0.25,
            day_count: 1,
            paused: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        self.time_of_day += dt / 600.0;
        if self.time_of_day >= 1.0 {
            self.time_of_day = 0.0;
            self.day_count += 1;
        }

        let player_chunk = self.world.world_to_chunk(self.player.position);
        self.world.update_around(player_chunk, 3);
        self.player.update(dt, &self.world);

        for entity in &mut self.entities {
            entity.update(dt, &self.world, &self.player);
        }

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
            0.15
        } else {
            let noon_factor = (self.time_of_day - 0.5).abs() * 2.0;
            0.3 + (1.0 - noon_factor) * 0.7
        }
    }
}

/// Main game
pub struct Game {
    pub state: GameState,
    pub renderer: Option<Renderer>,
    pub ui: GameUI,
    pub touch_input: TouchInput,
    pub save_system: SaveSystem,
    pub initialized: bool,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        log::info!("Creating new game with seed: {}", seed);
        Self {
            state: GameState::new(seed),
            renderer: None,
            ui: GameUI::new(),
            touch_input: TouchInput::new(),
            save_system: SaveSystem::new(),
            initialized: false,
        }
    }

    pub fn init_renderer(&mut self, window: Arc<Window>) {
        log::info!("Initializing renderer...");
        
        match pollster::block_on(async {
            Renderer::try_new(window).await
        }) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
                self.initialized = true;
                log::info!("Renderer initialized successfully!");
            }
            Err(e) => {
                log::error!("Failed to create renderer: {}", e);
                self.initialized = false;
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        if !self.initialized {
            return;
        }
        
        if let Some(move_dir) = self.touch_input.get_movement() {
            self.state.player.move_direction(move_dir, dt);
        }

        if let Some(look_delta) = self.touch_input.get_look_delta() {
            self.state.player.rotate(look_delta.x * 0.01, look_delta.y * 0.01);
        }

        if self.touch_input.is_jump_pressed() {
            self.state.player.jump();
        }

        self.state.update(dt);
        self.ui.update(&self.state, &self.touch_input);
    }

    pub fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.state, &self.ui);
        }
    }

    pub fn handle_touch(&mut self, touch: Touch) {
        self.touch_input.handle_touch(touch, self.ui.get_screen_size());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(renderer) = &mut self.renderer {
                renderer.resize(width, height);
            }
            self.ui.resize(width, height);
        }
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

// ============= ANDROID ENTRY POINT =============
#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // Initialize logging first
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("VoxelCraft"),
    );
    
    log::info!("========================================");
    log::info!("=== VoxelCraft v1.0.5 Starting ===");
    log::info!("========================================");
    
    if let Err(e) = run_with_winit(app) {
        log::error!("Game crashed: {}", e);
    }
}

#[cfg(target_os = "android")]
fn run_with_winit(app: winit::platform::android::activity::AndroidApp) -> Result<(), String> {
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::android::EventLoopBuilderExtAndroid;
    
    log::info!("Step 1: Creating event loop...");
    
    let event_loop = EventLoopBuilder::new()
        .with_android_app(app)
        .build()
        .map_err(|e| format!("EventLoop error: {}", e))?;
    
    log::info!("Step 2: Event loop created!");
    
    let mut game: Option<Game> = None;
    let mut window: Option<Arc<Window>> = None;
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::Resumed => {
                log::info!("=== RESUMED EVENT ===");
                
                if window.is_none() {
                    log::info!("Creating window...");
                    
                    match WindowBuilder::new()
                        .with_title("VoxelCraft")
                        .build(target) 
                    {
                        Ok(w) => {
                            let size = w.inner_size();
                            log::info!("Window created: {}x{}", size.width, size.height);
                            let w = Arc::new(w);
                            
                            if game.is_none() {
                                log::info!("Creating game instance...");
                                let mut g = Game::new(12345);
                                log::info!("Initializing renderer...");
                                g.init_renderer(w.clone());
                                
                                if g.initialized {
                                    log::info!("Game fully initialized!");
                                } else {
                                    log::error!("Game initialization failed!");
                                }
                                
                                game = Some(g);
                            } else if let Some(ref mut g) = game {
                                log::info!("Re-initializing renderer...");
                                g.init_renderer(w.clone());
                            }
                            
                            window = Some(w);
                        }
                        Err(e) => {
                            log::error!("Failed to create window: {}", e);
                        }
                    }
                }
            }
            
            Event::Suspended => {
                log::info!("=== SUSPENDED EVENT ===");
                if let Some(ref g) = game {
                    g.save();
                }
                if let Some(ref mut g) = game {
                    g.renderer = None;
                    g.initialized = false;
                }
                window = None;
            }
            
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested");
                        if let Some(ref g) = game {
                            g.save();
                        }
                        target.exit();
                    }
                    
                    WindowEvent::Resized(size) => {
                        log::info!("Resized to {}x{}", size.width, size.height);
                        if let Some(ref mut g) = game {
                            g.resize(size.width, size.height);
                        }
                    }
                    
                    WindowEvent::Touch(touch) => {
                        if let Some(ref mut g) = game {
                            g.handle_touch(touch);
                        }
                    }
                    
                    WindowEvent::RedrawRequested => {
                        let now = std::time::Instant::now();
                        let dt = (now - last_time).as_secs_f32().min(0.1);
                        last_time = now;

                        if let Some(ref mut g) = game {
                            if g.initialized {
                                g.update(dt);
                                g.render();
                            }
                        }
                        
                        if let Some(ref w) = window {
                            w.request_redraw();
                        }
                    }
                    
                    _ => {}
                }
            }
            
            Event::AboutToWait => {
                if let Some(ref w) = window {
                    w.request_redraw();
                }
            }
            
            _ => {}
        }
    }).map_err(|e| format!("Event loop error: {}", e))
}

// ============= DESKTOP ENTRY POINT =============
#[cfg(not(target_os = "android"))]
pub fn run_game() {
    use winit::event_loop::EventLoop;
    
    env_logger::init();
    log::info!("Starting VoxelCraft (Desktop)");
    
    let event_loop = EventLoop::new().unwrap();
    
    let window = WindowBuilder::new()
        .with_title("VoxelCraft")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let window = Arc::new(window);
    
    let mut game = Game::new(rand::random());
    game.init_renderer(window.clone());
    game.load();

    let mut last_time = std::time::Instant::now();

    let _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    game.save();
                    target.exit();
                }
                
                WindowEvent::Resized(size) => {
                    game.resize(size.width, size.height);
                }
                
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    game.update(dt.min(0.1));
                    game.render();
                    
                    window.request_redraw();
                }
                
                _ => {}
            },
            
            Event::AboutToWait => {
                window.request_redraw();
            }
            
            _ => {}
        }
    });
}
