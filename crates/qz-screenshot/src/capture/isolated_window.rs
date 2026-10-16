//! # Isolated Window Alpha + Dual-Layer Shadow Capture Subsystem
//!
//! Generates pixel-perfect isolated window captures on a transparent background with
//! Apple's dual-layer Gaussian `erf` shadow padding (Ambient $\sigma_1=4\text{px}, \Delta y_1=2\text{px}$ +
//! Deep Penumbra $\sigma_2=42\text{px}, \Delta y_2=24\text{px}$, $+90\text{px}$ quad margin)
//! and $G^2$ Squircle ($p = 2.35$) corner clipping.

/// Dual-layer shadow parameters for window capture framing (Pillar 3.4 & Pillar 12).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowShadowProfile {
    /// Ambient occlusion layer sigma in pixels ($\sigma_1 = 4.0\text{px}$).
    pub ambient_sigma: f32,
    /// Ambient occlusion vertical offset ($\Delta y_1 = 2.0\text{px}$).
    pub ambient_offset_y: f32,
    /// Deep penumbra layer sigma in pixels ($\sigma_2 = 42.0\text{px}$).
    pub deep_sigma: f32,
    /// Deep penumbra vertical offset ($\Delta y_2 = 24.0\text{px}$).
    pub deep_offset_y: f32,
    /// Horizontal padding on left and right in pixels ($+90.0\text{px}$).
    pub margin_x: f32,
    /// Vertical padding on top in pixels ($+90.0\text{px}$).
    pub margin_top: f32,
    /// Vertical padding on bottom in pixels ($+120.0\text{px}$ compensating for $+24\text{px}$ downward shadow cast).
    pub margin_bottom: f32,
}

impl Default for WindowShadowProfile {
    fn default() -> Self {
        Self {
            ambient_sigma: 4.0,
            ambient_offset_y: 2.0,
            deep_sigma: 42.0,
            deep_offset_y: 24.0,
            margin_x: 90.0,
            margin_top: 90.0,
            margin_bottom: 120.0,
        }
    }
}

#[inline]
fn erf_approx(x: f32) -> f32 {
    let s = x.signum();
    let a = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * a);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-a * a).exp();
    s * y
}

#[inline]
fn shadow_box_integral_x(x: f32, s: f32) -> f32 {
    0.5 + 0.5 * erf_approx(x * (0.70710678 / s.max(0.0001)))
}

#[inline]
fn gaussian_box_shadow(p: (f32, f32), half_size: (f32, f32), r: f32, sigma: f32) -> f32 {
    let b = ((half_size.0 - r * 0.35).max(1.0), (half_size.1 - r * 0.35).max(1.0));
    let ix = shadow_box_integral_x(p.0 + b.0, sigma) - shadow_box_integral_x(p.0 - b.0, sigma);
    let iy = shadow_box_integral_x(p.1 + b.1, sigma) - shadow_box_integral_x(p.1 - b.1, sigma);
    (ix * iy).clamp(0.0, 1.0)
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
        let corner_radius = 12.5;

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

    /// Evaluates whether an arbitrary canvas point $(x, y)$ falls within the window $G^2$ Squircle ($p = 2.35$).
    pub fn is_inside_window(&self, x: f32, y: f32) -> bool {
        let local_x = x - self.window_offset_x as f32;
        let local_y = y - self.window_offset_y as f32;

        if local_x < 0.0
            || local_x > self.window_width as f32
            || local_y < 0.0
            || local_y > self.window_height as f32
        {
            return false;
        }

        let w = self.window_width as f32;
        let h = self.window_height as f32;
        let r = self.corner_radius;

        // Check 4 corners with G2 superellipse squircle norm L^2.35 (Anti-Fraud Law #4)
        let in_top_left = local_x < r && local_y < r;
        let in_top_right = local_x > w - r && local_y < r;
        let in_bottom_left = local_x < r && local_y > h - r;
        let in_bottom_right = local_x > w - r && local_y > h - r;

        if in_top_left || in_top_right || in_bottom_left || in_bottom_right {
            let cx = if local_x < r { r } else { w - r };
            let cy = if local_y < r { r } else { h - r };
            let dx = (local_x - cx).abs() / r;
            let dy = (local_y - cy).abs() / r;
            (dx.powf(2.35) + dy.powf(2.35)) <= 1.0
        } else {
            true
        }
    }

    /// Evaluates the exact analytical dual-layer Gaussian `erf` shadow alpha at canvas coordinate $(x, y)$.
    pub fn evaluate_dual_erf_shadow_alpha(
        &self,
        x: f32,
        y: f32,
        is_focused: bool,
        profile: WindowShadowProfile,
    ) -> f32 {
        if !self.has_shadow || self.is_inside_window(x, y) {
            return 0.0;
        }
        let cx = self.window_offset_x as f32 + self.window_width as f32 * 0.5;
        let cy = self.window_offset_y as f32 + self.window_height as f32 * 0.5;
        let half_size = (self.window_width as f32 * 0.5, self.window_height as f32 * 0.5);
        let p = (x - cx, y - cy);

        let ao_alpha = if is_focused { 0.28 } else { 0.16 };
        let ao = gaussian_box_shadow(
            (p.0, p.1 - profile.ambient_offset_y),
            half_size,
            self.corner_radius,
            profile.ambient_sigma,
        ) * ao_alpha;

        let pen_alpha = if is_focused { 0.46 } else { 0.22 };
        let pen = gaussian_box_shadow(
            (p.0, p.1 - profile.deep_offset_y),
            half_size,
            self.corner_radius,
            profile.deep_sigma,
        ) * pen_alpha;

        (ao + pen * (1.0 - ao)).clamp(0.0, 1.0)
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
        assert_eq!(canvas.window_offset_x, 90);
        assert_eq!(canvas.window_offset_y, 90);
        assert_eq!(canvas.total_width, 1000 + 180);
        assert_eq!(canvas.total_height, 700 + 210);
        assert!(canvas.has_shadow);

        // Point in center of window is inside
        assert!(canvas.is_inside_window(90.0 + 500.0, 90.0 + 350.0));

        // Point outside in shadow margin is outside and has positive erf shadow alpha
        assert!(!canvas.is_inside_window(90.0 + 500.0, 90.0 + 705.0));
        let alpha = canvas.evaluate_dual_erf_shadow_alpha(
            90.0 + 500.0,
            90.0 + 705.0,
            true,
            WindowShadowProfile::default(),
        );
        assert!(alpha > 0.15, "Expected analytical erf shadow alpha > 0.15 below window, got {}", alpha);
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
