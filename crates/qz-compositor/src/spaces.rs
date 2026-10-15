//! # Spaces, Stage Manager, Hot Corners, and Split View Manager
//!
//! Controls multitasking semantics matching macOS Tahoe 26:
//! - **Mission Control & Spaces**: Virtual desktop management with dynamic Split View (50/50 adjustable).
//! - **Stage Manager**: Center active workspace with up to 4 recent groups on the left with 3D tilt (`rotateY(14°)`).
//! - **Hot Corners (Cantos Ativos)**: Triggering actions (Mission Control, Desktop, Quick Note) at the 4 screen corners.
//! - **Magnetic PiP**: Corner magnetism with `SpringParams::stage_manager()`, free movement with `Cmd`, and edge tucking.

use qz_animation::{Spring2D, SpringAnimation, SpringParams};
use glam::Vec2;
use std::collections::VecDeque;

/// Type of workspace Space.
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceKind {
    /// Standard desktop space with wallpaper, desktop stacks, and floating windows.
    Desktop,
    /// Dedicated fullscreen space allocated to a single application.
    Fullscreen {
        /// Window ID occupying the fullscreen space.
        window_id: u32,
    },
    /// Split View sharing the display between two applications side-by-side.
    SplitView {
        /// Left application window ID.
        left_window_id: u32,
        /// Right application window ID.
        right_window_id: u32,
        /// Divider position ratio in range $[0.25, 0.75]$ (default 0.50).
        divider_ratio: f32,
    },
}

/// A virtual desktop space.
#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    /// Unique space identifier.
    pub id: u64,
    /// Human-readable space name (e.g. "Desktop 1", "Safari").
    pub name: String,
    /// Space specialization.
    pub kind: SpaceKind,
    /// Window IDs residing in this space.
    pub window_ids: Vec<u32>,
}

/// Stage Manager lateral group.
#[derive(Debug, Clone, PartialEq)]
pub struct StageManagerGroup {
    /// Unique group identifier.
    pub id: u64,
    /// Window IDs clustered within this lateral stage.
    pub window_ids: Vec<u32>,
    /// Thumbnail 3D rotation angle in radians (14 degrees = ~0.2443 rad).
    pub tilt_angle_rad: f32,
}

impl StageManagerGroup {
    /// macOS Tahoe 26 Stage Manager 3D tilt: 14 degrees.
    pub const TILT_14_DEG_RAD: f32 = 14.0 * (std::f32::consts::PI / 180.0);

    /// Creates a new lateral stage manager group.
    pub fn new(id: u64, window_ids: Vec<u32>) -> Self {
        Self {
            id,
            window_ids,
            tilt_angle_rad: Self::TILT_14_DEG_RAD,
        }
    }
}

/// The 4 screen corners for Hot Corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenCorner {
    /// Top-left corner of the display.
    TopLeft,
    /// Top-right corner of the display.
    TopRight,
    /// Bottom-left corner of the display.
    BottomLeft,
    /// Bottom-right corner of the display.
    BottomRight,
}

/// Actions assignable to Hot Corners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotCornerAction {
    /// No action configured.
    None,
    /// Open Mission Control.
    MissionControl,
    /// Open App Exposé for the focused app.
    AppExpose,
    /// Scatter windows to reveal the Desktop.
    ShowDesktop,
    /// Summon App Library / Launchpad.
    AppLibrary,
    /// Open Notification Center.
    NotificationCenter,
    /// Lock screen session immediately.
    LockScreen,
    /// Quick Note sheet emerging from bottom-right corner.
    QuickNote,
}

/// Magnetic corners for Picture-in-Picture window snapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipCorner {
    /// Snapped to top-left corner.
    TopLeft,
    /// Snapped to top-right corner.
    TopRight,
    /// Snapped to bottom-left corner.
    BottomLeft,
    /// Snapped to bottom-right corner (default).
    BottomRight,
}

/// State of a magnetic Picture-in-Picture floating window.
#[derive(Debug, Clone, PartialEq)]
pub struct PipController {
    /// Current window position in screen space.
    pub position: Spring2D,
    /// Current target corner.
    pub current_corner: PipCorner,
    /// Whether user is currently dragging freely holding Cmd.
    pub is_free_moving_with_cmd: bool,
    /// Whether video frame is tucked off-screen leaving a chevron tab.
    pub is_tucked: bool,
    /// Display viewport resolution.
    pub screen_res: Vec2,
    /// PiP window dimensions.
    pub pip_size: Vec2,
}

impl PipController {
    /// Margin padding from screen edges (16px).
    pub const PADDING: f32 = 16.0;

    /// Creates a PiP controller initialized at the bottom-right corner.
    pub fn new(screen_res: Vec2, pip_size: Vec2) -> Self {
        let target = Self::corner_position(PipCorner::BottomRight, screen_res, pip_size);
        let spring = Spring2D::new(
            SpringParams::stage_manager(),
            (target.x as f64, target.y as f64),
            (0.0, 0.0),
            (target.x as f64, target.y as f64),
        );

        Self {
            position: spring,
            current_corner: PipCorner::BottomRight,
            is_free_moving_with_cmd: false,
            is_tucked: false,
            screen_res,
            pip_size,
        }
    }

    /// Computes resting target coordinates for a corner.
    pub fn corner_position(corner: PipCorner, screen: Vec2, size: Vec2) -> Vec2 {
        let p = Self::PADDING;
        match corner {
            PipCorner::TopLeft => Vec2::new(p, p),
            PipCorner::TopRight => Vec2::new(screen.x - size.x - p, p),
            PipCorner::BottomLeft => Vec2::new(p, screen.y - size.y - p),
            PipCorner::BottomRight => Vec2::new(screen.x - size.x - p, screen.y - size.y - p),
        }
    }

    /// Finds the closest corner to an arbitrary point `pos`.
    pub fn nearest_corner(&self, pos: Vec2) -> PipCorner {
        let mid_x = self.screen_res.x * 0.5;
        let mid_y = self.screen_res.y * 0.5;

        if pos.x < mid_x {
            if pos.y < mid_y { PipCorner::TopLeft } else { PipCorner::BottomLeft }
        } else if pos.y < mid_y {
            PipCorner::TopRight
        } else {
            PipCorner::BottomRight
        }
    }

    /// Snaps PiP to designated corner with stage manager spring physics.
    pub fn snap_to_corner(&mut self, corner: PipCorner) {
        self.current_corner = corner;
        self.is_tucked = false;
        let target = Self::corner_position(corner, self.screen_res, self.pip_size);
        self.position.retarget((target.x as f64, target.y as f64));
    }

    /// Updates drag position, respecting the `Cmd` key for free positioning.
    pub fn handle_drag(&mut self, cursor_pos: Vec2, is_cmd_down: bool) {
        self.is_free_moving_with_cmd = is_cmd_down;
        if is_cmd_down {
            // Free positioning anywhere on the display
            self.position.retarget((cursor_pos.x as f64, cursor_pos.y as f64));
        } else {
            // Magnetically attracted to nearest corner
            let corner = self.nearest_corner(cursor_pos);
            self.snap_to_corner(corner);
        }
    }

    /// Tucks the PiP video body off-screen leaving only an edge tab.
    pub fn tuck_to_edge(&mut self) {
        self.is_tucked = true;
        let mut target = Self::corner_position(self.current_corner, self.screen_res, self.pip_size);
        // Push off screen on horizontal edge leaving 24px tab
        match self.current_corner {
            PipCorner::TopLeft | PipCorner::BottomLeft => {
                target.x = -self.pip_size.x + 24.0;
            }
            PipCorner::TopRight | PipCorner::BottomRight => {
                target.x = self.screen_res.x - 24.0;
            }
        }
        self.position.retarget((target.x as f64, target.y as f64));
    }

    /// Steps the spring animation forward by `dt` seconds.
    pub fn step(&mut self, dt: f64) -> Vec2 {
        let (pos, _) = self.position.step(dt);
        Vec2::new(pos.0 as f32, pos.1 as f32)
    }
}

/// Comprehensive Spaces and Stage Manager subsystem coordinator.
pub struct SpacesCoordinator {
    spaces: Vec<Space>,
    active_space_index: usize,
    stage_groups: VecDeque<StageManagerGroup>,
    is_stage_manager_enabled: bool,
    hot_corners: [HotCornerAction; 4],
    transition_spring: SpringAnimation,
}

impl Default for SpacesCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl SpacesCoordinator {
    /// Creates a coordinator initialized with Desktop 1.
    pub fn new() -> Self {
        let default_space = Space {
            id: 1,
            name: "Desktop 1".to_string(),
            kind: SpaceKind::Desktop,
            window_ids: Vec::new(),
        };

        Self {
            spaces: vec![default_space],
            active_space_index: 0,
            stage_groups: VecDeque::with_capacity(4),
            is_stage_manager_enabled: false,
            hot_corners: [
                HotCornerAction::None,           // TopLeft
                HotCornerAction::None,           // TopRight
                HotCornerAction::None,           // BottomLeft
                HotCornerAction::QuickNote,      // BottomRight (macOS default: Quick Note)
            ],
            transition_spring: SpringAnimation::new(SpringParams::mission_control(), 0.0, 0.0, 0.0),
        }
    }

    /// Returns currently active Space.
    pub fn active_space(&self) -> &Space {
        &self.spaces[self.active_space_index]
    }

    /// Configures action for a Hot Corner.
    pub fn set_hot_corner(&mut self, corner: ScreenCorner, action: HotCornerAction) {
        let index = match corner {
            ScreenCorner::TopLeft => 0,
            ScreenCorner::TopRight => 1,
            ScreenCorner::BottomLeft => 2,
            ScreenCorner::BottomRight => 3,
        };
        self.hot_corners[index] = action;
    }

    /// Evaluates whether cursor at `(px, py)` triggers a Hot Corner.
    pub fn evaluate_hot_corner(&self, px: f64, py: f64, screen_w: f64, screen_h: f64) -> HotCornerAction {
        const THRESHOLD: f64 = 4.0;
        if px <= THRESHOLD && py <= THRESHOLD {
            self.hot_corners[0] // TopLeft
        } else if px >= (screen_w - THRESHOLD) && py <= THRESHOLD {
            self.hot_corners[1] // TopRight
        } else if px <= THRESHOLD && py >= (screen_h - THRESHOLD) {
            self.hot_corners[2] // BottomLeft
        } else if px >= (screen_w - THRESHOLD) && py >= (screen_h - THRESHOLD) {
            self.hot_corners[3] // BottomRight
        } else {
            HotCornerAction::None
        }
    }

    /// Creates a Split View space merging two application windows.
    pub fn create_split_view(&mut self, left_id: u32, right_id: u32) -> u64 {
        let new_id = self.spaces.len() as u64 + 1;
        let split_space = Space {
            id: new_id,
            name: format!("Split View {}", new_id),
            kind: SpaceKind::SplitView {
                left_window_id: left_id,
                right_window_id: right_id,
                divider_ratio: 0.50,
            },
            window_ids: vec![left_id, right_id],
        };
        self.spaces.push(split_space);
        self.active_space_index = self.spaces.len() - 1;
        new_id
    }

    /// Toggles Stage Manager and groups inactive windows to the lateral stage.
    pub fn toggle_stage_manager(&mut self, enabled: bool) {
        self.is_stage_manager_enabled = enabled;
        if !enabled {
            self.stage_groups.clear();
        }
    }

    /// Registers a lateral window group in Stage Manager (capped at 4 recent groups).
    pub fn push_stage_group(&mut self, group: StageManagerGroup) {
        if self.stage_groups.len() >= 4 {
            self.stage_groups.pop_back();
        }
        self.stage_groups.push_front(group);
    }

    /// Returns currently visible Stage Manager lateral groups.
    pub fn stage_groups(&self) -> &VecDeque<StageManagerGroup> {
        &self.stage_groups
    }

    /// Spaces transition spring.
    pub fn transition_spring(&self) -> &SpringAnimation {
        &self.transition_spring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_corner_evaluation() {
        let coordinator = SpacesCoordinator::new();

        // Bottom right corner should trigger default Quick Note
        let action = coordinator.evaluate_hot_corner(1919.0, 1079.0, 1920.0, 1080.0);
        assert_eq!(action, HotCornerAction::QuickNote);

        // Center screen should trigger nothing
        let center = coordinator.evaluate_hot_corner(960.0, 540.0, 1920.0, 1080.0);
        assert_eq!(center, HotCornerAction::None);
    }

    #[test]
    fn test_pip_magnetic_corner_snap() {
        let mut pip = PipController::new(Vec2::new(1920.0, 1080.0), Vec2::new(400.0, 225.0));
        assert_eq!(pip.current_corner, PipCorner::BottomRight);

        // Dragging near top-left without Cmd snaps magnetically to TopLeft
        pip.handle_drag(Vec2::new(100.0, 100.0), false);
        assert_eq!(pip.current_corner, PipCorner::TopLeft);

        // Free positioning with Cmd
        pip.handle_drag(Vec2::new(500.0, 400.0), true);
        assert!(pip.is_free_moving_with_cmd);
    }

    #[test]
    fn test_stage_manager_group_capacity() {
        let mut coordinator = SpacesCoordinator::new();
        coordinator.toggle_stage_manager(true);

        for i in 1..=6 {
            coordinator.push_stage_group(StageManagerGroup::new(i, vec![i as u32]));
        }

        // Capped at 4 recent groups
        assert_eq!(coordinator.stage_groups().len(), 4);
        assert_eq!(coordinator.stage_groups().front().unwrap().id, 6);
    }
}
