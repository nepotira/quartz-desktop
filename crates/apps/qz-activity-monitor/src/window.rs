//! # Activity Monitor Window Controller & Liquid Glass Chrome (`qz-activity-monitor::window`)
//!
//! Production-grade window controller for macOS Tahoe 26 Activity Monitor (`Activity Monitor.app`).
//!
//! ### Key Capabilities:
//! - **Unified Titlebar with Segmented Tab Bar:** 5 canonical tabs: `CPU`, `Memory`, `Energy`, `Disk`, and `Network`.
//! - **Action Buttons:** Force Quit (`X` octagon), Inspect Process (`i` circle), and live search field.
//! - **Adaptive Bottom Summary Panel:** Displays live metrics and graphs corresponding to the active tab.
//! - **Confirmation Modal Sheets:** Liquid Glass sheet for confirming Quit (`SIGTERM`) or Force Quit (`SIGKILL`).

use crate::cpu::{CpuHistory, CpuSnapshot};
use crate::disk_network::{DiskSnapshot, IoHistory, NetworkSnapshot};
use crate::energy::PowerTelemetry;
use crate::memory::{MemoryPressureHistory, MemorySnapshot};
use crate::process::{ProcessError, ProcessFilter, ProcessManager, ProcessSortColumn, SortDirection};
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Canonical Activity Monitor tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityTab {
    /// CPU utilization and per-core thread activity.
    Cpu,
    /// Memory Pressure graph and paging breakdown.
    Memory,
    /// Energy impact, 12hr power, and battery telemetry.
    Energy,
    /// Disk I/O throughput and cumulative read/write.
    Disk,
    /// Network inbound/outbound bandwidth and packet traffic.
    Network,
}

impl ActivityTab {
    /// Returns the localized title of the tab.
    #[must_use]
    pub fn title(&self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "Memória",
            Self::Energy => "Energia",
            Self::Disk => "Disco",
            Self::Network => "Rede",
        }
    }
}

/// Force Quit confirmation modal sheet state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForceQuitSheetState {
    /// Target process PID.
    pub pid: u32,
    /// Process display name.
    pub process_name: String,
    /// `true` if sheet is currently presented.
    pub is_presented: bool,
}

/// Process Inspector modal sheet state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InspectorSheetState {
    /// Target process PID.
    pub pid: u32,
    /// Process display name.
    pub process_name: String,
    /// `true` if sheet is currently presented.
    pub is_presented: bool,
}

/// Window controller managing Activity Monitor's UI state and telemetry pipelines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityMonitorWindowController {
    /// Active tab selection.
    pub active_tab: ActivityTab,
    /// Process table manager.
    pub process_manager: ProcessManager,
    /// Memory pressure history buffer.
    pub memory_history: MemoryPressureHistory,
    /// Latest memory snapshot.
    pub latest_memory: Option<MemorySnapshot>,
    /// CPU history buffer.
    pub cpu_history: CpuHistory,
    /// Latest CPU snapshot.
    pub latest_cpu: Option<CpuSnapshot>,
    /// Latest power and battery telemetry.
    pub latest_power: Option<PowerTelemetry>,
    /// Disk I/O history buffer.
    pub disk_history: IoHistory,
    /// Latest disk snapshot.
    pub latest_disk: Option<DiskSnapshot>,
    /// Network I/O history buffer.
    pub network_history: IoHistory,
    /// Latest network snapshot.
    pub latest_network: Option<NetworkSnapshot>,
    /// Currently selected process PID in the table.
    pub selected_pid: Option<u32>,
    /// Force quit confirmation sheet state.
    pub force_quit_sheet: Option<ForceQuitSheetState>,
    /// Process inspector sheet state.
    pub inspector_sheet: Option<InspectorSheetState>,
    /// Window frame dimensions $[W, H]$ in points.
    pub window_size: Vec2,
    /// Window screen origin $[x, y]$.
    pub window_origin: Vec2,
    /// Search filter string.
    pub search_query: String,
}

impl ActivityMonitorWindowController {
    /// Creates a new `ActivityMonitorWindowController`.
    #[must_use]
    pub fn new(window_size: Vec2) -> Self {
        Self {
            active_tab: ActivityTab::Cpu,
            process_manager: ProcessManager::new(),
            memory_history: MemoryPressureHistory::new(120),
            latest_memory: None,
            cpu_history: CpuHistory::new(120),
            latest_cpu: None,
            latest_power: None,
            disk_history: IoHistory::new(120),
            latest_disk: None,
            network_history: IoHistory::new(120),
            latest_network: None,
            selected_pid: None,
            force_quit_sheet: None,
            inspector_sheet: None,
            window_size,
            window_origin: Vec2::new(120.0, 80.0),
            search_query: String::new(),
        }
    }

    /// Switches the active telemetry tab.
    ///
    /// Automatically aligns table sorting with the selected tab:
    /// - `Cpu` -> Sort by CPU %
    /// - `Memory` -> Sort by Memory bytes
    /// - `Energy` -> Sort by Energy Impact
    /// - `Disk` -> Sort by Disk Write/Read
    /// - `Network` -> Sort by Net Rx/Tx
    pub fn switch_tab(&mut self, tab: ActivityTab) {
        self.active_tab = tab;
        match tab {
            ActivityTab::Cpu => self.process_manager.set_sort(ProcessSortColumn::Cpu, SortDirection::Descending),
            ActivityTab::Memory => self.process_manager.set_sort(ProcessSortColumn::Memory, SortDirection::Descending),
            ActivityTab::Energy => self.process_manager.set_sort(ProcessSortColumn::Energy, SortDirection::Descending),
            ActivityTab::Disk => self.process_manager.set_sort(ProcessSortColumn::DiskWrite, SortDirection::Descending),
            ActivityTab::Network => self.process_manager.set_sort(ProcessSortColumn::NetRx, SortDirection::Descending),
        }
    }

    /// Updates search filter query.
    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.to_string();
        if self.search_query.is_empty() {
            self.process_manager.filter = ProcessFilter::All;
        } else {
            self.process_manager.filter = ProcessFilter::Search(self.search_query.clone());
        }
    }

    /// Selects a process in the table by PID.
    pub fn select_process(&mut self, pid: u32) {
        self.selected_pid = Some(pid);
    }

    /// Presents the Force Quit confirmation sheet for the currently selected process.
    pub fn request_quit_selected(&mut self) -> bool {
        if let Some(pid) = self.selected_pid {
            let proc_name = self
                .process_manager
                .filtered_and_sorted_processes()
                .iter()
                .find(|p| p.pid == pid)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("PID {pid}"));

            self.force_quit_sheet = Some(ForceQuitSheetState {
                pid,
                process_name: proc_name,
                is_presented: true,
            });
            true
        } else {
            false
        }
    }

    /// Confirms termination from the modal sheet (`force = false` for Quit, `true` for Force Quit).
    pub fn confirm_quit(&mut self, force: bool) -> Result<(), ProcessError> {
        if let Some(sheet) = self.force_quit_sheet.take() {
            self.process_manager.terminate_process(sheet.pid, force)?;
            if self.selected_pid == Some(sheet.pid) {
                self.selected_pid = None;
            }
        }
        Ok(())
    }

    /// Dismisses the Force Quit sheet without action.
    pub fn cancel_quit_sheet(&mut self) {
        self.force_quit_sheet = None;
    }

    /// Opens the Process Inspector sheet for the selected process.
    pub fn open_inspector_selected(&mut self) -> bool {
        if let Some(pid) = self.selected_pid {
            let proc_name = self
                .process_manager
                .filtered_and_sorted_processes()
                .iter()
                .find(|p| p.pid == pid)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("PID {pid}"));

            self.inspector_sheet = Some(InspectorSheetState {
                pid,
                process_name: proc_name,
                is_presented: true,
            });
            true
        } else {
            false
        }
    }

    /// Dismisses the inspector sheet.
    pub fn close_inspector(&mut self) {
        self.inspector_sheet = None;
    }

    /// Returns the window title with process count.
    #[must_use]
    pub fn window_title(&self) -> String {
        format!("Monitor de Atividade ({} processos)", self.process_manager.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessEntry;

    fn sample_proc(pid: u32, name: &str, cpu: f32, mem_mb: u64) -> ProcessEntry {
        ProcessEntry {
            pid,
            name: name.to_string(),
            user: "nepo".to_string(),
            cpu_percent: cpu,
            cpu_time_secs: 10.0,
            threads: 4,
            idle_wakeups_per_sec: 10,
            gpu_percent: 0.0,
            memory_bytes: mem_mb * 1024 * 1024,
            compressed_bytes: 0,
            energy_impact: cpu,
            power_12hr: cpu,
            app_nap_active: false,
            disk_read_bytes_sec: 0,
            disk_write_bytes_sec: 0,
            net_rx_bytes_sec: 0,
            net_tx_bytes_sec: 0,
            architecture: crate::process::ProcessArchitecture::AppleSilicon,
            is_windowed: true,
        }
    }

    #[test]
    fn test_tab_switching_sort_alignment() {
        let mut controller = ActivityMonitorWindowController::new(Vec2::new(1000.0, 700.0));
        assert_eq!(controller.active_tab, ActivityTab::Cpu);
        assert_eq!(controller.process_manager.sort_column, ProcessSortColumn::Cpu);

        controller.switch_tab(ActivityTab::Memory);
        assert_eq!(controller.active_tab, ActivityTab::Memory);
        assert_eq!(controller.process_manager.sort_column, ProcessSortColumn::Memory);

        controller.switch_tab(ActivityTab::Energy);
        assert_eq!(controller.active_tab, ActivityTab::Energy);
        assert_eq!(controller.process_manager.sort_column, ProcessSortColumn::Energy);
    }

    #[test]
    fn test_force_quit_sheet_lifecycle() {
        let mut controller = ActivityMonitorWindowController::new(Vec2::new(1000.0, 700.0));
        controller.process_manager.update_processes(vec![
            sample_proc(501, "Safari", 15.0, 512),
        ]);

        // No selection -> request fails
        assert!(!controller.request_quit_selected());

        // Select and request quit
        controller.select_process(501);
        assert!(controller.request_quit_selected());
        assert!(controller.force_quit_sheet.is_some());
        assert_eq!(controller.force_quit_sheet.as_ref().unwrap().process_name, "Safari");

        // Confirm quit
        assert!(controller.confirm_quit(true).is_ok());
        assert!(controller.force_quit_sheet.is_none());
        assert_eq!(controller.process_manager.len(), 0);
    }

    #[test]
    fn test_search_and_inspector() {
        let mut controller = ActivityMonitorWindowController::new(Vec2::new(1000.0, 700.0));
        controller.process_manager.update_processes(vec![
            sample_proc(501, "Safari", 15.0, 512),
            sample_proc(502, "Terminal", 2.0, 128),
        ]);

        controller.set_search_query("term");
        assert_eq!(controller.process_manager.filtered_and_sorted_processes().len(), 1);

        controller.select_process(502);
        assert!(controller.open_inspector_selected());
        assert!(controller.inspector_sheet.is_some());
        controller.close_inspector();
        assert!(controller.inspector_sheet.is_none());
    }
}
