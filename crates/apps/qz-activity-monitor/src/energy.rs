//! # Energy Impact & Battery Telemetry Engine (`qz-activity-monitor::energy`)
//!
//! Production-grade energy monitoring for macOS Tahoe 26 Activity Monitor.
//!
//! ### Key Capabilities:
//! - **Instantaneous Energy Impact Score:** Analytical formulation:
//!   $$\text{Score} = 1.0 \times \text{CPU}_{\%} + 1.5 \times \text{GPU}_{\%} + 0.05 \times \text{Wakeups}_{\text{sec}}$$
//! - **12-Hour Power Moving Average (EWMA):** Long-term historical energy tracking.
//! - **App Nap State Detection:** Identifies apps running in low-power occluded coalescing mode.
//! - **Battery & Power Adapter Telemetry:** Battery percentage, power source, charging status,
//!   and estimated time to empty/full.

use serde::{Deserialize, Serialize};

/// Power source status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PowerSource {
    /// Running on internal chemical battery.
    Battery,
    /// Connected to external AC / MagSafe / USB-C power adapter.
    PowerAdapter,
}

/// Instantaneous battery and system power state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PowerTelemetry {
    /// Active power delivery source.
    pub source: PowerSource,
    /// Current battery charge percentage ($0.0\%$ to $100.0\%$).
    pub battery_percent: f32,
    /// `true` if battery is actively receiving charge from adapter.
    pub is_charging: bool,
    /// `true` if battery has finished charging to $100\%$ (or $80\%$ in optimized charging).
    pub is_fully_charged: bool,
    /// Estimated time remaining in minutes (to empty if on battery, to full if charging).
    pub time_remaining_minutes: Option<u32>,
    /// Accumulated battery discharge cycles.
    pub cycle_count: u32,
    /// Maximum battery capacity health percentage compared to factory design.
    pub health_percent: f32,
}

impl PowerTelemetry {
    /// Formats time remaining into a user-friendly string (e.g. "4:32 remaining" or "Calculating...").
    #[must_use]
    pub fn formatted_time_remaining(&self) -> String {
        if let Some(mins) = self.time_remaining_minutes {
            let h = mins / 60;
            let m = mins % 60;
            if self.is_charging {
                format!("{h}:{m:02} until full")
            } else {
                format!("{h}:{m:02} remaining")
            }
        } else {
            "Calculating…".to_string()
        }
    }
}

/// Computes the standard macOS Tahoe 26 instantaneous energy impact score for a process.
///
/// Formula:
/// $$\text{Energy Impact} = 1.0 \times \text{CPU}_{\%} + 1.5 \times \text{GPU}_{\%} + 0.05 \times \text{IdleWakeups}$$
#[must_use]
pub fn compute_energy_impact(cpu_percent: f32, gpu_percent: f32, idle_wakeups_per_sec: u32) -> f32 {
    let score = (1.0 * cpu_percent) + (1.5 * gpu_percent) + (0.05 * idle_wakeups_per_sec as f32);
    score.max(0.0)
}

/// Updates 12-hour power moving average using Exponentially Weighted Moving Average (EWMA).
///
/// Formula:
/// $$P_{t} = \lambda P_{\text{instant}} + (1 - \lambda) P_{t-1}$$
#[must_use]
pub fn update_12hr_power_ewma(previous_power: f32, current_impact: f32, lambda: f32) -> f32 {
    let l = lambda.clamp(0.001, 1.0);
    l * current_impact + (1.0 - l) * previous_power
}

/// Evaluates whether a process qualifies as a "Significant Energy Consumer"
/// (used in battery menu bar extras and warnings).
#[must_use]
pub fn is_high_energy_consumer(energy_impact: f32, power_12hr: f32) -> bool {
    energy_impact > 20.0 || power_12hr > 15.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_impact_calculation() {
        // High CPU (50%) + Moderate GPU (20%) + 100 wakeups
        // 50*1.0 + 20*1.5 + 100*0.05 = 50 + 30 + 5 = 85.0
        let impact = compute_energy_impact(50.0, 20.0, 100);
        assert!((impact - 85.0).abs() < 1e-4);
        assert!(is_high_energy_consumer(impact, 0.0));

        // Idle process: 0.1% CPU, 0% GPU, 2 wakeups
        // 0.1 + 0 + 0.1 = 0.2
        let idle_impact = compute_energy_impact(0.1, 0.0, 2);
        assert!(idle_impact < 1.0);
        assert!(!is_high_energy_consumer(idle_impact, 0.5));
    }

    #[test]
    fn test_ewma_power_update() {
        let p0 = 10.0;
        let impact = 20.0;
        let lambda = 0.1;
        let p1 = update_12hr_power_ewma(p0, impact, lambda);
        // 0.1 * 20 + 0.9 * 10 = 2 + 9 = 11.0
        assert!((p1 - 11.0).abs() < 1e-4);
    }

    #[test]
    fn test_power_telemetry_formatting() {
        let battery = PowerTelemetry {
            source: PowerSource::Battery,
            battery_percent: 85.0,
            is_charging: false,
            is_fully_charged: false,
            time_remaining_minutes: Some(272), // 4h 32m
            cycle_count: 42,
            health_percent: 99.0,
        };
        assert_eq!(battery.formatted_time_remaining(), "4:32 remaining");

        let charging = PowerTelemetry {
            source: PowerSource::PowerAdapter,
            battery_percent: 75.0,
            is_charging: true,
            is_fully_charged: false,
            time_remaining_minutes: Some(45), // 0h 45m
            cycle_count: 42,
            health_percent: 99.0,
        };
        assert_eq!(charging.formatted_time_remaining(), "0:45 until full");
    }
}
