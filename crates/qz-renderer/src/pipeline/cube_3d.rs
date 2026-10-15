//! # 3D Cube Fast User Switching Pipeline
//!
//! Renders the 3D cube rotation transition when switching user sessions:
//! - Perspective camera matrix ($45^\circ$ FOV).
//! - Continuous spring-driven rotation angle $\theta \in [0, \pi/2]$.
//! - Dual-face texturing (outgoing session and incoming session).

use glam::{Mat4, Vec3};
use std::f32::consts::PI;

/// Face orientation of the 3D session cube.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeFace {
    /// Active front user session face.
    Front,
    /// Incoming user session face (rotated 90 degrees).
    Right,
    /// Back face.
    Back,
    /// Left face.
    Left,
}

/// Push constant uniforms matching `cube_3d.vert` and `cube_3d.frag`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubeUniforms {
    /// Combined Model-View-Projection matrix.
    pub mvp_matrix: Mat4,
    /// Current rotation angle in radians $[0.0, \pi/2]$.
    pub rotation_angle: f32,
    /// Active face ID.
    pub active_face: i32,
    /// Ambient light term (default 0.7).
    pub ambient_light: f32,
    /// Shading contrast (default 0.4).
    pub shading_strength: f32,
}

impl CubeUniforms {
    /// Computes the MVP matrix for the 3D cube given aspect ratio and normalized rotation progress `t` in $[0.0, 1.0]$.
    pub fn compute(aspect_ratio: f32, progress: f32, rotate_left: bool) -> Self {
        let angle = progress.clamp(0.0, 1.0) * (PI * 0.5);
        let signed_angle = if rotate_left { -angle } else { angle };

        // Camera positioned to view unit cube
        let fov_rad = 45.0 * (PI / 180.0);
        let proj = Mat4::perspective_rh(fov_rad, aspect_ratio, 0.1, 100.0);

        // Pull camera back slightly during rotation to keep corners within viewport
        let camera_distance = 2.414 + 0.3 * (progress * PI).sin();
        let view = Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, camera_distance),
            Vec3::ZERO,
            Vec3::Y,
        );

        // Rotation around Y axis
        let model = Mat4::from_rotation_y(signed_angle);
        let mvp = proj * view * model;

        Self {
            mvp_matrix: mvp,
            rotation_angle: signed_angle,
            active_face: 0,
            ambient_light: 0.70,
            shading_strength: 0.40,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cube_mvp_matrix_finite() {
        let uniforms = CubeUniforms::compute(16.0 / 9.0, 0.5, false);
        assert!(uniforms.mvp_matrix.is_finite());
        assert!(uniforms.rotation_angle > 0.0);
    }
}
