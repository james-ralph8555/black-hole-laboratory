/// Module for generating 3D geometric primitives

use crate::Vertex;
use std::f32::consts::PI;

/// Generates vertices and indices for a sphere
/// Returns (vertices, indices)
pub fn generate_sphere(radius: f32, latitude_segments: u32, longitude_segments: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices
    for lat in 0..=latitude_segments {
        let theta = lat as f32 * PI / latitude_segments as f32; // 0 to PI
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();
        
        for lon in 0..=longitude_segments {
            let phi = lon as f32 * 2.0 * PI / longitude_segments as f32; // 0 to 2*PI
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();
            
            // Spherical coordinates to Cartesian
            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;
            
            // Texture coordinates (UV mapping)
            let u = lon as f32 / longitude_segments as f32;
            let v = lat as f32 / latitude_segments as f32;
            
            vertices.push(Vertex {
                position: [x, y, z],
                tex_coords: [u, v],
            });
        }
    }

    // Generate indices for triangular faces
    for lat in 0..latitude_segments {
        for lon in 0..longitude_segments {
            let current_row = lat * (longitude_segments + 1);
            let next_row = (lat + 1) * (longitude_segments + 1);
            
            let current = (current_row + lon) as u16;
            let current_next = (current_row + lon + 1) as u16;
            let next = (next_row + lon) as u16;
            let next_next = (next_row + lon + 1) as u16;
            
            // Two triangles per quad
            if lat == 0 {
                // Top triangles (avoiding degenerate triangles at poles)
                indices.extend_from_slice(&[current, next, next_next]);
            } else if lat == latitude_segments - 1 {
                // Bottom triangles (avoiding degenerate triangles at poles)
                indices.extend_from_slice(&[current, current_next, next]);
            } else {
                // Regular quads
                indices.extend_from_slice(&[
                    current, next, current_next,
                    current_next, next, next_next,
                ]);
            }
        }
    }

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere_generation() {
        let (vertices, indices) = generate_sphere(1.0, 8, 16);
        
        // Should have (lat_segments + 1) * (lon_segments + 1) vertices
        assert_eq!(vertices.len(), (8 + 1) * (16 + 1));
        
        // Check that indices are valid
        for &index in &indices {
            assert!((index as usize) < vertices.len());
        }
        
        // Check that some vertices are on the sphere surface
        for vertex in &vertices[0..5] {
            let pos = vertex.position;
            let distance = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
            assert!((distance - 1.0).abs() < 1e-6);
        }
    }
    
    #[test]
    fn test_sphere_texture_coordinates() {
        let (vertices, _) = generate_sphere(2.0, 4, 8);
        
        // Check that texture coordinates are in valid range [0, 1]
        for vertex in &vertices {
            assert!(vertex.tex_coords[0] >= 0.0 && vertex.tex_coords[0] <= 1.0);
            assert!(vertex.tex_coords[1] >= 0.0 && vertex.tex_coords[1] <= 1.0);
        }
    }
}