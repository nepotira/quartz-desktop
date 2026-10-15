//! # Finder Column View Subsystem (`Cmd+3` / NeXT Miller Columns)
//!
//! NeXT Miller Columns providing an infinite horizontal hierarchy with keyboard navigation
//! (Left/Right/Up/Down), selection path chains, and rightmost File Preview metadata pane.

use std::path::PathBuf;

/// An item within a single column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnItem {
    /// File system path.
    pub path: PathBuf,
    /// Display file name.
    pub name: String,
    /// Whether item is a directory (navigates deeper into next column).
    pub is_directory: bool,
    /// Byte size for files.
    pub size_bytes: u64,
    /// Modification unix timestamp.
    pub modified_secs: u64,
    /// Kind description.
    pub kind_str: String,
    /// Optional preview image path or thumbnail.
    pub preview_image_path: Option<String>,
}

/// A single Miller column representing the contents of a directory.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Directory path this column represents.
    pub dir_path: PathBuf,
    /// Ordered list of directory entries.
    pub items: Vec<ColumnItem>,
    /// Selected index within this column.
    pub selected_index: Option<usize>,
    /// Pixel column width (standard 220px to 320px).
    pub width: f32,
}

impl Column {
    /// Creates a new column for a directory.
    pub fn new(dir_path: PathBuf, items: Vec<ColumnItem>, width: f32) -> Self {
        Self {
            dir_path,
            items,
            selected_index: None,
            width: width.clamp(180.0, 480.0),
        }
    }

    /// Selected item in this column if any.
    pub fn selected_item(&self) -> Option<&ColumnItem> {
        self.selected_index.and_then(|idx| self.items.get(idx))
    }
}

/// Metadata attributes and quick actions for the rightmost Preview Column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnPreviewPane {
    /// File path being inspected.
    pub file_path: PathBuf,
    /// Display name.
    pub title: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Kind/MIME description.
    pub kind_str: String,
    /// Modification date string.
    pub modified_date_str: String,
    /// Creation date string.
    pub created_date_str: String,
    /// Dimensions for media/images if known (e.g. "3840 × 2160").
    pub dimensions: Option<String>,
    /// Available quick action buttons (e.g. ["Markup", "Rotate Left", "Create PDF"]).
    pub quick_actions: Vec<String>,
}

/// Miller Columns controller coordinating horizontal chain and keyboard navigation.
pub struct ColumnViewManager {
    columns: Vec<Column>,
    active_column_index: usize,
    preview_pane: Option<ColumnPreviewPane>,
    scroll_x_offset: f32,
}

impl Default for ColumnViewManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ColumnViewManager {
    /// Creates an empty Column View manager.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            active_column_index: 0,
            preview_pane: None,
            scroll_x_offset: 0.0,
        }
    }

    /// Initializes columns with a root directory column.
    pub fn set_root_column(&mut self, root_path: PathBuf, items: Vec<ColumnItem>) {
        self.columns.clear();
        self.columns.push(Column::new(root_path, items, 240.0));
        self.active_column_index = 0;
        self.preview_pane = None;
        self.scroll_x_offset = 0.0;
    }

    /// Selects an item in the active column by index.
    pub fn select_item_in_active(&mut self, item_index: usize) -> Option<PathBuf> {
        if self.active_column_index >= self.columns.len() {
            return None;
        }

        let col = &mut self.columns[self.active_column_index];
        if item_index >= col.items.len() {
            return None;
        }

        col.selected_index = Some(item_index);
        let selected_item = col.items[item_index].clone();

        // Truncate subsequent columns beyond active
        self.columns.truncate(self.active_column_index + 1);

        if selected_item.is_directory {
            self.preview_pane = None;
            Some(selected_item.path)
        } else {
            // Leaf file: populate preview pane
            self.preview_pane = Some(ColumnPreviewPane {
                file_path: selected_item.path.clone(),
                title: selected_item.name,
                size_bytes: selected_item.size_bytes,
                kind_str: selected_item.kind_str,
                modified_date_str: "Today".to_string(),
                created_date_str: "Today".to_string(),
                dimensions: None,
                quick_actions: vec!["Markup".to_string(), "Create PDF".to_string()],
            });
            None
        }
    }

    /// Appends a new child directory column to the right.
    pub fn push_child_column(&mut self, dir_path: PathBuf, items: Vec<ColumnItem>) {
        self.columns.push(Column::new(dir_path, items, 240.0));
        self.active_column_index = self.columns.len() - 1;
        self.update_scroll_offset();
    }

    /// Navigates up in active column.
    pub fn navigate_up(&mut self) {
        if let Some(col) = self.columns.get_mut(self.active_column_index) {
            if col.items.is_empty() {
                return;
            }
            let next_idx = match col.selected_index {
                Some(idx) => {
                    if idx == 0 {
                        col.items.len() - 1
                    } else {
                        idx - 1
                    }
                }
                None => 0,
            };
            col.selected_index = Some(next_idx);
        }
    }

    /// Navigates down in active column.
    pub fn navigate_down(&mut self) {
        if let Some(col) = self.columns.get_mut(self.active_column_index) {
            if col.items.is_empty() {
                return;
            }
            let next_idx = match col.selected_index {
                Some(idx) => (idx + 1) % col.items.len(),
                None => 0,
            };
            col.selected_index = Some(next_idx);
        }
    }

    /// Navigates left to parent column.
    pub fn navigate_left(&mut self) {
        if self.active_column_index > 0 {
            self.active_column_index -= 1;
            self.update_scroll_offset();
        }
    }

    /// Navigates right into child column if active item is a directory.
    pub fn navigate_right(&mut self) {
        if self.active_column_index + 1 < self.columns.len() {
            self.active_column_index += 1;
            self.update_scroll_offset();
        }
    }

    fn update_scroll_offset(&mut self) {
        let total_cols_w: f32 = self.columns.iter().map(|c| c.width).sum();
        self.scroll_x_offset = (total_cols_w - 800.0).max(0.0);
    }

    /// Returns all columns in the horizontal chain.
    pub fn columns(&self) -> &[Column] {
        &self.columns
    }

    /// Active preview pane if a file is selected.
    pub fn preview_pane(&self) -> Option<&ColumnPreviewPane> {
        self.preview_pane.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miller_column_navigation() {
        let mut manager = ColumnViewManager::new();

        let root_items = vec![
            ColumnItem {
                path: PathBuf::from("/Users/demo/Projects"),
                name: "Projects".into(),
                is_directory: true,
                size_bytes: 0,
                modified_secs: 100,
                kind_str: "Folder".into(),
                preview_image_path: None,
            },
            ColumnItem {
                path: PathBuf::from("/Users/demo/notes.txt"),
                name: "notes.txt".into(),
                is_directory: false,
                size_bytes: 512,
                modified_secs: 100,
                kind_str: "Plain Text".into(),
                preview_image_path: None,
            },
        ];

        manager.set_root_column(PathBuf::from("/Users/demo"), root_items);

        // Select directory "Projects"
        let child_path = manager.select_item_in_active(0);
        assert_eq!(child_path, Some(PathBuf::from("/Users/demo/Projects")));

        // Push child column
        manager.push_child_column(
            PathBuf::from("/Users/demo/Projects"),
            vec![ColumnItem {
                path: PathBuf::from("/Users/demo/Projects/main.rs"),
                name: "main.rs".into(),
                is_directory: false,
                size_bytes: 2048,
                modified_secs: 100,
                kind_str: "Rust Source".into(),
                preview_image_path: None,
            }],
        );

        assert_eq!(manager.columns().len(), 2);

        // Select file in child column
        manager.select_item_in_active(0);
        assert!(manager.preview_pane().is_some());
        assert_eq!(manager.preview_pane().unwrap().title, "main.rs");
    }
}
