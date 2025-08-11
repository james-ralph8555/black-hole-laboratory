Real-Time Black Hole Simulator

This project is a plan to develop a real-time, physically-accurate black hole simulator that runs on the web using Rust and WebGPU. The application will visualize the fascinating and extreme environment around a black hole by ray tracing the paths of light through curved spacetime.

The core of the project is separated into two main components: a physics simulation crate and a rendering crate.

    simulation crate: Handles the heavy lifting of general relativity, solving the geodesic equations to determine how light travels.

    renderer crate: Uses wgpu to create the visuals, tracing rays for each pixel on the screen and coloring them based on the results from the simulation.

!(https://placehold.co/800x400/000000/FFFFFF?text=Black+Hole+Simulation)
Core Concepts

To accurately render a black hole, we must simulate how light behaves in the intensely curved spacetime described by Einstein's theory of General Relativity.
The Schwarzschild Metric & Kerr-Schild Coordinates

For our initial implementation, we will model a Schwarzschild black hole: one that is spherically symmetric and does not spin. The geometry of spacetime around such an object is described by the Schwarzschild metric.

However, the standard Schwarzschild coordinates have a mathematical problem called a "coordinate singularity" at the event horizon. This can cause numerical simulations to fail. To avoid this, we will use Kerr-Schild coordinates. These coordinates are "well-behaved" across the event horizon, making them ideal for our simulation, as we need to trace light rays that may cross this boundary.

The Kerr-Schild metric g can be expressed as a flat spacetime metric η plus a null vector term k:
gμν​=ημν​+fkμ​kν​

For a Schwarzschild black hole, the function f is 2M/r (where M is the mass and r is the radial coordinate), and k is a null vector field. This formulation is elegant and computationally advantageous.
The Geodesic Equation

In curved spacetime, particles and light do not travel in straight lines but follow paths called geodesics. A geodesic is the straightest possible path through a curved manifold. For photons (light), we are interested in "null geodesics."

The path of a geodesic is governed by the geodesic equation:
dλ2d2xμ​+Γαβμ​dλdxα​dλdxβ​=0

    x^\mu: The four-dimensional coordinates of the photon (t, x, y, z).

    λ: The affine parameter, which tracks the photon's progress along its path.

    Γ^\mu_{\alpha\beta}: The Christoffel symbols, which are functions of the metric tensor g_{\mu\nu}. They essentially describe the curvature of spacetime.

To find the path of a light ray, we must solve this system of second-order ordinary differential equations (ODEs). We will use a numerical integration method, such as the 4th-order Runge-Kutta algorithm, to achieve this.
Ray Tracing

The renderer will work by "backwards" ray tracing. For each pixel on the screen, we will cast a ray from a virtual camera backwards in time into the scene. We then use our geodesic solver to calculate the path of this ray through the curved spacetime around the black hole.

The final color of the pixel is determined by the fate of the ray:

    If the ray's path ends up crossing the event horizon, it means that light from that direction is trapped by the black hole. The pixel will be colored black, forming the iconic "shadow."

    If the ray escapes to infinity, its final direction is calculated. We can then sample a background texture (like a star map or a nebula) to determine the pixel's color. This process will naturally produce gravitational lensing.

Project Architecture

The project will be structured as a Rust workspace to maintain a clean separation of concerns.

/black-hole-simulator
|-- Cargo.toml
|-- /simulation
|   |-- Cargo.toml
|   |-- src/lib.rs  // Defines the metric, Christoffel symbols, and geodesic solver.
|-- /renderer
|   |-- Cargo.toml
|   |-- src/main.rs // wgpu setup, render loop, shader loading.
|   |-- shaders/
|       |-- render.wgsl // The shader that performs the ray tracing.

simulation Crate

    Responsibilities: Pure physics calculations.

    Defines the spacetime metric in Kerr-Schild coordinates.

    Calculates the Christoffel symbols from the metric.

    Provides a function that takes the initial conditions of a ray (position and momentum) and integrates the geodesic equation over a number of steps.

    Will be compiled to a library that can be linked by the renderer crate.

renderer Crate

    Responsibilities: All rendering and user interaction logic.

    Uses wgpu for cross-platform graphics that can target both native desktops and the web (via WASM).

    Initializes a WebGPU device and sets up a render pipeline.

    Creates a simple scene consisting of a single, screen-sized quad.

    The core logic will be in a compute or fragment shader. For each pixel, this shader will:

        Calculate the initial direction of a light ray corresponding to that pixel.

        Repeatedly call a function (ported from the simulation crate's logic) to step the ray along its geodesic.

        Determine the final state of the ray and write the appropriate color to the screen.

Roadmap & Future Improvements

This project is designed to be extensible. After establishing the core simulation of a Schwarzschild black hole, we can add more complex and visually stunning phenomena.

    [ ] Accretion Disk: We will add a glowing, superheated disk of matter orbiting the black hole. This will be modeled as a flat, textured disk on the equatorial plane. The ray tracing algorithm will be updated to calculate intersections with this disk.

    [ ] Gravitational Lensing: This effect will emerge naturally from the correct implementation of the geodesic ray tracer. Light rays from distant stars that pass near the black hole will be bent, causing the background to appear distorted and warped, creating Einstein rings and other phenomena.

    [ ] Relativistic Doppler & Beaming: The material in the accretion disk is moving at relativistic speeds. We will model the Doppler effect (redshifting and blueshifting of light) and relativistic beaming (aberration). This will make the side of the disk moving towards the camera appear brighter and bluer, while the side moving away will be dimmer and redder.

    [ ] Kerr (Spinning) Black Hole: We will upgrade the simulation to a Kerr black hole. This requires implementing the more complex Kerr metric. A spinning black hole drags spacetime around with it (an effect called frame-dragging), which changes the shape of the event horizon and the black hole's shadow.

    [ ] Multiple Black Holes: A highly ambitious goal would be to simulate the spacetime of a binary black hole system. This would likely require moving beyond analytical metrics and into the realm of numerical relativity to approximate the combined spacetime curvature.
