// VoxelStrike - Advanced HUD shader with FPS counter, health bars, etc.

struct HudUniform {
    screen_size: vec2<f32>,
    time: f32,
    _padding: f32,
};

@group(0) @binding(0) var<uniform> hud: HudUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) params: vec4<f32>, // x = element_type, y = value, z/w = extra
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) params: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Convert screen coordinates to clip space
    let ndc_x = (input.position.x / hud.screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (input.position.y / hud.screen_size.y) * 2.0;
    
    output.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    output.uv = input.uv;
    output.color = input.color;
    output.params = input.params;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let element_type = i32(input.params.x);
    
    // Element types:
    // 0 = Solid color
    // 1 = Health bar
    // 2 = Ammo bar
    // 3 = Crosshair
    // 4 = FPS counter digit
    // 5 = Glass/transparent panel
    
    var color = input.color;
    
    switch(element_type) {
        case 0: {
            // Solid color
            // No modification needed
        }
        case 1: {
            // Health bar with gradient
            let health_pct = input.params.y;
            let local_x = input.uv.x;
            
            if (local_x > health_pct) {
                color = vec4<f32>(0.1, 0.1, 0.1, 0.7); // Empty portion
            } else {
                // Health color gradient
                let health_color = mix(
                    vec3<f32>(0.9, 0.2, 0.1), // Red (low health)
                    vec3<f32>(0.2, 0.9, 0.2), // Green (full health)
                    health_pct
                );
                color = vec4<f32>(health_color, 0.9);
                
                // Pulse effect when low
                if (health_pct < 0.25) {
                    let pulse = sin(hud.time * 5.0) * 0.3 + 0.7;
                    color.rgb *= pulse;
                }
            }
        }
        case 2: {
            // Ammo bar
            let ammo_pct = input.params.y;
            let local_x = input.uv.x;
            
            if (local_x > ammo_pct) {
                color = vec4<f32>(0.1, 0.1, 0.1, 0.7);
            } else {
                color = vec4<f32>(0.9, 0.7, 0.2, 0.9); // Ammo yellow
            }
        }
        case 3: {
            // Crosshair with gap
            let dist = length(input.uv - vec2<f32>(0.5, 0.5));
            let inner_gap = 0.1;
            let outer_size = 0.4;
            let thickness = 0.08;
            
            // Horizontal and vertical lines
            let h_line = abs(input.uv.y - 0.5) < thickness && abs(input.uv.x - 0.5) > inner_gap && abs(input.uv.x - 0.5) < outer_size;
            let v_line = abs(input.uv.x - 0.5) < thickness && abs(input.uv.y - 0.5) > inner_gap && abs(input.uv.y - 0.5) < outer_size;
            
            if (h_line || v_line) {
                color = vec4<f32>(0.0, 1.0, 0.0, 0.9);
            } else {
                discard;
            }
        }
        case 4: {
            // FPS digit (simplified 7-segment style)
            let digit = i32(input.params.y);
            let local_uv = input.uv;
            
            // Each digit is rendered as 7 segments
            var show = false;
            let seg_w = 0.3;
            let seg_h = 0.1;
            
            // Segment patterns for digits 0-9
            // Using simplified UV-based segments
            let y_top = local_uv.y < 0.15;
            let y_mid = local_uv.y > 0.42 && local_uv.y < 0.58;
            let y_bot = local_uv.y > 0.85;
            let x_left = local_uv.x < 0.2;
            let x_right = local_uv.x > 0.8;
            let y_upper = local_uv.y < 0.5;
            let y_lower = local_uv.y > 0.5;
            
            // Digit patterns
            switch(digit) {
                case 0: { show = (y_top || y_bot || x_left || x_right) && !y_mid; }
                case 1: { show = x_right; }
                case 2: { show = y_top || (x_right && y_upper) || y_mid || (x_left && y_lower) || y_bot; }
                case 3: { show = y_top || y_mid || y_bot || x_right; }
                case 4: { show = (x_left && y_upper) || y_mid || x_right; }
                case 5: { show = y_top || (x_left && y_upper) || y_mid || (x_right && y_lower) || y_bot; }
                case 6: { show = y_top || x_left || y_mid || (x_right && y_lower) || y_bot; }
                case 7: { show = y_top || x_right; }
                case 8: { show = y_top || y_mid || y_bot || x_left || x_right; }
                case 9: { show = y_top || (x_left && y_upper) || y_mid || x_right || y_bot; }
                default: { show = false; }
            }
            
            if (show) {
                color = vec4<f32>(0.0, 1.0, 0.0, 1.0);
            } else {
                color = vec4<f32>(0.0, 0.2, 0.0, 0.3);
            }
        }
        case 5: {
            // Glass panel
            let edge_dist = min(min(input.uv.x, 1.0 - input.uv.x), min(input.uv.y, 1.0 - input.uv.y));
            let edge_glow = smoothstep(0.0, 0.05, edge_dist);
            color.a *= 0.6 + (1.0 - edge_glow) * 0.3;
        }
        default: {
            // Unknown type
        }
    }
    
    return color;
}
