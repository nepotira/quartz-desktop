//! # Finder List View Subsystem (`Cmd+2`)
//!
//! Hierarchical table view with disclosure triangles, sortable columns
//! (Name, Date Modified, Size, Kind), inline folder size calculation, and indentation.

use std::path::PathBuf;

/// Sortable column identifiers in List View.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListColumnId {
    /// File or folder name.
    Name,
    /// Date of last modification.
    DateModified,
    /// Date of creation.
    DateCreated,
    /// Byte size or folder item count.
    Size,
    /// Kind/MIME description.
    Kind,
    /// Software version string.
    Version,
    /// Spotlight comments.
    Comments,
    /// Finder colored tags.
    Tags,
}

/// Sort direction for active column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// A to Z, smallest to largest, oldest to newest.
    Ascending,
    /// Z to A, largest to smallest, newest to oldest.
    Descending,
}

/// A column configuration in List View.
#[derive(Debug, Clone, PartialEq)]
pub struct ListColumnConfig {
    /// Column identifier.
    pub id: ListColumnId,
    /// Header title.
    pub title: &'static str,
    /// Pixel width.
    pub width: f32,
    /// Whether column is currently visible in table.
    pub is_visible: bool,
}

/// A node within the hierarchical List View tree.
#[derive(Debug, Clone, PartialEq)]
pub struct ListNode {
    /// File system path.
    pub path: PathBuf,
    /// Visible display name.
    pub name: String,
    /// Whether this node represents a directory.
    pub is_directory: bool,
    /// Byte size (if directory, computed if calculate_all_sizes is active).
    pub size_bytes: Option<u64>,
    /// Date modified unix timestamp.
    pub date_modified_secs: u64,
    /// Date created unix timestamp.
    pub date_created_secs: u64,
    /// Human-readable kind description.
    pub kind_str: String,
    /// Tree indentation depth (0 for root items).
    pub depth: usize,
    /// Whether disclosure triangle is expanded open.
    pub is_expanded: bool,
    /// Child items if this directory is expanded.
    pub children: Vec<ListNode>,
    /// Whether item is selected.
    pub is_selected: bool,
}

impl ListNode {
    /// Creates a leaf file node.
    pub fn file(path: PathBuf, name: String, size_bytes: u64, date_modified_secs: u64, kind_str: String, depth: usize) -> Self {
        Self {
            path,
            name,
            is_directory: false,
            size_bytes: Some(size_bytes),
            date_modified_secs,
            date_created_secs: date_modified_secs,
            kind_str,
            depth,
            is_expanded: false,
            children: Vec::new(),
            is_selected: false,
        }
    }

    /// Creates a folder directory node.
    pub fn directory(path: PathBuf, name: String, date_modified_secs: u64, depth: usize) -> Self {
        Self {
            path,
            name,
            is_directory: true,
            size_bytes: None,
            date_modified_secs,
            date_created_secs: date_modified_secs,
            kind_str: "Folder".to_string(),
            depth,
            is_expanded: false,
            children: Vec::new(),
            is_selected: false,
        }
    }

    /// Formats size text for display (e.g. "4.2 MB", "12 items", or "--").
    pub fn formatted_size(&self) -> String {
        if let Some(bytes) = self.size_bytes {
            Self::format_bytes(bytes)
        } else if self.is_directory {
            if self.is_expanded {
                format!("{} items", self.children.len())
            } else {
                "--".to_string()
            }
        } else {
            "--".to_string()
        }
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{bytes} B")
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Settings governing List View display.
#[derive(Debug, Clone, PartialEq)]
pub struct ListViewSettings {
    /// Whether to recursively calculate folder sizes.
    pub calculate_all_sizes: bool,
    /// Active sorted column.
    pub sort_column: ListColumnId,
    /// Sort order direction.
    pub sort_direction: SortDirection,
    /// Text font size in points.
    pub font_size: f32,
    /// Row height in pixels (standard 20px, compact 16px, large 28px).
    pub row_height: f32,
}

impl Default for ListViewSettings {
    fn default() -> Self {
        Self {
            calculate_all_sizes: false,
            sort_column: ListColumnId::Name,
            sort_direction: SortDirection::Ascending,
            font_size: 13.0,
            row_height: 22.0,
        }
    }
}

/// List View tree manager providing expansion and flattened row rendering.
pub struct ListViewManager {
    settings: ListViewSettings,
    columns: Vec<ListColumnConfig>,
    root_nodes: Vec<ListNode>,
}

impl Default for ListViewManager {
    fn default() -> Self {
        Self::new(ListViewSettings::default())
    }
}

impl ListViewManager {
    /// Creates a new List View manager initialized with standard columns.
    pub fn new(settings: ListViewSettings) -> Self {
        let columns = vec![
            ListColumnConfig {
                id: ListColumnId::Name,
                title: "Name",
                width: 280.0,
                is_visible: true,
            },
            ListColumnConfig {
                id: ListColumnId::DateModified,
                title: "Date Modified",
                width: 160.0,
                is_visible: true,
            },
            ListColumnConfig {
                id: ListColumnId::Size,
                title: "Size",
                width: 90.0,
                is_visible: true,
            },
            ListColumnConfig {
                id: ListColumnId::Kind,
                title: "Kind",
                width: 140.0,
                is_visible: true,
            },
        ];

        Self {
            settings,
            columns,
            root_nodes: Vec::new(),
        }
    }

    /// Sets root nodes and sorts them.
    pub fn set_root_nodes(&mut self, mut nodes: Vec<ListNode>) {
        self.sort_nodes(&mut nodes);
        self.root_nodes = nodes;
    }

    /// Toggles disclosure expansion for a directory matching the given path.
    pub fn toggle_disclosure(&mut self, path: &PathBuf) -> bool {
        Self::toggle_recursive(&mut self.root_nodes, path)
    }

    fn toggle_recursive(nodes: &mut [ListNode], target_path: &PathBuf) -> bool {
        for node in nodes.iter_mut() {
            if &node.path == target_path && node.is_directory {
                node.is_expanded = !node.is_expanded;
                return true;
            }
            if Self::toggle_recursive(&mut node.children, target_path) {
                return true;
            }
        }
        false
    }

    /// Produces a flat list of all currently visible rows respecting disclosure expansion.
    pub fn flattened_visible_rows(&self) -> Vec<&ListNode> {
        let mut rows = Vec::new();
        for node in &self.root_nodes {
            self.collect_visible_recursive(node, &mut rows);
        }
        rows
    }

    fn collect_visible_recursive<'a>(&'a self, node: &'a ListNode, rows: &mut Vec<&'a ListNode>) {
        rows.push(node);
        if node.is_directory && node.is_expanded {
            for child in &node.children {
                self.collect_visible_recursive(child, rows);
            }
        }
    }

    fn sort_nodes(&self, nodes: &mut [ListNode]) {
        let sort_col = self.settings.sort_column;
        let sort_dir = self.settings.sort_direction;

        nodes.sort_by(|a, b| {
            // Directories always sorted first like macOS Finder
            if a.is_directory != b.is_directory {
                return b.is_directory.cmp(&a.is_directory);
            }

            let cmp = match sort_col {
                ListColumnId::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                ListColumnId::DateModified => a.date_modified_secs.cmp(&b.date_modified_secs),
                ListColumnId::Size => a.size_bytes.cmp(&b.size_bytes),
                ListColumnId::Kind => a.kind_str.cmp(&b.kind_str),
                _ => a.name.cmp(&b.name),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });

        // Recursively sort children
        for node in nodes.iter_mut() {
            if !node.children.is_empty() {
                self.sort_nodes(&mut node.children);
            }
        }
    }

    /// Access reference to columns configuration.
    pub fn columns(&self) -> &[ListColumnConfig] {
        &self.columns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_view_disclosure_expansion() {
        let mut dir = ListNode::directory(PathBuf::from("/Users/demo/Documents"), "Documents".into(), 100, 0);
        let file = ListNode::file(
            PathBuf::from("/Users/demo/Documents/report.pdf"),
            "report.pdf".into(),
            1024 * 1024 * 4,
            120,
            "PDF Document".into(),
            1,
        );
        dir.children.push(file);

        let mut manager = ListViewManager::default();
        manager.set_root_nodes(vec![dir]);

        // Before expansion: only 1 root row visible
        assert_eq!(manager.flattened_visible_rows().len(), 1);

        // Expand directory
        assert!(manager.toggle_disclosure(&PathBuf::from("/Users/demo/Documents")));

        // After expansion: 2 rows visible (directory + child file)
        assert_eq!(manager.flattened_visible_rows().len(), 2);
    }
}
