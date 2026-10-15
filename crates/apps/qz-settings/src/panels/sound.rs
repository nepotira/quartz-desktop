//! # Sound & CoreAudio Settings Panel (`qz-settings::panels::sound`)
//!
//! Production-grade configuration panel for macOS Tahoe 26 Sound, alerts,
//! output balance, and CoreAudio auto-pause policies.
//!
//! ### Key Capabilities:
//! - **Master Volume & Balance:** Real-time volume slider with perceptual cubic dB curve.
//! - **System Alert Sound Selector:** Canonical macOS chimes (`Sosumi`, `Basso`, `Ping`, `Funk`, `Hero`, `Tink`).
//! - **CoreAudio Auto-Pause Toggle:** Configures automatic media playback pausing upon headphone disconnection.

use serde::{Deserialize, Serialize};

/// Canonical Apple alert sound names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSoundChoice {
    /// Canonical classic alert tone ("Sosumi").
    Sosumi,
    /// Low frequency alert tone ("Basso").
    Basso,
    /// Clear bell chime ("Ping").
    Ping,
    /// Groovy alert chime ("Funk").
    Funk,
    /// Uplifting chime ("Hero").
    Hero,
    /// High-pitched crystal click ("Tink").
    Tink,
}

impl AlertSoundChoice {
    /// Label for UI dropdown.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Sosumi => "Sosumi",
            Self::Basso => "Basso",
            Self::Ping => "Ping",
            Self::Funk => "Funk",
            Self::Hero => "Hero",
            Self::Tink => "Tink",
        }
    }
}

/// Sound and CoreAudio configuration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoundSettings {
    /// Master output volume level ($0.0..=1.0$).
    pub master_volume: f64,
    /// Master mute state.
    pub is_muted: bool,
    /// Selected system alert tone.
    pub alert_sound: AlertSoundChoice,
    /// Alert volume level relative to master ($0.0..=1.0$).
    pub alert_volume: f64,
    /// Play chime on system startup.
    pub play_sound_on_startup: bool,
    /// Play UI sound effects (Trash, Screenshot shutter).
    pub play_ui_sound_effects: bool,
    /// Play discrete volume tick feedback when volume is changed.
    pub play_feedback_on_volume_change: bool,
    /// Stereo output balance ($-1.0 = \text{left}, 0.0 = \text{center}, 1.0 = \text{right}$).
    pub balance: f64,
    /// Input microphone volume ($0.0..=1.0$).
    pub input_volume: f64,
    /// Automatically pause media playback on headphone disconnect.
    pub auto_pause_on_headphone_disconnect: bool,
    /// Ambient noise reduction enabled on microphone.
    pub ambient_noise_reduction_enabled: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.65,
            is_muted: false,
            alert_sound: AlertSoundChoice::Sosumi,
            alert_volume: 0.80,
            play_sound_on_startup: true,
            play_ui_sound_effects: true,
            play_feedback_on_volume_change: true,
            balance: 0.0,
            input_volume: 0.75,
            auto_pause_on_headphone_disconnect: true,
            ambient_noise_reduction_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sound_defaults() {
        let settings = SoundSettings::default();
        assert_eq!(settings.master_volume, 0.65);
        assert!(!settings.is_muted);
        assert_eq!(settings.alert_sound, AlertSoundChoice::Sosumi);
        assert!(settings.auto_pause_on_headphone_disconnect);
        assert_eq!(settings.balance, 0.0);
    }
}
