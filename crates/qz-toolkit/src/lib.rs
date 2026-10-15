//! # QuartzLinux AquaUI Framework (`qz-toolkit`)
//!
//! Production-grade declarative UI toolkit implementing the visual, physical,
//! and typographic design language of macOS Tahoe 26.
//!
//! ### Subsystems Included:
//! 1. **Floating Glass Pills & Squircle Widgets ([`widgets`]):**
//!    - $G^2$ continuous superellipse bounding curves ($n = 4.0$).
//!    - Liquid Glass optical material (refraction $\eta = 1.52$, dispersion $\Delta \eta = 0.015$,
//!      Dual-Kawase blur $\sigma = 20.0\text{px}$).
//!    - Spring-driven micro-interactions (GlassButton, GlassSegmentedControl, GlassToggle).
//! 2. **Modal Sheets ([`sheets`]):**
//!    - Titlebar drop-down anchoring with parent scrim dimming overlay ($0.0 \to 0.35$).
//!    - Interactive drag-to-dismiss gesture with Cocoa rubber-band resistance and $120\text{px}$ threshold.
//! 3. **Native Tab Bar ([`tabs`]):**
//!    - Flocking drag-to-reorder physics with spring-driven neighbor displacement.
//!    - Unsaved modifications dirty state (bullet dot `•` morphing to `x` on hover).
//!    - Adaptive width distribution and sliding glass indicator pill.
//! 4. **Cocoa Text Engine ([`text`]):**
//!    - CoreText-style typography with quarter-pixel subpixel horizontal positioning.
//!    - Adaptive stem darkening adapting to background surface luminance ($L \in [0.0, 1.0]$).
//!    - Smooth cosine wave caret blinking.
//! 5. **Declarative Layout Engine ([`layout`]):**
//!    - `VStack` and `HStack` flex engines with cross-axis alignments and edge insets.

#![deny(missing_docs)]

pub mod layout;
pub mod sheets;
pub mod tabs;
pub mod text;
pub mod widgets;

pub use layout::{
    EdgeInsets, HStackLayout, HorizontalAlignment, PlacementRect, VStackLayout,
    VerticalAlignment,
};
pub use sheets::{ModalSheet, SheetAction, SheetState};
pub use tabs::{NativeTabBar, TabDragState, TabItem};
pub use text::{CocoaTextEngine, FontMetrics, LayoutLine, TextClippingMode};
pub use widgets::{
    GlassButton, GlassSegmentedControl, GlassToggle, InteractionState, LiquidGlassMaterial,
    Squircle,
};

use glam::Vec4;
use serde::{Deserialize, Serialize};

/// System appearance mode for AquaUI themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppearanceMode {
    /// Light Aqua appearance.
    Light,
    /// Dark Aqua appearance with reduced eye fatigue.
    Dark,
}

/// Global theme styling parameters for AquaUI components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AquaTheme {
    /// Current appearance mode.
    pub appearance: AppearanceMode,
    /// Accent color tint (e.g. Cupertino Blue, Purple, Graphite).
    pub accent_color: Vec4,
    /// Window background vibrancy level ($0.0..=1.0$).
    pub vibrancy_level: f32,
    /// Whether high-contrast mode is active for accessibility.
    pub high_contrast: bool,
}

impl Default for AquaTheme {
    fn default() -> Self {
        Self {
            appearance: AppearanceMode::Dark,
            // macOS Tahoe Canonical Cupertino Accent Blue
            accent_color: Vec4::new(0.0, 0.478, 1.0, 1.0),
            vibrancy_level: 0.85,
            high_contrast: false,
        }
    }
}

impl AquaTheme {
    /// Creates a light appearance theme.
    #[must_use]
    pub fn light() -> Self {
        Self {
            appearance: AppearanceMode::Light,
            accent_color: Vec4::new(0.0, 0.478, 1.0, 1.0),
            vibrancy_level: 0.75,
            high_contrast: false,
        }
    }

    /// Background luminance estimate for the current theme appearance.
    #[must_use]
    pub fn surface_luminance(&self) -> f64 {
        match self.appearance {
            AppearanceMode::Light => 0.92,
            AppearanceMode::Dark => 0.12,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_luminance_estimates() {
        let dark_theme = AquaTheme::default();
        assert_eq!(dark_theme.appearance, AppearanceMode::Dark);
        assert!(dark_theme.surface_luminance() < 0.5);

        let light_theme = AquaTheme::light();
        assert_eq!(light_theme.appearance, AppearanceMode::Light);
        assert!(light_theme.surface_luminance() > 0.5);
    }

    #[test]
    fn test_framework_re_exports() {
        // Ensure exported types instantiate cleanly
        let squircle = Squircle::new(0.0, 0.0, 100.0, 40.0, 8.0);
        assert_eq!(squircle.corner_radius, 8.0);

        let vstack = VStackLayout::default();
        assert_eq!(vstack.spacing, 8.0);
    }
}
