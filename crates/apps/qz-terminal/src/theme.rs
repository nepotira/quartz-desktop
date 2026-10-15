//! # Liquid Glass Terminal Profiles & Color Palettes (`qz-terminal::theme`)
//!
//! Production-grade color themes and optical glass configurations for macOS Tahoe 26 Terminal (`qz-terminal`).
//!
//! ### Key Capabilities:
//! - **Canonical Tahoe 26 Visual Profiles:** Tahoe Glass (Translucent Acrylic), Pro (Deep Black Glass),
//!   Silver (Light Translucent), Homebrew (Phosphor Green on Black), and Ocean (Deep Navy Glass).
//! - **16-Color ANSI & 24-Bit Palette:** Complete Cupertino terminal palette definitions.
//! - **Liquid Glass Optical Material:** Background Kawase blur ($\sigma = 24.0\text{px}$),
//!   refraction index ($\eta = 1.52$), and customizable glass opacity ($0.50$ to $1.0$).
//! - **Typography Engine:** Font family (SF Mono / Menlo / Monaco), font size scaling (`Cmd++`, `Cmd+-`, `Cmd+0`).

use serde::{Deserialize, Serialize};

/// Visual profile for the terminal window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalProfile {
    /// Identifier string.
    pub id: String,
    /// User-visible profile name.
    pub name: String,
    /// RGBA background color with alpha transparency for Liquid Glass.
    pub background_color: [f32; 4],
    /// Primary RGBA text color.
    pub foreground_color: [f32; 4],
    /// Text cursor color.
    pub cursor_color: [f32; 4],
    /// Text selection highlight color.
    pub selection_color: [f32; 4],
    /// 16-color ANSI palette (8 standard + 8 bright).
    pub ansi_colors: [[f32; 4]; 16],
    /// Monospaced font family name.
    pub font_family: String,
    /// Font size in points.
    pub font_size_pt: f32,
    /// Line spacing height multiplier.
    pub line_spacing: f32,
    /// Background Dual-Kawase blur radius in pixels.
    pub blur_sigma: f32,
    /// Optical refraction index.
    pub refraction_eta: f32,
}

impl TerminalProfile {
    /// Tahoe 26 Liquid Glass default profile: Dark translucent acrylic with cyan cursor.
    #[must_use]
    pub fn tahoe_glass() -> Self {
        Self {
            id: "tahoe-glass".to_string(),
            name: "Tahoe Glass".to_string(),
            background_color: [0.09, 0.10, 0.13, 0.76], // #171A21 @ 76%
            foreground_color: [0.94, 0.94, 0.96, 1.0],  // Off-white
            cursor_color: [0.18, 0.80, 0.92, 1.0],      // Tahoe Cyan
            selection_color: [0.0, 0.48, 1.0, 0.35],    // Cupertino Blue @ 35%
            ansi_colors: Self::default_ansi_palette(),
            font_family: "SF Mono".to_string(),
            font_size_pt: 13.0,
            line_spacing: 1.18,
            blur_sigma: 24.0,
            refraction_eta: 1.52,
        }
    }

    /// Pro profile: Solid dark charcoal glass with bright green cursor.
    #[must_use]
    pub fn pro() -> Self {
        Self {
            id: "pro".to_string(),
            name: "Pro".to_string(),
            background_color: [0.0, 0.0, 0.0, 0.88],
            foreground_color: [0.96, 0.96, 0.96, 1.0],
            cursor_color: [0.20, 0.78, 0.35, 1.0], // Green
            selection_color: [0.3, 0.3, 0.3, 0.5],
            ansi_colors: Self::default_ansi_palette(),
            font_family: "SF Mono".to_string(),
            font_size_pt: 13.0,
            line_spacing: 1.15,
            blur_sigma: 20.0,
            refraction_eta: 1.50,
        }
    }

    /// Silver profile: Light translucent glass with dark typography.
    #[must_use]
    pub fn silver() -> Self {
        Self {
            id: "silver".to_string(),
            name: "Silver".to_string(),
            background_color: [0.95, 0.96, 0.97, 0.82],
            foreground_color: [0.12, 0.12, 0.14, 1.0],
            cursor_color: [0.0, 0.48, 1.0, 1.0], // Blue
            selection_color: [0.0, 0.48, 1.0, 0.25],
            ansi_colors: Self::default_ansi_palette(),
            font_family: "SF Mono".to_string(),
            font_size_pt: 13.0,
            line_spacing: 1.18,
            blur_sigma: 24.0,
            refraction_eta: 1.52,
        }
    }

    /// Homebrew profile: Classic CRT phosphor green on deep black.
    #[must_use]
    pub fn homebrew() -> Self {
        Self {
            id: "homebrew".to_string(),
            name: "Homebrew".to_string(),
            background_color: [0.0, 0.0, 0.0, 0.92],
            foreground_color: [0.20, 1.0, 0.0, 1.0], // #33FF00 Phosphor
            cursor_color: [0.20, 1.0, 0.0, 1.0],
            selection_color: [0.20, 1.0, 0.0, 0.30],
            ansi_colors: Self::default_ansi_palette(),
            font_family: "Menlo".to_string(),
            font_size_pt: 13.0,
            line_spacing: 1.15,
            blur_sigma: 16.0,
            refraction_eta: 1.48,
        }
    }

    /// Ocean profile: Deep oceanic navy acrylic with amber cursor.
    #[must_use]
    pub fn ocean() -> Self {
        Self {
            id: "ocean".to_string(),
            name: "Ocean".to_string(),
            background_color: [0.05, 0.12, 0.22, 0.80],
            foreground_color: [0.85, 0.95, 1.0, 1.0],
            cursor_color: [1.0, 0.75, 0.0, 1.0], // Amber
            selection_color: [0.2, 0.5, 0.8, 0.35],
            ansi_colors: Self::default_ansi_palette(),
            font_family: "SF Mono".to_string(),
            font_size_pt: 13.0,
            line_spacing: 1.18,
            blur_sigma: 24.0,
            refraction_eta: 1.52,
        }
    }

    /// Increases font size by 1.0 point (`Cmd++`).
    pub fn zoom_in(&mut self) {
        self.font_size_pt = (self.font_size_pt + 1.0).min(36.0);
    }

    /// Decreases font size by 1.0 point (`Cmd+-`).
    pub fn zoom_out(&mut self) {
        self.font_size_pt = (self.font_size_pt - 1.0).max(8.0);
    }

    /// Resets font size to default 13.0 points (`Cmd+0`).
    pub fn zoom_actual_size(&mut self) {
        self.font_size_pt = 13.0;
    }

    fn default_ansi_palette() -> [[f32; 4]; 16] {
        [
            [0.0, 0.0, 0.0, 1.0],       // 0: Black
            [0.80, 0.19, 0.19, 1.0],    // 1: Red
            [0.05, 0.74, 0.47, 1.0],    // 2: Green
            [0.90, 0.90, 0.06, 1.0],    // 3: Yellow
            [0.14, 0.45, 0.78, 1.0],    // 4: Blue
            [0.74, 0.25, 0.74, 1.0],    // 5: Magenta
            [0.07, 0.66, 0.80, 1.0],    // 6: Cyan
            [0.90, 0.90, 0.90, 1.0],    // 7: White
            [0.40, 0.40, 0.40, 1.0],    // 8: Bright Black
            [0.95, 0.30, 0.30, 1.0],    // 9: Bright Red
            [0.14, 0.82, 0.55, 1.0],    // 10: Bright Green
            [0.96, 0.96, 0.26, 1.0],    // 11: Bright Yellow
            [0.23, 0.56, 0.92, 1.0],    // 12: Bright Blue
            [0.84, 0.44, 0.84, 1.0],    // 13: Bright Magenta
            [0.16, 0.72, 0.86, 1.0],    // 14: Bright Cyan
            [1.0, 1.0, 1.0, 1.0],       // 15: Bright White
        ]
    }
}

impl Default for TerminalProfile {
    fn default() -> Self {
        Self::tahoe_glass()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_presets() {
        let glass = TerminalProfile::tahoe_glass();
        assert_eq!(glass.name, "Tahoe Glass");
        assert!(glass.background_color[3] < 1.0); // Translucent
        assert_eq!(glass.font_size_pt, 13.0);

        let pro = TerminalProfile::pro();
        assert_eq!(pro.name, "Pro");

        let silver = TerminalProfile::silver();
        assert_eq!(silver.name, "Silver");

        let homebrew = TerminalProfile::homebrew();
        assert_eq!(homebrew.name, "Homebrew");

        let ocean = TerminalProfile::ocean();
        assert_eq!(ocean.name, "Ocean");
    }

    #[test]
    fn test_font_zoom_and_clamping() {
        let mut prof = TerminalProfile::tahoe_glass();
        assert_eq!(prof.font_size_pt, 13.0);

        prof.zoom_in();
        assert_eq!(prof.font_size_pt, 14.0);

        prof.zoom_actual_size();
        assert_eq!(prof.font_size_pt, 13.0);

        // Clamping to minimum 8.0
        for _ in 0..10 {
            prof.zoom_out();
        }
        assert_eq!(prof.font_size_pt, 8.0);
    }
}
