// VoxelCraft - Rendering System with Pattern Lighting

mod loading;

use crate::{GameState, GameUI};
use crate::world::{ChunkMesh, ChunkMesher, CHUNK_SIZE};
use std::sync::Arc;
use std::collections::HashMap;
use wgpu::util::DeviceExt;
use glam::{Vec3, Mat4};
use winit::window::Window;

use loading::LoadingScreen;

/// Main renderer
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    
    // Pipelines
    terrain_pipeline: wgpu::RenderPipeline,
    water_pipeline: wgpu::RenderPipeline,
    
    // Camera
    pub camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    
    // Lighting
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    
    // Chunk meshes
    chunk_meshes: HashMap<(i32, i32), ChunkRenderData>,
    
    // Time for animations
    time: f32,
    
    // Loading screen
    loading_screen: LoadingScreen,
    show_loading: bool,
    loading_time: f32,
}

struct ChunkRenderData {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
    water_vertex_buffer: Option<wgpu::Buffer>,
    water_index_buffer: Option<wgpu::Buffer>,
    water_index_count: u32,
}

impl Renderer {
    pub async fn try_new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let width = size.width.max(100);
        let height = size.height.max(100);

        log::info!("Renderer: window size {}x{}", width, height);

        // Create instance - try Vulkan first, then GL
        log::info!("Creating wgpu instance...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });

        log::info!("Creating surface...");
        let surface = instance.create_surface(window.clone())
            .map_err(|e| format!("Failed to create surface: {}", e))?;

        log::info!("Requesting adapter...");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("No suitable GPU adapter found")?;

        let info = adapter.get_info();
        log::info!("Adapter: {} ({:?})", info.name, info.backend);

        log::info!("Requesting device...");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
                    label: Some("VoxelCraft Device"),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create device: {}", e))?;

        log::info!("Configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        log::info!("Available formats: {:?}", surface_caps.formats);
        
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        
        log::info!("Using format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Shaders
        log::info!("Creating shaders...");
        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Terrain Shader"),
            source: wgpu::ShaderSource::Wgsl(TERRAIN_SHADER.into()),
        });

        let water_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Water Shader"),
            source: wgpu::ShaderSource::Wgsl(WATER_SHADER.into()),
        });

        // Camera uniform
        let camera = Camera::new(width as f32, height as f32);
        let camera_uniform = CameraUniform::from_camera(&camera);
        
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        // Light uniform
        let light_uniform = LightUniform::default();
        
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[light_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Bind Group"),
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
        });

        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
            push_constant_ranges: &[],
        });

        log::info!("Creating terrain pipeline...");
        let terrain_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: "vs_main",
                buffers: &[TerrainVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        log::info!("Creating water pipeline...");
        let water_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Water Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &water_shader,
                entry_point: "vs_main",
                buffers: &[TerrainVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &water_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        log::info!("Creating loading screen...");
        let loading_screen = LoadingScreen::new(&device, surface_format);

        log::info!("Renderer created successfully!");
        
        Ok(Self {
            device,
            queue,
            surface,
            config,
            depth_texture,
            depth_view,
            terrain_pipeline,
            water_pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            light_bind_group,
            chunk_meshes: HashMap::new(),
            time: 0.0,
            loading_screen,
            show_loading: true,
            loading_time: 0.0,
        })
    }

    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        log::info!("Resizing to {}x{}", width, height);

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        self.camera.resize(width as f32, height as f32);
    }

    pub fn render(&mut self, state: &GameState, _ui: &GameUI) {
        self.time += 0.016;
        self.loading_time += 0.016;

        // Get surface texture
        let output = match self.surface.get_current_texture() {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed to get surface texture: {:?}", e);
                self.surface.configure(&self.device, &self.config);
                return;
            }
        };
        
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Show loading screen for first 4 seconds
        if self.show_loading {
            self.loading_screen.render(&mut encoder, &view, &self.queue, self.loading_time);
            
            if self.loading_time > 4.0 {
                self.show_loading = false;
            }
            
            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
            return;
        }

        // Update camera
        self.camera.position = state.player.get_eye_position();
        self.camera.yaw = state.player.rotation.0;
        self.camera.pitch = state.player.rotation.1;

        let camera_uniform = CameraUniform::from_camera(&self.camera);
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));

        // Update lighting
        let light_uniform = LightUniform {
            sun_direction: state.get_sun_direction().to_array(),
            _padding1: 0.0,
            ambient: state.get_ambient_light(),
            time: self.time,
            _padding2: [0.0; 2],
        };
        self.queue.write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&[light_uniform]));

        // Update chunk meshes
        self.update_chunk_meshes(state);

        // Render pass
        {
            // Sky color based on time of day
            let sky_color = if state.is_night() {
                wgpu::Color { r: 0.02, g: 0.02, b: 0.08, a: 1.0 }
            } else {
                wgpu::Color { r: 0.4, g: 0.6, b: 0.9, a: 1.0 }
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(sky_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Render terrain
            render_pass.set_pipeline(&self.terrain_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.light_bind_group, &[]);

            for (_, mesh_data) in &self.chunk_meshes {
                if mesh_data.index_count > 0 {
                    render_pass.set_vertex_buffer(0, mesh_data.vertex_buffer.slice(..));
                    render_pass.set_index_buffer(mesh_data.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..mesh_data.index_count, 0, 0..1);
                }
            }

            // Render water
            render_pass.set_pipeline(&self.water_pipeline);
            
            for (_, mesh_data) in &self.chunk_meshes {
                if mesh_data.water_index_count > 0 {
                    if let (Some(vb), Some(ib)) = (&mesh_data.water_vertex_buffer, &mesh_data.water_index_buffer) {
                        render_pass.set_vertex_buffer(0, vb.slice(..));
                        render_pass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(0..mesh_data.water_index_count, 0, 0..1);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn update_chunk_meshes(&mut self, state: &GameState) {
        // Remove meshes for unloaded chunks
        self.chunk_meshes.retain(|k, _| state.world.chunks.contains_key(k));

        // Update dirty chunks
        for ((cx, cz), chunk) in &state.world.chunks {
            if chunk.dirty || !self.chunk_meshes.contains_key(&(*cx, *cz)) {
                let neighbors = [
                    state.world.chunks.get(&(cx - 1, *cz)),
                    state.world.chunks.get(&(cx + 1, *cz)),
                    state.world.chunks.get(&(*cx, cz - 1)),
                    state.world.chunks.get(&(*cx, cz + 1)),
                ];

                let mesh = ChunkMesher::generate_mesh(chunk, &neighbors);

                if !mesh.is_empty() {
                    let terrain_vertices: Vec<TerrainVertex> = mesh.vertices.iter().map(|v| {
                        TerrainVertex {
                            position: v.position,
                            normal: v.normal,
                            uv: v.uv,
                            ao: v.ao,
                        }
                    }).collect();

                    let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Chunk Vertex Buffer"),
                        contents: bytemuck::cast_slice(&terrain_vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    });

                    let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Chunk Index Buffer"),
                        contents: bytemuck::cast_slice(&mesh.indices),
                        usage: wgpu::BufferUsages::INDEX,
                    });

                    let (water_vertex_buffer, water_index_buffer) = if !mesh.water_vertices.is_empty() {
                        let water_verts: Vec<TerrainVertex> = mesh.water_vertices.iter().map(|v| {
                            TerrainVertex {
                                position: v.position,
                                normal: v.normal,
                                uv: v.uv,
                                ao: v.ao,
                            }
                        }).collect();

                        (
                            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Water Vertex Buffer"),
                                contents: bytemuck::cast_slice(&water_verts),
                                usage: wgpu::BufferUsages::VERTEX,
                            })),
                            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Water Index Buffer"),
                                contents: bytemuck::cast_slice(&mesh.water_indices),
                                usage: wgpu::BufferUsages::INDEX,
                            })),
                        )
                    } else {
                        (None, None)
                    };

                    self.chunk_meshes.insert((*cx, *cz), ChunkRenderData {
                        vertex_buffer,
                        index_buffer,
                        index_count: mesh.indices.len() as u32,
                        water_vertex_buffer,
                        water_index_buffer,
                        water_index_count: mesh.water_indices.len() as u32,
                    });
                }
            }
        }
    }
}

/// Camera
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 64.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0_f32.to_radians(),
            aspect: width / height,
            near: 0.1,
            far: 300.0,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height.max(1.0);
    }

    pub fn view_matrix(&self) -> Mat4 {
        let direction = Vec3::new(
            -self.yaw.sin() * self.pitch.cos(),
            -self.pitch.sin(),
            -self.yaw.cos() * self.pitch.cos(),
        ).normalize();

        Mat4::look_to_rh(self.position, direction, Vec3::Y)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    _padding: f32,
}

impl CameraUniform {
    fn from_camera(camera: &Camera) -> Self {
        let view_proj = camera.projection_matrix() * camera.view_matrix();
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            position: camera.position.to_array(),
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    sun_direction: [f32; 3],
    _padding1: f32,
    ambient: f32,
    time: f32,
    _padding2: [f32; 2],
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            sun_direction: [0.5, 0.8, 0.3],
            _padding1: 0.0,
            ambient: 0.4,
            time: 0.0,
            _padding2: [0.0; 2],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TerrainVertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
    ao: f32,
}

impl TerrainVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

// ============== EMBEDDED SHADERS ==============

const TERRAIN_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct LightUniform {
    sun_direction: vec3<f32>,
    _padding1: f32,
    ambient: f32,
    time: f32,
    _padding2: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) ao: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal = in.normal;
    out.uv = in.uv;
    out.ao = in.ao;
    return out;
}

fn get_block_color(uv: vec2<f32>, world_pos: vec3<f32>) -> vec3<f32> {
    let atlas_pos = floor(uv * 16.0);
    let noise = fract(sin(dot(world_pos.xz, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    
    // Grass top
    if atlas_pos.x == 0.0 && atlas_pos.y == 0.0 {
        return mix(vec3<f32>(0.2, 0.6, 0.15), vec3<f32>(0.3, 0.7, 0.2), noise);
    }
    // Stone
    if atlas_pos.x == 1.0 && atlas_pos.y == 0.0 {
        return mix(vec3<f32>(0.4, 0.4, 0.4), vec3<f32>(0.55, 0.55, 0.55), noise);
    }
    // Dirt
    if atlas_pos.x == 2.0 && atlas_pos.y == 0.0 {
        return mix(vec3<f32>(0.45, 0.3, 0.15), vec3<f32>(0.55, 0.35, 0.2), noise);
    }
    // Grass side  
    if atlas_pos.x == 3.0 && atlas_pos.y == 0.0 {
        let grass_blend = smoothstep(0.3, 0.7, fract(uv.y * 16.0));
        let dirt = mix(vec3<f32>(0.45, 0.3, 0.15), vec3<f32>(0.55, 0.35, 0.2), noise);
        let grass = vec3<f32>(0.25, 0.55, 0.15);
        return mix(dirt, grass, grass_blend);
    }
    // Wood
    if atlas_pos.x == 4.0 && atlas_pos.y == 1.0 {
        let ring = sin(world_pos.y * 8.0) * 0.1;
        return vec3<f32>(0.5 + ring, 0.35 + ring * 0.5, 0.2);
    }
    // Sand
    if atlas_pos.x == 2.0 && atlas_pos.y == 1.0 {
        return mix(vec3<f32>(0.85, 0.8, 0.55), vec3<f32>(0.92, 0.88, 0.65), noise);
    }
    // Leaves
    if atlas_pos.x == 4.0 && atlas_pos.y == 3.0 {
        return mix(vec3<f32>(0.15, 0.45, 0.1), vec3<f32>(0.2, 0.55, 0.15), noise);
    }
    // Cobblestone
    if atlas_pos.x == 0.0 && atlas_pos.y == 1.0 {
        return mix(vec3<f32>(0.35, 0.35, 0.35), vec3<f32>(0.5, 0.5, 0.5), noise);
    }
    // Coal ore
    if atlas_pos.x == 2.0 && atlas_pos.y == 2.0 {
        let ore = step(0.7, noise);
        return mix(vec3<f32>(0.45, 0.45, 0.45), vec3<f32>(0.1, 0.1, 0.1), ore);
    }
    // Iron ore
    if atlas_pos.x == 1.0 && atlas_pos.y == 2.0 {
        let ore = step(0.75, noise);
        return mix(vec3<f32>(0.45, 0.45, 0.45), vec3<f32>(0.7, 0.6, 0.5), ore);
    }
    // Diamond ore
    if atlas_pos.x == 2.0 && atlas_pos.y == 3.0 {
        let sparkle = sin(light.time * 5.0 + world_pos.x * 10.0) * 0.5 + 0.5;
        let ore = step(0.8, noise);
        return mix(vec3<f32>(0.45, 0.45, 0.45), vec3<f32>(0.3, 0.8, 0.9) * (1.0 + sparkle * 0.3), ore);
    }
    // Snow
    if atlas_pos.x == 2.0 && atlas_pos.y == 4.0 {
        return vec3<f32>(0.95, 0.97, 1.0);
    }
    // Planks
    if atlas_pos.x == 4.0 && atlas_pos.y == 0.0 {
        let grain = sin(world_pos.z * 15.0) * 0.05;
        return vec3<f32>(0.65 + grain, 0.45 + grain * 0.5, 0.25);
    }
    
    return vec3<f32>(0.5, 0.5, 0.5);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = get_block_color(in.uv, in.world_pos);
    
    // Lighting
    let ndotl = max(dot(in.normal, light.sun_direction), 0.0);
    let diffuse = ndotl * 0.7;
    
    // Ambient occlusion
    let ao = in.ao * in.ao;
    
    var final_color = base_color * (light.ambient + diffuse) * ao;
    
    // Fog
    let dist = length(camera.position - in.world_pos);
    let fog_factor = 1.0 - exp(-dist * 0.008);
    let fog_color = vec3<f32>(0.5, 0.65, 0.85);
    final_color = mix(final_color, fog_color, clamp(fog_factor, 0.0, 0.7));
    
    return vec4<f32>(final_color, 1.0);
}
"#;

const WATER_SHADER: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

struct LightUniform {
    sun_direction: vec3<f32>,
    _padding1: f32,
    ambient: f32,
    time: f32,
    _padding2: vec2<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> light: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    var pos = in.position;
    
    // Simple wave animation
    let wave = sin(pos.x * 0.5 + light.time * 2.0) * cos(pos.z * 0.5 + light.time * 1.5) * 0.1;
    pos.y += wave;
    
    out.world_pos = pos;
    out.clip_position = camera.view_proj * vec4<f32>(pos, 1.0);
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(camera.position - in.world_pos);
    
    // Water color
    let deep_color = vec3<f32>(0.05, 0.2, 0.35);
    let shallow_color = vec3<f32>(0.1, 0.4, 0.5);
    let water_color = mix(deep_color, shallow_color, 0.5);
    
    // Fresnel reflection
    let fresnel = pow(1.0 - max(dot(view_dir, in.normal), 0.0), 3.0);
    let sky_color = vec3<f32>(0.5, 0.7, 0.95);
    
    // Sun reflection
    let reflect_dir = reflect(-view_dir, in.normal);
    let sun_reflect = pow(max(dot(reflect_dir, light.sun_direction), 0.0), 128.0);
    
    var final_color = mix(water_color, sky_color, fresnel * 0.5);
    final_color += vec3<f32>(1.0, 0.95, 0.8) * sun_reflect;
    
    // Lighting
    final_color *= (light.ambient + 0.3);
    
    return vec4<f32>(final_color, 0.75);
}
"#;
