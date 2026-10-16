//! # Finder Icon View Subsystem (`Cmd+1`) & Bipartite Selection (Anti-Fraud Law #6)
//!
//! Freeform or grid-aligned desktop/folder icon display with dynamic icon scaling
//! (`16px` to `512px`), label positioning (`Bottom` or `Right`), snap-to-grid math,
//! and strict macOS Tahoe 26 Bipartite Selection:
//! - Behind **icon** only: neutral rounded square `rgba(255, 255, 255, 0.12)`, `R = 10.0px`, **zero blue border**.
//! - Behind **text label** only: solid blue pill `#0058D0` (`R = 4.5px`, padding `2px 6px`) with `#FFFFFF` Semibold text.

/// Label placement relative to the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    /// Label placed below the icon (standard desktop/folder layout).
    Bottom,
    /// Label placed to the right of the icon.
    Right,
}

/// Icon arrangement sorting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconArrangement {
    /// Freeform placement preserved across sessions.
    None,
    /// Automatic snap to invisible grid.
    SnapToGrid,
    /// Sorted by Name.
    SortByName,
    /// Sorted by Kind.
    SortByKind,
    /// Sorted by Date Modified.
    SortByDateModified,
    /// Sorted by Size.
    SortBySize,
}

/// Canonical macOS Tahoe 26 Bipartite Selection geometry and colors (Anti-Fraud Law #6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BipartiteSelectionGeometry {
    /// Neutral rounded square behind the icon only `[x, y, width, height]`.
    pub icon_box_rect: [f32; 4],
    /// Corner radius of the icon backdrop (`10.0px`).
    pub icon_box_radius: f32,
    /// Neutral fill color behind the icon (`rgba(255, 255, 255, 0.12)`).
    pub icon_box_rgba: [f32; 4],
    /// Must always be `false` (prohibited to draw a blue border around the icon!).
    pub has_blue_icon_border: bool,
    /// Tight pill behind the text label only `[x, y, width, height]`.
    pub label_pill_rect: [f32; 4],
    /// Corner radius of the text label pill (`4.5px`).
    pub label_pill_radius: f32,
    /// Solid blue fill of the text label pill (`#0058D0` = `[0x00, 0x58, 0xD0]`).
    pub label_pill_rgb: [u8; 3],
    /// Text color inside the selected pill (`#FFFFFF`).
    pub label_text_rgb: [u8; 3],
}

impl BipartiteSelectionGeometry {
    /// Neutral backdrop color behind the selected icon: `rgba(255, 255, 255, 0.12)`.
    pub const ICON_BOX_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 0.12];
    /// Corner radius for the icon backdrop (`10.0px`).
    pub const ICON_BOX_RADIUS: f32 = 10.0;
    /// Solid blue pill color behind the selected text label (`#0058D0`).
    pub const LABEL_PILL_RGB: [u8; 3] = [0x00, 0x58, 0xD0];
    /// Corner radius for the text label pill (`4.5px`).
    pub const LABEL_PILL_RADIUS: f32 = 4.5;
    /// Horizontal padding on each side of the text label (`6.0px`).
    pub const LABEL_PAD_X: f32 = 6.0;
    /// Vertical padding above and below the text label (`2.0px`).
    pub const LABEL_PAD_Y: f32 = 2.0;

    /// Computes the disjoint icon backdrop and text label pill rectangles for a selected item.
    pub fn compute(center: [f32; 2], icon_size: f32, text_width: f32, text_height: f32) -> Self {
        let box_dim = icon_size + 16.0;
        let icon_box_rect = [
            center[0] - box_dim * 0.5,
            center[1] - box_dim * 0.5,
            box_dim,
            box_dim,
        ];

        let pill_w = text_width + Self::LABEL_PAD_X * 2.0;
        let pill_h = text_height + Self::LABEL_PAD_Y * 2.0;
        let label_pill_rect = [
            center[0] - pill_w * 0.5,
            center[1] + box_dim * 0.5 + 4.0,
            pill_w,
            pill_h,
        ];

        Self {
            icon_box_rect,
            icon_box_radius: Self::ICON_BOX_RADIUS,
            icon_box_rgba: Self::ICON_BOX_RGBA,
            has_blue_icon_border: false,
            label_pill_rect,
            label_pill_radius: Self::LABEL_PILL_RADIUS,
            label_pill_rgb: Self::LABEL_PILL_RGB,
            label_text_rgb: [0xFF, 0xFF, 0xFF],
        }
    }
}

/// Computes the initial text selection byte range `[start, end)` when pressing `Enter` to rename a file,
/// selecting only the filename stem and excluding the file extension (Pillar 10).
pub fn rename_stem_selection_range(filename: &str, is_directory: bool) -> (usize, usize) {
    if is_directory {
        return (0, filename.len());
    }
    if let Some(dot_idx) = filename.rfind('.') {
        if dot_idx > 0 {
            return (0, dot_idx);
        }
    }
    (0, filename.len())
}

/// Settings governing the Icon View presentation.
#[derive(Debug, Clone, PartialEq)]
pub struct IconViewSettings {
    /// Icon render size in pixels $[16.0, 512.0]$.
    pub icon_size: f32,
    /// Horizontal and vertical grid cell spacing $[54.0, 128.0]$.
    pub grid_spacing: f32,
    /// Text label size in points $[10.0, 16.0]$.
    pub label_font_size: f32,
    /// Where label text is placed relative to icon.
    pub label_position: LabelPosition,
    /// Whether item info (e.g. image dimensions, item count) is displayed under the label (`Cmd+J`).
    pub show_item_info: bool,
    /// Whether live thumbnail previews are generated.
    pub show_icon_preview: bool,
    /// Arrangement mode.
    pub arrangement: IconArrangement,
}

impl Default for IconViewSettings {
    fn default() -> Self {
        Self {
            icon_size: 64.0,
            grid_spacing: 80.0,
            label_font_size: 12.0,
            label_position: LabelPosition::Bottom,
            show_item_info: true,
            show_icon_preview: true,
            arrangement: IconArrangement::SnapToGrid,
        }
    }
}

/// A positioned file icon within Icon View.
#[derive(Debug, Clone, PartialEq)]
pub struct PositionedIcon {
    /// Identifier or file path.
    pub item_id: String,
    /// Display name.
    pub name: String,
    /// Secondary info text (e.g. `"1,920 × 1,080"` or `"12 items"`).
    pub info_text: Option<String>,
    /// Canvas coordinates $(x, y)$ in pixels.
    pub pos: [f32; 2],
    /// Whether item is currently selected.
    pub is_selected: bool,
}

/// Layout engine for Icon View computing grid alignments and bipartite selection bounds.
pub struct IconViewLayout {
    settings: IconViewSettings,
    items: Vec<PositionedIcon>,
}

impl Default for IconViewLayout {
    fn default() -> Self {
        Self::new(IconViewSettings::default())
    }
}

impl IconViewLayout {
    /// Creates a new Icon View layout.
    pub fn new(settings: IconViewSettings) -> Self {
        Self {
            settings,
            items: Vec::new(),
        }
    }

    /// Access reference to settings.
    pub fn settings(&self) -> &IconViewSettings {
        &self.settings
    }

    /// Access mutable reference to settings.
    pub fn settings_mut(&mut self) -> &mut IconViewSettings {
        &mut self.settings
    }

    /// Sets items and computes layout positions based on current arrangement.
    pub fn set_items(&mut self, mut items: Vec<PositionedIcon>, container_width: f32) {
        match self.settings.arrangement {
            IconArrangement::SnapToGrid => {
                for item in &mut items {
                    item.pos = self.snap_to_grid(item.pos);
                }
            }
            IconArrangement::SortByName => {
                items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                self.arrange_in_columns(&mut items, container_width);
            }
            _ => {}
        }
        self.items = items;
    }

    /// Snaps an arbitrary $(x, y)$ coordinate to the nearest grid cell center.
    pub fn snap_to_grid(&self, pos: [f32; 2]) -> [f32; 2] {
        let cell_w = self.settings.grid_spacing;
        let cell_h = self.settings.grid_spacing + 24.0;

        let col = (pos[0] / cell_w).round();
        let row = (pos[1] / cell_h).round();

        [col * cell_w + (cell_w / 2.0), row * cell_h + (cell_h / 2.0)]
    }

    fn arrange_in_columns(&self, items: &mut [PositionedIcon], container_width: f32) {
        let cell_w = self.settings.grid_spacing;
        let cell_h = self.settings.grid_spacing + 28.0;
        let padding = 24.0;

        let cols = ((container_width - padding * 2.0) / cell_w).floor().max(1.0) as usize;

        for (idx, item) in items.iter_mut().enumerate() {
            let col = idx % cols;
            let row = idx / cols;

            item.pos = [
                padding + (col as f32 * cell_w) + (cell_w / 2.0),
                padding + (row as f32 * cell_h) + (cell_h / 2.0),
            ];
        }
    }

    /// Returns all positioned icons.
    pub fn items(&self) -> &[PositionedIcon] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_view_grid_snapping_and_bipartite_selection() {
        let layout = IconViewLayout::new(IconViewSettings {
            grid_spacing: 100.0,
            ..Default::default()
        });

        let snapped = layout.snap_to_grid([110.0, 130.0]);
        assert_eq!(snapped[0], 150.0);

        let sel = BipartiteSelectionGeometry::compute([200.0, 200.0], 68.0, 72.0, 14.0);
        assert!(!sel.has_blue_icon_border);
        assert_eq!(sel.icon_box_radius, 10.0);
        assert_eq!(sel.label_pill_radius, 4.5);
        assert_eq!(sel.label_pill_rgb, [0x00, 0x58, 0xD0]);
        assert_eq!(sel.label_pill_rect[2], 72.0 + 12.0);
        assert_eq!(sel.label_pill_rect[3], 14.0 + 4.0);
    }

    #[test]
    fn test_rename_stem_selection_excludes_extension() {
        assert_eq!(rename_stem_selection_range("Relatorio_Final.pdf", false), (0, 15));
        assert_eq!(rename_stem_selection_range(".zshrc", false), (0, 6));
        assert_eq!(rename_stem_selection_range("Applications.app", true), (0, 16));
    }
}
