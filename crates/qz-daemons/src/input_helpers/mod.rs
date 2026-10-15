//! # Input Helpers Subsystem (`qz-daemons::input_helpers`)
//!
//! Production-grade input enhancement daemons for macOS Tahoe 26:
//! - **Accents Character Picker:** Diacritics menu on press-and-hold (>300ms).
//! - **Emoji & Symbols Palette:** Canonical `Ctrl+Cmd+Space` character popover with search & skin tones.
//! - **Speech Dictation Listener:** Sub-400ms double-tap `Fn` activation with audio RMS meter.

pub mod accents;
pub mod dictation;
pub mod emoji;

pub use accents::{AccentsAction, AccentsManager, AccentsState};
pub use dictation::{DictationAction, DictationDaemon, DictationState};
pub use emoji::{EmojiCategory, EmojiEntry, EmojiPaletteDaemon, SkinTone, EMOJI_CATALOG};

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Unified events emitted by the input helpers coordinator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputHelperEvent {
    /// Character or string to be committed to the active text field.
    CommitText(String),
    /// Sound effect to be dispatched.
    PlaySound(crate::audio::SystemSoundEffect),
    /// Request display of diacritic picker at given cursor coordinate.
    ShowAccentsPicker {
        /// Base character.
        base_char: char,
        /// Diacritic candidates.
        candidates: Vec<char>,
        /// Screen position `[x, y]`.
        position: [i32; 2],
    },
    /// Dismiss diacritic picker.
    DismissAccentsPicker,
}

/// Unified coordinator managing all text input helpers.
#[derive(Debug, Clone)]
pub struct InputHelpersCoordinator {
    /// Press-and-hold accents manager.
    pub accents: AccentsManager,
    /// Character palette daemon (`Ctrl+Cmd+Space`).
    pub emoji_palette: EmojiPaletteDaemon,
    /// Speech dictation listener (`Fn+Fn`).
    pub dictation: DictationDaemon,
}

impl Default for InputHelpersCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHelpersCoordinator {
    /// Creates a new `InputHelpersCoordinator`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            accents: AccentsManager::new(),
            emoji_palette: EmojiPaletteDaemon::new(),
            dictation: DictationDaemon::new(),
        }
    }

    /// Handles a key-down event from the compositor.
    pub fn handle_key_down(&mut self, key: char, cursor_pos: Vec2) -> Option<InputHelperEvent> {
        let action = self.accents.handle_key_down(key, cursor_pos);
        match action {
            AccentsAction::None => None,
            AccentsAction::ShowPicker { base_char, candidates, cursor_position } => {
                Some(InputHelperEvent::ShowAccentsPicker {
                    base_char,
                    candidates,
                    position: cursor_position,
                })
            }
            AccentsAction::CommitCharacter(c) => {
                Some(InputHelperEvent::CommitText(c.to_string()))
            }
            AccentsAction::Dismiss => Some(InputHelperEvent::DismissAccentsPicker),
        }
    }

    /// Advances timers across input helpers.
    pub fn tick(&mut self, dt: f64) -> Option<InputHelperEvent> {
        let action = self.accents.tick(dt);
        match action {
            AccentsAction::None => None,
            AccentsAction::ShowPicker { base_char, candidates, cursor_position } => {
                Some(InputHelperEvent::ShowAccentsPicker {
                    base_char,
                    candidates,
                    position: cursor_position,
                })
            }
            AccentsAction::CommitCharacter(c) => {
                Some(InputHelperEvent::CommitText(c.to_string()))
            }
            AccentsAction::Dismiss => Some(InputHelperEvent::DismissAccentsPicker),
        }
    }

    /// Toggles the Emoji & Symbols Palette (`Ctrl+Cmd+Space`).
    pub fn toggle_emoji_palette(&mut self) -> bool {
        self.emoji_palette.toggle_palette()
    }

    /// Handles `Fn` key press for dictation double-tap.
    pub fn handle_fn_key_press(&mut self, timestamp: f64) -> Option<InputHelperEvent> {
        let action = self.dictation.handle_fn_key_press(timestamp);
        match action {
            DictationAction::None => None,
            DictationAction::StartListening => {
                Some(InputHelperEvent::PlaySound(crate::audio::SystemSoundEffect::DictationStart))
            }
            DictationAction::StopListening => {
                Some(InputHelperEvent::PlaySound(crate::audio::SystemSoundEffect::DictationStop))
            }
            DictationAction::CommitTranscript(text) => Some(InputHelperEvent::CommitText(text)),
            DictationAction::PlaySound(snd) => Some(InputHelperEvent::PlaySound(snd)),
        }
    }
}
