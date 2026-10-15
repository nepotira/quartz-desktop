//! # Liquid Glass Refraction & Squircle G2 Pipeline
//!
//! Controls the Tahoe 26 refractive lens profile:
//! - Squircle $G^2$ (Superellipse $L^4$) SDF boundary clipping.
//! - Dual-Kawase backdrop refraction with radial chromatic dispersion.
//! - Rec.709 vibrancy saturation boost ($S = 1.85$).
//! - Color Burn / Dodge vibrancy blend.
//! - 1px light-reactive specular rim highlight.

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
    /// Corner radius in pixels (12.0 for windows/pills, 16.0 for menus/Spotlight).
    pub corner_radius: f32,
    /// Refractive lens border width (default 14.0px).
    pub lens_width: f32,
    /// Maximum refraction displacement (default 6.5px).
    pub refract_px: f32,
    /// 1 = Dark Mode, 0 = Light Mode.
    pub is_dark_mode: i32,
}

impl LiquidGlassUniforms {
    /// Standard Dark Mode tint: `rgba(0.11, 0.11, 0.13, 0.66)`.
    pub const DARK_TINT: Vec4 = Vec4::new(0.11, 0.11, 0.13, 0.66);
    /// Standard Light Mode tint: `rgba(0.95, 0.95, 0.97, 0.70)`.
    pub const LIGHT_TINT: Vec4 = Vec4::new(0.95, 0.95, 0.97, 0.70);

    /// Constructs Tahoe 26 glass profile for a standard window panel.
    pub fn for_window(width: f32, height: f32, screen_width: f32, screen_height: f32, dark_mode: bool) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            screen_res: Vec2::new(screen_width, screen_height),
            tint_color: if dark_mode { Self::DARK_TINT } else { Self::LIGHT_TINT },
            corner_radius: 12.0,
            lens_width: 14.0,
            refract_px: 6.5,
            is_dark_mode: if dark_mode { 1 } else { 0 },
        }
    }

    /// Constructs Tahoe 26 glass profile for a Floating Glass Pill (Sidebar or Toolbar).
    pub fn for_floating_pill(width: f32, height: f32, screen_width: f32, screen_height: f32, dark_mode: bool) -> Self {
        Self {
            half_size: Vec2::new(width * 0.5, height * 0.5),
            screen_res: Vec2::new(screen_width, screen_height),
            tint_color: if dark_mode {
                Vec4::new(0.15, 0.15, 0.18, 0.72)
            } else {
                Vec4::new(0.98, 0.98, 1.00, 0.75)
            },
            corner_radius: 12.0,
            lens_width: 12.0,
            refract_px: 5.5,
            is_dark_mode: if dark_mode { 1 } else { 0 },
        }
    }

    /// Evaluates the CPU analytical Squircle $G^2$ ($L^4$ norm) Signed Distance Field at point `p`.
    ///
    /// Negative values are inside the squircle; positive values are outside.
    pub fn sd_squircle(&self, p: Vec2) -> f32 {
        let b = self.half_size;
        let r = self.corner_radius;
        let q = p.abs() - (b - Vec2::splat(r));
        let m = q.max(Vec2::ZERO);
        let l4 = (m.x.powi(4) + m.y.powi(4)).powf(0.25);
        l4 + q.x.max(q.y).min(0.0) - r
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squircle_sdf_center_and_edges() {
        let uniforms = LiquidGlassUniforms::for_window(800.0, 600.0, 1920.0, 1080.0, true);

        // Center must be strictly inside (negative distance)
        let d_center = uniforms.sd_squircle(Vec2::ZERO);
        assert!(d_center < -200.0);

        // Outside the bounds must be strictly positive
        let d_outside = uniforms.sd_squircle(Vec2::new(500.0, 0.0));
        assert!(d_outside > 0.0);

        // On the edge near (400, 0) should be close to 0.0
        let d_edge = uniforms.sd_squircle(Vec2::new(400.0, 0.0));
        assert!(d_edge.abs() < 1.0);
    }
}
