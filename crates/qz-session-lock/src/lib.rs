//! # QuartzLinux Session Lock Subsystem (`qz-session-lock`)
//!
//! Production-grade session lock screen, security protocol, aerial screensaver decelerator,
//! and 3D Cube Fast User Switch for macOS Tahoe 26.
//!
//! ### Subsystems Included:
//! 1. **Tahoe 26 Depth Lock Screen ([`depth_lock`]):**
//!    - Volumetric clock with Depth Effect rendered behind segmented wallpaper subjects.
//!    - Occlusion ratio safeguard disabling depth effect if >35% of digits are hidden.
//!    - Damped sinusoidal harmonic oscillator password shake animation ($x(t) = A e^{-\gamma t} \cos(\omega t)$).
//!    - Adaptive text tinting based on backdrop luminance.
//! 2. **120fps Aerial Video Decelerator ([`aerial_decelerator`]):**
//!    - 120fps ProMotion continuous drone video screensaver.
//!    - $C^2$ continuous cubic Hermite deceleration upon unlock from 1.0x to 0.0x over 2.4s.
//!    - Exact frame freezing seamlessly morphing into static desktop wallpaper.
//! 3. **3D Cube Fast User Switch ([`cube_switch`]):**
//!    - Spatial perspective projection ($\text{FOV} = 45^\circ$, analytical pullback distance $d \approx 2.4142$).
//!    - Interactive two-finger trackpad drag with $45^\circ$ commit threshold.
//!    - Continuous spring physics with `mission_control` parameters ($\zeta = 1.0$).
//! 4. **Zero-Flicker Boot Handshake ([`boot_handshake`]):**
//!    - DRM KMS scanout framebuffer preservation eliminating black flash from Plymouth.
//!    - Wayland `ext-session-lock-v1` security compliance and smooth 300ms desktop crossfade.

#![deny(missing_docs)]

pub mod aerial_decelerator;
pub mod boot_handshake;
pub mod cube_switch;
pub mod depth_lock;

pub use aerial_decelerator::{AerialMode, AerialVideoDecelerator};
pub use boot_handshake::{BootHandshakeManager, HandshakePhase};
pub use cube_switch::{CubeRotationDirection, CubeSwitchEngine, CubeSwitchState};
pub use depth_lock::{
    AuthState, ClockColorPalette, DepthLockScreen, Rect, SubjectSegmentationMask,
};

use serde::{Deserialize, Serialize};

/// High-level events emitted by the session lock coordinator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionLockEvent {
    /// Screen unlocked; desktop should be revealed.
    Unlocked,
    /// Fast user switch committed to new user session.
    SwitchedUser(u32),
    /// Redraw requested for new visual frame.
    RequestRedraw,
}

/// Unified coordinator managing all lock screen and session transition components.
#[derive(Debug, Clone)]
pub struct SessionLockCoordinator {
    /// Volumetric depth effect lock screen controller.
    pub lock_screen: DepthLockScreen,
    /// 120fps aerial video screensaver decelerator.
    pub aerial: AerialVideoDecelerator,
    /// 3D cube fast user switcher.
    pub cube: CubeSwitchEngine,
    /// Zero-flicker boot and Wayland security handshake manager.
    pub handshake: BootHandshakeManager,
}

impl SessionLockCoordinator {
    /// Creates a new `SessionLockCoordinator` for a display with given dimensions.
    #[must_use]
    pub fn new(screen_width: f64, screen_height: f64) -> Self {
        let aspect = (screen_width / screen_height.max(1.0)) as f32;
        let lock_screen = DepthLockScreen::new(screen_width, screen_height);
        let aerial = AerialVideoDecelerator::default();
        let cube = CubeSwitchEngine::new(aspect, 501, 502);
        let mut handshake = BootHandshakeManager::default();
        handshake.present_session_lock();

        Self {
            lock_screen,
            aerial,
            cube,
            handshake,
        }
    }

    /// Advances the simulation clock across all session lock subsystems by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) -> Vec<SessionLockEvent> {
        let mut events = Vec::new();

        self.lock_screen.tick(dt);
        self.aerial.tick(dt);
        self.cube.tick(dt);
        self.handshake.tick(dt);

        // Check if authentication completed
        if self.lock_screen.auth_state() == &AuthState::Success
            && self.handshake.phase() == HandshakePhase::SessionLockActive
        {
            self.aerial.trigger_unlock_deceleration();
            self.handshake.trigger_unlock();
        }

        // Check if handshake finished crossfade to desktop
        if self.handshake.phase() == HandshakePhase::UnlockedDesktop {
            events.push(SessionLockEvent::Unlocked);
        }

        // Check if 3D cube user switch finished
        if let CubeSwitchState::Completed { active_user_id } = *self.cube.state() {
            events.push(SessionLockEvent::SwitchedUser(active_user_id));
        }

        events
    }

    /// Submits entered password for verification.
    pub fn submit_password(&mut self, expected: &str) -> bool {
        let res = self.lock_screen.submit_password(expected);
        if res == AuthState::Success {
            self.aerial.trigger_unlock_deceleration();
            self.handshake.trigger_unlock();
            true
        } else {
            false
        }
    }

    /// Triggers 3D cube user switch.
    pub fn switch_user(&mut self, direction: CubeRotationDirection) {
        self.cube.trigger_switch(direction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_initial_state() {
        let coordinator = SessionLockCoordinator::new(1920.0, 1080.0);
        assert!(coordinator.handshake.is_locked());
        assert_eq!(coordinator.aerial.mode(), AerialMode::ScreensaverPlaying);
        assert_eq!(coordinator.lock_screen.auth_state(), &AuthState::Locked);
    }

    #[test]
    fn test_unlock_pipeline_end_to_end() {
        let mut coordinator = SessionLockCoordinator::new(1920.0, 1080.0);

        // Type password "tahoe26"
        for c in "tahoe26".chars() {
            coordinator.lock_screen.input_char(c);
        }

        // Submit password
        let success = coordinator.submit_password("tahoe26");
        assert!(success);
        assert_eq!(coordinator.aerial.mode(), AerialMode::Decelerating);
        assert_eq!(coordinator.handshake.phase(), HandshakePhase::CrossfadingToDesktop);

        // Advance 0.35s to complete crossfade
        let events = coordinator.tick(0.35);
        assert!(events.contains(&SessionLockEvent::Unlocked));
        assert!(!coordinator.handshake.is_locked());
    }

    #[test]
    fn test_coordinator_cube_switch() {
        let mut coordinator = SessionLockCoordinator::new(1920.0, 1080.0);
        coordinator.switch_user(CubeRotationDirection::RotateRight);

        // Advance until spring settles
        let mut switched = false;
        for _ in 0..100 {
            let events = coordinator.tick(0.02);
            if events.contains(&SessionLockEvent::SwitchedUser(502)) {
                switched = true;
                break;
            }
        }
        assert!(switched);
    }
}
