use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;

pub struct Renderer {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    point_buffer: Option<Buffer>,
    num_points: usize,
    // HIP/ROCm fallback for AMD Vega 20 (would need rocm-smi integration)
    use_hip_fallback: bool,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self, Box<dyn std::error::Error>> {
        let size = window.inner_size();
        
        // Create instance with Vulkan backend
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN | Backends::GL,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window)?;
        
        // Request adapter
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).ok_or("Failed to find adapter")?;
        
        // Check for AMD Vega 20 (would need GPU detection)
        let use_hip_fallback = false; // TODO: Detect AMD Vega 20
        
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
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
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);
        
        // Create shader
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Point Cloud Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/point_cloud.wgsl").into()),
        });
        
        // Create render pipeline
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[VertexBufferLayout {
                    array_stride: 24, // 3 f32 position + 3 f32 color
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
                    ],
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            point_buffer: None,
            num_points: 0,
            use_hip_fallback,
        })
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }
    
    pub fn update_point_cloud(&mut self, points: &[([f32; 3], [f32; 3])]) {
        if points.is_empty() {
            return;
        }
        
        // Flatten point data
        let mut data = Vec::with_capacity(points.len() * 6);
        for (pos, color) in points {
            data.extend_from_slice(pos);
            data.extend_from_slice(color);
        }
        
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Point Cloud Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: BufferUsages::VERTEX,
        });
        
        self.point_buffer = Some(buffer);
        self.num_points = points.len();
    }
    
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            
            render_pass.set_pipeline(&self.render_pipeline);
            
            if let Some(ref buffer) = self.point_buffer {
                render_pass.set_vertex_buffer(0, buffer.slice(..));
                render_pass.draw(0..self.num_points as u32, 0..1);
            }
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
}

// HIP/ROCm fallback implementation (stub)
pub struct HipFallback {
    // Would integrate with rocm-smi for AMD Vega 20
}

impl HipFallback {
    pub fn new() -> Option<Self> {
        // TODO: Detect and initialize HIP/ROCm
        None
    }
    
    pub fn render(&mut self, _points: &[([f32; 3], [f32; 3])]) {
        // HIP rendering implementation
    }
}
