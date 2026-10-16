//! # Liquid Glass Refraction & Squircle $G^2$ ($p = 2.35$) Pipeline
//!
//! Controls the Tahoe 26 refractive lens profile (Pillar 3.2 & 3.3):
//! - Squircle $G^2$ (Superellipse $L_p$ with $p = 2.35$) SDF boundary clipping.
//! - Edge lens refraction over the outer `12.0px` (`5.5px` optical displacement).
//! - 5-Level Dual-Kawase backdrop + 32-tap Vogel Disk sampling with radial chromatic dispersion.
//! - Rec.709 Luma vibrancy saturation boost ($S = 1.95\times$).
//! - Dark Mode (`rgba(30, 30, 34, 0.52)`) ColorBurn and Light Mode (`rgba(246, 246, 248, 0.68)`) ColorDodge.
//! - `1.0px` inner specular rim highlight + `0.5px` outer hairline.

use glam::{Vec2, Vec4};

/// Uniform payload matching `liquid_glass_tahoe.frag` push constants.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiquidGlassUniforms {
    /// Half-dimensions in pixels `(width/2, height/2)`.
    pub half_size: Vec2,
    /// Physical screen resolution `(width, height)`.
    pub screen_res: Vec2,
    /// Tint color in RGBA.
    pub tint_color: Vec4,
    /// Corner radius in pixels (`11.0` for Floating Glass Pill Sidebar, `12.0` for windows).
    pub corner_radius: f32,
    /// Refractive lens border width (`12.0px`).
    pub lens_width: f32,
    /// Maximum refraction displacement (`5.5px`).
    pub refract_px: f32,
    /// `1` = Dark Mode, `0` = Light Mode.
    pub is_dark_mode: i32,
}

impl LiquidGlassUniforms {
    /// Standard Dark Mode tint: `rgba(30/255, 30/255, 34/255, 0.52)`.
    pub const DARK_TINT: Vec4 = Vec4::new(30.0 / 255.0, 30.0 / 255.0, 34.0 / 255.0, 0.52);
    /// Standard Light Mode tint: `rgba(246/255, 246/255, 248/255, 0.68)`.
    pub const LIGHT_TINT: Vec4 = Vec4::new(246.0 / 255.0, 246.0 / 255.0, 248.0 / 255.0, 0.68);
    /// Rec.709 Luma saturation boost factor ($1.95\times$).
    pub const SATURATION_BOOST: f32 = 1.95;

    /// Constructs Tahoe 26 glass profile for a standard window panel.
    pub fn for_window(width: f32, height: f32, screen_width: f32, screen_height: f32, dark_mode: bool) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            screen_res: Vec2::new(screen_width, screen_height),
            tint_color: if dark_mode { Self::DARK_TINT } else { Self::LIGHT_TINT },
            corner_radius: 12.0,
            lens_width: 12.0,
            refract_px: 5.5,
            is_dark_mode: if dark_mode { 1 } else { 0 },
        }
    }

    /// Constructs Tahoe 26 glass profile for a Floating Glass Pill Sidebar (`R = 11.0px`, `10px` inset).
    pub fn for_floating_pill(width: f32, height: f32, screen_width: f32, screen_height: f32, dark_mode: bool) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            screen_res: Vec2::new(screen_width, screen_height),
            tint_color: if dark_mode { Self::DARK_TINT } else { Self::LIGHT_TINT },
            corner_radius: 11.0,
            lens_width: 12.0,
            refract_px: 5.5,
            is_dark_mode: if dark_mode { 1 } else { 0 },
        }
    }

    /// Evaluates the CPU analytical Squircle $G^2$ ($L_p$ norm with $p = 2.35$) Signed Distance Field at point `p`.
    ///
    /// Negative values are inside the squircle; positive values are outside.
    pub fn sd_squircle(&self, p: Vec2) -> f32 {
        Self::sd_squircle_g2(p, self.half_size, self.corner_radius)
    }

    /// Static evaluation of `sdSquircleG2(p, half_size, r)` with $p = 2.35$ (Pillar 3.2).
    #[inline]
    pub fn sd_squircle_g2(p: Vec2, half_size: Vec2, r: f32) -> f32 {
        let q = p.abs() - (half_size - Vec2::splat(r));
        let m = q.max(Vec2::ZERO);
        let lp = (m.x.powf(2.35) + m.y.powf(2.35)).powf(1.0 / 2.35);
        lp + q.x.max(q.y).min(0.0) - r
    }

    /// Evaluates the 2D normal gradient `sdSquircleNormal(p, half_size, r)` (Pillar 3.2).
    #[inline]
    pub fn sd_squircle_normal(p: Vec2, half_size: Vec2, r: f32) -> Vec2 {
        let eps = 0.5_f32;
        let dx = Self::sd_squircle_g2(p + Vec2::new(eps, 0.0), half_size, r)
            - Self::sd_squircle_g2(p - Vec2::new(eps, 0.0), half_size, r);
        let dy = Self::sd_squircle_g2(p + Vec2::new(0.0, eps), half_size, r)
            - Self::sd_squircle_g2(p - Vec2::new(0.0, eps), half_size, r);
        (Vec2::new(dx, dy) + Vec2::splat(1e-6)).normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squircle_g2_p235_sdf_and_normal() {
        let uniforms = LiquidGlassUniforms::for_floating_pill(205.0, 540.0, 1920.0, 1080.0, true);
        assert_eq!(uniforms.corner_radius, 11.0);

        let d_center = uniforms.sd_squircle(Vec2::ZERO);
        assert!(d_center < -90.0);

        let d_outside = uniforms.sd_squircle(Vec2::new(300.0, 0.0));
        assert!(d_outside > 0.0);

        let n_right = LiquidGlassUniforms::sd_squircle_normal(
            Vec2::new(102.5, 0.0),
            uniforms.half_size,
            uniforms.corner_radius,
        );
        assert!(n_right.x > 0.95);
    }
}
