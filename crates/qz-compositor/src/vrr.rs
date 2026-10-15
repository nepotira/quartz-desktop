//! # ProMotion Dynamic VRR Frame Pacing Engine (10Hz–120Hz)
//!
//! Controls adaptive variable refresh rate (VRR) and frame scheduling matching macOS ProMotion:
//! - **120Hz (8.33ms budget)**: Engaged during active spring physics animations, multi-touch gestures,
//!   or pointer activity within the last 100ms.
//! - **60Hz (16.66ms budget)**: Video playback or discrete UI updates.
//! - **10Hz–24Hz (41.67ms–100.0ms budget)**: Static screen preserving GPU power.
//! - Interacts with Linux DRM connectors via `VRR_ENABLED` property when `vrr_capable == 1`.

use std::time::Duration;

/// ProMotion target refresh rate operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePaceMode {
    /// Ultra-low power static desktop state (10Hz, 100ms budget).
    Idle10Hz,
    /// Low-power static desktop with light background activity (24Hz, 41.67ms budget).
    LowPower24Hz,
    /// Video playback and discrete user interface redraws (60Hz, 16.67ms budget).
    Standard60Hz,
    /// High-responsiveness fluid interaction and spring physics (120Hz, 8.33ms budget).
    ProMotion120Hz,
}

impl FramePaceMode {
    /// Frame presentation budget in milliseconds.
    pub fn budget_millis(&self) -> f64 {
        match self {
            Self::Idle10Hz => 100.0,
            Self::LowPower24Hz => 1000.0 / 24.0, // ~41.6667ms
            Self::Standard60Hz => 1000.0 / 60.0, // ~16.6667ms
            Self::ProMotion120Hz => 1000.0 / 120.0, // ~8.3333ms
        }
    }

    /// Frame budget converted to standard `Duration`.
    pub fn frame_duration(&self) -> Duration {
        match self {
            Self::Idle10Hz => Duration::from_millis(100),
            Self::LowPower24Hz => Duration::from_nanos(41_666_667),
            Self::Standard60Hz => Duration::from_nanos(16_666_667),
            Self::ProMotion120Hz => Duration::from_nanos(8_333_333),
        }
    }

    /// Nominal target frequency in Hertz.
    pub fn target_hertz(&self) -> u32 {
        match self {
            Self::Idle10Hz => 10,
            Self::LowPower24Hz => 24,
            Self::Standard60Hz => 60,
            Self::ProMotion120Hz => 120,
        }
    }
}

/// Dynamic ProMotion VRR Scheduler tracking scene graph activity.
pub struct ProMotionScheduler {
    active_springs_count: usize,
    active_gestures_count: usize,
    last_pointer_motion_time: f64,
    video_playback_active: bool,
    discrete_ui_dirty: bool,
    vrr_capable: bool,
    vrr_enabled: bool,
}

impl Default for ProMotionScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProMotionScheduler {
    /// Recency window during which pointer motion holds the compositor at 120Hz (100ms).
    pub const POINTER_ACTIVE_WINDOW: f64 = 0.100;

    /// Creates a new ProMotion scheduler.
    pub fn new() -> Self {
        Self {
            active_springs_count: 0,
            active_gestures_count: 0,
            last_pointer_motion_time: -100.0,
            video_playback_active: false,
            discrete_ui_dirty: false,
            vrr_capable: false,
            vrr_enabled: false,
        }
    }

    /// Configures DRM hardware capability for Variable Refresh Rate.
    pub fn set_drm_vrr_capable(&mut self, capable: bool) {
        self.vrr_capable = capable;
        if !capable {
            self.vrr_enabled = false;
        }
    }

    /// Whether the DRM connector `VRR_ENABLED` property should be set to 1.
    #[inline]
    pub fn is_vrr_enabled(&self) -> bool {
        self.vrr_enabled
    }

    /// Notifies scheduler of a pointer movement event at timestamp `now_secs`.
    pub fn notify_pointer_motion(&mut self, now_secs: f64) {
        self.last_pointer_motion_time = now_secs;
    }

    /// Registers a newly initiated multi-touch gesture.
    pub fn notify_gesture_started(&mut self) {
        self.active_gestures_count = self.active_gestures_count.saturating_add(1);
    }

    /// Registers a completed or cancelled gesture.
    pub fn notify_gesture_ended(&mut self) {
        self.active_gestures_count = self.active_gestures_count.saturating_sub(1);
    }

    /// Updates the number of active spring animations currently running in the scene graph.
    pub fn set_active_springs_count(&mut self, count: usize) {
        self.active_springs_count = count;
    }

    /// Flags whether video decoding/playback is active on any visible surface.
    pub fn set_video_playback_active(&mut self, active: bool) {
        self.video_playback_active = active;
    }

    /// Flags discrete UI invalidations (blinking cursor, clock tick, progress bar).
    pub fn set_discrete_ui_dirty(&mut self, dirty: bool) {
        self.discrete_ui_dirty = dirty;
    }

    /// Computes the optimal frame pacing mode and target sleep duration for the next `calloop` iteration.
    pub fn compute_frame_budget(&mut self, current_time_secs: f64) -> (FramePaceMode, Duration) {
        let pointer_recently_active =
            (current_time_secs - self.last_pointer_motion_time).abs() <= Self::POINTER_ACTIVE_WINDOW;

        let requires_120hz = self.active_springs_count > 0
            || self.active_gestures_count > 0
            || pointer_recently_active;

        let mode = if requires_120hz {
            // High-precision ProMotion 120Hz
            if self.vrr_capable {
                self.vrr_enabled = true;
            }
            FramePaceMode::ProMotion120Hz
        } else if self.video_playback_active || self.discrete_ui_dirty {
            // Standard 60Hz UI refresh
            if self.vrr_capable {
                self.vrr_enabled = true;
            }
            FramePaceMode::Standard60Hz
        } else {
            // Static desktop: low power 10Hz/24Hz
            if self.vrr_capable {
                self.vrr_enabled = true;
            }
            FramePaceMode::Idle10Hz
        };

        // Reset one-shot dirty flag
        self.discrete_ui_dirty = false;

        (mode, mode.frame_duration())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotion_scheduler_transitions() {
        let mut scheduler = ProMotionScheduler::new();
        scheduler.set_drm_vrr_capable(true);

        // Initially idle: should choose 10Hz
        let (mode0, dur0) = scheduler.compute_frame_budget(0.0);
        assert_eq!(mode0, FramePaceMode::Idle10Hz);
        assert_eq!(dur0, Duration::from_millis(100));

        // Pointer moves at t = 1.0: should escalate to 120Hz
        scheduler.notify_pointer_motion(1.0);
        let (mode1, dur1) = scheduler.compute_frame_budget(1.050); // 50ms later (< 100ms)
        assert_eq!(mode1, FramePaceMode::ProMotion120Hz);
        assert_eq!(dur1, Duration::from_nanos(8_333_333));
        assert!(scheduler.is_vrr_enabled());

        // At t = 1.200 (150ms after pointer motion, no springs): returns to idle
        let (mode2, _) = scheduler.compute_frame_budget(1.200);
        assert_eq!(mode2, FramePaceMode::Idle10Hz);

        // Active spring: should jump to 120Hz
        scheduler.set_active_springs_count(1);
        let (mode3, _) = scheduler.compute_frame_budget(1.205);
        assert_eq!(mode3, FramePaceMode::ProMotion120Hz);

        // Active video: should stay at 60Hz
        scheduler.set_active_springs_count(0);
        scheduler.set_video_playback_active(true);
        let (mode4, _) = scheduler.compute_frame_budget(1.300);
        assert_eq!(mode4, FramePaceMode::Standard60Hz);
    }
}
