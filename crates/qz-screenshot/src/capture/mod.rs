//! # Capture Subsystem Module
//!
//! Capture modes (Full display, Rectangular selection, Isolated window) and Alpha+Shadow compositor.

pub mod isolated_window;
pub mod mode;

pub use isolated_window::{WindowCaptureCanvas, WindowShadowProfile};
pub use mode::{CaptureMode, CrosshairSelection};
