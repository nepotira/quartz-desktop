//! # qz-compositor
//!
//! Wayland Core Compositor engine for QuartzLinux (macOS Tahoe 26 architecture).
//! Implements DMA-BUF Zero-Copy pipelines, ProMotion VRR frame pacing, Retina bicubic
//! scaling, and smart macOS interaction models (Keymapper, Click-Through, Window Tiling).

#![deny(missing_docs)]

pub mod input;

pub use input::{ClickThroughDispatcher, SmartKeymapper};
