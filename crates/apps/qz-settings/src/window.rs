//! # Settings Window Controller (`qz-settings::window`)
//!
//! Production-grade main window controller for macOS Tahoe 26 System Settings (`qz-settings`).
//!
//! ### Key Capabilities:
//! - **Two-Pane Liquid Glass Layout:** Left floating sidebar ($260\text{px}$) and right scrollable panel.
//! - **Back/Forward Navigation Stack:** Preserves history of visited settings panels with arrow navigation.
//! - **Unified Configuration Store:** Coordinates settings state across all subsystem panels.

use crate::panels::{
    AppearanceSettings, BatterySettings, DisplaysSettings, DockDesktopSettings, GeneralSettings,
    KeyboardTrackpadSettings, SoundSettings,
};
use crate::sidebar::{SettingsCategory, SettingsSidebar};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Main System Settings Window Controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWindow {
    /// Window top-left position $[x, y]$.
    pub position: Vec2,
    /// Window dimensions $[W, H]$ (standard: $960 \times 680\text{px}$).
    pub size: Vec2,
    /// Minimum allowed dimensions ($780 \times 540\text{px}$).
    pub min_size: Vec2,
    /// Sidebar navigation and search index.
    pub sidebar: SettingsSidebar,
    /// Navigation history stack.
    history_stack: Vec<SettingsCategory>,
    /// Active index within history stack.
    history_index: usize,
    /// Appearance panel configuration.
    pub appearance: AppearanceSettings,
    /// Displays & ProMotion panel configuration.
    pub displays: DisplaysSettings,
    /// Desktop, Dock & Stage Manager panel configuration.
    pub dock_desktop: DockDesktopSettings,
    /// Sound & CoreAudio panel configuration.
    pub sound: SoundSettings,
    /// Battery & Energy panel configuration.
    pub battery: BatterySettings,
    /// Keyboard & Trackpad panel configuration.
    pub keyboard_trackpad: KeyboardTrackpadSettings,
    /// General & About This Mac configuration.
    pub general: GeneralSettings,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self::new(Vec2::new(120.0, 80.0), Vec2::new(960.0, 680.0))
    }
}

impl SettingsWindow {
    /// Creates a new `SettingsWindow`.
    #[must_use]
    pub fn new(position: Vec2, size: Vec2) -> Self {
        let min_size = Vec2::new(780.0, 540.0);
        let clamped_size = Vec2::new(size.x.max(min_size.x), size.y.max(min_size.y));

        let initial_category = SettingsCategory::Appearance;
        let mut sidebar = SettingsSidebar::default();
        sidebar.select_category(initial_category);

        Self {
            position,
            size: clamped_size,
            min_size,
            sidebar,
            history_stack: vec![initial_category],
            history_index: 0,
            appearance: AppearanceSettings::default(),
            displays: DisplaysSettings::default(),
            dock_desktop: DockDesktopSettings::default(),
            sound: SoundSettings::default(),
            battery: BatterySettings::default(),
            keyboard_trackpad: KeyboardTrackpadSettings::default(),
            general: GeneralSettings::default(),
        }
    }

    /// Navigates to a settings category and records it in history.
    pub fn navigate_to(&mut self, category: SettingsCategory) {
        if self.sidebar.active_category == category {
            return;
        }

        self.sidebar.select_category(category);

        // Truncate forward history if navigating from middle
        if self.history_index + 1 < self.history_stack.len() {
            self.history_stack.truncate(self.history_index + 1);
        }

        self.history_stack.push(category);
        self.history_index = self.history_stack.len() - 1;
    }

    /// Navigates to the previous category in history.
    pub fn go_back(&mut self) -> bool {
        if self.can_go_back() {
            self.history_index -= 1;
            let cat = self.history_stack[self.history_index];
            self.sidebar.select_category(cat);
            true
        } else {
            false
        }
    }

    /// Navigates forward in history.
    pub fn go_forward(&mut self) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            let cat = self.history_stack[self.history_index];
            self.sidebar.select_category(cat);
            true
        } else {
            false
        }
    }

    /// Returns `true` if previous history exists.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    /// Returns `true` if forward history exists.
    #[must_use]
    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history_stack.len()
    }

    /// Active category currently displayed in the main pane.
    #[must_use]
    pub fn active_category(&self) -> SettingsCategory {
        self.sidebar.active_category
    }

    /// Window title formatted according to active category.
    #[must_use]
    pub fn window_title(&self) -> String {
        format!("Ajustes do Sistema — {}", self.active_category().title())
    }

    /// Bounding rectangle $[x, y, w, h]$ for the Floating Glass Pill Sidebar with 8px margins.
    #[must_use]
    pub fn sidebar_pill_bounds(&self) -> (f64, f64, f64, f64) {
        let margin = 8.0;
        let pill_w = 244.0;
        let pill_h = (self.size.y as f64 - 2.0 * margin).max(100.0);
        (margin, margin, pill_w, pill_h)
    }

    /// Usable bounds $[x, y, w, h]$ for the right content pane scrolling underneath the sidebar glass.
    #[must_use]
    pub fn content_pane_bounds(&self) -> (f64, f64, f64, f64) {
        let left_offset = 260.0; // 8px margin + 244px pill + 8px gap
        let content_w = (self.size.x as f64 - left_offset).max(100.0);
        let content_h = self.size.y as f64;
        (left_offset, 0.0, content_w, content_h)
    }

    /// Evaluates if a point $[x, y]$ in local window coordinates is over the floating sidebar pill.
    #[must_use]
    pub fn is_point_over_sidebar_pill(&self, local_x: f64, local_y: f64) -> bool {
        let (bx, by, bw, bh) = self.sidebar_pill_bounds();
        local_x >= bx && local_x <= (bx + bw) && local_y >= by && local_y <= (by + bh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history_back_forward() {
        let mut win = SettingsWindow::default();
        assert_eq!(win.active_category(), SettingsCategory::Appearance);
        assert!(!win.can_go_back());
        assert!(!win.can_go_forward());

        // Navigate to Displays
        win.navigate_to(SettingsCategory::Displays);
        assert_eq!(win.active_category(), SettingsCategory::Displays);
        assert!(win.can_go_back());
        assert!(!win.can_go_forward());

        // Navigate to Sound
        win.navigate_to(SettingsCategory::Sound);
        assert_eq!(win.active_category(), SettingsCategory::Sound);

        // Go Back -> Displays
        assert!(win.go_back());
        assert_eq!(win.active_category(), SettingsCategory::Displays);
        assert!(win.can_go_forward());

        // Go Back -> Appearance
        assert!(win.go_back());
        assert_eq!(win.active_category(), SettingsCategory::Appearance);
        assert!(!win.can_go_back());

        // Go Forward -> Displays
        assert!(win.go_forward());
        assert_eq!(win.active_category(), SettingsCategory::Displays);
    }

    #[test]
    fn test_window_title() {
        let mut win = SettingsWindow::default();
        assert_eq!(win.window_title(), "Ajustes do Sistema — Aparência");

        win.navigate_to(SettingsCategory::Displays);
        assert_eq!(win.window_title(), "Ajustes do Sistema — Telas");
    }

    #[test]
    fn test_sidebar_pill_geometry() {
        let win = SettingsWindow::default();
        let (bx, by, bw, bh) = win.sidebar_pill_bounds();
        assert_eq!((bx, by, bw), (8.0, 8.0, 244.0));
        assert_eq!(bh, 680.0 - 16.0);

        let (cx, cy, cw, ch) = win.content_pane_bounds();
        assert_eq!((cx, cy), (260.0, 0.0));
        assert_eq!(cw, 960.0 - 260.0);
        assert_eq!(ch, 680.0);

        assert!(win.is_point_over_sidebar_pill(50.0, 50.0));
        assert!(!win.is_point_over_sidebar_pill(300.0, 50.0));
    }
}
