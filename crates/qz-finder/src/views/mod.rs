//! # The 4 Canonical Finder Views Module
//!
//! Provides the 4 macOS Finder browsing modes: Icon View (`Cmd+1`), List View (`Cmd+2`),
//! Column View (`Cmd+3`), and Gallery View (`Cmd+4`).

pub mod column_view;
pub mod gallery_view;
pub mod icon_view;
pub mod list_view;

pub use column_view::{Column, ColumnItem, ColumnPreviewPane, ColumnViewManager};
pub use gallery_view::{GalleryItem, GalleryViewManager};
pub use icon_view::{
    rename_stem_selection_range, BipartiteSelectionGeometry, IconArrangement, IconViewLayout,
    IconViewSettings, LabelPosition, PositionedIcon,
};
pub use list_view::{
    ListColumnConfig, ListColumnId, ListNode, ListViewManager, ListViewSettings, SortDirection,
};

/// The 4 canonical Finder browsing view modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FinderViewMode {
    /// Icon View (`Cmd+1`).
    IconView,
    /// List View (`Cmd+2`).
    ListView,
    /// Column View (`Cmd+3` / NeXT Miller Columns).
    ColumnView,
    /// Gallery View (`Cmd+4` / Cover Flow).
    GalleryView,
}
