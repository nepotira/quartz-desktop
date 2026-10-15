//! # CPU Telemetry & Multi-Core Load History Engine (`qz-activity-monitor::cpu`)
//!
//! Production-grade CPU metrics for macOS Tahoe 26 Activity Monitor.
//!
//! ### Key Capabilities:
//! - **CPU Utilization Breakdown:** System-wide User % (`#007AFF` Blue), System % (`#FF3B30` Red),
//!   and Idle %.
//! - **Heterogeneous Core Architecture:** Dedicated tracking for Performance cores (P-cores)
//!   and Efficiency cores (E-cores) on Apple Silicon / modern ARM/x86 processors.
//! - **Kernel Load Averages:** 1-minute, 5-minute, and 15-minute load averages ($L_1, L_5, L_{15}$).
//! - **Rolling History Ring Buffer:** Records time-series samples for CPU history charts.

use serde::{Deserialize, Serialize};

/// Classification of processor cores in modern heterogeneous CPUs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreKind {
    /// High-performance compute core (P-core).
    Performance,
    /// Energy-efficient background core (E-core).
    Efficiency,
    /// Standard symmetric core.
    Standard,
}

/// Instantaneous utilization of an individual CPU core.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreUtilization {
    /// 0-indexed core index.
    pub core_id: u32,
    /// Type of core.
    pub kind: CoreKind,
    /// Current utilization percentage ($0.0\%$ to $100.0\%$).
    pub percent: f32,
}

/// System-wide CPU utilization snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuSnapshot {
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Percentage spent in user-space execution ($0.0$ to $100.0$).
    pub user_percent: f32,
    /// Percentage spent in kernel / system execution ($0.0$ to $100.0$).
    pub system_percent: f32,
    /// Kernel 1-minute, 5-minute, and 15-minute load averages.
    pub load_averages: (f32, f32, f32),
    /// Per-core utilization list.
    pub cores: Vec<CoreUtilization>,
    /// Total system threads running.
    pub total_threads: u32,
    /// Total system processes running.
    pub total_processes: u32,
}

impl CpuSnapshot {
    /// Computes percentage of CPU capacity currently idle.
    #[must_use]
    pub fn idle_percent(&self) -> f32 {
        (100.0 - self.user_percent - self.system_percent).max(0.0)
    }

    /// Average utilization across all Performance cores.
    #[must_use]
    pub fn p_cores_average(&self) -> f32 {
        let p_cores: Vec<_> = self.cores.iter().filter(|c| c.kind == CoreKind::Performance).collect();
        if p_cores.is_empty() {
            0.0
        } else {
            let sum: f32 = p_cores.iter().map(|c| c.percent).sum();
            sum / p_cores.len() as f32
        }
    }

    /// Average utilization across all Efficiency cores.
    #[must_use]
    pub fn e_cores_average(&self) -> f32 {
        let e_cores: Vec<_> = self.cores.iter().filter(|c| c.kind == CoreKind::Efficiency).collect();
        if e_cores.is_empty() {
            0.0
        } else {
            let sum: f32 = e_cores.iter().map(|c| c.percent).sum();
            sum / e_cores.len() as f32
        }
    }
}

/// A sample stored in the CPU history ring buffer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CpuHistorySample {
    /// Timestamp of sample.
    pub timestamp: f64,
    /// User CPU percentage.
    pub user_percent: f32,
    /// System CPU percentage.
    pub system_percent: f32,
    /// Idle CPU percentage.
    pub idle_percent: f32,
}

/// Rolling history ring buffer for CPU time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuHistory {
    samples: Vec<CpuHistorySample>,
    capacity: usize,
}

impl Default for CpuHistory {
    fn default() -> Self {
        Self::new(120) // 120 samples default
    }
}

impl CpuHistory {
    /// Creates a new `CpuHistory` with the specified capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Records a new CPU snapshot into history.
    pub fn record_snapshot(&mut self, snapshot: &CpuSnapshot) {
        let sample = CpuHistorySample {
            timestamp: snapshot.timestamp,
            user_percent: snapshot.user_percent,
            system_percent: snapshot.system_percent,
            idle_percent: snapshot.idle_percent(),
        };

        if self.samples.len() >= self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    /// Returns all recorded samples.
    #[must_use]
    pub fn samples(&self) -> &[CpuHistorySample] {
        &self.samples
    }

    /// Number of recorded samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_snapshot_idle_and_core_averages() {
        let cores = vec![
            CoreUtilization { core_id: 0, kind: CoreKind::Performance, percent: 80.0 },
            CoreUtilization { core_id: 1, kind: CoreKind::Performance, percent: 60.0 },
            CoreUtilization { core_id: 2, kind: CoreKind::Efficiency, percent: 20.0 },
            CoreUtilization { core_id: 3, kind: CoreKind::Efficiency, percent: 10.0 },
        ];

        let snapshot = CpuSnapshot {
            timestamp: 1000.0,
            user_percent: 45.0,
            system_percent: 15.0,
            load_averages: (2.1, 1.8, 1.4),
            cores,
            total_threads: 1840,
            total_processes: 412,
        };

        assert_eq!(snapshot.idle_percent(), 40.0);
        assert_eq!(snapshot.p_cores_average(), 70.0);
        assert_eq!(snapshot.e_cores_average(), 15.0);
    }

    #[test]
    fn test_cpu_history_ring_buffer() {
        let mut history = CpuHistory::new(2);
        assert_eq!(history.len(), 0);

        let snap = CpuSnapshot {
            timestamp: 100.0,
            user_percent: 10.0,
            system_percent: 5.0,
            load_averages: (1.0, 1.0, 1.0),
            cores: Vec::new(),
            total_threads: 100,
            total_processes: 50,
        };

        history.record_snapshot(&snap);
        history.record_snapshot(&snap);
        assert_eq!(history.len(), 2);

        history.record_snapshot(&snap);
        assert_eq!(history.len(), 2);
    }
}
