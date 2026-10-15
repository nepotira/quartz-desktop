//! # 3D Cube Fast User Switch (`qz-session-lock::cube_switch`)
//!
//! Production-grade 3D Cube Fast User Switching spatial compositor for macOS Tahoe 26.
//!
//! ### Key Mathematical & Spatial Capabilities:
//! - **3D Perspective Projection & Spatial Geometry:**
//!   - Field of view $\text{FOV} = 45^\circ$ ($0.785398\text{ rad}$).
//!   - Exact analytical camera pullback distance $d = \frac{1}{\tan(\text{FOV} / 2)} \approx 2.41421356$
//!     ensuring the active desktop surface matches screen dimensions at rotation angle $\theta = 0^\circ$.
//!   - Model-View-Projection ($\mathbf{MVP} = \mathbf{P} \cdot \mathbf{V} \cdot \mathbf{R}_y(\theta)$)
//!     matrix generation for Vulkan/OpenGL rendering.
//! - **1:1 Interactive Gesture & Continuous Spring Physics:**
//!   - Interactive two-finger trackpad drag mapping real-time delta to cube rotation angle.
//!   - $45^\circ$ commit threshold: releasing past $45^\circ$ commits rotation to target user
//!     at $90^\circ$ ($\pi/2$), while releasing prior to $45^\circ$ snaps back to $0^\circ$.
//!   - Driven by `qz_animation::SpringAnimation` (preset `mission_control`, $\zeta = 1.0$).

use glam::{Mat4, Vec3};
use qz_animation::{SpringAnimation, SpringParams};
use serde::{Deserialize, Serialize};

/// Direction of cube rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CubeRotationDirection {
    /// Rotate clockwise to next user (right face, target angle $-\pi/2$).
    RotateRight,
    /// Rotate counter-clockwise to previous user (left face, target angle $+\pi/2$).
    RotateLeft,
}

/// Operational state of the 3D cube user switcher.
#[derive(Debug, Clone, PartialEq)]
pub enum CubeSwitchState {
    /// Cube is resting flat at angle 0.0 with single active user.
    Idle,
    /// User is interactively dragging the cube via two-finger trackpad gesture.
    InteractiveDrag {
        /// Current rotation angle in radians.
        current_angle_rad: f64,
        /// Screen width used for gesture ratio calculations.
        screen_width: f64,
    },
    /// Cube is actively animating toward an equilibrium angle via spring physics.
    SpringAnimating {
        /// 1D analytical spring solver driving angle $\theta(t)$.
        spring: SpringAnimation,
        /// Target destination angle in radians ($0.0$ or $\pm \pi/2$).
        target_angle_rad: f64,
        /// Destination user identifier if switch commits.
        target_user_id: u32,
    },
    /// Rotation committed and settled at $\pm 90^\circ$; target session is now active.
    Completed {
        /// Target user session identifier.
        active_user_id: u32,
    },
}

/// 3D Cube Fast User Switch Spatial Engine.
#[derive(Debug, Clone)]
pub struct CubeSwitchEngine {
    /// Active operational state.
    state: CubeSwitchState,
    /// Display aspect ratio ($W / H$).
    pub aspect_ratio: f32,
    /// Vertical Field of View in radians ($45^\circ = \pi / 4$).
    pub fov_y_rad: f32,
    /// Analytical camera distance along Z axis.
    pub camera_distance: f32,
    /// Active rotation angle around Y axis in radians.
    current_angle_rad: f64,
    /// Current source user ID.
    pub source_user_id: u32,
    /// Target destination user ID.
    pub target_user_id: u32,
}

impl Default for CubeSwitchEngine {
    fn default() -> Self {
        Self::new(16.0 / 9.0, 501, 502)
    }
}

impl CubeSwitchEngine {
    /// Creates a new `CubeSwitchEngine` with given aspect ratio and user IDs.
    #[must_use]
    pub fn new(aspect_ratio: f32, source_user_id: u32, target_user_id: u32) -> Self {
        let fov_deg = 45.0_f32;
        let fov_rad = fov_deg.to_radians();
        // Analytical camera distance: d = 1.0 / tan(fov / 2)
        let half_fov = fov_rad / 2.0;
        let camera_distance = 1.0 / half_fov.tan();

        Self {
            state: CubeSwitchState::Idle,
            aspect_ratio: aspect_ratio.max(0.1),
            fov_y_rad: fov_rad,
            camera_distance,
            current_angle_rad: 0.0,
            source_user_id,
            target_user_id,
        }
    }

    /// Computes the complete 3D Model-View-Projection (MVP) matrix for the active cube angle:
    ///
    /// $$\mathbf{MVP} = \mathbf{P} \cdot \mathbf{V} \cdot \mathbf{R}_y(\theta)$$
    #[must_use]
    pub fn calculate_mvp_matrix(&self) -> Mat4 {
        let projection = Mat4::perspective_rh(
            self.fov_y_rad,
            self.aspect_ratio,
            0.1,    // Near plane
            100.0,  // Far plane
        );

        let eye = Vec3::new(0.0, 0.0, self.camera_distance);
        let target = Vec3::ZERO;
        let up = Vec3::new(0.0, 1.0, 0.0);
        let view = Mat4::look_at_rh(eye, target, up);

        let model = Mat4::from_rotation_y(self.current_angle_rad as f32);

        projection * view * model
    }

    /// Begins an interactive trackpad two-finger drag to rotate the cube.
    pub fn begin_interactive_drag(&mut self, screen_width: f64) {
        self.state = CubeSwitchState::InteractiveDrag {
            current_angle_rad: self.current_angle_rad,
            screen_width: screen_width.max(100.0),
        };
    }

    /// Updates interactive rotation during two-finger pan gesture.
    ///
    /// `delta_x_pixels`: horizontal movement on trackpad.
    pub fn update_interactive_drag(&mut self, delta_x_pixels: f64) {
        if let CubeSwitchState::InteractiveDrag { current_angle_rad, screen_width } = &mut self.state {
            // Dragging full width corresponds to a 90 degree (pi/2) rotation
            let angle_delta = (delta_x_pixels / *screen_width) * (std::f64::consts::PI / 2.0);
            *current_angle_rad = (*current_angle_rad + angle_delta).clamp(
                -std::f64::consts::PI / 2.0,
                std::f64::consts::PI / 2.0,
            );
            self.current_angle_rad = *current_angle_rad;
        }
    }

    /// Concludes the interactive gesture.
    ///
    /// Evaluates the $45^\circ$ ($\pi/4$) commit threshold:
    /// - If $|\theta| \ge \pi/4$: commits rotation to $\pm \pi/2$ (target user).
    /// - If $|\theta| < \pi/4$: snaps back to $0.0$ (original user).
    pub fn end_interactive_drag(&mut self, velocity_x: f64) {
        let half_turn = std::f64::consts::PI / 4.0; // 45 degrees
        let quarter_turn = std::f64::consts::PI / 2.0; // 90 degrees

        let target_angle = if self.current_angle_rad >= half_turn || velocity_x > 500.0 {
            quarter_turn
        } else if self.current_angle_rad <= -half_turn || velocity_x < -500.0 {
            -quarter_turn
        } else {
            0.0
        };

        let target_user = if target_angle.abs() > 0.01 {
            self.target_user_id
        } else {
            self.source_user_id
        };

        let params = SpringParams::mission_control();
        let spring = SpringAnimation::new(
            params,
            self.current_angle_rad,
            velocity_x * 0.001,
            target_angle,
        );

        self.state = CubeSwitchState::SpringAnimating {
            spring,
            target_angle_rad: target_angle,
            target_user_id: target_user,
        };
    }

    /// Programmatically triggers an animated cube switch in the specified direction.
    pub fn trigger_switch(&mut self, direction: CubeRotationDirection) {
        let target_angle = match direction {
            CubeRotationDirection::RotateRight => -std::f64::consts::PI / 2.0,
            CubeRotationDirection::RotateLeft => std::f64::consts::PI / 2.0,
        };

        let params = SpringParams::mission_control();
        let spring = SpringAnimation::new(
            params,
            self.current_angle_rad,
            0.0,
            target_angle,
        );

        self.state = CubeSwitchState::SpringAnimating {
            spring,
            target_angle_rad: target_angle,
            target_user_id: self.target_user_id,
        };
    }

    /// Advances the spring animation simulation clock by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        if let CubeSwitchState::SpringAnimating { spring, target_angle_rad, target_user_id } = &mut self.state {
            let state = spring.step(dt);
            self.current_angle_rad = state.position;

            if spring.is_settled() {
                let target_u = *target_user_id;
                let target_a = *target_angle_rad;
                self.current_angle_rad = target_a;
                self.state = CubeSwitchState::Completed {
                    active_user_id: target_u,
                };
            }
        }
    }

    /// Current rotation angle in radians.
    #[must_use]
    pub fn current_angle_rad(&self) -> f64 {
        self.current_angle_rad
    }

    /// Current operational state.
    #[must_use]
    pub fn state(&self) -> &CubeSwitchState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_distance_calculation() {
        let engine = CubeSwitchEngine::new(16.0 / 9.0, 501, 502);
        // At 45 deg FOV, half angle is 22.5 deg.
        // tan(22.5 deg) = sqrt(2) - 1 ~= 0.41421356
        // d = 1 / 0.41421356 ~= 2.41421356
        assert!((engine.camera_distance - 2.41421356).abs() < 1e-4);
    }

    #[test]
    fn test_mvp_matrix_validity() {
        let engine = CubeSwitchEngine::new(16.0 / 9.0, 501, 502);
        let mvp = engine.calculate_mvp_matrix();
        // Determinant should be non-zero and finite
        assert!(mvp.determinant().is_finite());
        assert!(mvp.determinant().abs() > 1e-5);
    }

    #[test]
    fn test_interactive_drag_commit_threshold() {
        let mut engine = CubeSwitchEngine::new(16.0 / 9.0, 501, 502);
        engine.begin_interactive_drag(1920.0);

        // Drag 600px right (600 / 1920 * pi/2 = 0.3125 * 1.5708 ~= 0.491 rad < 45 deg = 0.785 rad)
        engine.update_interactive_drag(600.0);
        // Release with 0 velocity -> should snap back to 0.0
        engine.end_interactive_drag(0.0);

        match engine.state() {
            CubeSwitchState::SpringAnimating { target_angle_rad, target_user_id, .. } => {
                assert_eq!(*target_angle_rad, 0.0);
                assert_eq!(*target_user_id, 501); // original user
            }
            _ => panic!("Expected SpringAnimating"),
        }
    }

    #[test]
    fn test_interactive_drag_past_threshold_commits() {
        let mut engine = CubeSwitchEngine::new(16.0 / 9.0, 501, 502);
        engine.begin_interactive_drag(1920.0);

        // Drag 1200px (1200 / 1920 * pi/2 = 0.625 * 1.5708 = 0.98 rad > 45 deg = 0.785 rad)
        engine.update_interactive_drag(1200.0);
        engine.end_interactive_drag(0.0);

        match engine.state() {
            CubeSwitchState::SpringAnimating { target_angle_rad, target_user_id, .. } => {
                let expected = std::f64::consts::PI / 2.0;
                assert!((*target_angle_rad - expected).abs() < 1e-4);
                assert_eq!(*target_user_id, 502); // target user committed!
            }
            _ => panic!("Expected SpringAnimating"),
        }

        // Advance spring animation to completion
        for _ in 0..100 {
            engine.tick(0.02);
        }

        match engine.state() {
            CubeSwitchState::Completed { active_user_id } => {
                assert_eq!(*active_user_id, 502);
            }
            _ => panic!("Expected Completed state"),
        }
    }
}
