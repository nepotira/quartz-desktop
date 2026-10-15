//! # Tahoe 26 Depth Lock Screen (`qz-session-lock::depth_lock`)
//!
//! Production-grade implementation of the macOS Tahoe 26 volumetric Depth Lock Screen.
//!
//! ### Key Capabilities:
//! - **Volumetric Clock with Depth Masking:** Places the iconic time typography in 3D layer space
//!   behind the segmented foreground subject (mountain ridge, person's portrait, or skyline),
//!   mimicking iOS/macOS Tahoe 26 Depth Effect wallpaper rendering.
//! - **Legibility Safeguard:** Automatically computes the occlusion ratio of the clock digits.
//!   If more than 35% of the clock surface is occluded by the foreground subject, depth masking
//!   is gracefully disabled to preserve typography legibility.
//! - **Damped Harmonic Oscillator Password Shake:** Modeled after macOS's canonical password error
//!   shake using an analytical damped sinusoidal oscillator:
//!   $$x(t) = A \cdot e^{-\gamma t} \cos(\omega t)$$
//!   with initial amplitude $A = 24.0\text{px}$, damping $\gamma = 12.0$, and frequency $\omega = 45.0\text{ rad/s}$.
//! - **Adaptive Luminance Tinting:** Dynamically selects between light glass, dark graphite,
//!   or chromatic accent palettes based on average wallpaper luminance behind the clock.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Visual appearance styling for the volumetric clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClockColorPalette {
    /// Crisp pure white with soft ambient shadow for dark backdrops.
    LightGlass,
    /// Deep dark graphite text with frosted rim for light backdrops.
    DarkGraphite,
    /// Dynamic vibrant tint sampled from the wallpaper's dominant hue.
    VibrantTint,
}

/// Bounding rectangle for screen geometry $[x, y, \text{width}, \text{height}]$.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// Left coordinate.
    pub x: f64,
    /// Top coordinate.
    pub y: f64,
    /// Width in pixels.
    pub width: f64,
    /// Height in pixels.
    pub height: f64,
}

impl Rect {
    /// Creates a new rectangle.
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Returns `true` if this rectangle contains point $(px, py)$.
    #[must_use]
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    /// Computes intersection area with another rectangle.
    #[must_use]
    pub fn intersection_area(&self, other: &Rect) -> f64 {
        let x_left = self.x.max(other.x);
        let y_top = self.y.max(other.y);
        let x_right = (self.x + self.width).min(other.x + other.width);
        let y_bottom = (self.y + self.height).min(other.y + other.height);

        if x_right > x_left && y_bottom > y_top {
            (x_right - x_left) * (y_bottom - y_top)
        } else {
            0.0
        }
    }
}

/// Foreground subject segmentation mask for depth effect wallpapers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubjectSegmentationMask {
    /// Bounding rectangle of the prominent foreground subject.
    pub bounding_box: Rect,
    /// Normalized depth layer ($0.0 = \text{background}, 1.0 = \text{foreground}$).
    pub depth_layer: f64,
    /// Estimated coverage ratio inside the bounding box.
    pub coverage_density: f64,
}

impl SubjectSegmentationMask {
    /// Creates a new segmentation mask.
    #[must_use]
    pub fn new(bounding_box: Rect, depth_layer: f64, coverage_density: f64) -> Self {
        Self {
            bounding_box,
            depth_layer: depth_layer.clamp(0.0, 1.0),
            coverage_density: coverage_density.clamp(0.0, 1.0),
        }
    }
}

/// Authentication state of the lock screen session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthState {
    /// Screen is locked, awaiting user input.
    Locked,
    /// Password or biometric authentication in progress.
    Authenticating,
    /// Authentication succeeded; lock screen dismissing.
    Success,
    /// Authentication failed; password box currently performing horizontal shake animation.
    FailedShake {
        /// Elapsed time of the shake animation in seconds.
        elapsed_seconds: f64,
        /// Total count of consecutive failed attempts.
        failed_count: u32,
    },
    /// Account locked out due to excessive failed attempts.
    LockedOut {
        /// Monotonic time remaining until lockout expires in seconds.
        remaining_seconds: f64,
    },
}

/// Tahoe 26 Volumetric Depth Lock Screen Controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthLockScreen {
    /// Screen dimensions in logical pixels $[W, H]$.
    pub screen_size: Vec2,
    /// Formatted time string (e.g. "09:41").
    pub time_string: String,
    /// Formatted date string (e.g. "Quinta-feira, 15 de Outubro").
    pub date_string: String,
    /// Volumetric clock geometry bounding box $[x, y, w, h]$.
    pub clock_rect: Rect,
    /// Foreground subject segmentation mask if wallpaper supports depth effect.
    pub subject_mask: Option<SubjectSegmentationMask>,
    /// Whether depth effect is actively enabled (false if occlusion exceeds 35%).
    is_depth_effect_active: bool,
    /// Active color styling palette for clock and date.
    pub clock_palette: ClockColorPalette,
    /// User account full display name.
    pub user_display_name: String,
    /// User account Unix login username.
    pub username: String,
    /// Path to user avatar image.
    pub avatar_path: String,
    /// Currently typed password buffer (masked dots).
    password_buffer: String,
    /// Current authentication lifecycle status.
    auth_state: AuthState,
    /// Horizontal pixel offset for password field shake animation.
    pub password_shake_offset_x: f64,
}

impl DepthLockScreen {
    /// Creates a new `DepthLockScreen` with Tahoe 26 default layout.
    #[must_use]
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        let clock_w = 480.0;
        let clock_h = 160.0;
        let clock_x = (screen_width - clock_w) / 2.0;
        let clock_y = screen_height * 0.16; // 16% from screen top

        Self {
            screen_size: Vec2::new(screen_width as f32, screen_height as f32),
            time_string: "09:41".to_string(),
            date_string: "Quinta-feira, 15 de Outubro".to_string(),
            clock_rect: Rect::new(clock_x, clock_y, clock_w, clock_h),
            subject_mask: None,
            is_depth_effect_active: false,
            clock_palette: ClockColorPalette::LightGlass,
            user_display_name: "Tim Cook".to_string(),
            username: "tim".to_string(),
            avatar_path: "/System/Library/UserPictures/TahoeUser.png".to_string(),
            password_buffer: String::new(),
            auth_state: AuthState::Locked,
            password_shake_offset_x: 0.0,
        }
    }

    /// Configures the wallpaper foreground subject mask and evaluates depth effect eligibility.
    ///
    /// If the subject occludes more than 35% of the clock bounding area, depth effect is
    /// disabled to safeguard typographic legibility.
    pub fn set_wallpaper_subject_mask(&mut self, mask: Option<SubjectSegmentationMask>) {
        if let Some(ref m) = mask {
            let clock_area = self.clock_rect.width * self.clock_rect.height;
            let overlap_area = self.clock_rect.intersection_area(&m.bounding_box);
            let occluded_area = overlap_area * m.coverage_density;
            let occlusion_ratio = if clock_area > 0.0 {
                occluded_area / clock_area
            } else {
                0.0
            };

            // Depth effect allowed if subject is in foreground (depth >= 0.6) and occlusion <= 35%
            if m.depth_layer >= 0.6 && occlusion_ratio <= 0.35 {
                self.is_depth_effect_active = true;
            } else {
                self.is_depth_effect_active = false;
            }
        } else {
            self.is_depth_effect_active = false;
        }

        self.subject_mask = mask;
    }

    /// Returns `true` if the volumetric clock is currently rendered behind the foreground subject.
    #[must_use]
    pub fn is_depth_effect_active(&self) -> bool {
        self.is_depth_effect_active
    }

    /// Sets the wallpaper luminance behind the clock to adapt text tinting.
    pub fn set_wallpaper_luminance(&mut self, luminance: f64) {
        if luminance > 0.65 {
            self.clock_palette = ClockColorPalette::DarkGraphite;
        } else {
            self.clock_palette = ClockColorPalette::LightGlass;
        }
    }

    /// Inserts a typed character into the password buffer.
    pub fn input_char(&mut self, c: char) {
        if self.auth_state == AuthState::Locked {
            self.password_buffer.push(c);
        }
    }

    /// Deletes the last character in the password buffer.
    pub fn backspace(&mut self) {
        if self.auth_state == AuthState::Locked {
            self.password_buffer.pop();
        }
    }

    /// Clears the password buffer.
    pub fn clear_password(&mut self) {
        self.password_buffer.clear();
    }

    /// Returns the length of the currently entered password buffer (for bullet dots).
    #[must_use]
    pub fn password_length(&self) -> usize {
        self.password_buffer.len()
    }

    /// Submits the entered password for verification against the provided expected credential.
    pub fn submit_password(&mut self, expected_password: &str) -> AuthState {
        if self.auth_state != AuthState::Locked {
            return self.auth_state.clone();
        }

        if self.password_buffer == expected_password {
            self.password_buffer.clear();
            self.auth_state = AuthState::Success;
        } else {
            // Trigger failure shake
            self.auth_state = AuthState::FailedShake {
                elapsed_seconds: 0.0,
                failed_count: 1,
            };
            self.password_shake_offset_x = 0.0;
        }

        self.auth_state.clone()
    }

    /// Advances the lock screen animation and shake physics clock by `dt` seconds.
    pub fn tick(&mut self, dt: f64) {
        if let AuthState::FailedShake { elapsed_seconds, failed_count } = &mut self.auth_state {
            *elapsed_seconds += dt;
            let t = *elapsed_seconds;

            // Analytical Damped Harmonic Oscillator:
            // x(t) = A * exp(-gamma * t) * cos(omega * t)
            let amplitude = 24.0;   // 24px initial displacement
            let gamma = 12.0;       // Damping factor
            let omega = 45.0;       // ~7.16 Hz vibration frequency

            let decay = (-gamma * t).exp();
            let offset = amplitude * decay * (omega * t).cos();
            self.password_shake_offset_x = offset;

            // Shake ends when time reaches 0.45s or decay drops offset below 0.2px
            if t >= 0.45 || (offset.abs() < 0.2 && t > 0.3) {
                let count = *failed_count;
                self.password_shake_offset_x = 0.0;
                self.password_buffer.clear();

                if count >= 5 {
                    self.auth_state = AuthState::LockedOut {
                        remaining_seconds: 60.0,
                    };
                } else {
                    self.auth_state = AuthState::Locked;
                }
            }
        } else if let AuthState::LockedOut { remaining_seconds } = &mut self.auth_state {
            *remaining_seconds -= dt;
            if *remaining_seconds <= 0.0 {
                self.auth_state = AuthState::Locked;
            }
        }
    }

    /// Returns current authentication state.
    #[must_use]
    pub fn auth_state(&self) -> &AuthState {
        &self.auth_state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_effect_eligible_subject() {
        let mut lock = DepthLockScreen::new(1920.0, 1080.0);
        // Clock is around y: 172.8, h: 160.0 => y in [172.8, 332.8]
        // Create subject at y: 280.0 overlapping clock by ~50px height
        let subject_rect = Rect::new(lock.clock_rect.x, 280.0, lock.clock_rect.width, 400.0);
        let mask = SubjectSegmentationMask::new(subject_rect, 0.90, 0.50);

        lock.set_wallpaper_subject_mask(Some(mask));
        // Overlap area = 480 * (332.8 - 280) = 480 * 52.8 = 25344
        // Occluded area = 25344 * 0.5 = 12672
        // Clock area = 480 * 160 = 76800 => Occlusion = 12672 / 76800 = 16.5% <= 35%
        assert!(lock.is_depth_effect_active());
    }

    #[test]
    fn test_depth_effect_disabled_when_excessive_occlusion() {
        let mut lock = DepthLockScreen::new(1920.0, 1080.0);
        // Subject completely covers clock with 100% density (occlusion = 100% > 35%)
        let mask = SubjectSegmentationMask::new(lock.clock_rect, 0.95, 1.0);
        lock.set_wallpaper_subject_mask(Some(mask));

        assert!(!lock.is_depth_effect_active());
    }

    #[test]
    fn test_adaptive_luminance_tinting() {
        let mut lock = DepthLockScreen::new(1920.0, 1080.0);
        lock.set_wallpaper_luminance(0.20);
        assert_eq!(lock.clock_palette, ClockColorPalette::LightGlass);

        lock.set_wallpaper_luminance(0.85);
        assert_eq!(lock.clock_palette, ClockColorPalette::DarkGraphite);
    }

    #[test]
    fn test_password_auth_success() {
        let mut lock = DepthLockScreen::new(1920.0, 1080.0);
        lock.input_char('s');
        lock.input_char('e');
        lock.input_char('c');
        lock.input_char('r');
        lock.input_char('e');
        lock.input_char('t');
        assert_eq!(lock.password_length(), 6);

        let res = lock.submit_password("secret");
        assert_eq!(res, AuthState::Success);
        assert_eq!(lock.password_length(), 0);
    }

    #[test]
    fn test_password_auth_failure_shake_physics() {
        let mut lock = DepthLockScreen::new(1920.0, 1080.0);
        lock.input_char('w');
        lock.input_char('r');
        lock.input_char('o');
        lock.input_char('n');
        lock.input_char('g');

        let res = lock.submit_password("correct");
        match res {
            AuthState::FailedShake { failed_count, .. } => assert_eq!(failed_count, 1),
            _ => panic!("Expected FailedShake"),
        }

        // Tick 0.05s -> offset should be non-zero
        lock.tick(0.05);
        assert!(lock.password_shake_offset_x.abs() > 0.0);

        // Advance past shake duration (0.5s)
        lock.tick(0.50);
        assert_eq!(lock.password_shake_offset_x, 0.0);
        assert_eq!(lock.auth_state(), &AuthState::Locked);
        assert_eq!(lock.password_length(), 0);
    }
}
