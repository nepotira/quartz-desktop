//! # GPU Shading and Geometry Pipelines
//!
//! Submodules:
//! - `kawase`: 4-pass downsample + 4-pass upsample Dual-Kawase blur pyramid.
//! - `liquid_glass`: Squircle G2 SDF, refractive chromatic dispersion, and specular rim highlight.
//! - `genie`: 64x64 tessellated mesh and non-linear minimize deformation.
//! - `shadow`: Analytical dual-layer Gaussian erf shadow solver.
//! - `catmull_rom`: 9-tap bicubic Catmull-Rom downsampler for fractional Retina scaling.
//! - `cube_3d`: 3D Cube Fast User Switching perspective transition.

pub mod catmull_rom;
pub mod cube_3d;
pub mod genie;
pub mod kawase;
pub mod liquid_glass;
pub mod shadow;

pub use catmull_rom::CatmullRomUniforms;
pub use cube_3d::{CubeFace, CubeUniforms};
pub use genie::{GenieMesh, GenieUniforms, GenieVertex};
pub use kawase::{KawasePass, KawasePushConstants, KawasePyramid};
pub use liquid_glass::LiquidGlassUniforms;
pub use shadow::ShadowUniforms;
