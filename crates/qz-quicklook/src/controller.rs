//! # Quick Look Daemon Master Controller Subsystem
//!
//! Orchestrates the Quick Look presentation lifecycle, integrating 120fps rect expansion,
//! in-place retargeting on arrow key navigation, Markup, Trimming, and Subject Lift.

use crate::animation::{ExpansionPhase, RectBounds, RectExpansionAnimator};
use crate::markup::MarkupSession;
use crate::preview::{PreviewGenerator, PreviewPayload, QuickLookPreview};
use crate::trim::MediaTrimController;
use crate::visual_lookup::{LiftedSubject, SubjectLiftController};
use std::path::{Path, PathBuf};

/// Quick Look window action event.
#[derive(Debug, Clone, PartialEq)]
pub enum QuickLookEvent {
    /// Window was dismissed and closed.
    Dismissed,
    /// Requested to open the file with its default application (`Cmd+Enter`).
    OpenWithDefaultApp(PathBuf),
    /// Content was copied to clipboard (`Cmd+C`).
    CopiedToClipboard,
    /// Document was rotated 90 degrees (`Cmd+R`).
    Rotated(u32),
}

/// Master Quick Look daemon managing the preview window.
pub struct QuickLookController {
    is_open: bool,
    active_path: Option<PathBuf>,
    active_preview: Option<QuickLookPreview>,
    animator: RectExpansionAnimator,
    markup: MarkupSession,
    trim: Option<MediaTrimController>,
    subject_lift: SubjectLiftController,
}

impl Default for QuickLookController {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickLookController {
    /// Creates a new Quick Look daemon controller.
    pub fn new() -> Self {
        Self {
            is_open: false,
            active_path: None,
            active_preview: None,
            animator: RectExpansionAnimator::default(),
            markup: MarkupSession::new(),
            trim: None,
            subject_lift: SubjectLiftController::new(),
        }
    }

    /// Whether Quick Look preview window is currently active or opening.
    #[inline]
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Toggles Quick Look for the selected file and its screen thumbnail rect.
    pub fn toggle(&mut self, path: &Path, source_rect: RectBounds) {
        if self.is_open && self.active_path.as_deref() == Some(path) {
            self.dismiss();
        } else {
            self.present(path, source_rect);
        }
    }

    /// Presents Quick Look for the given file, expanding smoothly from `source_rect`.
    pub fn present(&mut self, path: &Path, source_rect: RectBounds) {
        let preview = PreviewGenerator::generate_preview(path);

        // Center preview on screen given its ideal dimensions
        let screen_w = 1920.0;
        let screen_h = 1080.0;
        let dest_w = preview.ideal_dimensions[0];
        let dest_h = preview.ideal_dimensions[1];
        let dest_x = (screen_w - dest_w) * 0.5;
        let dest_y = (screen_h - dest_h) * 0.5;
        let target_rect = RectBounds::new(dest_x, dest_y, dest_w, dest_h);

        self.animator = RectExpansionAnimator::new(source_rect, target_rect);
        self.animator.open();

        // Initialize Trim controller if media file
        if let PreviewPayload::Media { duration_secs, .. } = preview.payload {
            self.trim = Some(MediaTrimController::new(duration_secs));
        } else {
            self.trim = None;
        }

        // Initialize Subject Lift if image file
        if let PreviewPayload::Image { width, height, .. } = preview.payload {
            let mock_subject = LiftedSubject {
                normalized_bounds: [0.25, 0.20, 0.50, 0.60],
                contour_points: vec![
                    [0.25, 0.20],
                    [0.75, 0.20],
                    [0.75, 0.80],
                    [0.25, 0.80],
                ],
                lookup_card: None,
            };
            let _ = (width, height);
            self.subject_lift.set_recognized_subject(Some(mock_subject));
        } else {
            self.subject_lift.set_recognized_subject(None);
        }

        self.active_path = Some(path.to_path_buf());
        self.active_preview = Some(preview);
        self.is_open = true;
    }

    /// Retargets the active preview in-place when user browses with arrow keys in Finder.
    pub fn retarget_in_place(&mut self, path: &Path, new_source_rect: RectBounds) {
        if !self.is_open {
            self.present(path, new_source_rect);
            return;
        }

        let preview = PreviewGenerator::generate_preview(path);
        let screen_w = 1920.0;
        let screen_h = 1080.0;
        let dest_w = preview.ideal_dimensions[0];
        let dest_h = preview.ideal_dimensions[1];
        let dest_x = (screen_w - dest_w) * 0.5;
        let dest_y = (screen_h - dest_h) * 0.5;
        let target_rect = RectBounds::new(dest_x, dest_y, dest_w, dest_h);

        self.animator.retarget_source(new_source_rect);
        self.animator.retarget_destination(target_rect);

        self.active_path = Some(path.to_path_buf());
        self.active_preview = Some(preview);
    }

    /// Dismisses Quick Look window, collapsing back into source rect.
    pub fn dismiss(&mut self) {
        self.animator.close();
        self.is_open = false;
    }

    /// Advances physics and animation clocks.
    pub fn update(&mut self, dt: f32) {
        self.animator.update(dt);
        self.subject_lift.update(dt);
        if let Some(trim) = &mut self.trim {
            trim.update(dt as f64);
        }
    }

    /// Rotates the active document 90 degrees clockwise (`Cmd+R`).
    pub fn rotate_clockwise(&mut self) -> u32 {
        self.markup.rotate_clockwise()
    }

    /// Executes Copy (`Cmd+C`), copying lifted subject if active.
    pub fn execute_copy(&self) -> bool {
        if self.subject_lift.is_subject_lifted() {
            self.subject_lift.copy_subject_to_clipboard().is_some()
        } else {
            true
        }
    }

    /// Current animated bounding rect for window rendering.
    pub fn current_window_rect(&self) -> RectBounds {
        self.animator.current_rect()
    }

    /// Active preview model.
    pub fn active_preview(&self) -> Option<&QuickLookPreview> {
        self.active_preview.as_ref()
    }

    /// Active file path.
    pub fn active_path(&self) -> Option<&Path> {
        self.active_path.as_deref()
    }

    /// Reference to Markup session.
    pub fn markup(&self) -> &MarkupSession {
        &self.markup
    }

    /// Mutable reference to Markup session.
    pub fn markup_mut(&mut self) -> &mut MarkupSession {
        &mut self.markup
    }

    /// Reference to Media Trim controller if media is loaded.
    pub fn trim(&self) -> Option<&MediaTrimController> {
        self.trim.as_ref()
    }

    /// Mutable reference to Media Trim controller.
    pub fn trim_mut(&mut self) -> Option<&mut MediaTrimController> {
        self.trim.as_mut()
    }

    /// Reference to Subject Lift controller.
    pub fn subject_lift(&self) -> &SubjectLiftController {
        &self.subject_lift
    }

    /// Mutable reference to Subject Lift controller.
    pub fn subject_lift_mut(&mut self) -> &mut SubjectLiftController {
        &mut self.subject_lift
    }

    /// Animation phase.
    #[inline]
    pub fn phase(&self) -> ExpansionPhase {
        self.animator.phase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_look_daemon_presentation_and_retarget() {
        let mut ql = QuickLookController::new();
        assert!(!ql.is_open());

        let file1 = PathBuf::from("/Users/demo/photo.jpg");
        let source1 = RectBounds::new(100.0, 100.0, 64.0, 64.0);

        // Present file1
        ql.present(&file1, source1);
        assert!(ql.is_open());
        assert_eq!(ql.active_path(), Some(file1.as_path()));

        // Advance frames
        for _ in 0..60 {
            ql.update(1.0 / 120.0);
        }

        // Retarget in place to file2 (arrow key navigation)
        let file2 = PathBuf::from("/Users/demo/doc.pdf");
        let source2 = RectBounds::new(180.0, 100.0, 64.0, 64.0);
        ql.retarget_in_place(&file2, source2);

        assert_eq!(ql.active_path(), Some(file2.as_path()));
        assert_eq!(ql.animator.source_rect(), source2);

        // Dismiss
        ql.dismiss();
        assert!(!ql.is_open());
    }
}
