//! # qz-renderer
//!
//! Production-grade Vulkan 1.3 / GLES 3.2 graphics rendering engine for QuartzLinux (Tahoe 26):
//! - **Liquid Glass Refraction**: Squircle $G^2$ ($L^4$) SDF lens profile, chromatic dispersion,
//!   Rec.709 saturation boost, and dynamic specular rim highlight.
//! - **Dual-Kawase Blur Pyramid**: 4 downsample + 4 upsample offscreen passes across 1/2 to 1/16 FBOs.
//! - **Genie Minimize Effect**: $64 \times 64$ quadrilateral mesh deformed with non-linear Hermite curves.
//! - **Analytical Dual-Layer Shadows**: Continuous Gaussian erf approximation integrating rounded box penumbras.
//! - **Retina Fractional Scaling**: 9-tap bicubic Catmull-Rom downsampling from integer @2x framebuffers.
//! - **Zero-Copy DMA-BUF Import**: Direct Wayland buffer binding via `VK_EXT_external_memory_dma_buf`.
//! - **Fast User Switching 3D Cube**: Perspective transformation with spring physics rotation.

#![deny(missing_docs)]

pub mod dmabuf;
pub mod error;
pub mod pipeline;

pub use dmabuf::{drm_formats, DmaBufAttributes, DmaBufPlane};
pub use error::RendererError;
pub use pipeline::{
    CatmullRomUniforms, CubeFace, CubeUniforms, GenieMesh, GenieUniforms, GenieVertex,
    KawasePass, KawasePushConstants, KawasePyramid, LiquidGlassUniforms, ShadowUniforms,
};

/// Embedded GLSL 450 shader source programs.
pub mod shaders {
    /// Dual-Kawase blur downsample fragment shader.
    pub const KAWASE_DOWN_FRAG: &str = include_str!("../shaders/kawase_down.frag");
    /// Dual-Kawase blur upsample fragment shader.
    pub const KAWASE_UP_FRAG: &str = include_str!("../shaders/kawase_up.frag");
    /// macOS Tahoe 26 Liquid Glass Refraction and Squircle G2 fragment shader.
    pub const LIQUID_GLASS_TAHOE_FRAG: &str = include_str!("../shaders/liquid_glass_tahoe.frag");
    /// Genie effect 64x64 mesh vertex shader.
    pub const GENIE_VERT: &str = include_str!("../shaders/genie.vert");
    /// Genie effect texture sampling fragment shader.
    pub const GENIE_FRAG: &str = include_str!("../shaders/genie.frag");
    /// Analytical Dual-Layer Gaussian erf shadow fragment shader.
    pub const SHADOW_SDF_FRAG: &str = include_str!("../shaders/shadow_sdf.frag");
    /// Catmull-Rom 9-tap bicubic downsampler for Retina fractional display scaling.
    pub const CATMULL_ROM_FRAG: &str = include_str!("../shaders/catmull_rom.frag");
    /// 3D Cube Fast User Switching vertex shader.
    pub const CUBE_3D_VERT: &str = include_str!("../shaders/cube_3d.vert");
    /// 3D Cube Fast User Switching fragment shader.
    pub const CUBE_3D_FRAG: &str = include_str!("../shaders/cube_3d.frag");
    /// macOS Tahoe 26 Procedural SDF UI Uber Shader (Zero External Fonts/Icons).
    pub const UI_UBER_SHADER_FRAG: &str = include_str!("../shaders/ui_uber_shader.frag");
}
