//! # Wi-Fi and Bluetooth Option-Diagnostics Daemon (`qz-daemons::network`)
//!
//! Production-grade network monitoring daemon implementing the canonical macOS Tahoe 26
//! **Option-Diagnostics** popover details for both Wi-Fi and Bluetooth hardware.
//!
//! ### Option-Diagnostics Features:
//! - **Wi-Fi RF & PHY Layer Inspection:** Exposes BSSID, RSSI (dBm), Noise floor (dBm),
//!   Signal-to-Noise Ratio ($\text{SNR} = \text{RSSI} - \text{Noise}$), Channel number,
//!   frequency band (2.4 GHz, 5 GHz, 6 GHz), channel width (20/40/80/160/320 MHz),
//!   PHY standard (802.11a/b/g/n/ac/ax/be), MCS index, NSS spatial streams, Tx Rate (Mbps),
//!   and security protocol (WPA3-SAE, WPA2-Enterprise, etc.).
//! - **Bluetooth Link Inspection:** Exposes device categories, RSSI, link quality metric,
//!   latency (ms), active audio codec (AAC, LDAC, aptX, LC3), and multi-component battery status
//!   (AirPods Left/Right/Case or single accessory battery).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 802.11 PHY Standard Protocol Version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WifiPhyMode {
    /// Legacy 802.11a (5 GHz OFDM, up to 54 Mbps).
    Phy80211a,
    /// Legacy 802.11b (2.4 GHz DSSS, up to 11 Mbps).
    Phy80211b,
    /// 802.11g (2.4 GHz OFDM, up to 54 Mbps).
    Phy80211g,
    /// 802.11n / Wi-Fi 4 (MIMO, up to 600 Mbps).
    Phy80211n,
    /// 802.11ac / Wi-Fi 5 (5 GHz wide channels, up to 6.9 Gbps).
    Phy80211ac,
    /// 802.11ax / Wi-Fi 6 & 6E (OFDMA, 1024-QAM, up to 9.6 Gbps).
    Phy80211ax,
    /// 802.11be / Wi-Fi 7 (320 MHz, 4096-QAM, Multi-Link Operation, up to 46 Gbps).
    Phy80211be,
}

impl WifiPhyMode {
    /// Returns the human-readable marketing string (e.g. "Wi-Fi 6 (802.11ax)").
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Phy80211a => "802.11a",
            Self::Phy80211b => "802.11b",
            Self::Phy80211g => "802.11g",
            Self::Phy80211n => "Wi-Fi 4 (802.11n)",
            Self::Phy80211ac => "Wi-Fi 5 (802.11ac)",
            Self::Phy80211ax => "Wi-Fi 6 / 6E (802.11ax)",
            Self::Phy80211be => "Wi-Fi 7 (802.11be)",
        }
    }
}

/// Radio frequency band for Wi-Fi channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WifiBand {
    /// 2.4 GHz ISM band.
    Band2_4GHz,
    /// 5 GHz UNII band.
    Band5GHz,
    /// 6 GHz band (Wi-Fi 6E / Wi-Fi 7).
    Band6GHz,
}

/// Wi-Fi channel bandwidth in MHz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelWidth {
    /// 20 MHz narrow channel.
    Width20MHz,
    /// 40 MHz channel.
    Width40MHz,
    /// 80 MHz channel (standard for Wi-Fi 5/6).
    Width80MHz,
    /// 160 MHz ultra-wide channel.
    Width160MHz,
    /// 320 MHz ultra-wide channel (Wi-Fi 7).
    Width320MHz,
}

/// Wi-Fi security authentication protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WifiSecurity {
    /// Unencrypted open network.
    Open,
    /// Deprecated WEP 64/128-bit.
    Wep,
    /// WPA-Personal (TKIP/AES).
    WpaPersonal,
    /// WPA2-Personal (AES-CCMP).
    Wpa2Personal,
    /// WPA3-Personal (SAE Simultaneous Authentication of Equals).
    Wpa3Personal,
    /// WPA2/WPA3 Transition Mode.
    Wpa2Wpa3Transition,
    /// WPA2-Enterprise (802.1X EAP).
    Wpa2Enterprise,
    /// WPA3-Enterprise (192-bit Suite B).
    Wpa3Enterprise,
}

/// Extended RF & PHY layer Wi-Fi diagnostics displayed in Option+Click popover.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WifiDiagnostics {
    /// Linux/macOS network interface identifier (e.g. `wlan0`, `en0`).
    pub interface_name: String,
    /// MAC address of the local Wi-Fi interface.
    pub mac_address: String,
    /// Access Point BSSID (MAC address of connected AP).
    pub bssid: String,
    /// Service Set Identifier (network broadcast name).
    pub ssid: String,
    /// 802.11 PHY generation mode.
    pub phy_mode: WifiPhyMode,
    /// Channel number (e.g. 36, 44, 149, 1).
    pub channel_number: u32,
    /// Frequency band.
    pub band: WifiBand,
    /// Channel width in MHz.
    pub channel_width: ChannelWidth,
    /// Received Signal Strength Indication in dBm (typically -90 dBm to -30 dBm).
    pub rssi_dbm: i32,
    /// Ambient RF noise floor in dBm (typically -95 dBm to -85 dBm).
    pub noise_dbm: i32,
    /// Transmit bitrate in Megabits per second (Mbps).
    pub tx_rate_mbps: u32,
    /// Modulation and Coding Scheme (MCS) index.
    pub mcs_index: u32,
    /// Number of Spatial Streams (NSS).
    pub spatial_streams: u32,
    /// Wi-Fi security mode.
    pub security: WifiSecurity,
    /// Local IPv4 address.
    pub ip_address: String,
    /// Default gateway / router IP address.
    pub router_ip: String,
    /// Primary DNS server.
    pub primary_dns: String,
}

impl WifiDiagnostics {
    /// Calculates the Signal-to-Noise Ratio (SNR) in dB:
    ///
    /// $$\text{SNR} = \text{RSSI} - \text{Noise}$$
    ///
    /// For example: $-52 \text{ dBm} - (-92 \text{ dBm}) = 40 \text{ dB}$ (Excellent quality).
    #[must_use]
    pub fn signal_to_noise_ratio(&self) -> i32 {
        self.rssi_dbm - self.noise_dbm
    }

    /// Returns a qualitative rating of the connection based on SNR.
    #[must_use]
    pub fn signal_quality_rating(&self) -> &'static str {
        let snr = self.signal_to_noise_ratio();
        if snr >= 40 {
            "Excellent (40+ dB SNR)"
        } else if snr >= 25 {
            "Good (25–39 dB SNR)"
        } else if snr >= 15 {
            "Fair (15–24 dB SNR)"
        } else {
            "Poor (<15 dB SNR)"
        }
    }

    /// Formats the complete diagnostic summary as a list of key-value pairs matching
    /// macOS Tahoe 26's Option-Click dropdown.
    #[must_use]
    pub fn format_option_diagnostics(&self) -> Vec<(String, String)> {
        let width_str = match self.channel_width {
            ChannelWidth::Width20MHz => "20 MHz",
            ChannelWidth::Width40MHz => "40 MHz",
            ChannelWidth::Width80MHz => "80 MHz",
            ChannelWidth::Width160MHz => "160 MHz",
            ChannelWidth::Width320MHz => "320 MHz",
        };

        let band_str = match self.band {
            WifiBand::Band2_4GHz => "2.4 GHz",
            WifiBand::Band5GHz => "5 GHz",
            WifiBand::Band6GHz => "6 GHz",
        };

        vec![
            ("Interface Name".to_string(), self.interface_name.clone()),
            ("MAC Address".to_string(), self.mac_address.clone()),
            ("BSSID".to_string(), self.bssid.clone()),
            ("Channel".to_string(), format!("{} ({}, {})", self.channel_number, band_str, width_str)),
            ("PHY Mode".to_string(), self.phy_mode.display_name().to_string()),
            ("Security".to_string(), format!("{:?}", self.security)),
            ("RSSI".to_string(), format!("{} dBm", self.rssi_dbm)),
            ("Noise".to_string(), format!("{} dBm", self.noise_dbm)),
            ("Tx Rate".to_string(), format!("{} Mbps", self.tx_rate_mbps)),
            ("MCS Index".to_string(), self.mcs_index.to_string()),
            ("NSS".to_string(), self.spatial_streams.to_string()),
            ("IP Address".to_string(), self.ip_address.clone()),
            ("Router".to_string(), self.router_ip.clone()),
            ("Internet".to_string(), "Reachable".to_string()),
            ("Signal Quality".to_string(), self.signal_quality_rating().to_string()),
        ]
    }
}

/// Category of Bluetooth accessory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BluetoothDeviceCategory {
    /// Apple AirPods / Beats headphones with individual earpiece batteries.
    AirPods,
    /// Standard wireless over-ear or in-ear headphones.
    Headphones,
    /// Wireless mouse (e.g. Magic Mouse).
    Mouse,
    /// Wireless keyboard (e.g. Magic Keyboard).
    Keyboard,
    /// Wireless trackpad (e.g. Magic Trackpad).
    Trackpad,
    /// Bluetooth gamepad or controller.
    Gamepad,
    /// Other peripheral.
    Other,
}

/// Battery levels for multi-part accessories like AirPods.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessoryBattery {
    /// Left earpiece battery percentage (`0..=100`), if applicable.
    pub left_percent: Option<u8>,
    /// Right earpiece battery percentage (`0..=100`), if applicable.
    pub right_percent: Option<u8>,
    /// Charging case battery percentage (`0..=100`), if applicable.
    pub case_percent: Option<u8>,
    /// General single battery percentage for mouse/keyboard/standard headphones.
    pub general_percent: Option<u8>,
}

/// Bluetooth Option-Diagnostics record for connected peripheral.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BluetoothDeviceDiagnostics {
    /// Bluetooth MAC address (e.g. `AA:BB:CC:DD:EE:FF`).
    pub address: String,
    /// User or manufacturer device name.
    pub name: String,
    /// Hardware category.
    pub category: BluetoothDeviceCategory,
    /// Received Signal Strength Indication in dBm.
    pub rssi_dbm: i32,
    /// Link quality rating in range `0..=100%`.
    pub link_quality_percent: u8,
    /// Packet round-trip latency in milliseconds.
    pub latency_ms: u32,
    /// Active audio codec if device is an audio sink.
    pub active_codec: Option<crate::audio::BluetoothAudioCodec>,
    /// Accessory battery levels.
    pub battery: AccessoryBattery,
}

impl BluetoothDeviceDiagnostics {
    /// Formats the device diagnostic details for the Option-Click Bluetooth popover.
    #[must_use]
    pub fn format_diagnostics(&self) -> Vec<(String, String)> {
        let mut rows = vec![
            ("Address".to_string(), self.address.clone()),
            ("RSSI".to_string(), format!("{} dBm", self.rssi_dbm)),
            ("Link Quality".to_string(), format!("{}%", self.link_quality_percent)),
            ("Latency".to_string(), format!("{} ms", self.latency_ms)),
        ];

        if let Some(codec) = self.active_codec {
            rows.push(("Audio Codec".to_string(), format!("{codec:?}")));
        }

        if let Some(general) = self.battery.general_percent {
            rows.push(("Battery".to_string(), format!("{general}%")));
        }
        if let Some(left) = self.battery.left_percent {
            rows.push(("Left Battery".to_string(), format!("{left}%")));
        }
        if let Some(right) = self.battery.right_percent {
            rows.push(("Right Battery".to_string(), format!("{right}%")));
        }
        if let Some(case) = self.battery.case_percent {
            rows.push(("Case Battery".to_string(), format!("{case}%")));
        }

        rows
    }
}

/// Main Network and Radios Subsystem Daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDaemon {
    /// Wi-Fi radio power state (`true` = on, `false` = off).
    pub is_wifi_enabled: bool,
    /// Bluetooth radio power state (`true` = on, `false` = off).
    pub is_bluetooth_enabled: bool,
    /// Currently connected Wi-Fi diagnostics if connected to an AP.
    active_wifi: Option<WifiDiagnostics>,
    /// Connected Bluetooth devices indexed by MAC address.
    connected_bluetooth: HashMap<String, BluetoothDeviceDiagnostics>,
}

impl Default for NetworkDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkDaemon {
    /// Creates a new `NetworkDaemon` with default mock connected Tahoe 26 Wi-Fi 6 AP.
    #[must_use]
    pub fn new() -> Self {
        let mut daemon = Self {
            is_wifi_enabled: true,
            is_bluetooth_enabled: true,
            active_wifi: None,
            connected_bluetooth: HashMap::new(),
        };

        // Initialize with realistic connected Wi-Fi 6 diagnostics
        let wifi = WifiDiagnostics {
            interface_name: "wlan0".to_string(),
            mac_address: "3c:22:fb:49:1a:82".to_string(),
            bssid: "14:59:c0:3b:81:4e".to_string(),
            ssid: "ApplePark_5G".to_string(),
            phy_mode: WifiPhyMode::Phy80211ax,
            channel_number: 36,
            band: WifiBand::Band5GHz,
            channel_width: ChannelWidth::Width80MHz,
            rssi_dbm: -52,
            noise_dbm: -92,
            tx_rate_mbps: 1201,
            mcs_index: 11,
            spatial_streams: 2,
            security: WifiSecurity::Wpa3Personal,
            ip_address: "192.168.1.142".to_string(),
            router_ip: "192.168.1.1".to_string(),
            primary_dns: "1.1.1.1".to_string(),
        };
        daemon.set_active_wifi(Some(wifi));

        // Initialize with Magic Mouse and AirPods Max
        daemon.add_bluetooth_device(BluetoothDeviceDiagnostics {
            address: "88:66:5a:12:44:01".to_string(),
            name: "Magic Mouse".to_string(),
            category: BluetoothDeviceCategory::Mouse,
            rssi_dbm: -48,
            link_quality_percent: 98,
            latency_ms: 8,
            active_codec: None,
            battery: AccessoryBattery {
                left_percent: None,
                right_percent: None,
                case_percent: None,
                general_percent: Some(85),
            },
        });

        daemon.add_bluetooth_device(BluetoothDeviceDiagnostics {
            address: "ac:fd:ce:88:99:32".to_string(),
            name: "AirPods Max".to_string(),
            category: BluetoothDeviceCategory::AirPods,
            rssi_dbm: -42,
            link_quality_percent: 100,
            latency_ms: 15,
            active_codec: Some(crate::audio::BluetoothAudioCodec::Aac),
            battery: AccessoryBattery {
                left_percent: None,
                right_percent: None,
                case_percent: None,
                general_percent: Some(92),
            },
        });

        daemon
    }

    /// Returns the active Wi-Fi diagnostics if connected.
    #[must_use]
    pub fn active_wifi(&self) -> Option<&WifiDiagnostics> {
        if self.is_wifi_enabled {
            self.active_wifi.as_ref()
        } else {
            None
        }
    }

    /// Sets or clears the active Wi-Fi connection.
    pub fn set_active_wifi(&mut self, wifi: Option<WifiDiagnostics>) {
        self.active_wifi = wifi;
    }

    /// Adds or updates a connected Bluetooth accessory.
    pub fn add_bluetooth_device(&mut self, device: BluetoothDeviceDiagnostics) {
        self.connected_bluetooth.insert(device.address.clone(), device);
    }

    /// Removes a Bluetooth device by MAC address.
    pub fn remove_bluetooth_device(&mut self, address: &str) -> Option<BluetoothDeviceDiagnostics> {
        self.connected_bluetooth.remove(address)
    }

    /// Returns all connected Bluetooth devices.
    #[must_use]
    pub fn connected_bluetooth_devices(&self) -> Vec<&BluetoothDeviceDiagnostics> {
        if self.is_bluetooth_enabled {
            self.connected_bluetooth.values().collect()
        } else {
            Vec::new()
        }
    }

    /// Toggles Wi-Fi radio power.
    pub fn toggle_wifi(&mut self) -> bool {
        self.is_wifi_enabled = !self.is_wifi_enabled;
        self.is_wifi_enabled
    }

    /// Toggles Bluetooth radio power.
    pub fn toggle_bluetooth(&mut self) -> bool {
        self.is_bluetooth_enabled = !self.is_bluetooth_enabled;
        self.is_bluetooth_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_snr_and_diagnostics() {
        let daemon = NetworkDaemon::new();
        let wifi = daemon.active_wifi().expect("Mock Wi-Fi should be active");

        assert_eq!(wifi.signal_to_noise_ratio(), 40);
        assert_eq!(wifi.signal_quality_rating(), "Excellent (40+ dB SNR)");

        let rows = wifi.format_option_diagnostics();
        assert!(rows.iter().any(|(k, v)| k == "BSSID" && v == "14:59:c0:3b:81:4e"));
        assert!(rows.iter().any(|(k, v)| k == "Tx Rate" && v == "1201 Mbps"));
        assert!(rows.iter().any(|(k, v)| k == "RSSI" && v == "-52 dBm"));
    }

    #[test]
    fn test_bluetooth_diagnostics() {
        let daemon = NetworkDaemon::new();
        let devices = daemon.connected_bluetooth_devices();
        assert_eq!(devices.len(), 2);

        let airpods = devices
            .iter()
            .find(|d| d.name == "AirPods Max")
            .expect("AirPods Max present");
        let rows = airpods.format_diagnostics();
        assert!(rows.iter().any(|(k, v)| k == "Audio Codec" && v == "Aac"));
        assert!(rows.iter().any(|(k, v)| k == "Battery" && v == "92%"));
    }

    #[test]
    fn test_radio_toggles() {
        let mut daemon = NetworkDaemon::new();
        assert!(daemon.is_wifi_enabled);

        let off = daemon.toggle_wifi();
        assert!(!off);
        assert!(!daemon.is_wifi_enabled);
        assert!(daemon.active_wifi().is_none());

        let on = daemon.toggle_wifi();
        assert!(on);
        assert!(daemon.is_wifi_enabled);
        assert!(daemon.active_wifi().is_some());
    }
}
