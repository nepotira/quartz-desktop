//! # Canonical Apple Menu () Subsystem
//!
//! Controls the root system menu:
//! - About This Mac, System Settings (badge indicator), App Store, Recent Items submenu.
//! - Force Quit Applications dialog (`Opt+Cmd+Esc`).
//! - Sleep, Restart, and Shut Down:
//!   - Standard click presents a 60-second confirmation Sheet with "Reopen windows on login".
//!   - **Option-Click:** Instantly executes Sleep/Restart/Shutdown without confirmation prompt!
//! - Lock Screen (`Ctrl+Cmd+Q`) and Log Out (`Shift+Cmd+Q`).

/// Canonical Apple Menu action commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppleMenuAction {
    /// Open "About This Mac" system overview window.
    AboutThisMac,
    /// Open System Settings (optionally highlighting software update pane).
    SystemSettings {
        /// Number of pending software updates.
        pending_updates: u32,
    },
    /// Launch App Store / Package Manager.
    AppStore,
    /// Clear Recent Items history.
    ClearRecentItems,
    /// Summon Force Quit Applications dialog (`Opt+Cmd+Esc`).
    ForceQuitDialog,
    /// Put system to sleep.
    Sleep {
        /// If true, executes immediately without confirmation sheet (e.g. Option held).
        immediate_no_confirm: bool,
    },
    /// Restart system.
    Restart {
        /// If true, executes immediately without confirmation sheet (e.g. Option held).
        immediate_no_confirm: bool,
    },
    /// Power down system.
    ShutDown {
        /// If true, executes immediately without confirmation sheet (e.g. Option held).
        immediate_no_confirm: bool,
    },
    /// Lock display session immediately (`Ctrl+Cmd+Q`).
    LockScreen,
    /// Terminate user login session (`Shift+Cmd+Q`).
    LogOut {
        /// If true, logs out immediately without confirmation countdown.
        immediate_no_confirm: bool,
    },
}

/// An entry within the Apple Menu tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppleMenuItem {
    /// Display label.
    pub label: String,
    /// Associated keyboard shortcut accelerator if any.
    pub shortcut: Option<String>,
    /// Action triggered on click.
    pub action: Option<AppleMenuAction>,
    /// True if item is a horizontal separator line.
    pub is_separator: bool,
    /// Alternate label and action shown when user holds the `Option` key.
    pub option_variant: Option<(String, AppleMenuAction)>,
}

impl AppleMenuItem {
    /// Creates a clickable action menu item.
    pub fn action(label: &str, shortcut: Option<&str>, action: AppleMenuAction) -> Self {
        Self {
            label: label.to_string(),
            shortcut: shortcut.map(|s| s.to_string()),
            action: Some(action),
            is_separator: false,
            option_variant: None,
        }
    }

    /// Attaches an Option-key mutation to this item.
    pub fn with_option_variant(mut self, alt_label: &str, alt_action: AppleMenuAction) -> Self {
        self.option_variant = Some((alt_label.to_string(), alt_action));
        self
    }

    /// Creates a horizontal separator.
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: None,
            is_separator: true,
            option_variant: None,
        }
    }

    /// Evaluates current label and action based on whether `Option` is depressed.
    pub fn resolve(&self, option_held: bool) -> (&str, Option<&AppleMenuAction>) {
        if option_held {
            if let Some((ref alt_label, ref alt_action)) = self.option_variant {
                return (alt_label.as_str(), Some(alt_action));
            }
        }
        (self.label.as_str(), self.action.as_ref())
    }
}

/// Factory constructing the full Apple Menu tree.
pub struct AppleMenuBuilder;

impl AppleMenuBuilder {
    /// Builds the standard macOS Tahoe 26 Apple Menu items list.
    pub fn build(pending_updates: u32) -> Vec<AppleMenuItem> {
        let settings_label = if pending_updates > 0 {
            format!("System Settings... ({} update)", pending_updates)
        } else {
            "System Settings...".to_string()
        };

        vec![
            AppleMenuItem::action("About This Mac", None, AppleMenuAction::AboutThisMac),
            AppleMenuItem::action(&settings_label, None, AppleMenuAction::SystemSettings { pending_updates }),
            AppleMenuItem::action("App Store...", None, AppleMenuAction::AppStore),
            AppleMenuItem::separator(),
            AppleMenuItem::action(
                "Force Quit...",
                Some("⌥⌘⎋"),
                AppleMenuAction::ForceQuitDialog,
            ),
            AppleMenuItem::separator(),
            AppleMenuItem::action("Sleep", None, AppleMenuAction::Sleep { immediate_no_confirm: false }),
            AppleMenuItem::action("Restart...", None, AppleMenuAction::Restart { immediate_no_confirm: false })
                .with_option_variant("Restart", AppleMenuAction::Restart { immediate_no_confirm: true }),
            AppleMenuItem::action("Shut Down...", None, AppleMenuAction::ShutDown { immediate_no_confirm: false })
                .with_option_variant("Shut Down", AppleMenuAction::ShutDown { immediate_no_confirm: true }),
            AppleMenuItem::separator(),
            AppleMenuItem::action("Lock Screen", Some("⌃⌘Q"), AppleMenuAction::LockScreen),
            AppleMenuItem::action("Log Out...", Some("⇧⌘Q"), AppleMenuAction::LogOut { immediate_no_confirm: false })
                .with_option_variant("Log Out", AppleMenuAction::LogOut { immediate_no_confirm: true }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_menu_option_mutation() {
        let menu = AppleMenuBuilder::build(1);

        // Find the "Restart..." item (index 7)
        let restart_item = menu.iter().find(|i| i.label == "Restart...").unwrap();

        // Normal state: label is "Restart...", action requires confirmation
        let (label_normal, action_normal) = restart_item.resolve(false);
        assert_eq!(label_normal, "Restart...");
        assert_eq!(action_normal, Some(&AppleMenuAction::Restart { immediate_no_confirm: false }));

        // Holding Option: label mutates to "Restart", executes immediately without confirmation!
        let (label_opt, action_opt) = restart_item.resolve(true);
        assert_eq!(label_opt, "Restart");
        assert_eq!(action_opt, Some(&AppleMenuAction::Restart { immediate_no_confirm: true }));
    }
}
