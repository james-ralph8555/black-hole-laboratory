// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    background_mode: f32,
    camera_forward: vec3<f32>,
    fovy: f32,
    camera_right: vec3<f32>,
    _padding3: f32,
    camera_up: vec3<f32>,
    _padding4: f32,
    show_stars: f32,
    show_grid: f32,
    show_help: f32,
    aspect_ratio: f32,
    render_width: f32,
    render_height: f32,
    _padding5: vec2<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct BlackHoleUniform {
    position: vec3<f32>,
    _padding1: f32,
    mass: f32,
    spin: f32,
    ray_steps: f32,
    schwarzschild_radius: f32,
    effective_horizon: f32,
    effective_horizon_sq: f32,
    frame_drag_coefficient: f32,
    escape_distance_sq: f32,
    _padding2: vec4<f32>,
};
@group(1) @binding(0)
var<uniform> black_hole: BlackHoleUniform;

@group(2) @binding(0)
var t_sky: texture_2d<f32>;
@group(2) @binding(1)
var s_sky: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_pos = model.position;
    // Pass through clip position directly for full-screen quad
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Ray tracing functions

fn cartesian_to_spherical(pos: vec3<f32>) -> vec3<f32> {
    let r = length(pos);
    let theta = acos(pos.z / r);
    let phi = atan2(pos.y, pos.x);
    return vec3<f32>(r, theta, phi);
}

fn schwarzschild_radius(mass: f32) -> f32 {
    return 2.0 * mass;
}

// Relativistic geodesic ray tracing using Kerr metric approximation
fn trace_ray(start_pos: vec3<f32>, ray_dir: vec3<f32>, mass: f32, max_steps: i32) -> vec3<f32> {
    var pos = start_pos;
    var dir = normalize(ray_dir);
    let bh_pos = black_hole.position;
    
    // Use precomputed constants from uniform buffer
    let effective_horizon_sq = black_hole.effective_horizon_sq;
    let escape_distance_sq = black_hole.escape_distance_sq;
    let frame_drag_coefficient = black_hole.frame_drag_coefficient;
    let rs_factor = 1.5 * black_hole.schwarzschild_radius;
    let up_vector = vec3<f32>(0.0, 1.0, 0.0);

    for (var i = 0; i < max_steps; i++) {
        let to_bh = bh_pos - pos;
        let r_sq = dot(to_bh, to_bh);

        let step_size = clamp(sqrt(r_sq) * 0.1, 0.005, 0.2);
        
        // Efficient horizon check using squared distance
        if (r_sq <= effective_horizon_sq) {
            return vec3<f32>(0.0, 0.0, 0.0);
        }

        // Optimize acceleration calculations with reduced sqrt calls
        let r = sqrt(r_sq);
        let r_cubed = r_sq * r;
        let base_accel = to_bh * rs_factor / r_cubed;
        
        let tangential = cross(up_vector, to_bh);
        let tangential_normalized = normalize(tangential);
        let frame_drag_accel = tangential_normalized * frame_drag_coefficient / (r_sq * r_sq);
        
        let total_accel = base_accel + frame_drag_accel;
        
        dir = normalize(dir + total_accel * step_size);
        pos += dir * step_size;
        
        // Use squared distance comparison to avoid sqrt
        let new_r_sq = dot(bh_pos - pos, bh_pos - pos);
        if (new_r_sq > escape_distance_sq) {
            return sample_environment(dir);
        }
    }

    return sample_environment(dir);
}

// Sample environment (stars, etc.) based on ray direction
fn sample_environment(dir: vec3<f32>) -> vec3<f32> {
    // Convert direction to spherical coordinates for equirectangular mapping.
    // The horizontal texture coordinate (u) is flipped to correctly map the panoramic skybox.
    let uv = vec2<f32>(
        1.0 - (atan2(dir.x, dir.z) / (2.0 * 3.14159) + 0.5),
        acos(dir.y) / 3.14159
    );

    var color = vec3<f32>(0.0);

    if (camera.show_stars > 0.5) {
        if (camera.background_mode < 0.5) {
            // Mode 0: Skybox texture
            color = textureSample(t_sky, s_sky, uv).rgb;
        } else {
            // Mode 1: Procedural stars
            let star_density = 2000.0; // Lower density for "bigger" stars
            let star_coords = floor(uv * star_density);
            let star_hash_base = dot(star_coords, vec2<f32>(12.9898, 78.233));
            let star_hash = fract(sin(star_hash_base) * 43758.5453);
            
            if (star_hash > 0.998) { // Adjusted threshold for lower density
                // Glittery, colorful stars
                let star_color_hash = fract(sin(star_hash_base * 0.5) * 43758.5453);
                let star_color = vec3<f32>(
                    0.6 + 0.4 * sin(star_color_hash * 6.2831), 
                    0.6 + 0.4 * sin(star_color_hash * 6.2831 + 2.0), 
                    0.6 + 0.4 * sin(star_color_hash * 6.2831 + 4.0)
                );
                let brightness = 0.8 + 0.2 * fract(star_hash * 100.0);
                color = star_color * brightness;
            } else if (star_hash > 0.99) { // Adjusted threshold for lower density
                // Dimmer, background stars
                color = vec3<f32>(0.4, 0.4, 0.6);
            } else {
                // Dark sky background with a slight hue
                color = vec3<f32>(0.02, 0.02, 0.05);
            }
        }
    } else {
        // Mode 2: No background (solid gradient)
        let gradient = 0.3 * (1.0 - abs(dir.y));
        color = vec3<f32>(gradient * 0.2, gradient * 0.3, gradient * 0.6);
    }
    
    // Add lat/long grid lines if enabled
    if (camera.show_grid > 0.5) {
        let grid_density = 20.0; // Number of grid lines around sphere
        let lat_lines = abs(fract(uv.y * grid_density) - 0.5);
        let lon_lines = abs(fract(uv.x * grid_density) - 0.5);
        
        let grid_thickness = 0.02;
        if (lat_lines < grid_thickness || lon_lines < grid_thickness) {
            // Grid line color (bright cyan for visibility)
            color = mix(color, vec3<f32>(0.0, 1.0, 1.0), 0.7);
        }
        
        // Add brighter lines for major coordinates (every 4th line)
        let major_lat = abs(fract(uv.y * grid_density / 4.0) - 0.5);
        let major_lon = abs(fract(uv.x * grid_density / 4.0) - 0.5);
        
        if (major_lat < grid_thickness || major_lon < grid_thickness) {
            color = mix(color, vec3<f32>(1.0, 1.0, 0.0), 0.8); // Bright yellow
        }
    }
    
    return color;
}

// Fragment shader with ray tracing

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let screen_pos = (in.tex_coords - 0.5) * 2.0;
    
    // Use matrix-based ray generation like reference shader
    let fov_scale = tan(camera.fovy * 0.5 * 0.017453292);
    
    // Create camera rotation matrix from basis vectors
    let camera_rotation = mat3x3<f32>(
        camera.camera_right,
        camera.camera_up, 
        camera.camera_forward
    );
    
    // Generate ray direction using matrix multiplication
    let ray_dir_camera = vec3<f32>(
        screen_pos.x * camera.aspect_ratio * fov_scale,
        screen_pos.y * fov_scale,
        -1.0
    );
    
    let ray_dir = normalize(camera_rotation * ray_dir_camera);
    
    let color = trace_ray(camera.camera_pos, ray_dir, black_hole.mass, i32(black_hole.ray_steps));
    
    return vec4<f32>(color, 1.0);
}
