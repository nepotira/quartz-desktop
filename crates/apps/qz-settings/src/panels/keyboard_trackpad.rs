//! # Keyboard & Trackpad Settings Panel (`qz-settings::panels::keyboard_trackpad`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 Keyboard, Trackpad,
//! key repeat rates, natural scrolling, and 1:1 multi-touch gestures.
//!
//! ### Key Capabilities:
//! - **Key Repeat Dynamics:** Hardware key repeat rate and delay-until-repeat calibration.
//! - **Accents Press-and-Hold Toggle:** Enables the 300ms diacritics selection popup.
//! - **Trackpad Gestures:** Natural scrolling, tap-to-click, pinch zoom, and 3/4-finger Spaces swipes.

use serde::{Deserialize, Serialize};

/// Click feedback pressure sensitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClickPressure {
    /// Soft light tap.
    Light,
    /// Balanced tactile response.
    Medium,
    /// Solid firm click.
    Firm,
}

/// Dictation activation trigger key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DictationShortcutKey {
    /// Canonical double-tap of Fn (Globe) key within 400ms.
    DoubleTapFn,
    /// Press Option key twice.
    DoubleTapOption,
    /// Dictation disabled.
    Off,
}

/// Keyboard, Trackpad, and Gesture configuration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardTrackpadSettings {
    /// Key repeat frequency in Hertz (10 Hz to 60 Hz, standard: 30 Hz).
    pub key_repeat_rate_hz: f64,
    /// Initial delay before repeat begins in milliseconds (150ms to 600ms, standard: 250ms).
    pub delay_until_repeat_ms: f64,
    /// Press and hold key for accents popover enabled.
    pub press_and_hold_accents_enabled: bool,
    /// Dictation activation shortcut.
    pub dictation_shortcut: DictationShortcutKey,
    /// Trackpad pointer tracking speed (1 to 10, standard: 6).
    pub tracking_speed: u8,
    /// Haptic click pressure.
    pub click_pressure: ClickPressure,
    /// Natural scrolling (content tracks finger direction directly).
    pub natural_scrolling: bool,
    /// Tap to click without physical depress.
    pub tap_to_click: bool,
    /// Pinch with two fingers to zoom in or out.
    pub pinch_to_zoom: bool,
    /// Double-tap with two fingers for smart zoom.
    pub smart_zoom: bool,
    /// Swipe horizontally with three fingers between full-screen Spaces.
    pub swipe_between_spaces: bool,
    /// Swipe up with three fingers for Mission Control.
    pub mission_control_gesture: bool,
}

impl Default for KeyboardTrackpadSettings {
    fn default() -> Self {
        Self {
            key_repeat_rate_hz: 30.0,
            delay_until_repeat_ms: 250.0,
            press_and_hold_accents_enabled: true,
            dictation_shortcut: DictationShortcutKey::DoubleTapFn,
            tracking_speed: 6,
            click_pressure: ClickPressure::Medium,
            natural_scrolling: true,
            tap_to_click: true,
            pinch_to_zoom: true,
            smart_zoom: true,
            swipe_between_spaces: true,
            mission_control_gesture: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_trackpad_defaults() {
        let settings = KeyboardTrackpadSettings::default();
        assert_eq!(settings.key_repeat_rate_hz, 30.0);
        assert_eq!(settings.delay_until_repeat_ms, 250.0);
        assert!(settings.press_and_hold_accents_enabled);
        assert!(settings.natural_scrolling);
        assert!(settings.tap_to_click);
        assert_eq!(settings.click_pressure, ClickPressure::Medium);
    }
}
