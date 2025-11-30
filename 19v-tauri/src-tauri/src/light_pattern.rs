use std::{convert::TryInto, time::Instant};

use anyhow::{Context, Result};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const WORKGROUP_SIZE: u32 = 64;
const DEFAULT_ELEMENTS: usize = 512;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default, Debug)]
struct LightUniformRaw {
    time: f32,
    bias: f32,
    randomness: f32,
    pad: f32,
}

#[derive(Clone, Debug)]
pub struct LightPatternSnapshot {
    pub values: Vec<f32>,
    pub timestamp: Instant,
}

pub struct LightPatternCompute {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    pattern_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    staging_buffer: wgpu::Buffer,
    element_count: usize,
    start: Instant,
}

impl LightPatternCompute {
    pub fn new(element_count: usize) -> Result<Self> {
        pollster::block_on(Self::init(element_count))
    }

    async fn init(element_count: usize) -> Result<Self> {
        let element_count = if element_count == 0 {
            DEFAULT_ELEMENTS
        } else {
            element_count
        };

        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to obtain GPU adapter")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("19V LightPattern Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("LightPattern Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/light_pattern.wgsl").into(),
            ),
        });

        let pattern_byte_len = (element_count * std::mem::size_of::<f32>()) as u64;

        let pattern_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("LightPattern Storage"),
            size: pattern_byte_len,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("LightPattern Uniform"),
            contents: bytemuck::bytes_of(&LightUniformRaw::default()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("LightPattern Staging"),
            size: pattern_byte_len,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("LightPattern Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("LightPattern BindGroup"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pattern_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("LightPattern Pipeline"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("LightPattern Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
            compilation_options: Default::default(),
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            bind_group,
            pattern_buffer,
            uniform_buffer,
            staging_buffer,
            element_count,
            start: Instant::now(),
        })
    }

    pub fn step(&mut self, energy_bias: f32, stochastic: f32) -> Result<LightPatternSnapshot> {
        let elapsed = self.start.elapsed().as_secs_f32();
        let uniform = LightUniformRaw {
            time: elapsed,
            bias: energy_bias,
            randomness: stochastic,
            pad: 0.0,
        };
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("LightPattern Encoder"),
            });

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("LightPattern Pass"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            let workgroups = ((self.element_count as u32) + WORKGROUP_SIZE - 1) / WORKGROUP_SIZE;
            pass.dispatch_workgroups(workgroups, 1, 1);
        }

        encoder.copy_buffer_to_buffer(
            &self.pattern_buffer,
            0,
            &self.staging_buffer,
            0,
            (self.element_count * std::mem::size_of::<f32>()) as u64,
        );

        self.queue.submit(Some(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);

        let slice = self.staging_buffer.slice(..);
        pollster::block_on(slice.map_async(wgpu::MapMode::Read))?;
        let data = slice.get_mapped_range();
        let mut values = Vec::with_capacity(self.element_count);
        for chunk in data.chunks_exact(std::mem::size_of::<f32>()) {
            values.push(f32::from_le_bytes(chunk.try_into().unwrap()));
        }
        drop(data);
        self.staging_buffer.unmap();

        Ok(LightPatternSnapshot {
            values,
            timestamp: Instant::now(),
        })
    }

    pub fn element_count(&self) -> usize {
        self.element_count
    }
}

impl Default for LightPatternCompute {
    fn default() -> Self {
        Self::new(DEFAULT_ELEMENTS).expect("Failed to initialize LightPatternCompute")
    }
}

#[derive(Clone, Debug, Default)]
pub struct LightPatternState {
    pub snapshot: LightPatternSnapshot,
}

impl LightPatternState {
    pub fn new(snapshot: LightPatternSnapshot) -> Self {
        Self { snapshot }
    }

    pub fn blend_energy(&self) -> f32 {
        if self.snapshot.values.is_empty() {
            return 0.0;
        }
        self.snapshot.values.iter().sum::<f32>() / self.snapshot.values.len() as f32
    }
}
