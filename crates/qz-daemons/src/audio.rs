//! # CoreAudio Subsystem Daemon (`qz-daemons::audio`)
//!
//! Production-grade CoreAudio-inspired audio subsystem daemon for macOS Tahoe 26.
//!
//! ### Key Capabilities:
//! - **Auto-Pause on Disconnect:** Automatically pauses active media playback when
//!   headphones or Bluetooth audio devices are disconnected, preventing accidental sound blasts
//!   through laptop speakers.
//! - **Auto-Pause on Sleep / Screen Lock:** Pauses media playback when the display sleeps or
//!   the session locks.
//! - **Perceptual Decibel Curve:** Implements cubic perceptual loudness mapping
//!   ($V_{\text{amp}} = x^3$) and decibel conversions ($\text{dB} = 20 \log_{10}(V)$)
//!   matching human hearing sensitivity.
//! - **Canonical Tahoe 26 System Sound Effects:** Built-in triggers for volume tick, alerts
//!   (Basso, Sosumi, Ping, Funk, Hero, Tink), screenshot camera shutter, and trash empty.
//! - **Device Switcher & State Management:** Tracks input/output sinks, default devices,
//!   and per-device volume profiles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Audio device connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioDeviceType {
    /// Internal built-in speakers.
    BuiltInSpeaker,
    /// 3.5mm analog stereo headphone jack.
    Headphones35mm,
    /// Wireless Bluetooth A2DP audio device.
    BluetoothA2DP,
    /// USB audio interface or DAC.
    UsbAudio,
    /// HDMI or DisplayPort digital audio sink.
    DisplayAudio,
    /// Built-in microphone array.
    BuiltInMicrophone,
    /// External microphone or USB interface.
    ExternalMicrophone,
}

impl AudioDeviceType {
    /// Returns `true` if this device type is considered a headphone or personal listening device
    /// that should trigger auto-pause upon disconnection.
    #[must_use]
    pub fn is_headphone(&self) -> bool {
        matches!(self, Self::Headphones35mm | Self::BluetoothA2DP)
    }
}

/// Bluetooth audio streaming codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BluetoothAudioCodec {
    /// Standard Low Complexity Subband Coding.
    Sbc,
    /// Advanced Audio Coding (macOS default for AirPods and Beats).
    Aac,
    /// Qualcomm aptX.
    Aptx,
    /// Qualcomm aptX HD high-resolution.
    AptxHd,
    /// Sony LDAC 990kbps high-resolution.
    Ldac,
    /// Bluetooth LE Audio Low Complexity Communications Codec.
    Lc3,
}

/// An audio endpoint device representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioDevice {
    /// Unique identifier for the audio device (e.g. ALSA/PulseAudio/PipeWire sink ID).
    pub id: String,
    /// Human-readable device display name.
    pub name: String,
    /// Device hardware connection category.
    pub device_type: AudioDeviceType,
    /// `true` if this is an input device (microphone), `false` if output (speaker/headphones).
    pub is_input: bool,
    /// Current volume level in normalized linear range `0.0..=1.0`.
    pub volume: f64,
    /// Mute status.
    pub is_muted: bool,
    /// Optional Bluetooth audio codec if connected via Bluetooth A2DP.
    pub bluetooth_codec: Option<BluetoothAudioCodec>,
}

/// Canonical system sound effect types in macOS Tahoe 26.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemSoundEffect {
    /// Volume adjustment discrete tick sound.
    VolumeTick,
    /// Low frequency alert tone ("Basso").
    AlertBasso,
    /// Canonical classic alert tone ("Sosumi").
    AlertSosumi,
    /// Clear bell chime ("Ping").
    AlertPing,
    /// Groovy alert chime ("Funk").
    AlertFunk,
    /// Uplifting chime ("Hero").
    AlertHero,
    /// High-pitched crystal click ("Tink").
    AlertTink,
    /// Mechanical camera shutter click on screenshot.
    ScreenshotCapture,
    /// Crumpled paper rustle when emptying Trash.
    TrashEmpty,
    /// Quick chime when speech dictation begins.
    DictationStart,
    /// Low tone when speech dictation concludes.
    DictationStop,
}

/// Auto-pause reaction action triggered by audio events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPauseAction {
    /// Send pause command to active media sessions (MPRIS / Web browsers / Native players).
    PauseMedia,
    /// No auto-pause action necessary.
    None,
}

/// Main CoreAudio subsystem daemon managing volume, devices, and playback policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDaemon {
    /// Registered output audio devices indexed by their unique device ID.
    output_devices: HashMap<String, AudioDevice>,
    /// Registered input audio devices indexed by their unique device ID.
    input_devices: HashMap<String, AudioDevice>,
    /// Active default output device ID.
    default_output_id: Option<String>,
    /// Active default input device ID.
    default_input_id: Option<String>,
    /// Master output volume in range `0.0..=1.0`.
    master_volume: f64,
    /// Master mute state.
    is_master_muted: bool,
    /// Cached unmuted volume level for restoration after unmuting.
    unmuted_volume_cache: f64,
    /// Sound feedback on volume change enabled (System Preferences toggle).
    pub play_feedback_on_volume_change: bool,
    /// Event log of dispatched system sound effects (useful for testing and telemetry).
    recent_sound_effects: Vec<SystemSoundEffect>,
}

impl Default for AudioDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioDaemon {
    /// Creates a new `AudioDaemon` with default built-in speaker and microphone.
    #[must_use]
    pub fn new() -> Self {
        let mut daemon = Self {
            output_devices: HashMap::new(),
            input_devices: HashMap::new(),
            default_output_id: None,
            default_input_id: None,
            master_volume: 0.50,
            is_master_muted: false,
            unmuted_volume_cache: 0.50,
            play_feedback_on_volume_change: true,
            recent_sound_effects: Vec::new(),
        };

        // Register default built-in speaker
        let builtin_speaker = AudioDevice {
            id: "sink-builtin-speakers".to_string(),
            name: "MacBook Pro Speakers".to_string(),
            device_type: AudioDeviceType::BuiltInSpeaker,
            is_input: false,
            volume: 0.50,
            is_muted: false,
            bluetooth_codec: None,
        };
        daemon.add_device(builtin_speaker);
        daemon.set_default_output("sink-builtin-speakers");

        // Register default built-in microphone
        let builtin_mic = AudioDevice {
            id: "source-builtin-mic".to_string(),
            name: "MacBook Pro Microphone".to_string(),
            device_type: AudioDeviceType::BuiltInMicrophone,
            is_input: true,
            volume: 0.80,
            is_muted: false,
            bluetooth_codec: None,
        };
        daemon.add_device(builtin_mic);
        daemon.set_default_input("source-builtin-mic");

        daemon
    }

    /// Adds or updates an audio device.
    pub fn add_device(&mut self, device: AudioDevice) {
        if device.is_input {
            self.input_devices.insert(device.id.clone(), device);
        } else {
            self.output_devices.insert(device.id.clone(), device);
        }
    }

    /// Removes an audio device by ID.
    ///
    /// If the removed device was the active default output and was a headphone/Bluetooth device,
    /// an `AutoPauseAction::PauseMedia` is returned to protect user privacy and avoid blasting
    /// audio through built-in speakers.
    pub fn remove_device(&mut self, device_id: &str) -> AutoPauseAction {
        let mut auto_pause = AutoPauseAction::None;

        if let Some(removed) = self.output_devices.remove(device_id) {
            if Some(device_id) == self.default_output_id.as_deref() {
                // If headphones or bluetooth were active and unplugged, trigger auto-pause!
                if removed.device_type.is_headphone() {
                    auto_pause = AutoPauseAction::PauseMedia;
                }
                // Fall back to built-in speaker or first available output
                self.default_output_id = self
                    .output_devices
                    .values()
                    .find(|d| d.device_type == AudioDeviceType::BuiltInSpeaker)
                    .map(|d| d.id.clone())
                    .or_else(|| self.output_devices.keys().next().cloned());
            }
        }

        self.input_devices.remove(device_id);
        if Some(device_id) == self.default_input_id.as_deref() {
            self.default_input_id = self.input_devices.keys().next().cloned();
        }

        auto_pause
    }

    /// Sets the active default output sink.
    pub fn set_default_output(&mut self, device_id: &str) -> bool {
        if self.output_devices.contains_key(device_id) {
            self.default_output_id = Some(device_id.to_string());
            true
        } else {
            false
        }
    }

    /// Sets the active default input source.
    pub fn set_default_input(&mut self, device_id: &str) -> bool {
        if self.input_devices.contains_key(device_id) {
            self.default_input_id = Some(device_id.to_string());
            true
        } else {
            false
        }
    }

    /// Returns the currently active default output device.
    #[must_use]
    pub fn default_output(&self) -> Option<&AudioDevice> {
        self.default_output_id
            .as_ref()
            .and_then(|id| self.output_devices.get(id))
    }

    /// Returns the currently active default input device.
    #[must_use]
    pub fn default_input(&self) -> Option<&AudioDevice> {
        self.default_input_id
            .as_ref()
            .and_then(|id| self.input_devices.get(id))
    }

    /// Returns the master volume in range `0.0..=1.0`.
    #[must_use]
    pub fn master_volume(&self) -> f64 {
        if self.is_master_muted {
            0.0
        } else {
            self.master_volume
        }
    }

    /// Sets the master volume in range `0.0..=1.0`.
    ///
    /// Automatically unmutes if volume is set above `0.0`.
    pub fn set_master_volume(&mut self, volume: f64) {
        let clamped = volume.clamp(0.0, 1.0);
        self.master_volume = clamped;
        if clamped > 0.0 && self.is_master_muted {
            self.is_master_muted = false;
        }
        if clamped > 0.0 {
            self.unmuted_volume_cache = clamped;
        }
        if self.play_feedback_on_volume_change {
            self.play_sound(SystemSoundEffect::VolumeTick);
        }
    }

    /// Toggles master mute state.
    ///
    /// When unmuting, restores the previously cached volume level.
    pub fn toggle_mute(&mut self) -> bool {
        self.is_master_muted = !self.is_master_muted;
        if !self.is_master_muted && self.master_volume == 0.0 {
            self.master_volume = if self.unmuted_volume_cache > 0.0 {
                self.unmuted_volume_cache
            } else {
                0.25
            };
        }
        self.is_master_muted
    }

    /// Returns `true` if master audio is currently muted.
    #[must_use]
    pub fn is_muted(&self) -> bool {
        self.is_master_muted || self.master_volume == 0.0
    }

    /// Plays a system sound effect and registers it in the event log.
    pub fn play_sound(&mut self, effect: SystemSoundEffect) {
        self.recent_sound_effects.push(effect);
        if self.recent_sound_effects.len() > 64 {
            self.recent_sound_effects.remove(0);
        }
    }

    /// Handles system suspend or display lock by generating an auto-pause signal.
    #[must_use]
    pub fn handle_system_sleep(&mut self) -> AutoPauseAction {
        AutoPauseAction::PauseMedia
    }

    /// Converts a linear slider value `0.0..=1.0` into a perceptual cubic amplitude multiplier:
    ///
    /// $$V_{\text{amp}} = x^3$$
    ///
    /// This accurately models human ear logarithmic perception of loudness.
    #[must_use]
    pub fn linear_to_perceptual(linear: f64) -> f64 {
        let x = linear.clamp(0.0, 1.0);
        x * x * x
    }

    /// Converts a perceptual amplitude multiplier back to a linear slider position:
    ///
    /// $$x = \sqrt[3]{V_{\text{amp}}}$$
    #[must_use]
    pub fn perceptual_to_linear(perceptual: f64) -> f64 {
        let p = perceptual.clamp(0.0, 1.0);
        p.cbrt()
    }

    /// Converts a normalized volume level `0.0..=1.0` to decibels (dB), where `1.0` is `0.0 dB`
    /// and `0.0` maps to negative infinity (clamped to `-60.0 dB` floor).
    #[must_use]
    pub fn volume_to_decibels(volume: f64) -> f64 {
        if volume <= 0.001 {
            -60.0
        } else {
            let amp = Self::linear_to_perceptual(volume);
            let db = 20.0 * amp.log10();
            db.max(-60.0)
        }
    }

    /// Returns the recent sound effects played.
    #[must_use]
    pub fn recent_sounds(&self) -> &[SystemSoundEffect] {
        &self.recent_sound_effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let daemon = AudioDaemon::new();
        assert_eq!(daemon.master_volume(), 0.50);
        assert!(!daemon.is_muted());
        assert_eq!(
            daemon.default_output().map(|d| d.device_type),
            Some(AudioDeviceType::BuiltInSpeaker)
        );
        assert_eq!(
            daemon.default_input().map(|d| d.device_type),
            Some(AudioDeviceType::BuiltInMicrophone)
        );
    }

    #[test]
    fn test_headphone_disconnect_auto_pause() {
        let mut daemon = AudioDaemon::new();

        // Connect AirPods Max
        let airpods = AudioDevice {
            id: "bt-airpods-max".to_string(),
            name: "AirPods Max".to_string(),
            device_type: AudioDeviceType::BluetoothA2DP,
            is_input: false,
            volume: 0.70,
            is_muted: false,
            bluetooth_codec: Some(BluetoothAudioCodec::Aac),
        };
        daemon.add_device(airpods);
        assert!(daemon.set_default_output("bt-airpods-max"));
        assert_eq!(
            daemon.default_output().unwrap().device_type,
            AudioDeviceType::BluetoothA2DP
        );

        // Disconnect AirPods Max -> must trigger AutoPauseAction::PauseMedia
        let action = daemon.remove_device("bt-airpods-max");
        assert_eq!(action, AutoPauseAction::PauseMedia);

        // Active output should have fallen back to built-in speakers
        assert_eq!(
            daemon.default_output().unwrap().device_type,
            AudioDeviceType::BuiltInSpeaker
        );
    }

    #[test]
    fn test_speaker_disconnect_no_auto_pause() {
        let mut daemon = AudioDaemon::new();

        // Connect HDMI Display Audio
        let hdmi = AudioDevice {
            id: "sink-hdmi".to_string(),
            name: "Studio Display Audio".to_string(),
            device_type: AudioDeviceType::DisplayAudio,
            is_input: false,
            volume: 0.60,
            is_muted: false,
            bluetooth_codec: None,
        };
        daemon.add_device(hdmi);
        daemon.set_default_output("sink-hdmi");

        // Disconnecting a non-headphone speaker should not trigger auto-pause
        let action = daemon.remove_device("sink-hdmi");
        assert_eq!(action, AutoPauseAction::None);
    }

    #[test]
    fn test_mute_and_restore_volume() {
        let mut daemon = AudioDaemon::new();
        daemon.set_master_volume(0.80);
        assert_eq!(daemon.master_volume(), 0.80);

        // Mute
        let muted = daemon.toggle_mute();
        assert!(muted);
        assert!(daemon.is_muted());
        assert_eq!(daemon.master_volume(), 0.0);

        // Unmute -> restores 0.80
        let unmuted = daemon.toggle_mute();
        assert!(!unmuted);
        assert!(!daemon.is_muted());
        assert_eq!(daemon.master_volume(), 0.80);
    }

    #[test]
    fn test_perceptual_volume_math() {
        // At 0.0, perceptual is 0.0
        assert_eq!(AudioDaemon::linear_to_perceptual(0.0), 0.0);
        // At 1.0, perceptual is 1.0
        assert_eq!(AudioDaemon::linear_to_perceptual(1.0), 1.0);
        // At 0.5, perceptual is 0.5^3 = 0.125
        assert!((AudioDaemon::linear_to_perceptual(0.5) - 0.125).abs() < 1e-6);

        // Roundtrip identity
        let orig = 0.65;
        let p = AudioDaemon::linear_to_perceptual(orig);
        let back = AudioDaemon::perceptual_to_linear(p);
        assert!((orig - back).abs() < 1e-6);

        // Decibels check: 1.0 is 0dB
        assert_eq!(AudioDaemon::volume_to_decibels(1.0), 0.0);
        // Decibels floor
        assert_eq!(AudioDaemon::volume_to_decibels(0.0), -60.0);
    }

    #[test]
    fn test_system_sleep_auto_pause() {
        let mut daemon = AudioDaemon::new();
        let action = daemon.handle_system_sleep();
        assert_eq!(action, AutoPauseAction::PauseMedia);
    }
}
