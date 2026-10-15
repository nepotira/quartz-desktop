//! # QuartzLinux System Daemons (`qz-daemons`)
//!
//! Production-grade background system daemons implementing macOS Tahoe 26 hardware,
//! telemetry, user feedback, and input subsystems.
//!
//! ### Subsystems Included:
//! 1. **CoreAudio Daemon ([`audio`]):**
//!    - Auto-pause on headphone/Bluetooth disconnect and system sleep.
//!    - Device switching and master volume control with perceptual cubic decibel scaling.
//!    - System sound effects dispatcher (Sosumi, Basso, Ping, VolumeTick, Shutter).
//! 2. **Network & Option-Diagnostics Daemon ([`network`]):**
//!    - Deep RF & PHY inspection: BSSID, RSSI, Noise, $\text{SNR} = \text{RSSI} - \text{Noise}$,
//!      MCS index, spatial streams, channel width, Wi-Fi 6/7 standards.
//!    - Bluetooth diagnostics: RSSI, link quality, latency, audio codecs (AAC, LDAC, LC3),
//!      and AirPods multi-battery status.
//! 3. **Power, Battery & Clamshell Daemon ([`power`]):**
//!    - Deterministic clamshell mode state machine (external display + AC power requirement).
//!    - Battery telemetry, health percentage, cycle count, and time remaining estimation.
//!    - Low Power Mode signaling (VRR 120Hz $\to$ 60Hz throttle).
//! 4. **Bezel On-Screen Display HUD ([`osd`]):**
//!    - Bezel popover with analytical $C^2$ continuous spring physics.
//!    - **Fine-step 1/64 granularity (64 discrete levels)** triggered by
//!      `Option + Shift + Volume/Brightness`, alongside standard 1/16 steps (16 notches).
//!    - Auto-dismissal after 1.8 seconds with smooth alpha decay.
//! 5. **Input Helpers Subsystem ([`input_helpers`]):**
//!    - Character Accents Picker on press-and-hold (>300ms) with digit shortcut commit.
//!    - Emoji & Symbols Palette (`Ctrl + Cmd + Space`) with full 9-category catalog & Fitzpatrick skin tones.
//!    - Speech Dictation Listener with sub-400ms double-tap `Fn` activation & RMS audio meter.

#![deny(missing_docs)]

pub mod audio;
pub mod input_helpers;
pub mod network;
pub mod osd;
pub mod power;

pub use audio::{
    AudioDaemon, AudioDevice, AudioDeviceType, AutoPauseAction, BluetoothAudioCodec,
    SystemSoundEffect,
};
pub use input_helpers::{
    accents::{AccentsAction, AccentsManager, AccentsState},
    dictation::{DictationAction, DictationDaemon, DictationState},
    emoji::{EmojiCategory, EmojiEntry, EmojiPaletteDaemon, SkinTone, EMOJI_CATALOG},
    InputHelperEvent, InputHelpersCoordinator,
};
pub use network::{
    AccessoryBattery, BluetoothDeviceCategory, BluetoothDeviceDiagnostics, ChannelWidth,
    NetworkDaemon, WifiBand, WifiDiagnostics, WifiPhyMode, WifiSecurity,
};
pub use osd::{OsdGlyph, OsdManager, OsdType, OsdVisualState};
pub use power::{
    BatteryChargingState, BatteryCondition, BatteryTelemetry, ClamshellAction, ClamshellState,
    LidState, PowerDaemon,
};

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// High-level events emitted by the system daemon coordinator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SystemDaemonEvent {
    /// Request media playback pause due to headphone unplug or sleep.
    MediaAutoPause,
    /// Request display reconfiguration due to clamshell mode change.
    ClamshellChange(ClamshellAction),
    /// Commit text from input helper into active Wayland focus.
    CommitInputText(String),
    /// Play a system feedback sound.
    PlaySound(SystemSoundEffect),
}

/// Unified coordinator managing all background system daemons.
#[derive(Debug, Clone)]
pub struct SystemDaemonsCoordinator {
    /// CoreAudio subsystem daemon.
    pub audio: AudioDaemon,
    /// Radios and Option-Diagnostics daemon.
    pub network: NetworkDaemon,
    /// Power and Clamshell daemon.
    pub power: PowerDaemon,
    /// Bezel On-Screen Display HUD manager.
    pub osd: OsdManager,
    /// Input helpers (accents, emoji, dictation).
    pub input_helpers: InputHelpersCoordinator,
}

impl Default for SystemDaemonsCoordinator {
    fn default() -> Self {
        Self::new(Vec2::new(960.0, 540.0))
    }
}

impl SystemDaemonsCoordinator {
    /// Creates a new `SystemDaemonsCoordinator` initialized for a display with given center.
    #[must_use]
    pub fn new(display_center: Vec2) -> Self {
        Self {
            audio: AudioDaemon::new(),
            network: NetworkDaemon::new(),
            power: PowerDaemon::new(),
            osd: OsdManager::new(display_center),
            input_helpers: InputHelpersCoordinator::new(),
        }
    }

    /// Advances the internal simulation clock across all daemons by `dt` seconds.
    pub fn tick(&mut self, dt: f64) -> Vec<SystemDaemonEvent> {
        let mut events = Vec::new();

        // Advance OSD animations and timer
        self.osd.tick(dt);

        // Advance input helper hold timers
        if let Some(input_event) = self.input_helpers.tick(dt) {
            match input_event {
                InputHelperEvent::CommitText(text) => {
                    events.push(SystemDaemonEvent::CommitInputText(text));
                }
                InputHelperEvent::PlaySound(snd) => {
                    self.audio.play_sound(snd);
                    events.push(SystemDaemonEvent::PlaySound(snd));
                }
                InputHelperEvent::ShowAccentsPicker { .. } | InputHelperEvent::DismissAccentsPicker => {}
            }
        }

        events
    }

    /// Handles laptop lid state change.
    pub fn handle_lid_event(&mut self, lid: LidState) -> Vec<SystemDaemonEvent> {
        let mut events = Vec::new();
        let action = self.power.set_lid_state(lid);
        if action != ClamshellAction::NoOp {
            events.push(SystemDaemonEvent::ClamshellChange(action));
            if action == ClamshellAction::TriggerSleep {
                let pause = self.audio.handle_system_sleep();
                if pause == AutoPauseAction::PauseMedia {
                    events.push(SystemDaemonEvent::MediaAutoPause);
                }
            }
        }
        events
    }

    /// Handles device disconnection in audio subsystem.
    pub fn handle_audio_device_disconnected(&mut self, device_id: &str) -> Vec<SystemDaemonEvent> {
        let mut events = Vec::new();
        let action = self.audio.remove_device(device_id);
        if action == AutoPauseAction::PauseMedia {
            events.push(SystemDaemonEvent::MediaAutoPause);
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_coordinator_init() {
        let coordinator = SystemDaemonsCoordinator::new(Vec2::new(960.0, 540.0));
        assert!(coordinator.network.is_wifi_enabled);
        assert_eq!(coordinator.power.clamshell_state(), ClamshellState::LidOpen);
        assert!(!coordinator.osd.is_visible());
    }

    #[test]
    fn test_lid_close_without_dock_triggers_sleep_and_pause() {
        let mut coordinator = SystemDaemonsCoordinator::new(Vec2::new(960.0, 540.0));
        coordinator.power.set_external_displays_count(0); // No external monitor

        let events = coordinator.handle_lid_event(LidState::Closed);
        assert!(events.contains(&SystemDaemonEvent::ClamshellChange(ClamshellAction::TriggerSleep)));
        assert!(events.contains(&SystemDaemonEvent::MediaAutoPause));
    }

    #[test]
    fn test_headphone_disconnect_event_propagation() {
        let mut coordinator = SystemDaemonsCoordinator::new(Vec2::new(960.0, 540.0));

        // Add headphone
        coordinator.audio.add_device(AudioDevice {
            id: "hp-jack".to_string(),
            name: "Headphones Jack".to_string(),
            device_type: AudioDeviceType::Headphones35mm,
            is_input: false,
            volume: 0.60,
            is_muted: false,
            bluetooth_codec: None,
        });
        coordinator.audio.set_default_output("hp-jack");

        // Unplug headphone -> should emit MediaAutoPause
        let events = coordinator.handle_audio_device_disconnected("hp-jack");
        assert_eq!(events, vec![SystemDaemonEvent::MediaAutoPause]);
    }

    #[test]
    fn test_osd_fine_step_integration() {
        let mut coordinator = SystemDaemonsCoordinator::new(Vec2::new(960.0, 540.0));
        let state = coordinator.osd.adjust_volume(1, true); // fine step 1/64
        assert!(state.is_fine_step);
        assert_eq!(state.level_64, 33);
        assert!(coordinator.osd.is_visible());
    }
}
