// VoxelCraft - Pure OpenGL ES Renderer

use crate::{GameState, GameUI};
use glam::{Vec3, Mat4};
use std::sync::Arc;
use glow::HasContext;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

/// Camera
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 64.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 70.0_f32.to_radians(),
            aspect: width / height.max(1.0),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.aspect = width / height.max(1.0);
    }
}

/// OpenGL ES Renderer
pub struct Renderer {
    pub camera: Camera,
    time: f32,
    width: u32,
    height: u32,
    // OpenGL context will be managed by the Android activity
    gl: Option<glow::Context>,
    program: Option<glow::Program>,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        log::info!("Creating OpenGL ES Renderer {}x{}", width, height);
        
        Self {
            camera: Camera::new(width as f32, height as f32),
            time: 0.0,
            width,
            height,
            gl: None,
            program: None,
        }
    }

    pub fn init_gl(&mut self, gl: glow::Context) {
        log::info!("Initializing OpenGL...");
        
        unsafe {
            // Create shader program
            let program = gl.create_program().expect("Cannot create program");
            
            let vertex_shader = gl.create_shader(glow::VERTEX_SHADER).expect("Cannot create vertex shader");
            gl.shader_source(vertex_shader, VERTEX_SHADER);
            gl.compile_shader(vertex_shader);
            
            if !gl.get_shader_compile_status(vertex_shader) {
                log::error!("Vertex shader error: {}", gl.get_shader_info_log(vertex_shader));
            }
            
            let fragment_shader = gl.create_shader(glow::FRAGMENT_SHADER).expect("Cannot create fragment shader");
            gl.shader_source(fragment_shader, FRAGMENT_SHADER);
            gl.compile_shader(fragment_shader);
            
            if !gl.get_shader_compile_status(fragment_shader) {
                log::error!("Fragment shader error: {}", gl.get_shader_info_log(fragment_shader));
            }
            
            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);
            
            if !gl.get_program_link_status(program) {
                log::error!("Program link error: {}", gl.get_program_info_log(program));
            }
            
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);
            
            self.program = Some(program);
            log::info!("OpenGL initialized successfully!");
        }
        
        self.gl = Some(gl);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.camera.resize(width as f32, height as f32);
        
        if let Some(gl) = &self.gl {
            unsafe {
                gl.viewport(0, 0, width as i32, height as i32);
            }
        }
    }

    pub fn render(&mut self, state: &GameState, _ui: &GameUI) {
        self.time += 0.016;

        // Update camera
        self.camera.position = state.player.get_eye_position();
        self.camera.yaw = state.player.rotation.0;
        self.camera.pitch = state.player.rotation.1;

        let Some(gl) = &self.gl else {
            return;
        };
        
        let Some(program) = self.program else {
            return;
        };

        unsafe {
            // Animated background color
            let t = self.time;
            let r = (t.sin() * 0.3 + 0.4).clamp(0.0, 1.0);
            let g = ((t * 0.7).sin() * 0.3 + 0.5).clamp(0.0, 1.0);
            let b = ((t * 0.5).sin() * 0.2 + 0.6).clamp(0.0, 1.0);
            
            gl.clear_color(r, g, b, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            
            gl.use_program(Some(program));
            
            // Set uniforms
            let time_loc = gl.get_uniform_location(program, "u_time");
            gl.uniform_1_f32(time_loc.as_ref(), self.time);
            
            let resolution_loc = gl.get_uniform_location(program, "u_resolution");
            gl.uniform_2_f32(resolution_loc.as_ref(), self.width as f32, self.height as f32);
            
            // Draw fullscreen quad
            gl.draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

// Simple vertex shader - fullscreen quad
const VERTEX_SHADER: &str = r#"#version 300 es
precision mediump float;

out vec2 v_uv;

void main() {
    float x = float((gl_VertexID & 1) << 2) - 1.0;
    float y = float((gl_VertexID & 2) << 1) - 1.0;
    v_uv = vec2(x * 0.5 + 0.5, y * 0.5 + 0.5);
    gl_Position = vec4(x, y, 0.0, 1.0);
}
"#;

// Fragment shader with PULOTIGE logo
const FRAGMENT_SHADER: &str = r#"#version 300 es
precision mediump float;

in vec2 v_uv;
out vec4 fragColor;

uniform float u_time;
uniform vec2 u_resolution;

float pixel(vec2 uv, vec2 pos, float size) {
    vec2 d = abs(uv - pos);
    return step(d.x, size) * step(d.y, size);
}

void main() {
    vec2 uv = v_uv;
    float aspect = u_resolution.x / u_resolution.y;
    uv.x *= aspect;
    
    // Background gradient
    vec3 col = mix(
        vec3(0.1, 0.1, 0.2),
        vec3(0.2, 0.15, 0.3),
        uv.y
    );
    
    // Pixel grid effect
    float grid = step(0.97, fract(uv.x * 50.0)) + step(0.97, fract(uv.y * 50.0));
    col += vec3(0.03) * grid;
    
    // Center coordinates
    float cx = aspect * 0.5;
    float cy = 0.5;
    float p = 0.02;
    
    // Draw pixel art character (simplified Socrates with pickaxe)
    vec3 skin = vec3(0.9, 0.75, 0.6);
    vec3 beard = vec3(0.85, 0.85, 0.85);
    vec3 toga = vec3(0.95, 0.92, 0.85);
    vec3 handle = vec3(0.55, 0.35, 0.2);
    vec3 metal = vec3(0.6, 0.65, 0.7);
    
    float swing = sin(u_time * 3.0) * 0.01;
    
    // Head
    col = mix(col, skin, pixel(uv, vec2(cx, cy + 0.08), p));
    col = mix(col, skin, pixel(uv, vec2(cx - p, cy + 0.08), p));
    col = mix(col, skin, pixel(uv, vec2(cx + p, cy + 0.08), p));
    col = mix(col, skin, pixel(uv, vec2(cx, cy + 0.1), p));
    col = mix(col, skin, pixel(uv, vec2(cx - p, cy + 0.1), p));
    col = mix(col, skin, pixel(uv, vec2(cx + p, cy + 0.1), p));
    
    // Beard
    col = mix(col, beard, pixel(uv, vec2(cx, cy + 0.06), p));
    col = mix(col, beard, pixel(uv, vec2(cx - p, cy + 0.06), p));
    col = mix(col, beard, pixel(uv, vec2(cx + p, cy + 0.06), p));
    col = mix(col, beard, pixel(uv, vec2(cx, cy + 0.04), p));
    
    // Hair
    col = mix(col, beard, pixel(uv, vec2(cx, cy + 0.12), p));
    col = mix(col, beard, pixel(uv, vec2(cx - p, cy + 0.12), p));
    col = mix(col, beard, pixel(uv, vec2(cx + p, cy + 0.12), p));
    col = mix(col, beard, pixel(uv, vec2(cx - p*2.0, cy + 0.1), p));
    col = mix(col, beard, pixel(uv, vec2(cx + p*2.0, cy + 0.1), p));
    
    // Body
    col = mix(col, toga, pixel(uv, vec2(cx, cy + 0.02), p));
    col = mix(col, toga, pixel(uv, vec2(cx - p, cy + 0.02), p));
    col = mix(col, toga, pixel(uv, vec2(cx + p, cy + 0.02), p));
    col = mix(col, toga, pixel(uv, vec2(cx, cy), p));
    col = mix(col, toga, pixel(uv, vec2(cx - p, cy), p));
    col = mix(col, toga, pixel(uv, vec2(cx + p, cy), p));
    col = mix(col, toga, pixel(uv, vec2(cx - p*2.0, cy), p));
    col = mix(col, toga, pixel(uv, vec2(cx + p*2.0, cy), p));
    
    // Legs
    col = mix(col, toga, pixel(uv, vec2(cx - p, cy - 0.02), p));
    col = mix(col, toga, pixel(uv, vec2(cx + p, cy - 0.02), p));
    
    // Arm
    col = mix(col, skin, pixel(uv, vec2(cx + p*3.0, cy + 0.02 + swing), p));
    col = mix(col, skin, pixel(uv, vec2(cx + p*4.0, cy + 0.04 + swing), p));
    
    // Pickaxe handle
    col = mix(col, handle, pixel(uv, vec2(cx + p*5.0, cy + 0.06 + swing), p));
    col = mix(col, handle, pixel(uv, vec2(cx + p*6.0, cy + 0.08 + swing), p));
    
    // Pickaxe head
    col = mix(col, metal, pixel(uv, vec2(cx + p*7.0, cy + 0.1 + swing), p));
    col = mix(col, metal, pixel(uv, vec2(cx + p*8.0, cy + 0.1 + swing), p));
    col = mix(col, metal, pixel(uv, vec2(cx + p*6.0, cy + 0.1 + swing), p));
    col = mix(col, metal, pixel(uv, vec2(cx + p*7.0, cy + 0.12 + swing), p));
    
    // PULOTIGE text
    vec3 textCol = vec3(0.9, 0.85, 0.7);
    float textY = 0.25;
    float ts = 0.012;
    float tx = cx - 0.15;
    
    // P
    col = mix(col, textCol, pixel(uv, vec2(tx, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*2.0), ts));
    
    // U
    tx += ts * 3.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts*3.0), ts));
    
    // L
    tx += ts * 4.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    
    // O
    tx += ts * 3.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts*2.0), ts));
    
    // T
    tx += ts * 4.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    
    // I
    tx += ts * 4.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    
    // G
    tx += ts * 2.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts*2.0, textY + ts*2.0), ts));
    
    // E
    tx += ts * 4.0;
    col = mix(col, textCol, pixel(uv, vec2(tx, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*2.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx, textY + ts*3.0), ts));
    col = mix(col, textCol, pixel(uv, vec2(tx + ts, textY + ts*3.0), ts));
    
    // Vignette
    float vig = 1.0 - length((v_uv - 0.5) * 1.3);
    col *= vig;
    
    // Fade in
    float fade = clamp(u_time * 0.5, 0.0, 1.0);
    col *= fade;
    
    fragColor = vec4(col, 1.0);
}
"#;
