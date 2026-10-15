//! # Native Tab Bar Subsystem (`qz-toolkit::tabs`)
//!
//! Production-grade implementation of macOS Tahoe 26 Native Tab Bar.
//!
//! ### Key Capabilities:
//! - **Flocking Drag-to-Reorder Physics:** When dragging a tab, neighboring tabs dynamically
//!   slide out of the way via spring animation (`SpringParams::dock_bounce`) to preview insertion.
//! - **Canonical Dirty State Close Glyph:** If a tab has unsaved modifications (`is_dirty = true`),
//!   the close button morphs into a centered bullet dot (`•`), dynamically switching to an `x`
//!   when hovered (AppKit canonical behavior).
//! - **Adaptive Width Clamping:** Distributes tab widths evenly between $W_{\text{min}} = 100\text{px}$
//!   and $W_{\text{max}} = 240\text{px}$, accommodating a persistent glass `+` button.
//! - **Active Sliding Glass Pill:** Background indicator smoothly glides to active tab position
//!   using continuous analytical spring physics.

use qz_animation::{SpringAnimation, SpringParams};
use serde::{Deserialize, Serialize};

/// Individual tab descriptor.
#[derive(Debug, Clone)]
pub struct TabItem {
    /// Unique tab identifier.
    pub id: u64,
    /// Human-readable title label.
    pub title: String,
    /// Optional icon glyph name.
    pub icon_name: Option<String>,
    /// Unsaved modifications dirty state (shows bullet `•` instead of `x`).
    pub is_dirty: bool,
    /// Whether this tab can be closed by the user.
    pub can_close: bool,
    /// Spring animation for the tab's horizontal visual resting position.
    spring_x: SpringAnimation,
    /// Target horizontal position.
    target_x: f64,
}

impl TabItem {
    /// Creates a new `TabItem`.
    #[must_use]
    pub fn new(id: u64, title: &str) -> Self {
        let params = SpringParams::dock_bounce();
        Self {
            id,
            title: title.to_string(),
            icon_name: None,
            is_dirty: false,
            can_close: true,
            spring_x: SpringAnimation::new(params, 0.0, 0.0, 0.0),
            target_x: 0.0,
        }
    }

    /// Retargets the tab's horizontal spring position.
    pub fn retarget_x(&mut self, new_x: f64) {
        self.target_x = new_x;
        self.spring_x.retarget(new_x);
    }

    /// Advances the tab's spring clock by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        self.spring_x.step(dt);
    }

    /// Current visual X position.
    #[must_use]
    pub fn current_x(&self) -> f64 {
        self.spring_x.sample_at(self.spring_x.elapsed_time()).position
    }
}

/// Operational state of interactive tab dragging.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TabDragState {
    /// ID of tab being dragged.
    pub tab_id: u64,
    /// Original array index prior to drag.
    pub original_index: usize,
    /// Horizontal offset between pointer and tab origin.
    pub grab_offset_x: f64,
    /// Real-time pointer X coordinate.
    pub current_pointer_x: f64,
}

/// Native Tab Bar Component.
#[derive(Debug, Clone)]
pub struct NativeTabBar {
    /// Total bar width in logical pixels.
    pub width: f64,
    /// Bar height ($38.0\text{px}$ standard).
    pub height: f64,
    /// List of tabs currently open.
    tabs: Vec<TabItem>,
    /// Active selected tab ID.
    active_tab_id: Option<u64>,
    /// Spring driving the active sliding glass pill indicator.
    active_pill_spring: SpringAnimation,
    /// Active interactive dragging state if reordering.
    drag_state: Option<TabDragState>,
    /// Minimum tab width ($100\text{px}$).
    pub min_tab_width: f64,
    /// Maximum tab width ($240\text{px}$).
    pub max_tab_width: f64,
    /// Width of the `+` new tab button ($36\text{px}$).
    pub new_tab_button_width: f64,
}

impl Default for NativeTabBar {
    fn default() -> Self {
        Self::new(960.0)
    }
}

impl NativeTabBar {
    /// Creates a new `NativeTabBar` with the specified bar width.
    #[must_use]
    pub fn new(width: f64) -> Self {
        let params = SpringParams::dock_bounce();
        Self {
            width,
            height: 38.0,
            tabs: Vec::new(),
            active_tab_id: None,
            active_pill_spring: SpringAnimation::new(params, 0.0, 0.0, 0.0),
            drag_state: None,
            min_tab_width: 100.0,
            max_tab_width: 240.0,
            new_tab_button_width: 36.0,
        }
    }

    /// Adds a new tab and makes it active.
    pub fn add_tab(&mut self, id: u64, title: &str) {
        let tab = TabItem::new(id, title);
        self.tabs.push(tab);
        self.active_tab_id = Some(id);
        self.relayout_tabs();

        // Snap active pill spring to new tab
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            let tab_w = self.tab_width();
            let target_x = (idx as f64) * tab_w;
            self.active_pill_spring.retarget(target_x);
        }
    }

    /// Closes a tab by ID.
    pub fn close_tab(&mut self, id: u64) -> Option<u64> {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(idx);

            // Re-select adjacent tab if closed was active
            if self.active_tab_id == Some(id) {
                if !self.tabs.is_empty() {
                    let new_idx = idx.min(self.tabs.len() - 1);
                    self.active_tab_id = Some(self.tabs[new_idx].id);
                } else {
                    self.active_tab_id = None;
                }
            }

            self.relayout_tabs();
            self.active_tab_id
        } else {
            None
        }
    }

    /// Selects active tab by ID.
    pub fn select_tab(&mut self, id: u64) -> bool {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == id) {
            self.active_tab_id = Some(id);
            let tab_w = self.tab_width();
            let target_x = (idx as f64) * tab_w;
            self.active_pill_spring.retarget(target_x);
            true
        } else {
            false
        }
    }

    /// Selects the next tab (`Ctrl+Tab`).
    pub fn select_next_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        if let Some(curr) = self.active_tab_id {
            if let Some(idx) = self.tabs.iter().position(|t| t.id == curr) {
                let next_idx = (idx + 1) % self.tabs.len();
                self.select_tab(self.tabs[next_idx].id);
            }
        }
    }

    /// Selects the previous tab (`Ctrl+Shift+Tab`).
    pub fn select_previous_tab(&mut self) {
        if self.tabs.is_empty() {
            return;
        }

        if let Some(curr) = self.active_tab_id {
            if let Some(idx) = self.tabs.iter().position(|t| t.id == curr) {
                let prev_idx = if idx == 0 { self.tabs.len() - 1 } else { idx - 1 };
                self.select_tab(self.tabs[prev_idx].id);
            }
        }
    }

    /// Calculates the dynamic clamped width of each tab.
    #[must_use]
    pub fn tab_width(&self) -> f64 {
        if self.tabs.is_empty() {
            return self.max_tab_width;
        }

        let avail_w = (self.width - self.new_tab_button_width).max(100.0);
        let equal_w = avail_w / self.tabs.len() as f64;
        equal_w.clamp(self.min_tab_width, self.max_tab_width)
    }

    /// Re-evaluates resting target X coordinates for all tabs.
    fn relayout_tabs(&mut self) {
        let tab_w = self.tab_width();
        for (i, tab) in self.tabs.iter_mut().enumerate() {
            let target = (i as f64) * tab_w;
            tab.retarget_x(target);
        }

        if let Some(active_id) = self.active_tab_id {
            if let Some(idx) = self.tabs.iter().position(|t| t.id == active_id) {
                let target = (idx as f64) * tab_w;
                self.active_pill_spring.retarget(target);
            }
        }
    }

    /// Begins interactive tab reordering drag.
    pub fn begin_tab_drag(&mut self, tab_id: u64, pointer_x: f64) {
        if let Some(idx) = self.tabs.iter().position(|t| t.id == tab_id) {
            let tab_w = self.tab_width();
            let tab_x = (idx as f64) * tab_w;
            let grab_offset = pointer_x - tab_x;

            self.drag_state = Some(TabDragState {
                tab_id,
                original_index: idx,
                grab_offset_x: grab_offset,
                current_pointer_x: pointer_x,
            });
            self.select_tab(tab_id);
        }
    }

    /// Updates interactive dragging and executes flocking reordering of other tabs.
    pub fn update_tab_drag(&mut self, pointer_x: f64) {
        let tab_w = self.tab_width();
        let (drag_tab_id, grab_offset_x) = match self.drag_state {
            Some(ref mut drag) => {
                drag.current_pointer_x = pointer_x;
                (drag.tab_id, drag.grab_offset_x)
            }
            None => return,
        };

        let dragged_x = (pointer_x - grab_offset_x).max(0.0);

        // Compute current index insertion slot based on midpoint
        let current_idx = self.tabs.iter().position(|t| t.id == drag_tab_id).unwrap_or(0);
        let target_slot = ((dragged_x + tab_w / 2.0) / tab_w).floor() as usize;
        let clamped_slot = target_slot.min(self.tabs.len() - 1);

        if clamped_slot != current_idx {
            // Swap position in array
            let item = self.tabs.remove(current_idx);
            self.tabs.insert(clamped_slot, item);
            // Retarget all tabs to their new slots
            self.relayout_tabs();
        }
    }

    /// Concludes interactive tab dragging.
    pub fn end_tab_drag(&mut self) {
        self.drag_state = None;
        self.relayout_tabs();
    }

    /// Sets dirty state on a tab.
    pub fn set_tab_dirty(&mut self, id: u64, is_dirty: bool) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.is_dirty = is_dirty;
        }
    }

    /// Advances spring animations across all tabs by $\Delta t$ seconds.
    pub fn tick(&mut self, dt: f64) {
        for tab in &mut self.tabs {
            tab.tick(dt);
        }
        self.active_pill_spring.step(dt);
    }

    /// Current visual X position of the active sliding pill indicator.
    #[must_use]
    pub fn active_pill_x(&self) -> f64 {
        self.active_pill_spring.sample_at(self.active_pill_spring.elapsed_time()).position
    }

    /// Number of open tabs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns `true` if there are no open tabs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// List of tab items.
    #[must_use]
    pub fn tabs(&self) -> &[TabItem] {
        &self.tabs
    }

    /// Active tab ID.
    #[must_use]
    pub fn active_tab_id(&self) -> Option<u64> {
        self.active_tab_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_addition_and_clamped_widths() {
        let mut bar = NativeTabBar::new(600.0);
        bar.add_tab(1, "Documento 1");
        assert_eq!(bar.len(), 1);
        // With 1 tab, width is clamped to max 240.0
        assert_eq!(bar.tab_width(), 240.0);

        // Add 5 more tabs (total 6 tabs)
        for i in 2..=6 {
            bar.add_tab(i, &format!("Doc {i}"));
        }
        assert_eq!(bar.len(), 6);
        // Available width = 600 - 36 = 564. 564 / 6 = 94.0 < min (100.0) => clamped to 100.0
        assert_eq!(bar.tab_width(), 100.0);
    }

    #[test]
    fn test_tab_cycling() {
        let mut bar = NativeTabBar::new(800.0);
        bar.add_tab(10, "A");
        bar.add_tab(20, "B");
        bar.add_tab(30, "C");
        assert_eq!(bar.active_tab_id(), Some(30));

        // Cycle next from C -> A
        bar.select_next_tab();
        assert_eq!(bar.active_tab_id(), Some(10));

        // Cycle next from A -> B
        bar.select_next_tab();
        assert_eq!(bar.active_tab_id(), Some(20));

        // Cycle prev from B -> A
        bar.select_previous_tab();
        assert_eq!(bar.active_tab_id(), Some(10));
    }

    #[test]
    fn test_dirty_state_toggle() {
        let mut bar = NativeTabBar::new(800.0);
        bar.add_tab(100, "main.rs");
        assert!(!bar.tabs()[0].is_dirty);

        bar.set_tab_dirty(100, true);
        assert!(bar.tabs()[0].is_dirty);
    }

    #[test]
    fn test_tab_reordering_flocking() {
        let mut bar = NativeTabBar::new(800.0);
        bar.add_tab(1, "Tab 1");
        bar.add_tab(2, "Tab 2");
        bar.add_tab(3, "Tab 3");

        let tab_w = bar.tab_width();
        // Drag Tab 1 (idx 0) past Tab 2 (to x = 2.5 * tab_w)
        bar.begin_tab_drag(1, 10.0);
        bar.update_tab_drag(2.5 * tab_w);
        bar.end_tab_drag();

        // Tab 1 should now be at the end (idx 2)
        assert_eq!(bar.tabs()[0].id, 2);
        assert_eq!(bar.tabs()[1].id, 3);
        assert_eq!(bar.tabs()[2].id, 1);
    }
}
