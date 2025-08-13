// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _padding1: f32,
    camera_forward: vec3<f32>,
    _padding2: f32,
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
    mass: f32,
};
@group(1) @binding(0)
var<uniform> black_hole: BlackHoleUniform;

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

// Simplified geodesic step for ray tracing
fn trace_ray(start_pos: vec3<f32>, ray_dir: vec3<f32>, mass: f32, max_steps: i32) -> vec3<f32> {
    var pos = start_pos;
    var dir = normalize(ray_dir);
    let step_size = 0.1;
    let bh_pos = black_hole.position;
    
    for (var i = 0; i < max_steps; i++) {
        let to_bh = pos - bh_pos;
        let r = length(to_bh);
        
        // Check if we hit the event horizon
        if (r <= schwarzschild_radius(mass)) {
            return vec3<f32>(0.0, 0.0, 0.0); // Black (absorbed)
        }
        
        // Check if we escaped far enough
        if (r > 50.0 * mass) {
            // Sample environment based on final direction
            let env_color = sample_environment(dir);
            return env_color;
        }
        
        // Simplified gravitational deflection
        let gravity_strength = mass / (r * r);
        let gravity_dir = -normalize(to_bh);
        
        // Apply gravitational acceleration (simplified)
        dir = normalize(dir + gravity_dir * gravity_strength * step_size);
        
        // Move along ray
        pos += dir * step_size;
    }
    
    // If we ran out of steps, assume it escaped
    return sample_environment(dir);
}

// Sample environment (stars, etc.) based on ray direction
fn sample_environment(dir: vec3<f32>) -> vec3<f32> {
    // Convert direction to spherical coordinates (lat/lon)
    let uv = vec2<f32>(
        atan2(dir.z, dir.x) / (2.0 * 3.14159) + 0.5,
        acos(dir.y) / 3.14159
    );
    
    var color = vec3<f32>(0.0);
    
    // Background color (solid or gradient)
    if (camera.show_stars > 0.5) {
        // Show stars
        let star_density = 500.0;
        let star_coords = floor(uv * star_density);
        let star_hash = fract(sin(dot(star_coords, vec2<f32>(12.9898, 78.233))) * 43758.5453);
        
        if (star_hash > 0.995) {
            color = vec3<f32>(1.0, 1.0, 0.8); // Bright star
        } else if (star_hash > 0.99) {
            color = vec3<f32>(0.5, 0.5, 0.4); // Dim star
        } else {
            // Space background with slight gradient
            let gradient = 0.1 * (1.0 - abs(dir.y));
            color = vec3<f32>(gradient * 0.1, gradient * 0.15, gradient * 0.3);
        }
    } else {
        // Solid gradient background
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
    // Convert screen coordinates to normalized device coordinates
    let screen_pos = (in.tex_coords - 0.5) * 2.0;
    
    // Generate ray direction using camera basis vectors
    // This creates proper perspective projection for ray tracing
    let fov_scale = tan(45.0 * 0.5 * 3.14159 / 180.0); // 45 degree FOV
    
    let ray_dir_camera_space = vec3<f32>(
        screen_pos.x * camera.aspect_ratio * fov_scale,
        screen_pos.y * fov_scale,
        -1.0
    );
    
    // Transform ray direction to world space using camera vectors
    let ray_dir = normalize(
        ray_dir_camera_space.x * camera.camera_right +
        ray_dir_camera_space.y * camera.camera_up +
        ray_dir_camera_space.z * camera.camera_forward
    );
    
    // Trace the ray through spacetime from camera position
    var color = trace_ray(camera.camera_pos, ray_dir, black_hole.mass, 200);
    
    return vec4<f32>(color, 1.0);
}