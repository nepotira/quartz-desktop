//! # QuartzLinux Screenshot & Screen Recording Subsystem (`qz-screenshot`)
//!
//! Complete, production-grade screenshot capture and screen recording engine
//! for macOS Tahoe 26 on Wayland:
//!
//! 1. **Capture Shortcuts (`capture`)**:
//!    - `Cmd+Shift+3`: Instant full-screen capture of active or all displays.
//!    - `Cmd+Shift+4`: Rectangular crosshair selection with Shift aspect lock and Option center expansion.
//!      Tapping `Space` transforms crosshair into Camera Mode for window capture.
//!    - **Isolated Window Alpha + Dual-Layer Shadow**: Captures target window on transparent background
//!      with Apple's analytical Gaussian erf shadow (Ambient $\sigma=4\text{px}$, Deep $\sigma=38\text{px}$)
//!      and Squircle $G^2$ corner radius. Holding `Option` suppresses shadow capture.
//!
//! 2. **Interactive HUD Toolbar (`hud`)**:
//!    - `Cmd+Shift+5`: Liquid Glass toolbar with 5 operational segments:
//!      Capture Screen, Capture Window, Capture Portion, Record Screen, Record Portion.
//!    - Options popover: Save destination (Desktop, Documents, Clipboard, Mail, Messages),
//!      5s/10s timer delay with audible countdown, microphone audio toggle, and mouse click ripples.
//!
//! 3. **Floating Thumbnail Subsystem (`thumbnail`)**:
//!    - Post-capture thumbnail sliding into the bottom-right corner via spring physics (`qz_animation`).
//!    - 5-second countdown timer paused on mouse hover.
//!    - Swipe right to immediately commit to disk, or drag & drop directly into other applications.
//!
//! 4. **PipeWire Screen Recorder Daemon (`recorder`)**:
//!    - Zero-copy DMA-BUF video stream encoding (HEVC/H.264/AV1 at 60fps/120fps ProMotion).
//!    - System audio loopback recording with optional microphone commentary.
//!    - Top Menu Bar recording capsule indicator with elapsed time counter and `Cmd+Ctrl+Esc` stop shortcut.
//!
//! 5. **Screenshot Controller (`controller`)**:
//!    - Master event dispatcher coordinating shortcuts, captures, HUD, thumbnails, and recordings.

#![deny(missing_docs)]

pub mod capture;
pub mod controller;
pub mod hud;
pub mod recorder;
pub mod thumbnail;

// Re-exports
pub use capture::{
    CaptureMode, CrosshairSelection, WindowCaptureCanvas, WindowShadowProfile,
};
pub use controller::{ScreenshotController, ScreenshotEvent};
pub use hud::{
    HudOperation, HudOptions, SaveDestination, ScreenshotHudController, TimerDelay,
};
pub use recorder::{
    RecorderConfig, RecorderError, RecordingState, RecordingTarget, ScreenRecorderDaemon,
};
pub use thumbnail::{FloatingThumbnail, ThumbnailLifecycle};
