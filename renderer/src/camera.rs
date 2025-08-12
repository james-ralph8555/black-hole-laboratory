use cgmath::*;
use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new<
        V: Into<Point3<f32>>,
        Y: Into<Point3<f32>>,
        U: Into<Vector3<f32>>,
    >(
        eye: V,
        target: Y,
        up: U,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            eye: eye.into(),
            target: target.into(),
            up: up.into(),
            aspect,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding1: f32,
    pub camera_forward: [f32; 3],
    pub _padding2: f32,
    pub camera_right: [f32; 3],
    pub _padding3: f32,
    pub camera_up: [f32; 3],
    pub _padding4: f32,
    pub show_stars: f32,  // bool as f32 (1.0 or 0.0)
    pub show_grid: f32,   // bool as f32 (1.0 or 0.0)
    pub show_help: f32,   // bool as f32 (1.0 or 0.0)
    pub _padding5: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        // Compile-time size check to ensure proper GPU buffer alignment
        const _: () = assert!(std::mem::size_of::<CameraUniform>() == 144);
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            camera_pos: [0.0; 3],
            _padding1: 0.0,
            camera_forward: [0.0, 0.0, -1.0],
            _padding2: 0.0,
            camera_right: [1.0, 0.0, 0.0],
            _padding3: 0.0,
            camera_up: [0.0, 1.0, 0.0],
            _padding4: 0.0,
            show_stars: 1.0,
            show_grid: 0.0,
            show_help: 1.0,
            _padding5: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, show_stars: bool, show_grid: bool, show_help: bool) {
        self.view_proj = camera.build_view_projection_matrix().into();
        
        // Update camera vectors for ray tracing
        self.camera_pos = camera.eye.into();
        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward).normalize();
        
        self.camera_forward = forward.into();
        self.camera_right = right.into();
        self.camera_up = up.into();
        
        // Update toggle states
        self.show_stars = if show_stars { 1.0 } else { 0.0 };
        self.show_grid = if show_grid { 1.0 } else { 0.0 };
        self.show_help = if show_help { 1.0 } else { 0.0 };
    }
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    speed: f32,
    sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub show_stars: bool,
    pub show_grid: bool,
    pub show_help: bool,
    pub help_startup_timer: f32,
    pub last_key: Option<KeyCode>,
    pub frame_count: u32,
    pub fps: f32,
    pub last_fps_time: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            speed,
            sensitivity: 0.1,
            yaw: 270.0, // 270° looks towards -Z (black hole at origin from camera at -Z)
            pitch: 0.0,
            show_stars: true,
            show_grid: false,
            show_help: true,  // Start with help visible
            help_startup_timer: 5.0,  // Show help for 5 seconds on startup
            last_key: None,
            frame_count: 0,
            fps: 0.0,
            last_fps_time: 0.0,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        
        // Track last pressed key for debug display
        if state == ElementState::Pressed {
            self.last_key = Some(key);
        }
        
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.amount_forward = amount;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.amount_backward = amount;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.amount_left = amount;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.amount_right = amount;
                true
            }
            KeyCode::Space => {
                self.amount_up = amount;
                true
            }
            KeyCode::ShiftLeft => {
                self.amount_down = amount;
                true
            }
            KeyCode::KeyQ => {
                // Turn left
                self.yaw += 2.0;
                true
            }
            KeyCode::KeyE => {
                // Turn right
                self.yaw -= 2.0;
                true
            }
            KeyCode::KeyB => {
                // Toggle background (stars vs solid)
                if state == ElementState::Pressed {
                    self.show_stars = !self.show_stars;
                }
                true
            }
            KeyCode::KeyG => {
                // Toggle grid lines
                if state == ElementState::Pressed {
                    self.show_grid = !self.show_grid;
                }
                true
            }
            KeyCode::Slash => {
                // Toggle help with ? key
                if state == ElementState::Pressed {
                    self.show_help = !self.show_help;
                    // Reset startup timer when manually toggled
                    self.help_startup_timer = 0.0;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();

        // Update camera direction based on yaw and pitch
        // Standard FPS camera: yaw=0° looks down +X axis, yaw=90° looks down -Z axis
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        
        let forward = Vector3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();

        let right = forward.cross(Vector3::unit_y()).normalize();
        let up = right.cross(forward).normalize();

        // Update FPS tracking
        self.frame_count += 1;
        self.last_fps_time += dt;
        if self.last_fps_time >= 1.0 {
            self.fps = self.frame_count as f32 / self.last_fps_time;
            self.frame_count = 0;
            self.last_fps_time = 0.0;
        }

        // Handle startup help timer
        if self.help_startup_timer > 0.0 {
            self.help_startup_timer -= dt;
            if self.help_startup_timer <= 0.0 {
                self.show_help = false;  // Auto-hide help after timeout
            }
        }

        // Move camera position using the calculated direction vectors
        // Negate forward movement so W moves toward black hole at origin
        camera.eye += forward * (self.amount_backward - self.amount_forward) * self.speed * dt;
        camera.eye += right * (self.amount_right - self.amount_left) * self.speed * dt;
        camera.eye += up * (self.amount_up - self.amount_down) * self.speed * dt;

        // Update target to be in front of camera
        camera.target = camera.eye + forward;
        camera.up = up;
    }

    pub fn get_help_text(&self, camera: &Camera) -> String {
        format!(
r#"🕳️ BLACK HOLE SIMULATOR - HELP

MOVEMENT CONTROLS:
  W/A/S/D     - Move forward/left/backward/right
  Space/Shift - Move up/down
  Q/E         - Turn left/right

VISUAL TOGGLES:
  B - Toggle background (stars/gradient)
  G - Toggle lat/long grid lines
  ? - Toggle this help

DEBUG INFO:
  Position:    ({:.2}, {:.2}, {:.2})
  Orientation: Yaw {:.1}°, Pitch {:.1}°
  Last Key:    {:?}
  FPS:         {:.1}
  Renderer:    Ray Tracing @ 1920x1080

PHYSICS:
  Ray tracing through curved spacetime using
  simplified Schwarzschild metric. Grid lines
  show gravitational lensing distortion.

Press ? to hide this help."#,
            camera.eye.x, camera.eye.y, camera.eye.z,
            self.yaw, self.pitch,
            self.last_key.map(|k| format!("{:?}", k)).unwrap_or_else(|| "None".to_string()),
            self.fps
        )
    }
}