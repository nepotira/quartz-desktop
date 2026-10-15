//! # General & About This Mac Settings Panel (`qz-settings::panels::general`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 General, About This Mac,
//! hardware specs, storage breakdown, and AirDrop.
//!
//! ### Key Capabilities:
//! - **About This Mac System Telemetry:** Chip specs, unified memory, OS version and build number.
//! - **Storage Allocation Breakdown:** Segmented bar visualization of disk utilization.
//! - **AirDrop & Handoff Policies:** Visibility configuration and cross-device clipboard sync.

use serde::{Deserialize, Serialize};

/// AirDrop discovery visibility mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AirDropVisibility {
    /// Inactive; invisible to nearby devices.
    NoOne,
    /// Discoverable only to contacts in user's address book.
    ContactsOnly,
    /// Discoverable to all nearby devices for 10 minutes.
    EveryoneFor10Minutes,
}

/// Storage volume category distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageBreakdown {
    /// Total drive capacity in Gigabytes.
    pub total_capacity_gb: f64,
    /// Available free space in Gigabytes.
    pub available_gb: f64,
    /// Space occupied by Applications in Gigabytes.
    pub apps_gb: f64,
    /// Space occupied by Developer SDKs and Rust targets in Gigabytes.
    pub developer_gb: f64,
    /// Space occupied by System / macOS Tahoe OS files in Gigabytes.
    pub system_gb: f64,
    /// Space occupied by User Documents and Media in Gigabytes.
    pub documents_gb: f64,
}

impl StorageBreakdown {
    /// Percentage of storage currently utilized.
    #[must_use]
    pub fn used_percentage(&self) -> f64 {
        if self.total_capacity_gb <= 0.0 {
            0.0
        } else {
            let used = self.total_capacity_gb - self.available_gb;
            (used / self.total_capacity_gb * 100.0).clamp(0.0, 100.0)
        }
    }
}

/// General settings and About This Mac specs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// Hardware product marketing name.
    pub device_name: String,
    /// Processor / SoC description.
    pub chip_name: String,
    /// Unified memory in Gigabytes.
    pub memory_gb: u32,
    /// Hardware serial number.
    pub serial_number: String,
    /// Operating system marketing name.
    pub os_name: String,
    /// Operating system version string.
    pub os_version: String,
    /// System build number.
    pub build_number: String,
    /// Storage drive telemetry.
    pub storage: StorageBreakdown,
    /// AirDrop discoverability.
    pub airdrop_visibility: AirDropVisibility,
    /// Handoff cross-device continuity enabled.
    pub handoff_enabled: bool,
    /// Automatically keep Mac updated.
    pub auto_software_update: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            device_name: "MacBook Pro (16 polegadas, 2026)".to_string(),
            chip_name: "Apple M4 Max (16-core CPU, 40-core GPU)".to_string(),
            memory_gb: 64,
            serial_number: "C02G8842MD6R".to_string(),
            os_name: "macOS Tahoe".to_string(),
            os_version: "26.0".to_string(),
            build_number: "26A5284g".to_string(),
            storage: StorageBreakdown {
                total_capacity_gb: 2048.0,
                available_gb: 1420.0,
                apps_gb: 180.0,
                developer_gb: 145.0,
                system_gb: 48.0,
                documents_gb: 255.0,
            },
            airdrop_visibility: AirDropVisibility::ContactsOnly,
            handoff_enabled: true,
            auto_software_update: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_math() {
        let general = GeneralSettings::default();
        let used_pct = general.storage.used_percentage();
        // 2048 - 1420 = 628 GB used. 628 / 2048 * 100 ~= 30.66%
        assert!((used_pct - 30.66).abs() < 0.1);
    }

    #[test]
    fn test_system_identifiers() {
        let general = GeneralSettings::default();
        assert_eq!(general.os_name, "macOS Tahoe");
        assert_eq!(general.os_version, "26.0");
        assert_eq!(general.memory_gb, 64);
    }
}
