//! VoxelForge - 3D Game Constructor
//! 
//! Main entry point for the editor application.

use std::sync::Arc;
use std::time::{Duration, Instant};
use glam::{Vec2, Vec3};
use winit::{
    event::{Event, WindowEvent, MouseButton, ElementState, MouseScrollDelta, KeyEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    keyboard::{Key, NamedKey},
    dpi::PhysicalSize,
};

use adaptive_entity_engine::editor::{
    ui::{EditorState, KeyCode},
    scene::{ObjectType, PrimitiveType, LightType, TeamType},
    tools::ToolType,
    viewport::GizmoType,
    project::ProjectTemplate,
};

use editor_renderer::EditorRenderer;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║           🎮 VoxelForge - 3D Game Constructor 🎮           ║");
    println!("║                     Version 1.0.0                          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Create FPS games visually!                                ║");
    println!("║  • Drag & drop 3D models                                   ║");
    println!("║  • Design maps with built-in tools                         ║");
    println!("║  • Configure gameplay settings                             ║");
    println!("║  • Export to standalone game                               ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Starting VoxelForge...");
    
    // Create event loop
    let event_loop = match EventLoop::new() {
        Ok(el) => el,
        Err(e) => {
            eprintln!("Failed to create event loop: {}", e);
            println!("Press Enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };
    
    // Create window
    println!("Creating window...");
    let window = match WindowBuilder::new()
        .with_title("VoxelForge - 3D Game Constructor")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_min_inner_size(PhysicalSize::new(800, 600))
        .with_visible(true)
        .build(&event_loop) 
    {
        Ok(w) => Arc::new(w),
        Err(e) => {
            eprintln!("Failed to create window: {}", e);
            println!("Press Enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };
    
    println!("Initializing renderer...");
    
    // Create app with error handling
    let mut app = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        App::new(window.clone())
    })) {
        Ok(app) => app,
        Err(_) => {
            eprintln!("Failed to initialize graphics. Make sure you have a compatible GPU.");
            println!("Press Enter to exit...");
            let _ = std::io::stdin().read_line(&mut String::new());
            return;
        }
    };
    
    println!("VoxelForge ready!");
    println!();
    println!("Controls:");
    println!("  Right-click + drag: Orbit camera");
    println!("  Middle-click + drag: Pan camera");
    println!("  Scroll: Zoom");
    println!("  Q/W/E/R/T: Select/Move/Rotate/Scale/Place tools");
    println!("  Ctrl+N: New project");
    println!("  Ctrl+S: Save");
    println!("  Ctrl+P: Play mode");
    println!();
    
    // Run event loop
    let _ = event_loop.run(move |event, target| {
        // Use Wait for less CPU usage, request redraw when needed
        target.set_control_flow(ControlFlow::Wait);
        
        match event {
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("Closing VoxelForge...");
                        target.exit();
                    }
                    WindowEvent::Resized(size) => {
                        if size.width > 0 && size.height > 0 {
                            app.resize(size.width, size.height);
                        }
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        app.update();
                        app.render();
                        // Request next frame
                        window.request_redraw();
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        app.handle_mouse_button(button, state == ElementState::Pressed);
                        window.request_redraw();
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        app.handle_mouse_move(position.x as f32, position.y as f32);
                        window.request_redraw();
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        let scroll = match delta {
                            MouseScrollDelta::LineDelta(_, y) => y,
                            MouseScrollDelta::PixelDelta(pos) => pos.y as f32 * 0.1,
                        };
                        app.handle_scroll(scroll);
                        window.request_redraw();
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        app.handle_keyboard(event);
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::Resumed => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

/// Main application
struct App {
    window: Arc<Window>,
    renderer: Option<EditorRenderer>,
    state: EditorState,
    
    // Input state
    mouse_pos: Vec2,
    last_mouse_pos: Vec2,
    mouse_buttons: [bool; 3],
    keys_pressed: std::collections::HashSet<KeyCode>,
    ctrl_pressed: bool,
    shift_pressed: bool,
    alt_pressed: bool,
    
    // Timing
    last_frame: Instant,
    frame_count: u32,
    fps_timer: Instant,
}

impl App {
    fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();
        
        // Try to create renderer, may fail on systems without GPU
        let renderer = match pollster::block_on(EditorRenderer::try_new(window.clone(), size.width, size.height)) {
            Ok(r) => {
                println!("Renderer initialized successfully!");
                Some(r)
            }
            Err(e) => {
                eprintln!("Warning: Could not initialize renderer: {}", e);
                eprintln!("Running in fallback mode without 3D rendering.");
                None
            }
        };
        
        let mut state = EditorState::new();
        state.viewport.resize(size.width as f32, size.height as f32);
        // Create a default project automatically
        state.new_project("MyGame", ProjectTemplate::Deathmatch);
        
        Self {
            window,
            renderer,
            state,
            mouse_pos: Vec2::ZERO,
            last_mouse_pos: Vec2::ZERO,
            mouse_buttons: [false; 3],
            keys_pressed: std::collections::HashSet::new(),
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            last_frame: Instant::now(),
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Some(renderer) = &mut self.renderer {
                renderer.resize(width, height);
            }
            self.state.viewport.resize(width as f32, height as f32);
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        
        // FPS counter
        self.frame_count += 1;
        if self.fps_timer.elapsed() >= Duration::from_secs(1) {
            self.state.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.fps_timer = Instant::now();
        }

        // Camera fly mode with WASD
        if self.state.viewport.camera.is_flying {
            let forward = if self.keys_pressed.contains(&KeyCode::W) { 1.0 } else { 0.0 }
                - if self.keys_pressed.contains(&KeyCode::S) { 1.0 } else { 0.0 };
            let right = if self.keys_pressed.contains(&KeyCode::D) { 1.0 } else { 0.0 }
                - if self.keys_pressed.contains(&KeyCode::A) { 1.0 } else { 0.0 };
            let up = if self.keys_pressed.contains(&KeyCode::E) { 1.0 } else { 0.0 }
                - if self.keys_pressed.contains(&KeyCode::Q) { 1.0 } else { 0.0 };
            
            self.state.viewport.camera.fly_move(forward, right, up, delta_time);
        }

        // Update window title
        let title = if let Some(project) = &self.state.project {
            let modified = if project.is_modified { "*" } else { "" };
            format!("VoxelForge - {}{} | FPS: {:.0}", project.settings.name, modified, self.state.fps)
        } else {
            format!("VoxelForge | FPS: {:.0}", self.state.fps)
        };
        self.window.set_title(&title);
    }

    fn render(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.state);
        }
    }

    fn handle_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        let index = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return,
        };
        self.mouse_buttons[index] = pressed;

        if pressed {
            // Check if over viewport (not UI panels)
            let in_viewport = self.is_in_viewport(self.mouse_pos);
            
            if in_viewport {
                match button {
                    MouseButton::Left => {
                        // Left click - tool action or selection
                        let world_pos = self.screen_to_world(self.mouse_pos);
                        let camera_pos = self.state.viewport.camera.position;
                        let mouse_pos = self.mouse_pos;
                        let active = self.state.active_scene;
                        if active < self.state.scenes.len() {
                            let scene = &mut self.state.scenes[active];
                            self.state.tools.on_mouse_down(
                                scene,
                                world_pos,
                                mouse_pos,
                                camera_pos,
                            );
                        }
                    }
                    MouseButton::Right => {
                        // Right click - camera orbit
                        self.state.viewport.camera.is_orbiting = true;
                    }
                    MouseButton::Middle => {
                        // Middle click - camera pan
                        self.state.viewport.camera.is_panning = true;
                    }
                    _ => {}
                }
            }
        } else {
            match button {
                MouseButton::Left => {
                    self.state.tools.on_mouse_up();
                }
                MouseButton::Right => {
                    self.state.viewport.camera.is_orbiting = false;
                }
                MouseButton::Middle => {
                    self.state.viewport.camera.is_panning = false;
                }
                _ => {}
            }
        }
    }

    fn handle_mouse_move(&mut self, x: f32, y: f32) {
        self.last_mouse_pos = self.mouse_pos;
        self.mouse_pos = Vec2::new(x, y);
        let delta = self.mouse_pos - self.last_mouse_pos;

        // Camera orbit
        if self.state.viewport.camera.is_orbiting {
            self.state.viewport.camera.orbit(delta);
        }

        // Camera pan
        if self.state.viewport.camera.is_panning {
            self.state.viewport.camera.pan(delta);
        }

        // Tool drag
        if self.state.tools.is_dragging {
            let world_pos = self.screen_to_world(self.mouse_pos);
            let active = self.state.active_scene;
            if active < self.state.scenes.len() {
                let scene = &mut self.state.scenes[active];
                self.state.tools.on_mouse_move(scene, world_pos, delta);
            }
        }

        // Gizmo hover detection
        if !self.state.tools.is_dragging {
            let selected_pos = if let Some(scene) = self.state.current_scene() {
                if let Some(&selected_id) = scene.selected.first() {
                    scene.get_object(selected_id).map(|obj| obj.position)
                } else {
                    None
                }
            } else {
                None
            };
            
            if let Some(pos) = selected_pos {
                let gizmo_type = match self.state.tools.current_tool {
                    ToolType::Move => Some(GizmoType::Translate),
                    ToolType::Rotate => Some(GizmoType::Rotate),
                    ToolType::Scale => Some(GizmoType::Scale),
                    _ => None,
                };
                
                if let Some(gtype) = gizmo_type {
                    self.state.tools.hovered_axis = 
                        self.state.viewport.hit_test_gizmo(self.mouse_pos, pos, gtype);
                }
            }
        }
    }

    fn handle_scroll(&mut self, delta: f32) {
        if self.is_in_viewport(self.mouse_pos) {
            self.state.viewport.camera.zoom(delta);
        }
    }

    fn handle_keyboard(&mut self, event: KeyEvent) {
        let pressed = event.state == ElementState::Pressed;
        
        // Modifier keys
        match &event.logical_key {
            Key::Named(NamedKey::Control) => self.ctrl_pressed = pressed,
            Key::Named(NamedKey::Shift) => self.shift_pressed = pressed,
            Key::Named(NamedKey::Alt) => self.alt_pressed = pressed,
            _ => {}
        }

        if pressed {
            // Convert to KeyCode
            if let Some(key_code) = self.logical_key_to_keycode(&event.logical_key) {
                self.keys_pressed.insert(key_code);
                
                // Handle shortcuts
                self.state.handle_shortcut(key_code, self.ctrl_pressed, self.shift_pressed);
                
                // Quick add objects with number keys (when in place mode)
                if self.state.tools.current_tool == ToolType::Place {
                    match key_code {
                        KeyCode::Key1 => { self.state.add_object(ObjectType::Primitive(PrimitiveType::Box)); }
                        KeyCode::Key2 => { self.state.add_object(ObjectType::Primitive(PrimitiveType::Sphere)); }
                        KeyCode::Key3 => { self.state.add_object(ObjectType::Primitive(PrimitiveType::Cylinder)); }
                        KeyCode::Key4 => { self.state.add_object(ObjectType::Primitive(PrimitiveType::Stairs)); }
                        KeyCode::Key5 => { self.state.add_object(ObjectType::Light(LightType::Point)); }
                        _ => {}
                    }
                }
                
                // New project shortcut
                if key_code == KeyCode::N && self.ctrl_pressed && !self.shift_pressed {
                    self.state.new_project("NewGame", ProjectTemplate::Deathmatch);
                }
            }
        } else {
            if let Some(key_code) = self.logical_key_to_keycode(&event.logical_key) {
                self.keys_pressed.remove(&key_code);
            }
        }
    }

    fn logical_key_to_keycode(&self, key: &Key) -> Option<KeyCode> {
        match key {
            Key::Character(c) => {
                match c.to_uppercase().as_str() {
                    "A" => Some(KeyCode::A),
                    "B" => Some(KeyCode::B),
                    "C" => Some(KeyCode::C),
                    "D" => Some(KeyCode::D),
                    "E" => Some(KeyCode::E),
                    "F" => Some(KeyCode::F),
                    "G" => Some(KeyCode::G),
                    "H" => Some(KeyCode::H),
                    "I" => Some(KeyCode::I),
                    "J" => Some(KeyCode::J),
                    "K" => Some(KeyCode::K),
                    "L" => Some(KeyCode::L),
                    "M" => Some(KeyCode::M),
                    "N" => Some(KeyCode::N),
                    "O" => Some(KeyCode::O),
                    "P" => Some(KeyCode::P),
                    "Q" => Some(KeyCode::Q),
                    "R" => Some(KeyCode::R),
                    "S" => Some(KeyCode::S),
                    "T" => Some(KeyCode::T),
                    "U" => Some(KeyCode::U),
                    "V" => Some(KeyCode::V),
                    "W" => Some(KeyCode::W),
                    "X" => Some(KeyCode::X),
                    "Y" => Some(KeyCode::Y),
                    "Z" => Some(KeyCode::Z),
                    "1" => Some(KeyCode::Key1),
                    "2" => Some(KeyCode::Key2),
                    "3" => Some(KeyCode::Key3),
                    "4" => Some(KeyCode::Key4),
                    "5" => Some(KeyCode::Key5),
                    "6" => Some(KeyCode::Key6),
                    "7" => Some(KeyCode::Key7),
                    "8" => Some(KeyCode::Key8),
                    "9" => Some(KeyCode::Key9),
                    "0" => Some(KeyCode::Key0),
                    _ => None,
                }
            }
            Key::Named(named) => {
                match named {
                    NamedKey::Escape => Some(KeyCode::Escape),
                    NamedKey::Tab => Some(KeyCode::Tab),
                    NamedKey::Space => Some(KeyCode::Space),
                    NamedKey::Enter => Some(KeyCode::Enter),
                    NamedKey::Backspace => Some(KeyCode::Backspace),
                    NamedKey::Delete => Some(KeyCode::Delete),
                    NamedKey::ArrowLeft => Some(KeyCode::Left),
                    NamedKey::ArrowRight => Some(KeyCode::Right),
                    NamedKey::ArrowUp => Some(KeyCode::Up),
                    NamedKey::ArrowDown => Some(KeyCode::Down),
                    NamedKey::Home => Some(KeyCode::Home),
                    NamedKey::End => Some(KeyCode::End),
                    NamedKey::PageUp => Some(KeyCode::PageUp),
                    NamedKey::PageDown => Some(KeyCode::PageDown),
                    NamedKey::Insert => Some(KeyCode::Insert),
                    NamedKey::F1 => Some(KeyCode::F1),
                    NamedKey::F2 => Some(KeyCode::F2),
                    NamedKey::F3 => Some(KeyCode::F3),
                    NamedKey::F4 => Some(KeyCode::F4),
                    NamedKey::F5 => Some(KeyCode::F5),
                    NamedKey::F6 => Some(KeyCode::F6),
                    NamedKey::F7 => Some(KeyCode::F7),
                    NamedKey::F8 => Some(KeyCode::F8),
                    NamedKey::F9 => Some(KeyCode::F9),
                    NamedKey::F10 => Some(KeyCode::F10),
                    NamedKey::F11 => Some(KeyCode::F11),
                    NamedKey::F12 => Some(KeyCode::F12),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn is_in_viewport(&self, pos: Vec2) -> bool {
        // Check if mouse is not over UI panels
        let left_panel = self.state.hierarchy_width;
        let right_panel = self.state.inspector_width;
        let bottom_panel = self.state.asset_browser_height;
        let top_bar = 60.0; // Toolbar + menu height
        
        let size = self.state.viewport.size;
        
        pos.x > left_panel && 
        pos.x < size.x - right_panel &&
        pos.y > top_bar &&
        pos.y < size.y - bottom_panel
    }

    fn screen_to_world(&self, screen_pos: Vec2) -> Vec3 {
        let (origin, direction) = self.state.viewport.camera.screen_to_ray(
            screen_pos,
            self.state.viewport.size,
        );
        
        // Intersect with ground plane (y = 0)
        if direction.y.abs() > 0.001 {
            let t = -origin.y / direction.y;
            if t > 0.0 {
                return origin + direction * t;
            }
        }
        
        // Fallback: project to distance
        origin + direction * 10.0
    }
}

/// Editor renderer module
mod editor_renderer {
    use std::sync::Arc;
    use wgpu::*;
    use wgpu::util::DeviceExt;
    use winit::window::Window;
    use glam::{Mat4, Vec3};
    use crate::EditorState;

    pub struct EditorRenderer {
        device: Device,
        queue: Queue,
        surface: Surface<'static>,
        config: SurfaceConfiguration,
        depth_texture: Texture,
        depth_view: TextureView,
        
        // Pipelines
        grid_pipeline: RenderPipeline,
        mesh_pipeline: RenderPipeline,
        ui_pipeline: RenderPipeline,
        
        // Buffers
        camera_uniform: Buffer,
        camera_bind_group: BindGroup,
        
        grid_vertex_buffer: Buffer,
        grid_vertex_count: u32,
    }

    impl EditorRenderer {
        pub async fn try_new(window: Arc<Window>, width: u32, height: u32) -> Result<Self, String> {
            let instance = Instance::new(InstanceDescriptor {
                backends: Backends::all(),
                ..Default::default()
            });

            let surface = instance.create_surface(window)
                .map_err(|e| format!("Failed to create surface: {}", e))?;
            
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or_else(|| "Failed to find GPU adapter".to_string())?;

            let (device, queue) = adapter
                .request_device(&DeviceDescriptor {
                    label: Some("Editor Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                }, None)
                .await
                .map_err(|e| format!("Failed to create device: {}", e))?;

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps.formats.iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: PresentMode::AutoVsync,
                alpha_mode: surface_caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);

            // Create depth texture
            let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

            // Create shader modules
            let grid_shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Grid Shader"),
                source: ShaderSource::Wgsl(include_str!("shaders/grid.wgsl").into()),
            });

            let mesh_shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("Mesh Shader"),
                source: ShaderSource::Wgsl(include_str!("shaders/mesh.wgsl").into()),
            });

            let ui_shader = device.create_shader_module(ShaderModuleDescriptor {
                label: Some("UI Shader"),
                source: ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
            });

            // Camera uniform buffer
            let camera_uniform = device.create_buffer(&BufferDescriptor {
                label: Some("Camera Uniform"),
                size: 128,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

            let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: camera_uniform.as_entire_binding(),
                }],
            });

            // Grid pipeline
            let grid_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Grid Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

            let grid_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Grid Pipeline"),
                layout: Some(&grid_pipeline_layout),
                vertex: VertexState {
                    module: &grid_shader,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 28,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute { offset: 0, shader_location: 0, format: VertexFormat::Float32x3 },
                            VertexAttribute { offset: 12, shader_location: 1, format: VertexFormat::Float32x4 },
                        ],
                    }],
                },
                fragment: Some(FragmentState {
                    module: &grid_shader,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: surface_format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::LineList,
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                multiview: None,
            });

            // Mesh pipeline
            let mesh_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Mesh Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

            let mesh_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Mesh Pipeline"),
                layout: Some(&mesh_pipeline_layout),
                vertex: VertexState {
                    module: &mesh_shader,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 48,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute { offset: 0, shader_location: 0, format: VertexFormat::Float32x3 },
                            VertexAttribute { offset: 12, shader_location: 1, format: VertexFormat::Float32x3 },
                            VertexAttribute { offset: 24, shader_location: 2, format: VertexFormat::Float32x2 },
                            VertexAttribute { offset: 32, shader_location: 3, format: VertexFormat::Float32x4 },
                        ],
                    }],
                },
                fragment: Some(FragmentState {
                    module: &mesh_shader,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: surface_format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                multiview: None,
            });

            // UI pipeline
            let ui_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("UI Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let ui_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("UI Pipeline"),
                layout: Some(&ui_pipeline_layout),
                vertex: VertexState {
                    module: &ui_shader,
                    entry_point: "vs_main",
                    buffers: &[VertexBufferLayout {
                        array_stride: 24,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute { offset: 0, shader_location: 0, format: VertexFormat::Float32x2 },
                            VertexAttribute { offset: 8, shader_location: 1, format: VertexFormat::Float32x4 },
                        ],
                    }],
                },
                fragment: Some(FragmentState {
                    module: &ui_shader,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: surface_format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            });

            // Create grid vertex buffer
            let grid_vertices = Self::generate_grid(50.0, 50);
            let grid_vertex_count = grid_vertices.len() as u32;
            let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Vertex Buffer"),
                contents: bytemuck::cast_slice(&grid_vertices),
                usage: BufferUsages::VERTEX,
            });

            Ok(Self {
                device,
                queue,
                surface,
                config,
                depth_texture,
                depth_view,
                grid_pipeline,
                mesh_pipeline,
                ui_pipeline,
                camera_uniform,
                camera_bind_group,
                grid_vertex_buffer,
                grid_vertex_count,
            })
        }

        pub fn resize(&mut self, width: u32, height: u32) {
            if width > 0 && height > 0 {
                self.config.width = width;
                self.config.height = height;
                self.surface.configure(&self.device, &self.config);
                
                let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
                self.depth_texture = depth_texture;
                self.depth_view = depth_view;
            }
        }

        pub fn render(&mut self, state: &EditorState) {
            let output = match self.surface.get_current_texture() {
                Ok(output) => output,
                Err(_) => {
                    self.surface.configure(&self.device, &self.config);
                    return;
                }
            };

            let view = output.texture.create_view(&TextureViewDescriptor::default());

            // Update camera uniform
            let view_proj = state.viewport.camera.view_projection();
            let camera_data: [[f32; 4]; 8] = [
                view_proj.col(0).into(),
                view_proj.col(1).into(),
                view_proj.col(2).into(),
                view_proj.col(3).into(),
                state.viewport.camera.position.extend(1.0).into(),
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ];
            self.queue.write_buffer(&self.camera_uniform, 0, bytemuck::cast_slice(&camera_data));

            let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Editor Encoder"),
            });

            // Main render pass
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Editor Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.15,
                                g: 0.15,
                                b: 0.18,
                                a: 1.0,
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                        view: &self.depth_view,
                        depth_ops: Some(Operations {
                            load: LoadOp::Clear(1.0),
                            store: StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });

                // Draw grid
                if state.viewport.grid_visible {
                    render_pass.set_pipeline(&self.grid_pipeline);
                    render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
                    render_pass.draw(0..self.grid_vertex_count, 0..1);
                }

                // Draw scene objects
                if let Some(scene) = state.current_scene() {
                    // Generate mesh for each object and render
                    // For now, we just show the grid and basic structure
                }
            }

            // UI render pass (no depth test)
            {
                let ui_vertices = self.generate_ui_vertices(state);
                if !ui_vertices.is_empty() {
                    let ui_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("UI Buffer"),
                        contents: bytemuck::cast_slice(&ui_vertices),
                        usage: BufferUsages::VERTEX,
                    });

                    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("UI Pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Load,
                                store: StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        ..Default::default()
                    });

                    render_pass.set_pipeline(&self.ui_pipeline);
                    render_pass.set_vertex_buffer(0, ui_buffer.slice(..));
                    render_pass.draw(0..ui_vertices.len() as u32, 0..1);
                }
            }

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
            let texture = device.create_texture(&TextureDescriptor {
                label: Some("Depth Texture"),
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&TextureViewDescriptor::default());
            (texture, view)
        }

        fn generate_grid(size: f32, divisions: u32) -> Vec<[f32; 7]> {
            let mut vertices = Vec::new();
            let step = size * 2.0 / divisions as f32;
            
            let main_color = [0.35, 0.35, 0.35, 1.0];
            let sub_color = [0.25, 0.25, 0.25, 0.7];
            
            for i in 0..=divisions {
                let pos = -size + i as f32 * step;
                let color = if i % 10 == divisions / 2 { main_color } else { sub_color };
                
                // X lines
                vertices.push([pos, 0.0, -size, color[0], color[1], color[2], color[3]]);
                vertices.push([pos, 0.0, size, color[0], color[1], color[2], color[3]]);
                
                // Z lines
                vertices.push([-size, 0.0, pos, color[0], color[1], color[2], color[3]]);
                vertices.push([size, 0.0, pos, color[0], color[1], color[2], color[3]]);
            }
            
            // Axis lines
            // X axis (red)
            vertices.push([0.0, 0.01, 0.0, 0.9, 0.2, 0.2, 1.0]);
            vertices.push([size, 0.01, 0.0, 0.9, 0.2, 0.2, 1.0]);
            
            // Z axis (blue)  
            vertices.push([0.0, 0.01, 0.0, 0.2, 0.2, 0.9, 1.0]);
            vertices.push([0.0, 0.01, size, 0.2, 0.2, 0.9, 1.0]);
            
            vertices
        }

        fn generate_ui_vertices(&self, state: &EditorState) -> Vec<[f32; 6]> {
            let mut vertices = Vec::new();
            let w = self.config.width as f32;
            let h = self.config.height as f32;
            
            // Convert screen coords to NDC
            let to_ndc = |x: f32, y: f32| -> [f32; 2] {
                [(x / w) * 2.0 - 1.0, 1.0 - (y / h) * 2.0]
            };
            
            // Left panel (hierarchy)
            let panel_color = [0.12, 0.12, 0.14, 0.95];
            let lp = state.hierarchy_width;
            self.add_rect(&mut vertices, to_ndc, 0.0, 0.0, lp, h, panel_color);
            
            // Right panel (inspector)
            let rp = state.inspector_width;
            self.add_rect(&mut vertices, to_ndc, w - rp, 0.0, rp, h, panel_color);
            
            // Top bar
            let top_bar = 50.0;
            let bar_color = [0.1, 0.1, 0.12, 0.98];
            self.add_rect(&mut vertices, to_ndc, 0.0, 0.0, w, top_bar, bar_color);
            
            // Bottom panel (asset browser)
            let bp = state.asset_browser_height;
            self.add_rect(&mut vertices, to_ndc, lp, h - bp, w - lp - rp, bp, panel_color);
            
            // Status bar
            let status_height = 24.0;
            let status_color = [0.08, 0.08, 0.1, 0.98];
            self.add_rect(&mut vertices, to_ndc, 0.0, h - status_height, w, status_height, status_color);
            
            // Tool buttons in toolbar
            let tool_size = 32.0;
            let tool_margin = 8.0;
            let tools_start = lp + 20.0;
            let tools_y = 9.0;
            
            let tool_colors = [
                if state.tools.current_tool == crate::ToolType::Select { [0.3, 0.5, 0.8, 1.0] } else { [0.2, 0.2, 0.22, 1.0] },
                if state.tools.current_tool == crate::ToolType::Move { [0.3, 0.5, 0.8, 1.0] } else { [0.2, 0.2, 0.22, 1.0] },
                if state.tools.current_tool == crate::ToolType::Rotate { [0.3, 0.5, 0.8, 1.0] } else { [0.2, 0.2, 0.22, 1.0] },
                if state.tools.current_tool == crate::ToolType::Scale { [0.3, 0.5, 0.8, 1.0] } else { [0.2, 0.2, 0.22, 1.0] },
            ];
            
            for (i, color) in tool_colors.iter().enumerate() {
                let x = tools_start + i as f32 * (tool_size + tool_margin);
                self.add_rect(&mut vertices, to_ndc, x, tools_y, tool_size, tool_size, *color);
            }
            
            // Play button
            let play_x = w / 2.0 - 50.0;
            let play_color = if state.is_playing { [0.2, 0.6, 0.3, 1.0] } else { [0.2, 0.2, 0.22, 1.0] };
            self.add_rect(&mut vertices, to_ndc, play_x, tools_y, tool_size, tool_size, play_color);
            
            // Pause button
            let pause_color = if state.is_paused { [0.6, 0.5, 0.2, 1.0] } else { [0.2, 0.2, 0.22, 1.0] };
            self.add_rect(&mut vertices, to_ndc, play_x + tool_size + tool_margin, tools_y, tool_size, tool_size, pause_color);
            
            // Stop button
            self.add_rect(&mut vertices, to_ndc, play_x + (tool_size + tool_margin) * 2.0, tools_y, tool_size, tool_size, [0.2, 0.2, 0.22, 1.0]);
            
            vertices
        }

        fn add_rect<F>(&self, vertices: &mut Vec<[f32; 6]>, to_ndc: F, x: f32, y: f32, w: f32, h: f32, color: [f32; 4])
        where
            F: Fn(f32, f32) -> [f32; 2],
        {
            let tl = to_ndc(x, y);
            let tr = to_ndc(x + w, y);
            let bl = to_ndc(x, y + h);
            let br = to_ndc(x + w, y + h);
            
            // Two triangles
            vertices.push([tl[0], tl[1], color[0], color[1], color[2], color[3]]);
            vertices.push([bl[0], bl[1], color[0], color[1], color[2], color[3]]);
            vertices.push([tr[0], tr[1], color[0], color[1], color[2], color[3]]);
            
            vertices.push([tr[0], tr[1], color[0], color[1], color[2], color[3]]);
            vertices.push([bl[0], bl[1], color[0], color[1], color[2], color[3]]);
            vertices.push([br[0], br[1], color[0], color[1], color[2], color[3]]);
        }
    }
}
