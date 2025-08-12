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
- **Current state**: Contains foundational structs for physics simulation (`VolumetricMass`, `Geodesic`, `LightRay`) and a simplified Schwarzschild metric implementation. The main ray tracing logic is currently implemented directly in the shader for performance.
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
- **Integration method**: Plans to use 4th-order Runge-Kutta for geodesic equation

## Future Roadmap

The codebase is designed for extensibility. Current features and future plans include:
- [x] **Schwarzschild Black Hole**: A non-spinning black hole is simulated.
- [x] **Gravitational Lensing**: The background is distorted by the black hole's gravity.
- [ ] **Accretion Disk**: Add a glowing, superheated disk of matter orbiting the black hole.
- [ ] **Relativistic Doppler Effects**: Model redshift/blueshift of the accretion disk.
- [ ] **Kerr (Spinning) Black Holes**: Upgrade the simulation to a spinning black hole with frame-dragging effects.
- [ ] **Improved Physics**: Integrate the geodesic equation using the Runge-Kutta solver in the `simulation` crate for more accurate ray paths.
- [ ] **Multiple Black Hole Systems**: An ambitious goal to simulate binary systems.
