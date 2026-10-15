//! # Finder Window Master Controller Subsystem
//!
//! Coordinates the macOS Tahoe 26 Finder window: Unified Liquid Glass Toolbar,
//! Sidebar with Favorites and Mounted Drives, View Switcher (`Cmd+1`..`Cmd+4`),
//! Path Bar, Status Bar, and Filesystem Transaction Undo/Redo integration.

use crate::bundle_dmg::DiskImageManager;
use crate::desktop::DesktopStacksManager;
use crate::drag_physics::{FlockingDragManager, SpringLoadedFolderTracker};
use crate::undo::{FsOperation, FsTransactionJournal, UndoError};
use crate::views::{
    ColumnViewManager, FinderViewMode, GalleryViewManager, IconViewLayout, ListViewManager,
};
use std::path::{Path, PathBuf};

/// A section within the Finder sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarSection {
    /// Pinned user favorites (Applications, Desktop, Documents, Downloads).
    Favorites,
    /// Cloud storage sync drives.
    ICloud,
    /// Physical disks, mounted DMGs, and network shares.
    Locations,
    /// Colored Finder tags.
    Tags,
}

/// An entry in the Finder sidebar.
#[derive(Debug, Clone, PartialEq)]
pub struct SidebarItem {
    /// Section grouping.
    pub section: SidebarSection,
    /// Visible display label.
    pub label: String,
    /// Target path on disk.
    pub target_path: PathBuf,
    /// SF Symbols / icon identifier.
    pub icon_name: String,
    /// Whether item is currently active / selected.
    pub is_active: bool,
    /// Optional eject action button for removable drives.
    pub is_ejectable: bool,
}

/// State and geometry of a Finder window.
pub struct FinderWindow {
    window_id: u32,
    current_path: PathBuf,
    history_back: Vec<PathBuf>,
    history_forward: Vec<PathBuf>,
    view_mode: FinderViewMode,
    sidebar_items: Vec<SidebarItem>,
    is_sidebar_visible: bool,
    sidebar_width: f32,
    window_size: [f32; 2],

    // View Subsystems
    icon_view: IconViewLayout,
    list_view: ListViewManager,
    column_view: ColumnViewManager,
    gallery_view: GalleryViewManager,

    // Core System Engines
    journal: FsTransactionJournal,
    spring_tracker: SpringLoadedFolderTracker,
    flocking_drag: FlockingDragManager,
    dmg_manager: DiskImageManager,
    desktop_stacks: DesktopStacksManager,
}

impl FinderWindow {
    /// Creates a new Finder Window initialized at the specified directory path.
    pub fn new(window_id: u32, initial_path: PathBuf) -> Self {
        let sidebar_items = vec![
            SidebarItem {
                section: SidebarSection::Favorites,
                label: "Applications".to_string(),
                target_path: PathBuf::from("/Applications"),
                icon_name: "app.badge".to_string(),
                is_active: false,
                is_ejectable: false,
            },
            SidebarItem {
                section: SidebarSection::Favorites,
                label: "Desktop".to_string(),
                target_path: PathBuf::from("/Users/demo/Desktop"),
                icon_name: "menubar.dock.rectangle".to_string(),
                is_active: false,
                is_ejectable: false,
            },
            SidebarItem {
                section: SidebarSection::Favorites,
                label: "Documents".to_string(),
                target_path: PathBuf::from("/Users/demo/Documents"),
                icon_name: "doc.text".to_string(),
                is_active: false,
                is_ejectable: false,
            },
            SidebarItem {
                section: SidebarSection::Favorites,
                label: "Downloads".to_string(),
                target_path: PathBuf::from("/Users/demo/Downloads"),
                icon_name: "arrow.down.circle".to_string(),
                is_active: false,
                is_ejectable: false,
            },
            SidebarItem {
                section: SidebarSection::Locations,
                label: "Macintosh HD".to_string(),
                target_path: PathBuf::from("/"),
                icon_name: "internaldrive".to_string(),
                is_active: false,
                is_ejectable: false,
            },
        ];

        Self {
            window_id,
            current_path: initial_path,
            history_back: Vec::new(),
            history_forward: Vec::new(),
            view_mode: FinderViewMode::IconView,
            sidebar_items,
            is_sidebar_visible: true,
            sidebar_width: 210.0,
            window_size: [960.0, 620.0],

            icon_view: IconViewLayout::default(),
            list_view: ListViewManager::default(),
            column_view: ColumnViewManager::default(),
            gallery_view: GalleryViewManager::default(),

            journal: FsTransactionJournal::default(),
            spring_tracker: SpringLoadedFolderTracker::default(),
            flocking_drag: FlockingDragManager::default(),
            dmg_manager: DiskImageManager::default(),
            desktop_stacks: DesktopStacksManager::default(),
        }
    }

    /// Window identifier.
    #[inline]
    pub fn window_id(&self) -> u32 {
        self.window_id
    }

    /// Current working directory path.
    #[inline]
    pub fn current_path(&self) -> &Path {
        &self.current_path
    }

    /// Active view browsing mode.
    #[inline]
    pub fn view_mode(&self) -> FinderViewMode {
        self.view_mode
    }

    /// Switches active view mode (`Cmd+1` to `Cmd+4`).
    pub fn set_view_mode(&mut self, mode: FinderViewMode) {
        self.view_mode = mode;
    }

    /// Navigates to a new directory path, preserving history.
    pub fn navigate_to(&mut self, new_path: PathBuf) {
        if self.current_path != new_path {
            self.history_back.push(self.current_path.clone());
            self.history_forward.clear();
            self.current_path = new_path;
            self.sync_active_sidebar();
        }
    }

    /// Navigates back in history (`Cmd+[`).
    pub fn navigate_back(&mut self) -> bool {
        if let Some(prev) = self.history_back.pop() {
            self.history_forward.push(self.current_path.clone());
            self.current_path = prev;
            self.sync_active_sidebar();
            true
        } else {
            false
        }
    }

    /// Navigates forward in history (`Cmd+]`).
    pub fn navigate_forward(&mut self) -> bool {
        if let Some(next) = self.history_forward.pop() {
            self.history_back.push(self.current_path.clone());
            self.current_path = next;
            self.sync_active_sidebar();
            true
        } else {
            false
        }
    }

    /// Navigates up to enclosing directory (`Cmd+Up`).
    pub fn navigate_up_enclosing(&mut self) -> bool {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            if parent != self.current_path {
                self.navigate_to(parent);
                return true;
            }
        }
        false
    }

    fn sync_active_sidebar(&mut self) {
        for item in &mut self.sidebar_items {
            item.is_active = item.target_path == self.current_path;
        }
    }

    /// Executes filesystem Undo (`Cmd+Z`).
    pub fn undo_filesystem(&mut self) -> Result<FsOperation, UndoError> {
        self.journal.prepare_undo()
    }

    /// Executes filesystem Redo (`Shift+Cmd+Z`).
    pub fn redo_filesystem(&mut self) -> Result<FsOperation, UndoError> {
        self.journal.prepare_redo()
    }

    /// Records an executed filesystem action in the transaction journal.
    pub fn record_filesystem_action(&mut self, op: FsOperation) {
        self.journal.record_operation(op);
    }

    /// Access reference to Disk Image manager.
    pub fn dmg_manager(&self) -> &DiskImageManager {
        &self.dmg_manager
    }

    /// Access mutable reference to Disk Image manager.
    pub fn dmg_manager_mut(&mut self) -> &mut DiskImageManager {
        &mut self.dmg_manager
    }

    /// Access reference to Desktop Stacks manager.
    pub fn desktop_stacks(&self) -> &DesktopStacksManager {
        &self.desktop_stacks
    }

    /// Access mutable reference to Desktop Stacks manager.
    pub fn desktop_stacks_mut(&mut self) -> &mut DesktopStacksManager {
        &mut self.desktop_stacks
    }

    /// Access reference to Spring-Loaded Folder tracker.
    pub fn spring_tracker_mut(&mut self) -> &mut SpringLoadedFolderTracker {
        &mut self.spring_tracker
    }

    /// Access reference to Flocking Drag manager.
    pub fn flocking_drag_mut(&mut self) -> &mut FlockingDragManager {
        &mut self.flocking_drag
    }

    /// Whether the sidebar is currently visible.
    #[inline]
    pub fn is_sidebar_visible(&self) -> bool {
        self.is_sidebar_visible
    }

    /// Toggles sidebar visibility.
    pub fn toggle_sidebar(&mut self) -> bool {
        self.is_sidebar_visible = !self.is_sidebar_visible;
        self.is_sidebar_visible
    }

    /// Current sidebar width in pixels.
    #[inline]
    pub fn sidebar_width(&self) -> f32 {
        self.sidebar_width
    }

    /// Sets sidebar width in pixels clamped to $[160.0, 380.0]$.
    pub fn set_sidebar_width(&mut self, width: f32) {
        self.sidebar_width = width.clamp(160.0, 380.0);
    }

    /// Current window dimensions $[width, height]$.
    #[inline]
    pub fn window_size(&self) -> [f32; 2] {
        self.window_size
    }

    /// Resizes window dimensions.
    pub fn set_window_size(&mut self, size: [f32; 2]) {
        self.window_size = [size[0].max(480.0), size[1].max(320.0)];
    }

    /// Reference to Icon View layout engine.
    #[inline]
    pub fn icon_view(&self) -> &IconViewLayout {
        &self.icon_view
    }

    /// Mutable reference to Icon View layout engine.
    #[inline]
    pub fn icon_view_mut(&mut self) -> &mut IconViewLayout {
        &mut self.icon_view
    }

    /// Reference to List View manager.
    #[inline]
    pub fn list_view(&self) -> &ListViewManager {
        &self.list_view
    }

    /// Mutable reference to List View manager.
    #[inline]
    pub fn list_view_mut(&mut self) -> &mut ListViewManager {
        &mut self.list_view
    }

    /// Reference to Column View manager.
    #[inline]
    pub fn column_view(&self) -> &ColumnViewManager {
        &self.column_view
    }

    /// Mutable reference to Column View manager.
    #[inline]
    pub fn column_view_mut(&mut self) -> &mut ColumnViewManager {
        &mut self.column_view
    }

    /// Reference to Gallery View manager.
    #[inline]
    pub fn gallery_view(&self) -> &GalleryViewManager {
        &self.gallery_view
    }

    /// Mutable reference to Gallery View manager.
    #[inline]
    pub fn gallery_view_mut(&mut self) -> &mut GalleryViewManager {
        &mut self.gallery_view
    }

    /// Read-only access to sidebar items.
    pub fn sidebar_items(&self) -> &[SidebarItem] {
        &self.sidebar_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finder_window_navigation_history_and_modes() {
        let mut window = FinderWindow::new(1, PathBuf::from("/Users/demo"));
        assert_eq!(window.view_mode(), FinderViewMode::IconView);

        // Switch to Column View
        window.set_view_mode(FinderViewMode::ColumnView);
        assert_eq!(window.view_mode(), FinderViewMode::ColumnView);

        // Navigate to subfolder
        window.navigate_to(PathBuf::from("/Users/demo/Documents"));
        assert_eq!(window.current_path(), Path::new("/Users/demo/Documents"));

        // Navigate back
        assert!(window.navigate_back());
        assert_eq!(window.current_path(), Path::new("/Users/demo"));

        // Navigate forward
        assert!(window.navigate_forward());
        assert_eq!(window.current_path(), Path::new("/Users/demo/Documents"));

        // Navigate to enclosing parent
        assert!(window.navigate_up_enclosing());
        assert_eq!(window.current_path(), Path::new("/Users/demo"));
    }
}
