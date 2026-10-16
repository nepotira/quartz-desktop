//! # Analytical Dual-Layer Gaussian `erf` Shadow Pipeline
//!
//! Evaluates the Apple dual-layer window shadow (Anti-Fraud Law #3 & Pillar 3.4):
//! - Layer 1: Contact Ambient Occlusion ($\sigma_1 = 4.0\text{px}$, $\Delta y_1 = 2.0\text{px}$, $\alpha_{\text{focused}} = 0.30$, $\alpha_{\text{unfocused}} = 0.18$).
//! - Layer 2: Deep Atmospheric Penumbra ($\sigma_2 = 42.0\text{px}$, $\Delta y_2 = 24.0\text{px}$, $\alpha_{\text{focused}} = 0.52$, $\alpha_{\text{unfocused}} = 0.20$).
//! - Rendered on an expanded quad (`+90px` margin in each direction).

use glam::{Vec2, Vec4};

/// Uniforms matching `shadow_sdf.frag` push constants.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowUniforms {
    /// Window half-dimensions in pixels `(width/2, height/2)`.
    pub half_size: Vec2,
    /// Corner radius in pixels.
    pub corner_radius: f32,
    /// `1` = Active/Focused Window, `0` = Inactive Window, `2` = Floating Glass Pill.
    pub is_active: i32,
}

impl ShadowUniforms {
    /// Minimum expanded quad margin around the window in pixels (`+90.0px` per Pillar 1 Law #3).
    pub const EXPANDED_QUAD_MARGIN_PX: f32 = 90.0;

    /// Active window dual-layer shadow configuration.
    pub fn for_active_window(width: f32, height: f32, corner_radius: f32) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            corner_radius,
            is_active: 1,
        }
    }

    /// Inactive window dual-layer shadow configuration.
    pub fn for_inactive_window(width: f32, height: f32, corner_radius: f32) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            corner_radius,
            is_active: 0,
        }
    }

    /// Floating glass pill shadow configuration (`R = 11.0px`).
    pub fn for_floating_pill(width: f32, height: f32, corner_radius: f32) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            corner_radius,
            is_active: 2,
        }
    }

    /// Margin needed around the window rectangle to fully contain the shadow penumbra.
    pub fn shadow_extent(&self) -> f32 {
        match self.is_active {
            1 => (42.0_f32 * 3.0 + 24.0).max(Self::EXPANDED_QUAD_MARGIN_PX), // 150px (>= 90px)
            0 => (42.0_f32 * 2.2 + 24.0).max(Self::EXPANDED_QUAD_MARGIN_PX), // 116.4px (>= 90px)
            2 => 34.0,
            _ => 150.0,
        }
    }

    /// Abramowitz & Stegun polynomial approximation for the Gaussian Error Function `erf(x)` (Pillar 3.4).
    #[inline]
    pub fn erf_approx(x: Vec2) -> Vec2 {
        let s = x.signum();
        let a = x.abs();
        let poly = Vec2::ONE + (Vec2::splat(0.278393) + (Vec2::splat(0.230389) + Vec2::splat(0.078108) * (a * a)) * a) * a;
        let p2 = poly * poly;
        s - s / (p2 * p2)
    }

    /// Horizontal 1D analytical integral of a rounded box shadow at `(x, y)` (Pillar 3.4).
    #[inline]
    pub fn shadow_box_integral_x(x: f32, y: f32, sigma: f32, r: f32, b: Vec2) -> f32 {
        let delta = (b.y - r - y.abs()).min(0.0);
        let curved = b.x - r + (r * r - delta * delta).max(0.0).sqrt();
        let arg = (Vec2::splat(x) + Vec2::new(-curved, curved)) * (0.70710678 / sigma);
        let integral = Vec2::splat(0.5) + Vec2::splat(0.5) * Self::erf_approx(arg);
        integral.y - integral.x
    }

    /// 2D Gaussian box shadow integral via 4-point vertical quadrature (Pillar 3.4).
    #[inline]
    pub fn gaussian_box_shadow(p: Vec2, b: Vec2, r: f32, sigma: f32, dy: f32) -> f32 {
        let q = p - Vec2::new(0.0, dy);
        let low = q.y - b.y;
        let high = q.y + b.y;
        let start = (-3.0 * sigma).clamp(low, high);
        let end = (3.0 * sigma).clamp(low, high);
        let step_y = (end - start) * 0.25;
        let mut y = start + step_y * 0.5;
        let mut sum = 0.0;
        for _ in 0..4 {
            sum += Self::shadow_box_integral_x(q.x, q.y - y, sigma, r, b)
                * (-(y * y) / (2.0 * sigma * sigma)).exp();
            y += step_y;
        }
        sum * step_y / (2.50662827 * sigma)
    }

    /// Evaluates the exact dual-layer shadow RGBA at `local_pos` (Pillar 3.4).
    pub fn eval_window_dual_shadow(&self, local_pos: Vec2) -> Vec4 {
        if self.is_active == 2 {
            let s = Self::gaussian_box_shadow(local_pos, self.half_size, self.corner_radius, 10.0, 4.0) * 0.22;
            return Vec4::new(0.0, 0.0, 0.0, s.clamp(0.0, 1.0));
        }
        let ao_a = if self.is_active == 1 { 0.30 } else { 0.18 };
        let umbra_a = if self.is_active == 1 { 0.52 } else { 0.20 };
        let s1 = Self::gaussian_box_shadow(local_pos, self.half_size, self.corner_radius, 4.0, 2.0) * ao_a;
        let s2 = Self::gaussian_box_shadow(local_pos, self.half_size, self.corner_radius, 42.0, 24.0) * umbra_a;
        Vec4::new(0.0, 0.0, 0.0, (s1 + s2).clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_shadow_extents_and_evaluation() {
        let active = ShadowUniforms::for_active_window(800.0, 600.0, 12.0);
        let inactive = ShadowUniforms::for_inactive_window(800.0, 600.0, 12.0);
        let pill = ShadowUniforms::for_floating_pill(205.0, 500.0, 11.0);

        assert!(active.shadow_extent() >= ShadowUniforms::EXPANDED_QUAD_MARGIN_PX);
        assert!(active.shadow_extent() > inactive.shadow_extent());
        assert!(inactive.shadow_extent() > pill.shadow_extent());

        // Center of active window must have higher shadow density than inactive window
        let c_act = active.eval_window_dual_shadow(Vec2::new(0.0, 20.0));
        let c_inact = inactive.eval_window_dual_shadow(Vec2::new(0.0, 20.0));
        assert!(c_act.w > c_inact.w);
    }
}
