// The shader that performs the ray tracing.

// For each pixel on the screen, this shader will:
// 1. Calculate the initial direction of a light ray corresponding to that pixel.
// 2. Repeatedly step the ray along its geodesic.
// 3. Determine the final state of the ray and write the appropriate color to the screen.
