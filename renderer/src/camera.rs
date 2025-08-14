use cgmath::*;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[cfg(target_arch = "wasm32")]
use web_sys;

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

    pub fn update_aspect_ratio(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub background_mode: f32,
    pub camera_forward: [f32; 3],
    pub fovy: f32,
    pub camera_right: [f32; 3],
    pub _padding3: f32,
    pub camera_up: [f32; 3],
    pub _padding4: f32,
    pub show_stars: f32,  // bool as f32 (1.0 or 0.0)
    pub show_grid: f32,   // bool as f32 (1.0 or 0.0)
    pub show_help: f32,   // bool as f32 (1.0 or 0.0)
    pub aspect_ratio: f32,
    pub render_width: f32,
    pub render_height: f32,
    pub _padding5: [f32; 2],  // Maintain 16-byte alignment
}

impl CameraUniform {
    pub fn new() -> Self {
        // Compile-time size check to ensure proper GPU buffer alignment
        const _: () = assert!(std::mem::size_of::<CameraUniform>() == 160);
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            camera_pos: [0.0; 3],
            background_mode: 0.0,
            camera_forward: [0.0, 0.0, -1.0],
            fovy: 80.0,
            camera_right: [1.0, 0.0, 0.0],
            _padding3: 0.0,
            camera_up: [0.0, 1.0, 0.0],
            _padding4: 0.0,
            show_stars: 1.0,
            show_grid: 0.0,
            show_help: 0.0,  // Start with help hidden
            aspect_ratio: 16.0 / 9.0,  // Default aspect ratio
            render_width: 1920.0,
            render_height: 1080.0,
            _padding5: [0.0; 2],
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera, show_stars: bool, show_grid: bool, show_help: bool) {
        self.update_view_proj_with_resolution(camera, show_stars, show_grid, show_help, 1920.0, 1080.0);
    }

    pub fn update_view_proj_with_resolution(&mut self, camera: &Camera, show_stars: bool, show_grid: bool, show_help: bool, width: f32, height: f32) {
        self.view_proj = camera.build_view_projection_matrix().into();
        
        // Update camera vectors for ray tracing
        self.camera_pos = camera.eye.into();
        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up).normalize();
        let up = right.cross(forward).normalize();
        
        self.camera_forward = forward.into();
        self.camera_right = right.into();
        self.camera_up = up.into();
        self.fovy = camera.fovy;
        
        // Update toggle states
        self.show_stars = if show_stars { 1.0 } else { 0.0 };
        self.show_grid = if show_grid { 1.0 } else { 0.0 };
        self.show_help = if show_help { 1.0 } else { 0.0 };
        
        // Update rendering resolution and aspect ratio
        self.aspect_ratio = width / height;
        self.render_width = width;
        self.render_height = height;
    }
}

pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    mousewheel_forward: f32,
    speed: f32,
    max_speed: f32,
    acceleration: f32,
    pub current_velocity: Vector3<f32>,
    sensitivity: f32,
    pub yaw: f32,
    pub pitch: f32,
    mouse_pressed: bool,
    last_mouse_pos: Option<Vector2<f64>>,
    touch_joystick_id: Option<u64>,
    touch_joystick_center: Option<Vector2<f64>>,
    touch_joystick_current: Option<Vector2<f64>>,
    touch_look_id: Option<u64>,
    touch_look_start_pos: Option<Vector2<f64>>,
    pub show_stars: bool,
    pub show_grid: bool,
    pub show_help: bool,
    pub show_fps: bool,
    pub show_profiling: bool,
    pub last_key: Option<KeyCode>,
    pub frame_count: u32,
    pub fps: f32,
    pub last_fps_time: f32,
    // Camera reset functionality
    initial_position: Point3<f32>,
    initial_yaw: f32,
    initial_pitch: f32,
    reset_requested: bool,
    // Triple tap detection for mobile
    last_tap_times: Vec<f64>,
    triple_tap_threshold: f64,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        let initial_yaw = 270.0; // 270° looks towards -Z (black hole at origin from camera at -Z)
        let initial_pitch = 0.0;
        
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            mousewheel_forward: 0.0,
            speed,
            max_speed: speed * 3.0,  // Maximum speed is 3x base speed
            acceleration: speed * 5.0,  // Acceleration rate
            current_velocity: Vector3::zero(),
            sensitivity: 0.1,
            yaw: initial_yaw,
            pitch: initial_pitch,
            mouse_pressed: false,
            last_mouse_pos: None,
            touch_joystick_id: None,
            touch_joystick_center: None,
            touch_joystick_current: None,
            touch_look_id: None,
            touch_look_start_pos: None,
            show_stars: true,
            show_grid: false,
            show_help: false,  // Start with help hidden (flash message shows instead)
            show_fps: false,   // Start with FPS counter hidden
            show_profiling: false, // Start with profiling hidden
            last_key: None,
            frame_count: 0,
            fps: 0.0,
            last_fps_time: 0.0,
            // Camera reset functionality - will be set properly when camera is initialized
            initial_position: Point3::new(0.0, 0.0, -40.0),
            initial_yaw,
            initial_pitch,
            reset_requested: false,
            // Triple tap detection for mobile
            last_tap_times: Vec::new(),
            triple_tap_threshold: 0.5, // 500ms between taps
        }
    }

    pub fn set_initial_camera_state(&mut self, position: Point3<f32>, yaw: f32, pitch: f32) {
        self.initial_position = position;
        self.initial_yaw = yaw;
        self.initial_pitch = pitch;
    }

    pub fn reset_camera(&mut self, camera: &mut Camera) {
        camera.eye = self.initial_position;
        self.yaw = self.initial_yaw;
        self.pitch = self.initial_pitch;
        self.current_velocity = Vector3::zero();
        
        // Update camera target to match the reset orientation
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        
        let forward = Vector3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize();
        
        camera.target = camera.eye + forward;
        camera.up = Vector3::unit_y();
    }

    pub fn check_and_clear_reset_request(&mut self) -> bool {
        if self.reset_requested {
            self.reset_requested = false;
            true
        } else {
            false
        }
    }

    pub fn process_mouse_button(&mut self, state: ElementState) {
        if state == ElementState::Pressed {
            self.mouse_pressed = true;
        } else {
            self.mouse_pressed = false;
            self.last_mouse_pos = None;
        }
    }

    pub fn process_cursor_move(&mut self, pos: winit::dpi::PhysicalPosition<f64>) {
        if !self.mouse_pressed {
            return;
        }
        let current_pos = vec2(pos.x, pos.y);
        if let Some(last_pos) = self.last_mouse_pos {
            let delta = current_pos - last_pos;
            // Adjust sensitivity for mouse look
            self.yaw += delta.x as f32 * self.sensitivity;
            self.pitch -= delta.y as f32 * self.sensitivity;
        }
        self.last_mouse_pos = Some(current_pos);
    }

    pub fn process_scroll(&mut self, delta: winit::event::MouseScrollDelta) {
        match delta {
            winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                // Each line is typically equivalent to 3 units of movement
                self.mousewheel_forward += y * 3.0;
            }
            winit::event::MouseScrollDelta::PixelDelta(pos) => {
                // Convert pixel delta to movement (scale down as pixels are small)
                self.mousewheel_forward += pos.y as f32 * 0.01;
            }
        }
    }

    fn check_triple_tap(&mut self, timestamp: f64) -> bool {
        // Add current tap time
        self.last_tap_times.push(timestamp);
        
        // Keep only recent taps (within threshold)
        let cutoff_time = timestamp - self.triple_tap_threshold;
        self.last_tap_times.retain(|&t| t > cutoff_time);
        
        // Check if we have 3 or more recent taps
        if self.last_tap_times.len() >= 3 {
            self.last_tap_times.clear(); // Clear to prevent multiple resets
            true
        } else {
            false
        }
    }

    pub fn process_touch(&mut self, touch: &winit::event::Touch, window_size: winit::dpi::PhysicalSize<u32>) {
        let pos = vec2(touch.location.x, touch.location.y);
        let half_width = window_size.width as f64 / 2.0;

        match touch.phase {
            winit::event::TouchPhase::Started => {
                // Check for triple tap 
                let timestamp = {
                    #[cfg(target_arch = "wasm32")]
                    {
                        web_sys::window().unwrap().performance().unwrap().now() / 1000.0
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs_f64()
                    }
                };
                
                if self.check_triple_tap(timestamp) {
                    self.reset_requested = true;
                    return; // Don't process as regular touch if it's a triple tap
                }
                
                if pos.x < half_width { // Left side: movement
                    if self.touch_joystick_id.is_none() {
                        self.touch_joystick_id = Some(touch.id);
                        self.touch_joystick_center = Some(pos);
                        self.touch_joystick_current = Some(pos);
                    }
                } else { // Right side: look
                    if self.touch_look_id.is_none() {
                        self.touch_look_id = Some(touch.id);
                        self.touch_look_start_pos = Some(pos);
                    }
                }
            }
            winit::event::TouchPhase::Moved => {
                if Some(touch.id) == self.touch_joystick_id {
                    self.touch_joystick_current = Some(pos);
                } else if Some(touch.id) == self.touch_look_id {
                    if let Some(start_pos) = self.touch_look_start_pos {
                        let delta = pos - start_pos;
                        self.yaw -= delta.x as f32 * self.sensitivity * 0.5; // Touch has different sensitivity - inverted
                        self.pitch += delta.y as f32 * self.sensitivity * 0.5; // Inverted
                    }
                    self.touch_look_start_pos = Some(pos);
                }
            }
            winit::event::TouchPhase::Ended | winit::event::TouchPhase::Cancelled => {
                if Some(touch.id) == self.touch_joystick_id {
                    self.touch_joystick_id = None;
                    self.touch_joystick_center = None;
                    self.touch_joystick_current = None;
                } else if Some(touch.id) == self.touch_look_id {
                    self.touch_look_id = None;
                    self.touch_look_start_pos = None;
                }
            }
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
                }
                true
            }
            KeyCode::KeyF => {
                // Toggle FPS counter
                if state == ElementState::Pressed {
                    self.show_fps = !self.show_fps;
                }
                true
            }
            KeyCode::KeyP => {
                // Toggle profiling info
                if state == ElementState::Pressed {
                    self.show_profiling = !self.show_profiling;
                }
                true
            }
            KeyCode::KeyR => {
                // Reset camera to initial position
                if state == ElementState::Pressed {
                    self.reset_requested = true;
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: std::time::Duration) {
        let dt = dt.as_secs_f32();
        
        // Check for reset request first
        if self.check_and_clear_reset_request() {
            self.reset_camera(camera);
            return; // Early return after reset
        }

        // Clamp pitch to prevent flipping
        self.pitch = self.pitch.clamp(-89.0, 89.0);

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

        // Note: Startup help flash message is now handled purely in JavaScript

        // Handle touch input for movement
        let mut touch_fwd = 0.0;
        let mut touch_strafe = 0.0;
        if let (Some(center), Some(current)) = (self.touch_joystick_center, self.touch_joystick_current) {
            let delta = current - center;
            let joystick_radius = 100.0; // Virtual joystick size in pixels
            touch_fwd = (-delta.y / joystick_radius).clamp(-1.0, 1.0) as f32;
            touch_strafe = (delta.x / joystick_radius).clamp(-1.0, 1.0) as f32;
        }

        // Calculate desired movement input with acceleration
        // Negate forward movement so W moves toward black hole at origin
        let move_forward = self.amount_forward + touch_fwd + (self.mousewheel_forward * 0.1);
        let move_strafe = (self.amount_right - self.amount_left) + touch_strafe;
        let move_vertical = self.amount_up - self.amount_down;
        
        // Calculate target velocity based on input
        let target_velocity = Vector3::new(
            move_strafe * self.max_speed,
            move_vertical * self.max_speed,
            (self.amount_backward - move_forward) * self.max_speed
        );
        
        // Apply acceleration towards target velocity
        let velocity_diff = target_velocity - self.current_velocity;
        let velocity_diff_magnitude = velocity_diff.magnitude();
        
        if velocity_diff_magnitude > 0.01 {
            let acceleration_step = velocity_diff.normalize() * self.acceleration * dt;
            
            // Limit acceleration step to not overshoot target
            if acceleration_step.magnitude() > velocity_diff_magnitude {
                self.current_velocity = target_velocity;
            } else {
                self.current_velocity += acceleration_step;
            }
        } else {
            self.current_velocity = target_velocity;
        }
        
        // Apply velocity with dampening when no input
        if target_velocity.magnitude() < 0.01 {
            self.current_velocity *= 0.95; // Gradual slowdown when no input
        }
        
        // Move camera using current velocity in world space
        camera.eye += forward * self.current_velocity.z * dt;
        camera.eye += right * self.current_velocity.x * dt;
        camera.eye += up * self.current_velocity.y * dt;
        
        // Decay mousewheel input
        self.mousewheel_forward *= 0.9;

        // Update target to be in front of camera
        camera.target = camera.eye + forward;
        camera.up = up;
    }

}
