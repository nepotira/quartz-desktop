//! # Disk & Network I/O Telemetry Subsystems (`qz-activity-monitor::disk_network`)
//!
//! Production-grade storage and network analytics for macOS Tahoe 26 Activity Monitor.
//!
//! ### Key Capabilities:
//! - **Disk I/O Subsystem:** Read/Write throughput rates ($\text{bytes/sec}$), total bytes read/written,
//!   and IOPS operation counts.
//! - **Network I/O Subsystem:** Inbound/Outbound data rates, total data transferred, and packet rates.
//! - **Rolling Time Series Buffers:** Ring buffers storing traffic samples for live Area/Bar charts.

use serde::{Deserialize, Serialize};

/// Formats a data rate in bytes per second into human-readable macOS strings (e.g. "12.4 MB/s").
#[must_use]
pub fn format_rate(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{bytes_per_sec} B/s")
    }
}

/// Instantaneous Disk I/O telemetry snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskSnapshot {
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Instantaneous read throughput in bytes/second.
    pub read_bytes_sec: u64,
    /// Instantaneous write throughput in bytes/second.
    pub write_bytes_sec: u64,
    /// Total read operations since boot.
    pub reads_in: u64,
    /// Total write operations since boot.
    pub writes_out: u64,
    /// Cumulative data read since boot in bytes.
    pub data_read_total: u64,
    /// Cumulative data written since boot in bytes.
    pub data_written_total: u64,
}

/// Instantaneous Network I/O telemetry snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    /// Timestamp in seconds.
    pub timestamp: f64,
    /// Packets received per second.
    pub packets_in_sec: u64,
    /// Packets transmitted per second.
    pub packets_out_sec: u64,
    /// Download rate in bytes/second.
    pub data_received_sec: u64,
    /// Upload rate in bytes/second.
    pub data_sent_sec: u64,
    /// Total data received since boot in bytes.
    pub data_received_total: u64,
    /// Total data sent since boot in bytes.
    pub data_sent_total: u64,
}

/// A sample stored in the I/O throughput history buffer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IoThroughputSample {
    /// Timestamp of sample.
    pub timestamp: f64,
    /// Inbound rate (Disk Read / Network In) in bytes/second.
    pub inbound_rate: u64,
    /// Outbound rate (Disk Write / Network Out) in bytes/second.
    pub outbound_rate: u64,
}

/// Ring buffer recording real-time throughput for Disk or Network charts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoHistory {
    samples: Vec<IoThroughputSample>,
    capacity: usize,
}

impl Default for IoHistory {
    fn default() -> Self {
        Self::new(120)
    }
}

impl IoHistory {
    /// Creates a new `IoHistory` buffer with specified capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: Vec::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Records a disk snapshot.
    pub fn record_disk(&mut self, snapshot: &DiskSnapshot) {
        self.record_sample(IoThroughputSample {
            timestamp: snapshot.timestamp,
            inbound_rate: snapshot.read_bytes_sec,
            outbound_rate: snapshot.write_bytes_sec,
        });
    }

    /// Records a network snapshot.
    pub fn record_network(&mut self, snapshot: &NetworkSnapshot) {
        self.record_sample(IoThroughputSample {
            timestamp: snapshot.timestamp,
            inbound_rate: snapshot.data_received_sec,
            outbound_rate: snapshot.data_sent_sec,
        });
    }

    fn record_sample(&mut self, sample: IoThroughputSample) {
        if self.samples.len() >= self.capacity {
            self.samples.remove(0);
        }
        self.samples.push(sample);
    }

    /// Returns all recorded samples.
    #[must_use]
    pub fn samples(&self) -> &[IoThroughputSample] {
        &self.samples
    }

    /// Number of samples stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns `true` if buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_rate() {
        assert_eq!(format_rate(500), "500 B/s");
        assert_eq!(format_rate(1024 * 64), "64 KB/s");
        assert_eq!(format_rate(1024 * 1024 * 45), "45.0 MB/s");
        assert_eq!(format_rate(1024 * 1024 * 1024 * 2), "2.00 GB/s");
    }

    #[test]
    fn test_disk_and_network_history_ring_buffer() {
        let mut disk_hist = IoHistory::new(2);
        let disk_snap = DiskSnapshot {
            timestamp: 100.0,
            read_bytes_sec: 1024 * 1024 * 10,
            write_bytes_sec: 1024 * 1024 * 5,
            reads_in: 12000,
            writes_out: 8000,
            data_read_total: 1024 * 1024 * 1024 * 100,
            data_written_total: 1024 * 1024 * 1024 * 50,
        };

        disk_hist.record_disk(&disk_snap);
        disk_hist.record_disk(&disk_snap);
        assert_eq!(disk_hist.len(), 2);
        disk_hist.record_disk(&disk_snap);
        assert_eq!(disk_hist.len(), 2);

        let mut net_hist = IoHistory::new(2);
        let net_snap = NetworkSnapshot {
            timestamp: 100.0,
            packets_in_sec: 250,
            packets_out_sec: 180,
            data_received_sec: 1024 * 500,
            data_sent_sec: 1024 * 120,
            data_received_total: 1024 * 1024 * 1024,
            data_sent_total: 1024 * 1024 * 500,
        };
        net_hist.record_network(&net_snap);
        assert_eq!(net_hist.len(), 1);
        assert_eq!(net_hist.samples()[0].inbound_rate, 1024 * 500);
    }
}
