//! # Qt Window Titlebar & Traffic Lights Decoration Engine (`qt-quartz-theme::decorations`)
//!
//! Production-grade window decorations for Qt5/Qt6 on QuartzLinux.
//!
//! ### Key Capabilities:
//! - **Left-Aligned Traffic Light Coordinates:** Close ($x=12$), Minimize ($x=32$), Maximize ($x=52$)
//!   with 12px diameter buttons.
//! - **Precision Circular Hit-Testing:** Detects mouse clicks within exact circular bounds.
//! - **Centered Title Alignment:** Geometry calculations to place window titles at the optical center.

use serde::{Deserialize, Serialize};

/// Action triggered by interacting with a traffic light button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TitlebarAction {
    /// Request window close.
    Close,
    /// Request window minimization.
    Minimize,
    /// Request window zoom or fullscreen.
    Maximize,
}

/// Titlebar decoration geometry and layout metrics for Qt windows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TitlebarMetrics {
    /// Total height of the titlebar in pixels (standard: 38.0px).
    pub height: f32,
    /// Diameter of traffic light circular buttons (12.0px).
    pub button_diameter: f32,
    /// Left margin offset to the first button (12.0px).
    pub left_margin: f32,
    /// Spacing between consecutive buttons (8.0px).
    pub button_spacing: f32,
}

impl Default for TitlebarMetrics {
    fn default() -> Self {
        Self {
            height: 38.0,
            button_diameter: 12.0,
            left_margin: 12.0,
            button_spacing: 8.0,
        }
    }
}

impl TitlebarMetrics {
    /// Returns the center coordinate $[x, y]$ of the Close button.
    #[must_use]
    pub fn close_button_center(&self) -> (f32, f32) {
        let radius = self.button_diameter * 0.5;
        (self.left_margin + radius, self.height * 0.5)
    }

    /// Returns the center coordinate $[x, y]$ of the Minimize button.
    #[must_use]
    pub fn minimize_button_center(&self) -> (f32, f32) {
        let radius = self.button_diameter * 0.5;
        let x = self.left_margin + self.button_diameter + self.button_spacing + radius;
        (x, self.height * 0.5)
    }

    /// Returns the center coordinate $[x, y]$ of the Maximize button.
    #[must_use]
    pub fn maximize_button_center(&self) -> (f32, f32) {
        let radius = self.button_diameter * 0.5;
        let x = self.left_margin + (self.button_diameter + self.button_spacing) * 2.0 + radius;
        (x, self.height * 0.5)
    }

    /// Evaluates if a cursor point $(x, y)$ hits any of the traffic light buttons.
    #[must_use]
    pub fn hit_test(&self, x: f32, y: f32) -> Option<TitlebarAction> {
        let radius = self.button_diameter * 0.5;
        let r2 = radius * radius;

        let check_hit = |center: (f32, f32)| {
            let dx = x - center.0;
            let dy = y - center.1;
            (dx * dx + dy * dy) <= r2
        };

        if check_hit(self.close_button_center()) {
            Some(TitlebarAction::Close)
        } else if check_hit(self.minimize_button_center()) {
            Some(TitlebarAction::Minimize)
        } else if check_hit(self.maximize_button_center()) {
            Some(TitlebarAction::Maximize)
        } else {
            None
        }
    }

    /// Calculates centered title text rectangle $[x, y, w, h]$ for a window of width $W$.
    #[must_use]
    pub fn title_rect(&self, window_width: f32, title_width: f32) -> (f32, f32, f32, f32) {
        let x = ((window_width - title_width) * 0.5).max(self.left_margin + 64.0);
        (x, 0.0, title_width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_lights_geometry_and_hit_testing() {
        let metrics = TitlebarMetrics::default();

        let (cx, cy) = metrics.close_button_center();
        assert_eq!((cx, cy), (18.0, 19.0));

        // Direct hit on close button center
        assert_eq!(metrics.hit_test(18.0, 19.0), Some(TitlebarAction::Close));

        // Direct hit on minimize button
        let (mx, my) = metrics.minimize_button_center();
        assert_eq!(metrics.hit_test(mx, my), Some(TitlebarAction::Minimize));

        // Direct hit on maximize button
        let (zx, zy) = metrics.maximize_button_center();
        assert_eq!(metrics.hit_test(zx, zy), Some(TitlebarAction::Maximize));

        // Point outside buttons
        assert_eq!(metrics.hit_test(100.0, 19.0), None);
    }

    #[test]
    fn test_title_centering_geometry() {
        let metrics = TitlebarMetrics::default();
        let (x, y, w, h) = metrics.title_rect(1000.0, 200.0);
        assert_eq!(x, 400.0);
        assert_eq!(y, 0.0);
        assert_eq!(w, 200.0);
        assert_eq!(h, 38.0);
    }
}
