//! # Battery & Energy Saver Settings Panel (`qz-settings::panels::battery`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 Battery, health metrics,
//! Low Power Mode scheduling, and display sleep timeouts.
//!
//! ### Key Capabilities:
//! - **Low Power Mode Scheduling:** Automatic ProMotion 60Hz throttling based on power source.
//! - **Battery Health Telemetry:** Maximum capacity percentage, cycle count, and service alerts.
//! - **Optimized Battery Charging:** Preserves battery longevity by delaying full 100% charge.

use serde::{Deserialize, Serialize};

/// Policy for automatic Low Power Mode activation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LowPowerModePolicy {
    /// Low Power Mode is never enabled automatically.
    Never,
    /// Always enabled regardless of power source.
    Always,
    /// Enabled only when running on internal battery power.
    OnlyOnBattery,
    /// Enabled only when connected to power adapter.
    OnlyOnPowerAdapter,
}

impl LowPowerModePolicy {
    /// Label for UI selector.
    #[must_use]
    pub fn display_label(&self) -> &'static str {
        match self {
            Self::Never => "Nunca",
            Self::Always => "Sempre",
            Self::OnlyOnBattery => "Apenas na Bateria",
            Self::OnlyOnPowerAdapter => "Apenas no Adaptador de Alimentação",
        }
    }
}

/// Battery and energy saving preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BatterySettings {
    /// Low Power Mode scheduling policy.
    pub low_power_mode_policy: LowPowerModePolicy,
    /// Maximum capacity health percentage relative to design (e.g. 98.0%).
    pub health_percentage: f64,
    /// Lifetime charge cycle count.
    pub cycle_count: u32,
    /// Optimized Battery Charging enabled (pauses charge at 80% until needed).
    pub optimized_battery_charging: bool,
    /// Minutes of idle inactivity before turning display off.
    pub display_sleep_minutes: u32,
    /// Slightly dim display when switching to battery power.
    pub slightly_dim_on_battery: bool,
    /// Prevent automatic sleeping when display is turned off on AC power.
    pub prevent_sleep_on_ac: bool,
}

impl Default for BatterySettings {
    fn default() -> Self {
        Self {
            low_power_mode_policy: LowPowerModePolicy::OnlyOnBattery,
            health_percentage: 98.0,
            cycle_count: 72,
            optimized_battery_charging: true,
            display_sleep_minutes: 15,
            slightly_dim_on_battery: true,
            prevent_sleep_on_ac: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_battery_defaults() {
        let settings = BatterySettings::default();
        assert_eq!(settings.low_power_mode_policy, LowPowerModePolicy::OnlyOnBattery);
        assert_eq!(settings.health_percentage, 98.0);
        assert!(settings.optimized_battery_charging);
        assert_eq!(settings.display_sleep_minutes, 15);
    }
}
