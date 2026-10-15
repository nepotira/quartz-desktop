//! # Spotlight Master Controller Subsystem
//!
//! Coordinates the 4-Mode Tahoe 26 Launcher, dynamic window height spring transitions,
//! keyboard navigation, search query routing, and Quick Look previews.

use crate::actions::{MenuRunner, SystemActionRegistry};
use crate::app_library::AppLibrary;
use crate::clipboard::{ClipboardContent, ClipboardHistory};
use crate::search::{SearchEngine, SearchResultItem};
use qz_animation::{SpringAnimation, SpringParams};

/// The active mode of the Spotlight launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotlightMode {
    /// Mode 1 (`Cmd+1` / `Cmd+Space`): Universal Search across apps, files, math, units, and OCR.
    UniversalSearch,
    /// Mode 2 (`Cmd+2`): System commands and active application menu item runner.
    SystemActions,
    /// Mode 3 (`Cmd+3`): Native ring-buffered clipboard history with swatches and pinning.
    ClipboardHistory,
    /// Mode 4 (`Cmd+4`): Categorized application library grid with Jiggle mode.
    AppLibrary,
}

/// Action outcome produced by keyboard or mouse input.
#[derive(Debug, Clone, PartialEq)]
pub enum ControllerEvent {
    /// Window was closed/dismissed.
    Dismiss,
    /// Mode was changed.
    ModeChanged(SpotlightMode),
    /// Search query text was updated.
    QueryUpdated(String),
    /// Selected index changed.
    SelectionChanged(usize),
    /// Item activated for execution.
    Executed {
        /// Mode in which execution occurred.
        mode: SpotlightMode,
        /// Item identifier.
        item_id: String,
    },
    /// Quick Look preview toggled.
    QuickLookToggled(bool),
}

/// Master Spotlight Controller managing all subsystems and UI animation state.
pub struct SpotlightController {
    is_visible: bool,
    mode: SpotlightMode,
    query: String,
    selected_index: usize,
    quick_look_active: bool,
    height_spring: SpringAnimation,
    current_height: f32,
    target_height: f32,

    search_engine: SearchEngine,
    system_actions: SystemActionRegistry,
    menu_runner: MenuRunner,
    clipboard: ClipboardHistory,
    app_library: AppLibrary,
}

impl Default for SpotlightController {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotlightController {
    /// Compact search bar height in pixels.
    pub const COMPACT_HEIGHT: f32 = 56.0;
    /// Maximum expanded results window height in pixels.
    pub const MAX_EXPANDED_HEIGHT: f32 = 520.0;
    /// Standard window width in pixels.
    pub const WINDOW_WIDTH: f32 = 680.0;

    /// Creates a new Spotlight Controller initialized with Tahoe 26 spring physics.
    pub fn new() -> Self {
        let spring_params = SpringParams::window_open();
        let height_spring = SpringAnimation::new(
            spring_params,
            Self::COMPACT_HEIGHT as f64,
            0.0,
            Self::COMPACT_HEIGHT as f64,
        );

        Self {
            is_visible: false,
            mode: SpotlightMode::UniversalSearch,
            query: String::new(),
            selected_index: 0,
            quick_look_active: false,
            height_spring,
            current_height: Self::COMPACT_HEIGHT,
            target_height: Self::COMPACT_HEIGHT,

            search_engine: SearchEngine::new(),
            system_actions: SystemActionRegistry::new(),
            menu_runner: MenuRunner::new(),
            clipboard: ClipboardHistory::new(200),
            app_library: AppLibrary::new(),
        }
    }

    /// Whether Spotlight window is currently open and visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Current launcher mode.
    #[inline]
    pub fn mode(&self) -> SpotlightMode {
        self.mode
    }

    /// Current search query string.
    #[inline]
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Current keyboard selection row index.
    #[inline]
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Whether Quick Look preview panel is currently open over Spotlight.
    #[inline]
    pub fn is_quick_look_active(&self) -> bool {
        self.quick_look_active
    }

    /// Current animated window height in pixels.
    #[inline]
    pub fn current_height(&self) -> f32 {
        self.current_height
    }

    /// Access reference to Search Engine.
    pub fn search_engine(&self) -> &SearchEngine {
        &self.search_engine
    }

    /// Access mutable reference to Search Engine.
    pub fn search_engine_mut(&mut self) -> &mut SearchEngine {
        &mut self.search_engine
    }

    /// Access reference to System Actions.
    pub fn system_actions(&self) -> &SystemActionRegistry {
        &self.system_actions
    }

    /// Access reference to Menu Runner.
    pub fn menu_runner(&self) -> &MenuRunner {
        &self.menu_runner
    }

    /// Access mutable reference to Menu Runner.
    pub fn menu_runner_mut(&mut self) -> &mut MenuRunner {
        &mut self.menu_runner
    }

    /// Access reference to Clipboard History.
    pub fn clipboard(&self) -> &ClipboardHistory {
        &self.clipboard
    }

    /// Access mutable reference to Clipboard History.
    pub fn clipboard_mut(&mut self) -> &mut ClipboardHistory {
        &mut self.clipboard
    }

    /// Access reference to App Library.
    pub fn app_library(&self) -> &AppLibrary {
        &self.app_library
    }

    /// Access mutable reference to App Library.
    pub fn app_library_mut(&mut self) -> &mut AppLibrary {
        &mut self.app_library
    }

    /// Toggles Spotlight visibility on `Cmd+Space`.
    pub fn toggle_visibility(&mut self) {
        if self.is_visible {
            self.dismiss();
        } else {
            self.open();
        }
    }

    /// Opens Spotlight and resets query and selection.
    pub fn open(&mut self) {
        self.is_visible = true;
        self.query.clear();
        self.selected_index = 0;
        self.quick_look_active = false;
        self.target_height = Self::COMPACT_HEIGHT;
        self.height_spring.retarget(Self::COMPACT_HEIGHT as f64);
    }

    /// Dismisses Spotlight and closes any open popovers or quick looks.
    pub fn dismiss(&mut self) {
        self.is_visible = false;
        self.quick_look_active = false;
        if self.app_library.jiggle_state().is_active {
            self.app_library.toggle_jiggle_mode();
        }
    }

    /// Switches the active launcher mode (`Cmd+1` to `Cmd+4`).
    pub fn set_mode(&mut self, mode: SpotlightMode) {
        if self.mode != mode {
            self.mode = mode;
            self.selected_index = 0;
            self.quick_look_active = false;
            self.update_target_height();
        }
    }

    /// Sets query text and recalculates results and window height.
    pub fn set_query(&mut self, text: &str) {
        self.query = text.to_string();
        self.selected_index = 0;
        self.update_target_height();
    }

    /// Advances physics simulation by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        let state = self.height_spring.step(dt as f64);
        self.current_height = state.position as f32;
        self.app_library.jiggle_state_mut().update(dt);
    }

    fn update_target_height(&mut self) {
        let count = self.active_item_count();
        if count == 0 && self.query.trim().is_empty() && self.mode == SpotlightMode::UniversalSearch {
            self.target_height = Self::COMPACT_HEIGHT;
        } else {
            let row_height = 54.0;
            let padding = Self::COMPACT_HEIGHT + 16.0;
            let computed = padding + (count as f32 * row_height);
            self.target_height = computed.clamp(Self::COMPACT_HEIGHT, Self::MAX_EXPANDED_HEIGHT);
        }
        self.height_spring.retarget(self.target_height as f64);
    }

    /// Number of items returned in the active view.
    pub fn active_item_count(&self) -> usize {
        match self.mode {
            SpotlightMode::UniversalSearch => self.search_engine.query(&self.query).len(),
            SpotlightMode::SystemActions => {
                self.system_actions.search(&self.query).len()
                    + self.menu_runner.search(&self.query).len()
            }
            SpotlightMode::ClipboardHistory => self.clipboard.search(&self.query).len(),
            SpotlightMode::AppLibrary => self.app_library.search(&self.query).len(),
        }
    }

    /// Handles keyboard arrow down navigation.
    pub fn navigate_down(&mut self) {
        let count = self.active_item_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    /// Handles keyboard arrow up navigation.
    pub fn navigate_up(&mut self) {
        let count = self.active_item_count();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Toggles Quick Look preview on Space bar press.
    pub fn toggle_quick_look(&mut self) {
        self.quick_look_active = !self.quick_look_active;
    }

    /// Retrieves current Universal Search results if in UniversalSearch mode.
    pub fn current_search_results(&self) -> Vec<SearchResultItem> {
        if self.mode == SpotlightMode::UniversalSearch {
            self.search_engine.query(&self.query)
        } else {
            Vec::new()
        }
    }

    /// Records a new clipboard entry into history.
    pub fn record_clipboard_copy(&mut self, content: ClipboardContent) -> u64 {
        self.clipboard.push(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spotlight_controller_modes_and_spring_height() {
        let mut controller = SpotlightController::new();
        assert_eq!(controller.current_height(), SpotlightController::COMPACT_HEIGHT);

        controller.open();
        assert!(controller.is_visible());

        // Type query that evaluates in math calculator
        controller.set_query("15 * 4");
        assert_eq!(controller.active_item_count(), 1);

        // Advance physics
        for _ in 0..60 {
            controller.update(1.0 / 120.0);
        }

        // Window height should have expanded smoothly
        assert!(controller.current_height() > SpotlightController::COMPACT_HEIGHT);

        // Switch modes
        controller.set_mode(SpotlightMode::SystemActions);
        assert_eq!(controller.mode(), SpotlightMode::SystemActions);

        controller.set_mode(SpotlightMode::ClipboardHistory);
        assert_eq!(controller.mode(), SpotlightMode::ClipboardHistory);

        controller.set_mode(SpotlightMode::AppLibrary);
        assert_eq!(controller.mode(), SpotlightMode::AppLibrary);

        controller.dismiss();
        assert!(!controller.is_visible());
    }

    #[test]
    fn test_spotlight_navigation_cycling() {
        let mut controller = SpotlightController::new();
        controller.open();

        // System actions mode has many items
        controller.set_mode(SpotlightMode::SystemActions);
        let count = controller.active_item_count();
        assert!(count > 0);

        controller.navigate_down();
        assert_eq!(controller.selected_index(), 1);

        controller.navigate_up();
        assert_eq!(controller.selected_index(), 0);

        controller.navigate_up();
        assert_eq!(controller.selected_index(), count - 1);
    }
}
