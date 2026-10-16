//! # Quick Look 120fps Rect Expansion Animation Subsystem
//!
//! Provides continuous $C^2$ geometric morphing between the source icon bounding rect
//! $[x, y, w, h]$ and the destination preview window rect $[x, y, w, h]$ with spring physics.

use glam::Vec4;
use qz_animation::{SpringAnimation, SpringParams};

/// Rectangular geometry defined by $[x, y, width, height]$ in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectBounds {
    /// Top-left horizontal coordinate.
    pub x: f32,
    /// Top-left vertical coordinate.
    pub y: f32,
    /// Width extent in pixels.
    pub width: f32,
    /// Height extent in pixels.
    pub height: f32,
}

impl RectBounds {
    /// Creates a new bounding rect.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width: width.max(1.0),
            height: height.max(1.0),
        }
    }

    /// Converts bounds to a 4D vector $[x, y, w, h]$.
    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.x, self.y, self.width, self.height)
    }

    /// Calculates center coordinates $[cx, cy]$.
    pub fn center(&self) -> [f32; 2] {
        [self.x + self.width * 0.5, self.y + self.height * 0.5]
    }
}

/// The state of the Quick Look window expansion transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpansionPhase {
    /// Window is closed or completely dismissed into source rect.
    Dismissed,
    /// Window is smoothly expanding from thumbnail to preview dimensions.
    Expanding,
    /// Window is fully open at destination preview rect.
    Open,
    /// Window is shrinking back into source thumbnail rect.
    Collapsing,
}

/// 120fps analytical spring-driven geometric expansion controller.
pub struct RectExpansionAnimator {
    source_rect: RectBounds,
    target_rect: RectBounds,
    phase: ExpansionPhase,
    spring_progress: SpringAnimation,
    current_progress: f32,
}

impl Default for RectExpansionAnimator {
    fn default() -> Self {
        let dummy = RectBounds::new(0.0, 0.0, 100.0, 100.0);
        Self::new(dummy, dummy)
    }
}

impl RectExpansionAnimator {
    /// Canonical Quick Look Spacebar Morph spring stiffness ($k = 420.0$).
    pub const QUICKLOOK_SPRING_K: f64 = 420.0;
    /// Canonical Quick Look Spacebar Morph spring damping ($c = 36.0$).
    pub const QUICKLOOK_SPRING_C: f64 = 36.0;

    /// Creates a new animator morphing between `source_rect` and `target_rect`.
    pub fn new(source_rect: RectBounds, target_rect: RectBounds) -> Self {
        let spring_params = SpringParams {
            mass: 1.0,
            stiffness: Self::QUICKLOOK_SPRING_K,
            damping: Self::QUICKLOOK_SPRING_C,
        };
        let spring_progress = SpringAnimation::new(spring_params, 0.0, 0.0, 0.0);

        Self {
            source_rect,
            target_rect,
            phase: ExpansionPhase::Dismissed,
            spring_progress,
            current_progress: 0.0,
        }
    }

    /// Triggers the opening expansion animation toward target rect.
    pub fn open(&mut self) {
        self.phase = ExpansionPhase::Expanding;
        self.spring_progress.retarget(1.0);
    }

    /// Triggers the closing collapse animation toward source rect.
    pub fn close(&mut self) {
        self.phase = ExpansionPhase::Collapsing;
        self.spring_progress.retarget(0.0);
    }

    /// Updates destination rect (e.g. when retargeting to a file with different aspect ratio).
    pub fn retarget_destination(&mut self, new_target: RectBounds) {
        self.target_rect = new_target;
    }

    /// Updates source rect (e.g. when user selected a new file in Finder via arrow keys).
    pub fn retarget_source(&mut self, new_source: RectBounds) {
        self.source_rect = new_source;
    }

    /// Advances the spring physics simulation by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        let state = self.spring_progress.step(dt as f64);
        self.current_progress = state.position as f32;

        match self.phase {
            ExpansionPhase::Expanding => {
                if (self.current_progress - 1.0).abs() < 0.005 && state.velocity.abs() < 0.05 {
                    self.phase = ExpansionPhase::Open;
                    self.current_progress = 1.0;
                }
            }
            ExpansionPhase::Collapsing => {
                if self.current_progress.abs() < 0.005 && state.velocity.abs() < 0.05 {
                    self.phase = ExpansionPhase::Dismissed;
                    self.current_progress = 0.0;
                }
            }
            _ => {}
        }
    }

    /// Computes the instantaneous interpolated bounding rect $[x, y, w, h]$.
    pub fn current_rect(&self) -> RectBounds {
        let t = self.current_progress;

        let x = self.source_rect.x + (self.target_rect.x - self.source_rect.x) * t;
        let y = self.source_rect.y + (self.target_rect.y - self.source_rect.y) * t;
        let w = self.source_rect.width + (self.target_rect.width - self.source_rect.width) * t;
        let h = self.source_rect.height + (self.target_rect.height - self.source_rect.height) * t;

        RectBounds::new(x, y, w, h)
    }

    /// Current expansion progress factor $[0.0, 1.0]$.
    #[inline]
    pub fn progress(&self) -> f32 {
        self.current_progress
    }

    /// Current phase of the animation.
    #[inline]
    pub fn phase(&self) -> ExpansionPhase {
        self.phase
    }

    /// Source thumbnail rect.
    #[inline]
    pub fn source_rect(&self) -> RectBounds {
        self.source_rect
    }

    /// Target preview rect.
    #[inline]
    pub fn target_rect(&self) -> RectBounds {
        self.target_rect
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_expansion_continuous_interpolation() {
        let source = RectBounds::new(100.0, 200.0, 64.0, 64.0);
        let target = RectBounds::new(400.0, 300.0, 800.0, 600.0);

        let mut animator = RectExpansionAnimator::new(source, target);
        assert_eq!(animator.phase(), ExpansionPhase::Dismissed);
        assert_eq!(animator.current_rect(), source);

        animator.open();
        assert_eq!(animator.phase(), ExpansionPhase::Expanding);

        // Advance 120fps frames
        for _ in 0..60 {
            animator.update(1.0 / 120.0);
        }

        assert!(animator.progress() > 0.5);
        let current = animator.current_rect();
        assert!(current.width > 64.0);
        assert!(current.height > 64.0);

        // Mid-flight reversal (closing)
        animator.close();
        assert_eq!(animator.phase(), ExpansionPhase::Collapsing);

        for _ in 0..120 {
            animator.update(1.0 / 120.0);
        }

        assert_eq!(animator.phase(), ExpansionPhase::Dismissed);
        assert!((animator.current_rect().width - 64.0).abs() < 1.0);
    }
}
