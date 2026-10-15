//! # Categorized App Library Subsystem
//!
//! Organizes all desktop applications into automatic semantic categories
//! with Jiggle Mode oscillation physics, usage tracking, and search filtering.

use std::collections::HashMap;

/// Functional categorization for applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AppCategory {
    /// Office, spreadsheets, notes, calendar, finance.
    ProductivityAndFinance,
    /// System utilities, archivers, terminal, calculators.
    Utilities,
    /// IDEs, code editors, compilers, git GUI.
    DeveloperTools,
    /// Photo, vector design, 3D rendering, video editing.
    CreativityAndGraphics,
    /// Music, movies, podcast players, games.
    EntertainmentAndMedia,
    /// Messaging, email, browsers, social networking.
    SocialAndCommunication,
    /// System preferences, settings, activity monitor.
    SystemAndSettings,
    /// Uncategorized or miscellaneous.
    Other,
}

impl AppCategory {
    /// Human-readable category display title.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ProductivityAndFinance => "Productivity & Finance",
            Self::Utilities => "Utilities",
            Self::DeveloperTools => "Developer Tools",
            Self::CreativityAndGraphics => "Creativity & Graphics",
            Self::EntertainmentAndMedia => "Entertainment & Media",
            Self::SocialAndCommunication => "Social & Communication",
            Self::SystemAndSettings => "System & Settings",
            Self::Other => "Other",
        }
    }
}

/// An installed application entry in the App Library.
#[derive(Debug, Clone, PartialEq)]
pub struct AppEntry {
    /// Canonical bundle identifier (e.g. "com.apple.Safari" or "org.mozilla.firefox").
    pub bundle_id: String,
    /// Localized display name.
    pub display_name: String,
    /// Semantic category.
    pub category: AppCategory,
    /// Icon theme or image name.
    pub icon_name: String,
    /// Executable launch command.
    pub exec_path: String,
    /// Launch frequency counter.
    pub launch_count: u32,
    /// User favorite toggle.
    pub is_favorite: bool,
    /// Whether user hid this app from the library grid.
    pub is_hidden: bool,
    /// Whether app is a system app (cannot be deleted in Jiggle mode).
    pub is_system_app: bool,
}

/// Physics state for Jiggle Mode oscillation.
#[derive(Debug, Clone, PartialEq)]
pub struct JiggleState {
    /// Whether Jiggle Mode is currently active.
    pub is_active: bool,
    /// Oscillation amplitude in radians ($1.5^\circ \approx 0.02618$).
    pub amplitude_rad: f32,
    /// Oscillation frequency in Hertz ($8.0\text{Hz}$).
    pub frequency_hz: f32,
    /// Elapsed seconds in jiggle mode.
    pub elapsed_time: f32,
}

impl Default for JiggleState {
    fn default() -> Self {
        Self::new()
    }
}

impl JiggleState {
    /// Creates a new default jiggle state.
    pub fn new() -> Self {
        Self {
            is_active: false,
            amplitude_rad: 1.5 * (std::f32::consts::PI / 180.0),
            frequency_hz: 8.0,
            elapsed_time: 0.0,
        }
    }

    /// Advances simulation time by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if self.is_active {
            self.elapsed_time += dt;
        }
    }

    /// Computes rotation angle for an item given its index in the grid.
    /// Alternates phase per index to avoid uniform rigid movement.
    pub fn rotation_angle_for_item(&self, item_index: usize) -> f32 {
        if !self.is_active {
            return 0.0;
        }

        let phase_offset = (item_index as f32) * 1.05; // slight desynchronization
        let omega = 2.0 * std::f32::consts::PI * self.frequency_hz;
        self.amplitude_rad * (omega * self.elapsed_time + phase_offset).sin()
    }
}

/// App Library catalog management.
pub struct AppLibrary {
    apps: Vec<AppEntry>,
    jiggle_state: JiggleState,
}

impl Default for AppLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl AppLibrary {
    /// Creates a new App Library initialized with standard categories.
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            jiggle_state: JiggleState::new(),
        }
    }

    /// Access reference to Jiggle State.
    pub fn jiggle_state(&self) -> &JiggleState {
        &self.jiggle_state
    }

    /// Access mutable reference to Jiggle State.
    pub fn jiggle_state_mut(&mut self) -> &mut JiggleState {
        &mut self.jiggle_state
    }

    /// Toggles Jiggle Mode on or off.
    pub fn toggle_jiggle_mode(&mut self) -> bool {
        self.jiggle_state.is_active = !self.jiggle_state.is_active;
        if !self.jiggle_state.is_active {
            self.jiggle_state.elapsed_time = 0.0;
        }
        self.jiggle_state.is_active
    }

    /// Registers an application in the library.
    pub fn register_app(&mut self, app: AppEntry) {
        if let Some(pos) = self.apps.iter().position(|a| a.bundle_id == app.bundle_id) {
            self.apps[pos] = app;
        } else {
            self.apps.push(app);
        }
    }

    /// Records application launch, incrementing its usage count.
    pub fn record_launch(&mut self, bundle_id: &str) {
        if let Some(app) = self.apps.iter_mut().find(|a| a.bundle_id == bundle_id) {
            app.launch_count += 1;
        }
    }

    /// Groups non-hidden apps by semantic category.
    pub fn grouped_by_category(&self) -> HashMap<AppCategory, Vec<&AppEntry>> {
        let mut grouped: HashMap<AppCategory, Vec<&AppEntry>> = HashMap::new();

        for app in &self.apps {
            if !app.is_hidden {
                grouped.entry(app.category).or_default().push(app);
            }
        }

        // Sort each category by launch count descending, then display name
        for list in grouped.values_mut() {
            list.sort_by(|a, b| {
                b.launch_count
                    .cmp(&a.launch_count)
                    .then_with(|| a.display_name.cmp(&b.display_name))
            });
        }

        grouped
    }

    /// Searches apps matching query string.
    pub fn search(&self, query: &str) -> Vec<&AppEntry> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            return self.apps.iter().filter(|a| !a.is_hidden).collect();
        }

        self.apps
            .iter()
            .filter(|a| {
                !a.is_hidden
                    && (a.display_name.to_lowercase().contains(&trimmed)
                        || a.bundle_id.to_lowercase().contains(&trimmed)
                        || a.category.display_name().to_lowercase().contains(&trimmed))
            })
            .collect()
    }

    /// Returns all registered apps.
    pub fn apps(&self) -> &[AppEntry] {
        &self.apps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_library_grouping_and_launch_tracking() {
        let mut library = AppLibrary::new();

        library.register_app(AppEntry {
            bundle_id: "com.apple.Safari".into(),
            display_name: "Safari".into(),
            category: AppCategory::SocialAndCommunication,
            icon_name: "safari".into(),
            exec_path: "safari".into(),
            launch_count: 5,
            is_favorite: true,
            is_hidden: false,
            is_system_app: true,
        });

        library.register_app(AppEntry {
            bundle_id: "com.apple.Terminal".into(),
            display_name: "Terminal".into(),
            category: AppCategory::Utilities,
            icon_name: "terminal".into(),
            exec_path: "terminal".into(),
            launch_count: 10,
            is_favorite: true,
            is_hidden: false,
            is_system_app: true,
        });

        library.record_launch("com.apple.Safari");

        let app = library.apps().iter().find(|a| a.bundle_id == "com.apple.Safari").unwrap();
        assert_eq!(app.launch_count, 6);

        let grouped = library.grouped_by_category();
        assert!(grouped.contains_key(&AppCategory::SocialAndCommunication));
        assert!(grouped.contains_key(&AppCategory::Utilities));
    }

    #[test]
    fn test_jiggle_mode_oscillation() {
        let mut jiggle = JiggleState::new();
        assert_eq!(jiggle.rotation_angle_for_item(0), 0.0);

        jiggle.is_active = true;
        jiggle.update(0.05);

        let angle0 = jiggle.rotation_angle_for_item(0);
        let angle1 = jiggle.rotation_angle_for_item(1);

        assert!(angle0.abs() > 0.0);
        // Desynchronized phase means different items have different instantaneous angles
        assert_ne!(angle0, angle1);
    }
}
