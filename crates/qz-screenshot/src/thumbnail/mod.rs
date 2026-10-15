//! # Floating Thumbnail Subsystem
//!
//! Provides the Tahoe 26 post-capture floating thumbnail sliding into the bottom-right
//! corner with spring physics, 5-second auto-commit timer, swipe-to-dismiss, and drag-and-drop.

use qz_animation::{SpringAnimation, SpringParams};
use std::path::PathBuf;

/// State of the floating thumbnail in its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailLifecycle {
    /// Sliding in onto screen from bottom-right.
    Entering,
    /// Resting in place awaiting user interaction or auto-commit timeout.
    Resting,
    /// User is currently dragging the thumbnail to another application or window.
    Dragging,
    /// Sliding off-screen to the right on commit or swipe.
    Exiting,
    /// Dismissed and removed from screen.
    Dismissed,
}

/// Floating thumbnail interactive widget controller.
pub struct FloatingThumbnail {
    file_path: PathBuf,
    is_video_recording: bool,
    lifecycle: ThumbnailLifecycle,
    resting_pos: [f32; 2],
    current_pos: [f32; 2],
    spring_x: SpringAnimation,
    timeout_remaining_secs: f32,
    is_hovered: bool,
    width: f32,
    height: f32,
}

impl FloatingThumbnail {
    /// Thumbnail standard pixel width.
    pub const THUMBNAIL_WIDTH: f32 = 180.0;
    /// Thumbnail standard pixel height.
    pub const THUMBNAIL_HEIGHT: f32 = 112.0;
    /// Default time on screen before auto-committing to disk.
    pub const AUTO_COMMIT_TIMEOUT_SECS: f32 = 5.0;

    /// Creates a new floating thumbnail sliding in for the given captured file.
    pub fn new(file_path: PathBuf, is_video: bool, screen_width: f32, screen_height: f32) -> Self {
        let resting_x = screen_width - Self::THUMBNAIL_WIDTH - 24.0;
        let resting_y = screen_height - Self::THUMBNAIL_HEIGHT - 32.0;

        // Start offscreen to the right
        let start_x = screen_width + 40.0;

        let spring_params = SpringParams::dock_bounce();
        let mut spring_x = SpringAnimation::new(spring_params, start_x as f64, 0.0, resting_x as f64);
        spring_x.retarget(resting_x as f64);

        Self {
            file_path,
            is_video_recording: is_video,
            lifecycle: ThumbnailLifecycle::Entering,
            resting_pos: [resting_x, resting_y],
            current_pos: [start_x, resting_y],
            spring_x,
            timeout_remaining_secs: Self::AUTO_COMMIT_TIMEOUT_SECS,
            is_hovered: false,
            width: Self::THUMBNAIL_WIDTH,
            height: Self::THUMBNAIL_HEIGHT,
        }
    }

    /// Sets mouse hover state (pauses auto-commit countdown).
    pub fn set_hovered(&mut self, hovered: bool) {
        self.is_hovered = hovered;
    }

    /// Swipes thumbnail to the right, accelerating off-screen dismissal.
    pub fn swipe_dismiss_right(&mut self) {
        self.lifecycle = ThumbnailLifecycle::Exiting;
        let exit_target = self.resting_pos[0] + Self::THUMBNAIL_WIDTH + 60.0;
        self.spring_x.retarget(exit_target as f64);
    }

    /// Advances physics and timeout clock. Returns true when thumbnail has fully exited and should be unrendered.
    pub fn update(&mut self, dt: f32) -> bool {
        let state = self.spring_x.step(dt as f64);
        self.current_pos[0] = state.position as f32;

        match self.lifecycle {
            ThumbnailLifecycle::Entering => {
                if (self.current_pos[0] - self.resting_pos[0]).abs() < 1.0 && state.velocity.abs() < 1.0 {
                    self.lifecycle = ThumbnailLifecycle::Resting;
                    self.current_pos[0] = self.resting_pos[0];
                }
            }
            ThumbnailLifecycle::Resting => {
                if !self.is_hovered {
                    self.timeout_remaining_secs -= dt;
                    if self.timeout_remaining_secs <= 0.0 {
                        self.swipe_dismiss_right();
                    }
                }
            }
            ThumbnailLifecycle::Exiting => {
                if self.current_pos[0] >= self.resting_pos[0] + Self::THUMBNAIL_WIDTH + 40.0 {
                    self.lifecycle = ThumbnailLifecycle::Dismissed;
                    return true;
                }
            }
            ThumbnailLifecycle::Dismissed => return true,
            _ => {}
        }

        false
    }

    /// Current screen coordinates $[x, y]$.
    #[inline]
    pub fn current_pos(&self) -> [f32; 2] {
        self.current_pos
    }

    /// Active lifecycle state.
    #[inline]
    pub fn lifecycle(&self) -> ThumbnailLifecycle {
        self.lifecycle
    }

    /// Associated captured file path.
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Whether capture was a video recording.
    #[inline]
    pub fn is_video_recording(&self) -> bool {
        self.is_video_recording
    }

    /// Seconds remaining before automatic commit.
    #[inline]
    pub fn timeout_remaining(&self) -> f32 {
        self.timeout_remaining_secs
    }

    /// Dimensions $[width, height]$.
    #[inline]
    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_floating_thumbnail_entry_and_swipe_dismissal() {
        let mut thumb = FloatingThumbnail::new(
            PathBuf::from("/Users/demo/Desktop/Screen Shot 2026-10-15.png"),
            false,
            1920.0,
            1080.0,
        );

        assert_eq!(thumb.lifecycle(), ThumbnailLifecycle::Entering);
        assert!(thumb.current_pos()[0] > 1920.0);

        // Advance frames until entered
        for _ in 0..120 {
            thumb.update(1.0 / 60.0);
        }

        assert_eq!(thumb.lifecycle(), ThumbnailLifecycle::Resting);

        // Swipe right
        thumb.swipe_dismiss_right();
        assert_eq!(thumb.lifecycle(), ThumbnailLifecycle::Exiting);

        // Advance frames until exited
        let mut finished = false;
        for _ in 0..120 {
            if thumb.update(1.0 / 60.0) {
                finished = true;
                break;
            }
        }

        assert!(finished);
        assert_eq!(thumb.lifecycle(), ThumbnailLifecycle::Dismissed);
    }
}
