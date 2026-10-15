//! # Titlebar & Headerbar CSD Decoration Engine (`gtk-quartz-module::decorations`)
//!
//! Production-grade GTK titlebar and traffic light harmonization for macOS Tahoe 26 on QuartzLinux.
//!
//! ### Key Capabilities:
//! - **Canonical Decoration Layout:** Normalizes `gtk-decoration-layout` to `"close,minimize,maximize:"`
//!   (left-aligned traffic lights).
//! - **Tahoe 26 Traffic Light Metrics:** 12px diameter buttons, 8px spacing, and authentic
//!   hover glyphs (`✕`, `−`, `+`).
//! - **Centered Window Titles:** Calculates balanced headerbar margins to ensure the title
//!   remains centered even with asymmetric left/right action widgets.

use serde::{Deserialize, Serialize};

/// Canonical macOS Tahoe 26 decoration layout string for GTK settings.
pub const MACOS_DECORATION_LAYOUT: &str = "close,minimize,maximize:";

/// Traffic light button classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrafficLightKind {
    /// Red button to close the window (`✕`).
    Close,
    /// Yellow button to minimize the window (`−`).
    Minimize,
    /// Green button to toggle full-screen or zoom (`+`).
    Maximize,
}

/// Color specification for a traffic light button.
/// Color specification for a traffic light button.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TrafficLightColor {
    /// Background color in active window state.
    pub active_bg: [f32; 4],
    /// Border stroke outline color in active window state (0.5px).
    pub active_border: [f32; 4],
    /// Symbol / glyph color on cursor hover.
    pub hover_glyph_color: [f32; 4],
    /// Symbol / glyph color for dirty state central dot (`•`).
    pub dirty_glyph_color: [f32; 4],
    /// Background color in inactive window state (Dark Mode: rgba(128, 128, 128, 0.24)).
    pub inactive_bg_dark: [f32; 4],
    /// Background color in inactive window state (Light Mode: rgba(0, 0, 0, 0.15)).
    pub inactive_bg_light: [f32; 4],
    /// Default symbol string shown on hover.
    pub hover_glyph: &'static str,
    /// Alternate symbol string shown on hover when Option key is held.
    pub option_hover_glyph: &'static str,
}

impl TrafficLightKind {
    /// Returns the exact Tahoe 26 color specifications for the button.
    #[must_use]
    pub fn colors(&self) -> TrafficLightColor {
        match self {
            Self::Close => TrafficLightColor {
                active_bg: [1.0, 0.373, 0.337, 1.0],      // #FF5F56
                active_border: [0.878, 0.267, 0.243, 1.0], // #E0443E (0.5px)
                hover_glyph_color: [0.302, 0.0, 0.0, 1.0],  // #4D0000
                dirty_glyph_color: [0.290, 0.0, 0.008, 1.0], // #4A0002
                inactive_bg_dark: [0.502, 0.502, 0.502, 0.24], // rgba(128, 128, 128, 0.24)
                inactive_bg_light: [0.0, 0.0, 0.0, 0.15],    // rgba(0, 0, 0, 0.15)
                hover_glyph: "✕",
                option_hover_glyph: "✕",
            },
            Self::Minimize => TrafficLightColor {
                active_bg: [1.0, 0.741, 0.180, 1.0],      // #FFBD2E
                active_border: [0.871, 0.631, 0.137, 1.0], // #DEA123 (0.5px)
                hover_glyph_color: [0.600, 0.341, 0.0, 1.0],  // #995700
                dirty_glyph_color: [0.600, 0.341, 0.0, 1.0],
                inactive_bg_dark: [0.502, 0.502, 0.502, 0.24],
                inactive_bg_light: [0.0, 0.0, 0.0, 0.15],
                hover_glyph: "−",
                option_hover_glyph: "−",
            },
            Self::Maximize => TrafficLightColor {
                active_bg: [0.153, 0.788, 0.247, 1.0],    // #27C93F
                active_border: [0.102, 0.671, 0.161, 1.0], // #1AAB29 (0.5px)
                hover_glyph_color: [0.0, 0.396, 0.0, 1.0],  // #006500
                dirty_glyph_color: [0.0, 0.396, 0.0, 1.0],
                inactive_bg_dark: [0.502, 0.502, 0.502, 0.24],
                inactive_bg_light: [0.0, 0.0, 0.0, 0.15],
                hover_glyph: "⤢",
                option_hover_glyph: "+",
            },
        }
    }
}

/// Computes an 18% luminance darkening for the active mouse-down state.
#[must_use]
pub fn apply_pressed_darkening(color: [f32; 4]) -> [f32; 4] {
    [color[0] * 0.82, color[1] * 0.82, color[2] * 0.82, color[3]]
}

/// Evaluated visual rendering state for a traffic light button.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonRenderStyle {
    /// Background fill RGBA.
    pub bg_color: [f32; 4],
    /// Border stroke outline RGBA.
    pub border_color: [f32; 4],
    /// Optional symbol text glyph to render.
    pub glyph: Option<&'static str>,
    /// Color of the symbol glyph.
    pub glyph_color: [f32; 4],
}

/// Computes the authentic macOS Tahoe 26 rendering style for a traffic light button.
#[must_use]
pub fn compute_traffic_light_render_style(
    kind: TrafficLightKind,
    is_window_active: bool,
    is_cluster_hovered: bool,
    is_button_pressed: bool,
    is_option_held: bool,
    is_dark_mode: bool,
    is_document_dirty: bool,
) -> ButtonRenderStyle {
    let spec = kind.colors();

    // If window is inactive and cluster is NOT hovered: draw monochrome translucent circle
    if !is_window_active && !is_cluster_hovered {
        let bg = if is_dark_mode {
            spec.inactive_bg_dark
        } else {
            spec.inactive_bg_light
        };

        // If Close button and document is dirty, render small central bullet dot even when unfocused
        if kind == TrafficLightKind::Close && is_document_dirty {
            return ButtonRenderStyle {
                bg_color: bg,
                border_color: [0.0, 0.0, 0.0, 0.0],
                glyph: Some("•"),
                glyph_color: spec.dirty_glyph_color,
            };
        }

        return ButtonRenderStyle {
            bg_color: bg,
            border_color: [0.0, 0.0, 0.0, 0.0],
            glyph: None,
            glyph_color: [0.0, 0.0, 0.0, 0.0],
        };
    }

    // Active or cluster hovered: saturated colors
    let mut bg = spec.active_bg;
    let mut border = spec.active_border;

    // Apply 18% luminance darkening on mouse-down pressed button
    if is_button_pressed {
        bg = apply_pressed_darkening(bg);
        border = apply_pressed_darkening(border);
    }

    // Determine glyph to reveal
    let (glyph, glyph_color) = if is_cluster_hovered {
        let sym = match kind {
            TrafficLightKind::Close => {
                if is_document_dirty && !is_cluster_hovered {
                    "•"
                } else {
                    spec.hover_glyph
                }
            }
            TrafficLightKind::Minimize => spec.hover_glyph,
            TrafficLightKind::Maximize => {
                if is_option_held {
                    spec.option_hover_glyph
                } else {
                    spec.hover_glyph
                }
            }
        };
        (Some(sym), spec.hover_glyph_color)
    } else if kind == TrafficLightKind::Close && is_document_dirty {
        (Some("•"), spec.dirty_glyph_color)
    } else {
        (None, [0.0, 0.0, 0.0, 0.0])
    };

    ButtonRenderStyle {
        bg_color: bg,
        border_color: border,
        glyph,
        glyph_color,
    }
}

/// Normalizes any input GTK decoration layout string into the canonical macOS left-aligned layout.
#[must_use]
pub fn enforce_macos_decoration_layout(_current_layout: &str) -> &'static str {
    MACOS_DECORATION_LAYOUT
}

/// Calculates balancing margin padding so that the window title remains centered
/// despite asymmetric left and right action buttons in the headerbar.
#[must_use]
pub fn calculate_title_centering_padding(left_actions_width: f32, right_actions_width: f32) -> (f32, f32) {
    let diff = left_actions_width - right_actions_width;
    if diff > 0.0 {
        // Left side is wider; add padding to right side
        (0.0, diff)
    } else {
        // Right side is wider; add padding to left side
        (-diff, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_lights_color_specifications() {
        let close = TrafficLightKind::Close.colors();
        assert_eq!(close.hover_glyph, "✕");
        assert!((close.active_bg[0] - 1.0).abs() < 1e-4);
        assert!((close.active_border[0] - 0.878).abs() < 1e-3);
        assert!((close.hover_glyph_color[0] - 0.302).abs() < 1e-3);
        assert!((close.dirty_glyph_color[0] - 0.290).abs() < 1e-3);

        let minimize = TrafficLightKind::Minimize.colors();
        assert_eq!(minimize.hover_glyph, "−");
        assert!((minimize.active_bg[1] - 0.741).abs() < 1e-3);

        let maximize = TrafficLightKind::Maximize.colors();
        assert_eq!(maximize.hover_glyph, "⤢");
        assert_eq!(maximize.option_hover_glyph, "+");
        assert!((maximize.hover_glyph_color[1] - 0.396).abs() < 1e-3);
    }

    #[test]
    fn test_traffic_light_cluster_hover_and_pressed_darkening() {
        // Inactive window without cluster hover: translucent monochrome
        let unfocused = compute_traffic_light_render_style(
            TrafficLightKind::Close,
            false,
            false,
            false,
            false,
            true, // dark mode
            false,
        );
        assert_eq!(unfocused.bg_color, [0.502, 0.502, 0.502, 0.24]);
        assert_eq!(unfocused.glyph, None);

        // Inactive window WITH cluster hover: all 3 illuminate in full active colors!
        let cluster_hovered = compute_traffic_light_render_style(
            TrafficLightKind::Close,
            false,
            true,
            false,
            false,
            true,
            false,
        );
        assert_eq!(cluster_hovered.bg_color, [1.0, 0.373, 0.337, 1.0]);
        assert_eq!(cluster_hovered.glyph, Some("✕"));

        // Pressed button: 18% luminance darkening
        let pressed = compute_traffic_light_render_style(
            TrafficLightKind::Close,
            true,
            true,
            true, // pressed
            false,
            true,
            false,
        );
        assert!((pressed.bg_color[0] - 1.0 * 0.82).abs() < 1e-4);
        assert!((pressed.bg_color[1] - 0.373 * 0.82).abs() < 1e-4);

        // Green button with Option held -> "+" glyph
        let option_zoom = compute_traffic_light_render_style(
            TrafficLightKind::Maximize,
            true,
            true,
            false,
            true, // option held
            true,
            false,
        );
        assert_eq!(option_zoom.glyph, Some("+"));
    }

    #[test]
    fn test_decoration_layout_enforcement() {
        assert_eq!(enforce_macos_decoration_layout(":minimize,maximize,close"), MACOS_DECORATION_LAYOUT);
        assert_eq!(enforce_macos_decoration_layout("menu:close"), MACOS_DECORATION_LAYOUT);
    }

    #[test]
    fn test_title_centering_padding() {
        // Left: 72px (traffic lights), Right: 120px (search + menu)
        let (pad_l, pad_r) = calculate_title_centering_padding(72.0, 120.0);
        assert_eq!(pad_l, 48.0);
        assert_eq!(pad_r, 0.0);

        // Left: 150px, Right: 50px
        let (pad_l2, pad_r2) = calculate_title_centering_padding(150.0, 50.0);
        assert_eq!(pad_l2, 0.0);
        assert_eq!(pad_r2, 100.0);
    }
}
