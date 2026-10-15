//! # Physical Display Notch Handling and Menu Partitioning
//!
//! Provides hardware cutout accommodation matching MacBook Pro / Apple Silicon displays:
//! - Pointer raycasts traverse the notch area smoothly without coordinate clipping.
//! - Menus flow naturally from the Apple icon to the right; if menu items reach
//!   within 16px of `notch.x`, the layout partitions and resumes immediately after
//!   the notch at `notch.x + notch.width + 16px`.

use glam::Vec2;

/// Physical dimensions of a display cutout (Notch).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayNotch {
    /// Origin X coordinate of the notch in screen space.
    pub x: f32,
    /// Origin Y coordinate (typically 0.0 at top of display).
    pub y: f32,
    /// Width of the notch cutout in physical pixels.
    pub width: f32,
    /// Height of the notch cutout in physical pixels (typically 32px-38px).
    pub height: f32,
}

impl DisplayNotch {
    /// Safety padding between menu labels and the physical notch edge (16.0px).
    pub const NOTCH_PADDING: f32 = 16.0;

    /// Creates a new display notch specification.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Left boundary before which menus must stop before crossing the notch.
    #[inline]
    pub fn left_boundary(&self) -> f32 {
        self.x - Self::NOTCH_PADDING
    }

    /// Right boundary after which partitioned menus resume rendering.
    #[inline]
    pub fn right_resume_boundary(&self) -> f32 {
        self.x + self.width + Self::NOTCH_PADDING
    }

    /// Checks if a 2D coordinate falls inside the physical cutout.
    #[inline]
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= (self.x + self.width)
            && point.y >= self.y
            && point.y <= (self.y + self.height)
    }
}

/// A positioned top-level menu header item.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedMenuItem {
    /// Text label (e.g. "File", "Edit", "View").
    pub label: String,
    /// Calculated X position on the menu bar.
    pub x: f32,
    /// Width of the label bounding box in pixels.
    pub width: f32,
    /// True if pushed to the right side of the notch.
    pub is_partitioned_past_notch: bool,
}

/// Partitions top-level menus around a hardware display cutout.
pub struct MenuNotchPartitioner;

impl MenuNotchPartitioner {
    /// Layouts a list of menu labels and widths, dynamically leaping across the notch if necessary.
    pub fn layout_menus(
        start_x: f32,
        item_padding: f32,
        items: &[(&str, f32)], // (Label, Width)
        notch: Option<&DisplayNotch>,
    ) -> Vec<PositionedMenuItem> {
        let mut result = Vec::with_capacity(items.len());
        let mut current_x = start_x;

        for &(label, item_width) in items {
            let next_x = current_x + item_width;

            let (final_x, past_notch) = match notch {
                Some(n) if next_x > n.left_boundary() && current_x < n.right_resume_boundary() => {
                    // Leap across the notch to the right side
                    let resume_x = n.right_resume_boundary();
                    (resume_x, true)
                }
                _ => (current_x, false),
            };

            result.push(PositionedMenuItem {
                label: label.to_string(),
                x: final_x,
                width: item_width,
                is_partitioned_past_notch: past_notch,
            });

            current_x = final_x + item_width + item_padding;
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_leaping_across_notch() {
        // Display 3024x1964, Notch at center: x = 1412.0, width = 200.0 (1412 to 1612)
        let notch = DisplayNotch::new(1412.0, 0.0, 200.0, 34.0);

        let items = [
            ("", 28.0),
            ("Finder", 54.0),
            ("File", 42.0),
            ("Edit", 42.0),
            ("View", 44.0),
            ("Go", 36.0),
            ("Window", 60.0),
            ("Help", 45.0),
        ];

        // Start menus near notch to test leap
        let layout = MenuNotchPartitioner::layout_menus(1300.0, 14.0, &items, Some(&notch));

        // Early items should be on the left
        assert!(!layout[0].is_partitioned_past_notch);

        // An item crossing x=1412-16=1396 should jump to 1412+200+16 = 1628
        let jumped_item = layout.iter().find(|i| i.is_partitioned_past_notch);
        assert!(jumped_item.is_some());
        assert!(jumped_item.unwrap().x >= notch.right_resume_boundary());
    }
}
