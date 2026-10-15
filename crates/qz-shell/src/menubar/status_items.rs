//! # Status Items (System Tray) and Option-Diagnostics
//!
//! Controls right-side status indicators on the Top Menu Bar:
//! - **Cmd+Drag**: Dragging with `Cmd` allows reordering status icons; dragging $> 45\text{px}$
//!   away from the bar triggers the **Poof** smoke dissipation effect and removes the item.
//! - **Option + Wi-Fi**: Comprehensive radio diagnostics (MAC, BSSID, Channel, Band, RSSI, Noise, Tx rate, MCS, PHY, Security).
//! - **Option + Bluetooth**: Controller version, MAC, RSSI, and active audio codec (AAC/aptX/LDAC).
//! - **Option + Sound**: Direct audio input and output device picker.
//! - **Option + Clock**: Instant toggle of Do Not Disturb focus mode.

use glam::Vec2;

/// Exhaustive Wi-Fi diagnostic metrics revealed on Option-Click.
#[derive(Debug, Clone, PartialEq)]
pub struct WifiDiagnostics {
    /// Interface name (e.g. "wlan0").
    pub interface: String,
    /// Hardware MAC address.
    pub mac_address: String,
    /// Connected Access Point BSSID.
    pub bssid: String,
    /// Primary channel number (e.g. 36).
    pub channel: u32,
    /// Operating frequency band (2.4 GHz, 5 GHz, or 6 GHz).
    pub band_ghz: f32,
    /// Received Signal Strength Indication in dBm (e.g. -48 dBm).
    pub rssi_dbm: i32,
    /// Ambient noise floor in dBm (e.g. -92 dBm).
    pub noise_dbm: i32,
    /// Current transmission bitrate in Mbps (e.g. 1201 Mbps).
    pub tx_rate_mbps: f32,
    /// Modulation and Coding Scheme index (e.g. MCS 11).
    pub mcs_index: u32,
    /// Physical 802.11 mode (e.g. "802.11ax (Wi-Fi 6)").
    pub phy_mode: String,
    /// Security protocol (e.g. "WPA3-Personal").
    pub security: String,
}

/// Bluetooth hardware and audio codec diagnostics revealed on Option-Click.
#[derive(Debug, Clone, PartialEq)]
pub struct BluetoothDiagnostics {
    /// Controller firmware version.
    pub controller_version: String,
    /// Host controller MAC address.
    pub mac_address: String,
    /// Signal strength to connected audio sink in dBm.
    pub rssi_dbm: i32,
    /// Active high-fidelity Bluetooth audio codec.
    pub active_codec: String, // "AAC", "aptX HD", "LDAC"
}

/// Identifiers for system status icons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusIconKind {
    /// Battery and power charge indicator.
    Battery,
    /// Wi-Fi network connection.
    Wifi,
    /// Bluetooth peripherals.
    Bluetooth,
    /// Audio volume and device selector.
    Sound,
    /// Hardware privacy indicator capsule.
    PrivacyCapsule,
    /// Control Center trigger button.
    ControlCenter,
    /// System clock and date (toggles Notification Center on click).
    Clock,
}

/// State of an individual status icon on the Top Menu Bar.
#[derive(Debug, Clone, PartialEq)]
pub struct StatusItem {
    /// Unique item kind.
    pub kind: StatusIconKind,
    /// Current screen X coordinate.
    pub x: f32,
    /// Icon bounding width.
    pub width: f32,
    /// True if item is removable by dragging.
    pub is_removable: bool,
}

/// Manager coordinating status items, drag reordering, and Option diagnostics.
pub struct StatusItemsManager {
    items: Vec<StatusItem>,
    is_dnd_active: bool,
    dragged_item: Option<(StatusIconKind, Vec2)>,
}

impl Default for StatusItemsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusItemsManager {
    /// Distance in pixels away from the bar required to trigger Poof removal (45px).
    pub const POOF_THRESHOLD_PX: f32 = 45.0;

    /// Creates standard right-side status tray layout.
    pub fn new() -> Self {
        let items = vec![
            StatusItem { kind: StatusIconKind::Battery, x: 0.0, width: 36.0, is_removable: true },
            StatusItem { kind: StatusIconKind::Wifi, x: 0.0, width: 28.0, is_removable: true },
            StatusItem { kind: StatusIconKind::Bluetooth, x: 0.0, width: 24.0, is_removable: true },
            StatusItem { kind: StatusIconKind::Sound, x: 0.0, width: 26.0, is_removable: true },
            StatusItem { kind: StatusIconKind::ControlCenter, x: 0.0, width: 32.0, is_removable: false },
            StatusItem { kind: StatusIconKind::Clock, x: 0.0, width: 84.0, is_removable: false },
        ];

        Self {
            items,
            is_dnd_active: false,
            dragged_item: None,
        }
    }

    /// Toggles Do Not Disturb mode (Option-Click on Clock).
    pub fn toggle_do_not_disturb(&mut self) -> bool {
        self.is_dnd_active = !self.is_dnd_active;
        self.is_dnd_active
    }

    /// Whether Do Not Disturb is currently engaged.
    #[inline]
    pub fn is_dnd_active(&self) -> bool {
        self.is_dnd_active
    }

    /// Handles Cmd-Drag of a status item.
    ///
    /// Returns `true` if item was dragged $> 45\text{px}$ away and should trigger **Poof**.
    pub fn handle_drag(&mut self, kind: StatusIconKind, cursor_pos: Vec2, bar_height: f32) -> bool {
        let dist_y = cursor_pos.y - bar_height;
        if dist_y > Self::POOF_THRESHOLD_PX {
            // Dragged out of bar: remove item with Poof smoke animation
            self.items.retain(|i| i.kind != kind || !i.is_removable);
            self.dragged_item = None;
            true
        } else {
            self.dragged_item = Some((kind, cursor_pos));
            false
        }
    }

    /// Formats Wi-Fi Option diagnostics popup text.
    pub fn format_wifi_diagnostics(d: &WifiDiagnostics) -> Vec<(String, String)> {
        vec![
            ("Interface".to_string(), d.interface.clone()),
            ("MAC Address".to_string(), d.mac_address.clone()),
            ("BSSID".to_string(), d.bssid.clone()),
            ("Channel".to_string(), format!("{} ({:.1} GHz)", d.channel, d.band_ghz)),
            ("RSSI".to_string(), format!("{} dBm", d.rssi_dbm)),
            ("Noise".to_string(), format!("{} dBm", d.noise_dbm)),
            ("Tx Rate".to_string(), format!("{:.0} Mbps", d.tx_rate_mbps)),
            ("PHY Mode".to_string(), d.phy_mode.clone()),
            ("MCS Index".to_string(), d.mcs_index.to_string()),
            ("Security".to_string(), d.security.clone()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poof_removal_on_drag_past_threshold() {
        let mut manager = StatusItemsManager::new();

        // Drag Bluetooth icon 50px below the 32px menu bar (total y = 82px)
        let removed = manager.handle_drag(
            StatusIconKind::Bluetooth,
            Vec2::new(1700.0, 82.0),
            32.0,
        );

        assert!(removed, "Should trigger poof removal when dragged > 45px past bar");
        // Bluetooth should no longer be present
        assert!(!manager.items.iter().any(|i| i.kind == StatusIconKind::Bluetooth));
    }

    #[test]
    fn test_dnd_option_clock_toggle() {
        let mut manager = StatusItemsManager::new();
        assert!(!manager.is_dnd_active());

        manager.toggle_do_not_disturb();
        assert!(manager.is_dnd_active());

        manager.toggle_do_not_disturb();
        assert!(!manager.is_dnd_active());
    }
}
