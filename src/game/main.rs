//! VoxelStrike - Main entry point
//! A Counter-Strike inspired FPS game built on the Adaptive Entity Engine

mod camera;
mod enemies;
mod hud;
mod map;
mod player;
mod renderer;
mod weapons;

use std::sync::Arc;
use std::time::Instant;
use glam::Vec2;
use winit::{
    event::{DeviceEvent, ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowBuilder},
};

use camera::MoveDirection;
use enemies::{EnemyManager, Team};
use hud::GameHUD;
use map::GameMap;
use player::Player;
use renderer::GameRenderer;

struct GameState {
    window: Arc<Window>,
    renderer: GameRenderer,
    player: Player,
    map: GameMap,
    enemies: EnemyManager,
    hud: GameHUD,
    
    // Input state
    keys_pressed: std::collections::HashSet<KeyCode>,
    mouse_captured: bool,
    
    // Timing
    start_time: Instant,
    last_frame: Instant,
    
    // Game state
    score: u32,
    round: u32,
    game_over: bool,
}

impl GameState {
    fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        let renderer = GameRenderer::new(window.clone()).expect("Failed to create renderer");
        
        let map = GameMap::de_dust_simple();
        
        // Spawn player at T spawn
        let spawn_pos = map.spawn_points_t.first().copied().unwrap_or(glam::Vec3::new(-50.0, 1.0, 0.0));
        let mut player = Player::new(spawn_pos);
        player.camera.aspect_ratio = size.width as f32 / size.height as f32;
        
        // Spawn enemies at CT spawn
        let mut enemies = EnemyManager::new();
        enemies.spawn_initial_enemies(&map.spawn_points_ct, Team::CounterTerrorist);
        
        let hud = GameHUD::new(size.width as f32, size.height as f32);
        
        Self {
            window,
            renderer,
            player,
            map,
            enemies,
            hud,
            keys_pressed: std::collections::HashSet::new(),
            mouse_captured: false,
            start_time: Instant::now(),
            last_frame: Instant::now(),
            score: 0,
            round: 1,
            game_over: false,
        }
    }
    
    fn capture_mouse(&mut self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::Confined);
        let _ = self.window.set_cursor_grab(CursorGrabMode::Locked);
        self.window.set_cursor_visible(false);
        self.mouse_captured = true;
    }
    
    fn release_mouse(&mut self) {
        let _ = self.window.set_cursor_grab(CursorGrabMode::None);
        self.window.set_cursor_visible(true);
        self.mouse_captured = false;
    }
    
    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        let current_time = (now - self.start_time).as_secs_f32();
        
        if self.game_over || !self.player.is_alive {
            return;
        }
        
        // Process movement
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            self.player.process_movement(MoveDirection::Forward, delta_time, &self.map);
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            self.player.process_movement(MoveDirection::Backward, delta_time, &self.map);
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            self.player.process_movement(MoveDirection::Left, delta_time, &self.map);
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            self.player.process_movement(MoveDirection::Right, delta_time, &self.map);
        }
        
        // Sprint
        self.player.is_sprinting = self.keys_pressed.contains(&KeyCode::ShiftLeft);
        
        // Crouch
        self.player.is_crouching = self.keys_pressed.contains(&KeyCode::ControlLeft);
        
        // Update player physics
        self.player.update(delta_time, &self.map);
        
        // Update weapon
        self.player.weapons.current_mut().update(current_time);
        
        // Update enemies
        self.enemies.update(self.player.get_position(), current_time, delta_time);
        
        // Check enemy attacks
        for enemy in &mut self.enemies.enemies {
            if enemy.can_attack(current_time) {
                let dist = (self.player.get_position() - enemy.position).length();
                if dist < enemy.attack_range {
                    let damage = enemy.attack(current_time);
                    self.player.take_damage(damage);
                    
                    if !self.player.is_alive {
                        self.hud.add_kill("Bot", "Player", "AK-47", false, current_time);
                    }
                }
            }
        }
        
        // Update HUD
        self.hud.update(current_time);
        
        // Check win condition (all enemies dead)
        if self.enemies.get_alive_enemies().is_empty() {
            self.round += 1;
            self.score += 100;
            
            // Respawn enemies
            self.enemies = EnemyManager::new();
            self.enemies.spawn_initial_enemies(&self.map.spawn_points_ct, Team::CounterTerrorist);
        }
    }
    
    fn shoot(&mut self) {
        let current_time = (Instant::now() - self.start_time).as_secs_f32();
        
        if let Some(shot) = self.player.weapons.current_mut().fire(current_time) {
            let origin = self.player.camera.position;
            let direction = self.player.camera.get_front();
            
            // Check enemy hit
            if let Some((dist, enemy_id)) = self.enemies.raycast(origin, direction, shot.range) {
                let killed = self.enemies.damage_enemy(enemy_id, shot.damage);
                
                if killed {
                    self.player.kills += 1;
                    self.score += 10;
                    self.hud.add_kill(
                        "Player",
                        &format!("Bot #{}", enemy_id),
                        &self.player.weapons.current().name,
                        false,
                        current_time,
                    );
                }
            }
        }
    }
    
    fn render(&mut self) {
        // Update camera
        let view_proj = self.player.camera.view_projection_matrix();
        self.renderer.update_camera(view_proj, self.player.camera.position);
        
        // Update enemies
        let alive_enemies: Vec<_> = self.enemies.get_alive_enemies();
        self.renderer.update_enemies(&alive_enemies);
        
        // Generate HUD data
        let hud_data = self.hud.generate_hud_data(&self.player);
        
        // Render
        if let Err(e) = self.renderer.render(&hud_data) {
            log::error!("Render error: {:?}", e);
        }
    }

    fn handle_key(&mut self, key: KeyCode, pressed: bool) -> bool {
        if pressed {
            self.keys_pressed.insert(key);
            
            match key {
                KeyCode::Escape => {
                    if self.mouse_captured {
                        self.release_mouse();
                    } else {
                        return true; // Exit
                    }
                }
                KeyCode::Space => {
                    self.player.jump();
                }
                KeyCode::KeyR => {
                    let time = (Instant::now() - self.start_time).as_secs_f32();
                    self.player.weapons.current_mut().start_reload(time);
                }
                KeyCode::Digit1 => self.player.weapons.switch(0),
                KeyCode::Digit2 => self.player.weapons.switch(1),
                KeyCode::Digit3 => self.player.weapons.switch(2),
                KeyCode::KeyQ => self.player.weapons.prev(),
                KeyCode::KeyE => self.player.weapons.next(),
                KeyCode::Tab => self.hud.show_scoreboard = true,
                _ => {}
            }
        } else {
            self.keys_pressed.remove(&key);
            
            if key == KeyCode::Tab {
                self.hud.show_scoreboard = false;
            }
        }
        false
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           VoxelStrike - Counter-Strike Style FPS          ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║  Controls:                                                ║");
    println!("║    WASD      - Move                                       ║");
    println!("║    Mouse     - Look around                                ║");
    println!("║    LMB       - Shoot                                      ║");
    println!("║    Space     - Jump                                       ║");
    println!("║    Shift     - Sprint                                     ║");
    println!("║    Ctrl      - Crouch                                     ║");
    println!("║    R         - Reload                                     ║");
    println!("║    1/2/3     - Switch weapons                             ║");
    println!("║    Q/E       - Prev/Next weapon                           ║");
    println!("║    Tab       - Scoreboard                                 ║");
    println!("║    Escape    - Release mouse / Exit                       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Starting game...");
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("VoxelStrike - Counter-Strike Style FPS")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap()
    );
    
    let mut state = GameState::new(window.clone());
    state.capture_mouse();
    
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    
                    WindowEvent::Resized(size) => {
                        state.renderer.resize(size.width, size.height);
                        state.player.camera.aspect_ratio = size.width as f32 / size.height.max(1) as f32;
                        state.hud.resize(size.width as f32, size.height as f32);
                    }
                    
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let PhysicalKey::Code(key) = event.physical_key {
                            let should_exit = state.handle_key(key, event.state == ElementState::Pressed);
                            if should_exit {
                                elwt.exit();
                            }
                        }
                    }
                    
                    WindowEvent::MouseInput { state: button_state, button, .. } => {
                        if !state.mouse_captured {
                            if button_state == ElementState::Pressed {
                                state.capture_mouse();
                            }
                            return;
                        }
                        
                        if button == MouseButton::Left && button_state == ElementState::Pressed {
                            state.shoot();
                        }
                    }
                    
                    WindowEvent::RedrawRequested => {
                        state.update();
                        state.render();
                    }
                    
                    _ => {}
                }
            }
            
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                if state.mouse_captured {
                    state.player.camera.process_mouse(Vec2::new(delta.0 as f32, delta.1 as f32));
                }
            }
            
            Event::AboutToWait => {
                state.window.request_redraw();
            }
            
            _ => {}
        }
    }).unwrap();
}
