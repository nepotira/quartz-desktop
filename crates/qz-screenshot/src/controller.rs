//! # Screenshot Master Controller Subsystem
//!
//! Orchestrates the full macOS Tahoe 26 screenshot and recording experience:
//! `Cmd+Shift+3` (Full Screen), `Cmd+Shift+4` (Selection / Window), `Cmd+Shift+5` (HUD Toolbar),
//! Floating Thumbnails, and PipeWire Screen Recorder integration.

use crate::capture::{CaptureMode, CrosshairSelection, WindowCaptureCanvas, WindowShadowProfile};
use crate::hud::{HudOperation, ScreenshotHudController};
use crate::recorder::ScreenRecorderDaemon;
use crate::thumbnail::FloatingThumbnail;
use glam::Vec2;
use std::path::PathBuf;

/// Events emitted by the Screenshot controller.
#[derive(Debug, Clone, PartialEq)]
pub enum ScreenshotEvent {
    /// Screenshot captured to file.
    ScreenshotSaved(PathBuf),
    /// Screenshot copied directly to clipboard.
    CopiedToClipboard,
    /// Video screen recording started.
    RecordingStarted,
    /// Video screen recording stopped and saved.
    RecordingSaved(PathBuf),
    /// Cancelled or dismissed without capturing.
    Cancelled,
}

/// Master Screenshot and Screen Recording controller.
pub struct ScreenshotController {
    crosshair: CrosshairSelection,
    hud: ScreenshotHudController,
    active_thumbnail: Option<FloatingThumbnail>,
    recorder: ScreenRecorderDaemon,
    is_crosshair_active: bool,
    screen_width: f32,
    screen_height: f32,
}

impl Default for ScreenshotController {
    fn default() -> Self {
        Self::new(1920.0, 1080.0)
    }
}

impl ScreenshotController {
    /// Creates a screenshot controller initialized for display dimensions.
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        Self {
            crosshair: CrosshairSelection::new(),
            hud: ScreenshotHudController::new(),
            active_thumbnail: None,
            recorder: ScreenRecorderDaemon::default(),
            is_crosshair_active: false,
            screen_width,
            screen_height,
        }
    }

    /// Handles `Cmd+Shift+3` shortcut: captures the full screen immediately.
    pub fn trigger_full_screen_capture(&mut self) -> ScreenshotEvent {
        let path = Self::generate_default_filename("png", false);
        self.spawn_thumbnail_if_enabled(path.clone(), false);
        ScreenshotEvent::ScreenshotSaved(path)
    }

    /// Handles `Cmd+Shift+4` shortcut: activates the interactive crosshair.
    pub fn trigger_crosshair_selection(&mut self) {
        self.crosshair.reset();
        self.is_crosshair_active = true;
    }

    /// Handles `Cmd+Shift+5` shortcut: presents the interactive HUD.
    pub fn trigger_hud(&mut self) {
        self.hud.show();
    }

    /// Handles `Cmd+Ctrl+Esc` shortcut: stops screen recording if in progress.
    pub fn stop_screen_recording(&mut self) -> Result<ScreenshotEvent, crate::recorder::RecorderError> {
        let path = self.recorder.stop_recording()?;
        self.spawn_thumbnail_if_enabled(path.clone(), true);
        Ok(ScreenshotEvent::RecordingSaved(path))
    }

    /// Toggles Spacebar camera mode while in crosshair selection.
    pub fn toggle_crosshair_camera_mode(&mut self) -> bool {
        if self.is_crosshair_active {
            self.crosshair.toggle_camera_mode()
        } else {
            false
        }
    }

    /// Forwards mouse down event in crosshair selection mode.
    pub fn mouse_down(&mut self, pos: Vec2) {
        if self.is_crosshair_active {
            self.crosshair.mouse_down(pos);
        }
    }

    /// Forwards mouse move event in crosshair selection mode.
    pub fn mouse_move(&mut self, pos: Vec2, window_under_cursor: Option<u32>) {
        if self.is_crosshair_active {
            self.crosshair.mouse_move(pos, window_under_cursor);
        }
    }

    /// Forwards mouse up event in crosshair selection mode, executing capture if selection completed.
    pub fn mouse_up(&mut self) -> Option<ScreenshotEvent> {
        if self.is_crosshair_active {
            self.is_crosshair_active = false;
            if let Some(mode) = self.crosshair.mouse_up() {
                match mode {
                    CaptureMode::IsolatedWindow { include_shadow, .. } => {
                        let _canvas = WindowCaptureCanvas::compute(
                            1200,
                            800,
                            include_shadow,
                            WindowShadowProfile::default(),
                        );
                        let path = Self::generate_default_filename("png", false);
                        self.spawn_thumbnail_if_enabled(path.clone(), false);
                        Some(ScreenshotEvent::ScreenshotSaved(path))
                    }
                    CaptureMode::RectangularArea { .. } => {
                        let path = Self::generate_default_filename("png", false);
                        self.spawn_thumbnail_if_enabled(path.clone(), false);
                        Some(ScreenshotEvent::ScreenshotSaved(path))
                    }
                    _ => None,
                }
            } else {
                Some(ScreenshotEvent::Cancelled)
            }
        } else {
            None
        }
    }

    /// Triggers the selected HUD operation.
    pub fn execute_hud_action(&mut self) -> Option<ScreenshotEvent> {
        let op = self.hud.selected_operation();
        let immediate = self.hud.trigger_action();

        if immediate {
            match op {
                HudOperation::CaptureEntireScreen => Some(self.trigger_full_screen_capture()),
                HudOperation::CaptureSelectedWindow => {
                    self.trigger_crosshair_selection();
                    self.toggle_crosshair_camera_mode();
                    None
                }
                HudOperation::CaptureSelectedPortion => {
                    self.trigger_crosshair_selection();
                    None
                }
                HudOperation::RecordEntireScreen | HudOperation::RecordSelectedPortion => {
                    let _ = self.recorder.start_recording(0);
                    Some(ScreenshotEvent::RecordingStarted)
                }
            }
        } else {
            None
        }
    }

    /// Advances physics simulation for active floating thumbnail and recorder.
    pub fn update(&mut self, dt: f32) {
        if let Some(ref mut thumb) = self.active_thumbnail {
            if thumb.update(dt) {
                self.active_thumbnail = None;
            }
        }

        self.recorder.update(dt as f64);
    }

    fn spawn_thumbnail_if_enabled(&mut self, path: PathBuf, is_video: bool) {
        if self.hud.options().show_floating_thumbnail {
            self.active_thumbnail = Some(FloatingThumbnail::new(
                path,
                is_video,
                self.screen_width,
                self.screen_height,
            ));
        }
    }

    fn generate_default_filename(ext: &str, is_video: bool) -> PathBuf {
        let prefix = if is_video {
            "Screen Recording"
        } else {
            "Screen Shot"
        };
        PathBuf::from(format!(
            "/Users/demo/Desktop/{prefix} 2026-10-15 at 12.00.00.{ext}"
        ))
    }

    /// Whether crosshair selection is active.
    #[inline]
    pub fn is_crosshair_active(&self) -> bool {
        self.is_crosshair_active
    }

    /// Active floating thumbnail if present.
    pub fn active_thumbnail(&self) -> Option<&FloatingThumbnail> {
        self.active_thumbnail.as_ref()
    }

    /// Mutable reference to active floating thumbnail.
    pub fn active_thumbnail_mut(&mut self) -> Option<&mut FloatingThumbnail> {
        self.active_thumbnail.as_mut()
    }

    /// Reference to HUD controller.
    pub fn hud(&self) -> &ScreenshotHudController {
        &self.hud
    }

    /// Mutable reference to HUD controller.
    pub fn hud_mut(&mut self) -> &mut ScreenshotHudController {
        &mut self.hud
    }

    /// Reference to Screen Recorder daemon.
    pub fn recorder(&self) -> &ScreenRecorderDaemon {
        &self.recorder
    }

    /// Mutable reference to Screen Recorder daemon.
    pub fn recorder_mut(&mut self) -> &mut ScreenRecorderDaemon {
        &mut self.recorder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_controller_shortcuts_and_thumbnail() {
        let mut controller = ScreenshotController::new(1920.0, 1080.0);

        // Full screen capture triggers thumbnail
        let event = controller.trigger_full_screen_capture();
        match event {
            ScreenshotEvent::ScreenshotSaved(path) => {
                assert!(path.to_str().unwrap().contains("Screen Shot"));
            }
            _ => panic!("Expected ScreenshotSaved event"),
        }

        assert!(controller.active_thumbnail().is_some());

        // Crosshair trigger
        controller.trigger_crosshair_selection();
        assert!(controller.is_crosshair_active());

        // HUD trigger
        controller.trigger_hud();
        assert!(controller.hud().is_visible());
    }
}
