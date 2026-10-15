//! # Genie Effect 64x64 Mesh Pipeline
//!
//! Generates a $64 \times 64$ quadrilateral tessellation mesh and computes
//! non-linear Hermite curve fluid deformation for window minimize/restore animations.

use glam::{Vec2, Vec4};

/// Uniform payload matching `genie.vert` push constants.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenieUniforms {
    /// Window origin and dimensions: `(x, y, width, height)`.
    pub win_rect: Vec4,
    /// Dock icon origin and dimensions: `(x, y, width, height)`.
    pub icon_rect: Vec4,
    /// Monitor screen resolution: `(width, height)`.
    pub screen_res: Vec2,
    /// Normalized animation progress $t \in [0.0, 1.0]$.
    pub t: f32,
    /// Dock placement edge: 0 = Bottom, 1 = Left, 2 = Right.
    pub dock_edge: i32,
}

/// Mesh vertex for the 64x64 Genie tessellation grid.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenieVertex {
    /// Normalized UV coordinate in $[0.0, 1.0]^2$.
    pub uv: Vec2,
}

/// Tessellated $64 \times 64$ grid geometry for Genie deformation.
pub struct GenieMesh {
    /// Vertex buffer containing $65 \times 65 = 4225$ vertices.
    pub vertices: Vec<GenieVertex>,
    /// Index buffer containing $64 \times 64 \times 6 = 24576$ indices.
    pub indices: Vec<u32>,
}

impl GenieMesh {
    /// Grid resolution requested by macOS Tahoe specification ($64 \times 64$).
    pub const GRID_DIVISIONS: usize = 64;

    /// Generates static UV grid vertices and index buffers for GPU uploads.
    pub fn generate() -> Self {
        let n = Self::GRID_DIVISIONS;
        let mut vertices = Vec::with_capacity((n + 1) * (n + 1));
        let mut indices = Vec::with_capacity(n * n * 6);

        for j in 0..=n {
            let v = j as f32 / n as f32;
            for i in 0..=n {
                let u = i as f32 / n as f32;
                vertices.push(GenieVertex { uv: Vec2::new(u, v) });
            }
        }

        let stride = (n + 1) as u32;
        for j in 0..n {
            for i in 0..n {
                let row = j as u32;
                let col = i as u32;

                let top_left = row * stride + col;
                let top_right = top_left + 1;
                let bottom_left = (row + 1) * stride + col;
                let bottom_right = bottom_left + 1;

                // Triangle 1: top_left -> bottom_left -> top_right
                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

                // Triangle 2: top_right -> bottom_left -> bottom_right
                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        Self { vertices, indices }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genie_mesh_generation_counts() {
        let mesh = GenieMesh::generate();

        // 65 * 65 = 4225 vertices
        assert_eq!(mesh.vertices.len(), 65 * 65);
        // 64 * 64 * 6 = 24576 indices
        assert_eq!(mesh.indices.len(), 64 * 64 * 6);

        // First vertex should be at (0, 0), last at (1, 1)
        assert_eq!(mesh.vertices.first().unwrap().uv, Vec2::new(0.0, 0.0));
        assert_eq!(mesh.vertices.last().unwrap().uv, Vec2::new(1.0, 1.0));
    }
}
