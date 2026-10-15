//! # Appearance Settings Panel (`qz-settings::panels::appearance`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 appearance, themes,
//! and Cupertino accent colors.
//!
//! ### Key Capabilities:
//! - **Appearance Modes:** `Light`, `Dark`, and `Auto` (switches automatically with sunrise/sunset).
//! - **Cupertino Accent Colors:** 8 canonical Apple accent colors (Blue, Purple, Pink, Red,
//!   Orange, Yellow, Green, Graphite, and Multicolor).
//! - **Window Vibrancy & Wallpaper Tinting:** Controls Liquid Glass frosted translucency and
//!   sidebar icon sizing.

use glam::Vec4;
use serde::{Deserialize, Serialize};

/// Appearance lighting theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AppearanceMode {
    /// Crisp bright light interface.
    Light,
    /// Dark high-contrast interface reducing eye strain.
    Dark,
    /// Automatically transitions between Light and Dark based on local solar time.
    Auto,
}

/// Canonical Apple Cupertino Accent Color Palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccentColor {
    /// Canonical Cupertino Blue (`#007AFF`).
    Blue,
    /// Vibrant Purple (`#AF52DE`).
    Purple,
    /// Hot Pink (`#FF2D55`).
    Pink,
    /// Apple Red (`#FF3B30`).
    Red,
    /// Warm Orange (`#FF9500`).
    Orange,
    /// Solar Yellow (`#FFCC00`).
    Yellow,
    /// Spring Green (`#34C759`).
    Green,
    /// Neutral Graphite (`#8E8E93`).
    Graphite,
    /// Dynamic Multicolor adapting to application brand colors.
    Multicolor,
}

impl AccentColor {
    /// Returns the RGBA color vector for this accent color.
    #[must_use]
    pub fn rgba(&self) -> Vec4 {
        match self {
            Self::Blue | Self::Multicolor => Vec4::new(0.0, 0.478, 1.0, 1.0),
            Self::Purple => Vec4::new(0.686, 0.322, 0.871, 1.0),
            Self::Pink => Vec4::new(1.0, 0.176, 0.333, 1.0),
            Self::Red => Vec4::new(1.0, 0.231, 0.188, 1.0),
            Self::Orange => Vec4::new(1.0, 0.584, 0.0, 1.0),
            Self::Yellow => Vec4::new(1.0, 0.800, 0.0, 1.0),
            Self::Green => Vec4::new(0.204, 0.780, 0.349, 1.0),
            Self::Graphite => Vec4::new(0.557, 0.557, 0.576, 1.0),
        }
    }

    /// Returns the human-readable color label in Portuguese.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Blue => "Azul",
            Self::Purple => "Roxo",
            Self::Pink => "Rosa",
            Self::Red => "Vermelho",
            Self::Orange => "Laranja",
            Self::Yellow => "Amarelo",
            Self::Green => "Verde",
            Self::Graphite => "Grafite",
            Self::Multicolor => "Multicor",
        }
    }
}

/// Sidebar icon rendering size preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidebarIconSize {
    /// Compact 14px icons.
    Small,
    /// Standard 18px icons.
    Medium,
    /// Large 22px icons.
    Large,
}

/// Appearance Settings configuration state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceSettings {
    /// Active appearance theme mode.
    pub mode: AppearanceMode,
    /// System accent color.
    pub accent_color: AccentColor,
    /// Text and control highlight color matching accent or custom.
    pub highlight_matches_accent: bool,
    /// Sidebar icon size.
    pub sidebar_icon_size: SidebarIconSize,
    /// Allow wallpaper color tinting in windows (Liquid Glass vibrancy).
    pub allow_wallpaper_tinting: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            mode: AppearanceMode::Dark,
            accent_color: AccentColor::Blue,
            highlight_matches_accent: true,
            sidebar_icon_size: SidebarIconSize::Medium,
            allow_wallpaper_tinting: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accent_colors_rgba() {
        assert_eq!(AccentColor::Blue.rgba(), Vec4::new(0.0, 0.478, 1.0, 1.0));
        assert_eq!(AccentColor::Red.rgba(), Vec4::new(1.0, 0.231, 0.188, 1.0));
        assert_eq!(AccentColor::Green.display_name(), "Verde");
    }

    #[test]
    fn test_appearance_defaults() {
        let settings = AppearanceSettings::default();
        assert_eq!(settings.mode, AppearanceMode::Dark);
        assert_eq!(settings.accent_color, AccentColor::Blue);
        assert!(settings.allow_wallpaper_tinting);
    }
}
