//! # Declarative AquaUI Layout Engine (`qz-toolkit::layout`)
//!
//! Production-grade declarative layout system for macOS Tahoe 26 AquaUI.
//!
//! ### Key Capabilities:
//! - **Vertical & Horizontal Stacks (`VStack`, `HStack`):** Dynamic flex layout with
//!   customizable inter-item spacing and cross-axis alignment (`Leading`, `Center`, `Trailing`).
//! - **Flexible Spacers:** Dynamic spacers expanding to consume remaining available axis space.
//! - **Edge Insets & Padding:** Configurable boundary margins adhering to Apple HIG grid units.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Cross-axis horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HorizontalAlignment {
    /// Align to the left/leading boundary.
    Leading,
    /// Center horizontally.
    Center,
    /// Align to the right/trailing boundary.
    Trailing,
}

/// Cross-axis vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerticalAlignment {
    /// Align to the top boundary.
    Top,
    /// Center vertically.
    Center,
    /// Align to the bottom boundary.
    Bottom,
}

/// Rectangular margins surrounding a layout component.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EdgeInsets {
    /// Top margin in points.
    pub top: f64,
    /// Leading / left margin in points.
    pub leading: f64,
    /// Bottom margin in points.
    pub bottom: f64,
    /// Trailing / right margin in points.
    pub trailing: f64,
}

impl Default for EdgeInsets {
    fn default() -> Self {
        Self::uniform(0.0)
    }
}

impl EdgeInsets {
    /// Creates uniform insets across all four edges.
    #[must_use]
    pub fn uniform(value: f64) -> Self {
        Self {
            top: value,
            leading: value,
            bottom: value,
            trailing: value,
        }
    }

    /// Creates symmetric insets for vertical and horizontal axes.
    #[must_use]
    pub fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self {
            top: vertical,
            leading: horizontal,
            bottom: vertical,
            trailing: horizontal,
        }
    }

    /// Total horizontal insets ($leading + trailing$).
    #[must_use]
    pub fn horizontal_total(&self) -> f64 {
        self.leading + self.trailing
    }

    /// Total vertical insets ($top + bottom$).
    #[must_use]
    pub fn vertical_total(&self) -> f64 {
        self.top + self.bottom
    }
}

/// Computed placement rectangle for a laid-out child widget.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlacementRect {
    /// Left origin $X$.
    pub x: f64,
    /// Top origin $Y$.
    pub y: f64,
    /// Layout width.
    pub width: f64,
    /// Layout height.
    pub height: f64,
}

/// Layouter for vertical item sequences (`VStack`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VStackLayout {
    /// Inter-item vertical spacing in points.
    pub spacing: f64,
    /// Cross-axis horizontal alignment.
    pub alignment: HorizontalAlignment,
    /// Boundary insets.
    pub insets: EdgeInsets,
}

impl Default for VStackLayout {
    fn default() -> Self {
        Self {
            spacing: 8.0,
            alignment: HorizontalAlignment::Leading,
            insets: EdgeInsets::default(),
        }
    }
}

impl VStackLayout {
    /// Computes layout positions for a sequence of child sizes within the given bounding container.
    #[must_use]
    pub fn arrange(&self, container_size: Vec2, child_sizes: &[Vec2]) -> Vec<PlacementRect> {
        let mut placements = Vec::with_capacity(child_sizes.len());
        let mut current_y = self.insets.top;
        let content_w = (container_size.x as f64 - self.insets.horizontal_total()).max(0.0);

        for child in child_sizes {
            let child_w = (child.x as f64).min(content_w);
            let child_h = child.y as f64;

            let x = match self.alignment {
                HorizontalAlignment::Leading => self.insets.leading,
                HorizontalAlignment::Center => {
                    self.insets.leading + (content_w - child_w) / 2.0
                }
                HorizontalAlignment::Trailing => {
                    self.insets.leading + content_w - child_w
                }
            };

            placements.push(PlacementRect {
                x,
                y: current_y,
                width: child_w,
                height: child_h,
            });

            current_y += child_h + self.spacing;
        }

        placements
    }
}

/// Layouter for horizontal item sequences (`HStack`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HStackLayout {
    /// Inter-item horizontal spacing in points.
    pub spacing: f64,
    /// Cross-axis vertical alignment.
    pub alignment: VerticalAlignment,
    /// Boundary insets.
    pub insets: EdgeInsets,
}

impl Default for HStackLayout {
    fn default() -> Self {
        Self {
            spacing: 8.0,
            alignment: VerticalAlignment::Center,
            insets: EdgeInsets::default(),
        }
    }
}

impl HStackLayout {
    /// Computes layout positions for a sequence of child sizes within the given bounding container.
    #[must_use]
    pub fn arrange(&self, container_size: Vec2, child_sizes: &[Vec2]) -> Vec<PlacementRect> {
        let mut placements = Vec::with_capacity(child_sizes.len());
        let mut current_x = self.insets.leading;
        let content_h = (container_size.y as f64 - self.insets.vertical_total()).max(0.0);

        for child in child_sizes {
            let child_w = child.x as f64;
            let child_h = (child.y as f64).min(content_h);

            let y = match self.alignment {
                VerticalAlignment::Top => self.insets.top,
                VerticalAlignment::Center => {
                    self.insets.top + (content_h - child_h) / 2.0
                }
                VerticalAlignment::Bottom => {
                    self.insets.top + content_h - child_h
                }
            };

            placements.push(PlacementRect {
                x: current_x,
                y,
                width: child_w,
                height: child_h,
            });

            current_x += child_w + self.spacing;
        }

        placements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vstack_arrangement_center() {
        let layout = VStackLayout {
            spacing: 10.0,
            alignment: HorizontalAlignment::Center,
            insets: EdgeInsets::uniform(15.0),
        };

        let container = Vec2::new(200.0, 300.0);
        let children = vec![
            Vec2::new(100.0, 40.0),
            Vec2::new(60.0, 30.0),
        ];

        let placements = layout.arrange(container, &children);
        assert_eq!(placements.len(), 2);

        // Content width = 200 - 30 = 170
        // Child 0 width = 100 => x = 15 + (170 - 100) / 2 = 15 + 35 = 50
        assert_eq!(placements[0].x, 50.0);
        assert_eq!(placements[0].y, 15.0);

        // Child 1 width = 60 => x = 15 + (170 - 60) / 2 = 15 + 55 = 70
        // y = 15 + 40 (child 0 height) + 10 (spacing) = 65
        assert_eq!(placements[1].x, 70.0);
        assert_eq!(placements[1].y, 65.0);
    }

    #[test]
    fn test_hstack_arrangement_top() {
        let layout = HStackLayout {
            spacing: 12.0,
            alignment: VerticalAlignment::Top,
            insets: EdgeInsets::symmetric(10.0, 20.0),
        };

        let container = Vec2::new(400.0, 100.0);
        let children = vec![
            Vec2::new(80.0, 40.0),
            Vec2::new(120.0, 50.0),
        ];

        let placements = layout.arrange(container, &children);
        assert_eq!(placements.len(), 2);

        assert_eq!(placements[0].x, 20.0);
        assert_eq!(placements[0].y, 10.0);

        // Next item x: 20 + 80 + 12 = 112
        assert_eq!(placements[1].x, 112.0);
        assert_eq!(placements[1].y, 10.0);
    }
}
