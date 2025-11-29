//! Procedural 4K texture generation

use glam::Vec3;
use std::f32::consts::PI;

/// Texture resolution presets
#[derive(Clone, Copy, Debug)]
pub enum TextureResolution {
    Low = 512,
    Medium = 1024,
    High = 2048,
    Ultra = 4096,
}

/// Procedural texture type
#[derive(Clone, Copy, Debug)]
pub enum TextureType {
    Albedo,
    Normal,
    Roughness,
    Metallic,
    AmbientOcclusion,
    Height,
}

/// Procedural texture generator
pub struct TextureGenerator {
    pub resolution: u32,
    seed: u32,
}

impl TextureGenerator {
    pub fn new(resolution: TextureResolution) -> Self {
        Self {
            resolution: resolution as u32,
            seed: 12345,
        }
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// Generate sandstone texture (4K ready)
    pub fn sandstone_albedo(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                // Multi-octave noise for variation
                let noise1 = fbm_noise(nx * 8.0, ny * 8.0, self.seed, 4);
                let noise2 = fbm_noise(nx * 32.0, ny * 32.0, self.seed + 100, 3);
                let noise3 = fbm_noise(nx * 128.0, ny * 128.0, self.seed + 200, 2);
                
                // Base sandstone colors
                let base_r = 0.76 + noise1 * 0.1;
                let base_g = 0.68 + noise1 * 0.08;
                let base_b = 0.50 + noise1 * 0.06;
                
                // Add detail
                let detail = noise2 * 0.05 + noise3 * 0.02;
                
                let r = ((base_r + detail) * 255.0).clamp(0.0, 255.0) as u8;
                let g = ((base_g + detail) * 255.0).clamp(0.0, 255.0) as u8;
                let b = ((base_b + detail) * 255.0).clamp(0.0, 255.0) as u8;
                
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate sandstone normal map
    pub fn sandstone_normal(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                // Sample height at neighboring pixels
                let step = 1.0 / size as f32;
                let h = fbm_noise(nx * 16.0, ny * 16.0, self.seed, 4);
                let hx = fbm_noise((nx + step) * 16.0, ny * 16.0, self.seed, 4);
                let hy = fbm_noise(nx * 16.0, (ny + step) * 16.0, self.seed, 4);
                
                // Calculate normal from height difference
                let dx = (hx - h) * 2.0;
                let dy = (hy - h) * 2.0;
                
                let normal = Vec3::new(-dx, -dy, 1.0).normalize();
                
                // Encode to [0, 255]
                data[idx] = ((normal.x * 0.5 + 0.5) * 255.0) as u8;
                data[idx + 1] = ((normal.y * 0.5 + 0.5) * 255.0) as u8;
                data[idx + 2] = ((normal.z * 0.5 + 0.5) * 255.0) as u8;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate concrete texture
    pub fn concrete_albedo(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                let noise1 = fbm_noise(nx * 4.0, ny * 4.0, self.seed + 300, 5);
                let noise2 = fbm_noise(nx * 16.0, ny * 16.0, self.seed + 400, 4);
                let speckle = fbm_noise(nx * 64.0, ny * 64.0, self.seed + 500, 2);
                
                let base = 0.55 + noise1 * 0.1;
                let detail = noise2 * 0.05;
                let spots = if speckle > 0.7 { 0.03 } else { 0.0 };
                
                let v = ((base + detail + spots) * 255.0).clamp(0.0, 255.0) as u8;
                
                data[idx] = v;
                data[idx + 1] = (v as f32 * 0.98) as u8;
                data[idx + 2] = (v as f32 * 0.95) as u8;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate wood texture
    pub fn wood_albedo(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                // Wood grain pattern
                let grain_freq = 20.0;
                let grain = ((ny * grain_freq + fbm_noise(nx * 2.0, ny * 0.5, self.seed + 600, 3) * 2.0).sin() * 0.5 + 0.5);
                
                let noise = fbm_noise(nx * 8.0, ny * 2.0, self.seed + 700, 3);
                
                // Wood colors
                let light = Vec3::new(0.65, 0.45, 0.25);
                let dark = Vec3::new(0.45, 0.28, 0.15);
                
                let color = light.lerp(dark, grain * 0.7 + noise * 0.3);
                
                data[idx] = (color.x * 255.0) as u8;
                data[idx + 1] = (color.y * 255.0) as u8;
                data[idx + 2] = (color.z * 255.0) as u8;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate metal texture
    pub fn metal_albedo(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                // Brushed metal pattern
                let brush = (ny * 200.0 + fbm_noise(nx * 4.0, ny * 0.5, self.seed + 800, 2) * 10.0).sin() * 0.5 + 0.5;
                
                let base = 0.7;
                let variation = fbm_noise(nx * 8.0, ny * 8.0, self.seed + 900, 3) * 0.1;
                let scratches = if fbm_noise(nx * 32.0, ny * 32.0, self.seed + 1000, 2) > 0.8 { 0.05 } else { 0.0 };
                
                let v = ((base + variation + brush * 0.05 - scratches) * 255.0).clamp(0.0, 255.0) as u8;
                
                data[idx] = v;
                data[idx + 1] = v;
                data[idx + 2] = (v as f32 * 1.02).min(255.0) as u8;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate roughness map
    pub fn roughness_map(&self, base_roughness: f32, variation: f32) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                let noise = fbm_noise(nx * 16.0, ny * 16.0, self.seed + 1100, 4);
                let v = ((base_roughness + noise * variation) * 255.0).clamp(0.0, 255.0) as u8;
                
                data[idx] = v;
                data[idx + 1] = v;
                data[idx + 2] = v;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate ambient occlusion map
    pub fn ao_map(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                let noise = fbm_noise(nx * 8.0, ny * 8.0, self.seed + 1200, 4);
                let v = ((0.8 + noise * 0.2) * 255.0).clamp(0.0, 255.0) as u8;
                
                data[idx] = v;
                data[idx + 1] = v;
                data[idx + 2] = v;
                data[idx + 3] = 255;
            }
        }

        data
    }

    /// Generate crate texture with planks
    pub fn crate_albedo(&self) -> Vec<u8> {
        let size = self.resolution as usize;
        let mut data = vec![0u8; size * size * 4];

        for y in 0..size {
            for x in 0..size {
                let idx = (y * size + x) * 4;
                
                let nx = x as f32 / size as f32;
                let ny = y as f32 / size as f32;
                
                // Plank pattern
                let plank_count = 4.0;
                let plank_y = (ny * plank_count).fract();
                let plank_edge = if plank_y < 0.02 || plank_y > 0.98 { 0.15 } else { 0.0 };
                
                // Wood grain
                let plank_id = (ny * plank_count).floor();
                let grain = ((plank_y * 50.0 + fbm_noise(nx * 3.0, plank_id, self.seed + 1300, 2) * 3.0).sin() * 0.5 + 0.5);
                
                let noise = fbm_noise(nx * 8.0, ny * 4.0, self.seed + 1400, 3);
                
                let base_color = Vec3::new(0.5, 0.32, 0.15);
                let dark_color = Vec3::new(0.35, 0.22, 0.1);
                
                let color = base_color.lerp(dark_color, grain * 0.5 + noise * 0.3 + plank_edge);
                
                data[idx] = (color.x * 255.0) as u8;
                data[idx + 1] = (color.y * 255.0) as u8;
                data[idx + 2] = (color.z * 255.0) as u8;
                data[idx + 3] = 255;
            }
        }

        data
    }
}

/// Fractional Brownian Motion noise
fn fbm_noise(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    for i in 0..octaves {
        value += amplitude * perlin_noise(x * frequency, y * frequency, seed + i);
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

/// Simple Perlin-like noise
fn perlin_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    
    // Smoothstep
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    
    // Hash corners
    let n00 = hash_noise(xi, yi, seed);
    let n10 = hash_noise(xi + 1, yi, seed);
    let n01 = hash_noise(xi, yi + 1, seed);
    let n11 = hash_noise(xi + 1, yi + 1, seed);
    
    // Bilinear interpolation
    let nx0 = n00 * (1.0 - u) + n10 * u;
    let nx1 = n01 * (1.0 - u) + n11 * u;
    
    nx0 * (1.0 - v) + nx1 * v
}

/// Hash function for pseudo-random values
fn hash_noise(x: i32, y: i32, seed: u32) -> f32 {
    let n = (x.wrapping_mul(374761393) as u32)
        .wrapping_add((y.wrapping_mul(668265263)) as u32)
        .wrapping_add(seed);
    let n = n.wrapping_mul(n ^ (n >> 13));
    let n = n.wrapping_mul(n ^ (n >> 7));
    (n & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32
}

impl Default for TextureGenerator {
    fn default() -> Self {
        Self::new(TextureResolution::Ultra)
    }
}
