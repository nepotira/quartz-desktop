//! # Zero-Copy DMA-BUF Import Engine
//!
//! Imports Linux `File Descriptors` originating from Wayland client surfaces
//! directly into GPU memory via `VK_EXT_external_memory_dma_buf` and
//! `VK_EXT_image_drm_format_modifier` with zero CPU-to-GPU copies.

use crate::error::RendererError;
use ash::vk;
use std::os::fd::RawFd;

/// Standard Linux DRM FourCC format codes.
pub mod drm_formats {
    /// 32-bit ARGB (8:8:8:8), Little-Endian.
    pub const DRM_FORMAT_ARGB8888: u32 = 0x34325241;
    /// 32-bit XRGB (8:8:8:8), Little-Endian.
    pub const DRM_FORMAT_XRGB8888: u32 = 0x34325258;
    /// 32-bit ABGR (8:8:8:8), Little-Endian.
    pub const DRM_FORMAT_ABGR8888: u32 = 0x34324241;
    /// 32-bit XBGR (8:8:8:8), Little-Endian.
    pub const DRM_FORMAT_XBGR8888: u32 = 0x34324258;
    /// Linear pitch buffer modifier.
    pub const DRM_FORMAT_MOD_LINEAR: u64 = 0x0;
    /// Invalid modifier sentinel.
    pub const DRM_FORMAT_MOD_INVALID: u64 = 0x00ffffffffffffff;
}

/// A single memory plane belonging to a DMA-BUF image.
#[derive(Debug, Clone)]
pub struct DmaBufPlane {
    /// Linux file descriptor for the underlying dmabuf memory.
    pub fd: RawFd,
    /// Byte offset within the dmabuf where this plane begins.
    pub offset: u32,
    /// Row stride in bytes.
    pub stride: u32,
    /// DRM format modifier explicitly governing tiling on this plane.
    pub modifier: u64,
}

/// Metadata describing a multi-planar DMA-BUF imported from a Wayland client.
#[derive(Debug, Clone)]
pub struct DmaBufAttributes {
    /// Width of the image buffer in pixels.
    pub width: u32,
    /// Height of the image buffer in pixels.
    pub height: u32,
    /// Linux DRM FourCC pixel format code.
    pub format: u32,
    /// Master DRM format modifier.
    pub modifier: u64,
    /// Contained memory planes (1 to 4 planes).
    pub planes: Vec<DmaBufPlane>,
}

impl DmaBufAttributes {
    /// Validates dimensional invariants and file descriptor safety.
    pub fn validate(&self) -> Result<(), RendererError> {
        if self.width == 0 || self.height == 0 || self.planes.is_empty() || self.planes.len() > 4 {
            return Err(RendererError::InvalidDimensions {
                width: self.width,
                height: self.height,
                planes: self.planes.len(),
            });
        }

        for plane in &self.planes {
            if plane.fd < 0 {
                return Err(RendererError::DmaBufImport {
                    format: self.format,
                    modifier: self.modifier,
                    details: format!("Negative or invalid file descriptor: {}", plane.fd),
                });
            }
            if plane.stride == 0 {
                return Err(RendererError::DmaBufImport {
                    format: self.format,
                    modifier: self.modifier,
                    details: "Row stride must be strictly positive".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Whether this pixel format carries an active transparent alpha channel.
    #[inline]
    pub fn has_alpha(&self) -> bool {
        matches!(
            self.format,
            drm_formats::DRM_FORMAT_ARGB8888 | drm_formats::DRM_FORMAT_ABGR8888
        )
    }

    /// Maps Linux DRM FourCC format to matching Vulkan 1.3 `vk::Format`.
    pub fn to_vk_format(&self) -> Result<vk::Format, RendererError> {
        match self.format {
            drm_formats::DRM_FORMAT_ARGB8888 => Ok(vk::Format::B8G8R8A8_UNORM),
            drm_formats::DRM_FORMAT_XRGB8888 => Ok(vk::Format::B8G8R8A8_UNORM),
            drm_formats::DRM_FORMAT_ABGR8888 => Ok(vk::Format::R8G8B8A8_UNORM),
            drm_formats::DRM_FORMAT_XBGR8888 => Ok(vk::Format::R8G8B8A8_UNORM),
            unsupported => Err(RendererError::DmaBufImport {
                format: unsupported,
                modifier: self.modifier,
                details: format!("Unsupported DRM FourCC pixel format: {:#x}", unsupported),
            }),
        }
    }

    /// Constructs Vulkan plane layouts for explicit DRM modifier image creation.
    pub fn build_subresource_layouts(&self) -> Vec<vk::SubresourceLayout> {
        self.planes
            .iter()
            .map(|plane| vk::SubresourceLayout {
                offset: plane.offset as vk::DeviceSize,
                size: 0, // Inferred by Vulkan driver from modifier
                row_pitch: plane.stride as vk::DeviceSize,
                array_pitch: 0,
                depth_pitch: 0,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dmabuf_validation() {
        let attrs = DmaBufAttributes {
            width: 1920,
            height: 1080,
            format: drm_formats::DRM_FORMAT_ARGB8888,
            modifier: drm_formats::DRM_FORMAT_MOD_LINEAR,
            planes: vec![DmaBufPlane {
                fd: 3,
                offset: 0,
                stride: 1920 * 4,
                modifier: drm_formats::DRM_FORMAT_MOD_LINEAR,
            }],
        };

        assert!(attrs.validate().is_ok());
        assert!(attrs.has_alpha());
        assert_eq!(attrs.to_vk_format().unwrap(), vk::Format::B8G8R8A8_UNORM);
    }
}
