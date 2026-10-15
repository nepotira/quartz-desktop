//! # QPalette Generation & Cupertino Color Role Engine (`qt-quartz-theme::palette`)
//!
//! Production-grade Qt5/Qt6 palette generator mapping macOS Tahoe 26 Cocoa colors into QPalette.
//!
//! ### Key Capabilities:
//! - **Standard QPalette Color Roles:** `Window`, `WindowText`, `Base`, `Text`, `AlternateBase`,
//!   `Button`, `ButtonText`, `Highlight`, `HighlightedText`, `ToolTipBase`, `ToolTipText`.
//! - **Color Group States:** Full distinction between `Active`, `Inactive`, and `Disabled` states.
//! - **Cupertino Accent Color Integration:** Dynamic highlight assignment based on user preferences.

use serde::{Deserialize, Serialize};

/// QPalette color roles defined in the Qt GUI framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorRole {
    /// General window background.
    Window,
    /// General window foreground text.
    WindowText,
    /// Background for editable text entries, lists, and trees.
    Base,
    /// Text color for editable widgets.
    Text,
    /// Alternate background for alternating list rows.
    AlternateBase,
    /// General push button background.
    Button,
    /// Push button foreground text.
    ButtonText,
    /// Selection highlight background (Cupertino accent color).
    Highlight,
    /// Foreground text color for highlighted selections.
    HighlightedText,
    /// Tooltip card background.
    ToolTipBase,
    /// Tooltip card text.
    ToolTipText,
    /// Subtle divider line / border color.
    Mid,
}

/// Color specification across the three Qt widget states.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoleColorGroup {
    /// Active window state color $[R, G, B, A] \in [0.0, 1.0]^4$.
    pub active: [f32; 4],
    /// Inactive (unfocused / background) window state color.
    pub inactive: [f32; 4],
    /// Disabled widget state color.
    pub disabled: [f32; 4],
}

/// Complete Qt QPalette representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QtPalette {
    /// Whether this palette represents a dark appearance.
    pub is_dark: bool,
    /// Role mapping entries.
    roles: Vec<(ColorRole, RoleColorGroup)>,
}

impl QtPalette {
    /// Generates a complete macOS Tahoe 26 QPalette for Qt5/Qt6 applications.
    #[must_use]
    pub fn create_quartz_palette(is_dark: bool, accent_color: [f32; 4]) -> Self {
        let (win_bg, win_fg, base_bg, text_fg, btn_bg, btn_fg, border_col) = if is_dark {
            (
                [0.118, 0.118, 0.133, 1.0], // #1E1E22
                [0.961, 0.961, 0.969, 1.0], // #F5F5F7
                [0.165, 0.169, 0.196, 1.0], // #2A2B32
                [0.961, 0.961, 0.969, 1.0],
                [0.216, 0.220, 0.251, 1.0], // #373840
                [0.961, 0.961, 0.969, 1.0],
                [0.30, 0.30, 0.32, 1.0],
            )
        } else {
            (
                [1.0, 1.0, 1.0, 1.0],       // #FFFFFF
                [0.114, 0.114, 0.122, 1.0], // #1D1D1F
                [0.949, 0.949, 0.969, 1.0], // #F2F2F7
                [0.114, 0.114, 0.122, 1.0],
                [0.973, 0.973, 0.980, 1.0], // #F8F8FA
                [0.114, 0.114, 0.122, 1.0],
                [0.85, 0.85, 0.87, 1.0],
            )
        };

        let disabled_fg = [win_fg[0], win_fg[1], win_fg[2], 0.38];
        let inactive_highlight = if is_dark {
            [0.25, 0.25, 0.28, 1.0]
        } else {
            [0.82, 0.82, 0.85, 1.0]
        };

        let roles = vec![
            (
                ColorRole::Window,
                RoleColorGroup {
                    active: win_bg,
                    inactive: win_bg,
                    disabled: win_bg,
                },
            ),
            (
                ColorRole::WindowText,
                RoleColorGroup {
                    active: win_fg,
                    inactive: [win_fg[0], win_fg[1], win_fg[2], 0.70],
                    disabled: disabled_fg,
                },
            ),
            (
                ColorRole::Base,
                RoleColorGroup {
                    active: base_bg,
                    inactive: base_bg,
                    disabled: win_bg,
                },
            ),
            (
                ColorRole::Text,
                RoleColorGroup {
                    active: text_fg,
                    inactive: [text_fg[0], text_fg[1], text_fg[2], 0.70],
                    disabled: disabled_fg,
                },
            ),
            (
                ColorRole::AlternateBase,
                RoleColorGroup {
                    active: if is_dark {
                        [0.14, 0.14, 0.16, 1.0]
                    } else {
                        [0.98, 0.98, 0.99, 1.0]
                    },
                    inactive: win_bg,
                    disabled: win_bg,
                },
            ),
            (
                ColorRole::Button,
                RoleColorGroup {
                    active: btn_bg,
                    inactive: btn_bg,
                    disabled: win_bg,
                },
            ),
            (
                ColorRole::ButtonText,
                RoleColorGroup {
                    active: btn_fg,
                    inactive: [btn_fg[0], btn_fg[1], btn_fg[2], 0.70],
                    disabled: disabled_fg,
                },
            ),
            (
                ColorRole::Highlight,
                RoleColorGroup {
                    active: accent_color,
                    inactive: inactive_highlight,
                    disabled: if is_dark {
                        [0.20, 0.20, 0.22, 1.0]
                    } else {
                        [0.90, 0.90, 0.92, 1.0]
                    },
                },
            ),
            (
                ColorRole::HighlightedText,
                RoleColorGroup {
                    active: [1.0, 1.0, 1.0, 1.0],
                    inactive: win_fg,
                    disabled: disabled_fg,
                },
            ),
            (
                ColorRole::ToolTipBase,
                RoleColorGroup {
                    active: if is_dark {
                        [0.22, 0.22, 0.25, 0.95]
                    } else {
                        [0.96, 0.96, 0.98, 0.95]
                    },
                    inactive: win_bg,
                    disabled: win_bg,
                },
            ),
            (
                ColorRole::ToolTipText,
                RoleColorGroup {
                    active: win_fg,
                    inactive: win_fg,
                    disabled: disabled_fg,
                },
            ),
            (
                ColorRole::Mid,
                RoleColorGroup {
                    active: border_col,
                    inactive: border_col,
                    disabled: border_col,
                },
            ),
        ];

        Self { is_dark, roles }
    }

    /// Retrieves color group for a specific role.
    #[must_use]
    pub fn color(&self, role: ColorRole) -> Option<RoleColorGroup> {
        self.roles.iter().find(|(r, _)| *r == role).map(|(_, g)| *g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quartz_palette_generation() {
        let cupertino_blue = [0.0, 0.478, 1.0, 1.0];
        let dark_palette = QtPalette::create_quartz_palette(true, cupertino_blue);
        assert!(dark_palette.is_dark);

        let highlight = dark_palette.color(ColorRole::Highlight).expect("highlight role exists");
        assert_eq!(highlight.active, cupertino_blue);
        assert_ne!(highlight.active, highlight.inactive); // Desaturated in inactive

        let light_palette = QtPalette::create_quartz_palette(false, cupertino_blue);
        assert!(!light_palette.is_dark);
        let win_bg = light_palette.color(ColorRole::Window).unwrap();
        assert_eq!(win_bg.active, [1.0, 1.0, 1.0, 1.0]);
    }
}
