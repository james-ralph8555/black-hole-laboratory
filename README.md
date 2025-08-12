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

![Black Hole Simulation](https://placehold.co/800x400/000000/FFFFFF?text=Black+Hole+Simulation)

## Core Concepts

To accurately render a black hole, we must simulate how light behaves in the intensely curved spacetime described by Einstein's theory of General Relativity.

### The Schwarzschild Metric & Kerr-Schild Coordinates

For our initial implementation, we will model a **Schwarzschild black hole**: one that is spherically symmetric and does not spin. The geometry of spacetime around such an object is described by the Schwarzschild metric.

However, the standard Schwarzschild coordinates have a mathematical problem called a "coordinate singularity" at the event horizon. This can cause numerical simulations to fail. To avoid this, we will use **Kerr-Schild coordinates**. These coordinates are "well-behaved" across the event horizon, making them ideal for our simulation, as we need to trace light rays that may cross this boundary.

The Kerr-Schild metric `g` can be expressed as a flat spacetime metric `η` plus a null vector term `k`:
```
gμν​=ημν​+fkμ​kν​
```

For a Schwarzschild black hole, the function `f` is `2M/r` (where `M` is the mass and `r` is the radial coordinate), and `k` is a null vector field. This formulation is elegant and computationally advantageous.

### The Geodesic Equation

In curved spacetime, particles and light do not travel in straight lines but follow paths called **geodesics**. A geodesic is the straightest possible path through a curved manifold. For photons (light), we are interested in "null geodesics."

The path of a geodesic is governed by the geodesic equation:
```
dλ2d2xμ​+Γαβμ​dλdxα​dλdxβ​=0
```

*   `x^\mu`: The four-dimensional coordinates of the photon (t, x, y, z).
*   `λ`: The affine parameter, which tracks the photon's progress along its path.
*   `Γ^\mu_{\alpha\beta}`: The Christoffel symbols, which are functions of the metric tensor `g_{\mu\nu}`. They essentially describe the curvature of spacetime.

To find the path of a light ray, we must solve this system of second-order ordinary differential equations (ODEs). We will use a numerical integration method, such as the **4th-order Runge-Kutta algorithm**, to achieve this.

### Ray Tracing

The renderer works by "backwards" ray tracing. For each pixel on the screen, a ray is cast from a virtual camera backwards in time into the scene. A simplified geodesic solver in the fragment shader calculates the path of this ray as it travels through the curved spacetime around the black hole.

The final color of the pixel is determined by the fate of the ray:

*   If the ray's path ends up crossing the event horizon, it means that light from that direction is trapped by the black hole. The pixel will be colored black, forming the iconic "shadow."
*   If the ray escapes to "infinity" (a certain distance from the black hole), its final direction is used to sample a background starfield. This process naturally produces the **gravitational lensing** effect, where the stars behind the black hole appear distorted and warped.

## Project Architecture

The project is structured as a Rust workspace to maintain a clean separation of concerns.

```text
/black-hole-simulator
|-- Cargo.toml
|-- /www                  // Web frontend assets
|   |-- index.html
|   |-- package.json
|   |-- webpack.config.js
|-- /simulation
|   |-- Cargo.toml
|   |-- src/lib.rs        // Defines the metric, Christoffel symbols, and geodesic solver.
|-- /renderer
|   |-- Cargo.toml
|   |-- src/main.rs       // Native entry point for development
|   |-- src/lib.rs        // Core renderer logic, compiles to WASM
|   |-- shaders/
|       |-- render.wgsl   // The shader that performs the ray tracing.
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

This project is designed to be extensible. After establishing the core simulation of a Schwarzschild black hole, we can add more complex and visually stunning phenomena.

- [x] **Schwarzschild Black Hole**: Simulation of a non-spinning black hole is complete.
- [x] **Gravitational Lensing**: The lensing effect is visible and emerges from the ray tracing implementation.
- [ ] **Accretion Disk**: Add a glowing, superheated disk of matter orbiting the black hole. This will be modeled as a flat, textured disk on the equatorial plane. The ray tracing algorithm will be updated to calculate intersections with this disk.
- [ ] **Relativistic Doppler & Beaming**: The material in the accretion disk moves at relativistic speeds. We will model the Doppler effect (redshifting and blueshifting of light) and relativistic beaming (aberration). This will make the side of the disk moving towards the camera appear brighter and bluer, while the side moving away will be dimmer and redder.
- [ ] **Kerr (Spinning) Black Hole**: Upgrade the simulation to a Kerr black hole. This requires implementing the more complex Kerr metric. A spinning black hole drags spacetime around with it (an effect called frame-dragging), which changes the shape of the event horizon and the black hole's shadow.
- [ ] **Improved Physics Accuracy**: Replace the simplified gravity in the shader with a more robust geodesic solver using the `simulation` crate.
- [ ] **Multiple Black Holes**: A highly ambitious goal would be to simulate the spacetime of a binary black hole system. This would likely require moving beyond analytical metrics and into the realm of numerical relativity to approximate the combined spacetime curvature.
