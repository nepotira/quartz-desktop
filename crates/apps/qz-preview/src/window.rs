//! # Preview Window Controller & Liquid Glass Chrome (`qz-preview::window`)
//!
//! Production-grade window controller for macOS Tahoe 26 Preview (`Preview.app`).
//!
//! ### Key Capabilities:
//! - **Unified Titlebar Chrome:** Traffic lights with dirty state bullet dot (`•`),
//!   Document Proxy Icon (supporting drag-to-retarget into Finder or Mail), and centered title.
//! - **Floating Liquid Glass Markup Toolbar:** Collapsible secondary pill toolbar providing
//!   instant access to sketch, shapes, text, highlighter, and signature tools.
//! - **Sidebar Drawer:** Multi-mode panel supporting Page Thumbnails, Table of Contents (Outline),
//!   and Highlights & Notes annotations list.
//! - **Interactive Viewport:** Canvas container routing multi-touch zoom and pan gestures.

use crate::document::{DocumentModel, ViewMode};
use crate::export::{DocumentExporter, ExportOptions, ExportReport};
use crate::markup::{MarkupManager, MarkupTool};
use crate::signature::SignatureVault;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Sidebar navigation tab classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SidebarTab {
    /// Visual page thumbnail cards with reordering support.
    Thumbnails,
    /// PDF Table of Contents bookmarks outline.
    TableOfContents,
    /// Filtered list of user highlights, notes, and vector annotations.
    Annotations,
}

/// Drag-and-drop state for the Document Proxy Icon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProxyIconDragState {
    /// Proxy icon is idle in the titlebar.
    Idle,
    /// Drag gesture initiated on proxy icon, carrying source file path.
    Dragging {
        /// Absolute source file path being dragged.
        file_path: String,
        /// Current cursor position during drag.
        cursor_pos: Vec2,
    },
}

/// Window controller managing Preview's complete UI shell and document session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewWindowController {
    /// Underlying document model.
    pub document: DocumentModel,
    /// Active markup manager and undo/redo engine.
    pub markup: MarkupManager,
    /// Persistent trackpad signature vault.
    pub signature_vault: SignatureVault,
    /// Window frame dimensions $[W, H]$ in points.
    pub window_size: Vec2,
    /// Window screen origin $[x, y]$.
    pub window_origin: Vec2,
    /// Whether the sidebar drawer is open.
    pub is_sidebar_open: bool,
    /// Active tab inside the sidebar.
    pub sidebar_tab: SidebarTab,
    /// Width of the sidebar drawer in points (default: 220.0).
    pub sidebar_width: f32,
    /// Whether the secondary Liquid Glass markup toolbar is visible.
    pub is_markup_bar_visible: bool,
    /// Whether window is in full-screen mode.
    pub is_fullscreen: bool,
    /// Document proxy icon drag state.
    pub proxy_icon_drag: ProxyIconDragState,
}

impl PreviewWindowController {
    /// Creates a new `PreviewWindowController` with an open document.
    #[must_use]
    pub fn new(document: DocumentModel, window_size: Vec2) -> Self {
        Self {
            document,
            markup: MarkupManager::new(),
            signature_vault: SignatureVault::new(),
            window_size,
            window_origin: Vec2::new(100.0, 100.0),
            is_sidebar_open: true,
            sidebar_tab: SidebarTab::Thumbnails,
            sidebar_width: 220.0,
            is_markup_bar_visible: false,
            is_fullscreen: false,
            proxy_icon_drag: ProxyIconDragState::Idle,
        }
    }

    /// Toggles sidebar drawer visibility (`Cmd+Option+1`).
    pub fn toggle_sidebar(&mut self) {
        self.is_sidebar_open = !self.is_sidebar_open;
    }

    /// Sets active sidebar tab.
    pub fn set_sidebar_tab(&mut self, tab: SidebarTab) {
        self.sidebar_tab = tab;
        self.is_sidebar_open = true;
    }

    /// Toggles secondary Liquid Glass markup toolbar visibility (`Cmd+Shift+A`).
    pub fn toggle_markup_bar(&mut self) {
        self.is_markup_bar_visible = !self.is_markup_bar_visible;
        if !self.is_markup_bar_visible {
            self.markup.active_tool = MarkupTool::Select;
        }
    }

    /// Formats window title string with dirty state indicator (`•` bullet).
    #[must_use]
    pub fn window_title(&self) -> String {
        if self.document.is_dirty() {
            format!("{} — Edited", self.document.display_name)
        } else {
            self.document.display_name.clone()
        }
    }

    /// Initiates proxy icon drag interaction.
    pub fn start_proxy_icon_drag(&mut self, cursor_pos: Vec2) {
        self.proxy_icon_drag = ProxyIconDragState::Dragging {
            file_path: self.document.file_path.clone(),
            cursor_pos,
        };
    }

    /// Updates proxy icon drag position.
    pub fn update_proxy_icon_drag(&mut self, cursor_pos: Vec2) {
        if let ProxyIconDragState::Dragging { ref file_path, .. } = self.proxy_icon_drag {
            self.proxy_icon_drag = ProxyIconDragState::Dragging {
                file_path: file_path.clone(),
                cursor_pos,
            };
        }
    }

    /// Ends proxy icon drag interaction.
    pub fn end_proxy_icon_drag(&mut self) {
        self.proxy_icon_drag = ProxyIconDragState::Idle;
    }

    /// Changes document presentation layout mode.
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.document.view_mode = mode;
    }

    /// Exports document with specified options.
    pub fn export_document(&self, options: &ExportOptions) -> Result<ExportReport, crate::export::ExportError> {
        DocumentExporter::export(&self.document, &self.markup, options)
    }

    /// Computes effective viewport area available for document pages taking sidebar into account.
    #[must_use]
    pub fn effective_viewport_size(&self) -> Vec2 {
        let titlebar_height = 52.0;
        let markup_bar_height = if self.is_markup_bar_visible { 44.0 } else { 0.0 };
        let sidebar_w = if self.is_sidebar_open { self.sidebar_width } else { 0.0 };

        Vec2::new(
            (self.window_size.x - sidebar_w).max(200.0),
            (self.window_size.y - titlebar_height - markup_bar_height).max(200.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{DocumentFormat, DocumentPage};

    #[test]
    fn test_window_controller_title_and_dirty_state() {
        let pages = vec![DocumentPage::new(0, Vec2::new(600.0, 800.0))];
        let doc = DocumentModel::new("invoice.pdf", "invoice.pdf", DocumentFormat::Pdf, pages, Vec2::new(1000.0, 800.0));
        let mut controller = PreviewWindowController::new(doc, Vec2::new(1000.0, 800.0));

        assert_eq!(controller.window_title(), "invoice.pdf");
        assert!(!controller.document.is_dirty());

        // Rotate page to make it dirty
        controller.document.rotate_active_page_clockwise();
        assert!(controller.document.is_dirty());
        assert_eq!(controller.window_title(), "invoice.pdf — Edited");
    }

    #[test]
    fn test_sidebar_and_markup_bar_toggling() {
        let pages = vec![DocumentPage::new(0, Vec2::new(600.0, 800.0))];
        let doc = DocumentModel::new("test.pdf", "test.pdf", DocumentFormat::Pdf, pages, Vec2::new(1000.0, 800.0));
        let mut controller = PreviewWindowController::new(doc, Vec2::new(1000.0, 800.0));

        assert!(controller.is_sidebar_open);
        controller.toggle_sidebar();
        assert!(!controller.is_sidebar_open);

        controller.set_sidebar_tab(SidebarTab::Annotations);
        assert!(controller.is_sidebar_open);
        assert_eq!(controller.sidebar_tab, SidebarTab::Annotations);

        assert!(!controller.is_markup_bar_visible);
        controller.toggle_markup_bar();
        assert!(controller.is_markup_bar_visible);
    }

    #[test]
    fn test_proxy_icon_drag_lifecycle() {
        let pages = vec![DocumentPage::new(0, Vec2::new(600.0, 800.0))];
        let doc = DocumentModel::new("/Users/nepo/Documents/doc.pdf", "doc.pdf", DocumentFormat::Pdf, pages, Vec2::new(1000.0, 800.0));
        let mut controller = PreviewWindowController::new(doc, Vec2::new(1000.0, 800.0));

        assert_eq!(controller.proxy_icon_drag, ProxyIconDragState::Idle);

        controller.start_proxy_icon_drag(Vec2::new(150.0, 26.0));
        match &controller.proxy_icon_drag {
            ProxyIconDragState::Dragging { file_path, cursor_pos } => {
                assert_eq!(file_path, "/Users/nepo/Documents/doc.pdf");
                assert_eq!(*cursor_pos, Vec2::new(150.0, 26.0));
            }
            _ => panic!("Expected Dragging state"),
        }

        controller.end_proxy_icon_drag();
        assert_eq!(controller.proxy_icon_drag, ProxyIconDragState::Idle);
    }
}
