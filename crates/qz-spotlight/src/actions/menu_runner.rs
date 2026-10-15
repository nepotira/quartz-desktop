//! # Active Application Menu Runner Subsystem
//!
//! Traverses the active application's hierarchical menubar, flattening commands
//! into searchable breadcrumb paths (`File > Export As > PDF...`) with keyboard shortcuts.

use qz_shell::menubar::global_menu::MenuNode;

/// A flattened menu command ready for execution from Spotlight.
#[derive(Debug, Clone, PartialEq)]
pub struct FlattenedMenuItem {
    /// Menu item unique identifier.
    pub item_id: u32,
    /// Full hierarchical breadcrumb path (e.g. "File > Export As > PDF...").
    pub breadcrumb_path: String,
    /// Direct action title.
    pub action_title: String,
    /// Associated keyboard shortcut string if available.
    pub shortcut: Option<String>,
}

/// Search hit result for an application menu item.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuActionHit {
    /// Target menu item unique identifier.
    pub item_id: u32,
    /// Hierarchical breadcrumb path.
    pub breadcrumb_path: String,
    /// Direct action title.
    pub action_title: String,
    /// Optional shortcut accelerator.
    pub shortcut: Option<String>,
    /// Match score.
    pub score: f32,
}

/// Menubar runner providing flattening and fuzzy search over active application menus.
pub struct MenuRunner {
    flattened_items: Vec<FlattenedMenuItem>,
}

impl Default for MenuRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuRunner {
    /// Creates a new empty Menu Runner.
    pub fn new() -> Self {
        Self {
            flattened_items: Vec::new(),
        }
    }

    /// Replaces the current indexed menubar with menus from the active application.
    pub fn update_active_menus(&mut self, menus: &[MenuNode]) {
        self.flattened_items.clear();
        for top_menu in menus {
            self.flatten_recursive(top_menu, &[]);
        }
    }

    fn flatten_recursive(&mut self, node: &MenuNode, parent_path: &[&str]) {
        let mut current_path = parent_path.to_vec();
        current_path.push(node.label.as_str());

        if node.children.is_empty() {
            // Leaf action node
            let breadcrumb = current_path.join(" > ");
            self.flattened_items.push(FlattenedMenuItem {
                item_id: node.id,
                breadcrumb_path: breadcrumb,
                action_title: node.label.clone(),
                shortcut: node.shortcut.clone(),
            });
        } else {
            // Submenu category node
            for child in &node.children {
                self.flatten_recursive(child, &current_path);
            }
        }
    }

    /// Searches active application menu items matching query string.
    pub fn search(&self, query: &str) -> Vec<MenuActionHit> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            return self
                .flattened_items
                .iter()
                .map(|item| MenuActionHit {
                    item_id: item.item_id,
                    breadcrumb_path: item.breadcrumb_path.clone(),
                    action_title: item.action_title.clone(),
                    shortcut: item.shortcut.clone(),
                    score: 100.0,
                })
                .collect();
        }

        let mut hits = Vec::new();

        for item in &self.flattened_items {
            let action_lower = item.action_title.to_lowercase();
            let breadcrumb_lower = item.breadcrumb_path.to_lowercase();

            let mut score = 0.0;

            if action_lower == trimmed {
                score = 1000.0;
            } else if action_lower.starts_with(&trimmed) {
                score = 800.0;
            } else if action_lower.contains(&trimmed) {
                score = 600.0;
            } else if breadcrumb_lower.contains(&trimmed) {
                score = 450.0;
            }

            if score > 0.0 {
                hits.push(MenuActionHit {
                    item_id: item.item_id,
                    breadcrumb_path: item.breadcrumb_path.clone(),
                    action_title: item.action_title.clone(),
                    shortcut: item.shortcut.clone(),
                    score,
                });
            }
        }

        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_runner_flatten_and_search() {
        let menus = vec![
            MenuNode::category(
                100,
                "File",
                vec![
                    MenuNode::action(101, "New Window", Some("⌘N")),
                    MenuNode::action(102, "Open...", Some("⌘O")),
                    MenuNode::category(
                        103,
                        "Export As",
                        vec![
                            MenuNode::action(104, "PDF Document...", None),
                            MenuNode::action(105, "Plain Text...", None),
                        ],
                    ),
                ],
            ),
            MenuNode::category(
                200,
                "Edit",
                vec![
                    MenuNode::action(201, "Undo", Some("⌘Z")),
                    MenuNode::action(202, "Redo", Some("⇧⌘Z")),
                ],
            ),
        ];

        let mut runner = MenuRunner::new();
        runner.update_active_menus(&menus);

        // Search for "pdf"
        let pdf_hits = runner.search("pdf");
        assert_eq!(pdf_hits.len(), 1);
        assert_eq!(pdf_hits[0].item_id, 104);
        assert_eq!(pdf_hits[0].breadcrumb_path, "File > Export As > PDF Document...");

        // Search for "undo"
        let undo_hits = runner.search("undo");
        assert_eq!(undo_hits.len(), 1);
        assert_eq!(undo_hits[0].action_title, "Undo");
        assert_eq!(undo_hits[0].shortcut, Some("⌘Z".to_string()));
    }
}
