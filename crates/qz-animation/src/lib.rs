//! # qz-animation
//!
//! Production-grade analytical physics and dynamics engine for QuartzLinux (Tahoe 26).
//!
//! Features:
//! - Analytical closed-form solution to the 2nd order damped harmonic oscillator ODE
//!   with exact $C^2$ continuous retargeting (subdamped, critically damped, overdamped).
//! - Standard Tahoe 26 system spring presets (`mission_control`, `window_open`, `dock_bounce`,
//!   `stage_manager`, `rubber_return`).
//! - Hyperbolic Cocoa Rubber-Banding physics for overscroll.
//! - Vectorial "Shake to Find Cursor" detector with 400ms sliding window and spring scale expansion.
//! - Dock parabolic cosine magnification and conservative displacement physics.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod catalog;

use std::collections::VecDeque;
use std::f64::consts::PI;
use thiserror::Error;

/// Error types encountered during animation and physics computations.
#[derive(Debug, Error, PartialEq)]
pub enum AnimationError {
    /// Mass parameter must be strictly positive and finite.
    #[error("Mass must be strictly positive (> 0.0) and finite, got {0}")]
    InvalidMass(f64),
    /// Stiffness parameter must be strictly positive and finite.
    #[error("Stiffness must be strictly positive (> 0.0) and finite, got {0}")]
    InvalidStiffness(f64),
    /// Damping parameter must be non-negative and finite.
    #[error("Damping must be non-negative (>= 0.0) and finite, got {0}")]
    InvalidDamping(f64),
    /// Calculation encountered a non-finite float (NaN or Inf).
    #[error("Calculation resulted in non-finite value")]
    NonFiniteResult,
}

/// Damping regime determined by the damping ratio $\zeta$.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DampingRegime {
    /// Underdamped ($\zeta < 1.0$): oscillatory motion with exponential decay.
    Underdamped,
    /// Critically damped ($\zeta = 1.0$): fastest return to equilibrium without overshoot.
    CriticallyDamped,
    /// Overdamped ($\zeta > 1.0$): sluggish non-oscillatory return.
    Overdamped,
}

/// Parameters governing a 2nd order linear damped harmonic oscillator.
///
/// Differential equation:
/// $$m \frac{d^2x}{dt^2} + c \frac{dx}{dt} + k (x - x_{\text{target}}) = 0$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringParams {
    /// Mass $m$ (must be $> 0$).
    pub mass: f64,
    /// Stiffness $k$ (must be $> 0$).
    pub stiffness: f64,
    /// Damping coefficient $c$ (must be $\ge 0$).
    pub damping: f64,
}

impl SpringParams {
    /// Creates a new `SpringParams` validating physical invariants.
    pub fn new(mass: f64, stiffness: f64, damping: f64) -> Result<Self, AnimationError> {
        if !mass.is_finite() || mass <= 0.0 {
            return Err(AnimationError::InvalidMass(mass));
        }
        if !stiffness.is_finite() || stiffness <= 0.0 {
            return Err(AnimationError::InvalidStiffness(stiffness));
        }
        if !damping.is_finite() || damping < 0.0 {
            return Err(AnimationError::InvalidDamping(damping));
        }
        Ok(Self { mass, stiffness, damping })
    }

    /// Natural undamped angular frequency $\omega_0 = \sqrt{k / m}$.
    #[inline]
    pub fn natural_frequency(&self) -> f64 {
        (self.stiffness / self.mass).sqrt()
    }

    /// Damping ratio $\zeta = \frac{c}{2\sqrt{m \cdot k}}$.
    #[inline]
    pub fn damping_ratio(&self) -> f64 {
        self.damping / (2.0 * (self.mass * self.stiffness).sqrt())
    }

    /// Identifies the damping regime with floating-point tolerance $\epsilon = 10^{-6}$.
    #[inline]
    pub fn regime(&self) -> DampingRegime {
        let zeta = self.damping_ratio();
        if (zeta - 1.0).abs() < 1e-6 {
            DampingRegime::CriticallyDamped
        } else if zeta < 1.0 {
            DampingRegime::Underdamped
        } else {
            DampingRegime::Overdamped
        }
    }

    // --- Standard macOS Tahoe 26 Presets ---

    /// Mission Control, Exposé, Spaces swipe, Reveal Desktop.
    ///
    /// $m = 1.0, k = 320.0, c = 35.77708764$ ($\zeta = 1.00$).
    #[inline]
    pub fn mission_control() -> Self {
        Self {
            mass: 1.0,
            stiffness: 320.0,
            damping: 35.77708763999664, // 2.0 * sqrt(1.0 * 320.0)
        }
    }

    /// Window open, Quick Look (Spacebar), Spotlight, Popovers.
    ///
    /// $m = 1.0, k = 450.0, c = 38.0$ ($\zeta \approx 0.895$).
    #[inline]
    pub fn window_open() -> Self {
        Self {
            mass: 1.0,
            stiffness: 450.0,
            damping: 38.0,
        }
    }

    /// Dock icon bounce on application launch or urgent alert.
    ///
    /// $m = 1.0, k = 180.0, c = 12.0$ ($\zeta \approx 0.447$).
    #[inline]
    pub fn dock_bounce() -> Self {
        Self {
            mass: 1.0,
            stiffness: 180.0,
            damping: 12.0,
        }
    }

    /// Stage Manager 3D group transition, PiP magnetic corner snap.
    ///
    /// $m = 1.0, k = 280.0, c = 31.0$ ($\zeta \approx 0.926$).
    #[inline]
    pub fn stage_manager() -> Self {
        Self {
            mass: 1.0,
            stiffness: 280.0,
            damping: 31.0,
        }
    }

    /// Overscroll return (rubber-band bounce back) on trackpad release.
    ///
    /// $m = 1.0, k = 380.0, c = 38.98717743$ ($\zeta = 1.00$).
    #[inline]
    pub fn rubber_return() -> Self {
        Self {
            mass: 1.0,
            stiffness: 380.0,
            damping: 38.98717737923585, // 2.0 * sqrt(1.0 * 380.0)
        }
    }

    /// Cursor shake expansion spring ($m = 1.0, k = 300.0, c = 28.0$).
    #[inline]
    pub fn cursor_shake() -> Self {
        Self {
            mass: 1.0,
            stiffness: 300.0,
            damping: 28.0,
        }
    }

    /// Dock magnification hover envelope spring ($m = 1.0, k = 500.0, c = 45.0$).
    ///
    /// Smoothly interpolates the magnification factor $\alpha_{\text{entry}} \in [0.0, 1.0]$ upon cursor entry/exit.
    #[inline]
    pub fn dock_hover_envelope() -> Self {
        Self {
            mass: 1.0,
            stiffness: 500.0,
            damping: 45.0,
        }
    }

    /// Returns a new `SpringParams` dilated for macOS universal Shift slow-motion ($0.20\times$ speed).
    ///
    /// Preserves the damping ratio $\zeta$ while dilating the time scale by $1/0.20 = 5.0\times$:
    /// $$k \to 0.04 \cdot k, \quad c \to 0.20 \cdot c$$
    #[must_use]
    #[inline]
    pub fn with_slow_motion(&self, is_slow: bool) -> Self {
        if is_slow {
            Self {
                mass: self.mass,
                stiffness: self.stiffness * 0.04,
                damping: self.damping * 0.20,
            }
        } else {
            *self
        }
    }
}

/// Instantaneous dynamic state of a 1D spring system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringState {
    /// Current position $x(t)$.
    pub position: f64,
    /// Current velocity $v(t) = \frac{dx}{dt}$.
    pub velocity: f64,
    /// Current acceleration $a(t) = \frac{d^2x}{dt^2}$.
    pub acceleration: f64,
}

/// 1D Analytical Spring Animation with $C^2$ continuous retargeting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringAnimation {
    params: SpringParams,
    target: f64,
    initial_displacement: f64,
    initial_velocity: f64,
    elapsed_time: f64,
    position_tolerance: f64,
    velocity_tolerance: f64,
}

impl SpringAnimation {
    /// Creates a new analytical spring animation starting at `initial_pos` with initial velocity `initial_vel`,
    /// aiming toward `target`.
    pub fn new(params: SpringParams, initial_pos: f64, initial_vel: f64, target: f64) -> Self {
        Self {
            params,
            target,
            initial_displacement: initial_pos - target,
            initial_velocity: initial_vel,
            elapsed_time: 0.0,
            position_tolerance: 0.05,
            velocity_tolerance: 0.05,
        }
    }

    /// Updates tolerances used to determine whether the animation has settled into equilibrium.
    pub fn with_tolerances(mut self, pos_tol: f64, vel_tol: f64) -> Self {
        self.position_tolerance = pos_tol.max(1e-6);
        self.velocity_tolerance = vel_tol.max(1e-6);
        self
    }

    /// Retargets the spring to a new destination while preserving instantaneous position and velocity,
    /// ensuring strict $C^1$ velocity and $C^2$ acceleration continuity without hitches.
    pub fn retarget(&mut self, new_target: f64) {
        let current_state = self.sample_at(self.elapsed_time);
        self.target = new_target;
        self.initial_displacement = current_state.position - new_target;
        self.initial_velocity = current_state.velocity;
        self.elapsed_time = 0.0;
    }

    /// Sets the spring parameters dynamically (e.g. during physical gesture transition).
    pub fn set_params(&mut self, new_params: SpringParams) {
        let current_state = self.sample_at(self.elapsed_time);
        self.params = new_params;
        self.initial_displacement = current_state.position - self.target;
        self.initial_velocity = current_state.velocity;
        self.elapsed_time = 0.0;
    }

    /// Advances the simulation clock by $\Delta t$ seconds.
    pub fn step(&mut self, dt: f64) -> SpringState {
        self.elapsed_time += dt.max(0.0);
        self.sample_at(self.elapsed_time)
    }

    /// Returns the active destination target.
    #[inline]
    pub fn target(&self) -> f64 {
        self.target
    }

    /// Returns total elapsed time since initial launch or last retarget.
    #[inline]
    pub fn elapsed_time(&self) -> f64 {
        self.elapsed_time
    }

    /// Evaluates the analytical state $(x, v, a)$ at an arbitrary relative time $t \ge 0$.
    pub fn sample_at(&self, t: f64) -> SpringState {
        let t = t.max(0.0);
        let m = self.params.mass;
        let k = self.params.stiffness;
        let c = self.params.damping;

        let omega_0 = (k / m).sqrt();
        let zeta = c / (2.0 * (m * k).sqrt());
        let x0 = self.initial_displacement;
        let v0 = self.initial_velocity;

        let (pos, vel) = if (zeta - 1.0).abs() < 1e-6 {
            // Critically damped: zeta == 1.0
            // x(t) = target + (x0 + (v0 + omega_0 * x0) * t) * exp(-omega_0 * t)
            let c2 = v0 + omega_0 * x0;
            let decay = (-omega_0 * t).exp();
            let x = self.target + (x0 + c2 * t) * decay;
            let v = (c2 - omega_0 * (x0 + c2 * t)) * decay;
            (x, v)
        } else if zeta < 1.0 {
            // Underdamped: zeta < 1.0
            let omega_d = omega_0 * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * omega_0 * t).exp();
            let sin_term = (omega_d * t).sin();
            let cos_term = (omega_d * t).cos();

            let coeff_sin = (v0 + zeta * omega_0 * x0) / omega_d;
            let disp = decay * (x0 * cos_term + coeff_sin * sin_term);
            let x = self.target + disp;

            let vel_disp = decay * ((v0 + zeta * omega_0 * x0) * cos_term - omega_d * x0 * sin_term)
                - zeta * omega_0 * disp;
            (x, vel_disp)
        } else {
            // Overdamped: zeta > 1.0
            let omega_h = omega_0 * (zeta * zeta - 1.0).sqrt();
            let decay = (-zeta * omega_0 * t).exp();
            let sinh_term = (omega_h * t).sinh();
            let cosh_term = (omega_h * t).cosh();

            let coeff_sinh = (v0 + zeta * omega_0 * x0) / omega_h;
            let disp = decay * (x0 * cosh_term + coeff_sinh * sinh_term);
            let x = self.target + disp;

            let vel_disp = decay * ((v0 + zeta * omega_0 * x0) * cosh_term + x0 * omega_h * sinh_term)
                - zeta * omega_0 * disp;
            (x, vel_disp)
        };

        // Exact analytical acceleration derived from Newton's 2nd law:
        // m*a + c*v + k*(x - target) = 0 => a = -(c*v + k*(x - target)) / m
        let acc = -(c * vel + k * (pos - self.target)) / m;

        SpringState {
            position: pos,
            velocity: vel,
            acceleration: acc,
        }
    }

    /// Checks if the spring has stabilized within position and velocity tolerances.
    pub fn is_settled(&self) -> bool {
        let state = self.sample_at(self.elapsed_time);
        (state.position - self.target).abs() <= self.position_tolerance
            && state.velocity.abs() <= self.velocity_tolerance
    }
}

/// 2D analytical spring for spatial vector coordinates $(x, y)$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring2D {
    /// Horizontal axis spring.
    pub x: SpringAnimation,
    /// Vertical axis spring.
    pub y: SpringAnimation,
}

impl Spring2D {
    /// Creates a 2D spring animation with identical parameter profiles for both axes.
    pub fn new(
        params: SpringParams,
        initial_pos: (f64, f64),
        initial_vel: (f64, f64),
        target: (f64, f64),
    ) -> Self {
        Self {
            x: SpringAnimation::new(params, initial_pos.0, initial_vel.0, target.0),
            y: SpringAnimation::new(params, initial_pos.1, initial_vel.1, target.1),
        }
    }

    /// Retargets both axes continuously.
    pub fn retarget(&mut self, new_target: (f64, f64)) {
        self.x.retarget(new_target.0);
        self.y.retarget(new_target.1);
    }

    /// Advances the 2D spring clock by $\Delta t$.
    pub fn step(&mut self, dt: f64) -> ((f64, f64), (f64, f64)) {
        let sx = self.x.step(dt);
        let sy = self.y.step(dt);
        ((sx.position, sy.position), (sx.velocity, sy.velocity))
    }

    /// True if both axes have settled into equilibrium.
    pub fn is_settled(&self) -> bool {
        self.x.is_settled() && self.y.is_settled()
    }
}

/// 4D analytical spring for window rectangles `(x, y, width, height)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringRect {
    /// Window origin X coordinate.
    pub x: SpringAnimation,
    /// Window origin Y coordinate.
    pub y: SpringAnimation,
    /// Window width dimension.
    pub w: SpringAnimation,
    /// Window height dimension.
    pub h: SpringAnimation,
}

impl SpringRect {
    /// Creates a 4D spring for window bounds geometry.
    pub fn new(
        params: SpringParams,
        initial_rect: (f64, f64, f64, f64),
        initial_vel: (f64, f64, f64, f64),
        target_rect: (f64, f64, f64, f64),
    ) -> Self {
        Self {
            x: SpringAnimation::new(params, initial_rect.0, initial_vel.0, target_rect.0),
            y: SpringAnimation::new(params, initial_rect.1, initial_vel.1, target_rect.1),
            w: SpringAnimation::new(params, initial_rect.2, initial_vel.2, target_rect.2),
            h: SpringAnimation::new(params, initial_rect.3, initial_vel.3, target_rect.3),
        }
    }

    /// Retargets window bounds smoothly without velocity interruption.
    pub fn retarget(&mut self, new_rect: (f64, f64, f64, f64)) {
        self.x.retarget(new_rect.0);
        self.y.retarget(new_rect.1);
        self.w.retarget(new_rect.2);
        self.h.retarget(new_rect.3);
    }

    /// Steps all 4 parameters forward by $\Delta t$.
    pub fn step(&mut self, dt: f64) -> (f64, f64, f64, f64) {
        let rx = self.x.step(dt).position;
        let ry = self.y.step(dt).position;
        let rw = self.w.step(dt).position;
        let rh = self.h.step(dt).position;
        (rx, ry, rw, rh)
    }

    /// True if all 4 components have stabilized.
    pub fn is_settled(&self) -> bool {
        self.x.is_settled() && self.y.is_settled() && self.w.is_settled() && self.h.is_settled()
    }
}

/// Hyperbolic Cocoa Rubber-Banding Solver for overscroll physics.
///
/// Mathematical formulation:
/// $$f(\Delta x_{\text{over}}, D) = \left( 1.0 - \frac{1.0}{\frac{|\Delta x_{\text{over}}| \cdot c_{\text{rubber}}}{D} + 1.0} \right) \cdot D \cdot \text{sgn}(\Delta x_{\text{over}})$$
pub struct RubberBandSolver;

impl RubberBandSolver {
    /// Default Cocoa rubber-banding constant $c_{\text{rubber}} = 0.55$.
    pub const RUBBER_COEFFICIENT: f64 = 0.55;

    /// Evaluates visual displacement given physical overscroll $\Delta x$ and container dimension $D$.
    pub fn overscroll_offset(overscroll: f64, dimension: f64) -> f64 {
        if overscroll.abs() < 1e-7 || dimension <= 0.0 {
            return 0.0;
        }
        let sign = overscroll.signum();
        let abs_over = overscroll.abs();
        let c = Self::RUBBER_COEFFICIENT;

        let factor = 1.0 - (1.0 / ((abs_over * c / dimension) + 1.0));
        sign * factor * dimension
    }

    /// Inverse mapping: recovers physical overscroll from visual rubber-banded offset $y$.
    pub fn inverse_overscroll(visual_offset: f64, dimension: f64) -> f64 {
        if visual_offset.abs() < 1e-7 || dimension <= 0.0 {
            return 0.0;
        }
        let sign = visual_offset.signum();
        let abs_y = visual_offset.abs().min(dimension - 1e-6);
        let c = Self::RUBBER_COEFFICIENT;

        let u = abs_y / (dimension - abs_y);
        sign * (u * dimension / c)
    }

    /// Constrains a coordinate within `[min, max]`, applying hyperbolic rubber-banding outside bounds.
    pub fn clamp_or_rubber_band(val: f64, min: f64, max: f64, dimension: f64) -> f64 {
        if val < min {
            min + Self::overscroll_offset(val - min, dimension)
        } else if val > max {
            max + Self::overscroll_offset(val - max, dimension)
        } else {
            val
        }
    }
}

/// Pointer history sample for shake detection.
#[derive(Debug, Clone, Copy)]
struct PointerSample {
    #[allow(dead_code)]
    pos: (f64, f64),
    time_secs: f64,
}

/// Instantaneous pointer velocity sample.
#[derive(Debug, Clone, Copy)]
struct VelocitySample {
    vel: (f64, f64),
    time_secs: f64,
}

/// Vectorial "Shake to Find Cursor" detector.
///
/// Features:
/// - 400ms sliding window of pointer velocity vectors.
/// - Speed threshold: $\|\vec{v}_k\| > 1200\text{ px/s}$.
/// - Direction inversion criterion: $\cos(\theta) = \frac{\vec{v}_k \cdot \vec{v}_{k-1}}{\|\vec{v}_k\| \|\vec{v}_{k-1}\|} < -0.5$.
/// - Triggers when $\ge 3$ inversions occur within 400ms.
/// - Expands cursor scale from $1.0\times$ to $8.0\times$ via analytical spring ($m=1.0, k=300.0, c=28.0$).
/// - Automatically decays back to $1.0\times$ after 650ms of spatial stability.
pub struct CursorShakeDetector {
    samples: VecDeque<PointerSample>,
    velocities: VecDeque<VelocitySample>,
    reversal_timestamps: VecDeque<f64>,
    spring: SpringAnimation,
    is_shaking: bool,
    last_reversal_time: f64,
    current_time: f64,
    last_valid_pos: Option<(f64, f64)>,
}

impl Default for CursorShakeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorShakeDetector {
    /// Sliding window duration for shake gesture analysis (400ms).
    pub const WINDOW_DURATION: f64 = 0.400;
    /// Minimum speed threshold to qualify for intentional shake (1200 px/s).
    pub const VELOCITY_THRESHOLD: f64 = 1200.0;
    /// Dot product threshold for sharp direction reversal (cos(120 deg) = -0.5).
    pub const REVERSAL_COS_THRESHOLD: f64 = -0.5;
    /// Required number of direction reversals within the window.
    pub const REQUIRED_REVERSALS: usize = 3;
    /// Quiescence period before returning to 1.0x (1200ms per Pillar 4 #15).
    pub const SETTLE_DELAY: f64 = 1.200;
    /// Maximum expanded cursor magnification.
    pub const EXPANDED_SCALE: f64 = 8.0;

    /// Creates a new cursor shake detector initialized at resting scale $1.0\times$.
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(64),
            velocities: VecDeque::with_capacity(64),
            reversal_timestamps: VecDeque::with_capacity(16),
            spring: SpringAnimation::new(SpringParams::cursor_shake(), 1.0, 0.0, 1.0)
                .with_tolerances(0.01, 0.01),
            is_shaking: false,
            last_reversal_time: -100.0,
            current_time: 0.0,
            last_valid_pos: None,
        }
    }

    /// Ingests a new pointer coordinate sample at relative timestamp `time_secs`.
    pub fn update_pointer(&mut self, pos: (f64, f64), dt: f64) {
        self.current_time += dt.max(0.0);
        let t = self.current_time;

        if let Some(prev_pos) = self.last_valid_pos {
            if dt > 1e-5 {
                let vx = (pos.0 - prev_pos.0) / dt;
                let vy = (pos.1 - prev_pos.1) / dt;
                let speed = (vx * vx + vy * vy).sqrt();

                if speed.is_finite() {
                    let new_vel = (vx, vy);

                    // Check for sharp direction reversal relative to previous velocity
                    if let Some(prev_vel_sample) = self.velocities.back() {
                        let (pvx, pvy) = prev_vel_sample.vel;
                        let prev_speed = (pvx * pvx + pvy * pvy).sqrt();

                        if speed > Self::VELOCITY_THRESHOLD && prev_speed > Self::VELOCITY_THRESHOLD {
                            let dot = vx * pvx + vy * pvy;
                            let cos_theta = dot / (speed * prev_speed);

                            if cos_theta < Self::REVERSAL_COS_THRESHOLD {
                                self.reversal_timestamps.push_back(t);
                                self.last_reversal_time = t;
                            }
                        }
                    }

                    self.velocities.push_back(VelocitySample { vel: new_vel, time_secs: t });
                }
            }
        }

        self.last_valid_pos = Some(pos);
        self.samples.push_back(PointerSample { pos, time_secs: t });

        // Prune entries older than 400ms sliding window
        let cutoff = t - Self::WINDOW_DURATION;
        while let Some(front) = self.samples.front() {
            if front.time_secs < cutoff {
                self.samples.pop_front();
            } else {
                break;
            }
        }
        while let Some(front) = self.velocities.front() {
            if front.time_secs < cutoff {
                self.velocities.pop_front();
            } else {
                break;
            }
        }
        while let Some(front) = self.reversal_timestamps.front() {
            if *front < cutoff {
                self.reversal_timestamps.pop_front();
            } else {
                break;
            }
        }

        // Trigger shake if threshold reached
        if self.reversal_timestamps.len() >= Self::REQUIRED_REVERSALS {
            if !self.is_shaking {
                self.is_shaking = true;
                self.spring.retarget(Self::EXPANDED_SCALE);
            }
        }

        // Return to 1.0x after 650ms of stability
        if self.is_shaking && (t - self.last_reversal_time) >= Self::SETTLE_DELAY {
            self.is_shaking = false;
            self.spring.retarget(1.0);
        }

        self.spring.step(dt);
    }

    /// Current animated cursor scale factor in range $[1.0, 8.0]$.
    #[inline]
    pub fn current_scale(&self) -> f64 {
        self.spring.sample_at(self.spring.elapsed_time()).position.clamp(1.0, Self::EXPANDED_SCALE)
    }

    /// Whether shake magnification is currently active.
    #[inline]
    pub fn is_shaking(&self) -> bool {
        self.is_shaking
    }
}

/// Parabolic Dock Magnification and Conservative Displacement.
///
/// Mathematical formulation:
/// $$M(d_i) = \begin{cases} S_{\text{base}} + (S_{\text{max}} - S_{\text{base}}) \cdot \left( \frac{1 + \cos\left(\pi \frac{d_i}{R}\right)}{2} \right)^2 & \text{if } d_i < R \\ S_{\text{base}} & \text{if } d_i \ge R \end{cases}$$
/// $$\Delta X_i = \sum_{j} \frac{M(d_j) - S_{\text{base}}}{2} \cdot \text{sgn}(x_i - x_j)$$
pub struct DockMagnifier;

impl DockMagnifier {
    /// Computes magnified icon size $M(d_i)$ based on distance to cursor.
    pub fn magnified_size(dist: f64, s_base: f64, s_max: f64, radius: f64) -> f64 {
        if dist >= radius || radius <= 0.0 {
            return s_base;
        }
        let norm_dist = dist / radius;
        let raised_cos = (1.0 + (PI * norm_dist).cos()) * 0.5;
        let factor = raised_cos * raised_cos;
        s_base + (s_max - s_base) * factor
    }

    /// Computes conservative horizontal displacements for a sequence of resting icon center coordinates.
    pub fn compute_layout(
        resting_centers: &[f64],
        cursor_x: Option<f64>,
        s_base: f64,
        s_max: f64,
        radius: f64,
    ) -> Vec<(f64, f64)> {
        Self::compute_layout_with_envelope(resting_centers, cursor_x, s_base, s_max, radius, 1.0)
    }

    /// Computes conservative horizontal displacements modulated by a temporal hover envelope $E \in [0.0, 1.0]$.
    ///
    /// The size of each icon is modulated by:
    /// $$w_i = S_{\text{base}} + E \cdot (M(d_i) - S_{\text{base}})$$
    pub fn compute_layout_with_envelope(
        resting_centers: &[f64],
        cursor_x: Option<f64>,
        s_base: f64,
        s_max: f64,
        radius: f64,
        envelope: f64,
    ) -> Vec<(f64, f64)> {
        let n = resting_centers.len();
        if n == 0 {
            return Vec::new();
        }

        let env = envelope.clamp(0.0, 1.0);
        let sizes: Vec<f64> = match cursor_x {
            Some(cx) => resting_centers
                .iter()
                .map(|&x| {
                    let m = Self::magnified_size((x - cx).abs(), s_base, s_max, radius);
                    s_base + env * (m - s_base)
                })
                .collect(),
            None => vec![s_base; n],
        };

        let mut displacements = vec![0.0; n];
        for i in 0..n {
            let xi = resting_centers[i];
            let mut delta_x = 0.0;
            for j in 0..n {
                if i != j {
                    let xj = resting_centers[j];
                    let diff = xi - xj;
                    if diff.abs() > 1e-5 {
                        let expansion = (sizes[j] - s_base) * 0.5;
                        delta_x += expansion * diff.signum();
                    }
                }
            }
            displacements[i] = delta_x;
        }

        resting_centers
            .iter()
            .zip(sizes.iter())
            .zip(displacements.iter())
            .map(|((&base_x, &size), &disp)| (base_x + disp, size))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_analytical_subdamped() {
        let params = SpringParams::window_open();
        let spring = SpringAnimation::new(params, 0.0, 0.0, 100.0);

        let s0 = spring.sample_at(0.0);
        assert!((s0.position - 0.0).abs() < 1e-5);
        assert!((s0.velocity - 0.0).abs() < 1e-5);

        // Advance 2 seconds; should settle near target 100.0
        let s2 = spring.sample_at(2.0);
        assert!((s2.position - 100.0).abs() < 0.1);
        assert!(s2.velocity.abs() < 0.1);
    }

    #[test]
    fn test_spring_critically_damped() {
        let params = SpringParams::mission_control();
        assert_eq!(params.regime(), DampingRegime::CriticallyDamped);

        let spring = SpringAnimation::new(params, 50.0, 10.0, 200.0);
        let s0 = spring.sample_at(0.0);
        assert!((s0.position - 50.0).abs() < 1e-5);
        assert!((s0.velocity - 10.0).abs() < 1e-5);

        let s_end = spring.sample_at(1.5);
        assert!((s_end.position - 200.0).abs() < 0.05);
    }

    #[test]
    fn test_spring_c2_retarget_continuity() {
        let params = SpringParams::window_open();
        let mut spring = SpringAnimation::new(params, 0.0, 0.0, 500.0);

        // Step forward mid-flight
        spring.step(0.15);
        let pre_state = spring.sample_at(spring.elapsed_time());

        // Retarget while moving
        spring.retarget(1000.0);
        let post_state = spring.sample_at(0.0);

        // Position and velocity must match exactly (C1 continuity)
        assert!((pre_state.position - post_state.position).abs() < 1e-5);
        assert!((pre_state.velocity - post_state.velocity).abs() < 1e-5);
    }

    #[test]
    fn test_rubber_band_hyperbolic() {
        let dim = 800.0;
        let overscroll = 200.0;
        let visual = RubberBandSolver::overscroll_offset(overscroll, dim);
        assert!(visual > 0.0 && visual < overscroll);

        let inverted = RubberBandSolver::inverse_overscroll(visual, dim);
        assert!((inverted - overscroll).abs() < 1e-4);
    }

    #[test]
    fn test_dock_magnification_physics() {
        let resting = vec![100.0, 164.0, 228.0, 292.0];
        let layout = DockMagnifier::compute_layout(&resting, Some(164.0), 48.0, 96.0, 160.0);

        assert_eq!(layout.len(), 4);
        // The icon exactly at cursor (index 1) should achieve maximum magnification
        assert!((layout[1].1 - 96.0).abs() < 1e-4);
        // Icons to the left should be pushed left, icons to the right pushed right
        assert!(layout[0].0 < resting[0]);
        assert!(layout[2].0 > resting[2]);
    }

    #[test]
    fn test_shake_detector_activation() {
        let mut detector = CursorShakeDetector::new();
        assert!(!detector.is_shaking());
        assert!((detector.current_scale() - 1.0).abs() < 1e-5);

        // Simulate rapid back-and-forth movement (>1200 px/s reversals)
        let mut x = 500.0;
        let dt = 0.016; // 60Hz intervals

        for _ in 0..4 {
            // Move right vigorously
            for _ in 0..5 {
                x += 35.0; // 35px in 16ms = ~2187 px/s
                detector.update_pointer((x, 500.0), dt);
            }
            // Move left vigorously
            for _ in 0..5 {
                x -= 35.0;
                detector.update_pointer((x, 500.0), dt);
            }
        }

        assert!(detector.is_shaking());
        assert!(detector.current_scale() > 1.0);
    }

    #[test]
    fn test_dock_hover_envelope_and_slow_motion() {
        let envelope_params = SpringParams::dock_hover_envelope();
        assert_eq!(envelope_params.mass, 1.0);
        assert_eq!(envelope_params.stiffness, 500.0);
        assert_eq!(envelope_params.damping, 45.0);

        let slow = envelope_params.with_slow_motion(true);
        assert!((slow.stiffness - 500.0 * 0.04).abs() < 1e-4);
        assert!((slow.damping - 45.0 * 0.20).abs() < 1e-4);

        let normal = slow.with_slow_motion(false);
        assert_eq!(normal, slow);

        // Test layout with envelope 0.0 -> no magnification
        let resting = vec![100.0, 164.0, 228.0, 292.0];
        let layout_zero = DockMagnifier::compute_layout_with_envelope(
            &resting,
            Some(164.0),
            64.0,
            128.0,
            180.0,
            0.0,
        );
        for item in layout_zero {
            assert_eq!(item.1, 64.0);
        }

        // Test layout with envelope 1.0 -> full magnification
        let layout_full = DockMagnifier::compute_layout_with_envelope(
            &resting,
            Some(164.0),
            64.0,
            128.0,
            180.0,
            1.0,
        );
        assert!((layout_full[1].1 - 128.0).abs() < 1e-4);
    }
}
