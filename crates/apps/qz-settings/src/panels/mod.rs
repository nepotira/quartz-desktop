//! # Settings Configuration Panels (`qz-settings::panels`)
//!
//! Modular configuration panels implementing macOS Tahoe 26 system settings.

pub mod appearance;
pub mod battery;
pub mod displays;
pub mod dock_desktop;
pub mod general;
pub mod keyboard_trackpad;
pub mod sound;

pub use appearance::{AccentColor, AppearanceMode, AppearanceSettings, SidebarIconSize};
pub use battery::{BatterySettings, LowPowerModePolicy};
pub use displays::{DisplaysSettings, NightShiftSchedule, RefreshRateOption, ResolutionScalingMode};
pub use dock_desktop::{
    DockDesktopSettings, DockPosition, HotCornerAction, MinimizeEffect,
    TitlebarDoubleClickAction,
};
pub use general::{AirDropVisibility, GeneralSettings, StorageBreakdown};
pub use keyboard_trackpad::{
    ClickPressure, DictationShortcutKey, KeyboardTrackpadSettings,
};
pub use sound::{AlertSoundChoice, SoundSettings};
