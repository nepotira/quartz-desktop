//! # Window Tiling & Green Button Popover Manager (`qz-compositor::tiling`)
//!
//! Production-grade implementation of macOS Tahoe 26 window tiling options:
//! - **500ms Green Traffic Light Hover**: Triggers the floating liquid glass tiling popover.
//! - **Tahoe 26 8px Margin Tiling**: Windows snapped via tiling maintain an authentic 8px gap
//!   between screen borders and between adjacent tiled windows.
//! - **Supported Tiling Arrangements**: Fullscreen, Left/Right halves, 4 Quadrants, Fill, and Center.

/// Canonical 8.0px margin spacing between tiled windows and display edges in macOS Tahoe 26.
pub const TAHOE_TILING_MARGIN: f64 = 8.0;

/// Default Top Menu Bar height in pixels.
pub const DEFAULT_TOP_BAR_HEIGHT: f64 = 24.0;

/// Tiling arrangement choices available in the Green Button Popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilingOption {
    /// Expand window to occupy entire screen without margins.
    Fullscreen,
    /// Tile window to left half of screen with 8px margins.
    LeftHalf,
    /// Tile window to right half of screen with 8px margins.
    RightHalf,
    /// Tile window to top-left quadrant (1/4 screen) with 8px margins.
    TopLeftQuadrant,
    /// Tile window to top-right quadrant (1/4 screen) with 8px margins.
    TopRightQuadrant,
    /// Tile window to bottom-left quadrant (1/4 screen) with 8px margins.
    BottomLeftQuadrant,
    /// Tile window to bottom-right quadrant (1/4 screen) with 8px margins.
    BottomRightQuadrant,
    /// Fill the usable screen area preserving 8px outer margins.
    Fill,
    /// Center the window on display with at least 8px margins.
    Center,
}

/// Window placement geometry $[x, y, \text{width}, \text{height}]$ resulting from tiling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TilingGeometry {
    /// Left X coordinate in screen space.
    pub x: f64,
    /// Top Y coordinate in screen space.
    pub y: f64,
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
}

/// Computes pixel-perfect tiled window bounds adhering to macOS Tahoe 26 layout metrics.
pub struct TilingLayoutManager {
    screen_width: f64,
    screen_height: f64,
    top_bar_height: f64,
    margin: f64,
}

impl TilingLayoutManager {
    /// Creates a new `TilingLayoutManager` for display dimensions `[screen_width, screen_height]`.
    #[must_use]
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        Self {
            screen_width,
            screen_height,
            top_bar_height: DEFAULT_TOP_BAR_HEIGHT,
            margin: TAHOE_TILING_MARGIN,
        }
    }

    /// Customizes the top menu bar offset (e.g. 32px on Retina/Notch displays).
    #[must_use]
    pub fn with_top_bar_height(mut self, height: f64) -> Self {
        self.top_bar_height = height;
        self
    }

    /// Computes window bounds for a chosen `TilingOption`.
    #[must_use]
    pub fn compute_geometry(&self, option: TilingOption, current_size: (f64, f64)) -> TilingGeometry {
        let sw = self.screen_width;
        let sh = self.screen_height;
        let top = self.top_bar_height;
        let m = self.margin;

        let usable_w = sw - 2.0 * m;
        let usable_h = sh - top - 2.0 * m;

        match option {
            TilingOption::Fullscreen => TilingGeometry {
                x: 0.0,
                y: 0.0,
                width: sw,
                height: sh,
            },
            TilingOption::Fill => TilingGeometry {
                x: m,
                y: top + m,
                width: usable_w,
                height: usable_h,
            },
            TilingOption::LeftHalf => {
                let half_w = (usable_w - m) / 2.0;
                TilingGeometry {
                    x: m,
                    y: top + m,
                    width: half_w,
                    height: usable_h,
                }
            }
            TilingOption::RightHalf => {
                let half_w = (usable_w - m) / 2.0;
                TilingGeometry {
                    x: m + half_w + m,
                    y: top + m,
                    width: half_w,
                    height: usable_h,
                }
            }
            TilingOption::TopLeftQuadrant => {
                let half_w = (usable_w - m) / 2.0;
                let half_h = (usable_h - m) / 2.0;
                TilingGeometry {
                    x: m,
                    y: top + m,
                    width: half_w,
                    height: half_h,
                }
            }
            TilingOption::TopRightQuadrant => {
                let half_w = (usable_w - m) / 2.0;
                let half_h = (usable_h - m) / 2.0;
                TilingGeometry {
                    x: m + half_w + m,
                    y: top + m,
                    width: half_w,
                    height: half_h,
                }
            }
            TilingOption::BottomLeftQuadrant => {
                let half_w = (usable_w - m) / 2.0;
                let half_h = (usable_h - m) / 2.0;
                TilingGeometry {
                    x: m,
                    y: top + m + half_h + m,
                    width: half_w,
                    height: half_h,
                }
            }
            TilingOption::BottomRightQuadrant => {
                let half_w = (usable_w - m) / 2.0;
                let half_h = (usable_h - m) / 2.0;
                TilingGeometry {
                    x: m + half_w + m,
                    y: top + m + half_h + m,
                    width: half_w,
                    height: half_h,
                }
            }
            TilingOption::Center => {
                let w = current_size.0.min(usable_w).max(320.0);
                let h = current_size.1.min(usable_h).max(240.0);
                let x = (sw - w) / 2.0;
                let y = top + (sh - top - h) / 2.0;
                TilingGeometry {
                    x,
                    y,
                    width: w,
                    height: h,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tiling_left_right_halves_8px_margin() {
        let manager = TilingLayoutManager::new(1920.0, 1080.0);
        let left = manager.compute_geometry(TilingOption::LeftHalf, (800.0, 600.0));
        let right = manager.compute_geometry(TilingOption::RightHalf, (800.0, 600.0));

        // Left starts at margin 8px
        assert_eq!(left.x, 8.0);
        assert_eq!(left.y, 32.0); // 24px top bar + 8px margin
        assert_eq!(left.height, 1080.0 - 24.0 - 16.0); // 1040px

        // Usable width: 1920 - 16 = 1904. Half width with middle 8px gap: (1904 - 8) / 2 = 948px
        assert_eq!(left.width, 948.0);
        assert_eq!(right.width, 948.0);

        // Right starts at 8 + 948 + 8 = 964px
        assert_eq!(right.x, 964.0);
        assert_eq!(right.y, 32.0);
    }

    #[test]
    fn test_tiling_quadrants() {
        let manager = TilingLayoutManager::new(1920.0, 1080.0);
        let tl = manager.compute_geometry(TilingOption::TopLeftQuadrant, (500.0, 400.0));
        let tr = manager.compute_geometry(TilingOption::TopRightQuadrant, (500.0, 400.0));
        let bl = manager.compute_geometry(TilingOption::BottomLeftQuadrant, (500.0, 400.0));
        let br = manager.compute_geometry(TilingOption::BottomRightQuadrant, (500.0, 400.0));

        assert_eq!(tl.x, 8.0);
        assert_eq!(tr.x, 964.0);
        assert_eq!(bl.x, 8.0);
        assert_eq!(br.x, 964.0);

        assert_eq!(tl.y, 32.0);
        assert_eq!(tr.y, 32.0);
        // Usable height: 1040. Half height with middle 8px gap: (1040 - 8) / 2 = 516px
        assert_eq!(tl.height, 516.0);
        assert_eq!(bl.y, 32.0 + 516.0 + 8.0); // 556.0
        assert_eq!(br.y, 556.0);
    }

    #[test]
    fn test_tiling_fullscreen_and_fill() {
        let manager = TilingLayoutManager::new(1920.0, 1080.0);
        let fs = manager.compute_geometry(TilingOption::Fullscreen, (600.0, 400.0));
        assert_eq!(fs, TilingGeometry { x: 0.0, y: 0.0, width: 1920.0, height: 1080.0 });

        let fill = manager.compute_geometry(TilingOption::Fill, (600.0, 400.0));
        assert_eq!(fill, TilingGeometry { x: 8.0, y: 32.0, width: 1904.0, height: 1040.0 });
    }
}
