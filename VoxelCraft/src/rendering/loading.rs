// VoxelCraft - Loading Screen with PULOTIGE Logo

use wgpu::util::DeviceExt;

pub struct LoadingScreen {
    pipeline: wgpu::RenderPipeline,
    time_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup,
}

impl LoadingScreen {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Loading Shader"),
            source: wgpu::ShaderSource::Wgsl(LOADING_SHADER.into()),
        });

        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32, 0.0f32, 0.0f32, 0.0f32]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let time_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Time Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Time Bind Group"),
            layout: &time_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: time_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Loading Pipeline Layout"),
            bind_group_layouts: &[&time_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Loading Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            time_buffer,
            time_bind_group,
        }
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, queue: &wgpu::Queue, time: f32) {
        queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[time, 0.0f32, 0.0f32, 0.0f32]));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Loading Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.05, g: 0.05, b: 0.1, a: 1.0 }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            ..Default::default()
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.time_bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

// Pixel art Socrates with pickaxe - PULOTIGE logo
const LOADING_SHADER: &str = r#"
struct TimeUniform {
    time: f32,
    _pad: vec3<f32>,
}

@group(0) @binding(0) var<uniform> u_time: TimeUniform;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );
    
    var out: VertexOutput;
    out.position = vec4<f32>(positions[idx], 0.0, 1.0);
    out.uv = positions[idx] * 0.5 + 0.5;
    return out;
}

// Draw a pixel at position
fn pixel(uv: vec2<f32>, pos: vec2<f32>, size: f32) -> f32 {
    let d = abs(uv - pos);
    return step(d.x, size) * step(d.y, size);
}

// Pixel art Socrates with pickaxe
fn draw_socrates(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let p = 0.012; // pixel size
    let ox = 0.5;  // center x
    let oy = 0.45; // center y
    
    // Animation - pickaxe swing
    let swing = sin(time * 4.0) * 0.02;
    
    var col = vec3<f32>(0.0);
    
    // === HEAD (skin color) ===
    let skin = vec3<f32>(0.9, 0.75, 0.6);
    // Head shape
    col = mix(col, skin, pixel(uv, vec2<f32>(ox, oy + 8.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox - p, oy + 8.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox + p, oy + 8.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox, oy + 9.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox - p, oy + 9.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox + p, oy + 9.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox, oy + 10.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox - p, oy + 10.0*p), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox + p, oy + 10.0*p), p));
    
    // === BEARD (white/gray) ===
    let beard = vec3<f32>(0.85, 0.85, 0.85);
    col = mix(col, beard, pixel(uv, vec2<f32>(ox, oy + 7.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox - p, oy + 7.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox + p, oy + 7.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox, oy + 6.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox - p, oy + 6.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox + p, oy + 6.0*p), p));
    col = mix(col, beard, pixel(uv, vec2<f32>(ox, oy + 5.0*p), p));
    
    // === HAIR (white) ===
    let hair = vec3<f32>(0.9, 0.9, 0.9);
    col = mix(col, hair, pixel(uv, vec2<f32>(ox - 2.0*p, oy + 9.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox + 2.0*p, oy + 9.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox - 2.0*p, oy + 10.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox + 2.0*p, oy + 10.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox, oy + 11.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox - p, oy + 11.0*p), p));
    col = mix(col, hair, pixel(uv, vec2<f32>(ox + p, oy + 11.0*p), p));
    
    // === EYES (dark) ===
    let eyes = vec3<f32>(0.1, 0.1, 0.1);
    col = mix(col, eyes, pixel(uv, vec2<f32>(ox - p, oy + 9.0*p), p) * 0.5);
    col = mix(col, eyes, pixel(uv, vec2<f32>(ox + p, oy + 9.0*p), p) * 0.5);
    
    // === BODY (toga - white/cream) ===
    let toga = vec3<f32>(0.95, 0.92, 0.85);
    col = mix(col, toga, pixel(uv, vec2<f32>(ox, oy + 4.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - p, oy + 4.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + p, oy + 4.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox, oy + 3.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - p, oy + 3.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + p, oy + 3.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - 2.0*p, oy + 3.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + 2.0*p, oy + 3.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox, oy + 2.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - p, oy + 2.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + p, oy + 2.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - 2.0*p, oy + 2.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + 2.0*p, oy + 2.0*p), p));
    
    // === LEGS ===
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - p, oy + 1.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + p, oy + 1.0*p), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox - p, oy), p));
    col = mix(col, toga, pixel(uv, vec2<f32>(ox + p, oy), p));
    
    // === SANDALS ===
    let sandal = vec3<f32>(0.5, 0.3, 0.2);
    col = mix(col, sandal, pixel(uv, vec2<f32>(ox - p, oy - p), p));
    col = mix(col, sandal, pixel(uv, vec2<f32>(ox + p, oy - p), p));
    
    // === ARM holding pickaxe ===
    col = mix(col, skin, pixel(uv, vec2<f32>(ox + 3.0*p, oy + 4.0*p + swing), p));
    col = mix(col, skin, pixel(uv, vec2<f32>(ox + 4.0*p, oy + 5.0*p + swing), p));
    
    // === PICKAXE ===
    let handle = vec3<f32>(0.55, 0.35, 0.2);
    let metal = vec3<f32>(0.6, 0.65, 0.7);
    
    // Handle
    col = mix(col, handle, pixel(uv, vec2<f32>(ox + 5.0*p, oy + 6.0*p + swing), p));
    col = mix(col, handle, pixel(uv, vec2<f32>(ox + 6.0*p, oy + 7.0*p + swing), p));
    col = mix(col, handle, pixel(uv, vec2<f32>(ox + 7.0*p, oy + 8.0*p + swing), p));
    
    // Pickaxe head
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 8.0*p, oy + 9.0*p + swing), p));
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 9.0*p, oy + 9.0*p + swing), p));
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 7.0*p, oy + 9.0*p + swing), p));
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 8.0*p, oy + 10.0*p + swing), p));
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 10.0*p, oy + 8.0*p + swing), p));
    col = mix(col, metal, pixel(uv, vec2<f32>(ox + 6.0*p, oy + 10.0*p + swing), p));
    
    return col;
}

// Draw text "PULOTIGE"
fn draw_text(uv: vec2<f32>) -> f32 {
    let p = 0.008;
    let y = 0.25;
    var txt = 0.0;
    
    // P
    let px = 0.25;
    txt += pixel(uv, vec2<f32>(px, y), p);
    txt += pixel(uv, vec2<f32>(px, y + p), p);
    txt += pixel(uv, vec2<f32>(px, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(px, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(px, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(px + p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(px + 2.0*p, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(px + p, y + 2.0*p), p);
    
    // U
    let ux = 0.30;
    txt += pixel(uv, vec2<f32>(ux, y + p), p);
    txt += pixel(uv, vec2<f32>(ux, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ux, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(ux, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(ux + p, y), p);
    txt += pixel(uv, vec2<f32>(ux + 2.0*p, y + p), p);
    txt += pixel(uv, vec2<f32>(ux + 2.0*p, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ux + 2.0*p, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(ux + 2.0*p, y + 4.0*p), p);
    
    // L
    let lx = 0.35;
    txt += pixel(uv, vec2<f32>(lx, y), p);
    txt += pixel(uv, vec2<f32>(lx, y + p), p);
    txt += pixel(uv, vec2<f32>(lx, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(lx, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(lx, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(lx + p, y), p);
    txt += pixel(uv, vec2<f32>(lx + 2.0*p, y), p);
    
    // O
    let ox2 = 0.40;
    txt += pixel(uv, vec2<f32>(ox2 + p, y), p);
    txt += pixel(uv, vec2<f32>(ox2, y + p), p);
    txt += pixel(uv, vec2<f32>(ox2, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ox2, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(ox2 + p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(ox2 + 2.0*p, y + p), p);
    txt += pixel(uv, vec2<f32>(ox2 + 2.0*p, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ox2 + 2.0*p, y + 3.0*p), p);
    
    // T
    let tx = 0.45;
    txt += pixel(uv, vec2<f32>(tx, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(tx + p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(tx + 2.0*p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(tx + p, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(tx + p, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(tx + p, y + p), p);
    txt += pixel(uv, vec2<f32>(tx + p, y), p);
    
    // I
    let ix = 0.50;
    txt += pixel(uv, vec2<f32>(ix, y), p);
    txt += pixel(uv, vec2<f32>(ix, y + p), p);
    txt += pixel(uv, vec2<f32>(ix, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ix, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(ix, y + 4.0*p), p);
    
    // G
    let gx = 0.54;
    txt += pixel(uv, vec2<f32>(gx + p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(gx + 2.0*p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(gx, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(gx, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(gx, y + p), p);
    txt += pixel(uv, vec2<f32>(gx + p, y), p);
    txt += pixel(uv, vec2<f32>(gx + 2.0*p, y), p);
    txt += pixel(uv, vec2<f32>(gx + 2.0*p, y + p), p);
    txt += pixel(uv, vec2<f32>(gx + 2.0*p, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(gx + p, y + 2.0*p), p);
    
    // E
    let ex = 0.59;
    txt += pixel(uv, vec2<f32>(ex, y), p);
    txt += pixel(uv, vec2<f32>(ex, y + p), p);
    txt += pixel(uv, vec2<f32>(ex, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ex, y + 3.0*p), p);
    txt += pixel(uv, vec2<f32>(ex, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(ex + p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(ex + 2.0*p, y + 4.0*p), p);
    txt += pixel(uv, vec2<f32>(ex + p, y + 2.0*p), p);
    txt += pixel(uv, vec2<f32>(ex + p, y), p);
    txt += pixel(uv, vec2<f32>(ex + 2.0*p, y), p);
    
    return clamp(txt, 0.0, 1.0);
}

// Loading bar
fn loading_bar(uv: vec2<f32>, progress: f32) -> vec3<f32> {
    let bar_y = 0.15;
    let bar_h = 0.015;
    let bar_x = 0.3;
    let bar_w = 0.4;
    
    if uv.y > bar_y - bar_h && uv.y < bar_y + bar_h && uv.x > bar_x && uv.x < bar_x + bar_w {
        // Border
        if uv.x < bar_x + 0.005 || uv.x > bar_x + bar_w - 0.005 || 
           uv.y < bar_y - bar_h + 0.005 || uv.y > bar_y + bar_h - 0.005 {
            return vec3<f32>(0.5, 0.5, 0.5);
        }
        // Fill
        let fill_x = bar_x + 0.005 + (bar_w - 0.01) * progress;
        if uv.x < fill_x {
            return vec3<f32>(0.3, 0.8, 0.4);
        }
        return vec3<f32>(0.15, 0.15, 0.2);
    }
    return vec3<f32>(0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let time = u_time.time;
    let uv = in.uv;
    
    // Background gradient
    var col = mix(
        vec3<f32>(0.05, 0.05, 0.15),
        vec3<f32>(0.1, 0.1, 0.25),
        uv.y
    );
    
    // Pixel grid effect
    let grid = step(0.95, fract(uv.x * 100.0)) + step(0.95, fract(uv.y * 100.0));
    col += vec3<f32>(0.02) * grid;
    
    // Draw Socrates
    let socrates = draw_socrates(uv, time);
    col = mix(col, socrates, step(0.01, length(socrates)));
    
    // Draw PULOTIGE text
    let text_color = vec3<f32>(0.9, 0.85, 0.7);
    let text = draw_text(uv);
    col = mix(col, text_color, text);
    
    // Loading bar
    let progress = clamp(time / 3.0, 0.0, 1.0);
    let bar = loading_bar(uv, progress);
    col = mix(col, bar, step(0.01, length(bar)));
    
    // "Loading..." text under bar
    let load_text = vec3<f32>(0.6, 0.6, 0.6);
    // Simple dots animation
    let dots = floor(fract(time) * 4.0);
    
    // Vignette
    let vig = 1.0 - length((uv - 0.5) * 1.2);
    col *= vig;
    
    // Fade in
    let fade = clamp(time * 2.0, 0.0, 1.0);
    col *= fade;
    
    return vec4<f32>(col, 1.0);
}
"#;
