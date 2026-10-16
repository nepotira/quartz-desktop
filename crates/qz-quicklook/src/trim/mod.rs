//! # Media Trimming Subsystem
//!
//! Non-destructive video and audio trimming in Quick Look with tactile in/out handles,
//! playhead scrubber synchronization, and export boundary calculation.

use thiserror::Error;

/// Trimming boundary errors.
#[derive(Debug, Error, PartialEq)]
pub enum TrimError {
    /// In-point is greater than or equal to Out-point.
    #[error("Trim start ({0:.2}s) must be strictly less than trim end ({1:.2}s)")]
    InvalidBounds(f64, f64),
    /// Boundary exceeds media total duration.
    #[error("Trim bound exceeds media duration: {0:.2}s > {1:.2}s")]
    OutOfBounds(f64, f64),
}

/// Calculated export trimming bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrimBounds {
    /// In-point timestamp in seconds.
    pub start_secs: f64,
    /// Out-point timestamp in seconds.
    pub end_secs: f64,
    /// Resulting trimmed duration in seconds.
    pub trimmed_duration_secs: f64,
}

/// Quick Look media trimming state controller.
pub struct MediaTrimController {
    is_trimming: bool,
    total_duration_secs: f64,
    trim_start_secs: f64,
    trim_end_secs: f64,
    playhead_secs: f64,
    is_playing: bool,
}

impl MediaTrimController {
    /// Canonical QuickTime trim handle yellow hex color (`#FFD60A`).
    pub const QUICKTIME_TRIM_YELLOW_HEX: &'static str = "#FFD60A";
    /// Canonical QuickTime trim handle normalized RGB (`[1.0, 0.839, 0.039]`).
    pub const QUICKTIME_TRIM_YELLOW_RGB: [f32; 3] = [1.0, 0.839, 0.039];

    /// Creates a trim controller for media of given duration.
    pub fn new(total_duration_secs: f64) -> Self {
        let duration = total_duration_secs.max(0.1);
        Self {
            is_trimming: false,
            total_duration_secs: duration,
            trim_start_secs: 0.0,
            trim_end_secs: duration,
            playhead_secs: 0.0,
            is_playing: false,
        }
    }

    /// Whether Trim mode is currently active.
    #[inline]
    pub fn is_trimming(&self) -> bool {
        self.is_trimming
    }

    /// Toggles trim editing mode.
    pub fn toggle_trim_mode(&mut self) -> bool {
        self.is_trimming = !self.is_trimming;
        self.is_trimming
    }

    /// Sets the In-point (trim start) in seconds.
    pub fn set_trim_start(&mut self, time_secs: f64) -> Result<(), TrimError> {
        let clamped = time_secs.clamp(0.0, self.total_duration_secs);
        if clamped >= self.trim_end_secs {
            return Err(TrimError::InvalidBounds(clamped, self.trim_end_secs));
        }
        self.trim_start_secs = clamped;
        if self.playhead_secs < self.trim_start_secs {
            self.playhead_secs = self.trim_start_secs;
        }
        Ok(())
    }

    /// Sets the Out-point (trim end) in seconds.
    pub fn set_trim_end(&mut self, time_secs: f64) -> Result<(), TrimError> {
        let clamped = time_secs.clamp(0.0, self.total_duration_secs);
        if clamped <= self.trim_start_secs {
            return Err(TrimError::InvalidBounds(self.trim_start_secs, clamped));
        }
        self.trim_end_secs = clamped;
        if self.playhead_secs > self.trim_end_secs {
            self.playhead_secs = self.trim_end_secs;
        }
        Ok(())
    }

    /// Scrubs playhead to the specified time, clamping within active trim bounds.
    pub fn scrub_to(&mut self, time_secs: f64) {
        self.playhead_secs = time_secs.clamp(self.trim_start_secs, self.trim_end_secs);
    }

    /// Toggles playback state.
    pub fn toggle_playback(&mut self) -> bool {
        self.is_playing = !self.is_playing;
        self.is_playing
    }

    /// Advances playback position by `dt` seconds, looping within trim bounds.
    pub fn update(&mut self, dt: f64) {
        if self.is_playing {
            self.playhead_secs += dt;
            if self.playhead_secs >= self.trim_end_secs {
                self.playhead_secs = self.trim_start_secs;
            }
        }
    }

    /// Computes final trim bounds for export.
    pub fn export_bounds(&self) -> TrimBounds {
        TrimBounds {
            start_secs: self.trim_start_secs,
            end_secs: self.trim_end_secs,
            trimmed_duration_secs: self.trim_end_secs - self.trim_start_secs,
        }
    }

    /// Current in-point (start) in seconds.
    #[inline]
    pub fn trim_start_secs(&self) -> f64 {
        self.trim_start_secs
    }

    /// Current out-point (end) in seconds.
    #[inline]
    pub fn trim_end_secs(&self) -> f64 {
        self.trim_end_secs
    }

    /// Current playhead position in seconds.
    #[inline]
    pub fn playhead_secs(&self) -> f64 {
        self.playhead_secs
    }

    /// Total media duration in seconds.
    #[inline]
    pub fn total_duration_secs(&self) -> f64 {
        self.total_duration_secs
    }

    /// Whether media is playing.
    #[inline]
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_trim_bounds_and_scrub() {
        let mut trim = MediaTrimController::new(120.0);
        assert_eq!(trim.total_duration_secs(), 120.0);
        assert_eq!(trim.trim_start_secs(), 0.0);
        assert_eq!(trim.trim_end_secs(), 120.0);

        trim.set_trim_start(10.0).unwrap();
        trim.set_trim_end(90.0).unwrap();

        assert_eq!(trim.export_bounds().trimmed_duration_secs, 80.0);

        // Scrub before start -> clamped to start
        trim.scrub_to(5.0);
        assert_eq!(trim.playhead_secs(), 10.0);

        // Scrub after end -> clamped to end
        trim.scrub_to(100.0);
        assert_eq!(trim.playhead_secs(), 90.0);
    }
}
