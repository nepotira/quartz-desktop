//! # Isolated Window Alpha + Dual-Layer Shadow Capture Subsystem
//!
//! Generates pixel-perfect isolated window captures on a transparent background with
//! Apple's dual-layer Gaussian erf shadow padding (Ambient $\sigma=4\text{px}$ + Deep $\sigma=38\text{px}$)
//! and Squircle $G^2$ corner clipping.

/// Dual-layer shadow parameters for window capture framing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowShadowProfile {
    /// Ambient occlusion layer sigma in pixels.
    pub ambient_sigma: f32,
    /// Ambient occlusion vertical offset.
    pub ambient_offset_y: f32,
    /// Deep penumbra layer sigma in pixels.
    pub deep_sigma: f32,
    /// Deep penumbra vertical offset.
    pub deep_offset_y: f32,
    /// Horizontal padding on left and right in pixels.
    pub margin_x: f32,
    /// Vertical padding on top in pixels.
    pub margin_top: f32,
    /// Vertical padding on bottom in pixels (compensating for downward shadow cast).
    pub margin_bottom: f32,
}

impl Default for WindowShadowProfile {
    fn default() -> Self {
        Self {
            ambient_sigma: 4.0,
            ambient_offset_y: 2.0,
            deep_sigma: 38.0,
            deep_offset_y: 24.0,
            margin_x: 84.0,
            margin_top: 48.0,
            margin_bottom: 120.0,
        }
    }
}

/// Specifications for allocating and rendering an isolated window capture buffer.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowCaptureCanvas {
    /// Final canvas pixel width including padding.
    pub total_width: u32,
    /// Final canvas pixel height including padding.
    pub total_height: u32,
    /// Top-left X position where the window surface is blitted.
    pub window_offset_x: u32,
    /// Top-left Y position where the window surface is blitted.
    pub window_offset_y: u32,
    /// Window pixel width.
    pub window_width: u32,
    /// Window pixel height.
    pub window_height: u32,
    /// Squircle corner radius in pixels.
    pub corner_radius: f32,
    /// Whether shadow margins are included.
    pub has_shadow: bool,
}

impl WindowCaptureCanvas {
    /// Computes canvas dimensions and offsets given the window size and shadow toggle.
    pub fn compute(
        window_width: u32,
        window_height: u32,
        include_shadow: bool,
        profile: WindowShadowProfile,
    ) -> Self {
        let corner_radius = 18.0;

        if include_shadow {
            let total_width = window_width + (profile.margin_x * 2.0) as u32;
            let total_height = window_height + (profile.margin_top + profile.margin_bottom) as u32;
            let window_offset_x = profile.margin_x as u32;
            let window_offset_y = profile.margin_top as u32;

            Self {
                total_width,
                total_height,
                window_offset_x,
                window_offset_y,
                window_width,
                window_height,
                corner_radius,
                has_shadow: true,
            }
        } else {
            // Option key held: strict tight boundary without shadow margin
            Self {
                total_width: window_width,
                total_height: window_height,
                window_offset_x: 0,
                window_offset_y: 0,
                window_width,
                window_height,
                corner_radius,
                has_shadow: false,
            }
        }
    }

    /// Evaluates whether an arbitrary canvas point $(x, y)$ falls within the window squircle.
    pub fn is_inside_window(&self, x: f32, y: f32) -> bool {
        let local_x = x - self.window_offset_x as f32;
        let local_y = y - self.window_offset_y as f32;

        if local_x < 0.0 || local_x > self.window_width as f32 || local_y < 0.0 || local_y > self.window_height as f32 {
            return false;
        }

        let w = self.window_width as f32;
        let h = self.window_height as f32;
        let r = self.corner_radius;

        // Check 4 corners with superellipse squircle norm L^4
        let in_top_left = local_x < r && local_y < r;
        let in_top_right = local_x > w - r && local_y < r;
        let in_bottom_left = local_x < r && local_y > h - r;
        let in_bottom_right = local_x > w - r && local_y > h - r;

        if in_top_left || in_top_right || in_bottom_left || in_bottom_right {
            let cx = if local_x < r { r } else { w - r };
            let cy = if local_y < r { r } else { h - r };
            let dx = (local_x - cx).abs() / r;
            let dy = (local_y - cy).abs() / r;
            // Squircle equation: dx^4 + dy^4 <= 1.0
            (dx.powi(4) + dy.powi(4)) <= 1.0
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_capture_canvas_with_shadow() {
        let canvas = WindowCaptureCanvas::compute(1000, 700, true, WindowShadowProfile::default());

        assert_eq!(canvas.window_width, 1000);
        assert_eq!(canvas.window_height, 700);
        assert_eq!(canvas.window_offset_x, 84);
        assert_eq!(canvas.window_offset_y, 48);
        assert_eq!(canvas.total_width, 1000 + 168);
        assert_eq!(canvas.total_height, 700 + 168);
        assert!(canvas.has_shadow);

        // Point in center of window is inside
        assert!(canvas.is_inside_window(84.0 + 500.0, 48.0 + 350.0));

        // Point outside in shadow margin is outside
        assert!(!canvas.is_inside_window(10.0, 10.0));
    }

    #[test]
    fn test_window_capture_canvas_shadow_suppressed() {
        let canvas = WindowCaptureCanvas::compute(800, 600, false, WindowShadowProfile::default());

        assert_eq!(canvas.total_width, 800);
        assert_eq!(canvas.total_height, 600);
        assert_eq!(canvas.window_offset_x, 0);
        assert_eq!(canvas.window_offset_y, 0);
        assert!(!canvas.has_shadow);
    }
}
