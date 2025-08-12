# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a real-time, physically-accurate black hole simulator using Rust, wgpu, and WebAssembly. The project visualizes black holes by ray tracing light paths through curved spacetime using general relativity equations.

## Common Commands

### Building
- **Build everything**: `npm run build` (builds web frontend)
- **Build Rust workspace**: `cargo build`
- **Build for release**: `cargo build --release`
- **Build WASM**: `wasm-pack build renderer --target web`

### Development Server
- **Start dev server**: `npm run serve` (starts webpack dev server)
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
- **Current state**: Placeholder implementation
- **Future**: Will implement Kerr-Schild coordinates, Christoffel symbols, and geodesic equation integration
- **Dependencies**: None (pure math)

### `renderer` crate (`renderer/`)
- **Purpose**: Cross-platform graphics using wgpu
- **Dual targets**: 
  - Native binary (`src/main.rs`) for development
  - WebAssembly library (`src/lib.rs`) for web deployment
- **Current state**: Functional 3D renderer with camera controls
- **Features implemented**: 
  - Graphics pipeline with vertex/fragment shaders
  - Camera system with view/projection matrices
  - WASD keyboard movement controls (+ Space/Shift for up/down)
  - Vertex buffer rendering with indexed geometry
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
  - Keyboard input handling (WASD + Space/Shift)
  - WASM-compatible time handling for smooth movement
- **Rendering pipeline**: Vertex/fragment shader setup with uniform buffer for camera
- **Vertex data**: Position and texture coordinate attributes with indexed rendering
- **Physics target**: Currently placeholder, planned for Schwarzschild black hole simulation
- **Ray tracing approach**: Fragment shader will perform backwards ray tracing from camera
- **Coordinate system**: Will use Kerr-Schild coordinates to avoid singularities at event horizon
- **Integration method**: Plans to use 4th-order Runge-Kutta for geodesic equation

## Future Roadmap

The codebase is designed for extensibility with planned features:
- Accretion disk visualization
- Gravitational lensing effects
- Relativistic Doppler effects
- Kerr (spinning) black holes
- Multiple black hole systems