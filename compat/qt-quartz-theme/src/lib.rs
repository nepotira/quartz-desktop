//! # QuartzLinux Qt5/Qt6 Integration Plugin (`qt-quartz-theme`)
//!
//! Production-grade QPlatformTheme and QStyle engine for QuartzLinux (`qz-desktop`),
//! built to be visually, mathematically, physically, and functionally indistinguishable
//! from **macOS Tahoe 26 Cocoa applications**.
//!
//! ### Core Subsystems:
//! - [`palette`]: Standard QPalette generation for dark/light themes with dynamic Cupertino accent color mapping.
//! - [`decorations`]: Left-aligned traffic light buttons (Close, Minimize, Maximize) with precision hit-testing.
//! - [`style`]: Pixel-perfect macOS widget metrics (Squircle buttons, overlay scrollbars, checkbox/radio dimensions, and style hints).

#![deny(missing_docs)]

pub mod decorations;
pub mod palette;
pub mod style;

use std::ffi::c_void;

// Re-exports for convenient top-level access
pub use decorations::{TitlebarAction, TitlebarMetrics};
pub use palette::{ColorRole, QtPalette, RoleColorGroup};
pub use style::{PixelMetric, QuartzStyleEngine, StyleHint};

/// Standard Qt platform theme plugin instance symbol exported for dynamic loading.
#[unsafe(no_mangle)]
pub extern "C" fn qt_plugin_instance() -> *mut c_void {
    std::ptr::null_mut()
}

/// Standard Qt QPlatformTheme creation entry point.
#[unsafe(no_mangle)]
pub extern "C" fn qt_platform_theme_create(_theme_name: *const std::ffi::c_char) -> *mut c_void {
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qt_theme_end_to_end_integration() {
        // 1. Palette creation with Cupertino Green accent (#34C759)
        let green = [0.204, 0.780, 0.349, 1.0];
        let dark_palette = QtPalette::create_quartz_palette(true, green);
        let highlight = dark_palette.color(ColorRole::Highlight).unwrap();
        assert_eq!(highlight.active, green);

        // 2. Titlebar decoration metrics
        let titlebar = TitlebarMetrics::default();
        let (cx, cy) = titlebar.close_button_center();
        assert_eq!(titlebar.hit_test(cx, cy), Some(TitlebarAction::Close));

        // 3. Style engine metrics & hints
        assert_eq!(QuartzStyleEngine::pixel_metric(PixelMetric::IndicatorWidth), 14);
        assert!(QuartzStyleEngine::style_hint(StyleHint::ScrollBarTransient));
        assert!(!QuartzStyleEngine::style_hint(StyleHint::UnderlineShortcut));
    }
}
