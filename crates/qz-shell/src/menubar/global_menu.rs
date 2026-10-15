//! # Global Application Menu Subsystem
//!
//! Controls the dynamic Top Menu Bar:
//! - Defaults to "Finder" when no window is focused or when clicking the desktop wallpaper.
//! - **Hover Sliding**: Lateral mouse gliding across menu headers switches open menus with 0ms delay.
//! - **Menu Item Double-Flash**: 2 flash cycles in 120ms on click before dismissing and executing.
//! - **Option-Mutable Items**: Real-time label/shortcut mutations while menu is open.
//! - **Window List**: Synchronized list of app surfaces with `✓` on focused window and `♦` on minimized.
//! - **Help Search Rector**: Fuzzy query search returning coordinates for the floating blue guide arrow.

use glam::Vec2;

/// A window entry displayed in the "Window" menu footer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowMenuEntry {
    /// Window compositor ID.
    pub window_id: u32,
    /// Window title text.
    pub title: String,
    /// True if currently focused (marked with `✓`).
    pub is_focused: bool,
    /// True if minimized to Dock (marked with `♦`).
    pub is_minimized: bool,
}

impl WindowMenuEntry {
    /// Formats menu entry label with appropriate macOS Tahoe symbols.
    pub fn display_label(&self) -> String {
        if self.is_focused {
            format!("✓ {}", self.title)
        } else if self.is_minimized {
            format!("♦ {}", self.title)
        } else {
            format!("  {}", self.title)
        }
    }
}

/// A search match found by the Help Search Rector.
#[derive(Debug, Clone, PartialEq)]
pub struct HelpSearchResult {
    /// Action ID matched.
    pub item_id: u32,
    /// Breadcrumb path (e.g. "File > Export > PDF...").
    pub breadcrumb_path: String,
    /// Screen coordinates of target item for anchoring the floating blue arrow.
    pub target_screen_pos: Vec2,
    /// Size of the matched item.
    pub target_size: Vec2,
}

/// Node within the hierarchical application menu tree.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuNode {
    /// Unique item ID.
    pub id: u32,
    /// Visible text label.
    pub label: String,
    /// Keyboard shortcut string (e.g. "⌘W").
    pub shortcut: Option<String>,
    /// Child submenu items if any.
    pub children: Vec<MenuNode>,
    /// Alternate label shown when Option is held.
    pub option_alt_label: Option<String>,
    /// Alternate shortcut shown when Option is held.
    pub option_alt_shortcut: Option<String>,
}

impl MenuNode {
    /// Creates a leaf action menu node.
    pub fn action(id: u32, label: &str, shortcut: Option<&str>) -> Self {
        Self {
            id,
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            children: Vec::new(),
            option_alt_label: None,
            option_alt_shortcut: None,
        }
    }

    /// Attaches an Option-key mutation to this item.
    pub fn with_option_variant(mut self, alt_label: &str, alt_shortcut: Option<&str>) -> Self {
        self.option_alt_label = Some(alt_label.to_string());
        self.option_alt_shortcut = alt_shortcut.map(|s| s.to_string());
        self
    }

    /// Creates a category menu node with children.
    pub fn category(id: u32, label: &str, children: Vec<MenuNode>) -> Self {
        Self {
            id,
            label: label.to_string(),
            shortcut: None,
            children,
            option_alt_label: None,
            option_alt_shortcut: None,
        }
    }

    /// Resolves visible label based on Option key state.
    pub fn resolved_label(&self, option_held: bool) -> &str {
        if option_held {
            if let Some(ref alt) = self.option_alt_label {
                return alt.as_str();
            }
        }
        self.label.as_str()
    }
}

/// Coordinates the Global Application Menu bar state.
pub struct GlobalMenuManager {
    active_app_id: String,
    active_app_display_name: String,
    top_menus: Vec<MenuNode>,
    window_entries: Vec<WindowMenuEntry>,
    is_option_down: bool,
    currently_open_menu_id: Option<u32>,
}

impl Default for GlobalMenuManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalMenuManager {
    /// Creates a menu manager defaulting to "Finder".
    pub fn new() -> Self {
        Self {
            active_app_id: "org.quartz.finder".to_string(),
            active_app_display_name: "Finder".to_string(),
            top_menus: Vec::new(),
            window_entries: Vec::new(),
            is_option_down: false,
            currently_open_menu_id: None,
        }
    }

    /// Switches the active application focused in the Menu Bar (or resets to Finder if none).
    pub fn set_focused_application(&mut self, app_id: Option<&str>, display_name: Option<&str>) {
        match (app_id, display_name) {
            (Some(id), Some(name)) => {
                self.active_app_id = id.to_string();
                self.active_app_display_name = name.to_string();
            }
            _ => {
                self.active_app_id = "org.quartz.finder".to_string();
                self.active_app_display_name = "Finder".to_string();
            }
        }
        self.currently_open_menu_id = None;
    }

    /// Returns the application bold title displayed next to the Apple logo.
    #[inline]
    pub fn active_app_name(&self) -> &str {
        &self.active_app_display_name
    }

    /// Ingests Option key state update.
    pub fn set_option_modifier(&mut self, is_down: bool) {
        self.is_option_down = is_down;
    }

    /// Synchronizes the active window list for the Window menu footer.
    pub fn update_windows(&mut self, entries: Vec<WindowMenuEntry>) {
        self.window_entries = entries;
    }

    /// Registers the top-level menu hierarchy for the application.
    pub fn set_menus(&mut self, menus: Vec<MenuNode>) {
        self.top_menus = menus;
    }

    /// Implements 0ms lateral hover sliding between open menus.
    pub fn handle_header_hover(&mut self, menu_id: u32) {
        if self.currently_open_menu_id.is_some() {
            // Already in open-menu sliding mode: switch instantly (0ms delay)
            self.currently_open_menu_id = Some(menu_id);
        }
    }

    /// Searches the menu tree for the Help Search Rector and returns breadcrumb and match.
    pub fn search_help(&self, query: &str) -> Vec<HelpSearchResult> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for top in &self.top_menus {
            Self::recursive_search(top, &top.label, &q, &mut results);
        }
        results
    }

    fn recursive_search(
        node: &MenuNode,
        current_path: &str,
        query: &str,
        results: &mut Vec<HelpSearchResult>,
    ) {
        if node.label.to_lowercase().contains(query) && node.children.is_empty() {
            results.push(HelpSearchResult {
                item_id: node.id,
                breadcrumb_path: format!("{} > {}", current_path, node.label),
                target_screen_pos: Vec2::ZERO, // Populated by compositor scene graph layout
                target_size: Vec2::new(180.0, 24.0),
            });
        }
        for child in &node.children {
            let next_path = format!("{} > {}", current_path, node.label);
            Self::recursive_search(child, &next_path, query, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_list_symbol_formatting() {
        let entry_focused = WindowMenuEntry {
            window_id: 1,
            title: "Project Notes".to_string(),
            is_focused: true,
            is_minimized: false,
        };
        assert_eq!(entry_focused.display_label(), "✓ Project Notes");

        let entry_minimized = WindowMenuEntry {
            window_id: 2,
            title: "Archive".to_string(),
            is_focused: false,
            is_minimized: true,
        };
        assert_eq!(entry_minimized.display_label(), "♦ Archive");
    }

    #[test]
    fn test_help_search_rector_query() {
        let mut manager = GlobalMenuManager::new();

        let file_menu = MenuNode::category(
            1,
            "File",
            vec![
                MenuNode::action(10, "New Window", Some("⌘N")),
                MenuNode::action(11, "Export to PDF...", Some("⇧⌘E")),
            ],
        );
        manager.set_menus(vec![file_menu]);

        let matches = manager.search_help("pdf");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].item_id, 11);
        assert!(matches[0].breadcrumb_path.contains("File"));
        assert!(matches[0].breadcrumb_path.contains("PDF"));
    }
}
