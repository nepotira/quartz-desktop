//! # AquaUI Floating Glass Pills & Squircle Widgets (`qz-toolkit::widgets`)
//!
//! Production-grade declarative UI widgets for macOS Tahoe 26.
//!
//! ### Key Capabilities:
//! - **$G^2$ Squircle Geometry:** Mathematically continuous superellipse bounding curves ($n = 4.0$).
//! - **Liquid Glass Physical Material:** Real-time simulation parameters for optical refraction
//!   ($\eta = 1.52$), chromatic dispersion ($\Delta \eta = 0.015$), ambient blur radius ($\sigma = 20.0\text{px}$),
//!   and specular rim highlights.
//! - **Spring-Driven Interactive Micro-interactions:** Buttons depress by $4\%$ on press
//!   ($1.0 \to 0.96$) and bounce back using continuous analytical spring physics.
//! - **Sliding Glass Segmented Control:** Segment selector pill driven by `SpringAnimation`
//!   sliding smoothly beneath segment labels.

use glam::{Vec2, Vec4};
use qz_animation::{SpringAnimation, SpringParams};
use serde::{Deserialize, Serialize};

/// $G^2$ Continuous Superellipse (Squircle) bounding geometry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Squircle {
    /// Left origin $X$.
    pub x: f64,
    /// Top origin $Y$.
    pub y: f64,
    /// Width in logical pixels.
    pub width: f64,
    /// Height in logical pixels.
    pub height: f64,
    /// Corner radius in logical pixels.
    pub corner_radius: f64,
    /// Superellipse curvature exponent ($n = 4.0$ matches Apple Squircle curvature).
    pub curvature_n: f64,
}

impl Squircle {
    /// Creates a new `Squircle` with Apple's standard $n = 4.0$ curvature.
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64, corner_radius: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius: corner_radius.min(width / 2.0).min(height / 2.0),
            curvature_n: 4.0,
        }
    }

    /// Evaluates if a point $(px, py)$ is contained within the continuous squircle boundary.
    ///
    /// Evaluates the superellipse equation in the corner quadrant:
    ///
    /// $$\left( \frac{|x - c_x|}{r} \right)^n + \left( \frac{|y - c_y|}{r} \right)^n \le 1$$
    #[must_use]
    pub fn contains_point(&self, px: f64, py: f64) -> bool {
        if px < self.x || px > self.x + self.width || py < self.y || py > self.y + self.height {
            return false;
        }

        let r = self.corner_radius;
        if r <= 0.0 {
            return true;
        }

        let inner_left = self.x + r;
        let inner_right = self.x + self.width - r;
        let inner_top = self.y + r;
        let inner_bottom = self.y + self.height - r;

        // Inside inner rectangular cross
        if px >= inner_left && px <= inner_right {
            return true;
        }
        if py >= inner_top && py <= inner_bottom {
            return true;
        }

        // Inside corner superellipse
        let cx = if px < inner_left { inner_left } else { inner_right };
        let cy = if py < inner_top { inner_top } else { inner_bottom };

        let dx = (px - cx).abs() / r;
        let dy = (py - cy).abs() / r;

        let n = self.curvature_n;
        (dx.powf(n) + dy.powf(n)) <= 1.0
    }
}

/// Optical physical parameters of the Liquid Glass shader material.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LiquidGlassMaterial {
    /// Base tint color in RGBA (`0.0..=1.0`).
    pub tint_color: Vec4,
    /// Refractive index ($\eta = 1.52$ for crown glass).
    pub refractive_index: f32,
    /// Chromatic dispersion delta ($\Delta \eta = 0.015$).
    pub chromatic_dispersion: f32,
    /// Dual-Kawase background blur radius in pixels ($\sigma = 20.0\text{px}$).
    pub blur_radius_px: f32,
    /// Specular rim highlight brightness ($0.0..=1.0$).
    pub specular_rim: f32,
}

impl Default for LiquidGlassMaterial {
    fn default() -> Self {
        Self {
            tint_color: Vec4::new(1.0, 1.0, 1.0, 0.22),
            refractive_index: 1.52,
            chromatic_dispersion: 0.015,
            blur_radius_px: 20.0,
            specular_rim: 0.65,
        }
    }
}

/// Interaction state of a clickable widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InteractionState {
    /// Resting un-hovered state.
    Normal,
    /// Pointer is hovering over widget surface.
    Hovered,
    /// Mouse button or touch is actively depressing the widget.
    Pressed,
    /// Widget is disabled and ignores interactions.
    Disabled,
}

/// Interactive Floating Glass Pill Button.
#[derive(Debug, Clone)]
pub struct GlassButton {
    /// Squircle layout bounds.
    pub bounds: Squircle,
    /// Text label.
    pub label: String,
    /// Optional SF Pro symbol / icon name.
    pub icon_name: Option<String>,
    /// Material optics.
    pub material: LiquidGlassMaterial,
    /// Current interaction state.
    pub state: InteractionState,
    /// Spring animation driving the 4% press depression scale ($1.0 \to 0.96 \to 1.0$).
    scale_spring: SpringAnimation,
}

impl GlassButton {
    /// Creates a new `GlassButton`.
    #[must_use]
    pub fn new(bounds: Squircle, label: &str) -> Self {
        let params = SpringParams::dock_bounce();
        Self {
            bounds,
            label: label.to_string(),
            icon_name: None,
            material: LiquidGlassMaterial::default(),
            state: InteractionState::Normal,
            scale_spring: SpringAnimation::new(params, 1.0, 0.0, 1.0),
        }
    }

    /// Handles pointer movement.
    pub fn handle_pointer_move(&mut self, px: f64, py: f64) {
        if self.state == InteractionState::Disabled {
            return;
        }

        let inside = self.bounds.contains_point(px, py);
        if inside && self.state == InteractionState::Normal {
            self.state = InteractionState::Hovered;
        } else if !inside && self.state == InteractionState::Hovered {
            self.state = InteractionState::Normal;
        }
    }

    /// Handles mouse button press down.
    pub fn handle_mouse_down(&mut self, px: f64, py: f64) -> bool {
        if self.state == InteractionState::Disabled {
            return false;
        }

        if self.bounds.contains_point(px, py) {
            self.state = InteractionState::Pressed;
            self.scale_spring.retarget(0.96); // Depress by 4%
            true
        } else {
            false
        }
    }

    /// Handles mouse button release.
    ///
    /// Returns `true` if release occurred inside bounds (triggering a Click action).
    pub fn handle_mouse_up(&mut self, px: f64, py: f64) -> bool {
        if self.state == InteractionState::Pressed {
            self.scale_spring.retarget(1.0); // Spring back to 100%
            let inside = self.bounds.contains_point(px, py);
            self.state = if inside {
                InteractionState::Hovered
            } else {
                InteractionState::Normal
            };
            inside
        } else {
            false
        }
    }

    /// Advances the spring animation simulation clock by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        self.scale_spring.step(dt);
    }

    /// Returns current scale factor driven by spring physics.
    #[must_use]
    pub fn current_scale(&self) -> f64 {
        self.scale_spring.sample_at(self.scale_spring.elapsed_time()).position
    }
}

/// Sliding Glass Segmented Control with spring-animated selection pill.
#[derive(Debug, Clone)]
pub struct GlassSegmentedControl {
    /// Bounding squircle container.
    pub bounds: Squircle,
    /// Segment title labels.
    pub segments: Vec<String>,
    /// Currently selected segment index (0-indexed).
    selected_index: usize,
    /// Spring animation for the sliding glass selection pill X coordinate.
    pill_x_spring: SpringAnimation,
}

impl GlassSegmentedControl {
    /// Creates a new `GlassSegmentedControl`.
    #[must_use]
    pub fn new(bounds: Squircle, segments: Vec<String>, initial_index: usize) -> Self {
        let count = segments.len().max(1);
        let segment_w = bounds.width / count as f64;
        let initial_x = bounds.x + (initial_index.min(count - 1) as f64) * segment_w;

        let params = SpringParams::dock_bounce();
        let pill_x_spring = SpringAnimation::new(params, initial_x, 0.0, initial_x);

        Self {
            bounds,
            segments,
            selected_index: initial_index.min(count - 1),
            pill_x_spring,
        }
    }

    /// Selects segment at specified index and springs the selection pill to its position.
    pub fn select_segment(&mut self, index: usize) {
        if index < self.segments.len() {
            self.selected_index = index;
            let segment_w = self.bounds.width / self.segments.len() as f64;
            let target_x = self.bounds.x + (index as f64) * segment_w;
            self.pill_x_spring.retarget(target_x);
        }
    }

    /// Handles pointer click inside the control.
    pub fn handle_click(&mut self, px: f64, py: f64) -> Option<usize> {
        if !self.bounds.contains_point(px, py) {
            return None;
        }

        let rel_x = (px - self.bounds.x).max(0.0);
        let segment_w = self.bounds.width / self.segments.len() as f64;
        let index = ((rel_x / segment_w).floor() as usize).min(self.segments.len() - 1);
        self.select_segment(index);
        Some(index)
    }

    /// Advances spring simulation by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        self.pill_x_spring.step(dt);
    }

    /// Current visual X position of the sliding selection pill.
    #[must_use]
    pub fn pill_x(&self) -> f64 {
        self.pill_x_spring.sample_at(self.pill_x_spring.elapsed_time()).position
    }

    /// Width of individual segment pill.
    #[must_use]
    pub fn segment_width(&self) -> f64 {
        self.bounds.width / self.segments.len().max(1) as f64
    }

    /// Active selected index.
    #[must_use]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }
}

/// Glass Toggle Switch (iOS/macOS style).
#[derive(Debug, Clone)]
pub struct GlassToggle {
    /// Bounding rectangle $[x, y, w, h]$ (standard: $51 \times 31\text{px}$).
    pub position: Vec2,
    /// Dimensions.
    pub size: Vec2,
    /// Active toggle state (`true` = on, `false` = off).
    is_on: bool,
    /// Spring animation for thumb circle position.
    thumb_spring: SpringAnimation,
}

impl Default for GlassToggle {
    fn default() -> Self {
        Self::new(Vec2::ZERO, false)
    }
}

impl GlassToggle {
    /// Creates a new `GlassToggle`.
    #[must_use]
    pub fn new(position: Vec2, initial_state: bool) -> Self {
        let size = Vec2::new(51.0, 31.0);
        let thumb_off_x = position.x as f64 + 2.0;
        let thumb_on_x = position.x as f64 + size.x as f64 - 29.0;
        let target_x = if initial_state { thumb_on_x } else { thumb_off_x };

        let params = SpringParams::dock_bounce();
        let thumb_spring = SpringAnimation::new(params, target_x, 0.0, target_x);

        Self {
            position,
            size,
            is_on: initial_state,
            thumb_spring,
        }
    }

    /// Toggles the switch state.
    pub fn toggle(&mut self) -> bool {
        self.is_on = !self.is_on;
        let thumb_off_x = self.position.x as f64 + 2.0;
        let thumb_on_x = self.position.x as f64 + self.size.x as f64 - 29.0;
        let target_x = if self.is_on { thumb_on_x } else { thumb_off_x };
        self.thumb_spring.retarget(target_x);
        self.is_on
    }

    /// Advances spring simulation by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        self.thumb_spring.step(dt);
    }

    /// Current visual X position of toggle thumb circle.
    #[must_use]
    pub fn thumb_x(&self) -> f64 {
        self.thumb_spring.sample_at(self.thumb_spring.elapsed_time()).position
    }

    /// Returns `true` if switch is in ON position.
    #[must_use]
    pub fn is_on(&self) -> bool {
        self.is_on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squircle_point_containment() {
        let squircle = Squircle::new(100.0, 100.0, 200.0, 100.0, 20.0);

        // Center point is inside
        assert!(squircle.contains_point(200.0, 150.0));
        // Outside bounds
        assert!(!squircle.contains_point(50.0, 50.0));
        assert!(!squircle.contains_point(350.0, 150.0));

        // Sharp corner outside superellipse curvature
        assert!(!squircle.contains_point(101.0, 101.0));
    }

    #[test]
    fn test_glass_button_press_and_spring() {
        let squircle = Squircle::new(0.0, 0.0, 120.0, 44.0, 12.0);
        let mut btn = GlassButton::new(squircle, "Continuar");

        assert_eq!(btn.current_scale(), 1.0);
        assert_eq!(btn.state, InteractionState::Normal);

        // Hover
        btn.handle_pointer_move(60.0, 22.0);
        assert_eq!(btn.state, InteractionState::Hovered);

        // Press down -> depresses to 0.96 target
        assert!(btn.handle_mouse_down(60.0, 22.0));
        assert_eq!(btn.state, InteractionState::Pressed);

        btn.tick(0.05);
        assert!(btn.current_scale() < 1.0);

        // Release inside -> triggers click
        assert!(btn.handle_mouse_up(60.0, 22.0));
        assert_eq!(btn.state, InteractionState::Hovered);

        // Advance to equilibrium -> returns to 1.0
        for _ in 0..50 {
            btn.tick(0.02);
        }
        assert!((btn.current_scale() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_segmented_control_selection_and_spring() {
        let squircle = Squircle::new(0.0, 0.0, 300.0, 36.0, 10.0);
        let mut ctrl = GlassSegmentedControl::new(
            squircle,
            vec!["Dia".to_string(), "Semana".to_string(), "Mês".to_string()],
            0,
        );

        assert_eq!(ctrl.selected_index(), 0);
        assert_eq!(ctrl.segment_width(), 100.0);
        assert_eq!(ctrl.pill_x(), 0.0);

        // Click on 2nd segment ("Semana", x in [100..200])
        let clicked = ctrl.handle_click(150.0, 18.0);
        assert_eq!(clicked, Some(1));
        assert_eq!(ctrl.selected_index(), 1);

        // Tick spring toward 100.0
        for _ in 0..100 {
            ctrl.tick(0.02);
        }
        assert!((ctrl.pill_x() - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_glass_toggle_state() {
        let mut toggle = GlassToggle::new(Vec2::new(10.0, 10.0), false);
        assert!(!toggle.is_on());
        let off_x = toggle.thumb_x();

        // Toggle on
        assert!(toggle.toggle());
        assert!(toggle.is_on());

        for _ in 0..50 {
            toggle.tick(0.02);
        }
        let on_x = toggle.thumb_x();
        assert!(on_x > off_x);
    }
}
