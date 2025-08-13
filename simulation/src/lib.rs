//! The simulation crate handles the heavy lifting of general relativity, 
//! solving the geodesic equations to determine how light travels.

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

/// Adaptive RK45 integrator for geodesic equations
#[derive(Debug, Clone)]
pub struct AdaptiveRK45 {
    /// Absolute tolerance for error control
    pub abs_tolerance: f32,
    /// Relative tolerance for error control
    pub rel_tolerance: f32,
    /// Minimum step size
    pub min_step: f32,
    /// Maximum step size
    pub max_step: f32,
    /// Safety factor for step size adjustment
    pub safety_factor: f32,
}

impl Default for AdaptiveRK45 {
    fn default() -> Self {
        Self {
            abs_tolerance: 1e-6,
            rel_tolerance: 1e-6,
            min_step: 1e-8,
            max_step: 1.0,
            safety_factor: 0.9,
        }
    }
}

impl AdaptiveRK45 {
    /// Take one adaptive step for geodesic integration
    /// Returns (new_state, actual_step_size, suggested_next_step)
    pub fn step(
        &self,
        state: Geodesic,
        step_size: f32,
        derivatives_fn: impl Fn(Geodesic) -> Geodesic,
    ) -> (Geodesic, f32, f32) {
        let h = step_size.clamp(self.min_step, self.max_step);
        
        // RK45 Butcher tableau coefficients
        let a = [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [1.0/4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [3.0/32.0, 9.0/32.0, 0.0, 0.0, 0.0, 0.0],
            [1932.0/2197.0, -7200.0/2197.0, 7296.0/2197.0, 0.0, 0.0, 0.0],
            [439.0/216.0, -8.0, 3680.0/513.0, -845.0/4104.0, 0.0, 0.0],
            [-8.0/27.0, 2.0, -3544.0/2565.0, 1859.0/4104.0, -11.0/40.0, 0.0],
        ];
        
        let b4 = [25.0/216.0, 0.0, 1408.0/2565.0, 2197.0/4104.0, -1.0/5.0, 0.0];
        let b5 = [16.0/135.0, 0.0, 6656.0/12825.0, 28561.0/56430.0, -9.0/50.0, 2.0/55.0];
        
        // Calculate k values
        let k1 = derivatives_fn(state);
        
        let mut temp_state = state;
        self.add_k_to_state(&mut temp_state, &k1, h * a[1][0]);
        let k2 = derivatives_fn(temp_state);
        
        temp_state = state;
        self.add_k_to_state(&mut temp_state, &k1, h * a[2][0]);
        self.add_k_to_state(&mut temp_state, &k2, h * a[2][1]);
        let k3 = derivatives_fn(temp_state);
        
        temp_state = state;
        self.add_k_to_state(&mut temp_state, &k1, h * a[3][0]);
        self.add_k_to_state(&mut temp_state, &k2, h * a[3][1]);
        self.add_k_to_state(&mut temp_state, &k3, h * a[3][2]);
        let k4 = derivatives_fn(temp_state);
        
        temp_state = state;
        self.add_k_to_state(&mut temp_state, &k1, h * a[4][0]);
        self.add_k_to_state(&mut temp_state, &k2, h * a[4][1]);
        self.add_k_to_state(&mut temp_state, &k3, h * a[4][2]);
        self.add_k_to_state(&mut temp_state, &k4, h * a[4][3]);
        let k5 = derivatives_fn(temp_state);
        
        temp_state = state;
        self.add_k_to_state(&mut temp_state, &k1, h * a[5][0]);
        self.add_k_to_state(&mut temp_state, &k2, h * a[5][1]);
        self.add_k_to_state(&mut temp_state, &k3, h * a[5][2]);
        self.add_k_to_state(&mut temp_state, &k4, h * a[5][3]);
        self.add_k_to_state(&mut temp_state, &k5, h * a[5][4]);
        let k6 = derivatives_fn(temp_state);
        
        // 4th order solution
        let mut y4 = state;
        self.add_k_to_state(&mut y4, &k1, h * b4[0]);
        self.add_k_to_state(&mut y4, &k3, h * b4[2]);
        self.add_k_to_state(&mut y4, &k4, h * b4[3]);
        self.add_k_to_state(&mut y4, &k5, h * b4[4]);
        
        // 5th order solution
        let mut y5 = state;
        self.add_k_to_state(&mut y5, &k1, h * b5[0]);
        self.add_k_to_state(&mut y5, &k3, h * b5[2]);
        self.add_k_to_state(&mut y5, &k4, h * b5[3]);
        self.add_k_to_state(&mut y5, &k5, h * b5[4]);
        self.add_k_to_state(&mut y5, &k6, h * b5[5]);
        
        // Error estimation
        let error = self.estimate_error(&y4, &y5);
        let tolerance = self.abs_tolerance + self.rel_tolerance * self.state_norm(&state);
        
        // Step size adjustment
        let ratio = tolerance / error.max(1e-14);
        let new_step = h * self.safety_factor * ratio.powf(0.2);
        let suggested_step = new_step.clamp(self.min_step, self.max_step);
        
        // Accept or reject step
        if error <= tolerance {
            (y5, h, suggested_step)
        } else {
            // Retry with smaller step
            self.step(state, suggested_step, derivatives_fn)
        }
    }
    
    /// Add k vector to geodesic state
    fn add_k_to_state(&self, state: &mut Geodesic, k: &Geodesic, factor: f32) {
        for i in 0..4 {
            state.position[i] += factor * k.position[i];
            state.momentum[i] += factor * k.momentum[i];
        }
    }
    
    /// Estimate error between 4th and 5th order solutions
    fn estimate_error(&self, y4: &Geodesic, y5: &Geodesic) -> f32 {
        let mut max_error: f32 = 0.0;
        
        for i in 0..4 {
            let pos_error = (y5.position[i] - y4.position[i]).abs();
            let mom_error = (y5.momentum[i] - y4.momentum[i]).abs();
            max_error = max_error.max(pos_error).max(mom_error);
        }
        
        max_error
    }
    
    /// Calculate norm of geodesic state for error scaling
    fn state_norm(&self, state: &Geodesic) -> f32 {
        let mut norm = 0.0;
        
        for i in 0..4 {
            norm += state.position[i] * state.position[i];
            norm += state.momentum[i] * state.momentum[i];
        }
        
        norm.sqrt()
    }
}

/// Ray tracing data structure for a light ray with Kerr geometry
#[derive(Debug, Clone)]
pub struct KerrLightRay {
    /// Current geodesic state
    pub geodesic: Geodesic,
    /// Conserved quantities
    pub conserved: ConservedQuantities,
    /// Black hole parameters
    pub black_hole: KerrBlackHole,
    /// Adaptive integrator
    pub integrator: AdaptiveRK45,
    /// Current step size
    pub step_size: f32,
    /// Maximum number of integration steps
    pub max_steps: u32,
    /// Current step count
    pub step_count: u32,
}

impl KerrLightRay {
    /// Create a new Kerr light ray from camera position and direction
    pub fn new(camera_pos: [f32; 3], ray_dir: [f32; 3], black_hole: KerrBlackHole) -> Self {
        // Convert to spherical coordinates
        let r = (camera_pos[0] * camera_pos[0] + camera_pos[1] * camera_pos[1] + camera_pos[2] * camera_pos[2]).sqrt();
        let theta = (camera_pos[2] / r).acos();
        let phi = camera_pos[1].atan2(camera_pos[0]);
        
        // Initial position in spacetime (t, r, theta, phi)
        let position = [0.0, r, theta, phi];
        
        // Convert ray direction to four-momentum for null geodesic
        // This is a simplified initialization - needs proper implementation
        let momentum = [1.0, ray_dir[0], ray_dir[1], ray_dir[2]];
        
        let geodesic = Geodesic::new(position, momentum);
        let conserved = ConservedQuantities::from_initial_conditions(position, momentum, black_hole.mass, black_hole.spin);
        
        Self {
            geodesic,
            conserved,
            black_hole,
            integrator: AdaptiveRK45::default(),
            step_size: 0.1,
            max_steps: 10000,
            step_count: 0,
        }
    }
    
    /// Take one integration step along the geodesic using adaptive RK45
    pub fn step(&mut self) -> bool {
        if self.step_count >= self.max_steps {
            return false;
        }
        
        if kerr_schild::is_inside_horizon(self.geodesic.radius(), &self.black_hole) {
            return false;
        }
        
        let (new_state, _actual_step, next_step) = self.integrator.step(
            self.geodesic,
            self.step_size,
            |state| self.compute_kerr_derivatives(state),
        );
        
        self.geodesic = new_state;
        self.step_size = next_step;
        self.step_count += 1;
        
        true
    }
    
    /// Compute derivatives for Kerr geodesic equation using conserved quantities
    /// This implements the first-order ODE system using energy, angular momentum, and Carter's constant
    fn compute_kerr_derivatives(&self, state: Geodesic) -> Geodesic {
        let r = state.position[1];
        let theta = state.position[2];
        let _phi = state.position[3];
        
        // Conserved quantities
        let e = self.conserved.energy;
        let lz = self.conserved.angular_momentum_z;
        let q = self.conserved.carter_constant;
        
        let mass = self.black_hole.mass;
        let spin = self.black_hole.spin;
        
        // Kerr metric functions
        let sigma = kerr_schild::sigma(r, theta, spin);
        let delta = kerr_schild::delta(r, mass, spin);
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        
        // First-order ODE system for null geodesics
        // Using conserved quantities to eliminate second derivatives
        
        // dt/dλ = (1/Δ) * [(r² + a²)E - aLz] * (r² + a²)/Σ + aE * sin²θ
        let dt_dlambda = (1.0 / delta) * ((r * r + spin * spin) * e - spin * lz) * (r * r + spin * spin) / sigma 
                        + spin * e * sin_theta.powi(2);
        
        // dr/dλ determined by radial equation of motion
        // R(r) = [E(r² + a²) - aLz]² - Δ[μr² + (Lz - aE)² + Q]
        let term1 = e * (r * r + spin * spin) - spin * lz;
        let term2 = lz - spin * e;
        let radial_potential = term1 * term1 - delta * (term2 * term2 + q);
        
        let dr_dlambda = if radial_potential >= 0.0 {
            radial_potential.sqrt() / sigma
        } else {
            0.0 // Ray is turning around
        };
        
        // dθ/dλ determined by polar equation of motion
        // Θ(θ) = Q - cos²θ[a²(μ - E²) + L²z/sin²θ]
        let theta_potential = q - cos_theta.powi(2) * (spin * spin * (0.0 - e * e) + lz * lz / sin_theta.powi(2));
        
        let dtheta_dlambda = if theta_potential >= 0.0 {
            theta_potential.sqrt() / sigma
        } else {
            0.0 // Ray is turning around in theta
        };
        
        // dφ/dλ = (1/Δ) * [-aE + Lz/sin²θ] + aE/Σ
        let dphi_dlambda = (1.0 / delta) * (-spin * e + lz / sin_theta.powi(2)) + spin * e / sigma;
        
        // Position derivatives (dx^μ/dλ)
        let pos_deriv = [dt_dlambda, dr_dlambda, dtheta_dlambda, dphi_dlambda];
        
        // For null geodesics, momentum is proportional to position derivatives
        // p_μ = g_μν dx^ν/dλ, but for our purposes we can use the same derivatives
        let mom_deriv = pos_deriv;
        
        Geodesic::new(pos_deriv, mom_deriv)
    }
    
    /// Check if ray has escaped to infinity
    pub fn has_escaped(&self) -> bool {
        self.geodesic.radius() > 100.0 * self.black_hole.mass
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

/// Conserved quantities for photon geodesics in Kerr spacetime
#[derive(Debug, Clone, Copy)]
pub struct ConservedQuantities {
    /// Energy (E) - conserved quantity related to time translation symmetry
    pub energy: f32,
    /// Axial angular momentum (Lz) - conserved quantity related to axial rotation symmetry
    pub angular_momentum_z: f32,
    /// Carter's constant (Q) - fourth conserved quantity in Kerr spacetime
    pub carter_constant: f32,
}

impl ConservedQuantities {
    /// Create new conserved quantities from initial conditions
    pub fn from_initial_conditions(
        position: [f32; 4], 
        momentum: [f32; 4], 
        mass: f32, 
        spin: f32
    ) -> Self {
        let r = position[1];
        let theta = position[2];
        let pt = momentum[0];
        let _pr = momentum[1];
        let ptheta = momentum[2];
        let pphi = momentum[3];
        
        // Calculate metric components for Kerr-Schild coordinates
        let _delta = r * r - 2.0 * mass * r + spin * spin;
        let _sigma = r * r + spin * spin * theta.cos().powi(2);
        let sin_theta = theta.sin();
        
        // Energy (E = -p_t in our sign convention)
        let energy = -pt;
        
        // Axial angular momentum (L_z = p_phi)
        let angular_momentum_z = pphi;
        
        // Carter's constant calculation
        let carter_constant = ptheta * ptheta + theta.cos().powi(2) * 
            (spin * spin * (energy * energy - 1.0) + angular_momentum_z * angular_momentum_z / sin_theta.powi(2));
        
        Self {
            energy,
            angular_momentum_z,
            carter_constant,
        }
    }
}

/// Kerr black hole parameters in Kerr-Schild coordinates
#[derive(Debug, Clone, Copy)]
pub struct KerrBlackHole {
    /// Mass of the black hole in geometric units
    pub mass: f32,
    /// Dimensionless spin parameter (a = J/Mc, where |a| ≤ M)
    pub spin: f32,
}

impl KerrBlackHole {
    /// Create a new Kerr black hole
    pub fn new(mass: f32, spin: f32) -> Self {
        // Ensure spin parameter is physical (|a| ≤ M)
        let spin = spin.clamp(-mass, mass);
        Self { mass, spin }
    }
    
    /// Create a Schwarzschild (non-spinning) black hole
    pub fn schwarzschild(mass: f32) -> Self {
        Self::new(mass, 0.0)
    }
    
    /// Calculate the outer event horizon radius
    pub fn outer_horizon(&self) -> f32 {
        self.mass + (self.mass * self.mass - self.spin * self.spin).sqrt()
    }
    
    /// Calculate the inner horizon radius (Cauchy horizon)
    pub fn inner_horizon(&self) -> f32 {
        self.mass - (self.mass * self.mass - self.spin * self.spin).sqrt()
    }
    
    /// Calculate the ergosphere radius at a given theta
    pub fn ergosphere_radius(&self, theta: f32) -> f32 {
        self.mass + (self.mass * self.mass - self.spin * self.spin * theta.cos().powi(2)).sqrt()
    }
    
    /// Calculate the ISCO (Innermost Stable Circular Orbit) radius
    pub fn isco_radius(&self) -> f32 {
        let a = self.spin / self.mass;
        let z1 = 1.0 + (1.0 - a * a).powf(1.0/3.0) * ((1.0 + a).powf(1.0/3.0) + (1.0 - a).powf(1.0/3.0));
        let z2 = (3.0 * a * a + z1 * z1).sqrt();
        
        self.mass * (3.0 + z2 - ((3.0 - z1) * (3.0 + z1 + 2.0 * z2)).sqrt())
    }
}

/// Kerr-Schild metric calculations
pub mod kerr_schild {
    use super::KerrBlackHole;
    
    /// Calculate Σ = r² + a²cos²θ
    pub fn sigma(r: f32, theta: f32, spin: f32) -> f32 {
        r * r + spin * spin * theta.cos().powi(2)
    }
    
    /// Calculate Δ = r² - 2Mr + a²
    pub fn delta(r: f32, mass: f32, spin: f32) -> f32 {
        r * r - 2.0 * mass * r + spin * spin
    }
    
    /// Calculate A = (r² + a²)² - a²Δsin²θ
    pub fn a_function(r: f32, theta: f32, mass: f32, spin: f32) -> f32 {
        let r2_plus_a2 = r * r + spin * spin;
        let sin_theta_sq = theta.sin().powi(2);
        r2_plus_a2 * r2_plus_a2 - spin * spin * delta(r, mass, spin) * sin_theta_sq
    }
    
    /// Calculate the Kerr-Schild metric components
    pub fn metric_components(r: f32, theta: f32, bh: &KerrBlackHole) -> [[f32; 4]; 4] {
        let mass = bh.mass;
        let spin = bh.spin;
        let sig = sigma(r, theta, spin);
        let _del = delta(r, mass, spin);
        let _a_func = a_function(r, theta, mass, spin);
        let sin_theta = theta.sin();
        let _cos_theta = theta.cos();
        
        let mut g = [[0.0; 4]; 4];
        
        // g_tt
        g[0][0] = -(1.0 - 2.0 * mass * r / sig);
        
        // g_tr = g_rt (mixed time-radial) - only non-zero for Kerr-Schild
        if spin.abs() > 1e-10 {
            g[0][1] = 2.0 * mass * r / sig;
            g[1][0] = g[0][1];
        }
        
        // g_tphi = g_phi_t (mixed time-azimuthal)
        g[0][3] = -2.0 * mass * r * spin * sin_theta.powi(2) / sig;
        g[3][0] = g[0][3];
        
        // g_rr
        if spin.abs() > 1e-10 {
            g[1][1] = 1.0 + 2.0 * mass * r / sig;
        } else {
            // Schwarzschild case: g_rr = 1/(1-2M/r)
            g[1][1] = sig / (sig - 2.0 * mass * r);
        }
        
        // g_rphi = g_phi_r (mixed radial-azimuthal)
        g[1][3] = -spin * sin_theta.powi(2) * (1.0 + 2.0 * mass * r / sig);
        g[3][1] = g[1][3];
        
        // g_theta_theta
        g[2][2] = sig;
        
        // g_phi_phi
        g[3][3] = sin_theta.powi(2) * (sig + spin * spin * sin_theta.powi(2) * (1.0 + 2.0 * mass * r / sig));
        
        g
    }
    
    /// Check if a position is inside the outer event horizon
    pub fn is_inside_horizon(r: f32, bh: &KerrBlackHole) -> bool {
        r <= bh.outer_horizon()
    }
    
    /// Check if a position is in the ergosphere
    pub fn is_in_ergosphere(r: f32, theta: f32, bh: &KerrBlackHole) -> bool {
        r <= bh.ergosphere_radius(theta) && r > bh.outer_horizon()
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
    
    #[test]
    fn test_kerr_black_hole_creation() {
        let bh = KerrBlackHole::new(1.0, 0.5);
        assert_eq!(bh.mass, 1.0);
        assert_eq!(bh.spin, 0.5);
        
        // Test spin clamping
        let bh_extreme = KerrBlackHole::new(1.0, 1.5); // Spin > mass
        assert_eq!(bh_extreme.spin, 1.0); // Should be clamped to mass
        
        // Test Schwarzschild case
        let bh_schwarzschild = KerrBlackHole::schwarzschild(2.0);
        assert_eq!(bh_schwarzschild.mass, 2.0);
        assert_eq!(bh_schwarzschild.spin, 0.0);
    }
    
    #[test]
    fn test_kerr_horizons() {
        let bh = KerrBlackHole::new(1.0, 0.6);
        
        let outer = bh.outer_horizon();
        let inner = bh.inner_horizon();
        
        assert!(outer > inner);
        assert!(outer <= 2.0 * bh.mass); // Should be <= Schwarzschild radius
        assert!(inner >= 0.0);
        
        // For Schwarzschild case (a=0), outer horizon should equal Schwarzschild radius
        let bh_schwarzschild = KerrBlackHole::schwarzschild(1.0);
        assert!((bh_schwarzschild.outer_horizon() - 2.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_kerr_isco() {
        let bh = KerrBlackHole::new(1.0, 0.5);
        let isco = bh.isco_radius();
        
        // ISCO should be between the outer horizon and 6M (Schwarzschild ISCO)
        assert!(isco > bh.outer_horizon());
        assert!(isco <= 6.0 * bh.mass);
        
        // For Schwarzschild case, ISCO should be 6M
        let bh_schwarzschild = KerrBlackHole::schwarzschild(1.0);
        let isco_schwarzschild = bh_schwarzschild.isco_radius();
        assert!((isco_schwarzschild - 6.0).abs() < 0.1); // Within reasonable tolerance
    }
    
    #[test]
    fn test_kerr_schild_metric_functions() {
        use super::kerr_schild::*;
        
        let mass = 1.0;
        let spin = 0.5;
        let r = 4.0;
        let theta = std::f32::consts::PI / 2.0; // Equatorial plane
        
        let sig = sigma(r, theta, spin);
        let del = delta(r, mass, spin);
        
        assert!(sig > 0.0);
        assert!(del > 0.0); // Outside horizon
        
        // Test metric components
        let bh = KerrBlackHole::new(mass, spin);
        let g = metric_components(r, theta, &bh);
        
        // g_tt should be negative (timelike)
        assert!(g[0][0] < 0.0);
        
        // g_rr should be positive (spacelike)
        assert!(g[1][1] > 0.0);
        
        // g_theta_theta should be positive
        assert!(g[2][2] > 0.0);
        
        // g_phi_phi should be positive
        assert!(g[3][3] > 0.0);
    }
    
    #[test]
    fn test_conserved_quantities() {
        let position = [0.0, 10.0, std::f32::consts::PI / 2.0, 0.0];
        let momentum = [-1.0, 0.1, 0.0, 0.2]; // Negative p_t for positive energy
        let mass = 1.0;
        let spin = 0.3;
        
        let conserved = ConservedQuantities::from_initial_conditions(position, momentum, mass, spin);
        
        // Energy should be positive for escaping photons
        assert!(conserved.energy > 0.0);
        
        // Angular momentum can be positive or negative
        assert!(conserved.angular_momentum_z.is_finite());
        
        // Carter's constant should be non-negative
        assert!(conserved.carter_constant >= 0.0);
    }
    
    #[test]
    fn test_kerr_light_ray_creation() {
        let camera_pos = [0.0, 0.0, 10.0];
        let ray_dir = [0.0, 0.0, -1.0]; // Towards black hole
        let bh = KerrBlackHole::new(1.0, 0.5);
        
        let ray = KerrLightRay::new(camera_pos, ray_dir, bh);
        
        assert_eq!(ray.black_hole.mass, 1.0);
        assert_eq!(ray.black_hole.spin, 0.5);
        assert_eq!(ray.step_count, 0);
        assert!(ray.geodesic.radius() > 0.0);
        assert!(!ray.has_escaped()); // Starting at r=10, should not be escaped yet
    }
    
    #[test]
    fn test_adaptive_rk45_integrator() {
        let integrator = AdaptiveRK45::default();
        
        // Test with a simple harmonic oscillator: d²x/dt² = -x
        // Rewritten as first order: dx/dt = v, dv/dt = -x
        let initial_state = Geodesic::new([0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0]); // x=1, v=0
        
        let derivatives_fn = |state: Geodesic| {
            let x = state.position[1];
            let v = state.momentum[2];
            Geodesic::new([0.0, v, 0.0, 0.0], [0.0, 0.0, -x, 0.0])
        };
        
        let (new_state, step_used, next_step) = integrator.step(initial_state, 0.1, derivatives_fn);
        
        assert!(step_used > 0.0);
        assert!(next_step > 0.0);
        assert!(new_state.position[1].is_finite()); // Position should be finite
        assert!(new_state.momentum[2].is_finite()); // Velocity should be finite
    }
    
    #[test]
    fn test_kerr_vs_schwarzschild_limit() {
        // Test that Kerr reduces to Schwarzschild when spin = 0
        let mass = 1.0;
        let r = 5.0;
        let theta = std::f32::consts::PI / 2.0;
        
        let bh_kerr = KerrBlackHole::new(mass, 0.0); // Zero spin
        let g_kerr = kerr_schild::metric_components(r, theta, &bh_kerr);
        
        // Compare with Schwarzschild metric components
        let g_tt_schwarzschild = schwarzschild::g_tt(mass, r);
        let g_rr_schwarzschild = schwarzschild::g_rr(mass, r);
        let g_theta_theta_schwarzschild = schwarzschild::g_theta_theta(r);
        let g_phi_phi_schwarzschild = schwarzschild::g_phi_phi(r, theta);
        
        // Allow for small numerical differences
        assert!((g_kerr[0][0] - g_tt_schwarzschild).abs() < 1e-6);
        assert!((g_kerr[1][1] - g_rr_schwarzschild).abs() < 1e-6);
        assert!((g_kerr[2][2] - g_theta_theta_schwarzschild).abs() < 1e-6);
        assert!((g_kerr[3][3] - g_phi_phi_schwarzschild).abs() < 1e-6);
        
        // Off-diagonal terms should be zero for Schwarzschild
        assert!(g_kerr[0][1].abs() < 1e-6);
        assert!(g_kerr[0][3].abs() < 1e-6);
        assert!(g_kerr[1][3].abs() < 1e-6);
    }
}
