//! # QuartzLinux Finder Subsystem (`qz-finder`)
//!
//! Complete, production-grade file manager and desktop manager for QuartzLinux,
//! visually, mathematically, and functionally indistinguishable from macOS Tahoe 26:
//!
//! 1. **Desktop Manager & Stacks (`desktop`)**:
//!    - Freeform desktop icon arrangement and tidy automatic Stacks (by Kind, Date, or Tags).
//!    - Spring-animated fan expansion and collapse driven by $C^2$ continuous spring physics.
//!
//! 2. **The 4 Canonical Finder Views (`views`)**:
//!    - **Icon View (`Cmd+1`)**: Freeform or grid layout, dynamic icon scaling (16px to 512px),
//!      label positioning (Bottom or Right), and snap-to-grid alignment.
//!    - **List View (`Cmd+2`)**: Hierarchical directory tree with disclosure triangles, sortable
//!      columns (Name, Date Modified, Size, Kind), and inline folder size calculation.
//!    - **Column View (`Cmd+3` / NeXT Miller Columns)**: Infinite horizontal hierarchy, keyboard
//!      navigation (Left/Right/Up/Down), selection path chains, and rightmost File Preview pane.
//!    - **Gallery View (`Cmd+4` / Cover Flow)**: Prominent central visual carousel, horizontal
//!      thumbnail track, and collapsible Inspector sidebar with metadata and quick actions.
//!
//! 3. **Filesystem Transaction & Undo/Redo Engine (`undo`)**:
//!    - Full `Cmd+Z` / `Shift+Cmd+Z` transactional rollback journal for Move, Copy, Rename,
//!      Trash, Put Back, and Batch Rename operations.
//!
//! 4. **Spring-Loaded Folders & Flocking Drag Physics (`drag_physics`)**:
//!    - 500ms hover lingering (or Spacebar tap) triggers spring expansion of destination folders.
//!    - Multi-item flocking drag physics gathering items elastically toward cursor with angular splay.
//!
//! 5. **Tahoe 26 Batch Rename Engine (`batch_rename`)**:
//!    - Replace Text, Add Text, and Format (Index, Counter, Date) with live preview and collision detection.
//!
//! 6. **`.app` Bundle & `.dmg` Disk Image Handler (`bundle_dmg`)**:
//!    - macOS bundle structure resolution (`Contents/Info.plist`, `Contents/MacOS/`).
//!    - Loopback mounting into `/Volumes/` with ejection lifecycle.
//!
//! 7. **Finder Window Controller (`window`)**:
//!    - Unified Liquid Glass toolbar, sidebar with favorites and locations, path bar, and status bar.

#![deny(missing_docs)]

pub mod batch_rename;
pub mod bundle_dmg;
pub mod desktop;
pub mod drag_physics;
pub mod undo;
pub mod views;
pub mod window;

// Re-exports
pub use batch_rename::{
    BatchRenameEngine, BatchRenameMode, FormatStyle, RenamePreviewItem, TextPlacement,
};
pub use bundle_dmg::{AppBundleMetadata, BundleDmgError, DiskImageManager, MountedVolume};
pub use desktop::{
    DesktopItem, DesktopStack, DesktopStacksManager, StackGrouping, StackKindCategory,
};
pub use drag_physics::{
    FlockingDragManager, FlockingItem, SpringLoadedFolderTracker,
};
pub use undo::{FsOperation, FsTransactionJournal, UndoError};
pub use views::{
    Column, ColumnItem, ColumnPreviewPane, ColumnViewManager, FinderViewMode, GalleryItem,
    GalleryViewManager, IconArrangement, IconViewLayout, IconViewSettings, LabelPosition,
    ListColumnConfig, ListColumnId, ListNode, ListViewManager, ListViewSettings, PositionedIcon,
    SortDirection,
};
pub use window::{FinderWindow, SidebarItem, SidebarSection};
