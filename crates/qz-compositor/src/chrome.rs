//! # Window Chrome, Traffic Lights, Proxy Icon & 8-Direction Resize (`qz-compositor::chrome`)
//!
//! Implements Pillar 6 of the macOS Tahoe 26 specification:
//! - **Traffic Lights**:
//!   - Red Close: `#FF5F56` (border `#E0443E`, glyph `#4D0000`), dark dirty dot (`•`) when `dirty_state == true`.
//!   - Yellow Minimize: `#FFBD2E` (border `#DEA123`, glyph `#995700`), `Option + Click` minimizes all app windows.
//!   - Green Zoom/Fullscreen: `#27C93F` (border `#1AAB29`, glyph `#006500`), 3 modes:
//!     1. Simple Click: Fullscreen in a new Space.
//!     2. `Option + Click`: Glyph `+`, Classic Zoom within usable desktop area.
//!     3. `0.5s` Hover: Opens Liquid Glass Window Tiling Popover (`8px` margins).
//!   - Inactive state `rgba(128, 128, 128, 0.24)` lighting up and revealing all 3 glyphs simultaneously on group hover.
//! - **Proxy Icon & Path Hierarchy Dropdown**:
//!   - Draggable Proxy Icon beside window title; `Cmd + Click` (or Right-Click) on title opens parent directory chain up to `/` (`Macintosh HD`).
//! - **8-Direction Resize**:
//!   - `Option + Drag`: Symmetric resize around window center.
//!   - `Shift + Drag`: Proportional aspect-ratio locked resize.

use std::path::{Path, PathBuf};

/// RGB/RGBA color specification for a Traffic Light button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficLightPalette {
    /// Fill RGB `[r, g, b]` in $0..=255$.
    pub fill_rgb: [u8; 3],
    /// 1px border RGB `[r, g, b]` in $0..=255$.
    pub border_rgb: [u8; 3],
    /// Internal glyph RGB `[r, g, b]` in $0..=255$.
    pub glyph_rgb: [u8; 3],
}

impl TrafficLightPalette {
    /// Red Close button: fill `#FF5F56`, border `#E0443E`, glyph `#4D0000`.
    pub const CLOSE_RED: Self = Self {
        fill_rgb: [0xFF, 0x5F, 0x56],
        border_rgb: [0xE0, 0x44, 0x3E],
        glyph_rgb: [0x4D, 0x00, 0x00],
    };

    /// Yellow Minimize button: fill `#FFBD2E`, border `#DEA123`, glyph `#995700`.
    pub const MINIMIZE_YELLOW: Self = Self {
        fill_rgb: [0xFF, 0xBD, 0x2E],
        border_rgb: [0xDE, 0xA1, 0x23],
        glyph_rgb: [0x99, 0x57, 0x00],
    };

    /// Green Fullscreen/Zoom button: fill `#27C93F`, border `#1AAB29`, glyph `#006500`.
    pub const ZOOM_GREEN: Self = Self {
        fill_rgb: [0x27, 0xC9, 0x3F],
        border_rgb: [0x1A, 0xAB, 0x29],
        glyph_rgb: [0x00, 0x65, 0x00],
    };

    /// Inactive monochromatic state: `rgba(128, 128, 128, 0.24)`.
    pub const INACTIVE_RGBA: [f32; 4] = [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 0.24];
}

/// Action triggered by interacting with the Traffic Light buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLightAction {
    /// Close the current window (`Cmd+W`).
    CloseWindow,
    /// Minimize the current window into the Dock (`Cmd+M`).
    MinimizeWindow,
    /// `Option + Click` on Yellow button: minimize all windows belonging to this application (`Opt+Cmd+M`).
    MinimizeAllAppWindows,
    /// Simple click on Green button: enter Fullscreen in a dedicated Space.
    EnterFullscreenSpace,
    /// `Option + Click` on Green button (`+` glyph): Classic Zoom fitting usable desktop area.
    ClassicZoomFit,
    /// Hovering Green button for $\ge 0.5\text{s}$: open the Liquid Glass Window Tiling Popover.
    ShowTilingPopover,
}

/// Glyph mode displayed inside the Green button when the traffic light group is hovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreenButtonGlyph {
    /// Diagonal fullscreen arrows (`⤢`).
    FullscreenArrows,
    /// Classic zoom plus sign (`+`) when `Option` is held.
    ClassicZoomPlus,
}

/// State controller for a window's Traffic Light group.
#[derive(Debug, Clone, PartialEq)]
pub struct TrafficLightController {
    /// Whether the window has unsaved changes (`•` dot inside the Red button).
    pub dirty_state: bool,
    /// Whether the window is currently focused.
    pub is_window_focused: bool,
    /// Whether the pointer is hovering over the 3-button bounding box.
    pub is_group_hovered: bool,
    /// Whether the pointer is specifically hovering the Green button.
    pub is_green_hovered: bool,
    /// Accumulated hover duration on the Green button in seconds.
    pub green_hover_elapsed: f64,
    /// Whether the Liquid Glass Window Tiling Popover is currently open.
    pub is_tiling_popover_open: bool,
}

impl Default for TrafficLightController {
    fn default() -> Self {
        Self::new()
    }
}

impl TrafficLightController {
    /// Required hover duration on the Green button to summon the Tiling Popover ($0.5\text{s}$).
    pub const TILING_HOVER_DELAY_SECS: f64 = 0.5;

    /// Creates a new focused Traffic Light controller.
    pub fn new() -> Self {
        Self {
            dirty_state: false,
            is_window_focused: true,
            is_group_hovered: false,
            is_green_hovered: false,
            green_hover_elapsed: 0.0,
            is_tiling_popover_open: false,
        }
    }

    /// True if the 3 buttons should render in full color (either window is focused OR group is hovered).
    #[inline]
    pub fn should_render_colored(&self) -> bool {
        self.is_window_focused || self.is_group_hovered
    }

    /// True if all 3 internal glyphs should be revealed simultaneously.
    #[inline]
    pub fn should_reveal_all_glyphs(&self) -> bool {
        self.is_group_hovered
    }

    /// Resolves the Green button glyph based on whether `Option` is held.
    #[inline]
    pub fn green_glyph(&self, option_held: bool) -> GreenButtonGlyph {
        if option_held {
            GreenButtonGlyph::ClassicZoomPlus
        } else {
            GreenButtonGlyph::FullscreenArrows
        }
    }

    /// Advances the Green button hover timer by `dt` seconds; opens Tiling Popover at $0.5\text{s}$.
    pub fn tick_hover(&mut self, dt: f64) -> Option<TrafficLightAction> {
        if self.is_green_hovered {
            let prev = self.green_hover_elapsed;
            self.green_hover_elapsed += dt.max(0.0);
            if prev < Self::TILING_HOVER_DELAY_SECS
                && self.green_hover_elapsed >= Self::TILING_HOVER_DELAY_SECS
            {
                self.is_tiling_popover_open = true;
                return Some(TrafficLightAction::ShowTilingPopover);
            }
        } else {
            self.green_hover_elapsed = 0.0;
        }
        None
    }

    /// Resolves click on Yellow button (`option_held = true` $\to$ `MinimizeAllAppWindows`).
    #[inline]
    pub fn click_yellow(&self, option_held: bool) -> TrafficLightAction {
        if option_held {
            TrafficLightAction::MinimizeAllAppWindows
        } else {
            TrafficLightAction::MinimizeWindow
        }
    }

    /// Resolves click on Green button (`option_held = true` $\to$ `ClassicZoomFit`).
    #[inline]
    pub fn click_green(&self, option_held: bool) -> TrafficLightAction {
        if option_held {
            TrafficLightAction::ClassicZoomFit
        } else {
            TrafficLightAction::EnterFullscreenSpace
        }
    }
}

/// Entry in the Titlebar Path Hierarchy Dropdown (`Cmd + Click` on window title).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitlePathEntry {
    /// Display name (e.g. `"Documents"`, `"nepo"`, `"Users"`, `"Macintosh HD"`).
    pub display_name: String,
    /// Full filesystem path.
    pub path: PathBuf,
}

/// Builds the parent directory hierarchy from `current_path` up to `/` (`Macintosh HD`),
/// triggered when the user `Cmd + Clicks` or Right-Clicks the window title alongside the Proxy Icon.
pub fn build_titlebar_path_hierarchy(current_path: &Path) -> Vec<TitlePathEntry> {
    let mut entries = Vec::new();
    let mut cursor = Some(current_path);

    while let Some(p) = cursor {
        let name = match p.file_name().and_then(|s| s.to_str()) {
            Some(n) if !n.is_empty() => n.to_string(),
            _ => "Macintosh HD".to_string(),
        };
        entries.push(TitlePathEntry {
            display_name: name,
            path: p.to_path_buf(),
        });
        cursor = p.parent();
        if cursor == Some(p) {
            break;
        }
    }

    if entries.is_empty() {
        entries.push(TitlePathEntry {
            display_name: "Macintosh HD".to_string(),
            path: PathBuf::from("/"),
        });
    }
    entries
}

/// 8-Direction Window Resize Edge/Corner Handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeEdge {
    /// North (top edge).
    North,
    /// South (bottom edge).
    South,
    /// East (right edge).
    East,
    /// West (left edge).
    West,
    /// North-East (top-right corner).
    NorthEast,
    /// North-West (top-left corner).
    NorthWest,
    /// South-East (bottom-right corner).
    SouthEast,
    /// South-West (bottom-left corner).
    SouthWest,
}

/// Computes 8-direction window resize geometry supporting:
/// - `option_symmetric`: `Option + Drag` resizes symmetrically from the window center.
/// - `shift_aspect_lock`: `Shift + Drag` preserves the initial aspect ratio $W_0 / H_0$.
pub fn compute_resized_bounds(
    initial_rect: (f64, f64, f64, f64),
    edge: ResizeEdge,
    delta: (f64, f64),
    option_symmetric: bool,
    shift_aspect_lock: bool,
    min_size: (f64, f64),
) -> (f64, f64, f64, f64) {
    let (mut x, mut y, mut w, mut h) = initial_rect;
    let aspect = (w / h.max(1.0)).max(0.1);
    let (dx, dy) = delta;

    let sym_mult = if option_symmetric { 2.0 } else { 1.0 };

    match edge {
        ResizeEdge::East => {
            w = (w + dx * sym_mult).max(min_size.0);
            if shift_aspect_lock {
                h = (w / aspect).max(min_size.1);
            }
        }
        ResizeEdge::West => {
            let new_w = (w - dx * sym_mult).max(min_size.0);
            if !option_symmetric {
                x += w - new_w;
            }
            w = new_w;
            if shift_aspect_lock {
                h = (w / aspect).max(min_size.1);
            }
        }
        ResizeEdge::South => {
            h = (h + dy * sym_mult).max(min_size.1);
            if shift_aspect_lock {
                w = (h * aspect).max(min_size.0);
            }
        }
        ResizeEdge::North => {
            let new_h = (h - dy * sym_mult).max(min_size.1);
            if !option_symmetric {
                y += h - new_h;
            }
            h = new_h;
            if shift_aspect_lock {
                w = (h * aspect).max(min_size.0);
            }
        }
        ResizeEdge::SouthEast => {
            w = (w + dx * sym_mult).max(min_size.0);
            h = if shift_aspect_lock {
                (w / aspect).max(min_size.1)
            } else {
                (h + dy * sym_mult).max(min_size.1)
            };
        }
        ResizeEdge::NorthWest => {
            let new_w = (w - dx * sym_mult).max(min_size.0);
            let new_h = if shift_aspect_lock {
                (new_w / aspect).max(min_size.1)
            } else {
                (h - dy * sym_mult).max(min_size.1)
            };
            if !option_symmetric {
                x += w - new_w;
                y += h - new_h;
            }
            w = new_w;
            h = new_h;
        }
        ResizeEdge::NorthEast => {
            w = (w + dx * sym_mult).max(min_size.0);
            let new_h = if shift_aspect_lock {
                (w / aspect).max(min_size.1)
            } else {
                (h - dy * sym_mult).max(min_size.1)
            };
            if !option_symmetric {
                y += h - new_h;
            }
            h = new_h;
        }
        ResizeEdge::SouthWest => {
            let new_w = (w - dx * sym_mult).max(min_size.0);
            if !option_symmetric {
                x += w - new_w;
            }
            w = new_w;
            h = if shift_aspect_lock {
                (w / aspect).max(min_size.1)
            } else {
                (h + dy * sym_mult).max(min_size.1)
            };
        }
    }

    if option_symmetric {
        let cx = initial_rect.0 + initial_rect.2 * 0.5;
        let cy = initial_rect.1 + initial_rect.3 * 0.5;
        x = cx - w * 0.5;
        y = cy - h * 0.5;
    }

    (x, y, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_lights_hover_and_option_modifiers() {
        let mut tl = TrafficLightController::new();
        tl.is_window_focused = false;
        assert!(!tl.should_render_colored());

        tl.is_group_hovered = true;
        tl.is_green_hovered = true;
        assert!(tl.should_render_colored());
        assert!(tl.should_reveal_all_glyphs());
        assert_eq!(tl.green_glyph(false), GreenButtonGlyph::FullscreenArrows);
        assert_eq!(tl.green_glyph(true), GreenButtonGlyph::ClassicZoomPlus);

        // Option + Click yellow -> MinimizeAllAppWindows
        assert_eq!(tl.click_yellow(true), TrafficLightAction::MinimizeAllAppWindows);
        assert_eq!(tl.click_green(true), TrafficLightAction::ClassicZoomFit);

        // 0.5s hover on green opens tiling popover
        assert_eq!(tl.tick_hover(0.3), None);
        assert_eq!(tl.tick_hover(0.25), Some(TrafficLightAction::ShowTilingPopover));
        assert!(tl.is_tiling_popover_open);
    }

    #[test]
    fn test_titlebar_path_hierarchy_to_macintosh_hd() {
        let chain = build_titlebar_path_hierarchy(Path::new("/Users/nepo/Documents/Projects"));
        assert_eq!(chain[0].display_name, "Projects");
        assert_eq!(chain[1].display_name, "Documents");
        assert_eq!(chain[2].display_name, "nepo");
        assert_eq!(chain[3].display_name, "Users");
        assert_eq!(chain[4].display_name, "Macintosh HD");
        assert_eq!(chain[4].path, PathBuf::from("/"));
    }

    #[test]
    fn test_8_direction_resize_option_symmetric_and_shift_aspect() {
        let initial = (100.0, 100.0, 800.0, 400.0); // 2:1 aspect, center = (500, 300)
        let sym = compute_resized_bounds(initial, ResizeEdge::SouthEast, (50.0, 30.0), true, true, (200.0, 100.0));
        // Width increases by 50 * 2 = 100 -> 900. Aspect 2:1 -> Height = 450.
        assert!((sym.2 - 900.0).abs() < 1e-5);
        assert!((sym.3 - 450.0).abs() < 1e-5);
        // Center preserved at (500, 300)
        assert!((sym.0 + sym.2 * 0.5 - 500.0).abs() < 1e-5);
        assert!((sym.1 + sym.3 * 0.5 - 300.0).abs() < 1e-5);
    }
}
