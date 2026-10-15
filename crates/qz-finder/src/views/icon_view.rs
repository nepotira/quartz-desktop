//! # Finder Icon View Subsystem (`Cmd+1`)
//!
//! Freeform or grid-aligned desktop/folder icon display with dynamic icon scaling
//! (16px to 512px), label positioning (Bottom or Right), and snap-to-grid math.

/// Label placement relative to the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPosition {
    /// Label placed below the icon (standard desktop layout).
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
    /// Whether item info (e.g. image dimensions, item count) is displayed under the label.
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
    /// Secondary info text (e.g. "1,920 × 1,080" or "12 items").
    pub info_text: Option<String>,
    /// Canvas coordinates $(x, y)$ in pixels.
    pub pos: [f32; 2],
    /// Whether item is currently selected.
    pub is_selected: bool,
}

/// Layout engine for Icon View computing grid alignments.
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
    fn test_icon_view_grid_snapping() {
        let layout = IconViewLayout::new(IconViewSettings {
            grid_spacing: 100.0,
            ..Default::default()
        });

        let snapped = layout.snap_to_grid([110.0, 130.0]);
        assert_eq!(snapped[0], 150.0);
    }
}
