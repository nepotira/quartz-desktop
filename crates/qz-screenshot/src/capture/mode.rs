//! # Screenshot Capture Mode Subsystem
//!
//! Handles shortcut invocation (`Cmd+Shift+3`, `Cmd+Shift+4`, `Cmd+Shift+5`),
//! crosshair area selection, Spacebar camera mode toggle, and Option shadow suppression.

use glam::Vec2;

/// Primary capture mode classification.
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureMode {
    /// Full display capture of active or all screens (`Cmd+Shift+3`).
    FullDisplay {
        /// Display index or connector identifier.
        display_id: u32,
    },
    /// Rectangular portion selection via interactive crosshair (`Cmd+Shift+4`).
    RectangularArea {
        /// Top-left anchor $[x_0, y_0]$.
        start_point: Vec2,
        /// Current drag point $[x_1, y_1]$.
        current_point: Vec2,
        /// Constrain to square aspect ratio (holding Shift).
        is_aspect_locked: bool,
        /// Expand selection from center outward (holding Option).
        is_center_expanded: bool,
    },
    /// Isolated window capture with camera icon (`Cmd+Shift+4` + `Space`).
    IsolatedWindow {
        /// Target window compositor ID.
        window_id: u32,
        /// Whether Apple dual-layer Gaussian erf shadow is included (suppressed on Option).
        include_shadow: bool,
    },
    /// Interactive HUD toolbar (`Cmd+Shift+5`).
    InteractiveHud,
}

/// Crosshair selection interactive state machine.
pub struct CrosshairSelection {
    start_point: Option<Vec2>,
    current_point: Option<Vec2>,
    is_camera_mode: bool,
    hovered_window_id: Option<u32>,
    is_option_down: bool,
    is_shift_down: bool,
}

impl Default for CrosshairSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl CrosshairSelection {
    /// Creates a new crosshair selection state.
    pub fn new() -> Self {
        Self {
            start_point: None,
            current_point: None,
            is_camera_mode: false,
            hovered_window_id: None,
            is_option_down: false,
            is_shift_down: false,
        }
    }

    /// Toggles between Crosshair rectangular drag and Camera window capture on Spacebar tap.
    pub fn toggle_camera_mode(&mut self) -> bool {
        if self.start_point.is_none() {
            self.is_camera_mode = !self.is_camera_mode;
            self.is_camera_mode
        } else {
            false
        }
    }

    /// Whether camera window selection mode is currently active.
    #[inline]
    pub fn is_camera_mode(&self) -> bool {
        self.is_camera_mode
    }

    /// Updates Option modifier state (suppresses window shadow in camera mode, expands center in rect mode).
    pub fn set_option_down(&mut self, is_down: bool) {
        self.is_option_down = is_down;
    }

    /// Updates Shift modifier state (locks aspect ratio).
    pub fn set_shift_down(&mut self, is_down: bool) {
        self.is_shift_down = is_down;
    }

    /// Mouse pressed down, starting selection rectangle.
    pub fn mouse_down(&mut self, pos: Vec2) {
        if !self.is_camera_mode {
            self.start_point = Some(pos);
            self.current_point = Some(pos);
        }
    }

    /// Mouse dragged to new position.
    pub fn mouse_move(&mut self, pos: Vec2, window_under_cursor: Option<u32>) {
        if self.is_camera_mode {
            self.hovered_window_id = window_under_cursor;
        } else if self.start_point.is_some() {
            self.current_point = Some(pos);
        }
    }

    /// Mouse released, concluding capture selection.
    pub fn mouse_up(&mut self) -> Option<CaptureMode> {
        if self.is_camera_mode {
            let res = self.hovered_window_id.map(|win_id| CaptureMode::IsolatedWindow {
                window_id: win_id,
                include_shadow: !self.is_option_down, // Option key suppresses shadow
            });
            if res.is_some() {
                self.reset();
            }
            res
        } else if let (Some(start), Some(curr)) = (self.start_point, self.current_point) {
            let mode = CaptureMode::RectangularArea {
                start_point: start,
                current_point: curr,
                is_aspect_locked: self.is_shift_down,
                is_center_expanded: self.is_option_down,
            };
            self.reset();
            Some(mode)
        } else {
            None
        }
    }

    /// Computes the normalized bounding rectangle $[x, y, width, height]$.
    pub fn computed_rect(&self) -> Option<[f32; 4]> {
        if let (Some(start), Some(curr)) = (self.start_point, self.current_point) {
            let mut x0 = start.x;
            let mut y0 = start.y;
            let mut x1 = curr.x;
            let mut y1 = curr.y;

            if self.is_option_down {
                let dx = (x1 - x0).abs();
                let dy = (y1 - y0).abs();
                x0 -= dx;
                y0 -= dy;
                x1 += dx;
                y1 += dy;
            }

            let min_x = x0.min(x1);
            let min_y = y0.min(y1);
            let mut width = (x0 - x1).abs();
            let mut height = (y0 - y1).abs();

            if self.is_shift_down {
                let max_dim = width.max(height);
                width = max_dim;
                height = max_dim;
            }

            Some([min_x, min_y, width, height])
        } else {
            None
        }
    }

    /// Resets all selection points.
    pub fn reset(&mut self) {
        self.start_point = None;
        self.current_point = None;
        self.is_camera_mode = false;
        self.hovered_window_id = None;
        self.is_option_down = false;
        self.is_shift_down = false;
    }

    /// Target window currently highlighted in camera mode.
    #[inline]
    pub fn hovered_window_id(&self) -> Option<u32> {
        self.hovered_window_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crosshair_camera_toggle_and_shadow_suppression() {
        let mut selection = CrosshairSelection::new();
        assert!(!selection.is_camera_mode());

        // Tap Space -> switch to Camera mode
        assert!(selection.toggle_camera_mode());
        assert!(selection.is_camera_mode());

        // Hover over window 42
        selection.mouse_move(Vec2::new(100.0, 100.0), Some(42));
        assert_eq!(selection.hovered_window_id(), Some(42));

        // Click without Option -> captures window WITH shadow
        let capture = selection.mouse_up().unwrap();
        assert_eq!(
            capture,
            CaptureMode::IsolatedWindow {
                window_id: 42,
                include_shadow: true,
            }
        );

        // Hover again and hold Option -> captures window WITHOUT shadow
        selection.toggle_camera_mode();
        selection.set_option_down(true);
        selection.mouse_move(Vec2::new(100.0, 100.0), Some(42));
        let capture_no_shadow = selection.mouse_up().unwrap();
        assert_eq!(
            capture_no_shadow,
            CaptureMode::IsolatedWindow {
                window_id: 42,
                include_shadow: false,
            }
        );
    }

    #[test]
    fn test_rectangular_drag_selection_and_shift_lock() {
        let mut selection = CrosshairSelection::new();

        selection.mouse_down(Vec2::new(10.0, 20.0));
        selection.set_shift_down(true); // 1:1 aspect ratio
        selection.mouse_move(Vec2::new(110.0, 70.0), None);

        let rect = selection.computed_rect().unwrap();
        assert_eq!(rect[0], 10.0);
        assert_eq!(rect[1], 20.0);
        assert_eq!(rect[2], 100.0); // max(100, 50) = 100
        assert_eq!(rect[3], 100.0); // locked aspect square
    }
}
