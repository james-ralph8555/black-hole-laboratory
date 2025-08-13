# Real-Time Black Hole Simulator

This project is a real-time, physically-accurate black hole simulator that runs natively for development and on the web using Rust and `wgpu`. The application visualizes the fascinating and extreme environment around a black hole by ray tracing the paths of light through curved spacetime.

The core of the project is separated into two main components: a physics simulation crate and a rendering crate.

*   **`simulation` crate**: Handles the heavy lifting of general relativity, providing the physics structs and calculations for how light travels through curved spacetime.
*   **`renderer` crate**: Uses `wgpu` for cross-platform rendering. It traces light rays for each pixel on the screen to visualize the black hole and the gravitational lensing of the background starfield.

## Current Status

The project has a working real-time ray tracer that simulates a black hole:
- ✅ Cross-platform ray tracing renderer (native + WebAssembly) using `wgpu`.
- ✅ Renders a Schwarzschild (non-spinning) black hole.
- ✅ Visualizes gravitational lensing by distorting the background starfield.
- ✅ Camera system with keyboard, mouse, and touch controls.
- ✅ Interactive help and debug overlay.
- ✅ Visual toggles for the starfield and a coordinate grid.

## Quick Start

### Prerequisites
- [Nix](https://nixos.org/download.html) (recommended for reproducible environment)
- Or: Rust toolchain with wasm32 target, Node.js 22+, wasm-pack

### Running for Development

To run the project with a local development server that supports live reloading:

```bash
# Enter development environment (with Nix)
nix develop

# Navigate to the web directory
cd www

# Install dependencies and start the dev server
npm install
npm run serve

# Open your browser to http://localhost:8080
```

### Deploying as a Static Site

The project is built as a static site and can be deployed to any static hosting provider (e.g., GitHub Pages, Netlify, Vercel).

```bash
# From the www directory, create an optimized production build
cd www
npm run build
```
This command will generate all necessary files in the `www/dist` directory. Upload the contents of this directory to your hosting provider.

### Deploying with AWS Amplify

This project is configured for automated builds and deployments on [AWS Amplify](https://aws.amazon.com/amplify/) using a custom Docker build image.

#### Custom Build Image
The `Dockerfile` defines a Node.js 22.18.0-slim based build environment with:
- Rust toolchain (nightly) installed via rustup
- wasm-pack for WebAssembly compilation
- Build tools (gcc, libc6-dev, make) for native dependencies

*Note: While the project includes a Nix flake for reproducible local development, the deployment uses a simplified Node.js-based Docker image for AWS Amplify compatibility.*

#### Deployment Steps
1.  **Build and Push the Custom Docker Image**: 
    ```bash
    # Example for Docker Hub
    docker build -t your-dockerhub-username/black-hole-sim-build:latest .
    docker push your-dockerhub-username/black-hole-sim-build:latest
    ```

2.  **Connect to Amplify**: In the AWS Amplify Console, connect this Git repository.
3.  **Configure Build Settings**:
    - Go to **Advanced settings** in the build configuration step
    - Provide the URL to your custom build image (e.g., `your-dockerhub-username/black-hole-sim-build:latest`)
    - Amplify will use the `amplify.yml` file which runs:
      - `cd www && npm install && npm run build`
      - Deploys static files from `/www/dist`
4.  **Deploy**: Amplify will pull your custom image, run the build commands, and host the static artifacts.

### Alternative (without Nix)
```bash
# Ensure wasm32 target is installed
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Navigate to the web directory
cd www

# Install dependencies
npm install

# Start dev server or build for production
npm run serve
# or
npm run build
```


## Core Concepts

To accurately render a black hole, we must simulate how light behaves in the intensely curved spacetime described by Einstein's theory of General Relativity. This project models a spinning **Kerr black hole** using Kerr-Schild coordinates and incorporates advanced physics like magnetic fields and relativistic effects on an accretion disk.

### From Schwarzschild to Kerr Black Holes

While the initial model focused on a **Schwarzschild black hole** (non-rotating, spherically symmetric), this project has evolved to simulate a **Kerr black hole**. The Kerr metric is necessary to model a rotating black hole and introduces fascinating phenomena like **frame-dragging**, where the black hole's rotation literally drags spacetime along with it.

To handle the complexities of a spinning black hole without numerical issues, we use **Kerr-Schild coordinates**. This formulation provides a stable, singularity-free metric that is well-behaved across the event horizon, making it ideal for simulations that trace light rays near or across this boundary.

### Geodesic Integration in Kerr Spacetime

In curved spacetime, light follows paths called **geodesics**. For a Kerr black hole, solving the geodesic equations is simplified by exploiting four conserved quantities for photons:
- **Energy (E)**
- **Axial Angular Momentum (Lz)**
- **Carter's Constant (Q)**
- **Rest mass (μ=0 for photons)**

These constants allow the complex second-order geodesic equations to be reformulated into a more manageable set of four first-order ordinary differential equations (ODEs). These equations are then solved numerically using an **adaptive 4th-order Runge-Kutta (RK45) method** to trace light rays backwards from each pixel on the screen. The adaptive step size ensures precision where spacetime curvature is high (near the black hole) and efficiency where it's flatter, stopping integration if a ray crosses the event horizon or escapes to a predefined distance.

### Ray Tracing and Relativistic Effects

The renderer works by "backwards" ray tracing from a virtual camera. The final color of each pixel is determined by what the ray encounters:

*   **The Black Hole Shadow**: If a ray's path crosses the event horizon, it is trapped. The pixel is colored black, forming the black hole's "shadow."
*   **The Accretion Disk**: If the ray intersects the accretion disk, we calculate the observed color based on several relativistic effects:
    *   **Relativistic Doppler Effect & Beaming**: The disk's rapid orbital motion causes light from the side moving towards the camera to be blueshifted and appear brighter, while light from the receding side is redshifted and dimmer.
    *   **Gravitational Redshift**: Photons lose energy escaping the black hole's gravity, shifting their color towards red.
    *   The combination of these effects determines the final observed temperature and intensity of the light from that point on the disk.
*   **Gravitational Lensing**: If a ray escapes to infinity, its final direction is used to sample a background starfield, producing the characteristic distortion of stars around the black hole.

### Modeling Magnetic Fields and the Accretion Disk

To create a realistic visualization, the simulation incorporates advanced physical models:
*   **General Relativistic Magnetohydrodynamics (GRMHD)**: This models the behavior of plasma and magnetic fields in the extreme gravity around the black hole, which is crucial for simulating accretion disks and jets. The GRMHD simulation provides the physical environment through which light rays are traced.
*   **Physically Motivated Accretion Disk**: Instead of a simple visual disk, the model uses the **Shakura-Sunyaev "thin disk"** model. This includes a temperature profile based on the **Novikov-Thorne model** for a fully relativistic treatment, with the luminous inner edge defined by the **Innermost Stable Circular Orbit (ISCO)**. The ISCO's radius depends heavily on the black hole's spin.

## Project Architecture

The project is structured as a Rust workspace to maintain a clean separation of concerns.

```text
/black-hole-simulator
|-- Cargo.toml            // Rust workspace configuration
|-- amplify.yml           // AWS Amplify deployment config
|-- CLAUDE.md             // AI assistant guidance
|-- Dockerfile            // Custom Docker build image
|-- README.md
|-- /renderer
|   |-- Cargo.toml
|   |-- src/
|   |   |-- camera.rs     // Camera and controls
|   |   |-- geometry.rs   // Vertex data for the screen quad
|   |   |-- lib.rs        // Core renderer logic (WASM entry)
|   |   |-- main.rs       // Native entry point
|   |   |-- shader.wgsl   // WGSL shader source (embedded)
|   |-- shaders/
|       |-- render.wgsl   // The primary WGSL shader for ray tracing
|-- /simulation
|   |-- Cargo.toml
|   |-- src/lib.rs        // Physics logic (geodesics, metrics)
|-- /www                  // Web frontend (HTML/JS)
|   |-- bootstrap.js      // JS entry point for WASM
|   |-- index.html
|   |-- package.json
|   |-- pkg/              // Generated WASM package for JS interop
|   |-- style.css         // CSS for the web page
|   |-- webpack.config.js
```

### `simulation` Crate

*   **Responsibilities**: Pure physics calculations.
*   Contains the foundational data structures (`VolumetricMass`, `Geodesic`) and logic for simulating general relativity.
*   The goal is for this crate to define the spacetime metric (e.g., in Kerr-Schild coordinates), calculate Christoffel symbols, and provide a robust geodesic equation solver (e.g., using Runge-Kutta 4).
*   It is compiled to a library that is used by the `renderer` crate. The current ray tracing is implemented directly in the shader for performance, with plans to use this crate for more accurate physics in the future.

### `renderer` Crate

*   **Responsibilities**: All rendering and user interaction logic.
*   The core logic is in `src/lib.rs`, which is compiled to WebAssembly to run in the browser.
*   A thin `src/main.rs` binary is included to run the application natively for development and testing.
*   Uses `wgpu` for cross-platform graphics with WebGL backend for web.
*   **Current implementation**: 
    *   Full graphics pipeline with vertex/fragment shaders
    *   Camera system with view/projection matrices (`src/camera.rs`)
    *   Input handling for keyboard, mouse, and touch (`camera.rs`)
    *   WASM-compatible async initialization and timing
    *   Renders a single full-screen quad to trigger fragment shader execution for every pixel.
*   **Ray Tracing Implementation**: The fragment shader (`render.wgsl`) performs the ray tracing. For each pixel, it:
    *   Calculates the initial direction of a light ray from the camera's perspective.
    *   Iteratively steps the ray through spacetime, applying a simplified gravitational pull from the black hole at each step.
    *   Determines if the ray falls into the event horizon (coloring the pixel black) or escapes (coloring it based on the background starfield).

## Roadmap & Future Improvements

This project is designed to be extensible. The current implementation provides a solid foundation for a real-time Schwarzschild black hole visualization. The next steps focus on transitioning to a more physically accurate and visually complex simulation of a spinning Kerr black hole.

- [x] **Schwarzschild Black Hole**: Simulation of a non-spinning black hole is complete.
- [x] **Gravitational Lensing**: The lensing effect is visible and emerges from the ray tracing implementation.
- [ ] **Kerr (Spinning) Black Hole**:
    - Transition from the Schwarzschild metric to the **Kerr metric** to simulate a rotating black hole and visualize effects like **frame-dragging**.
    - Implement the geodesic equations in **Kerr-Schild coordinates** to ensure numerical stability across the event horizon.
    - Reformulate the geodesic equations into a set of first-order ODEs using conserved quantities (Energy, Angular Momentum, Carter's Constant) for efficient numerical integration.

- [ ] **Physically-Based Accretion Disk**:
    - Replace the current visual placeholder with a physically-motivated accretion disk based on the **Shakura-Sunyaev "thin disk"** model.
    - Implement the **Novikov-Thorne model** to calculate the disk's temperature profile, with the luminous inner edge defined by the spin-dependent **Innermost Stable Circular Orbit (ISCO)**.
    - Develop an efficient algorithm to calculate ray-disk intersections, potentially using precomputed tables for real-time performance.

- [ ] **Advanced Relativistic Effects**:
    - Model **General Relativistic Magnetohydrodynamics (GRMHD)** to simulate the dynamics of plasma and magnetic fields, forming the basis for a realistic accretion disk and jets.
    - Implement relativistic optics for light emitted from the disk, including:
        - **Gravitational Redshift**: Light losing energy as it escapes the gravitational well.
        - **Relativistic Doppler Effect**: Redshifting/blueshifting of light due to the disk's orbital velocity.
        - **Relativistic Beaming**: The focusing of light in the direction of motion, making the approaching side of the disk appear significantly brighter.
    - Map the calculated observed temperature and intensity to final pixel colors, potentially using Planck's law for blackbody radiation.

- [ ] **Performance Optimization & Accuracy**:
    - Implement an **adaptive step-size Runge-Kutta solver (e.g., RK45)** for geodesic integration, improving accuracy near the black hole without sacrificing performance.
    - Enhance GPU acceleration by moving all ray-tracing calculations to shaders.
    - Utilize **precomputation** for performance-critical calculations, such as ray deflection tables, disk intersection lookups, and color transformations for Doppler effects.
    - Optimize calculations using **single-precision floating-point numbers** where possible.
