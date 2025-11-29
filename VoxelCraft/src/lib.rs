// VoxelCraft - Minecraft-like game for Android
// Pure OpenGL ES version

pub mod world;
pub mod player;
pub mod crafting;
pub mod entities;
pub mod ui;
pub mod rendering;
pub mod save;

use std::sync::Arc;
use std::num::NonZeroU32;
use winit::event::{Event, WindowEvent, Touch};
use winit::event_loop::ControlFlow;
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
        if self.is_night() { 0.15 } else { 0.7 }
    }
}

/// Main game struct
pub struct Game {
    pub state: GameState,
    pub renderer: Renderer,
    pub ui: GameUI,
    pub touch_input: TouchInput,
    pub save_system: SaveSystem,
    pub initialized: bool,
}

impl Game {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        log::info!("Creating new game");
        Self {
            state: GameState::new(seed),
            renderer: Renderer::new(width, height),
            ui: GameUI::new(),
            touch_input: TouchInput::new(),
            save_system: SaveSystem::new(),
            initialized: false,
        }
    }

    pub fn init_gl(&mut self, gl: glow::Context) {
        self.renderer.init_gl(gl);
        self.initialized = true;
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
        self.renderer.render(&self.state, &self.ui);
    }

    pub fn handle_touch(&mut self, touch: Touch) {
        self.touch_input.handle_touch(touch, self.ui.get_screen_size());
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.resize(width, height);
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
    use winit::event_loop::EventLoopBuilder;
    use winit::platform::android::EventLoopBuilderExtAndroid;
    use glutin::config::ConfigTemplateBuilder;
    use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
    use glutin::display::GetGlDisplay;
    use glutin::prelude::*;
    use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
    use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
    
    // Initialize logging
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("VoxelCraft"),
    );
    
    log::info!("========================================");
    log::info!("=== VoxelCraft v1.0.8 OpenGL ES ===");
    log::info!("========================================");
    
    let event_loop = EventLoopBuilder::new()
        .with_android_app(app)
        .build()
        .expect("Failed to create event loop");
    
    log::info!("Event loop created");
    
    struct GlState {
        surface: glutin::surface::Surface<WindowSurface>,
        context: glutin::context::PossiblyCurrentContext,
    }
    
    let mut game: Option<Game> = None;
    let mut window: Option<Arc<Window>> = None;
    let mut gl_state: Option<GlState> = None;
    let mut gl_display: Option<glutin::display::Display> = None;
    let mut last_time = std::time::Instant::now();

    let _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::Resumed => {
                log::info!("=== RESUMED ===");
                
                if window.is_none() {
                    log::info!("Creating window...");
                    
                    let win = WindowBuilder::new()
                        .with_title("VoxelCraft")
                        .build(target)
                        .expect("Failed to create window");
                    
                    let size = win.inner_size();
                    log::info!("Window created: {}x{}", size.width, size.height);
                    
                    // Create OpenGL context
                    log::info!("Creating OpenGL context...");
                    
                    let raw_display = win.raw_display_handle();
                    let raw_window = win.raw_window_handle();
                    
                    let display = unsafe {
                        glutin::display::Display::new(raw_display, glutin::display::DisplayApiPreference::Egl)
                            .expect("Failed to create display")
                    };
                    
                    log::info!("Display created");
                    
                    let config_template = ConfigTemplateBuilder::new()
                        .with_alpha_size(8)
                        .build();
                    
                    let config = unsafe {
                        display.find_configs(config_template)
                            .expect("Failed to find configs")
                            .next()
                            .expect("No configs found")
                    };
                    
                    log::info!("Config selected");
                    
                    let context_attrs = ContextAttributesBuilder::new()
                        .with_context_api(ContextApi::Gles(Some(Version::new(3, 0))))
                        .build(Some(raw_window));
                    
                    let context = unsafe {
                        display.create_context(&config, &context_attrs)
                            .expect("Failed to create context")
                    };
                    
                    log::info!("Context created");
                    
                    let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new()
                        .build(raw_window, 
                            NonZeroU32::new(size.width.max(1)).unwrap(),
                            NonZeroU32::new(size.height.max(1)).unwrap());
                    
                    let surface = unsafe {
                        display.create_window_surface(&config, &surface_attrs)
                            .expect("Failed to create surface")
                    };
                    
                    log::info!("Surface created");
                    
                    let context = context.make_current(&surface)
                        .expect("Failed to make context current");
                    
                    log::info!("Context made current");
                    
                    // Create glow context
                    let gl = unsafe {
                        glow::Context::from_loader_function_cstr(|s| {
                            display.get_proc_address(s) as *const _
                        })
                    };
                    
                    log::info!("Glow context created");
                    
                    // Create game
                    let mut g = Game::new(12345, size.width, size.height);
                    g.init_gl(gl);
                    let _ = g.load();
                    
                    game = Some(g);
                    window = Some(Arc::new(win));
                    gl_state = Some(GlState { surface, context });
                    gl_display = Some(display);
                    
                    log::info!("Game initialized!");
                }
            }
            
            Event::Suspended => {
                log::info!("=== SUSPENDED ===");
                if let Some(ref g) = game {
                    g.save();
                }
                gl_state = None;
                gl_display = None;
                window = None;
                if let Some(ref mut g) = game {
                    g.initialized = false;
                }
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
                        log::info!("Resized: {}x{}", size.width, size.height);
                        if let Some(ref mut g) = game {
                            g.resize(size.width, size.height);
                        }
                        if let Some(ref state) = gl_state {
                            state.surface.resize(&state.context, 
                                NonZeroU32::new(size.width.max(1)).unwrap(),
                                NonZeroU32::new(size.height.max(1)).unwrap());
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
                                
                                if let Some(ref state) = gl_state {
                                    state.surface.swap_buffers(&state.context)
                                        .expect("Failed to swap buffers");
                                }
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
    });
}

// ============= DESKTOP ENTRY POINT =============
#[cfg(not(target_os = "android"))]
pub fn run_game() {
    use winit::event_loop::EventLoop;
    use glutin::config::ConfigTemplateBuilder;
    use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
    use glutin::display::GetGlDisplay;
    use glutin::prelude::*;
    use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
    use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
    
    env_logger::init();
    log::info!("Starting VoxelCraft (Desktop)");
    
    let event_loop = EventLoop::new().unwrap();
    
    let window = WindowBuilder::new()
        .with_title("VoxelCraft")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();
    let raw_display = window.raw_display_handle();
    let raw_window = window.raw_window_handle();
    
    let display = unsafe {
        glutin::display::Display::new(raw_display, glutin::display::DisplayApiPreference::Egl)
            .or_else(|_| glutin::display::Display::new(raw_display, glutin::display::DisplayApiPreference::Wgl))
            .or_else(|_| glutin::display::Display::new(raw_display, glutin::display::DisplayApiPreference::Glx))
            .expect("Failed to create display")
    };
    
    let config = unsafe {
        display.find_configs(ConfigTemplateBuilder::new().build())
            .unwrap().next().unwrap()
    };
    
    let context = unsafe {
        display.create_context(&config, &ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window)))
            .unwrap()
    };
    
    let surface = unsafe {
        display.create_window_surface(&config, 
            &SurfaceAttributesBuilder::<WindowSurface>::new()
                .build(raw_window, 
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap()))
            .unwrap()
    };
    
    let context = context.make_current(&surface).unwrap();
    
    let gl = unsafe {
        glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s) as *const _)
    };
    
    let window = Arc::new(window);
    let mut game = Game::new(rand::random(), size.width, size.height);
    game.init_gl(gl);
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
                    surface.resize(&context,
                        NonZeroU32::new(size.width.max(1)).unwrap(),
                        NonZeroU32::new(size.height.max(1)).unwrap());
                }
                
                WindowEvent::RedrawRequested => {
                    let now = std::time::Instant::now();
                    let dt = (now - last_time).as_secs_f32();
                    last_time = now;

                    game.update(dt.min(0.1));
                    game.render();
                    surface.swap_buffers(&context).unwrap();
                    
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
