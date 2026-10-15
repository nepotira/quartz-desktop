//! # Catmull-Rom 9-Tap Bicubic Downsampler Pipeline
//!
//! Provides ultra-crisp fractional Retina downsampling from an integer `@2x`
//! offscreen framebuffer to physical monitor resolutions.

use glam::Vec2;

/// Uniforms matching `catmull_rom.frag` push constants.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatmullRomUniforms {
    /// Dimension of the @2x source texture in pixels.
    pub source_res: Vec2,
    /// Destination physical viewport dimensions.
    pub target_res: Vec2,
}

impl CatmullRomUniforms {
    /// Creates uniforms for downsampling from `(src_w, src_h)` to `(dst_w, dst_h)`.
    pub fn new(src_w: f32, src_h: f32, dst_w: f32, dst_h: f32) -> Self {
        Self {
            source_res: Vec2::new(src_w, src_h),
            target_res: Vec2::new(dst_w, dst_h),
        }
    }

    /// Scaling factor along horizontal and vertical axes.
    pub fn scale_factor(&self) -> Vec2 {
        Vec2::new(
            self.target_res.x / self.source_res.x,
            self.target_res.y / self.source_res.y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_scaling_ratios() {
        // e.g. 2x of 1440p (5120x2880) downscaled to 4K (3840x2160)
        let cr = CatmullRomUniforms::new(5120.0, 2880.0, 3840.0, 2160.0);
        let scale = cr.scale_factor();

        assert!((scale.x - 0.75).abs() < 1e-5);
        assert!((scale.y - 0.75).abs() < 1e-5);
    }
}
