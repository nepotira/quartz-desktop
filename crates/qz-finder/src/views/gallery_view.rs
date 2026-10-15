//! # Finder Gallery View Subsystem (`Cmd+4` / Cover Flow Evolution)
//!
//! Large visual central item carousel, horizontal thumbnail track, and collapsible
//! Inspector metadata sidebar with EXIF inspection and quick actions.

use qz_animation::{SpringAnimation, SpringParams};
use std::path::PathBuf;

/// An item in the Gallery View carousel.
#[derive(Debug, Clone, PartialEq)]
pub struct GalleryItem {
    /// File system path.
    pub path: PathBuf,
    /// Display file name.
    pub title: String,
    /// Kind description (e.g. "HEIC Image", "PDF Document").
    pub kind_str: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Media dimensions if applicable (e.g. "4032 × 3024").
    pub dimensions: Option<String>,
    /// Thumbnail image reference.
    pub thumbnail_path: Option<String>,
}

/// Gallery View presentation controller.
pub struct GalleryViewManager {
    items: Vec<GalleryItem>,
    active_index: usize,
    transition_spring: SpringAnimation,
    carousel_scroll_pos: f32,
    is_inspector_visible: bool,
}

impl Default for GalleryViewManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GalleryViewManager {
    /// Thumbnail width in bottom filmstrip.
    pub const THUMBNAIL_WIDTH: f32 = 64.0;
    /// Gap between thumbnails.
    pub const THUMBNAIL_GAP: f32 = 12.0;

    /// Creates a new Gallery View manager.
    pub fn new() -> Self {
        let spring_params = SpringParams::window_open();
        let transition_spring = SpringAnimation::new(spring_params, 0.0, 0.0, 0.0);

        Self {
            items: Vec::new(),
            active_index: 0,
            transition_spring,
            carousel_scroll_pos: 0.0,
            is_inspector_visible: true,
        }
    }

    /// Sets gallery items and resets carousel to first item.
    pub fn set_items(&mut self, items: Vec<GalleryItem>) {
        self.items = items;
        self.active_index = 0;
        self.carousel_scroll_pos = 0.0;
        self.transition_spring.retarget(0.0);
    }

    /// Selects an item by index, smoothly animating the carousel.
    pub fn select_index(&mut self, index: usize) {
        if self.items.is_empty() {
            return;
        }
        let clamped = index.min(self.items.len() - 1);
        if self.active_index != clamped {
            self.active_index = clamped;
            let target_scroll = clamped as f64 * (Self::THUMBNAIL_WIDTH + Self::THUMBNAIL_GAP) as f64;
            self.transition_spring.retarget(target_scroll);
        }
    }

    /// Navigates to previous item in carousel.
    pub fn select_previous(&mut self) {
        if self.active_index > 0 {
            self.select_index(self.active_index - 1);
        }
    }

    /// Navigates to next item in carousel.
    pub fn select_next(&mut self) {
        if self.active_index + 1 < self.items.len() {
            self.select_index(self.active_index + 1);
        }
    }

    /// Toggles right-side Inspector visibility.
    pub fn toggle_inspector(&mut self) -> bool {
        self.is_inspector_visible = !self.is_inspector_visible;
        self.is_inspector_visible
    }

    /// Advances spring animation clock.
    pub fn update(&mut self, dt: f32) {
        let state = self.transition_spring.step(dt as f64);
        self.carousel_scroll_pos = state.position as f32;
    }

    /// Currently active item displayed prominently in center.
    pub fn active_item(&self) -> Option<&GalleryItem> {
        self.items.get(self.active_index)
    }

    /// Active item index.
    #[inline]
    pub fn active_index(&self) -> usize {
        self.active_index
    }

    /// All items in gallery.
    pub fn items(&self) -> &[GalleryItem] {
        &self.items
    }

    /// Whether inspector sidebar is visible.
    #[inline]
    pub fn is_inspector_visible(&self) -> bool {
        self.is_inspector_visible
    }

    /// Current animated carousel horizontal scroll offset.
    #[inline]
    pub fn carousel_scroll_pos(&self) -> f32 {
        self.carousel_scroll_pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gallery_navigation_and_spring() {
        let mut gallery = GalleryViewManager::new();

        let items = vec![
            GalleryItem {
                path: PathBuf::from("/Users/demo/photo1.jpg"),
                title: "photo1.jpg".into(),
                kind_str: "JPEG image".into(),
                size_bytes: 1024 * 1024,
                dimensions: Some("3840 × 2160".into()),
                thumbnail_path: None,
            },
            GalleryItem {
                path: PathBuf::from("/Users/demo/photo2.jpg"),
                title: "photo2.jpg".into(),
                kind_str: "JPEG image".into(),
                size_bytes: 2 * 1024 * 1024,
                dimensions: Some("4032 × 3024".into()),
                thumbnail_path: None,
            },
        ];

        gallery.set_items(items);
        assert_eq!(gallery.active_index(), 0);

        gallery.select_next();
        assert_eq!(gallery.active_index(), 1);

        for _ in 0..60 {
            gallery.update(1.0 / 60.0);
        }

        assert!(gallery.carousel_scroll_pos() > 0.0);
    }
}
