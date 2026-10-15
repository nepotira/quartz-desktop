//! # Interactive Widgets and Adaptive Chromatic Transition Subsystem
//!
//! Controls desktop wallpaper and panel widgets:
//! - Native widget gallery: Weather, Calendar, Interactive Reminders, Clock, System Monitor, Battery, Photos.
//! - **Adaptive Chromatic Transition (macOS Tahoe 26)**:
//!   - When an application window receives focus, desktop widgets smoothly cross-fade (250ms)
//!     into a **monochrome translucent Liquid Glass mode**.
//!   - When the user clicks the wallpaper or triggers *Show Desktop*, widgets instantly
//!     recover their full vibrant saturated colors.

use glam::Vec2;

/// Widget categorization type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind {
    /// Weather forecast.
    Weather,
    /// Calendar events.
    Calendar,
    /// Interactive task reminders with checkbox state.
    Reminders,
    /// World clock.
    Clock,
    /// CPU and memory usage graphs.
    SystemMonitor,
    /// Battery levels for Mac and connected Bluetooth accessories.
    Battery,
    /// Featured photo memory frame.
    Photos,
}

/// Standard Apple widget sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetSize {
    /// Small square widget ($158 \times 158$ px).
    Small,
    /// Medium rectangular widget ($338 \times 158$ px).
    Medium,
    /// Large square widget ($338 \times 338$ px).
    Large,
    /// Extra-large rectangular widget ($698 \times 338$ px).
    ExtraLarge,
}

impl WidgetSize {
    /// Pixel dimensions in points.
    pub fn dimensions(&self) -> Vec2 {
        match self {
            Self::Small => Vec2::new(158.0, 158.0),
            Self::Medium => Vec2::new(338.0, 158.0),
            Self::Large => Vec2::new(338.0, 338.0),
            Self::ExtraLarge => Vec2::new(698.0, 338.0),
        }
    }
}

/// Chromatic appearance mode of desktop widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetColorMode {
    /// Full vibrant saturated colors (active when desktop is focused).
    FullColor,
    /// Monochromatic translucent Liquid Glass (engaged when application windows are focused).
    MonochromeTranslucent,
}

/// An active widget placed on the desktop wallpaper.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopWidget {
    /// Unique widget instance ID.
    pub id: u64,
    /// Widget type.
    pub kind: WidgetKind,
    /// Widget size format.
    pub size: WidgetSize,
    /// Screen position `(x, y)`.
    pub position: Vec2,
}

/// Coordinates desktop widgets and the 250ms adaptive chromatic cross-fade.
pub struct WidgetManager {
    widgets: Vec<DesktopWidget>,
    color_mode: WidgetColorMode,
    chromatic_transition_progress: f32, // 0.0 = FullColor, 1.0 = Monochrome
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WidgetManager {
    /// Duration of adaptive chromatic transition (250ms).
    pub const CHROMATIC_TRANSITION_DURATION_SECS: f32 = 0.250;

    /// Creates an empty widget manager.
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            color_mode: WidgetColorMode::FullColor,
            chromatic_transition_progress: 0.0,
        }
    }

    /// Adds a widget to the desktop.
    pub fn add_widget(&mut self, id: u64, kind: WidgetKind, size: WidgetSize, position: Vec2) {
        self.widgets.push(DesktopWidget { id, kind, size, position });
    }

    /// Notifies widget manager of window focus transition.
    ///
    /// If an app window is focused, fades to Monochrome. If desktop is focused, restores FullColor.
    pub fn set_window_focused(&mut self, is_window_focused: bool) {
        if is_window_focused {
            self.color_mode = WidgetColorMode::MonochromeTranslucent;
        } else {
            self.color_mode = WidgetColorMode::FullColor;
        }
    }

    /// Advances the 250ms chromatic cross-fade animation by `dt` seconds.
    pub fn step(&mut self, dt: f32) {
        let target = match self.color_mode {
            WidgetColorMode::FullColor => 0.0,
            WidgetColorMode::MonochromeTranslucent => 1.0,
        };

        let step_amount = dt / Self::CHROMATIC_TRANSITION_DURATION_SECS;
        if self.chromatic_transition_progress < target {
            self.chromatic_transition_progress = (self.chromatic_transition_progress + step_amount).min(target);
        } else if self.chromatic_transition_progress > target {
            self.chromatic_transition_progress = (self.chromatic_transition_progress - step_amount).max(target);
        }
    }

    /// Current chromatic monochrome blend factor $[0.0, 1.0]$:
    /// - `0.0`: 100% saturated full color.
    /// - `1.0`: 100% monochrome translucent Liquid Glass.
    #[inline]
    pub fn monochrome_blend_factor(&self) -> f32 {
        self.chromatic_transition_progress
    }

    /// Number of active desktop widgets.
    #[inline]
    pub fn widget_count(&self) -> usize {
        self.widgets.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_chromatic_transition() {
        let mut wm = WidgetManager::new();
        wm.add_widget(1, WidgetKind::Weather, WidgetSize::Small, Vec2::new(100.0, 100.0));

        assert_eq!(wm.monochrome_blend_factor(), 0.0);

        // Focus an application window
        wm.set_window_focused(true);

        // Step 125ms (halfway)
        wm.step(0.125);
        assert!((wm.monochrome_blend_factor() - 0.5).abs() < 0.05);

        // Step remaining 125ms (fully monochrome)
        wm.step(0.125);
        assert_eq!(wm.monochrome_blend_factor(), 1.0);

        // Click desktop: restore colors
        wm.set_window_focused(false);
        wm.step(0.250);
        assert_eq!(wm.monochrome_blend_factor(), 0.0);
    }
}
