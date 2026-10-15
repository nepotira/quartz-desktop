//! # QuartzLinux System Settings Application (`qz-settings`)
//!
//! Production-grade implementation of the macOS Tahoe 26 System Settings application.
//!
//! ### Subsystems Included:
//! 1. **Floating Glass Sidebar ([`sidebar`]):**
//!    - Canonical 5-section hierarchical groupings (Connectivity, Notifications & Sound,
//!      Core System, Workspace, Hardware & Security).
//!    - Instant real-time search engine matching panel titles, descriptions, and deep keywords
//!      (e.g. "vrr", "genie", "basso", "m4", "accent").
//! 2. **Modular Configuration Panels ([`panels`]):**
//!    - **Appearance:** Light, Dark, Auto modes, 8 Cupertino accent colors (`#007AFF`, `#AF52DE`, etc.),
//!      and Liquid Glass window tinting.
//!    - **Displays:** Retina resolution scaling, ProMotion 120Hz dynamic VRR, Night Shift, and True Tone.
//!    - **Desktop & Dock:** Icon sizing, parabolic magnification, 64x64 non-linear Genie effect,
//!      Stage Manager, and 4 Hot Corners.
//!    - **Sound:** Master volume, stereo balance, alert tones (`Sosumi`, `Basso`, `Ping`), and CoreAudio auto-pause.
//!    - **Battery:** Low Power Mode scheduling, health capacity percentage, cycle counts, and optimized charging.
//!    - **Keyboard & Trackpad:** Hardware repeat rates, accents picker toggle, natural scrolling, and multi-touch gestures.
//!    - **General:** About This Mac hardware telemetry (M4 Max specs, Unified Memory, storage breakdown).
//! 3. **Window Controller ([`window`]):**
//!    - Two-pane Liquid Glass window with back/forward history navigation stack.

#![deny(missing_docs)]

pub mod panels;
pub mod sidebar;
pub mod window;

pub use panels::{
    AccentColor, AirDropVisibility, AlertSoundChoice, AppearanceMode, AppearanceSettings,
    BatterySettings, ClickPressure, DictationShortcutKey, DisplaysSettings, DockDesktopSettings,
    DockPosition, GeneralSettings, HotCornerAction, KeyboardTrackpadSettings, LowPowerModePolicy,
    MinimizeEffect, NightShiftSchedule, RefreshRateOption, ResolutionScalingMode,
    SidebarIconSize, SoundSettings, StorageBreakdown, TitlebarDoubleClickAction,
};
pub use sidebar::{
    SettingsCategory, SettingsSection, SettingsSidebar, UserProfileCard,
};
pub use window::SettingsWindow;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_integration() {
        let mut app = SettingsWindow::default();
        assert_eq!(app.active_category(), SettingsCategory::Appearance);

        // Mutate appearance setting
        app.appearance.accent_color = AccentColor::Purple;
        assert_eq!(app.appearance.accent_color.rgba().x, 0.686);

        // Search for displays
        app.sidebar.set_search_query("promotion");
        let results = app.sidebar.filtered_categories();
        assert!(results.contains(&SettingsCategory::Displays));

        // Navigate to Displays
        app.navigate_to(SettingsCategory::Displays);
        assert_eq!(app.active_category(), SettingsCategory::Displays);
        assert_eq!(app.displays.refresh_rate, RefreshRateOption::ProMotion120Hz);
    }
}
