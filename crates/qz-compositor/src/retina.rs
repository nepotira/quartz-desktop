//! # Retina Scaling and Fractional DPI Engine
//!
//! Implements macOS Retina scaling semantics over Wayland protocols:
//! - Computes physical DPI from DRM EDID physical millimeter dimensions and pixel resolution.
//! - Quantizes scale factors to exact multiples of $1/120$ per `wp-fractional-scale-v1`.
//! - Bridges legacy XWayland clients by forcing `@2x` integer rendering (`Xft.dpi: 192`)
//!   combined with 9-tap Catmull-Rom bicubic downsampling to target screen resolution.

use glam::Vec2;

/// Standard baseline non-Retina display density (110 DPI).
pub const BASELINE_DESKTOP_DPI: f64 = 110.0;
/// Standard 2x Retina display density (220 DPI).
pub const RETINA_2X_DPI: f64 = 220.0;

/// Physical monitor specifications obtained from DRM connector EDID.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonitorPhysicalSpecs {
    /// Horizontal pixel resolution.
    pub width_pixels: u32,
    /// Vertical pixel resolution.
    pub height_pixels: u32,
    /// Horizontal physical panel dimension in millimeters.
    pub width_mm: u32,
    /// Vertical physical panel dimension in millimeters.
    pub height_mm: u32,
}

impl MonitorPhysicalSpecs {
    /// Computes diagonal screen size in inches.
    pub fn diagonal_inches(&self) -> f64 {
        let w_in = self.width_mm as f64 / 25.4;
        let h_in = self.height_mm as f64 / 25.4;
        (w_in * w_in + h_in * h_in).sqrt()
    }

    /// Computes physical dots-per-inch (DPI).
    pub fn physical_dpi(&self) -> f64 {
        if self.width_mm == 0 || self.height_mm == 0 {
            return BASELINE_DESKTOP_DPI;
        }
        let diag_pixels = ((self.width_pixels as f64).powi(2) + (self.height_pixels as f64).powi(2)).sqrt();
        let diag_inches = self.diagonal_inches();
        if diag_inches <= 0.0 {
            BASELINE_DESKTOP_DPI
        } else {
            diag_pixels / diag_inches
        }
    }

    /// Computes ideal continuous scale factor relative to standard 110 DPI.
    pub fn continuous_scale(&self) -> f64 {
        let dpi = self.physical_dpi();
        (dpi / BASELINE_DESKTOP_DPI).clamp(1.0, 3.0)
    }

    /// Computes scale quantized to multiples of $1/120$ as required by `wp-fractional-scale-v1`.
    ///
    /// Returns the integer numerator $N$ where $\text{scale} = N / 120$.
    pub fn preferred_scale_120th(&self) -> u32 {
        let raw = self.continuous_scale();
        (raw * 120.0).round() as u32
    }

    /// Computes the exact floating-point quantized scale factor ($N / 120.0$).
    pub fn quantized_scale(&self) -> f64 {
        self.preferred_scale_120th() as f64 / 120.0
    }
}

/// Scaling profile for a given window client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientScaleMode {
    /// Wayland client supporting native fractional scale via `wp-fractional-scale-v1`.
    NativeFractional {
        /// Scale numerator in 120ths.
        scale_120th: u32,
        /// Floating-point scale factor.
        scale: f64,
    },
    /// Legacy XWayland client rendered at @2x integer scale into offscreen FBO.
    LegacyXWaylandBicubic {
        /// Forcing integer 2.0x scale (192 DPI).
        render_scale: f64,
        /// Catmull-Rom downsampling ratio from 2x offscreen buffer to physical target.
        downsample_factor: Vec2,
    },
}

/// Retina display coordinator managing fractional scaling and bicubic bridge.
pub struct RetinaCoordinator;

impl RetinaCoordinator {
    /// Resolves the optimal scale mode for a client surface on a given monitor.
    pub fn resolve_client_scale(
        specs: &MonitorPhysicalSpecs,
        supports_fractional_scale: bool,
    ) -> ClientScaleMode {
        let scale_120th = specs.preferred_scale_120th();
        let quantized = specs.quantized_scale();

        if supports_fractional_scale {
            ClientScaleMode::NativeFractional {
                scale_120th,
                scale: quantized,
            }
        } else {
            // Force 2x integer rendering and apply Catmull-Rom 9-tap downsampling
            let render_scale = 2.0;
            let target_w = (specs.width_pixels as f64 / quantized) as f32;
            let target_h = (specs.height_pixels as f64 / quantized) as f32;
            let src_w = target_w * 2.0;
            let src_h = target_h * 2.0;

            let downsample_factor = Vec2::new(
                target_w / src_w,
                target_h / src_h,
            );

            ClientScaleMode::LegacyXWaylandBicubic {
                render_scale,
                downsample_factor,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_studio_display_5k_scaling() {
        // 27" 5K Apple Studio Display: 5120x2880, 597mm x 336mm (~218 DPI)
        let specs = MonitorPhysicalSpecs {
            width_pixels: 5120,
            height_pixels: 2880,
            width_mm: 597,
            height_mm: 336,
        };

        let dpi = specs.physical_dpi();
        assert!((dpi - 217.5).abs() < 2.0);

        // Continuous scale should be near 2.0x
        let scale = specs.quantized_scale();
        assert!((scale - 2.0).abs() < 0.1);

        let scale_120th = specs.preferred_scale_120th();
        // 2.0 * 120 = 240
        assert!((scale_120th as i32 - 240).abs() <= 5);
    }

    #[test]
    fn test_4k_27_inch_fractional_scaling() {
        // 27" 4K Monitor: 3840x2160, 597mm x 336mm (~163 DPI)
        let specs = MonitorPhysicalSpecs {
            width_pixels: 3840,
            height_pixels: 2160,
            width_mm: 597,
            height_mm: 336,
        };

        let scale_120th = specs.preferred_scale_120th();
        // 163 / 110 = ~1.48 -> round(1.48 * 120) = ~178
        assert!(scale_120th >= 170 && scale_120th <= 185);

        let mode = RetinaCoordinator::resolve_client_scale(&specs, true);
        match mode {
            ClientScaleMode::NativeFractional { scale_120th: s, .. } => {
                assert_eq!(s, scale_120th);
            }
            _ => panic!("Expected NativeFractional mode"),
        }
    }
}
