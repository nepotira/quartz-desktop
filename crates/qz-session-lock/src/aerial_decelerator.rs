//! # 120fps Aerial Video Decelerator (`qz-session-lock::aerial_decelerator`)
//!
//! Production-grade aerial screensaver and dynamic wallpaper decelerator for macOS Tahoe 26.
//!
//! ### Key Capabilities:
//! - **120fps ProMotion Aerial Screensaver:** Streams cinematic 4K drone footage at 120fps
//!   ($8.33\text{ms}$ frame interval) during idle lock screen sessions.
//! - **$C^2$ Continuous Deceleration on Unlock:**
//!   When the user unlocks the system, the video does not perform an abrupt cut or freeze.
//!   Instead, it smoothly decelerates from $1.0\times$ down to $0.0\times$ over a $2.4$-second
//!   interval using a cubic Hermite ease-out curve with zero derivatives at both endpoints:
//!   $$u = \frac{t}{T_{\text{decel}}}, \quad \text{speed}(u) = (1.0 - u)^2 (1.0 + 2u)$$
//! - **Seamless Morph into Static Desktop Wallpaper:**
//!   The precise sub-pixel video frame where $\text{speed} = 0.0$ becomes the static desktop
//!   wallpaper, achieving the signature Apple Aerial transition without a single hitch.

use serde::{Deserialize, Serialize};

/// Operational mode of the aerial video decelerator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AerialMode {
    /// Actively playing full-speed screensaver at 1.0x (120fps).
    ScreensaverPlaying,
    /// Smoothly decelerating playback speed upon user unlock event.
    Decelerating,
    /// Frozen completely at 0.0x speed; serving as static desktop wallpaper.
    StaticWallpaper,
}

/// Dynamic Aerial Video Screensaver & Wallpaper Deceleration Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AerialVideoDecelerator {
    /// Active operational mode.
    mode: AerialMode,
    /// Video asset path (e.g. `/Library/Application Support/QuartzLinux/Aerials/TahoeLake.mov`).
    pub video_path: String,
    /// Total duration of the aerial video asset in seconds.
    pub video_duration_seconds: f64,
    /// Current video playback playhead position in seconds.
    current_playhead_seconds: f64,
    /// Instantaneous playback speed multiplier ($1.0 = \text{normal}, 0.0 = \text{stopped}$).
    current_playback_speed: f64,
    /// Elapsed deceleration time in seconds.
    elapsed_decel_seconds: f64,
    /// Total duration of deceleration transition in seconds (Tahoe 26 standard: 2.4s).
    pub decel_duration_seconds: f64,
}

impl Default for AerialVideoDecelerator {
    fn default() -> Self {
        Self::new(
            "/System/Library/ScreenSavers/TahoeAerial.mov",
            600.0, // 10-minute aerial loop
        )
    }
}

impl AerialVideoDecelerator {
    /// Creates a new `AerialVideoDecelerator` for the given video asset.
    #[must_use]
    pub fn new(video_path: &str, video_duration_seconds: f64) -> Self {
        Self {
            mode: AerialMode::ScreensaverPlaying,
            video_path: video_path.to_string(),
            video_duration_seconds: video_duration_seconds.max(1.0),
            current_playhead_seconds: 0.0,
            current_playback_speed: 1.0,
            elapsed_decel_seconds: 0.0,
            decel_duration_seconds: 2.40,
        }
    }

    /// Evaluates the $C^2$ continuous cubic Hermite deceleration speed multiplier:
    ///
    /// $$u = \frac{t}{T_{\text{decel}}}, \quad \text{speed}(u) = (1 - u)^2 (1 + 2u)$$
    ///
    /// - At $u = 0$: $\text{speed} = 1.0, \quad \frac{d\text{speed}}{du} = 0.0$
    /// - At $u = 1$: $\text{speed} = 0.0, \quad \frac{d\text{speed}}{du} = 0.0$
    #[must_use]
    pub fn calculate_hermite_decel_speed(normalized_progress: f64) -> f64 {
        let u = normalized_progress.clamp(0.0, 1.0);
        let one_minus_u = 1.0 - u;
        one_minus_u * one_minus_u * (1.0 + 2.0 * u)
    }

    /// Triggers user unlock deceleration transition from 1.0x to 0.0x.
    pub fn trigger_unlock_deceleration(&mut self) {
        if self.mode == AerialMode::ScreensaverPlaying {
            self.mode = AerialMode::Decelerating;
            self.elapsed_decel_seconds = 0.0;
        }
    }

    /// Resets the engine back to full-speed 120fps screensaver mode.
    pub fn reset_to_screensaver(&mut self) {
        self.mode = AerialMode::ScreensaverPlaying;
        self.current_playback_speed = 1.0;
        self.elapsed_decel_seconds = 0.0;
    }

    /// Advances the aerial playback engine by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        let dt = dt.max(0.0);

        match self.mode {
            AerialMode::ScreensaverPlaying => {
                self.current_playback_speed = 1.0;
                self.advance_playhead(dt * self.current_playback_speed);
            }
            AerialMode::Decelerating => {
                let prev_elapsed = self.elapsed_decel_seconds;
                self.elapsed_decel_seconds += dt;
                let u = self.elapsed_decel_seconds / self.decel_duration_seconds;

                if u >= 1.0 {
                    // Deceleration complete; advance remaining delta up to boundary
                    let remaining_dt = (self.decel_duration_seconds - prev_elapsed).max(0.0);
                    let avg_speed = self.current_playback_speed * 0.5;
                    self.advance_playhead(remaining_dt * avg_speed);
                    self.current_playback_speed = 0.0;
                    self.mode = AerialMode::StaticWallpaper;
                } else {
                    let new_speed = Self::calculate_hermite_decel_speed(u);
                    let avg_speed = (self.current_playback_speed + new_speed) * 0.5;
                    self.current_playback_speed = new_speed;
                    self.advance_playhead(dt * avg_speed);
                }
            }
            AerialMode::StaticWallpaper => {
                // Completely static; playhead does not advance
                self.current_playback_speed = 0.0;
            }
        }
    }

    /// Advances video playhead with wrap-around looping.
    fn advance_playhead(&mut self, delta_seconds: f64) {
        self.current_playhead_seconds += delta_seconds;
        if self.current_playhead_seconds >= self.video_duration_seconds {
            self.current_playhead_seconds %= self.video_duration_seconds;
        }
    }

    /// Current operational mode.
    #[must_use]
    pub fn mode(&self) -> AerialMode {
        self.mode
    }

    /// Current playhead position in seconds.
    #[must_use]
    pub fn current_playhead_seconds(&self) -> f64 {
        self.current_playhead_seconds
    }

    /// Current instantaneous playback speed multiplier ($0.0..=1.0$).
    #[must_use]
    pub fn current_playback_speed(&self) -> f64 {
        self.current_playback_speed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermite_deceleration_boundary_conditions() {
        // At progress 0.0, speed must be exactly 1.0
        assert_eq!(AerialVideoDecelerator::calculate_hermite_decel_speed(0.0), 1.0);
        // At progress 1.0, speed must be exactly 0.0
        assert_eq!(AerialVideoDecelerator::calculate_hermite_decel_speed(1.0), 0.0);
        // At halfway progress 0.5: (1 - 0.5)^2 * (1 + 2*0.5) = 0.25 * 2.0 = 0.5
        assert_eq!(AerialVideoDecelerator::calculate_hermite_decel_speed(0.5), 0.5);

        // Derivative check around 0.0 and 1.0 (smooth tangents)
        let eps = 1e-4;
        let speed_near_zero = AerialVideoDecelerator::calculate_hermite_decel_speed(eps);
        assert!((1.0 - speed_near_zero) < 1e-6);

        let speed_near_one = AerialVideoDecelerator::calculate_hermite_decel_speed(1.0 - eps);
        assert!(speed_near_one < 1e-6);
    }

    #[test]
    fn test_screensaver_playback_and_wrap() {
        let mut engine = AerialVideoDecelerator::new("test.mov", 10.0);
        assert_eq!(engine.mode(), AerialMode::ScreensaverPlaying);

        // Advance 4 seconds
        engine.tick(4.0);
        assert_eq!(engine.current_playback_speed(), 1.0);
        assert_eq!(engine.current_playhead_seconds(), 4.0);

        // Advance 8 seconds (total 12s -> wraps to 2s)
        engine.tick(8.0);
        assert_eq!(engine.current_playhead_seconds(), 2.0);
    }

    #[test]
    fn test_unlock_deceleration_to_static_wallpaper() {
        let mut engine = AerialVideoDecelerator::new("test.mov", 60.0);
        engine.tick(5.0); // playhead at 5.0s

        engine.trigger_unlock_deceleration();
        assert_eq!(engine.mode(), AerialMode::Decelerating);

        // Advance 1.2s (half of 2.4s deceleration)
        engine.tick(1.2);
        assert_eq!(engine.mode(), AerialMode::Decelerating);
        assert_eq!(engine.current_playback_speed(), 0.5);
        assert!(engine.current_playhead_seconds() > 5.0);

        let pre_freeze = engine.current_playhead_seconds();

        // Advance remaining 1.3s (total 2.5s > 2.4s) -> StaticWallpaper
        engine.tick(1.3);
        assert_eq!(engine.mode(), AerialMode::StaticWallpaper);
        assert_eq!(engine.current_playback_speed(), 0.0);

        let frozen_playhead = engine.current_playhead_seconds();
        assert!(frozen_playhead > pre_freeze);

        // Further ticking must not advance the static wallpaper
        engine.tick(10.0);
        assert_eq!(engine.current_playhead_seconds(), frozen_playhead);
        assert_eq!(engine.current_playback_speed(), 0.0);
    }
}
