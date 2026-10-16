//! # Genie Effect $64 \times 64$ Mesh Warping Pipeline (`eval_genie_vertex`)
//!
//! Generates a $64 \times 64$ cell ($65 \times 65 = 4225$ vertices) tessellation mesh
//! and evaluates `eval_genie_vertex` (Pillar 3.5) for fluid window minimize/restore animations.

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
    /// Dock placement edge: `0` = Bottom, `1` = Left, `2` = Right.
    pub dock_edge: i32,
}

impl GenieUniforms {
    /// Evaluates the exact `eval_genie_vertex(uv, win_rect, icon_rect, t)` function on CPU (Pillar 3.5).
    #[inline]
    pub fn eval_genie_vertex(uv: Vec2, win_rect: Vec4, icon_rect: Vec4, t: f32) -> Vec2 {
        let smoothstep = |e0: f32, e1: f32, x: f32| {
            let u = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
            u * u * (3.0 - 2.0 * u)
        };
        let p1 = smoothstep(0.0, 0.65, t);
        let p2 = smoothstep(0.25, 1.0, t);
        let v = uv.y;
        let w = ((v - (1.0 - p1)) / (p1 + 1e-4)).clamp(0.0, 1.0);
        let h_w = w * w * (3.0 - 2.0 * w);
        let h_v = v * v * (3.0 - 2.0 * v);
        let blend_x = h_w + (1.0 - h_w) * p2;
        let xl = win_rect.x + (icon_rect.x - win_rect.x) * blend_x;
        let xr = (win_rect.x + win_rect.z) + ((icon_rect.x + icon_rect.z) - (win_rect.x + win_rect.z)) * blend_x;
        let x = xl + (xr - xl) * uv.x;
        let y_win = win_rect.y + v * win_rect.w;
        let y_icn = icon_rect.y + v * icon_rect.w;
        let y = y_win + (y_icn - y_win) * (p2 * (0.3 + 0.7 * h_v));
        Vec2::new(x, y)
    }
}

/// Mesh vertex for the $64 \times 64$ Genie tessellation grid.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenieVertex {
    /// Normalized UV coordinate in $[0.0, 1.0]^2$.
    pub uv: Vec2,
}

/// Tessellated $64 \times 64$ grid geometry ($65 \times 65$ vertices) for Genie deformation.
pub struct GenieMesh {
    /// Vertex buffer containing $65 \times 65 = 4225$ vertices.
    pub vertices: Vec<GenieVertex>,
    /// Index buffer containing $64 \times 64 \times 6 = 24576$ indices.
    pub indices: Vec<u32>,
}

impl GenieMesh {
    /// Grid resolution requested by macOS Tahoe specification ($64 \times 64$ cells).
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

                indices.push(top_left);
                indices.push(bottom_left);
                indices.push(top_right);

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
    fn test_genie_mesh_and_eval_genie_vertex() {
        let mesh = GenieMesh::generate();
        assert_eq!(mesh.vertices.len(), 65 * 65);
        assert_eq!(mesh.indices.len(), 64 * 64 * 6);

        let win = Vec4::new(100.0, 100.0, 800.0, 600.0);
        let icn = Vec4::new(500.0, 980.0, 64.0, 64.0);

        let p_start = GenieUniforms::eval_genie_vertex(Vec2::ZERO, win, icn, 0.0);
        assert!((p_start.x - 100.0).abs() < 1e-3);
        assert!((p_start.y - 100.0).abs() < 1e-3);

        let p_end = GenieUniforms::eval_genie_vertex(Vec2::ONE, win, icn, 1.0);
        assert!((p_end.x - 564.0).abs() < 1e-3);
        assert!((p_end.y - 1044.0).abs() < 1e-3);
    }
}
