//! # Interactive Screen Capture & Recording HUD Subsystem (`Cmd+Shift+5`)
//!
//! Provides the floating Liquid Glass HUD toolbar:
//! - 5 Mode Segments: Capture Screen, Capture Window, Capture Portion, Record Screen, Record Portion.
//! - Options Popover: Save destinations, 5s/10s countdown timer, microphone selection, mouse click highlights.

use std::path::PathBuf;

/// The selected mode segment on the HUD toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudOperation {
    /// Take screenshot of the entire display.
    CaptureEntireScreen,
    /// Take screenshot of a specific window.
    CaptureSelectedWindow,
    /// Take screenshot of the selected rectangular portion.
    CaptureSelectedPortion,
    /// Record video of the entire screen.
    RecordEntireScreen,
    /// Record video of the selected rectangular portion.
    RecordSelectedPortion,
}

impl HudOperation {
    /// True if operation records video instead of capturing static screenshot.
    #[inline]
    pub fn is_recording(&self) -> bool {
        matches!(self, Self::RecordEntireScreen | Self::RecordSelectedPortion)
    }

    /// Label for the primary action button ("Capture" vs "Record").
    pub fn action_button_label(&self) -> &'static str {
        if self.is_recording() {
            "Record"
        } else {
            "Capture"
        }
    }
}

/// Destination location where captures are delivered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveDestination {
    /// User's Desktop (`~/Desktop`).
    Desktop,
    /// User's Documents (`~/Documents`).
    Documents,
    /// Direct copy to system clipboard without writing to disk.
    Clipboard,
    /// Compose new email with capture attachment.
    Mail,
    /// Compose new message with capture attachment.
    Messages,
    /// Open immediately in preview / playback player.
    QuickTimePlayer,
    /// Custom filesystem path.
    Custom(PathBuf),
}

/// Countdown timer delay prior to capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerDelay {
    /// Instant capture on trigger.
    None,
    /// 5 second audible countdown.
    FiveSeconds,
    /// 10 second audible countdown.
    TenSeconds,
}

impl TimerDelay {
    /// Duration in seconds.
    pub fn duration_secs(&self) -> u32 {
        match self {
            Self::None => 0,
            Self::FiveSeconds => 5,
            Self::TenSeconds => 10,
        }
    }
}

/// Options configured inside the HUD popover.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HudOptions {
    /// Where captures are saved.
    pub save_to: SaveDestination,
    /// Delay timer before capture.
    pub timer: TimerDelay,
    /// Whether microphone audio is recorded.
    pub record_microphone: bool,
    /// Highlight mouse clicks with circular ripple in video recording.
    pub show_mouse_clicks: bool,
    /// Show floating thumbnail in bottom right corner.
    pub show_floating_thumbnail: bool,
    /// Remember rectangular area across sessions.
    pub remember_last_selection: bool,
}

impl Default for HudOptions {
    fn default() -> Self {
        Self {
            save_to: SaveDestination::Desktop,
            timer: TimerDelay::None,
            record_microphone: false,
            show_mouse_clicks: true,
            show_floating_thumbnail: true,
            remember_last_selection: true,
        }
    }
}

/// Interactive HUD state controller.
pub struct ScreenshotHudController {
    is_visible: bool,
    selected_operation: HudOperation,
    options: HudOptions,
    is_options_popover_open: bool,
    countdown_remaining_secs: Option<u32>,
    selected_area_rect: Option<[f32; 4]>,
}

impl Default for ScreenshotHudController {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenshotHudController {
    /// Creates a new HUD controller.
    pub fn new() -> Self {
        Self {
            is_visible: false,
            selected_operation: HudOperation::CaptureEntireScreen,
            options: HudOptions::default(),
            is_options_popover_open: false,
            countdown_remaining_secs: None,
            selected_area_rect: None,
        }
    }

    /// Whether HUD is currently displayed on screen.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Shows the HUD toolbar (`Cmd+Shift+5`).
    pub fn show(&mut self) {
        self.is_visible = true;
        self.countdown_remaining_secs = None;
    }

    /// Hides the HUD toolbar (`Esc`).
    pub fn dismiss(&mut self) {
        self.is_visible = false;
        self.is_options_popover_open = false;
        self.countdown_remaining_secs = None;
    }

    /// Selects an operation segment.
    pub fn set_operation(&mut self, op: HudOperation) {
        self.selected_operation = op;
    }

    /// Active operation segment.
    #[inline]
    pub fn selected_operation(&self) -> HudOperation {
        self.selected_operation
    }

    /// Access reference to HUD options.
    pub fn options(&self) -> &HudOptions {
        &self.options
    }

    /// Access mutable reference to HUD options.
    pub fn options_mut(&mut self) -> &mut HudOptions {
        &mut self.options
    }

    /// Toggles the Options popover menu.
    pub fn toggle_options_popover(&mut self) -> bool {
        self.is_options_popover_open = !self.is_options_popover_open;
        self.is_options_popover_open
    }

    /// Triggers the capture or recording, starting timer countdown if configured.
    pub fn trigger_action(&mut self) -> bool {
        let delay = self.options.timer.duration_secs();
        if delay > 0 {
            self.countdown_remaining_secs = Some(delay);
            false // Will execute when countdown reaches 0
        } else {
            self.dismiss();
            true // Execute immediately
        }
    }

    /// Advances countdown clock by 1 second. Returns true when countdown completes.
    pub fn tick_countdown(&mut self) -> bool {
        if let Some(ref mut remaining) = self.countdown_remaining_secs {
            if *remaining > 1 {
                *remaining -= 1;
                false
            } else {
                self.countdown_remaining_secs = None;
                self.dismiss();
                true
            }
        } else {
            false
        }
    }

    /// Remaining countdown seconds if timer is active.
    #[inline]
    pub fn countdown_remaining(&self) -> Option<u32> {
        self.countdown_remaining_secs
    }

    /// Stores the last rectangular selection region.
    pub fn set_selected_area(&mut self, rect: [f32; 4]) {
        self.selected_area_rect = Some(rect);
    }

    /// Last rectangular selection region.
    #[inline]
    pub fn selected_area(&self) -> Option<[f32; 4]> {
        self.selected_area_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_operation_labels_and_timer_countdown() {
        let mut hud = ScreenshotHudController::new();
        hud.show();
        assert!(hud.is_visible());

        assert_eq!(hud.selected_operation().action_button_label(), "Capture");

        // Switch to recording mode
        hud.set_operation(HudOperation::RecordEntireScreen);
        assert_eq!(hud.selected_operation().action_button_label(), "Record");
        assert!(hud.selected_operation().is_recording());

        // Set 5s timer
        hud.options_mut().timer = TimerDelay::FiveSeconds;

        // Trigger action initiates countdown
        assert!(!hud.trigger_action());
        assert_eq!(hud.countdown_remaining(), Some(5));

        // Advance 4 seconds
        assert!(!hud.tick_countdown()); // 4
        assert!(!hud.tick_countdown()); // 3
        assert!(!hud.tick_countdown()); // 2
        assert!(!hud.tick_countdown()); // 1

        // 5th second finishes countdown
        assert!(hud.tick_countdown());
        assert_eq!(hud.countdown_remaining(), None);
        assert!(!hud.is_visible());
    }
}
