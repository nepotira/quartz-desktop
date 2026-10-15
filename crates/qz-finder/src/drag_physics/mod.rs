//! # Spring-Loaded Folders & Multi-Item Flocking Drag Physics Subsystem
//!
//! Provides the physical drag-and-drop mechanics of macOS Tahoe 26 Finder:
//! - **Spring-Loaded Folders**: Hovering dragged items over a folder icon for 500ms
//!   (or tapping `Space`) triggers spring expansion of the destination folder.
//! - **Multi-Item Flocking Physics**: Dragging multiple items aggregates them organically toward
//!   the cursor with 2nd-order damped harmonic oscillator springs and a badge count.

use qz_animation::{SpringAnimation, SpringParams};
use std::path::PathBuf;

/// State tracker for Spring-Loaded Folders during a drag operation.
#[derive(Debug, Clone, PartialEq)]
pub struct SpringLoadedFolderTracker {
    target_folder: Option<PathBuf>,
    hover_duration_secs: f32,
    activation_threshold_secs: f32,
    is_triggered: bool,
    highlight_flash_phase: f32,
}

impl Default for SpringLoadedFolderTracker {
    fn default() -> Self {
        Self::new(0.50) // Standard 500ms hover delay
    }
}

impl SpringLoadedFolderTracker {
    /// Creates a tracker with the specified hover threshold duration.
    pub fn new(activation_threshold_secs: f32) -> Self {
        Self {
            target_folder: None,
            hover_duration_secs: 0.0,
            activation_threshold_secs: activation_threshold_secs.max(0.1),
            is_triggered: false,
            highlight_flash_phase: 0.0,
        }
    }

    /// Updates pointer hover over a candidate folder.
    pub fn update_hover(&mut self, folder_path: Option<PathBuf>, dt: f32) -> bool {
        match folder_path {
            Some(path) => {
                if self.target_folder.as_ref() == Some(&path) {
                    if !self.is_triggered {
                        self.hover_duration_secs += dt;
                        self.highlight_flash_phase += dt * 4.0;
                        if self.hover_duration_secs >= self.activation_threshold_secs {
                            self.is_triggered = true;
                            return true; // Trigger folder spring-open!
                        }
                    }
                } else {
                    // New target folder
                    self.target_folder = Some(path);
                    self.hover_duration_secs = dt;
                    self.is_triggered = false;
                    self.highlight_flash_phase = dt * 4.0;
                    if self.hover_duration_secs >= self.activation_threshold_secs {
                        self.is_triggered = true;
                        return true;
                    }
                }
            }
            None => {
                // Pointer left folder boundary
                self.reset();
            }
        }
        false
    }

    /// Instantly triggers spring expansion on Spacebar press during drag.
    pub fn trigger_immediate(&mut self) -> bool {
        if self.target_folder.is_some() && !self.is_triggered {
            self.is_triggered = true;
            true
        } else {
            false
        }
    }

    /// Resets hover tracker.
    pub fn reset(&mut self) {
        self.target_folder = None;
        self.hover_duration_secs = 0.0;
        self.is_triggered = false;
        self.highlight_flash_phase = 0.0;
    }

    /// Normalized progress towards spring-open trigger $[0.0, 1.0]$.
    #[inline]
    pub fn progress(&self) -> f32 {
        (self.hover_duration_secs / self.activation_threshold_secs).clamp(0.0, 1.0)
    }

    /// Active hover target folder path if any.
    #[inline]
    pub fn target_folder(&self) -> Option<&PathBuf> {
        self.target_folder.as_ref()
    }
}

/// A trailing item in a multi-item flocking drag.
#[derive(Debug, Clone)]
pub struct FlockingItem {
    /// Item identifier.
    pub item_id: String,
    /// X-axis spring tracking cursor X.
    pub spring_x: SpringAnimation,
    /// Y-axis spring tracking cursor Y.
    pub spring_y: SpringAnimation,
    /// Current rendered position.
    pub current_pos: [f32; 2],
    /// Splay rotation angle in radians.
    pub rotation_rad: f32,
}

/// Multi-Item Flocking Drag Physics manager.
pub struct FlockingDragManager {
    items: Vec<FlockingItem>,
    lead_cursor_pos: [f32; 2],
    is_dragging: bool,
}

impl Default for FlockingDragManager {
    fn default() -> Self {
        Self::new()
    }
}

impl FlockingDragManager {
    /// Creates a new flocking drag manager.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            lead_cursor_pos: [0.0, 0.0],
            is_dragging: false,
        }
    }

    /// Begins a multi-item drag at the specified cursor position.
    pub fn start_drag(&mut self, item_ids: &[String], initial_cursor_pos: [f32; 2]) {
        self.items.clear();
        self.lead_cursor_pos = initial_cursor_pos;
        self.is_dragging = true;

        let spring_params = SpringParams {
            mass: 1.0,
            stiffness: 420.0,
            damping: 34.0, // Snappy sub-amortecida flocking mola
        };

        for (idx, id) in item_ids.iter().enumerate() {
            // Subtle rotation splay between -4 degrees and +4 degrees
            let splay = ((idx as f32 % 5.0) - 2.0) * 0.035;

            self.items.push(FlockingItem {
                item_id: id.clone(),
                spring_x: SpringAnimation::new(
                    spring_params,
                    initial_cursor_pos[0] as f64,
                    0.0,
                    initial_cursor_pos[0] as f64,
                ),
                spring_y: SpringAnimation::new(
                    spring_params,
                    initial_cursor_pos[1] as f64,
                    0.0,
                    initial_cursor_pos[1] as f64,
                ),
                current_pos: initial_cursor_pos,
                rotation_rad: splay,
            });
        }
    }

    /// Updates cursor position during drag.
    pub fn update_cursor(&mut self, cursor_pos: [f32; 2]) {
        self.lead_cursor_pos = cursor_pos;

        for (idx, item) in self.items.iter_mut().enumerate() {
            // Trailing items gather into a compact fan near the cursor
            let offset_x = (idx.min(4) as f32) * 4.0;
            let offset_y = (idx.min(4) as f32) * 3.0;

            let target_x = (cursor_pos[0] + offset_x) as f64;
            let target_y = (cursor_pos[1] + offset_y) as f64;

            item.spring_x.retarget(target_x);
            item.spring_y.retarget(target_y);
        }
    }

    /// Advances physics simulation clock.
    pub fn update(&mut self, dt: f32) {
        if !self.is_dragging {
            return;
        }

        for item in &mut self.items {
            let state_x = item.spring_x.step(dt as f64);
            let state_y = item.spring_y.step(dt as f64);
            item.current_pos = [state_x.position as f32, state_y.position as f32];
        }
    }

    /// Finishes or cancels the drag operation.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        self.items.clear();
    }

    /// Number of items currently being dragged.
    #[inline]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Whether drag is currently in progress.
    #[inline]
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Trailing flock items with their current physics positions.
    pub fn items(&self) -> &[FlockingItem] {
        &self.items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spring_loaded_folder_hover_trigger() {
        let mut tracker = SpringLoadedFolderTracker::new(0.5);

        let folder = PathBuf::from("/Users/demo/Desktop/TargetFolder");

        // Hover for 300ms -> should not trigger yet
        assert!(!tracker.update_hover(Some(folder.clone()), 0.3));
        assert_eq!(tracker.progress(), 0.6);

        // Hover for another 250ms (total 550ms > 500ms) -> triggers!
        assert!(tracker.update_hover(Some(folder), 0.25));
        assert_eq!(tracker.progress(), 1.0);
    }

    #[test]
    fn test_multi_item_flocking_drag_convergence() {
        let mut flock = FlockingDragManager::new();

        let item_ids = vec!["file1.png".to_string(), "file2.png".to_string(), "file3.png".to_string()];
        flock.start_drag(&item_ids, [100.0, 100.0]);
        assert_eq!(flock.item_count(), 3);

        // Move cursor to (300.0, 400.0)
        flock.update_cursor([300.0, 400.0]);

        // Step physics forward
        for _ in 0..60 {
            flock.update(1.0 / 60.0);
        }

        // Flocking items should have moved near target cursor
        let first_item_pos = flock.items()[0].current_pos;
        assert!((first_item_pos[0] - 300.0).abs() < 5.0);
        assert!((first_item_pos[1] - 400.0).abs() < 5.0);
    }
}
