//! # Analytical Dual-Layer Gaussian erf Shadow Pipeline
//!
//! Evaluates the Apple dual-layer window shadow:
//! - Layer 1: Ambient Occlusion ($AO$, $\sigma = 4.0\text{px}$, $y_{\text{offset}} = 2.0\text{px}$).
//! - Layer 2: Deep Penumbra ($\sigma = 38.0\text{px}$, $y_{\text{offset}} = 22.0\text{px}$).
//! - Floating Glass Pill preset: $\sigma = 10.0\text{px}$, $y_{\text{offset}} = 4.0\text{px}$.

use glam::Vec2;

/// Uniforms matching `shadow_sdf.frag` push constants.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowUniforms {
    /// Window half-dimensions in pixels `(width/2, height/2)`.
    pub half_size: Vec2,
    /// Corner radius in pixels.
    pub corner_radius: f32,
    /// 1 = Active Window, 0 = Inactive Window, 2 = Floating Glass Pill.
    pub is_active: i32,
}

impl ShadowUniforms {
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

    /// Floating glass pill shadow configuration.
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
            1 => 38.0 * 3.0 + 22.0, // 3 * sigma + y_offset = 136px
            0 => 30.0 * 3.0 + 14.0, // 104px
            2 => 10.0 * 3.0 + 4.0,  // 34px
            _ => 136.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_extents() {
        let active = ShadowUniforms::for_active_window(800.0, 600.0, 12.0);
        let inactive = ShadowUniforms::for_inactive_window(800.0, 600.0, 12.0);
        let pill = ShadowUniforms::for_floating_pill(240.0, 500.0, 12.0);

        assert!(active.shadow_extent() > inactive.shadow_extent());
        assert!(inactive.shadow_extent() > pill.shadow_extent());
    }
}
