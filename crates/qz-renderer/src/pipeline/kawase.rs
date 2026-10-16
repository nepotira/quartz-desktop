//! # 5-Level Dual-Kawase Blur Pyramid Pipeline ($\sigma_{\text{eq}} \ge 48\text{px}$)
//!
//! Implements a 5-pass downsampling (`1/2, 1/4, 1/8, 1/16, 1/32`) and 5-pass upsampling
//! (`1/16, 1/8, 1/4, 1/2, 1/1`) chain across 5 offscreen `FBO_Kawase[0..4]` framebuffers.
//!
//! Guarantees $\sigma_{\text{eq}} \ge 48\text{px}$ so that no diagonal line or sharp wallpaper
//! shape can ever bleed through Liquid Glass surfaces (Anti-Fraud Law #2 & Pillar 3.1).

use glam::Vec2;

/// Push constant payload for Dual-Kawase shader passes.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KawasePushConstants {
    /// Half-pixel coordinate step: `0.5 / texture_size`.
    pub half_pixel: Vec2,
    /// Kernel expansion offset.
    pub offset: f32,
}

/// Description of a single pass within the 5-level Kawase blur pyramid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KawasePass {
    /// Pass index (`0..=4`).
    pub pass_index: usize,
    /// Destination framebuffer width.
    pub width: u32,
    /// Destination framebuffer height.
    pub height: u32,
    /// Sampling offset for this pass.
    pub offset: f32,
}

/// Multi-stage 5-Level Dual-Kawase Blur Pyramid solver (`FBO_Kawase[0..4]`).
pub struct KawasePyramid {
    /// Display physical width.
    pub base_width: u32,
    /// Display physical height.
    pub base_height: u32,
    /// Downsample pass chain (5 passes: `1/2, 1/4, 1/8, 1/16, 1/32`).
    pub down_passes: [KawasePass; 5],
    /// Upsample pass chain (5 passes: `1/16, 1/8, 1/4, 1/2, 1/1`).
    pub up_passes: [KawasePass; 5],
}

impl KawasePyramid {
    /// Number of pyramid levels required by Pillar 1 Law #2 and Pillar 3.1.
    pub const NUM_LEVELS: usize = 5;
    /// Expansion offsets for 5-pass downsampling achieving $\sigma_{\text{eq}} \ge 48\text{px}$.
    pub const DOWN_OFFSETS: [f32; 5] = [1.5, 2.0, 2.5, 3.0, 3.5];
    /// Reconstruction offsets for 5-pass upsampling.
    pub const UP_OFFSETS: [f32; 5] = [3.5, 3.0, 2.5, 2.0, 1.5];

    /// Builds a 5-level pyramid configuration tailored for base screen dimensions `(width, height)`.
    pub fn new(width: u32, height: u32) -> Self {
        let w = width.max(64);
        let h = height.max(64);

        let d0 = KawasePass { pass_index: 0, width: w / 2,  height: h / 2,  offset: Self::DOWN_OFFSETS[0] };
        let d1 = KawasePass { pass_index: 1, width: w / 4,  height: h / 4,  offset: Self::DOWN_OFFSETS[1] };
        let d2 = KawasePass { pass_index: 2, width: w / 8,  height: h / 8,  offset: Self::DOWN_OFFSETS[2] };
        let d3 = KawasePass { pass_index: 3, width: w / 16, height: h / 16, offset: Self::DOWN_OFFSETS[3] };
        let d4 = KawasePass { pass_index: 4, width: w / 32, height: h / 32, offset: Self::DOWN_OFFSETS[4] };

        let u0 = KawasePass { pass_index: 0, width: w / 16, height: h / 16, offset: Self::UP_OFFSETS[0] };
        let u1 = KawasePass { pass_index: 1, width: w / 8,  height: h / 8,  offset: Self::UP_OFFSETS[1] };
        let u2 = KawasePass { pass_index: 2, width: w / 4,  height: h / 4,  offset: Self::UP_OFFSETS[2] };
        let u3 = KawasePass { pass_index: 3, width: w / 2,  height: h / 2,  offset: Self::UP_OFFSETS[3] };
        let u4 = KawasePass { pass_index: 4, width: w,      height: h,      offset: Self::UP_OFFSETS[4] };

        Self {
            base_width: w,
            base_height: h,
            down_passes: [d0, d1, d2, d3, d4],
            up_passes: [u0, u1, u2, u3, u4],
        }
    }

    /// Computes push constant parameters for a given pass.
    pub fn push_constants(&self, pass: &KawasePass) -> KawasePushConstants {
        KawasePushConstants {
            half_pixel: Vec2::new(0.5 / pass.width as f32, 0.5 / pass.height as f32),
            offset: pass.offset,
        }
    }

    /// Returns the effective Gaussian standard deviation $\sigma_{\text{eq}}$ in pixels ($\ge 48.0\text{px}$).
    pub fn effective_sigma_px(&self) -> f32 {
        let mut variance = 0.0_f32;
        for (i, pass) in self.down_passes.iter().enumerate() {
            let scale = (1_u32 << (i + 1)) as f32;
            let step_sigma = pass.offset * scale * 0.85;
            variance += step_sigma * step_sigma;
        }
        variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kawase_5_level_pyramid_and_sigma_48px() {
        let pyramid = KawasePyramid::new(3840, 2160);

        assert_eq!(pyramid.down_passes.len(), 5);
        assert_eq!(pyramid.up_passes.len(), 5);

        assert_eq!(pyramid.down_passes[0].width, 1920);
        assert_eq!(pyramid.down_passes[1].width, 960);
        assert_eq!(pyramid.down_passes[2].width, 480);
        assert_eq!(pyramid.down_passes[3].width, 240);
        assert_eq!(pyramid.down_passes[4].width, 120);
        assert_eq!(pyramid.down_passes[4].height, 67);

        assert_eq!(pyramid.up_passes[4].width, 3840);
        assert_eq!(pyramid.up_passes[4].height, 2160);

        // Must guarantee sigma_eq >= 48px per Anti-Fraud Law #2
        assert!(pyramid.effective_sigma_px() >= 48.0);
    }
}
