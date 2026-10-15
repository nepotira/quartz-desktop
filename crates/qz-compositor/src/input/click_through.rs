//! # Click-Through and Pointer Routing Subsystem
//!
//! Implements macOS Tahoe 26 interaction semantics:
//! - **Cmd-Click-Through**: Holding `Cmd` while clicking an inactive background window delivers
//!   the pointer button event directly to controls (buttons, sliders, traffic lights) without
//!   raising the window or stealing keyboard focus from the active window.
//! - **Unfocused Scroll Dispatch**: Pointer wheel/axis scroll events are delivered to whichever
//!   surface is under the cursor without changing window focus or Z-order.
//! - **Traffic Lights Hit-Testing**: Evaluates hover states across active and inactive windows,
//!   illuminating traffic lights and tracking the 500ms hover delay for the green button tiling popover.

use crate::input::keymapper::ModifierState;

/// Traffic light buttons on the macOS window chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLightButton {
    /// Red button (Close window or Option+Close all).
    Close,
    /// Yellow button (Minimize with Genie effect or Option+Minimize all).
    Minimize,
    /// Green button (Fullscreen, Option+Classic Zoom, or Tiling popover on 500ms hover).
    ZoomOrTile,
}

/// Visual hover state of the traffic lights cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficLightState {
    /// Translucent monochrome circles (inactive window without pointer hover).
    Inactive,
    /// Saturated full colors (active window or hovered inactive window).
    Active,
    /// Hovering inside the cluster: glyphs (x, -, +) revealed.
    Hovered,
}

/// Bounding rectangle in 2D coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Origin X.
    pub x: f64,
    /// Origin Y.
    pub y: f64,
    /// Width dimension.
    pub width: f64,
    /// Height dimension.
    pub height: f64,
}

impl Rect {
    /// Creates a new rectangle.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Checks if a 2D point lies within rectangle bounds.
    #[inline]
    pub fn contains(&self, px: f64, py: f64) -> bool {
        px >= self.x && px <= (self.x + self.width) && py >= self.y && py <= (self.y + self.height)
    }
}

/// Metadata describing a window surface for hit-testing and event routing.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSurfaceInfo {
    /// Unique compositor window ID.
    pub id: u32,
    /// Client application identifier (e.g. "org.quartz.finder").
    pub app_id: String,
    /// Window title text.
    pub title: String,
    /// Bounding rectangle in compositor global desktop coordinates.
    pub bounds: Rect,
    /// True if currently focused and frontmost.
    pub is_active: bool,
    /// True if minimized to Dock.
    pub is_minimized: bool,
    /// Layer Z-index (higher value is in front).
    pub z_index: usize,
    /// Whether document has unsaved edits (draws inner dark dot in close button).
    pub has_document_edited: bool,
}

impl WindowSurfaceInfo {
    /// Global traffic lights bounding box metrics:
    /// Inset from top-left: x = 12.0px, y = 10.0px.
    /// Three buttons of diameter 12.0px with 8.0px spacing:
    /// Total width = 12 + 8 + 12 + 8 + 12 = 52.0px, height = 12.0px.
    /// Expanded hit-target padding of 4px on all sides:
    pub fn traffic_lights_hit_rect(&self) -> Rect {
        Rect::new(self.bounds.x + 8.0, self.bounds.y + 6.0, 60.0, 20.0)
    }

    /// Hit-tests an individual traffic light button from local coordinates.
    pub fn hit_test_traffic_light(&self, local_x: f64, local_y: f64) -> Option<TrafficLightButton> {
        // Red center: (18.0, 16.0), Yellow center: (38.0, 16.0), Green center: (58.0, 16.0), radius = 7.0px
        let cy = 16.0;
        let r_sq = 7.0 * 7.0;

        let dy = local_y - cy;

        // Red (Close)
        let dx_close = local_x - 18.0;
        if (dx_close * dx_close + dy * dy) <= r_sq {
            return Some(TrafficLightButton::Close);
        }

        // Yellow (Minimize)
        let dx_min = local_x - 38.0;
        if (dx_min * dx_min + dy * dy) <= r_sq {
            return Some(TrafficLightButton::Minimize);
        }

        // Green (Zoom / Tile)
        let dx_zoom = local_x - 58.0;
        if (dx_zoom * dx_zoom + dy * dy) <= r_sq {
            return Some(TrafficLightButton::ZoomOrTile);
        }

        None
    }
}

/// Action determined by the pointer button dispatcher.
#[derive(Debug, Clone, PartialEq)]
pub enum ClickThroughAction {
    /// Clicked normal window interior.
    ContentClick {
        /// Local X coordinate within the window frame.
        local_x: f64,
        /// Local Y coordinate within the window frame.
        local_y: f64,
    },
    /// Clicked a traffic light button.
    TrafficLightClick {
        /// Button clicked.
        button: TrafficLightButton,
        /// If document is edited, close button displays dirty state.
        document_edited: bool,
        /// True if Option key was held (e.g. Option+Close all or Option+Classic Zoom).
        option_held: bool,
    },
    /// Clicked empty desktop background wallpaper.
    DesktopBackgroundClick,
}

/// Outcome of dispatching a pointer click event.
#[derive(Debug, Clone, PartialEq)]
pub struct ClickThroughResult {
    /// Window ID that received the click event (or None if desktop background).
    pub target_window_id: Option<u32>,
    /// Whether the compositor should raise this window to the top of the Z-order.
    pub should_raise: bool,
    /// Whether keyboard focus should be transferred to this window.
    pub should_focus: bool,
    /// Specific click semantic action.
    pub action: ClickThroughAction,
}

/// Outcome of dispatching a scroll event.
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollDispatchResult {
    /// Target window ID under cursor.
    pub target_window_id: Option<u32>,
    /// Local X coordinate within target window.
    pub local_x: f64,
    /// Local Y coordinate within target window.
    pub local_y: f64,
    /// Horizontal scroll delta.
    pub delta_x: f64,
    /// Vertical scroll delta.
    pub delta_y: f64,
}

/// Manages pointer click-through, scroll routing, and traffic light hover timers.
pub struct ClickThroughDispatcher {
    hovered_window_id: Option<u32>,
    hovered_traffic_light: Option<TrafficLightButton>,
    green_button_hover_start_time: Option<f64>,
    tiling_popover_triggered: bool,
}

impl Default for ClickThroughDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickThroughDispatcher {
    /// Duration of green button hover required to trigger tiling popover (500ms).
    pub const TILING_POPOVER_HOVER_DELAY: f64 = 0.500;

    /// Creates a new dispatcher.
    pub fn new() -> Self {
        Self {
            hovered_window_id: None,
            hovered_traffic_light: None,
            green_button_hover_start_time: None,
            tiling_popover_triggered: false,
        }
    }

    /// Finds the frontmost visible window containing point `(px, py)`.
    pub fn find_surface_at<'a>(
        surfaces: &'a [WindowSurfaceInfo],
        px: f64,
        py: f64,
    ) -> Option<&'a WindowSurfaceInfo> {
        surfaces
            .iter()
            .filter(|s| !s.is_minimized && s.bounds.contains(px, py))
            .max_by_key(|s| s.z_index)
    }

    /// Dispatches pointer button click according to macOS Tahoe 26 Click-Through rules.
    pub fn dispatch_button_press(
        &mut self,
        pointer_pos: (f64, f64),
        modifiers: ModifierState,
        surfaces: &[WindowSurfaceInfo],
    ) -> ClickThroughResult {
        let (px, py) = pointer_pos;
        let is_cmd = modifiers.contains(ModifierState::COMMAND);
        let is_opt = modifiers.contains(ModifierState::OPTION);

        let target_surface = match Self::find_surface_at(surfaces, px, py) {
            Some(s) => s,
            None => {
                // Clicked on empty desktop wallpaper
                return ClickThroughResult {
                    target_window_id: None,
                    should_raise: false,
                    should_focus: false,
                    action: ClickThroughAction::DesktopBackgroundClick,
                };
            }
        };

        let local_x = px - target_surface.bounds.x;
        let local_y = py - target_surface.bounds.y;

        // Check for traffic lights hit
        let traffic_button = target_surface.hit_test_traffic_light(local_x, local_y);
        let action = match traffic_button {
            Some(button) => ClickThroughAction::TrafficLightClick {
                button,
                document_edited: target_surface.has_document_edited,
                option_held: is_opt,
            },
            None => ClickThroughAction::ContentClick { local_x, local_y },
        };

        if target_surface.is_active {
            // Clicking an already active window preserves focus and Z-order
            ClickThroughResult {
                target_window_id: Some(target_surface.id),
                should_raise: false,
                should_focus: true,
                action,
            }
        } else if is_cmd {
            // macOS Cmd-Click-Through:
            // Background window receives event directly WITHOUT being raised or focused!
            ClickThroughResult {
                target_window_id: Some(target_surface.id),
                should_raise: false,
                should_focus: false,
                action,
            }
        } else {
            // Standard click on background window: brings to front and focuses
            ClickThroughResult {
                target_window_id: Some(target_surface.id),
                should_raise: true,
                should_focus: true,
                action,
            }
        }
    }

    /// Dispatches axis scroll events directly to surface under pointer without focus theft.
    pub fn dispatch_scroll(
        pointer_pos: (f64, f64),
        delta_x: f64,
        delta_y: f64,
        surfaces: &[WindowSurfaceInfo],
    ) -> ScrollDispatchResult {
        let (px, py) = pointer_pos;
        if let Some(target) = Self::find_surface_at(surfaces, px, py) {
            ScrollDispatchResult {
                target_window_id: Some(target.id),
                local_x: px - target.bounds.x,
                local_y: py - target.bounds.y,
                delta_x,
                delta_y,
            }
        } else {
            ScrollDispatchResult {
                target_window_id: None,
                local_x: px,
                local_y: py,
                delta_x,
                delta_y,
            }
        }
    }

    /// Updates hover tracking across surfaces, returning whether the 500ms tiling popover should appear.
    pub fn update_pointer_motion(
        &mut self,
        pointer_pos: (f64, f64),
        current_time_secs: f64,
        surfaces: &[WindowSurfaceInfo],
    ) -> (Option<u32>, TrafficLightState, bool) {
        let (px, py) = pointer_pos;
        let target = Self::find_surface_at(surfaces, px, py);

        match target {
            Some(surface) => {
                let local_x = px - surface.bounds.x;
                let local_y = py - surface.bounds.y;
                let in_cluster = surface.traffic_lights_hit_rect().contains(px, py);
                let hit_btn = surface.hit_test_traffic_light(local_x, local_y);

                self.hovered_window_id = Some(surface.id);
                self.hovered_traffic_light = hit_btn;

                // Handle green button 500ms hover timer
                let mut trigger_tiling = false;
                if hit_btn == Some(TrafficLightButton::ZoomOrTile) {
                    if let Some(start) = self.green_button_hover_start_time {
                        if !self.tiling_popover_triggered
                            && (current_time_secs - start) >= Self::TILING_POPOVER_HOVER_DELAY
                        {
                            self.tiling_popover_triggered = true;
                            trigger_tiling = true;
                        }
                    } else {
                        self.green_button_hover_start_time = Some(current_time_secs);
                    }
                } else {
                    self.green_button_hover_start_time = None;
                    self.tiling_popover_triggered = false;
                }

                let state = if in_cluster {
                    TrafficLightState::Hovered
                } else if surface.is_active {
                    TrafficLightState::Active
                } else {
                    TrafficLightState::Inactive
                };

                (Some(surface.id), state, trigger_tiling)
            }
            None => {
                self.hovered_window_id = None;
                self.hovered_traffic_light = None;
                self.green_button_hover_start_time = None;
                self.tiling_popover_triggered = false;
                (None, TrafficLightState::Inactive, false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_surfaces() -> Vec<WindowSurfaceInfo> {
        vec![
            WindowSurfaceInfo {
                id: 1,
                app_id: "org.quartz.finder".to_string(),
                title: "Documents".to_string(),
                bounds: Rect::new(100.0, 100.0, 600.0, 400.0),
                is_active: false,
                is_minimized: false,
                z_index: 10,
                has_document_edited: false,
            },
            WindowSurfaceInfo {
                id: 2,
                app_id: "org.quartz.terminal".to_string(),
                title: "Terminal".to_string(),
                bounds: Rect::new(300.0, 200.0, 700.0, 500.0),
                is_active: true,
                is_minimized: false,
                z_index: 20,
                has_document_edited: true,
            },
        ]
    }

    #[test]
    fn test_normal_click_on_background_window_raises_and_focuses() {
        let mut dispatcher = ClickThroughDispatcher::new();
        let surfaces = make_test_surfaces();

        // Click at (150, 150) - inside Window 1 (background)
        let res = dispatcher.dispatch_button_press((150.0, 150.0), ModifierState::NONE, &surfaces);

        assert_eq!(res.target_window_id, Some(1));
        assert!(res.should_raise);
        assert!(res.should_focus);
    }

    #[test]
    fn test_cmd_click_through_on_background_window_delivers_without_raising_or_focusing() {
        let mut dispatcher = ClickThroughDispatcher::new();
        let surfaces = make_test_surfaces();

        // Click with Cmd at (150, 150) inside Window 1 (background)
        let res = dispatcher.dispatch_button_press((150.0, 150.0), ModifierState::COMMAND, &surfaces);

        assert_eq!(res.target_window_id, Some(1));
        assert!(!res.should_raise, "Cmd click-through must not raise background window!");
        assert!(!res.should_focus, "Cmd click-through must not focus background window!");
        assert_eq!(
            res.action,
            ClickThroughAction::ContentClick { local_x: 50.0, local_y: 50.0 }
        );
    }

    #[test]
    fn test_unfocused_scroll_routing() {
        let surfaces = make_test_surfaces();

        // Scroll over Window 1 (background) at (150, 150)
        let res = ClickThroughDispatcher::dispatch_scroll((150.0, 150.0), 0.0, -15.0, &surfaces);

        assert_eq!(res.target_window_id, Some(1));
        assert_eq!(res.local_x, 50.0);
        assert_eq!(res.local_y, 50.0);
        assert_eq!(res.delta_y, -15.0);
    }

    #[test]
    fn test_traffic_light_hit_and_dirty_state() {
        let mut dispatcher = ClickThroughDispatcher::new();
        let surfaces = make_test_surfaces();

        // Click Close button of Window 2 (active, edited) at (300 + 18, 200 + 16) = (318, 216)
        let res = dispatcher.dispatch_button_press((318.0, 216.0), ModifierState::NONE, &surfaces);

        assert_eq!(res.target_window_id, Some(2));
        assert_eq!(
            res.action,
            ClickThroughAction::TrafficLightClick {
                button: TrafficLightButton::Close,
                document_edited: true,
                option_held: false,
            }
        );
    }

    #[test]
    fn test_green_button_hover_500ms_popover() {
        let mut dispatcher = ClickThroughDispatcher::new();
        let surfaces = make_test_surfaces();

        // Green button of Window 2 is at (300 + 58, 200 + 16) = (358, 216)
        let (win, state, triggered) = dispatcher.update_pointer_motion((358.0, 216.0), 10.0, &surfaces);
        assert_eq!(win, Some(2));
        assert_eq!(state, TrafficLightState::Hovered);
        assert!(!triggered, "Should not trigger immediately");

        // Advance 200ms
        let (_, _, triggered_200) = dispatcher.update_pointer_motion((358.0, 216.0), 10.200, &surfaces);
        assert!(!triggered_200);

        // Advance past 500ms threshold (550ms total)
        let (_, _, triggered_550) = dispatcher.update_pointer_motion((358.0, 216.0), 10.550, &surfaces);
        assert!(triggered_550, "Should trigger tiling popover after 500ms hover");
    }
}
