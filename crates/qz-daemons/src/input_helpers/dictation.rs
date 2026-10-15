//! # Speech Dictation Listener Daemon (`qz-daemons::input_helpers::dictation`)
//!
//! Production-grade speech dictation state machine for macOS Tahoe 26.
//!
//! ### Features:
//! - **Double-Tap Fn Key Activation:** Sub-400ms double-tap timing detector for the `Fn`
//!   (Globe) key to toggle speech-to-text dictation.
//! - **Audio Level RMS Metering:** Exponentially smoothed volume envelope meter ($0.0..=1.0$)
//!   powering the pulsating purple microphone pill in the Privacy Capsule.
//! - **Interim Hypothesis & Finalized Text Buffers:** Supports live streaming partial speech
//!   recognition tokens and final committed phrases.
//! - **Chime Sound Feedback:** Emits canonical `DictationStart` and `DictationStop` chimes.

use crate::audio::SystemSoundEffect;
use serde::{Deserialize, Serialize};

/// Operational state of the speech dictation listener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DictationState {
    /// Inactive; microphone is dormant.
    Idle,
    /// Actively recording microphone input and parsing speech stream.
    Listening,
    /// Final acoustic/language model pass finalizing the transcription.
    Processing,
    /// Dictation encountered an error.
    Error(String),
}

/// Action produced by dictation event transitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DictationAction {
    /// No action required.
    None,
    /// Start audio capture hardware and emit start chime.
    StartListening,
    /// Halt audio capture hardware and emit stop chime.
    StopListening,
    /// Commit finalized speech transcription into active focused text client.
    CommitTranscript(String),
    /// Play a system sound effect.
    PlaySound(SystemSoundEffect),
}

/// Speech Dictation Daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationDaemon {
    /// Current state machine status.
    state: DictationState,
    /// Timestamp of last `Fn` key press for double-tap detection in seconds.
    last_fn_press_timestamp: Option<f64>,
    /// Maximum elapsed time allowed between two `Fn` taps (macOS standard: 0.400s).
    pub double_tap_window_seconds: f64,
    /// Current audio input RMS level `0.0..=1.0`.
    pub audio_rms_level: f64,
    /// Live interim partial hypothesis string (e.g. currently spoken words).
    pub interim_transcript: String,
    /// Finalized sentences ready for insertion.
    pub committed_transcript: String,
}

impl Default for DictationDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl DictationDaemon {
    /// Creates a new `DictationDaemon`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: DictationState::Idle,
            last_fn_press_timestamp: None,
            double_tap_window_seconds: 0.40,
            audio_rms_level: 0.0,
            interim_transcript: String::new(),
            committed_transcript: String::new(),
        }
    }

    /// Handles an `Fn` (Globe) key press at a given monotonic timestamp (in seconds).
    ///
    /// If two presses occur within `double_tap_window_seconds` (400ms), dictation starts or stops.
    pub fn handle_fn_key_press(&mut self, timestamp_seconds: f64) -> DictationAction {
        if let Some(prev_time) = self.last_fn_press_timestamp {
            let delta = timestamp_seconds - prev_time;
            if delta <= self.double_tap_window_seconds && delta > 0.05 {
                // Double tap detected! Reset timestamp
                self.last_fn_press_timestamp = None;
                return self.toggle_dictation();
            }
        }

        self.last_fn_press_timestamp = Some(timestamp_seconds);
        DictationAction::None
    }

    /// Toggles dictation listening on or off.
    pub fn toggle_dictation(&mut self) -> DictationAction {
        match self.state {
            DictationState::Idle => {
                self.state = DictationState::Listening;
                self.interim_transcript.clear();
                self.committed_transcript.clear();
                self.audio_rms_level = 0.0;
                DictationAction::StartListening
            }
            DictationState::Listening | DictationState::Processing => {
                self.finish_dictation()
            }
            DictationState::Error(_) => {
                self.state = DictationState::Idle;
                DictationAction::None
            }
        }
    }

    /// Manually cancels dictation without committing text.
    pub fn cancel(&mut self) -> DictationAction {
        if self.state == DictationState::Listening || self.state == DictationState::Processing {
            self.state = DictationState::Idle;
            self.interim_transcript.clear();
            DictationAction::PlaySound(SystemSoundEffect::DictationStop)
        } else {
            DictationAction::None
        }
    }

    /// Updates incoming audio sample buffer RMS level with exponential smoothing:
    ///
    /// $$L_{\text{new}} = (1 - \alpha) L_{\text{prev}} + \alpha L_{\text{raw}}$$
    pub fn update_audio_rms(&mut self, raw_level: f64) {
        let clamped = raw_level.clamp(0.0, 1.0);
        let alpha = 0.35; // Smoothing factor
        self.audio_rms_level = (1.0 - alpha) * self.audio_rms_level + alpha * clamped;
    }

    /// Pushes an interim hypothesis partial phrase from the speech recognition engine.
    pub fn update_interim_hypothesis(&mut self, text: &str) {
        if self.state == DictationState::Listening {
            self.interim_transcript = text.to_string();
        }
    }

    /// Finalizes a sentence chunk and appends to the committed transcript.
    pub fn commit_partial_sentence(&mut self, sentence: &str) {
        if self.state == DictationState::Listening {
            if !self.committed_transcript.is_empty() {
                self.committed_transcript.push(' ');
            }
            self.committed_transcript.push_str(sentence.trim());
            self.interim_transcript.clear();
        }
    }

    /// Completes dictation, commits transcript and returns to Idle.
    pub fn finish_dictation(&mut self) -> DictationAction {
        self.state = DictationState::Idle;
        let mut final_text = self.committed_transcript.clone();
        if !self.interim_transcript.is_empty() {
            if !final_text.is_empty() {
                final_text.push(' ');
            }
            final_text.push_str(self.interim_transcript.trim());
        }

        self.interim_transcript.clear();
        self.committed_transcript.clear();

        if !final_text.is_empty() {
            DictationAction::CommitTranscript(final_text)
        } else {
            DictationAction::PlaySound(SystemSoundEffect::DictationStop)
        }
    }

    /// Returns current state.
    #[must_use]
    pub fn state(&self) -> &DictationState {
        &self.state
    }

    /// Returns `true` if dictation is actively listening.
    #[must_use]
    pub fn is_listening(&self) -> bool {
        self.state == DictationState::Listening
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_tap_fn_activation() {
        let mut daemon = DictationDaemon::new();
        assert_eq!(daemon.state(), &DictationState::Idle);

        // First Fn tap at 10.0s -> records timestamp
        let act = daemon.handle_fn_key_press(10.0);
        assert_eq!(act, DictationAction::None);
        assert_eq!(daemon.state(), &DictationState::Idle);

        // Second Fn tap at 10.25s (250ms delta <= 400ms) -> triggers StartListening
        let act = daemon.handle_fn_key_press(10.25);
        assert_eq!(act, DictationAction::StartListening);
        assert!(daemon.is_listening());
    }

    #[test]
    fn test_slow_tap_fn_does_not_activate() {
        let mut daemon = DictationDaemon::new();
        daemon.handle_fn_key_press(1.0);

        // Second tap at 1.6s (600ms delta > 400ms threshold) -> No activation
        let act = daemon.handle_fn_key_press(1.6);
        assert_eq!(act, DictationAction::None);
        assert_eq!(daemon.state(), &DictationState::Idle);
    }

    #[test]
    fn test_transcript_streaming_and_completion() {
        let mut daemon = DictationDaemon::new();
        daemon.toggle_dictation();
        assert!(daemon.is_listening());

        daemon.commit_partial_sentence("Open Safari");
        daemon.update_interim_hypothesis("and visit Apple.com");

        // Finish dictation
        let act = daemon.finish_dictation();
        assert_eq!(
            act,
            DictationAction::CommitTranscript("Open Safari and visit Apple.com".to_string())
        );
        assert_eq!(daemon.state(), &DictationState::Idle);
    }

    #[test]
    fn test_rms_audio_metering() {
        let mut daemon = DictationDaemon::new();
        daemon.update_audio_rms(1.0);
        assert!(daemon.audio_rms_level > 0.3 && daemon.audio_rms_level < 0.4);

        daemon.update_audio_rms(1.0);
        assert!(daemon.audio_rms_level > 0.5);
    }
}
