//! # Renderer Error Definitions
//!
//! Error types emitted across Vulkan 1.3, OpenGL ES 3.2, and shader execution.

use thiserror::Error;

/// Errors produced during rendering, memory allocation, or DMA-BUF importing.
#[derive(Debug, Error)]
pub enum RendererError {
    /// Vulkan instance or device initialization failed.
    #[error("Vulkan initialization error: {0}")]
    VulkanInit(String),

    /// Missing required Vulkan extension.
    #[error("Missing required Vulkan extension: {0}")]
    MissingExtension(&'static str),

    /// DMA-BUF file descriptor import error.
    #[error("DMA-BUF import failed for format {format:#x}, modifier {modifier:#x}: {details}")]
    DmaBufImport {
        /// FourCC pixel format code.
        format: u32,
        /// DRM format modifier.
        modifier: u64,
        /// Diagnostic message.
        details: String,
    },

    /// Invalid DMA-BUF buffer dimensions or plane layout.
    #[error("Invalid buffer dimensions: {width}x{height}, plane count: {planes}")]
    InvalidDimensions {
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// Number of memory planes.
        planes: usize,
    },

    /// Shader bytecode or compilation error.
    #[error("Shader error in stage '{stage}': {details}")]
    Shader {
        /// Stage name (vertex, fragment).
        stage: &'static str,
        /// Error description.
        details: String,
    },

    /// Framebuffer or render pass configuration failure.
    #[error("Framebuffer creation failed: {0}")]
    Framebuffer(String),

    /// GPU memory allocation failure.
    #[error("GPU Out of Memory: {0}")]
    OutOfMemory(String),

    /// General internal pipeline error.
    #[error("Pipeline error: {0}")]
    Pipeline(String),
}
