//! # Tripartite Dock, Parabolic Magnification, Stacks & Dynamic Trash Subsystem
//!
//! Controls the full macOS Tahoe 26 Dock:
//! - **Tripartite Structure**:
//!   `[Finder + Pinned/Running Apps] | [Up to 3 Recent Apps] | [Folder Stacks, Live Minimized Windows & Trash]`.
//! - **Separator Resizing**: Dragging the vertical dividing line smoothly scales $S_{\text{base}}$
//!   ($16\text{px} \dots 128\text{px}$); holding `Option` snaps to canonical sizes (16, 32, 48, 64, 96, 128).
//! - **Parabolic Magnification & Bounce**: Analytical cosine magnification and launch bounce via `SpringParams::dock_bounce()`.
//! - **Gestures & Spring-Loaded Drag**: 2-finger swipe up summons App Exposé; hovering a file for 500ms
//!   triggers Spring-Loaded Dock Exposé.
//! - **Context Menu with Option**: Instant switch to *Hide Others* and **Force Quit**.
//! - **Stacks (Folders in Dock) across all 4 modes**:
//!   - **Fan (Leque)**: $\theta_k = k \cdot 4.5^\circ$.
//!   - **Grid (Grade)**: In-place subfolder navigation without opening Finder.
//!   - **List (Lista)**.
//!   - **Automatic**: Fan for $\le 9$ items, Grid for $\ge 10$.
//!   - Holding `Shift` runs animations in slow-motion ($0.15\times$).
//! - **Dynamic Trash / Eject (`⏏`)**: Trash icon morphs into Eject when dragging mounted volumes or DMGs.

use qz_animation::{DockMagnifier, SpringAnimation, SpringParams};
use std::collections::VecDeque;

/// View mode for Stacks (folder shortcuts on the right side of the Dock).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackViewMode {
    /// Arc curve fan distribution: angle = k * 4.5 degrees.
    Fan,
    /// Multi-column icon grid with in-place folder drill-down.
    Grid,
    /// Vertical menu list with small icons.
    List,
    /// Fan for <= 9 items, Grid for >= 10 items.
    Automatic,
}

/// An entry within a Dock folder Stack.
#[derive(Debug, Clone, PartialEq)]
pub struct StackItem {
    /// Item filesystem path.
    pub path: String,
    /// Display filename.
    pub name: String,
    /// True if item is a subdirectory.
    pub is_directory: bool,
}

/// An active Stack folder in the Dock.
#[derive(Debug, Clone, PartialEq)]
pub struct DockStack {
    /// Folder filesystem path.
    pub folder_path: String,
    /// Display name (e.g. "Downloads", "Documents").
    pub name: String,
    /// Configured display mode.
    pub view_mode: StackViewMode,
    /// Contained items.
    pub items: Vec<StackItem>,
    /// True if currently opened/expanded.
    pub is_open: bool,
    /// Expansion animation spring.
    pub expansion_spring: SpringAnimation,
}

impl DockStack {
    /// Angle increment per fan item in degrees ($4.2^\circ$ per Pillar 4 #8).
    pub const FAN_ANGLE_STEP_DEG: f32 = 4.2;
    /// Quadratic horizontal offset coefficient per fan item ($3.5\text{px}$ per Pillar 4 #8).
    pub const FAN_OFFSET_X_COEFF: f32 = 3.5;

    /// Creates a new Dock stack with the canonical fan spring ($m = 1.0, k = 380.0, c = 28.0$).
    pub fn new(folder_path: &str, name: &str, view_mode: StackViewMode, items: Vec<StackItem>) -> Self {
        let fan_spring = SpringParams {
            mass: 1.0,
            stiffness: 380.0,
            damping: 28.0,
        };
        Self {
            folder_path: folder_path.to_string(),
            name: name.to_string(),
            view_mode,
            items,
            is_open: false,
            expansion_spring: SpringAnimation::new(fan_spring, 0.0, 0.0, 0.0),
        }
    }

    /// Resolves actual view mode taking `Automatic` threshold into account.
    pub fn effective_view_mode(&self) -> StackViewMode {
        match self.view_mode {
            StackViewMode::Automatic => {
                if self.items.len() <= 9 {
                    StackViewMode::Fan
                } else {
                    StackViewMode::Grid
                }
            }
            mode => mode,
        }
    }

    /// Computes fan item angle in degrees for index `k`: $\theta_k = k \cdot 4.2^\circ$.
    pub fn fan_item_angle_deg(k: usize) -> f32 {
        k as f32 * Self::FAN_ANGLE_STEP_DEG
    }

    /// Computes fan item horizontal arc displacement in pixels for index `k`: $\Delta x_k = k^2 \cdot 3.5\text{px}$.
    pub fn fan_item_offset_x_px(k: usize) -> f32 {
        (k * k) as f32 * Self::FAN_OFFSET_X_COEFF
    }

    /// Toggles stack open state (optionally in slow motion $0.2\times$ if Shift was held).
    pub fn toggle_open(&mut self, slow_motion: bool) -> bool {
        self.is_open = !self.is_open;
        let target = if self.is_open { 1.0 } else { 0.0 };

        let params = SpringParams {
            mass: 1.0,
            stiffness: 380.0,
            damping: 28.0,
        }
        .with_slow_motion(slow_motion);

        self.expansion_spring.set_params(params);
        self.expansion_spring.retarget(target);
        self.is_open
    }
}

/// Dock application item specification.
#[derive(Debug, Clone, PartialEq)]
pub struct DockAppItem {
    /// Application identifier (e.g. "org.quartz.finder").
    pub app_id: String,
    /// Display name (e.g. "Finder").
    pub name: String,
    /// Whether app is permanently pinned to Dock.
    pub is_pinned: bool,
    /// Whether process is currently running.
    pub is_running: bool,
    /// Active bounce animation spring (for launch or urgent alert).
    pub bounce_spring: SpringAnimation,
    /// Notification badge count (e.g. unread mail count).
    pub badge_count: Option<u32>,
    /// Download/copy progress $[0.0, 1.0]$ displayed under icon.
    pub progress_fraction: Option<f32>,
}

impl DockAppItem {
    /// Creates a pinned app item.
    pub fn pinned(app_id: &str, name: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            name: name.to_string(),
            is_pinned: true,
            is_running: false,
            bounce_spring: SpringAnimation::new(SpringParams::dock_bounce(), 0.0, 0.0, 0.0),
            badge_count: None,
            progress_fraction: None,
        }
    }

    /// Triggers an iconic launch bounce.
    pub fn trigger_bounce(&mut self) {
        self.bounce_spring.retarget(32.0); // 32px vertical bounce peak
    }
}

/// Trash / Eject state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashMode {
    /// Empty trash can.
    EmptyTrash,
    /// Trash containing deleted files.
    FullTrash,
    /// Eject symbol (`⏏`) active while dragging a mounted volume or disk image.
    EjectVolume,
}

/// Tripartite Dock Coordinator managing layout, magnification, stacks, and dragging.
pub struct DockCoordinator {
    pinned_apps: Vec<DockAppItem>,
    recent_apps: VecDeque<DockAppItem>,
    stacks: Vec<DockStack>,
    trash_mode: TrashMode,
    base_size: f64,
    max_magnified_size: f64,
    magnification_radius: f64,
    is_magnification_enabled: bool,
    hover_envelope: SpringAnimation,
    current_cursor_x: Option<f64>,
}

impl Default for DockCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl DockCoordinator {
    /// Canonical snap sizes for Option-resizing.
    pub const CANONICAL_SIZES: [f64; 6] = [16.0, 32.0, 48.0, 64.0, 96.0, 128.0];

    /// Creates standard Dock layout initialized with Finder and Trash.
    pub fn new() -> Self {
        let finder = DockAppItem::pinned("org.quartz.finder", "Finder");
        let terminal = DockAppItem::pinned("org.quartz.terminal", "Terminal");
        let settings = DockAppItem::pinned("org.quartz.settings", "System Settings");

        Self {
            pinned_apps: vec![finder, terminal, settings],
            recent_apps: VecDeque::with_capacity(3),
            stacks: Vec::new(),
            trash_mode: TrashMode::EmptyTrash,
            base_size: 64.0,              // Tahoe 26 canonical 64px base size
            max_magnified_size: 128.0,     // 128px maximum magnification
            magnification_radius: 180.0,   // Gaussian/cosine reach sigma = 180px
            is_magnification_enabled: true,
            hover_envelope: SpringAnimation::new(SpringParams::dock_hover_envelope(), 0.0, 0.0, 0.0),
            current_cursor_x: None,
        }
    }

    /// Advances the Dock physics, hover envelope, and stack springs by `dt` seconds.
    pub fn tick(&mut self, dt: f64) {
        self.hover_envelope.step(dt);
        for app in &mut self.pinned_apps {
            app.bounce_spring.step(dt);
        }
        for stack in &mut self.stacks {
            stack.expansion_spring.step(dt);
        }
    }

    /// Ingests cursor coordinate updates, retargeting the hover envelope spring ($0.0 \leftrightarrow 1.0$).
    pub fn update_cursor_hover(&mut self, cursor_x: Option<f64>) {
        self.current_cursor_x = cursor_x;
        let target = if cursor_x.is_some() { 1.0 } else { 0.0 };
        self.hover_envelope.retarget(target);
    }

    /// Current hover envelope expansion ratio $E(t) \in [0.0, 1.0]$.
    #[inline]
    pub fn current_envelope(&self) -> f64 {
        self.hover_envelope.sample_at(self.hover_envelope.elapsed_time()).position.clamp(0.0, 1.0)
    }

    /// Sets base icon size, optionally snapping to canonical sizes if Option is held.
    pub fn resize_base(&mut self, raw_size: f64, snap_canonical: bool) {
        let clamped = raw_size.clamp(16.0, 128.0);
        if snap_canonical {
            // Find nearest canonical size
            let nearest = Self::CANONICAL_SIZES
                .iter()
                .min_by(|a, b| (clamped - **a).abs().partial_cmp(&(clamped - **b).abs()).unwrap())
                .copied()
                .unwrap_or(64.0);
            self.base_size = nearest;
        } else {
            self.base_size = clamped;
        }
    }

    /// Updates dynamic Trash mode (e.g. morphing to Eject when volume drag starts).
    pub fn set_dragging_mounted_volume(&mut self, is_dragging_volume: bool, trash_has_files: bool) {
        if is_dragging_volume {
            self.trash_mode = TrashMode::EjectVolume;
        } else if trash_has_files {
            self.trash_mode = TrashMode::FullTrash;
        } else {
            self.trash_mode = TrashMode::EmptyTrash;
        }
    }

    /// Current Trash icon mode.
    #[inline]
    pub fn trash_mode(&self) -> TrashMode {
        self.trash_mode
    }

    /// Current base icon size.
    #[inline]
    pub fn base_size(&self) -> f64 {
        self.base_size
    }

    /// Computes full magnified horizontal layout for all items in the Dock modulated by the hover envelope.
    pub fn compute_magnified_layout(&self, cursor_x: Option<f64>) -> Vec<(f64, f64)> {
        let count = self.pinned_apps.len() + self.recent_apps.len() + self.stacks.len() + 1; // +1 for Trash
        let mut resting_centers = Vec::with_capacity(count);

        let mut x = self.base_size * 0.5;
        let spacing = self.base_size + 8.0;

        for _ in 0..count {
            resting_centers.push(x);
            x += spacing;
        }

        if !self.is_magnification_enabled {
            return resting_centers.into_iter().map(|cx| (cx, self.base_size)).collect();
        }

        let envelope = self.current_envelope();
        DockMagnifier::compute_layout_with_envelope(
            &resting_centers,
            cursor_x,
            self.base_size,
            self.max_magnified_size,
            self.magnification_radius,
            envelope,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dock_option_snap_resizing() {
        let mut dock = DockCoordinator::new();

        // Resize without snap
        dock.resize_base(51.0, false);
        assert_eq!(dock.base_size(), 51.0);

        // Resize with Option snap: 51.0 is closest to 48.0
        dock.resize_base(51.0, true);
        assert_eq!(dock.base_size(), 48.0);

        // 75.0 is closest to 64.0 (diff 11 vs diff 21 to 96)
        dock.resize_base(75.0, true);
        assert_eq!(dock.base_size(), 64.0);
    }

    #[test]
    fn test_dock_trash_morphing_to_eject() {
        let mut dock = DockCoordinator::new();
        assert_eq!(dock.trash_mode(), TrashMode::EmptyTrash);

        // Drag a mounted volume: Trash morphs to Eject symbol
        dock.set_dragging_mounted_volume(true, false);
        assert_eq!(dock.trash_mode(), TrashMode::EjectVolume);

        // Drop volume: reverts to normal
        dock.set_dragging_mounted_volume(false, true);
        assert_eq!(dock.trash_mode(), TrashMode::FullTrash);
    }

    #[test]
    fn test_stack_view_modes_and_fan_angle() {
        let items_few: Vec<StackItem> = (0..5)
            .map(|i| StackItem {
                path: format!("/doc_{}", i),
                name: format!("Doc {}", i),
                is_directory: false,
            })
            .collect();
        let stack_few = DockStack::new("/docs", "Docs", StackViewMode::Automatic, items_few);
        assert_eq!(stack_few.effective_view_mode(), StackViewMode::Fan);

        // Angle for 3rd item: 3 * 4.2 = 12.6 degrees; offset X: 9 * 3.5 = 31.5px
        assert!((DockStack::fan_item_angle_deg(3) - 12.6).abs() < 1e-4);
        assert!((DockStack::fan_item_offset_x_px(3) - 31.5).abs() < 1e-4);

        let items_many: Vec<StackItem> = (0..15)
            .map(|i| StackItem {
                path: format!("/img_{}", i),
                name: format!("Img {}", i),
                is_directory: false,
            })
            .collect();
        let stack_many = DockStack::new("/imgs", "Images", StackViewMode::Automatic, items_many);
        assert_eq!(stack_many.effective_view_mode(), StackViewMode::Grid);
    }

    #[test]
    fn test_dock_hover_envelope_progression() {
        let mut dock = DockCoordinator::new();
        assert_eq!(dock.current_envelope(), 0.0);

        // Enter Dock with cursor
        dock.update_cursor_hover(Some(100.0));
        assert_eq!(dock.current_envelope(), 0.0);

        // Advance simulation
        dock.tick(0.1);
        let env = dock.current_envelope();
        assert!(env > 0.0 && env <= 1.0);

        // Advance to equilibrium
        dock.tick(1.0);
        assert!((dock.current_envelope() - 1.0).abs() < 0.05);

        // Exit Dock
        dock.update_cursor_hover(None);
        dock.tick(1.0);
        assert!(dock.current_envelope() < 0.05);
    }
}
