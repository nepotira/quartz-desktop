//! # QuartzLinux Activity Monitor Application (`qz-activity-monitor`)
//!
//! Production-grade Activity Monitor desktop application for QuartzLinux (`qz-desktop`),
//! built to be visually, mathematically, physically, and functionally indistinguishable
//! from **macOS Tahoe 26 Activity Monitor (`Activity Monitor.app`)**.
//!
//! ### Core Subsystems:
//! - [`process`]: Real-time process telemetry, multi-criteria sorting and filtering
//!   (All, My Processes, Active, Windowed, Search), and POSIX signal termination (`SIGTERM` / `SIGKILL`).
//! - [`memory`]: macOS Tahoe 26 Memory Pressure Graph (Normal Green, Warn Yellow, Critical Red),
//!   paged memory accounting (App, Wired, Compressed, Cached, Swap), and 120-sample rolling ring buffer.
//! - [`cpu`]: Multi-core CPU utilization (User, System, Idle), heterogeneous P-core vs E-core tracking,
//!   and kernel 1/5/15-minute load averages.
//! - [`energy`]: Instantaneous energy impact score formula, 12-hour EWMA power, App Nap state detection,
//!   and battery charging/remaining runtime telemetry.
//! - [`disk_network`]: Disk I/O throughput rates and IOPS, Network inbound/outbound bandwidth and packet traffic.
//! - [`window`]: Liquid Glass window controller, unified 5-tab segmented control (`CPU`, `Memory`, `Energy`, `Disk`, `Network`),
//!   real-time search filter, and Force Quit / Inspector confirmation modal sheets.

#![deny(missing_docs)]

pub mod cpu;
pub mod disk_network;
pub mod energy;
pub mod memory;
pub mod process;
pub mod window;

// Re-exports for convenient top-level access
pub use cpu::{CoreKind, CoreUtilization, CpuHistory, CpuHistorySample, CpuSnapshot};
pub use disk_network::{format_rate, DiskSnapshot, IoHistory, IoThroughputSample, NetworkSnapshot};
pub use energy::{
    compute_energy_impact, is_high_energy_consumer, update_12hr_power_ewma, PowerSource,
    PowerTelemetry,
};
pub use memory::{
    format_bytes, MemoryPressureHistory, MemoryPressureSample, MemoryPressureState, MemorySnapshot,
};
pub use process::{
    ProcessArchitecture, ProcessEntry, ProcessError, ProcessFilter, ProcessManager,
    ProcessSortColumn, SortDirection,
};
pub use window::{
    ActivityMonitorWindowController, ActivityTab, ForceQuitSheetState, InspectorSheetState,
};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_activity_monitor_end_to_end_workflow() {
        let mut controller = ActivityMonitorWindowController::new(Vec2::new(1100.0, 750.0));
        assert_eq!(controller.active_tab, ActivityTab::Cpu);

        // 1. Ingest processes
        let p1 = ProcessEntry {
            pid: 101,
            name: "QuartzCompositor".to_string(),
            user: "nepo".to_string(),
            cpu_percent: 18.5,
            cpu_time_secs: 1420.0,
            threads: 12,
            idle_wakeups_per_sec: 45,
            gpu_percent: 24.0,
            memory_bytes: 420 * 1024 * 1024,
            compressed_bytes: 0,
            energy_impact: compute_energy_impact(18.5, 24.0, 45),
            power_12hr: 15.0,
            app_nap_active: false,
            disk_read_bytes_sec: 1024 * 100,
            disk_write_bytes_sec: 1024 * 50,
            net_rx_bytes_sec: 0,
            net_tx_bytes_sec: 0,
            architecture: ProcessArchitecture::AppleSilicon,
            is_windowed: true,
        };

        let p2 = ProcessEntry {
            pid: 202,
            name: "UnresponsiveVideoRender".to_string(),
            user: "nepo".to_string(),
            cpu_percent: 285.0,
            cpu_time_secs: 820.0,
            threads: 32,
            idle_wakeups_per_sec: 500,
            gpu_percent: 92.0,
            memory_bytes: 8 * 1024 * 1024 * 1024,
            compressed_bytes: 512 * 1024 * 1024,
            energy_impact: compute_energy_impact(285.0, 92.0, 500),
            power_12hr: 180.0,
            app_nap_active: false,
            disk_read_bytes_sec: 1024 * 1024 * 50,
            disk_write_bytes_sec: 1024 * 1024 * 80,
            net_rx_bytes_sec: 1024 * 1024,
            net_tx_bytes_sec: 1024 * 512,
            architecture: ProcessArchitecture::AppleSilicon,
            is_windowed: true,
        };

        controller.process_manager.update_processes(vec![p1, p2]);
        assert_eq!(controller.process_manager.len(), 2);

        // 2. Ingest Memory Snapshot
        let mem_snap = MemorySnapshot {
            timestamp: 1000.0,
            physical_memory_bytes: 32 * 1024 * 1024 * 1024,
            app_memory_bytes: 14 * 1024 * 1024 * 1024,
            wired_memory_bytes: 4 * 1024 * 1024 * 1024,
            compressed_bytes: 1 * 1024 * 1024 * 1024,
            cached_files_bytes: 6 * 1024 * 1024 * 1024,
            swap_used_bytes: 0,
            swap_total_bytes: 4 * 1024 * 1024 * 1024,
        };
        controller.memory_history.record_snapshot(&mem_snap);
        controller.latest_memory = Some(mem_snap);

        // 3. Switch to Memory tab and check sort column
        controller.switch_tab(ActivityTab::Memory);
        assert_eq!(controller.process_manager.sort_column, ProcessSortColumn::Memory);
        let procs = controller.process_manager.filtered_and_sorted_processes();
        assert_eq!(procs[0].pid, 202); // 8 GB memory process comes first

        // 4. Force Quit flow for PID 202
        controller.select_process(202);
        assert!(controller.request_quit_selected());
        assert!(controller.force_quit_sheet.is_some());

        // Confirm force quit
        assert!(controller.confirm_quit(true).is_ok());
        assert_eq!(controller.process_manager.len(), 1);
        assert_eq!(controller.process_manager.filtered_and_sorted_processes()[0].pid, 101);
    }
}
