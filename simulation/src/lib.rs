//! The simulation crate handles the heavy lifting of general relativity, 
//! solving the geodesic equations to determine how light travels.

/// Represents a basic volumetric mass in 3D space
#[derive(Debug, Clone)]
pub struct VolumetricMass {
    /// Position of the mass center in 3D space (x, y, z)
    pub position: [f32; 3],
    /// Mass in solar masses
    pub mass: f32,
    /// Visual radius for rendering (not the Schwarzschild radius)
    pub radius: f32,
}

impl VolumetricMass {
    /// Create a new volumetric mass
    pub fn new(position: [f32; 3], mass: f32, radius: f32) -> Self {
        Self {
            position,
            mass,
            radius,
        }
    }
    
    /// Calculate the Schwarzschild radius (event horizon)
    /// r_s = 2GM/c² (in geometric units where G=c=1, r_s = 2M)
    pub fn schwarzschild_radius(&self) -> f32 {
        2.0 * self.mass
    }
    
    /// Get the gravitational field strength at a given distance
    /// This is a simplified Newtonian approximation for now
    pub fn field_strength_at_distance(&self, distance: f32) -> f32 {
        if distance <= 0.0 {
            f32::INFINITY
        } else {
            self.mass / (distance * distance)
        }
    }
    
    /// Calculate distance from mass center to a point
    pub fn distance_to_point(&self, point: [f32; 3]) -> f32 {
        let dx = point[0] - self.position[0];
        let dy = point[1] - self.position[1];
        let dz = point[2] - self.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Create a default black hole mass for testing
pub fn create_default_black_hole() -> VolumetricMass {
    VolumetricMass::new(
        [0.0, 0.0, 0.0],  // Centered at origin
        1.0,              // 1 solar mass
        0.5,              // Visual radius
    )
}

/// Represents a point in 4D spacetime with position and momentum
#[derive(Debug, Clone, Copy)]
pub struct Geodesic {
    /// Position in spacetime (t, r, theta, phi)
    pub position: [f32; 4],
    /// Four-momentum (pt, pr, ptheta, pphi)  
    pub momentum: [f32; 4],
}

impl Geodesic {
    /// Create a new geodesic state
    pub fn new(position: [f32; 4], momentum: [f32; 4]) -> Self {
        Self { position, momentum }
    }
    
    /// Get the radial distance from origin
    pub fn radius(&self) -> f32 {
        self.position[1]
    }
    
    /// Check if the photon has crossed the event horizon
    pub fn is_inside_event_horizon(&self, mass: f32) -> bool {
        self.radius() <= 2.0 * mass
    }
}

/// Ray tracing data structure for a light ray
#[derive(Debug, Clone)]
pub struct LightRay {
    /// Current geodesic state
    pub geodesic: Geodesic,
    /// Black hole mass
    pub mass: f32,
    /// Integration step size
    pub step_size: f32,
    /// Maximum number of integration steps
    pub max_steps: u32,
    /// Current step count
    pub step_count: u32,
}

impl LightRay {
    /// Create a new light ray from camera position and direction
    pub fn new(camera_pos: [f32; 3], ray_dir: [f32; 3], mass: f32) -> Self {
        // Convert to spherical coordinates
        let r = (camera_pos[0] * camera_pos[0] + camera_pos[1] * camera_pos[1] + camera_pos[2] * camera_pos[2]).sqrt();
        let theta = (camera_pos[2] / r).acos();
        let phi = camera_pos[1].atan2(camera_pos[0]);
        
        // Initial position in spacetime (t, r, theta, phi)
        let position = [0.0, r, theta, phi];
        
        // Convert ray direction to four-momentum for null geodesic
        // This is a simplified initialization - proper implementation needs more care
        let momentum = [1.0, ray_dir[0], ray_dir[1], ray_dir[2]];
        
        Self {
            geodesic: Geodesic::new(position, momentum),
            mass,
            step_size: 0.01,
            max_steps: 10000,
            step_count: 0,
        }
    }
    
    /// Take one integration step along the geodesic
    pub fn step(&mut self) -> bool {
        if self.step_count >= self.max_steps {
            return false; // Max steps reached
        }
        
        if self.geodesic.is_inside_event_horizon(self.mass) {
            return false; // Fell into black hole
        }
        
        // Simplified geodesic integration using Runge-Kutta 4th order
        let k1 = self.compute_derivatives(self.geodesic);
        
        let mut temp_state = self.geodesic;
        self.add_derivatives(&mut temp_state, &k1, self.step_size * 0.5);
        let k2 = self.compute_derivatives(temp_state);
        
        temp_state = self.geodesic;
        self.add_derivatives(&mut temp_state, &k2, self.step_size * 0.5);
        let k3 = self.compute_derivatives(temp_state);
        
        temp_state = self.geodesic;
        self.add_derivatives(&mut temp_state, &k3, self.step_size);
        let k4 = self.compute_derivatives(temp_state);
        
        // Combine derivatives for RK4
        let mut final_deriv_pos = [0.0; 4];
        let mut final_deriv_mom = [0.0; 4];
        for i in 0..4 {
            final_deriv_pos[i] = (k1.position[i] + 2.0 * k2.position[i] + 2.0 * k3.position[i] + k4.position[i]) / 6.0;
            final_deriv_mom[i] = (k1.momentum[i] + 2.0 * k2.momentum[i] + 2.0 * k3.momentum[i] + k4.momentum[i]) / 6.0;
        }
        
        // Apply the final derivatives
        for i in 0..4 {
            self.geodesic.position[i] += final_deriv_pos[i] * self.step_size;
            self.geodesic.momentum[i] += final_deriv_mom[i] * self.step_size;
        }
        self.step_count += 1;
        
        true
    }
    
    /// Compute derivatives for the geodesic equation
    fn compute_derivatives(&self, state: Geodesic) -> Geodesic {
        let r = state.position[1];
        let theta = state.position[2];
        
        // Simplified derivatives for Schwarzschild metric in spherical coordinates
        // d/dλ (position) = momentum
        let pos_deriv = state.momentum;
        
        // d/dλ (momentum) = -Γ^μ_αβ p^α p^β (Christoffel symbols)
        // This is a simplified version - full implementation needs all Christoffel symbols
        let mut mom_deriv = [0.0; 4];
        
        // Some key Christoffel symbols for Schwarzschild metric
        if r > 2.0 * self.mass {
            let rs_over_r = 2.0 * self.mass / r;
            
            // Simplified radial equation of motion
            mom_deriv[1] = -rs_over_r / (2.0 * r * r) * state.momentum[0] * state.momentum[0]
                         + rs_over_r * (1.0 - rs_over_r) / (2.0 * r * r) * state.momentum[1] * state.momentum[1]
                         + r * (1.0 - rs_over_r) * (state.momentum[2] * state.momentum[2] + theta.sin().powi(2) * state.momentum[3] * state.momentum[3]);
        }
        
        Geodesic::new(pos_deriv, mom_deriv)
    }
    
    /// Add derivatives to a geodesic state
    fn add_derivatives(&self, state: &mut Geodesic, deriv: &Geodesic, step: f32) {
        for i in 0..4 {
            state.position[i] += deriv.position[i] * step;
            state.momentum[i] += deriv.momentum[i] * step;
        }
    }
    
    /// Check if ray has escaped to infinity
    pub fn has_escaped(&self) -> bool {
        self.geodesic.radius() > 100.0 * self.mass // Far enough to consider "escaped"
    }
}

/// Basic Schwarzschild metric calculations
pub mod schwarzschild {
    /// Calculate the metric coefficient g_tt (time-time component)
    /// g_tt = -(1 - 2M/r) in geometric units
    pub fn g_tt(mass: f32, r: f32) -> f32 {
        -(1.0 - (2.0 * mass) / r)
    }
    
    /// Calculate the metric coefficient g_rr (radial-radial component)
    /// g_rr = 1/(1 - 2M/r) in geometric units
    pub fn g_rr(mass: f32, r: f32) -> f32 {
        1.0 / (1.0 - (2.0 * mass) / r)
    }
    
    /// Calculate the metric coefficient g_theta_theta (angular component)
    /// g_θθ = r²
    pub fn g_theta_theta(r: f32) -> f32 {
        r * r
    }
    
    /// Calculate the metric coefficient g_phi_phi (azimuthal component)
    /// g_φφ = r² sin²θ
    pub fn g_phi_phi(r: f32, theta: f32) -> f32 {
        r * r * theta.sin().powi(2)
    }
    
    /// Check if a position is inside the event horizon
    pub fn is_inside_event_horizon(mass: f32, r: f32) -> bool {
        r <= 2.0 * mass
    }
    
    /// Calculate the proper time dilation factor
    /// For a stationary observer at radius r
    pub fn time_dilation_factor(mass: f32, r: f32) -> f32 {
        if is_inside_event_horizon(mass, r) {
            0.0 // Time stops at the event horizon
        } else {
            (1.0 - (2.0 * mass) / r).sqrt()
        }
    }
}

/// This is a placeholder.
///
/// In the future, this crate will define the spacetime metric in Kerr-Schild coordinates,
/// calculate the Christoffel symbols, and provide a function to integrate the geodesic
/// equation.
pub fn get_placeholder_string() -> &'static str {
    "Hello from the simulation crate!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(get_placeholder_string(), "Hello from the simulation crate!");
    }

    #[test]
    fn test_volumetric_mass_creation() {
        let mass = VolumetricMass::new([1.0, 2.0, 3.0], 2.0, 0.8);
        assert_eq!(mass.position, [1.0, 2.0, 3.0]);
        assert_eq!(mass.mass, 2.0);
        assert_eq!(mass.radius, 0.8);
    }

    #[test]
    fn test_schwarzschild_radius() {
        let mass = VolumetricMass::new([0.0, 0.0, 0.0], 1.0, 0.5);
        assert_eq!(mass.schwarzschild_radius(), 2.0);
    }

    #[test]
    fn test_distance_calculation() {
        let mass = VolumetricMass::new([0.0, 0.0, 0.0], 1.0, 0.5);
        let distance = mass.distance_to_point([3.0, 4.0, 0.0]);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_default_black_hole() {
        let bh = create_default_black_hole();
        assert_eq!(bh.position, [0.0, 0.0, 0.0]);
        assert_eq!(bh.mass, 1.0);
        assert_eq!(bh.radius, 0.5);
        assert_eq!(bh.schwarzschild_radius(), 2.0);
    }

    #[test]
    fn test_schwarzschild_metric() {
        use super::schwarzschild::*;
        
        let mass = 1.0;
        let r = 4.0; // Far from event horizon
        
        // Test metric components
        assert!((g_tt(mass, r) - (-0.5)).abs() < 1e-6);
        assert!((g_rr(mass, r) - 2.0).abs() < 1e-6);
        assert_eq!(g_theta_theta(r), 16.0);
        assert!((g_phi_phi(r, std::f32::consts::PI / 2.0) - 16.0).abs() < 1e-6);
        
        // Test event horizon
        assert!(is_inside_event_horizon(mass, 1.0));
        assert!(!is_inside_event_horizon(mass, 3.0));
        
        // Test time dilation
        assert!(time_dilation_factor(mass, 4.0) > 0.0);
        assert_eq!(time_dilation_factor(mass, 1.0), 0.0); // At event horizon
    }

    #[test]
    fn test_geodesic_creation() {
        let pos = [0.0, 5.0, std::f32::consts::PI / 2.0, 0.0];
        let mom = [1.0, 0.0, 0.0, 0.1];
        let geodesic = Geodesic::new(pos, mom);
        
        assert_eq!(geodesic.radius(), 5.0);
        assert!(!geodesic.is_inside_event_horizon(1.0));
        assert!(geodesic.is_inside_event_horizon(3.0));
    }

    #[test]
    fn test_light_ray_creation() {
        let camera_pos = [0.0, 0.0, 5.0];
        let ray_dir = [0.0, 0.0, -1.0];
        let mass = 1.0;
        
        let ray = LightRay::new(camera_pos, ray_dir, mass);
        assert_eq!(ray.mass, mass);
        assert_eq!(ray.step_count, 0);
        assert!(ray.geodesic.radius() > 0.0);
    }

    #[test]
    fn test_light_ray_escape() {
        let camera_pos = [0.0, 0.0, 200.0]; // Far enough to trigger escape condition
        let ray_dir = [0.0, 0.0, 1.0]; // Away from black hole
        let mass = 1.0;
        
        let ray = LightRay::new(camera_pos, ray_dir, mass);
        assert!(ray.has_escaped()); // Should be > 100 * mass = 100
    }
}
