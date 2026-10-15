//! # Desktop, Dock & Stage Manager Settings Panel (`qz-settings::panels::dock_desktop`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 Dock, Desktop Stacks,
//! Stage Manager, and Mission Control hot corners.
//!
//! ### Key Capabilities:
//! - **Dock Sizing & Parabolic Magnification:** Real-time sliders for base tile size and hover scale.
//! - **Minimization Effects:** Canonical `GenieEffect` (64x64 non-linear mesh) and `ScaleEffect`.
//! - **Stage Manager:** 3D tilted grouping, recent application thumbnails, and desktop reveal toggles.
//! - **Hot Corners Configuration:** Customizable corner triggers (Mission Control, Desktop, Lock Screen).

use serde::{Deserialize, Serialize};

/// Position of the Dock along the edges of the primary display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockPosition {
    /// Anchored to the bottom screen edge (default).
    Bottom,
    /// Anchored vertically to the left screen edge.
    Left,
    /// Anchored vertically to the right screen edge.
    Right,
}

/// Window minimization transition effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MinimizeEffect {
    /// Canonical Liquid Glass Genie lamp effect with 64x64 non-linear vertex warping.
    GenieEffect,
    /// Standard linear scaling down into dock tile.
    ScaleEffect,
}

/// Window title bar double-click behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TitlebarDoubleClickAction {
    /// Zooms window to fill available workspace without covering Dock.
    Zoom,
    /// Minimizes window into Dock.
    Minimize,
    /// Ignores double-click.
    DoNothing,
}

/// Hot Corner screen boundary action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HotCornerAction {
    /// Dispatches Mission Control 3D workspace overview.
    MissionControl,
    /// Reveals desktop by flying open windows off-screen.
    DesktopReveal,
    /// Opens Quick Note scratchpad.
    QuickNote,
    /// Engages session lock immediately.
    LockScreen,
    /// Puts display to sleep.
    PutDisplayToSleep,
    /// No action assigned.
    None,
}

/// Desktop, Dock and Stage Manager settings configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockDesktopSettings {
    /// Dock icon tile size in pixels (16px to 128px, standard: 64px).
    pub dock_size_px: f64,
    /// Magnification on hover enabled.
    pub magnification_enabled: bool,
    /// Magnified icon size in pixels (16px to 128px, standard: 96px).
    pub magnified_size_px: f64,
    /// Edge position.
    pub position: DockPosition,
    /// Minimization visual effect.
    pub minimize_effect: MinimizeEffect,
    /// Title bar double-click action.
    pub titlebar_double_click: TitlebarDoubleClickAction,
    /// Minimize windows into application icon.
    pub minimize_into_app_icon: bool,
    /// Automatically hide and show the Dock.
    pub autohide_dock: bool,
    /// Animate opening applications (Dock bounce).
    pub animate_opening_apps: bool,
    /// Show dot indicators for running applications.
    pub show_indicators_for_open_apps: bool,
    /// Show recent applications in tripartite right partition.
    pub show_recent_apps: bool,
    /// Stage Manager enabled.
    pub stage_manager_enabled: bool,
    /// Show desktop icons when Stage Manager is active.
    pub stage_manager_show_desktop_items: bool,
    /// Hot corner assignments: [TopLeft, TopRight, BottomLeft, BottomRight].
    pub hot_corners: [HotCornerAction; 4],
}

impl Default for DockDesktopSettings {
    fn default() -> Self {
        Self {
            dock_size_px: 64.0,
            magnification_enabled: true,
            magnified_size_px: 96.0,
            position: DockPosition::Bottom,
            minimize_effect: MinimizeEffect::GenieEffect,
            titlebar_double_click: TitlebarDoubleClickAction::Zoom,
            minimize_into_app_icon: false,
            autohide_dock: false,
            animate_opening_apps: true,
            show_indicators_for_open_apps: true,
            show_recent_apps: true,
            stage_manager_enabled: false,
            stage_manager_show_desktop_items: true,
            hot_corners: [
                HotCornerAction::MissionControl,    // Top-Left
                HotCornerAction::QuickNote,         // Top-Right
                HotCornerAction::None,              // Bottom-Left
                HotCornerAction::LockScreen,        // Bottom-Right
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_defaults() {
        let settings = DockDesktopSettings::default();
        assert_eq!(settings.dock_size_px, 64.0);
        assert_eq!(settings.magnified_size_px, 96.0);
        assert_eq!(settings.position, DockPosition::Bottom);
        assert_eq!(settings.minimize_effect, MinimizeEffect::GenieEffect);
        assert_eq!(settings.hot_corners[0], HotCornerAction::MissionControl);
        assert_eq!(settings.hot_corners[3], HotCornerAction::LockScreen);
    }
}
