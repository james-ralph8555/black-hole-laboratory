use cfg_if::cfg_if;
mod texture;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = toggleHelp)]
    fn js_toggle_help();

    #[wasm_bindgen(js_name = setHelpVisible)]
    fn js_set_help_visible(visible: bool);

    #[wasm_bindgen(js_name = updateDebugInfo)]
    fn js_update_debug_info(
        position: &[f32],
        orientation: &[f32],
        last_key: &str,
        fps: f32,
        render_width: f32,
        render_height: f32,
        velocity: &[f32],
    );

    #[wasm_bindgen(js_name = updateProfilingInfo)]
    fn js_update_profiling_info(
        cpu_time: f32,
        gpu_time: f32,
        update_time: f32,
        render_time: f32,
        gpu_supported: bool,
    );

    #[wasm_bindgen(js_name = updateFpsCounter)]
    fn js_update_fps_counter(fps: f32, visible: bool);

    #[wasm_bindgen(js_name = setProfilingVisible)]
    fn js_set_profiling_visible(visible: bool);

    #[wasm_bindgen(js_name = hideLoadingScreen)]
    fn js_hide_loading_screen();
}

use wgpu::util::{DeviceExt, StagingBelt};

mod camera;
use camera::{Camera, CameraController, CameraUniform};

mod profiler;
use profiler::Profiler;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct BlackHoleUniform {
    /// Position of black hole in world space (16-byte aligned)
    position: [f32; 4], // Use 4 floats for proper vec3 alignment
    /// Mass of black hole
    mass: f32,
    /// Spin parameter (dimensionless)
    spin: f32,
    /// Ray marching steps
    ray_steps: f32,
    /// Precomputed Schwarzschild radius (2 * mass)
    schwarzschild_radius: f32,
    /// Precomputed effective horizon for Kerr black hole
    effective_horizon: f32,
    /// Precomputed effective horizon squared
    effective_horizon_sq: f32,
    /// Precomputed frame drag coefficient
    frame_drag_coefficient: f32,
    /// Precomputed escape distance squared
    escape_distance_sq: f32,
    /// Accretion disk inner radius
    disk_inner_radius: f32,
    /// Accretion disk outer radius
    disk_outer_radius: f32,
    /// Accretion disk temperature (Kelvin)
    disk_temperature: f32,
    /// Accretion disk opacity
    disk_opacity: f32,
    /// Time for animation
    time: f32,
    /// Padding to reach exactly 80 bytes (shader expects 80)
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Arc<Window>,
    render_pipeline: wgpu::RenderPipeline,
    debug_render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera: Camera,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    camera_controller: CameraController,
    black_hole: simulation::KerrBlackHole,
    last_help_state: bool,
    last_profiling_state: bool,
    black_hole_uniform: BlackHoleUniform,
    black_hole_buffer: wgpu::Buffer,
    black_hole_bind_group: wgpu::BindGroup,
    sky_texture: texture::Texture,
    sky_bind_group: wgpu::BindGroup,
    background_mode: u32,
    // HDR and bloom textures
    hdr_texture: wgpu::Texture,
    hdr_view: wgpu::TextureView,
    bloom_texture: wgpu::Texture,
    bloom_view: wgpu::TextureView,
    bloom_temp_texture: wgpu::Texture,
    bloom_temp_view: wgpu::TextureView,
    // Bloom pipelines (fragment shader based)
    bloom_extract_pipeline: wgpu::RenderPipeline,
    bloom_blur_h_pipeline: wgpu::RenderPipeline,
    bloom_blur_v_pipeline: wgpu::RenderPipeline,
    tone_mapping_pipeline: wgpu::RenderPipeline,
    // Bloom bind groups
    bloom_extract_bind_group: wgpu::BindGroup,
    bloom_blur_bind_group: wgpu::BindGroup,
    tone_mapping_bind_group: wgpu::BindGroup,
    // Texture sampler for bloom operations
    texture_sampler: wgpu::Sampler,
    // Debug parameters
    debug_fov: f32,
    debug_mass: f32,
    debug_spin: f32,
    debug_ray_steps: f32,
    #[cfg(not(target_arch = "wasm32"))]
    last_render_time: std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    last_render_time: f64, // Use f64 for JS performance.now() timestamp
    profiler: Profiler,
    staging_belt: StagingBelt,
}

impl<'a> State<'a> {
    async fn new(window: Arc<Window>) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::all(),
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        #[cfg(target_arch = "wasm32")]
        log::info!("Creating surface for window");

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        #[cfg(target_arch = "wasm32")]
        log::info!("Requesting adapter");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        log::info!("Got adapter, requesting device");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: if adapter
                        .features()
                        .contains(wgpu::Features::TIMESTAMP_QUERY)
                    {
                        wgpu::Features::TIMESTAMP_QUERY
                    } else {
                        wgpu::Features::empty()
                    },
                    required_limits: adapter.limits(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        #[cfg(target_arch = "wasm32")]
        log::info!(
            "Got device, configuring surface. Window size: {}x{}",
            size.width,
            size.height
        );

        let surface_caps = surface.get_capabilities(&adapter);

        // Be more defensive about surface format selection
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Use actual window size for viewport-responsive rendering
        // Ensure we don't configure with zero dimensions and respect texture size limits
        let limits = device.limits();
        let max_texture_size = limits.max_texture_dimension_2d;
        let width = size.width.max(1).min(max_texture_size);
        let height = size.height.max(1).min(max_texture_size);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        #[cfg(target_arch = "wasm32")]
        log::info!(
            "Requested size: {}x{}, max texture size: {}, using: {}x{}, format: {:?}",
            size.width,
            size.height,
            max_texture_size,
            width,
            height,
            surface_format
        );

        surface.configure(&device, &config);

        #[cfg(target_arch = "wasm32")]
        log::info!("Surface configured successfully");

        // Create full-screen quad for ray tracing
        let quad_vertices = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];
        let quad_indices = vec![0u16, 1, 2, 0, 2, 3];

        // Create vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&quad_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&quad_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = quad_indices.len() as u32;

        // Create camera with aspect ratio matching the actual window size
        // Start the camera at a good position to view the black hole
        let camera = Camera::new(
            (0.0, -1.0, -40.0), // Start far back for better testing perspective
            (0.0, 0.0, 0.0),   // Look towards the black hole at origin
            cgmath::Vector3::unit_y(),
            width as f32 / height as f32, // Dynamic aspect ratio
            80.0,
            0.1,
            1000.0, // Increase far plane for space exploration
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj_with_resolution(
            &camera,
            true,
            false,
            false,
            width as f32,
            height as f32,
        ); // Default: stars on, grid off, help off (flash message instead)

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        // Create default black hole for the simulation - SCHWARZSCHILD FOR TESTING
        let black_hole = simulation::KerrBlackHole::new(1.0, 1.0); // Maximal spin for frame-dragging

        // Initialize debug parameters
        let debug_fov = 80.0;
        let debug_mass = black_hole.mass;
        let debug_spin = black_hole.spin;
        let debug_ray_steps = 250.0;

        // Create black hole uniform with precomputed constants
        let schwarzschild_radius = 2.0 * debug_mass;
        let a = debug_spin * debug_mass;
        let effective_horizon = debug_mass + (debug_mass * debug_mass - a * a).max(0.0).sqrt();
        let effective_horizon_sq = effective_horizon * effective_horizon;
        let frame_drag_coefficient =
            (debug_spin * debug_spin) * schwarzschild_radius * schwarzschild_radius * 0.5;
        let escape_distance = 200.0 * debug_mass;
        let escape_distance_sq = escape_distance * escape_distance;

        // Accretion disk parameters
        let disk_inner_radius = 3.0 * schwarzschild_radius; // ISCO for Schwarzschild
        let disk_outer_radius = 5.5 * schwarzschild_radius; // Half the width (was 8.0, now 5.5)
        let disk_temperature = 10000.0; // Kelvin - hot enough to glow blue-white
        let disk_opacity = 0.8;

        let black_hole_uniform = BlackHoleUniform {
            position: [0.0, 0.0, 0.0, 0.0], // Centered at origin with padding
            mass: debug_mass,
            spin: debug_spin,
            ray_steps: debug_ray_steps,
            schwarzschild_radius,
            effective_horizon,
            effective_horizon_sq,
            frame_drag_coefficient,
            escape_distance_sq,
            disk_inner_radius,
            disk_outer_radius,
            disk_temperature,
            disk_opacity,
            time: 0.0,
            _padding: [0.0; 3],
        };

        let black_hole_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BlackHole Buffer"),
            contents: bytemuck::cast_slice(&[black_hole_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let black_hole_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                label: Some("black_hole_bind_group_layout"),
            });

        let black_hole_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &black_hole_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: black_hole_buffer.as_entire_binding(),
            }],
            label: Some("black_hole_bind_group"),
        });

        // Query the adapter's hardware capabilities.
        let limits = device.limits();
        let mut background_mode = 0; // Default to texture mode (0)

        // Check if the device's max_texture_dimension_2d can handle our high-resolution skybox.
        // The milkyway.jpg is typically 6000px wide, so we check for that capability.
        let sky_texture = if limits.max_texture_dimension_2d >= 6000 {
            #[cfg(target_arch = "wasm32")]
            log::info!(
                "Device supports high-res texture (limit: {}px).",
                limits.max_texture_dimension_2d
            );

            let sky_bytes = include_bytes!("milkyway.jpg");
            texture::Texture::from_bytes(&device, &queue, sky_bytes, "milkyway.jpg").unwrap()
        } else {
            #[cfg(target_arch = "wasm32")]
            log::warn!(
                "Device max texture size ({}) is less than required (6000px). Falling back to procedural stars.",
                limits.max_texture_dimension_2d
            );

            // Set mode to procedural stars (1)
            background_mode = 1;
            // Create a dummy 1x1 texture to satisfy the render pipeline
            texture::Texture::create_1x1_black_pixel(&device, &queue, "fallback_texture").unwrap()
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sky_texture.sampler),
                },
            ],
            label: Some("sky_bind_group"),
        });

        // Create the shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create the render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &black_hole_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        // Create HDR texture for rendering - use Rgba16Float for true HDR
        let hdr_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("HDR Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float, // HDR format for extended range
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let hdr_view = hdr_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create bloom texture for bright pixel extraction and blur
        let bloom_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom Texture"),
            size: wgpu::Extent3d {
                width: (width / 2).max(1), // Half resolution for performance, but at least 1
                height: (height / 2).max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let bloom_view = bloom_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create temporary bloom texture for ping-pong blur
        let bloom_temp_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Bloom Temp Texture"),
            size: wgpu::Extent3d {
                width: (width / 2).max(1),
                height: (height / 2).max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let bloom_temp_view = bloom_temp_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create HDR render pipeline (renders to HDR texture)
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("HDR Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float, // HDR format
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create debug render pipeline (renders to surface)
        let debug_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Debug Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format, // Surface format
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bloom fragment shaders
        let bloom_fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Bloom Fragment Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("bloom_fragment.wgsl").into()),
        });

        // Create tone mapping shader
        let tone_mapping_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tone Mapping Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tone_mapping.wgsl").into()),
        });

        // Create bloom bind group layout for fragment shaders
        let bloom_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Bloom Bind Group Layout"),
        });

        // Create bloom extract render pipeline
        let bloom_extract_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Bloom Extract Pipeline Layout"),
            bind_group_layouts: &[&bloom_bind_group_layout],
            push_constant_ranges: &[],
        });

        let bloom_extract_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Extract Pipeline"),
            layout: Some(&bloom_extract_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &bloom_fragment_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &bloom_fragment_shader,
                entry_point: "extract_bright_pixels",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bloom blur horizontal pipeline
        let bloom_blur_h_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Blur Horizontal Pipeline"),
            layout: Some(&bloom_extract_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &bloom_fragment_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &bloom_fragment_shader,
                entry_point: "blur_horizontal",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bloom blur vertical pipeline
        let bloom_blur_v_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Bloom Blur Vertical Pipeline"),
            layout: Some(&bloom_extract_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &bloom_fragment_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &bloom_fragment_shader,
                entry_point: "blur_vertical",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create tone mapping bind group layout
        let tone_mapping_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Tone Mapping Bind Group Layout"),
        });

        // Create tone mapping pipeline layout
        let tone_mapping_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Tone Mapping Pipeline Layout"),
            bind_group_layouts: &[&tone_mapping_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create tone mapping render pipeline
        let tone_mapping_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tone Mapping Pipeline"),
            layout: Some(&tone_mapping_layout),
            vertex: wgpu::VertexState {
                module: &tone_mapping_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &tone_mapping_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format, // Final surface format
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create texture sampler for tone mapping
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bloom extract bind group
        let bloom_extract_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("Bloom Extract Bind Group"),
        });

        // Create bloom blur bind group (for ping-pong between bloom textures)
        let bloom_blur_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bloom_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bloom_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("Bloom Blur Bind Group"),
        });

        // Create tone mapping bind group
        let tone_mapping_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &tone_mapping_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&hdr_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&bloom_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("Tone Mapping Bind Group"),
        });

        let mut camera_controller = CameraController::new(4.0);
        camera_controller.set_initial_camera_state(camera.eye, 270.0, 0.0);

        #[cfg(target_arch = "wasm32")]
        log::info!("Render pipeline created");

        let profiler = Profiler::new(&device);

        // Initialize staging belt for efficient buffer updates
        // Chunk size should be larger than uniform buffer updates (typically 256-4096 bytes is good)
        let staging_belt = StagingBelt::new(1024);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            debug_render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_controller,
            black_hole,
            last_help_state: false, // Match camera_controller.show_help initial state
            last_profiling_state: false, // Match camera_controller.show_profiling initial state
            black_hole_uniform,
            black_hole_buffer,
            black_hole_bind_group,
            sky_texture,
            sky_bind_group,
            background_mode, // 0: texture, 1: procedural, 2: none
            // HDR and bloom textures
            hdr_texture,
            hdr_view,
            bloom_texture,
            bloom_view,
            bloom_temp_texture,
            bloom_temp_view,
            // Bloom pipelines (fragment shader based)
            bloom_extract_pipeline,
            bloom_blur_h_pipeline,
            bloom_blur_v_pipeline,
            tone_mapping_pipeline,
            // Bloom bind groups
            bloom_extract_bind_group,
            bloom_blur_bind_group,
            tone_mapping_bind_group,
            texture_sampler,
            // Initialize debug parameters
            debug_fov,
            debug_mass,
            debug_spin,
            debug_ray_steps,
            #[cfg(not(target_arch = "wasm32"))]
            last_render_time: std::time::Instant::now(),
            #[cfg(target_arch = "wasm32")]
            last_render_time: web_sys::window().unwrap().performance().unwrap().now(),
            profiler,
            staging_belt,
        }
    }

    fn update_camera_fov(&mut self) {
        self.camera.fovy = self.debug_fov;
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        ..
                    },
                ..
            } => {
                if *state == winit::event::ElementState::Pressed {
                    match physical_key {
                        PhysicalKey::Code(winit::keyboard::KeyCode::KeyB) => {
                            self.background_mode = (self.background_mode + 1) % 3;
                            return true;
                        }
                        _ => {}
                    }
                }

                if let PhysicalKey::Code(key) = *physical_key {
                    self.camera_controller.process_keyboard(key, *state)
                } else {
                    false
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == winit::event::MouseButton::Left {
                    self.camera_controller.process_mouse_button(*state);
                    true
                } else {
                    false
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.camera_controller.process_cursor_move(*position);
                true
            }
            WindowEvent::Touch(touch) => {
                self.camera_controller.process_touch(touch, self.size);
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(*delta);
                true
            }
            _ => false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            // Ensure we don't configure with zero dimensions and respect texture size limits
            let limits = self.device.limits();
            let max_texture_size = limits.max_texture_dimension_2d;
            let width = new_size.width.max(1).min(max_texture_size);
            let height = new_size.height.max(1).min(max_texture_size);

            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);

            // Recreate HDR texture with new size
            self.hdr_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("HDR Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            });
            self.hdr_view = self.hdr_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Recreate bloom texture with new size (half resolution)
            self.bloom_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Bloom Texture"),
                size: wgpu::Extent3d {
                    width: (width / 2).max(1),
                    height: (height / 2).max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.bloom_view = self.bloom_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Recreate bloom temp texture with new size (half resolution)
            self.bloom_temp_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Bloom Temp Texture"),
                size: wgpu::Extent3d {
                    width: (width / 2).max(1),
                    height: (height / 2).max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            self.bloom_temp_view = self.bloom_temp_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // Update camera aspect ratio to match new window dimensions
            self.camera
                .update_aspect_ratio(width as f32 / height as f32);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.profiler.begin_frame();

        self.profiler.begin_update();
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // For WASM, use actual time measurements from performance.now()
                let now = web_sys::window().unwrap().performance().unwrap().now();
                let dt_ms = now - self.last_render_time;
                let dt = std::time::Duration::from_secs_f64(dt_ms / 1000.0);
                self.last_render_time = now;
                self.camera_controller.update_camera(&mut self.camera, dt);
            } else {
                let now = std::time::Instant::now();
                let dt = now - self.last_render_time;
                self.last_render_time = now;
                self.camera_controller.update_camera(&mut self.camera, dt);
            }
        }
        self.profiler.end_update();

        // Update camera uniform with toggle states and current resolution
        let show_stars = self.background_mode != 2;
        self.camera_uniform.update_view_proj_with_resolution(
            &self.camera,
            show_stars,
            self.camera_controller.show_grid,
            self.camera_controller.show_help,
            self.config.width as f32,
            self.config.height as f32,
        );
        self.camera_uniform.background_mode = if self.background_mode == 1 { 1.0 } else { 0.0 };

        // Update debug parameters from global state (WASM) or local state (native)
        #[cfg(target_arch = "wasm32")]
        {
            unsafe {
                if let Some(params) = &DEBUG_PARAMS {
                    if let Ok(params) = params.lock() {
                        self.debug_fov = params.fov;
                        self.debug_mass = params.mass;
                        self.debug_spin = params.spin;
                        self.debug_ray_steps = params.ray_steps;

                        // Update camera FOV if it changed
                        if (self.camera.fovy - self.debug_fov).abs() > 0.001 {
                            self.update_camera_fov();
                        }
                    }
                }
            }
        }

        // Update black hole uniform with debug parameters and recompute constants
        self.black_hole_uniform.mass = self.debug_mass;
        self.black_hole_uniform.spin = self.debug_spin;
        self.black_hole_uniform.ray_steps = self.debug_ray_steps;

        // Recompute precomputed constants when parameters change
        let schwarzschild_radius = 2.0 * self.debug_mass;
        let a = self.debug_spin * self.debug_mass;
        let effective_horizon =
            self.debug_mass + (self.debug_mass * self.debug_mass - a * a).max(0.0).sqrt();
        let effective_horizon_sq = effective_horizon * effective_horizon;
        let frame_drag_coefficient =
            (self.debug_spin * self.debug_spin) * schwarzschild_radius * schwarzschild_radius * 0.5;
        let escape_distance = 200.0 * self.debug_mass;
        let escape_distance_sq = escape_distance * escape_distance;

        // Update disk parameters based on current mass
        let disk_inner_radius = 3.0 * schwarzschild_radius; // ISCO for Schwarzschild
        let disk_outer_radius = 5.5 * schwarzschild_radius; // Half the width (was 8.0, now 5.5)

        self.black_hole_uniform.schwarzschild_radius = schwarzschild_radius;
        self.black_hole_uniform.effective_horizon = effective_horizon;
        self.black_hole_uniform.effective_horizon_sq = effective_horizon_sq;
        self.black_hole_uniform.frame_drag_coefficient = frame_drag_coefficient;
        self.black_hole_uniform.escape_distance_sq = escape_distance_sq;
        self.black_hole_uniform.disk_inner_radius = disk_inner_radius;
        self.black_hole_uniform.disk_outer_radius = disk_outer_radius;
        
        // Update time for Brownian motion animation
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                self.black_hole_uniform.time = (web_sys::window().unwrap().performance().unwrap().now() / 1000.0) as f32;
            } else {
                self.black_hole_uniform.time = self.last_render_time.elapsed().as_secs_f32();
            }
        }

        // Update HTML help overlay for WASM
        #[cfg(target_arch = "wasm32")]
        {
            // Update help overlay when state changes
            if self.camera_controller.show_help != self.last_help_state {
                self.last_help_state = self.camera_controller.show_help;
                js_set_help_visible(self.camera_controller.show_help);
            }

            // Update profiling overlay when state changes
            if self.camera_controller.show_profiling != self.last_profiling_state {
                self.last_profiling_state = self.camera_controller.show_profiling;
                js_set_profiling_visible(self.camera_controller.show_profiling);
            }

            // Update debug info continuously when help is visible
            if self.camera_controller.show_help {
                let position = [self.camera.eye.x, self.camera.eye.y, self.camera.eye.z];
                let orientation = [self.camera_controller.yaw, self.camera_controller.pitch];
                let velocity = [
                    self.camera_controller.current_velocity.x,
                    self.camera_controller.current_velocity.y,
                    self.camera_controller.current_velocity.z,
                ];
                let last_key = self
                    .camera_controller
                    .last_key
                    .map(|k| format!("{:?}", k))
                    .unwrap_or_else(|| "None".to_string());

                js_update_debug_info(
                    &position,
                    &orientation,
                    &last_key,
                    self.camera_controller.fps,
                    self.config.width as f32,
                    self.config.height as f32,
                    &velocity,
                );
            }

            // Update profiling info independently when profiling is visible
            if self.camera_controller.show_profiling {
                if let Some(latest_sample) = self.profiler.get_latest_sample() {
                    js_update_profiling_info(
                        latest_sample.cpu_time_ms,
                        latest_sample.gpu_time_ms.unwrap_or(0.0),
                        latest_sample.update_time_ms,
                        latest_sample.render_encode_time_ms,
                        self.profiler.is_gpu_timing_supported(),
                    );
                }
            }

            // Update FPS counter
            js_update_fps_counter(self.camera_controller.fps, self.camera_controller.show_fps);
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.profiler.begin_render_encode();
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Update uniform buffers using StagingBelt for better performance
        {
            let camera_uniform_array = [self.camera_uniform];
            let camera_data = bytemuck::cast_slice(&camera_uniform_array);
            let mut camera_view = self.staging_belt.write_buffer(
                &mut encoder,
                &self.camera_buffer,
                0,
                wgpu::BufferSize::new(camera_data.len() as u64).unwrap(),
                &self.device,
            );
            camera_view.copy_from_slice(camera_data);
        }

        {
            let black_hole_uniform_array = [self.black_hole_uniform];
            let black_hole_data = bytemuck::cast_slice(&black_hole_uniform_array);
            let mut black_hole_view = self.staging_belt.write_buffer(
                &mut encoder,
                &self.black_hole_buffer,
                0,
                wgpu::BufferSize::new(black_hole_data.len() as u64).unwrap(),
                &self.device,
            );
            black_hole_view.copy_from_slice(black_hole_data);
        }

        // Begin GPU timing
        self.profiler.begin_gpu_timing(&mut encoder);

        // Simple direct render to surface (bypassing complex bloom for now)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Direct Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.debug_render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_bind_group(1, &self.black_hole_bind_group, &[]);
            render_pass.set_bind_group(2, &self.sky_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // End GPU timing and resolve queries
        self.profiler.end_gpu_timing(&mut encoder);
        self.profiler.resolve_gpu_timing(&mut encoder);

        // Finish staging belt before submitting commands
        self.staging_belt.finish();

        self.profiler.end_render_encode();

        self.queue.submit(std::iter::once(encoder.finish()));

        // Try to read GPU timing results from previous frames
        self.profiler.try_read_gpu_timing(&self.device, &self.queue);

        // Recall staging belt after submission to reuse buffers
        self.staging_belt.recall();

        self.profiler.end_frame();

        output.present();

        Ok(())
    }
}

struct App {
    state: Rc<RefCell<Option<State<'static>>>>,
    window: Option<Arc<Window>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            state: Rc::new(RefCell::new(None)),
            window: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.borrow().is_some() {
            return;
        }

        let mut window_attributes = Window::default_attributes();
        window_attributes = window_attributes.with_title("Black Hole Simulator");

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            let canvas = web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.get_element_by_id("wasm-canvas"))
                .and_then(|canvas| canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .expect("Get canvas");

            window_attributes = window_attributes.with_canvas(Some(canvas));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());

        #[cfg(target_arch = "wasm32")]
        {
            // For web, let the canvas size be controlled by CSS and JavaScript
            // The canvas will automatically scale to viewport size
        }

        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // For WASM, use shared reference to store state
                let window_for_wasm = window.clone();
                let state_ref = Rc::clone(&self.state);
                wasm_bindgen_futures::spawn_local(async move {
                    let new_state = State::new(window_for_wasm).await;
                    *state_ref.borrow_mut() = Some(new_state);
                    log::info!("Successfully created and stored WASM state");
                    // Hide loading screen now that renderer is ready
                    js_hide_loading_screen();
                });
            } else {
                *self.state.borrow_mut() = Some(pollster::block_on(State::new(window)));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(window) = &self.window {
            if window.id() == id {
                // Process input first
                if let Some(state) = self.state.borrow_mut().as_mut() {
                    if state.input(&event) {
                        return;
                    }
                }

                match event {
                    WindowEvent::CloseRequested => event_loop.exit(),
                    WindowEvent::Resized(physical_size) => {
                        if let Some(state) = self.state.borrow_mut().as_mut() {
                            state.resize(physical_size);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Some(state) = self.state.borrow_mut().as_mut() {
                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    event_loop.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        } else {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // State not ready yet
                                log::debug!("WASM state not ready for rendering yet");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

// Global state for WASM slider controls
#[cfg(target_arch = "wasm32")]
static mut DEBUG_PARAMS: Option<std::sync::Arc<std::sync::Mutex<DebugParams>>> = None;

#[cfg(target_arch = "wasm32")]
struct DebugParams {
    fov: f32,
    mass: f32,
    spin: f32,
    ray_steps: f32,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_fov(value: f32) {
    unsafe {
        if let Some(params) = &DEBUG_PARAMS {
            if let Ok(mut params) = params.lock() {
                params.fov = value.clamp(10.0, 120.0);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_mass(value: f32) {
    unsafe {
        if let Some(params) = &DEBUG_PARAMS {
            if let Ok(mut params) = params.lock() {
                params.mass = value.clamp(0.1, 5.0);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_spin(value: f32) {
    unsafe {
        if let Some(params) = &DEBUG_PARAMS {
            if let Ok(mut params) = params.lock() {
                params.spin = value.clamp(-1.0, 1.0);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_ray_steps(value: f32) {
    unsafe {
        if let Some(params) = &DEBUG_PARAMS {
            if let Ok(mut params) = params.lock() {
                params.ray_steps = value.clamp(50.0, 1000.0);
            }
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

            // Initialize global debug parameters for WASM
            unsafe {
                DEBUG_PARAMS = Some(std::sync::Arc::new(std::sync::Mutex::new(DebugParams {
                    fov: 80.0,
                    mass: 1.0,
                    spin: 1.0,
                    ray_steps: 250.0,
                })));
            }
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            env_logger::init();
        }
    }

    println!("{}", simulation::get_placeholder_string());

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(
        &"🕳️ BLACK HOLE SIMULATOR LOADED! Press ? to toggle help overlay.".into(),
    );

    #[cfg(not(target_arch = "wasm32"))]
    println!("🕳️ BLACK HOLE SIMULATOR LOADED! Press ? for help.");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
