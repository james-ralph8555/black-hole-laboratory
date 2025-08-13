# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a real-time, physically-accurate black hole simulator using Rust, wgpu, and WebAssembly. The project visualizes black holes by ray tracing light paths through curved spacetime using general relativity equations.

## Common Commands

### Building
- **Build web frontend**: `cd www && npm run build`
- **Build Rust workspace**: `cargo build`
- **Build for release**: `cargo build --release`
- **Build WASM**: `wasm-pack build renderer --target web`

### Development Server
- **Start dev server**: `cd www && npm run serve`
- **Run native renderer**: `cargo run -p renderer`

### Testing
- **Run all tests**: `cargo test`
- **Run specific crate tests**: `cargo test -p simulation` or `cargo test -p renderer`

### Linting/Formatting
- **Format code**: `cargo fmt`
- **Check code**: `cargo check`
- **Run clippy**: `cargo clippy`

## Architecture

The project is a Rust workspace with clear separation of concerns:

### `simulation` crate (`simulation/`)
- **Purpose**: Pure physics calculations for general relativity
- **Current state**: Contains foundational structs for physics simulation (`KerrBlackHole`, `Geodesic`, `KerrLightRay`) and comprehensive Kerr metric implementation with adaptive RK45 integration. The main ray tracing logic is currently implemented directly in the shader for performance.
- **Future**: Will implement Kerr-Schild coordinates, Christoffel symbols, and more accurate geodesic equation integration using the `LightRay` struct.
- **Dependencies**: None (pure math)

### `renderer` crate (`renderer/`)
- **Purpose**: Cross-platform graphics using wgpu
- **Dual targets**: 
  - Native binary (`src/main.rs`) for development
  - WebAssembly library (`src/lib.rs`) for web deployment
- **Current state**: Ray tracing renderer that visualizes a Schwarzschild black hole.
- **Features implemented**: 
  - Fragment shader-based ray tracing on a full-screen quad
  - Graphics pipeline with vertex/fragment shaders
  - Camera system with view/projection matrices
  - Keyboard, mouse, and touch controls for movement and looking.
  - Visual toggles for starfield background and coordinate grid
- **Dependencies**: wgpu, winit, cgmath, bytemuck, simulation crate
- **Web integration**: Uses wasm-bindgen for browser compatibility with WASM-specific async handling

### Web frontend (`www/`)
- **Purpose**: HTML/JS wrapper for WASM module
- **Build system**: Webpack with wasm-pack-plugin
- **Target canvas**: Element with ID `wasm-canvas`

## Development Environment

The project uses Nix flakes for reproducible development environments:
- **Enter dev shell**: `nix develop`
- **Includes**: Rust toolchain with wasm32 target, wasm-pack, Node.js 22, language servers

## Key Implementation Details

- **Graphics API**: wgpu for cross-platform rendering (native + WebGL)
- **Camera system**: Modular camera implementation in `renderer/src/camera.rs`
  - View/projection matrix calculations using cgmath
  - Input handling for keyboard (WASD, etc.), mouse (drag-to-look), and touch (virtual joystick).
  - WASM-compatible time handling for smooth movement
- **Rendering pipeline**: Vertex/fragment shader setup with uniform buffers for camera and black hole properties.
- **Vertex data**: A single full-screen quad with position and texture coordinate attributes.
- **Physics target**: A simplified Schwarzschild black hole simulation is implemented directly in the fragment shader.
- **Ray tracing approach**: The fragment shader performs backwards ray tracing from the camera for each pixel. The path of each ray is deflected based on a simplified gravitational model, producing a gravitational lensing effect.
- **Coordinate system**: Currently uses a simplified model. Plans to use Kerr-Schild coordinates to avoid singularities at the event horizon for a more accurate simulation.
- **Integration method**: Plans to use an adaptive 4th-order Runge-Kutta (e.g., RK45) for geodesic integration.

## Deployment

The project is configured for deployment on AWS Amplify using a custom Docker build image. The deployment setup has been simplified to use Node.js and native tooling instead of Nix:

### Docker Build Image (`Dockerfile`)
- **Base**: Node.js 22.18.0-slim for compatibility with AWS Amplify
- **Rust toolchain**: Installed via rustup with nightly toolchain
- **WebAssembly support**: wasm-pack installed for building WASM modules
- **Build tools**: gcc, libc6-dev, make for compiling native dependencies

### Build Configuration (`amplify.yml`)
- **Build process**: Simple npm-based build in the `www/` directory
- **Commands**: `npm install` followed by `npm run build`
- **Output**: Static files deployed from `/www/dist` directory
- **Artifacts**: All files in the dist directory (`**/*`)

## Future Roadmap

The codebase is designed for extensibility. The future roadmap focuses on transitioning to a more physically accurate and visually complex simulation of a spinning Kerr black hole.

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
