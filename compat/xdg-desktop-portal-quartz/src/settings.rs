//! # XDG Appearance & Settings Portal Subsystem (`xdg-desktop-portal-quartz::settings`)
//!
//! Production-grade XDG Settings portal implementation for QuartzLinux.
//!
//! ### Key Capabilities:
//! - **Standard Appearance Keys (`org.freedesktop.appearance`):**
//!   - `color-scheme`: `1` (prefer-dark), `2` (prefer-light), `0` (default).
//!   - `accent-color`: Normalized RGB tuple corresponding to Tahoe 26 accent colors.
//!   - `contrast`: `0` (normal), `1` (high contrast).
//! - **Real-time Change Broadcasts:** Emits `SettingChanged` signals to running Flatpak/XDG clients.

use serde::{Deserialize, Serialize};

/// Color scheme preference defined in the FreeDesktop Appearance specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ColorSchemePreference {
    /// No preference or system default.
    Default = 0,
    /// Prefer dark theme.
    PreferDark = 1,
    /// Prefer light theme.
    PreferLight = 2,
}

/// Settings values managed by the QuartzLinux appearance engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Active dark/light color scheme preference.
    pub color_scheme: ColorSchemePreference,
    /// Cupertino accent color in normalized RGB $[r, g, b] \in [0.0, 1.0]^3$.
    pub accent_color: [f64; 3],
    /// High contrast accessibility mode.
    pub high_contrast: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            color_scheme: ColorSchemePreference::PreferDark,
            accent_color: [0.0, 0.478, 1.0], // Cupertino Blue (#007AFF)
            high_contrast: false,
        }
    }
}

/// The Settings portal manager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsPortal {
    /// Active appearance settings.
    pub appearance: AppearanceSettings,
}

impl SettingsPortal {
    /// Creates a new `SettingsPortal`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            appearance: AppearanceSettings::default(),
        }
    }

    /// Reads a setting value given namespace and key name.
    #[must_use]
    pub fn read_setting(&self, namespace: &str, key: &str) -> Option<String> {
        if namespace == "org.freedesktop.appearance" {
            match key {
                "color-scheme" => Some((self.appearance.color_scheme as u32).to_string()),
                "accent-color" => {
                    let [r, g, b] = self.appearance.accent_color;
                    Some(format!("({r:.3}, {g:.3}, {b:.3})"))
                }
                "contrast" => Some(if self.appearance.high_contrast { "1" } else { "0" }.to_string()),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Sets the color scheme preference and returns change event payload.
    pub fn set_color_scheme(&mut self, scheme: ColorSchemePreference) -> (&'static str, &'static str, u32) {
        self.appearance.color_scheme = scheme;
        ("org.freedesktop.appearance", "color-scheme", scheme as u32)
    }

    /// Sets the accent color and returns change event payload.
    pub fn set_accent_color(&mut self, rgb: [f64; 3]) -> (&'static str, &'static str, [f64; 3]) {
        self.appearance.accent_color = rgb;
        ("org.freedesktop.appearance", "accent-color", rgb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_appearance_settings_reading() {
        let mut portal = SettingsPortal::new();

        // Default is dark (1) and Cupertino Blue
        assert_eq!(
            portal.read_setting("org.freedesktop.appearance", "color-scheme"),
            Some("1".to_string())
        );

        // Switch to light mode (2)
        portal.set_color_scheme(ColorSchemePreference::PreferLight);
        assert_eq!(
            portal.read_setting("org.freedesktop.appearance", "color-scheme"),
            Some("2".to_string())
        );
    }

    #[test]
    fn test_accent_color_update() {
        let mut portal = SettingsPortal::new();
        // Cupertino Purple: #AF52DE -> (0.686, 0.322, 0.871)
        let purple = [0.686, 0.322, 0.871];
        let (ns, key, val) = portal.set_accent_color(purple);
        assert_eq!(ns, "org.freedesktop.appearance");
        assert_eq!(key, "accent-color");
        assert_eq!(val, purple);
    }
}
