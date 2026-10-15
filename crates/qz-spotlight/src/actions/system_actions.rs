//! # System Actions Subsystem
//!
//! Direct execution of system management tasks from Spotlight (`Cmd+2`),
//! including Power controls, Session Lock, Screen Capture, and Hardware toggles.

/// System action command identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemActionId {
    /// Put the machine to sleep.
    Sleep,
    /// Restart the system.
    Restart,
    /// Shut down and power off.
    ShutDown,
    /// Lock the active user session (`Ctrl+Cmd+Q`).
    LockScreen,
    /// Log out active user (`Shift+Cmd+Q`).
    LogOut,
    /// Empty system trash with shred option.
    EmptyTrash,
    /// Toggle between Dark Mode and Light Mode appearance.
    ToggleDarkMode,
    /// Toggle Wi-Fi radio interface on or off.
    ToggleWifi,
    /// Toggle Bluetooth radio interface on or off.
    ToggleBluetooth,
    /// Toggle Low Power Mode energy saving.
    ToggleLowPowerMode,
    /// Toggle Night Shift blue light filter.
    ToggleNightShift,
    /// Capture entire display to screenshot file (`Cmd+Shift+3`).
    TakeFullScreenshot,
    /// Capture selected rectangular area (`Cmd+Shift+4`).
    TakeSelectionScreenshot,
    /// Open screen recording utility (`Cmd+Shift+5`).
    ScreenRecording,
    /// Launch macOS Tahoe 26 Activity Monitor.
    OpenActivityMonitor,
    /// Summon Force Quit Applications dialog (`Opt+Cmd+Esc`).
    ForceQuitDialog,
}

/// A system command entry with title, icon, shortcut, and synonym keywords.
#[derive(Debug, Clone)]
pub struct SystemCommand {
    /// Command identifier.
    pub id: SystemActionId,
    /// Primary title (e.g. "Shut Down...").
    pub title: &'static str,
    /// Subtitle or explanation.
    pub description: &'static str,
    /// Keyboard shortcut string if standard.
    pub shortcut: Option<&'static str>,
    /// SF Symbols / icon identifier.
    pub icon_name: &'static str,
    /// Synonym terms for fuzzy matching.
    pub synonyms: &'static [&'static str],
}

/// System action search hit.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemActionHit {
    /// Action identifier.
    pub id: SystemActionId,
    /// Title string.
    pub title: String,
    /// Subtitle description.
    pub description: String,
    /// Optional accelerator shortcut.
    pub shortcut: Option<String>,
    /// Icon resource.
    pub icon_name: String,
    /// Match score.
    pub score: f32,
}

/// Registry of system actions and commands.
pub struct SystemActionRegistry {
    commands: Vec<SystemCommand>,
}

impl Default for SystemActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemActionRegistry {
    /// Initializes registry with the standard Tahoe 26 system actions.
    pub fn new() -> Self {
        let commands = vec![
            SystemCommand {
                id: SystemActionId::Sleep,
                title: "Sleep",
                description: "Put the computer into low-power sleep state",
                shortcut: None,
                icon_name: "moon.fill",
                synonyms: &["suspend", "hibernate", "standby", "dormir"],
            },
            SystemCommand {
                id: SystemActionId::Restart,
                title: "Restart...",
                description: "Reboot the operating system",
                shortcut: None,
                icon_name: "arrow.clockwise",
                synonyms: &["reboot", "reiniciar", "reset"],
            },
            SystemCommand {
                id: SystemActionId::ShutDown,
                title: "Shut Down...",
                description: "Safely power down all hardware",
                shortcut: None,
                icon_name: "power",
                synonyms: &["power off", "turn off", "desligar", "halt"],
            },
            SystemCommand {
                id: SystemActionId::LockScreen,
                title: "Lock Screen",
                description: "Lock display session requiring authentication",
                shortcut: Some("⌃⌘Q"),
                icon_name: "lock.fill",
                synonyms: &["bloquear", "lock", "screen lock", "proteger"],
            },
            SystemCommand {
                id: SystemActionId::LogOut,
                title: "Log Out...",
                description: "Terminate current user login session",
                shortcut: Some("⇧⌘Q"),
                icon_name: "rectangle.portrait.and.arrow.right",
                synonyms: &["sign out", "sair", "encerrar sessao", "exit"],
            },
            SystemCommand {
                id: SystemActionId::EmptyTrash,
                title: "Empty Trash",
                description: "Permanently erase items in the Trash",
                shortcut: Some("⇧⌘⌫"),
                icon_name: "trash",
                synonyms: &["clear trash", "esvaziar lixeira", "recycle bin", "delete"],
            },
            SystemCommand {
                id: SystemActionId::ToggleDarkMode,
                title: "Toggle Dark Mode",
                description: "Switch system appearance between Dark and Light mode",
                shortcut: None,
                icon_name: "circle.righthalf.filled",
                synonyms: &["dark mode", "light mode", "modo escuro", "modo claro", "theme"],
            },
            SystemCommand {
                id: SystemActionId::ToggleWifi,
                title: "Toggle Wi-Fi",
                description: "Turn Wi-Fi wireless networking on or off",
                shortcut: None,
                icon_name: "wifi",
                synonyms: &["wireless", "wlan", "internet", "rede sem fio"],
            },
            SystemCommand {
                id: SystemActionId::ToggleBluetooth,
                title: "Toggle Bluetooth",
                description: "Turn Bluetooth wireless controller on or off",
                shortcut: None,
                icon_name: "dot.radiowaves.left.and.right",
                synonyms: &["bt", "wireless devices"],
            },
            SystemCommand {
                id: SystemActionId::ToggleLowPowerMode,
                title: "Toggle Low Power Mode",
                description: "Reduce energy consumption to extend battery life",
                shortcut: None,
                icon_name: "battery.50percent",
                synonyms: &["battery saver", "economia de bateria", "power save"],
            },
            SystemCommand {
                id: SystemActionId::ToggleNightShift,
                title: "Toggle Night Shift",
                description: "Shift display colors to warmer end of spectrum",
                shortcut: None,
                icon_name: "sun.max.fill",
                synonyms: &["blue light", "night mode", "warm colors", "filtro azul"],
            },
            SystemCommand {
                id: SystemActionId::TakeFullScreenshot,
                title: "Take Full Screenshot",
                description: "Capture the entire desktop display to file",
                shortcut: Some("⇧⌘3"),
                icon_name: "camera.fill",
                synonyms: &["screenshot", "print screen", "captura de tela", "screen capture"],
            },
            SystemCommand {
                id: SystemActionId::TakeSelectionScreenshot,
                title: "Take Selection Screenshot",
                description: "Capture selected crosshair rectangle to file",
                shortcut: Some("⇧⌘4"),
                icon_name: "crop",
                synonyms: &["snip", "snipping tool", "area screenshot", "recorte"],
            },
            SystemCommand {
                id: SystemActionId::ScreenRecording,
                title: "Screen Recording Utility",
                description: "Record video of desktop or application window",
                shortcut: Some("⇧⌘5"),
                icon_name: "record.circle",
                synonyms: &["record screen", "video capture", "gravacao de tela"],
            },
            SystemCommand {
                id: SystemActionId::OpenActivityMonitor,
                title: "Open Activity Monitor",
                description: "View CPU, memory pressure, disk, network and energy",
                shortcut: None,
                icon_name: "waveform.path.ecg",
                synonyms: &["task manager", "processes", "monitor de atividade", "kill process"],
            },
            SystemCommand {
                id: SystemActionId::ForceQuitDialog,
                title: "Force Quit Applications...",
                description: "Terminate unresponsive applications",
                shortcut: Some("⌥⌘⎋"),
                icon_name: "xmark.circle.fill",
                synonyms: &["force quit", "forcar encerramento", "kill", "not responding"],
            },
        ];

        Self { commands }
    }

    /// Searches system commands matching the query string.
    pub fn search(&self, query: &str) -> Vec<SystemActionHit> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            // Return all commands with default ordering
            return self
                .commands
                .iter()
                .map(|cmd| SystemActionHit {
                    id: cmd.id,
                    title: cmd.title.to_string(),
                    description: cmd.description.to_string(),
                    shortcut: cmd.shortcut.map(String::from),
                    icon_name: cmd.icon_name.to_string(),
                    score: 100.0,
                })
                .collect();
        }

        let mut hits = Vec::new();

        for cmd in &self.commands {
            let title_lower = cmd.title.to_lowercase();
            let mut score = 0.0;

            if title_lower == trimmed {
                score = 1000.0;
            } else if title_lower.starts_with(&trimmed) {
                score = 800.0;
            } else if title_lower.contains(&trimmed) {
                score = 600.0;
            } else {
                for &syn in cmd.synonyms {
                    if syn.to_lowercase() == trimmed {
                        score = 750.0;
                        break;
                    } else if syn.to_lowercase().starts_with(&trimmed) {
                        score = 500.0;
                        break;
                    } else if syn.to_lowercase().contains(&trimmed) {
                        score = 400.0;
                        break;
                    }
                }
            }

            if score > 0.0 {
                hits.push(SystemActionHit {
                    id: cmd.id,
                    title: cmd.title.to_string(),
                    description: cmd.description.to_string(),
                    shortcut: cmd.shortcut.map(String::from),
                    icon_name: cmd.icon_name.to_string(),
                    score,
                });
            }
        }

        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        hits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_action_search_synonyms() {
        let registry = SystemActionRegistry::new();

        let reboot_hits = registry.search("reboot");
        assert!(!reboot_hits.is_empty());
        assert_eq!(reboot_hits[0].id, SystemActionId::Restart);

        let power_hits = registry.search("power off");
        assert!(!power_hits.is_empty());
        assert_eq!(power_hits[0].id, SystemActionId::ShutDown);

        let screenshot_hits = registry.search("screenshot");
        assert!(!screenshot_hits.is_empty());
        assert_eq!(screenshot_hits[0].id, SystemActionId::TakeFullScreenshot);
    }
}
