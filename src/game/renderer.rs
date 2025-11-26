//! Game renderer using wgpu

use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;
use glam::{Mat4, Vec3};
use std::sync::Arc;

use super::map::{GameMap, Vertex};
use super::enemies::Enemy;
use super::hud::HudRenderData;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    view_position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    direction: [f32; 4],
    color: [f32; 4],
    ambient: [f32; 4],
}

pub struct GameRenderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    size: (u32, u32),
    
    // 3D rendering
    render_pipeline: RenderPipeline,
    depth_texture: Texture,
    depth_view: TextureView,
    
    // Map rendering
    map_vertex_buffer: Buffer,
    map_index_buffer: Buffer,
    map_index_count: u32,
    
    // Enemy rendering
    enemy_vertex_buffer: Option<Buffer>,
    enemy_index_buffer: Option<Buffer>,
    enemy_index_count: u32,
    
    // Uniforms
    camera_buffer: Buffer,
    light_buffer: Buffer,
    uniform_bind_group: BindGroup,
    
    // HUD rendering
    hud_pipeline: RenderPipeline,
    hud_vertex_buffer: Buffer,
}

impl GameRenderer {
    pub fn new(window: Arc<Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        
        // Create instance
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        
        let surface = instance.create_surface(window)?;
        
        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or("Failed to find adapter")?;
        
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: Some("Game Device"),
                required_features: Features::empty(),
                required_limits: Limits::default(),
            },
            None,
        ))?;
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        // Create depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, size.width, size.height);
        
        // Create shaders
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Game Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/game.wgsl").into()),
        });
        
        let hud_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("HUD Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/hud.wgsl").into()),
        });
        
        // Create uniform buffers
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform {
                view_proj: Mat4::IDENTITY.to_cols_array_2d(),
                view_position: [0.0, 0.0, 0.0, 1.0],
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[LightUniform {
                direction: [-0.3, -1.0, -0.5, 0.0],
                color: [1.0, 0.95, 0.8, 1.0],
                ambient: [0.15, 0.15, 0.2, 1.0],
            }]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        
        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: light_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Game Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: 12,
                            shader_location: 1,
                            format: VertexFormat::Float32x3,
                        },
                        VertexAttribute {
                            offset: 24,
                            shader_location: 2,
                            format: VertexFormat::Float32x3,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        // HUD pipeline (no depth, alpha blending)
        let hud_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("HUD Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let hud_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("HUD Render Pipeline"),
            layout: Some(&hud_pipeline_layout),
            vertex: VertexState {
                module: &hud_shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 24,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x2,
                        },
                        VertexAttribute {
                            offset: 8,
                            shader_location: 1,
                            format: VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &hud_shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
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
        
        // Create initial map mesh
        let map = GameMap::de_dust_simple();
        let (vertices, indices) = map.generate_mesh_data();
        
        let map_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });
        
        let map_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });
        
        // HUD vertex buffer (will be updated each frame)
        let hud_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("HUD Vertex Buffer"),
            size: 65536, // 64KB should be enough
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            size: (size.width, size.height),
            render_pipeline,
            depth_texture,
            depth_view,
            map_vertex_buffer,
            map_index_buffer,
            map_index_count: indices.len() as u32,
            enemy_vertex_buffer: None,
            enemy_index_buffer: None,
            enemy_index_count: 0,
            camera_buffer,
            light_buffer,
            uniform_bind_group,
            hud_pipeline,
            hud_vertex_buffer,
        })
    }
    
    fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size: Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
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
    
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            
            let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
        }
    }
    
    pub fn update_camera(&mut self, view_proj: Mat4, position: Vec3) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
            view_position: [position.x, position.y, position.z, 1.0],
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
    
    pub fn update_enemies(&mut self, enemies: &[&Enemy]) {
        if enemies.is_empty() {
            self.enemy_index_count = 0;
            return;
        }
        
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        
        for enemy in enemies {
            let base_index = vertices.len() as u32;
            let (v, i) = generate_enemy_mesh(enemy);
            vertices.extend(v);
            for idx in i {
                indices.push(base_index + idx);
            }
        }
        
        self.enemy_vertex_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Enemy Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        }));
        
        self.enemy_index_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Enemy Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        }));
        
        self.enemy_index_count = indices.len() as u32;
    }
    
    pub fn render(&mut self, hud_data: &HudRenderData) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // 3D rendering pass
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("3D Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.4,
                            g: 0.6,
                            b: 0.9,
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
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            
            // Draw map
            render_pass.set_vertex_buffer(0, self.map_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.map_index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.map_index_count, 0, 0..1);
            
            // Draw enemies
            if self.enemy_index_count > 0 {
                if let (Some(vb), Some(ib)) = (&self.enemy_vertex_buffer, &self.enemy_index_buffer) {
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.set_index_buffer(ib.slice(..), IndexFormat::Uint32);
                    render_pass.draw_indexed(0..self.enemy_index_count, 0, 0..1);
                }
            }
        }
        
        // HUD rendering pass
        {
            // Generate HUD vertices
            let hud_vertices = self.generate_hud_vertices(hud_data);
            if !hud_vertices.is_empty() {
                self.queue.write_buffer(&self.hud_vertex_buffer, 0, bytemuck::cast_slice(&hud_vertices));
                
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("HUD Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Load,
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
                
                render_pass.set_pipeline(&self.hud_pipeline);
                render_pass.set_vertex_buffer(0, self.hud_vertex_buffer.slice(..));
                render_pass.draw(0..hud_vertices.len() as u32, 0..1);
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    fn generate_hud_vertices(&self, hud_data: &HudRenderData) -> Vec<[f32; 6]> {
        let mut vertices = Vec::new();
        let w = self.size.0 as f32;
        let h = self.size.1 as f32;
        
        // Helper to convert screen coords to NDC
        let to_ndc = |x: f32, y: f32| -> (f32, f32) {
            ((x / w) * 2.0 - 1.0, 1.0 - (y / h) * 2.0)
        };
        
        // Helper to add a quad
        let mut add_quad = |x: f32, y: f32, width: f32, height: f32, color: [f32; 4]| {
            let (x0, y0) = to_ndc(x, y);
            let (x1, y1) = to_ndc(x + width, y + height);
            
            // Triangle 1
            vertices.push([x0, y0, color[0], color[1], color[2], color[3]]);
            vertices.push([x1, y0, color[0], color[1], color[2], color[3]]);
            vertices.push([x0, y1, color[0], color[1], color[2], color[3]]);
            
            // Triangle 2
            vertices.push([x1, y0, color[0], color[1], color[2], color[3]]);
            vertices.push([x1, y1, color[0], color[1], color[2], color[3]]);
            vertices.push([x0, y1, color[0], color[1], color[2], color[3]]);
        };
        
        // Draw crosshair
        for elem in &hud_data.crosshair_elements {
            add_quad(elem.x, elem.y, elem.width, elem.height, elem.color);
        }
        
        // Draw health
        for elem in &hud_data.health_elements {
            add_quad(elem.x, elem.y, elem.width, elem.height, elem.color);
        }
        
        // Draw armor
        for elem in &hud_data.armor_elements {
            add_quad(elem.x, elem.y, elem.width, elem.height, elem.color);
        }
        
        // Draw ammo
        for elem in &hud_data.ammo_elements {
            add_quad(elem.x, elem.y, elem.width, elem.height, elem.color);
        }
        
        vertices
    }
    
    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }
}

fn generate_enemy_mesh(enemy: &Enemy) -> (Vec<Vertex>, Vec<u32>) {
    let pos = enemy.position;
    let color = enemy.get_color();
    let size = 0.6;
    let height = 1.8;
    
    // Simple box for enemy body
    let min = Vec3::new(pos.x - size, pos.y, pos.z - size);
    let max = Vec3::new(pos.x + size, pos.y + height, pos.z + size);
    
    use super::map::AABB;
    let aabb = AABB::new(min, max);
    
    // Reuse the box mesh generation but with enemy position
    let corners = [
        [min.x, min.y, min.z],
        [max.x, min.y, min.z],
        [max.x, max.y, min.z],
        [min.x, max.y, min.z],
        [min.x, min.y, max.z],
        [max.x, min.y, max.z],
        [max.x, max.y, max.z],
        [min.x, max.y, max.z],
    ];
    
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    
    let faces = [
        ([4, 5, 6, 7], [0.0, 0.0, 1.0]),
        ([1, 0, 3, 2], [0.0, 0.0, -1.0]),
        ([3, 7, 6, 2], [0.0, 1.0, 0.0]),
        ([4, 0, 1, 5], [0.0, -1.0, 0.0]),
        ([5, 1, 2, 6], [1.0, 0.0, 0.0]),
        ([0, 4, 7, 3], [-1.0, 0.0, 0.0]),
    ];
    
    for (corner_indices, normal) in faces {
        let base = vertices.len() as u32;
        
        for &idx in &corner_indices {
            vertices.push(Vertex {
                position: corners[idx],
                normal,
                color,
            });
        }
        
        indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);
    }
    
    (vertices, indices)
}
