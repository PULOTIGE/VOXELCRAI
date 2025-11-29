//! wgpu Render Pipeline для Instanced Voxel Rendering
//! 
//! Миллионы вокселей без лагов через instanced rendering.
//! Каждый воксель — один куб с индивидуальными параметрами.

use crate::voxel::VoxelGpuInstance;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use std::sync::Arc;

/// Вершина куба
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CubeVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl CubeVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CubeVertex>() as wgpu::BufferAddress,
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
            ],
        }
    }
}

/// Uniform буфер для камеры и времени
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub time: f32,
    pub pulse_phase: f32,
    pub mode: f32, // 0 = normal, 1 = ignite, 2 = trauma
    pub _padding: f32,
}

/// Основной рендерер
pub struct VoxelRenderer {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    pub surface_format: wgpu::TextureFormat,
    
    // Пайплайны
    render_pipeline: wgpu::RenderPipeline,
    
    // Буферы
    cube_vertex_buffer: wgpu::Buffer,
    cube_index_buffer: wgpu::Buffer,
    instance_buffer: Option<wgpu::Buffer>,
    uniform_buffer: wgpu::Buffer,
    
    // Bind groups
    uniform_bind_group: wgpu::BindGroup,
    
    // Состояние
    num_indices: u32,
    num_instances: u32,
    
    // Depth buffer
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

impl VoxelRenderer {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        // Создать шейдер
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Voxel Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/voxel_render.wgsl").into()),
        });
        
        // Создать куб
        let (vertices, indices) = Self::create_cube();
        
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let cube_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Uniform буфер
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<GlobalUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // Bind group layout
        let uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
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
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        // Pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Depth texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);
        
        // Render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Voxel Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[CubeVertex::desc(), VoxelGpuInstance::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Self {
            device,
            queue,
            surface_format,
            render_pipeline,
            cube_vertex_buffer,
            cube_index_buffer,
            instance_buffer: None,
            uniform_buffer,
            uniform_bind_group,
            num_indices: indices.len() as u32,
            num_instances: 0,
            depth_texture,
            depth_view,
        }
    }
    
    /// Создать куб (вершины + индексы)
    fn create_cube() -> (Vec<CubeVertex>, Vec<u16>) {
        let vertices = vec![
            // Front face
            CubeVertex { position: [-0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            CubeVertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            CubeVertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            CubeVertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 0.0, 1.0] },
            // Back face
            CubeVertex { position: [-0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            CubeVertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            CubeVertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            CubeVertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, 0.0, -1.0] },
            // Top face
            CubeVertex { position: [-0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0] },
            CubeVertex { position: [-0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0] },
            CubeVertex { position: [ 0.5,  0.5,  0.5], normal: [0.0, 1.0, 0.0] },
            CubeVertex { position: [ 0.5,  0.5, -0.5], normal: [0.0, 1.0, 0.0] },
            // Bottom face
            CubeVertex { position: [-0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
            CubeVertex { position: [ 0.5, -0.5, -0.5], normal: [0.0, -1.0, 0.0] },
            CubeVertex { position: [ 0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0] },
            CubeVertex { position: [-0.5, -0.5,  0.5], normal: [0.0, -1.0, 0.0] },
            // Right face
            CubeVertex { position: [ 0.5, -0.5, -0.5], normal: [1.0, 0.0, 0.0] },
            CubeVertex { position: [ 0.5,  0.5, -0.5], normal: [1.0, 0.0, 0.0] },
            CubeVertex { position: [ 0.5,  0.5,  0.5], normal: [1.0, 0.0, 0.0] },
            CubeVertex { position: [ 0.5, -0.5,  0.5], normal: [1.0, 0.0, 0.0] },
            // Left face
            CubeVertex { position: [-0.5, -0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
            CubeVertex { position: [-0.5, -0.5,  0.5], normal: [-1.0, 0.0, 0.0] },
            CubeVertex { position: [-0.5,  0.5,  0.5], normal: [-1.0, 0.0, 0.0] },
            CubeVertex { position: [-0.5,  0.5, -0.5], normal: [-1.0, 0.0, 0.0] },
        ];
        
        let indices: Vec<u16> = vec![
            0, 1, 2, 2, 3, 0,       // front
            4, 5, 6, 6, 7, 4,       // back
            8, 9, 10, 10, 11, 8,    // top
            12, 13, 14, 14, 15, 12, // bottom
            16, 17, 18, 18, 19, 16, // right
            20, 21, 22, 22, 23, 20, // left
        ];
        
        (vertices, indices)
    }
    
    /// Создать depth texture
    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        (texture, view)
    }
    
    /// Обновить размер
    pub fn resize(&mut self, width: u32, height: u32) {
        let (texture, view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = texture;
        self.depth_view = view;
    }
    
    /// Обновить instance buffer
    pub fn update_instances(&mut self, instances: &[VoxelGpuInstance]) {
        if instances.is_empty() {
            self.num_instances = 0;
            return;
        }
        
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.instance_buffer = Some(buffer);
        self.num_instances = instances.len() as u32;
    }
    
    /// Обновить uniform буфер
    pub fn update_uniforms(&mut self, uniforms: GlobalUniforms) {
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
    
    /// Рендерить кадр
    pub fn render(&self, view: &wgpu::TextureView) -> wgpu::CommandBuffer {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02,
                            g: 0.02,
                            b: 0.05,
                            a: 1.0,
                        }),
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
            
            if let Some(ref instance_buffer) = self.instance_buffer {
                render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                render_pass.set_index_buffer(self.cube_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..self.num_instances);
            }
        }
        
        encoder.finish()
    }
    
    /// Получить количество отрисованных инстансов
    pub fn instance_count(&self) -> u32 {
        self.num_instances
    }
}

/// Простой UI рендерер (текст поверх сцены)
pub struct UiRenderer {
    // В реальном проекте здесь был бы egui или свой текстовый рендерер
    // Для простоты используем overlay через отдельный пайплайн
}

impl UiRenderer {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Форматировать статистику для вывода в консоль
    pub fn format_stats(stats: &crate::consciousness::OrganismStats) -> String {
        format!(
            "═══════════════════════════════════════\n\
             ║ META-ORGANISM v1.0                  ║\n\
             ═══════════════════════════════════════\n\
             │ Вокселей: {:>7} / {:>7} живых    │\n\
             │ Здоровье: {:>5.1}%                   │\n\
             │ Память: {:>5.1} MB (сжатие: {:>4.1}%) │\n\
             │ Кластеров: {:>5}                    │\n\
             │ Интеграций: {:>4} | Отторжений: {:>4}│\n\
             │ Режим: {:?}                     │\n\
             ═══════════════════════════════════════",
            stats.alive_voxels, stats.total_voxels,
            stats.health_percent,
            stats.memory_mb, stats.memory_saved_percent,
            stats.clusters,
            stats.integrations, stats.rejections,
            stats.mode,
        )
    }
}

impl Default for UiRenderer {
    fn default() -> Self {
        Self::new()
    }
}
