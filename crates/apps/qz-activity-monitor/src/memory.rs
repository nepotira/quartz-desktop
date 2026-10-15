//! # Memory Pressure Graph & Telemetry Engine (`qz-activity-monitor::memory`)
//!
//! Production-grade memory analytics for macOS Tahoe 26 Activity Monitor.
//!
//! ### Key Capabilities:
//! - **Tahoe 26 Memory Pressure States:** Classified into `Normal` (Green, $<60\%$),
//!   `Warn` (Yellow, $60\%..=80\%$), and `Critical` (Red, $>80\%$).
//! - **Detailed Memory Breakdown:** Physical Memory, App Memory, Wired Memory, Compressed Memory,
//!   Cached Files, and Swap Space used.
//! - **Rolling Area Graph History:** Ring buffer recording up to 120 pressure history samples
//!   for rendering the real-time continuous Bezier/spline graph.

use serde::{Deserialize, Serialize};

/// Memory pressure state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryPressureState {
    /// Normal operating state (Green) — memory is plentiful, minimal pageout.
    Normal,
    /// Warning state (Yellow) — compression active, inactive memory reclaimed.
    Warn,
    /// Critical state (Red) — high paging to disk, potential allocation stalling.
    Critical,
}

impl MemoryPressureState {
    /// Returns the RGBA color associated with the pressure level.
    #[must_use]
    pub fn color_rgba(&self) -> [f32; 4] {
        match self {
            Self::Normal => [0.20, 0.78, 0.35, 1.0], // Cupertino Green #34C759
            Self::Warn => [1.0, 0.58, 0.0, 1.0],     // Cupertino Orange #FF9500
            Self::Critical => [1.0, 0.23, 0.19, 1.0], // Cupertino Red #FF3B30
        }
    }
}

/// A snapshot of system memory usage at a given instant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Total physical RAM installed in bytes.
    pub physical_memory_bytes: u64,
    /// Memory directly allocated by user applications in bytes.
    pub app_memory_bytes: u64,
    /// Kernel and non-pageable driver memory in bytes.
    pub wired_memory_bytes: u64,
    /// Compressed memory footprint in RAM in bytes.
    pub compressed_bytes: u64,
    /// File system cache available for immediate reclamation in bytes.
    pub cached_files_bytes: u64,
    /// Swap space used on disk/SSD in bytes.
    pub swap_used_bytes: u64,
    /// Total swap space available in bytes.
    pub swap_total_bytes: u64,
}

impl MemorySnapshot {
    /// Calculates total memory used ($\text{App} + \text{Wired} + \text{Compressed}$).
    #[must_use]
    pub fn memory_used_bytes(&self) -> u64 {
        self.app_memory_bytes + self.wired_memory_bytes + self.compressed_bytes
    }

    /// Calculates memory pressure percentage ($0.0\%$ to $100.0\%$).
    ///
    /// The pressure formula considers active allocations, compressed memory ratio,
    /// and swap activity relative to physical capacity:
    /// $$\text{Pressure} = \frac{\text{Used} + 1.5 \times \text{SwapUsed}}{\text{Physical}} \times 100.0$$
    #[must_use]
    pub fn pressure_percent(&self) -> f32 {
        if self.physical_memory_bytes == 0 {
            return 0.0;
        }
        let effective_burden = self.memory_used_bytes() as f64 + (self.swap_used_bytes as f64 * 1.5);
        let ratio = (effective_burden / self.physical_memory_bytes as f64) * 100.0;
        ratio.clamp(0.0, 100.0) as f32
    }

    /// Evaluates the current `MemoryPressureState`.
    #[must_use]
    pub fn pressure_state(&self) -> MemoryPressureState {
        let p = self.pressure_percent();
        if p >= 80.0 {
            MemoryPressureState::Critical
        } else if p >= 60.0 {
            MemoryPressureState::Warn
        } else {
            MemoryPressureState::Normal
        }
    }
}

/// Formats a byte quantity into a human-readable macOS string (e.g. "64.00 GB", "512.0 MB").
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// A single sample point in the memory pressure history ring buffer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryPressureSample {
    /// Timestamp of sample.
    pub timestamp: f64,
    /// Pressure percentage ($0.0$ to $100.0$).
    pub pressure_percent: f32,
    /// Operating state.
    pub state: MemoryPressureState,
}

/// Ring buffer recording real-time memory pressure for UI graph rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPressureHistory {
    samples: Vec<MemoryPressureSample>,
    capacity: usize,
}

impl Default for MemoryPressureHistory {
    fn default() -> Self {
        Self::new(120) // 120 samples default (e.g. 2 minutes at 1 sample/sec)
    }
}

impl MemoryPressureHistory {
    /// Creates a new `MemoryPressureHistory` ring buffer with the given capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Records a new memory snapshot into the history.
    pub fn record_snapshot(&mut self, snapshot: &MemorySnapshot) {
        let sample = MemoryPressureSample {
            timestamp: snapshot.timestamp,
            pressure_percent: snapshot.pressure_percent(),
            state: snapshot.pressure_state(),
        };

        if self.samples.len() >= self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    /// Returns all recorded samples in chronological order.
    #[must_use]
    pub fn samples(&self) -> &[MemoryPressureSample] {
        &self.samples
    }

    /// Clears all history samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Number of recorded samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` if no samples are recorded.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pressure_state_calculation() {
        let total = 64 * 1024 * 1024 * 1024; // 64 GB

        // Normal pressure (30 GB used, 0 swap) -> ~46%
        let normal = MemorySnapshot {
            timestamp: 1000.0,
            physical_memory_bytes: total,
            app_memory_bytes: 20 * 1024 * 1024 * 1024,
            wired_memory_bytes: 8 * 1024 * 1024 * 1024,
            compressed_bytes: 2 * 1024 * 1024 * 1024,
            cached_files_bytes: 14 * 1024 * 1024 * 1024,
            swap_used_bytes: 0,
            swap_total_bytes: 8 * 1024 * 1024 * 1024,
        };
        assert_eq!(normal.pressure_state(), MemoryPressureState::Normal);
        assert!(normal.pressure_percent() < 60.0);

        // Warning pressure (45 GB used, 0 swap) -> ~70%
        let warn = MemorySnapshot {
            timestamp: 1001.0,
            physical_memory_bytes: total,
            app_memory_bytes: 35 * 1024 * 1024 * 1024,
            wired_memory_bytes: 8 * 1024 * 1024 * 1024,
            compressed_bytes: 2 * 1024 * 1024 * 1024,
            cached_files_bytes: 4 * 1024 * 1024 * 1024,
            swap_used_bytes: 0,
            swap_total_bytes: 8 * 1024 * 1024 * 1024,
        };
        assert_eq!(warn.pressure_state(), MemoryPressureState::Warn);

        // Critical pressure (55 GB used + 8 GB swap) -> >90%
        let crit = MemorySnapshot {
            timestamp: 1002.0,
            physical_memory_bytes: total,
            app_memory_bytes: 45 * 1024 * 1024 * 1024,
            wired_memory_bytes: 8 * 1024 * 1024 * 1024,
            compressed_bytes: 2 * 1024 * 1024 * 1024,
            cached_files_bytes: 1 * 1024 * 1024 * 1024,
            swap_used_bytes: 8 * 1024 * 1024 * 1024,
            swap_total_bytes: 16 * 1024 * 1024 * 1024,
        };
        assert_eq!(crit.pressure_state(), MemoryPressureState::Critical);
    }

    #[test]
    fn test_memory_pressure_ring_buffer() {
        let mut history = MemoryPressureHistory::new(3);
        assert_eq!(history.len(), 0);

        let snap = MemorySnapshot {
            timestamp: 1000.0,
            physical_memory_bytes: 16 * 1024 * 1024 * 1024,
            app_memory_bytes: 4 * 1024 * 1024 * 1024,
            wired_memory_bytes: 2 * 1024 * 1024 * 1024,
            compressed_bytes: 0,
            cached_files_bytes: 4 * 1024 * 1024 * 1024,
            swap_used_bytes: 0,
            swap_total_bytes: 4 * 1024 * 1024 * 1024,
        };

        history.record_snapshot(&snap);
        history.record_snapshot(&snap);
        history.record_snapshot(&snap);
        assert_eq!(history.len(), 3);

        // Recording 4th should discard oldest and keep len at 3
        history.record_snapshot(&snap);
        assert_eq!(history.len(), 3);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024 * 512), "512 KB");
        assert_eq!(format_bytes(1024 * 1024 * 256), "256.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024 * 64), "64.00 GB");
    }
}
