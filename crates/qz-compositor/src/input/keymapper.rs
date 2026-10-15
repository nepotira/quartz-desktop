//! # Smart Keymapper for QuartzLinux
//!
//! Provides mathematically and semantically transparent key translation matching macOS Tahoe 26:
//! - Maps Command (Super/Meta) to standard desktop application accelerators while strictly
//!   preserving raw POSIX terminal signals (`Ctrl+C` = SIGINT, `Ctrl+Z` = SIGTSTP, `Ctrl+D` = EOF,
//!   `Ctrl+R` = reverse-i-search) in terminal emulators.
//! - Translates `Cmd+C`/`Cmd+V` in terminal emulators to clipboard copy/paste (`Ctrl+Shift+C`/`V`).
//! - Emits Cocoa/Emacs universal text editing bindings (`Ctrl+A/E/K/Y/D/T`, `Opt+Arrows`, `Cmd+Arrows`,
//!   `Opt+Backspace`, `Cmd+Backspace`) in standard GUI input fields.
//! - Tracks dynamic `Option` modifier state for real-time menu item label mutation.
//! - Intercepts global system shortcuts (Spotlight, App Switcher, Screenshots, Window Cycling).

#![allow(non_upper_case_globals)]

use bitflags::bitflags;
use std::collections::HashSet;

bitflags! {
    /// Active keyboard modifier mask matching Quartz & Cocoa conventions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ModifierState: u32 {
        /// No modifier keys active.
        const NONE = 0;
        /// Shift key depressed.
        const SHIFT = 1 << 0;
        /// Control key depressed.
        const CONTROL = 1 << 1;
        /// Option / Alt key depressed.
        const OPTION = 1 << 2;
        /// Command / Meta / Super key depressed.
        const COMMAND = 1 << 3;
        /// Caps Lock toggled active.
        const CAPS_LOCK = 1 << 4;
        /// Fn modifier depressed.
        const FN = 1 << 5;
    }
}

/// Category of the target focused window/application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetAppKind {
    /// Standard graphical application (browser, text editor, finder, settings).
    GuiApplication,
    /// Terminal emulator requiring raw POSIX signal preservation (qz-terminal, alacritty, etc.).
    TerminalEmulator,
}

/// Global system-level actions intercepted before passing to client surfaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {
    /// Toggle Spotlight search launcher (`Cmd+Space`).
    SpotlightSearch,
    /// Cycle application switcher (`Cmd+Tab` / `Cmd+Shift+Tab`).
    AppSwitcher {
        /// True if cycling forward (Tab), false if cycling backward (Shift+Tab).
        forward: bool,
    },
    /// Cycle windows of current active application (`Cmd+`` / `Cmd+Shift+``).
    WindowCycle {
        /// True if cycling forward, false if backward.
        forward: bool,
    },
    /// Capture all displays (`Cmd+Shift+3`).
    ScreenshotFull,
    /// Interactive crosshair / window capture (`Cmd+Shift+4`).
    ScreenshotCrop,
    /// Capture bar and PipeWire video recorder HUD (`Cmd+Shift+5`).
    ScreenshotHud,
    /// Immediate screen lock (`Ctrl+Cmd+Q`).
    LockSession,
    /// Summon Force Quit Applications dialog (`Opt+Cmd+Esc`).
    ForceQuitDialog,
    /// Toggle Mission Control (`F3` or `Ctrl+Up`).
    MissionControl,
    /// Toggle App Exposé (`Ctrl+Down`).
    AppExpose,
    /// Reveal Desktop (`F11` or thumb+3-finger pinch).
    ShowDesktop,
    /// Toggle Do Not Disturb (`Option+Click` on Clock or shortcut).
    ToggleDoNotDisturb,
    /// Fine 1/4-step volume adjustment (64 levels, e.g. +1 or -1 quarter steps).
    FineVolumeAdjust {
        /// Number of quarter-steps to adjust (+1 for volume up, -1 for down).
        quarter_steps: i32,
    },
    /// Fine 1/4-step display brightness adjustment (64 levels).
    FineBrightnessAdjust {
        /// Number of quarter-steps to adjust (+1 for brightness up, -1 for down).
        quarter_steps: i32,
    },
    /// Universal Emoji & Symbol picker popover (`Ctrl+Cmd+Space`).
    EmojiPicker,
    /// Local voice dictation toggle (Double-press `Fn`).
    VoiceDictation,
}

/// Action to take on an ingested key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyMappingResult {
    /// Event intercepted and handled exclusively by the compositor/desktop shell.
    Intercepted(SystemAction),
    /// Forward transformed keysym and modifier state to the Wayland client.
    ForwardTransformed {
        /// XKB keysym code.
        keysym: u32,
        /// Translated modifier mask.
        modifiers: ModifierState,
        /// UTF-8 text string to synthesize if applicable.
        text_payload: Option<String>,
    },
    /// Forward raw hardware keysym and unmodified modifier mask to the Wayland client.
    ForwardRaw {
        /// Raw hardware keysym code.
        keysym: u32,
        /// Hardware modifier mask.
        modifiers: ModifierState,
    },
    /// Discard/suppress key event entirely.
    Drop,
}

/// XKB Common keysym constant definitions used by the keymapper.
#[allow(non_upper_case_globals)]
pub mod xkb_keysyms {
    /// Space key
    pub const KEY_Space: u32 = 0x0020;
    /// Tab key
    pub const KEY_Tab: u32 = 0xff09;
    /// ISO Left Tab (Shift+Tab)
    pub const KEY_ISO_Left_Tab: u32 = 0xfe20;
    /// Grave / Tilde key (`)
    pub const KEY_grave: u32 = 0x0060;
    /// Return / Enter key
    pub const KEY_Return: u32 = 0xff0d;
    /// Escape key
    pub const KEY_Escape: u32 = 0xff1b;
    /// BackSpace key
    pub const KEY_BackSpace: u32 = 0xff08;
    /// Delete key
    pub const KEY_Delete: u32 = 0xffff;
    /// Home key
    pub const KEY_Home: u32 = 0xff50;
    /// End key
    pub const KEY_End: u32 = 0xff57;
    /// Left arrow
    pub const KEY_Left: u32 = 0xff51;
    /// Up arrow
    pub const KEY_Up: u32 = 0xff52;
    /// Right arrow
    pub const KEY_Right: u32 = 0xff53;
    /// Down arrow
    pub const KEY_Down: u32 = 0xff54;
    /// Function keys
    pub const KEY_F1: u32 = 0xffbe;
    /// F2
    pub const KEY_F2: u32 = 0xffbf;
    /// F3
    pub const KEY_F3: u32 = 0xffc0;
    /// F11
    pub const KEY_F11: u32 = 0xffc8;
    /// Letters
    pub const KEY_a: u32 = 0x0061;
    /// b
    pub const KEY_b: u32 = 0x0062;
    /// c
    pub const KEY_c: u32 = 0x0063;
    /// d
    pub const KEY_d: u32 = 0x0064;
    /// e
    pub const KEY_e: u32 = 0x0065;
    /// f
    pub const KEY_f: u32 = 0x0066;
    /// h
    pub const KEY_h: u32 = 0x0068;
    /// k
    pub const KEY_k: u32 = 0x006b;
    /// l
    pub const KEY_l: u32 = 0x006c;
    /// n
    pub const KEY_n: u32 = 0x006e;
    /// o
    pub const KEY_o: u32 = 0x006f;
    /// p
    pub const KEY_p: u32 = 0x0070;
    /// q
    pub const KEY_q: u32 = 0x0071;
    /// r
    pub const KEY_r: u32 = 0x0072;
    /// s
    pub const KEY_s: u32 = 0x0073;
    /// t
    pub const KEY_t: u32 = 0x0074;
    /// u
    pub const KEY_u: u32 = 0x0075;
    /// v
    pub const KEY_v: u32 = 0x0076;
    /// w
    pub const KEY_w: u32 = 0x0077;
    /// x
    pub const KEY_x: u32 = 0x0078;
    /// y
    pub const KEY_y: u32 = 0x0079;
    /// z
    pub const KEY_z: u32 = 0x007a;
    /// Numbers
    pub const KEY_3: u32 = 0x0033;
    /// 4
    pub const KEY_4: u32 = 0x0034;
    /// 5
    pub const KEY_5: u32 = 0x0035;
    /// Audio / Brightness media keys
    pub const XF86_AudioRaiseVolume: u32 = 0x1008ff13;
    /// AudioLowerVolume
    pub const XF86_AudioLowerVolume: u32 = 0x1008ff11;
    /// AudioMute
    pub const XF86_AudioMute: u32 = 0x1008ff12;
    /// MonBrightnessUp
    pub const XF86_MonBrightnessUp: u32 = 0x1008ff02;
    /// MonBrightnessDown
    pub const XF86_MonBrightnessDown: u32 = 0x1008ff03;
}

/// Intelligent keymapper maintaining stateful modifier tracking and Cocoa translation.
pub struct SmartKeymapper {
    known_terminals: HashSet<String>,
    current_modifiers: ModifierState,
    last_fn_press_time: f64,
    kill_ring_buffer: Option<String>,
}

impl Default for SmartKeymapper {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartKeymapper {
    /// Creates a keymapper pre-configured with known terminal emulator app IDs.
    pub fn new() -> Self {
        let mut known_terminals = HashSet::new();
        // Native and popular Wayland/X11 terminal emulators
        known_terminals.insert("qz-terminal".to_string());
        known_terminals.insert("org.quartz.terminal".to_string());
        known_terminals.insert("alacritty".to_string());
        known_terminals.insert("Alacritty".to_string());
        known_terminals.insert("kitty".to_string());
        known_terminals.insert("foot".to_string());
        known_terminals.insert("wezterm".to_string());
        known_terminals.insert("org.wezfurlong.wezterm".to_string());
        known_terminals.insert("gnome-terminal".to_string());
        known_terminals.insert("konsole".to_string());
        known_terminals.insert("xterm".to_string());
        known_terminals.insert("urxvt".to_string());

        Self {
            known_terminals,
            current_modifiers: ModifierState::NONE,
            last_fn_press_time: -10.0,
            kill_ring_buffer: None,
        }
    }

    /// Registers a client app identifier as a terminal emulator.
    pub fn register_terminal_app_id(&mut self, app_id: &str) {
        self.known_terminals.insert(app_id.to_string());
    }

    /// Classifies an application as GUI or Terminal.
    #[inline]
    pub fn classify_app(&self, app_id: Option<&str>) -> TargetAppKind {
        match app_id {
            Some(id) if self.known_terminals.contains(id) => TargetAppKind::TerminalEmulator,
            _ => TargetAppKind::GuiApplication,
        }
    }

    /// Ingests modifier updates and returns whether the `Option` key state has changed.
    pub fn update_modifiers(&mut self, new_modifiers: ModifierState) -> bool {
        let was_option = self.current_modifiers.contains(ModifierState::OPTION);
        let is_option = new_modifiers.contains(ModifierState::OPTION);
        self.current_modifiers = new_modifiers;
        was_option != is_option
    }

    /// Returns current active modifier state.
    #[inline]
    pub fn modifiers(&self) -> ModifierState {
        self.current_modifiers
    }

    /// Stores string into the internal Cocoa text kill ring.
    pub fn set_kill_buffer(&mut self, text: String) {
        self.kill_ring_buffer = Some(text);
    }

    /// Retrieves contents of the internal Cocoa text kill ring.
    #[inline]
    pub fn kill_buffer(&self) -> Option<&str> {
        self.kill_ring_buffer.as_deref()
    }

    /// Primary translation entry point for keyboard input events.
    pub fn translate_key(
        &mut self,
        keysym: u32,
        pressed: bool,
        target_kind: TargetAppKind,
        timestamp_secs: f64,
    ) -> KeyMappingResult {
        use xkb_keysyms::*;

        let mods = self.current_modifiers;
        let is_cmd = mods.contains(ModifierState::COMMAND);
        let is_ctrl = mods.contains(ModifierState::CONTROL);
        let is_opt = mods.contains(ModifierState::OPTION);
        let is_shift = mods.contains(ModifierState::SHIFT);

        // 1. Double Fn key press detection for Voice Dictation
        if keysym == 0x1008ff2b || keysym == 0xff63 { // XF86WakeUp / Fn key symbol
            if pressed {
                let dt = timestamp_secs - self.last_fn_press_time;
                self.last_fn_press_time = timestamp_secs;
                if dt > 0.05 && dt < 0.35 {
                    return KeyMappingResult::Intercepted(SystemAction::VoiceDictation);
                }
            }
            return KeyMappingResult::Drop;
        }

        // 2. Hardware media key fine adjustment (Shift + Option + Volume / Brightness)
        if pressed && is_shift && is_opt {
            match keysym {
                XF86_AudioRaiseVolume => {
                    return KeyMappingResult::Intercepted(SystemAction::FineVolumeAdjust { quarter_steps: 1 });
                }
                XF86_AudioLowerVolume => {
                    return KeyMappingResult::Intercepted(SystemAction::FineVolumeAdjust { quarter_steps: -1 });
                }
                XF86_MonBrightnessUp => {
                    return KeyMappingResult::Intercepted(SystemAction::FineBrightnessAdjust { quarter_steps: 1 });
                }
                XF86_MonBrightnessDown => {
                    return KeyMappingResult::Intercepted(SystemAction::FineBrightnessAdjust { quarter_steps: -1 });
                }
                _ => {}
            }
        }

        // 3. Global System Shortcuts (intercepted before any application delivery)
        if pressed {
            // Spotlight: Cmd + Space
            if is_cmd && !is_ctrl && !is_opt && !is_shift && keysym == KEY_Space {
                return KeyMappingResult::Intercepted(SystemAction::SpotlightSearch);
            }

            // Emoji Picker: Ctrl + Cmd + Space
            if is_cmd && is_ctrl && !is_opt && !is_shift && keysym == KEY_Space {
                return KeyMappingResult::Intercepted(SystemAction::EmojiPicker);
            }

            // Lock Screen: Ctrl + Cmd + Q
            if is_cmd && is_ctrl && !is_opt && !is_shift && keysym == KEY_q {
                return KeyMappingResult::Intercepted(SystemAction::LockSession);
            }

            // Force Quit Applications Dialog: Option + Cmd + Escape
            if is_cmd && is_opt && !is_ctrl && keysym == KEY_Escape {
                return KeyMappingResult::Intercepted(SystemAction::ForceQuitDialog);
            }

            // Screenshots: Cmd + Shift + 3 / 4 / 5
            if is_cmd && is_shift && !is_ctrl && !is_opt {
                match keysym {
                    KEY_3 => return KeyMappingResult::Intercepted(SystemAction::ScreenshotFull),
                    KEY_4 => return KeyMappingResult::Intercepted(SystemAction::ScreenshotCrop),
                    KEY_5 => return KeyMappingResult::Intercepted(SystemAction::ScreenshotHud),
                    _ => {}
                }
            }

            // App Switcher: Cmd + Tab / Cmd + Shift + Tab
            if is_cmd && !is_ctrl && !is_opt && (keysym == KEY_Tab || keysym == KEY_ISO_Left_Tab) {
                return KeyMappingResult::Intercepted(SystemAction::AppSwitcher {
                    forward: !is_shift,
                });
            }

            // Same-app Window Cycling: Cmd + `
            if is_cmd && !is_ctrl && !is_opt && keysym == KEY_grave {
                return KeyMappingResult::Intercepted(SystemAction::WindowCycle {
                    forward: !is_shift,
                });
            }

            // Mission Control: F3 or Ctrl + Up
            if (keysym == KEY_F3 && !is_cmd && !is_ctrl) || (is_ctrl && !is_cmd && keysym == KEY_Up) {
                return KeyMappingResult::Intercepted(SystemAction::MissionControl);
            }

            // App Exposé: Ctrl + Down
            if is_ctrl && !is_cmd && keysym == KEY_Down {
                return KeyMappingResult::Intercepted(SystemAction::AppExpose);
            }

            // Reveal Desktop: F11
            if keysym == KEY_F11 && !is_cmd && !is_ctrl {
                return KeyMappingResult::Intercepted(SystemAction::ShowDesktop);
            }
        }

        // 4. Terminal Emulator Routing Rules:
        // Strictly preserve hardware Ctrl signals (SIGINT, SIGTSTP, EOF, etc.)
        if target_kind == TargetAppKind::TerminalEmulator {
            if is_ctrl && !is_cmd {
                // Forward raw Ctrl combinations directly so the shell receives exact POSIX control chars
                return KeyMappingResult::ForwardRaw {
                    keysym,
                    modifiers: mods,
                };
            }

            // Terminal Command-accelerators translation:
            // Cmd+C -> Ctrl+Shift+C (Copy)
            // Cmd+V -> Ctrl+Shift+V (Paste)
            if is_cmd && !is_ctrl && !is_opt {
                match keysym {
                    KEY_c => {
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_c,
                            modifiers: ModifierState::CONTROL | ModifierState::SHIFT,
                            text_payload: None,
                        };
                    }
                    KEY_v => {
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_v,
                            modifiers: ModifierState::CONTROL | ModifierState::SHIFT,
                            text_payload: None,
                        };
                    }
                    _ => {}
                }
            }

            // All other terminal keys pass through cleanly
            return KeyMappingResult::ForwardRaw {
                keysym,
                modifiers: mods,
            };
        }

        // 5. GUI Applications (Non-Terminal): Cocoa Text & Navigation Engine
        if target_kind == TargetAppKind::GuiApplication && pressed {
            // A. Universal Cocoa / Emacs Control shortcuts in text inputs
            if is_ctrl && !is_cmd && !is_opt && !is_shift {
                match keysym {
                    KEY_a => {
                        // Move to beginning of line (Home)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Home,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_e => {
                        // Move to end of line (End)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_End,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_d => {
                        // Forward delete
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Delete,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_b => {
                        // Backward character (Left)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Left,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_f => {
                        // Forward character (Right)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Right,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_n => {
                        // Next line (Down)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Down,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_p => {
                        // Previous line (Up)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Up,
                            modifiers: ModifierState::NONE,
                            text_payload: None,
                        };
                    }
                    KEY_y => {
                        // Yank (Paste from internal kill buffer if present)
                        if let Some(ref text) = self.kill_ring_buffer {
                            return KeyMappingResult::ForwardTransformed {
                                keysym: KEY_v,
                                modifiers: ModifierState::CONTROL,
                                text_payload: Some(text.clone()),
                            };
                        }
                    }
                    _ => {}
                }
            }

            // B. Option + Arrows / Backspace: Word jump and word deletion
            if is_opt && !is_cmd && !is_ctrl {
                match keysym {
                    KEY_Left => {
                        // Jump back one word (Ctrl+Left on Linux)
                        let target_mods = if is_shift {
                            ModifierState::CONTROL | ModifierState::SHIFT
                        } else {
                            ModifierState::CONTROL
                        };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Left,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_Right => {
                        // Jump forward one word (Ctrl+Right on Linux)
                        let target_mods = if is_shift {
                            ModifierState::CONTROL | ModifierState::SHIFT
                        } else {
                            ModifierState::CONTROL
                        };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Right,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_BackSpace => {
                        // Delete word backward (Ctrl+BackSpace)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_BackSpace,
                            modifiers: ModifierState::CONTROL,
                            text_payload: None,
                        };
                    }
                    KEY_Delete => {
                        // Delete word forward (Ctrl+Delete)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Delete,
                            modifiers: ModifierState::CONTROL,
                            text_payload: None,
                        };
                    }
                    _ => {}
                }
            }

            // C. Command + Arrows / Backspace: Line/Document navigation and Line deletion
            if is_cmd && !is_ctrl && !is_opt {
                match keysym {
                    KEY_Left => {
                        // Line start (Home)
                        let target_mods = if is_shift { ModifierState::SHIFT } else { ModifierState::NONE };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Home,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_Right => {
                        // Line end (End)
                        let target_mods = if is_shift { ModifierState::SHIFT } else { ModifierState::NONE };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_End,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_Up => {
                        // Document start (Ctrl+Home)
                        let target_mods = if is_shift {
                            ModifierState::CONTROL | ModifierState::SHIFT
                        } else {
                            ModifierState::CONTROL
                        };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_Home,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_Down => {
                        // Document end (Ctrl+End)
                        let target_mods = if is_shift {
                            ModifierState::CONTROL | ModifierState::SHIFT
                        } else {
                            ModifierState::CONTROL
                        };
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_End,
                            modifiers: target_mods,
                            text_payload: None,
                        };
                    }
                    KEY_BackSpace => {
                        // Delete to start of line (Shift+Home then BackSpace)
                        return KeyMappingResult::ForwardTransformed {
                            keysym: KEY_BackSpace,
                            modifiers: ModifierState::CONTROL | ModifierState::SHIFT,
                            text_payload: None,
                        };
                    }
                    _ => {}
                }
            }

            // D. Standard GUI Application Accelerators: Cmd -> Ctrl mapping
            // (Cmd+C -> Ctrl+C, Cmd+V -> Ctrl+V, Cmd+Z -> Ctrl+Z, Cmd+A -> Ctrl+A, etc.)
            if is_cmd && !is_ctrl && !is_opt {
                match keysym {
                    KEY_c | KEY_v | KEY_x | KEY_z | KEY_a | KEY_s | KEY_o | KEY_f | KEY_w | KEY_q | KEY_r | KEY_t | KEY_n => {
                        let mut mapped_mods = ModifierState::CONTROL;
                        if is_shift {
                            mapped_mods |= ModifierState::SHIFT;
                        }
                        return KeyMappingResult::ForwardTransformed {
                            keysym,
                            modifiers: mapped_mods,
                            text_payload: None,
                        };
                    }
                    _ => {}
                }
            }
        }

        // Default: Forward raw event without modification
        KeyMappingResult::ForwardRaw {
            keysym,
            modifiers: mods,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::xkb_keysyms::*;

    #[test]
    fn test_terminal_sigint_preserved() {
        let mut km = SmartKeymapper::new();
        km.update_modifiers(ModifierState::CONTROL);

        // Physical Ctrl+C in terminal must remain raw Ctrl+C
        let res = km.translate_key(KEY_c, true, TargetAppKind::TerminalEmulator, 1.0);
        assert_eq!(
            res,
            KeyMappingResult::ForwardRaw {
                keysym: KEY_c,
                modifiers: ModifierState::CONTROL,
            }
        );
    }

    #[test]
    fn test_terminal_cmd_c_v_translated_to_clipboard() {
        let mut km = SmartKeymapper::new();
        km.update_modifiers(ModifierState::COMMAND);

        let res_c = km.translate_key(KEY_c, true, TargetAppKind::TerminalEmulator, 1.0);
        assert_eq!(
            res_c,
            KeyMappingResult::ForwardTransformed {
                keysym: KEY_c,
                modifiers: ModifierState::CONTROL | ModifierState::SHIFT,
                text_payload: None,
            }
        );

        let res_v = km.translate_key(KEY_v, true, TargetAppKind::TerminalEmulator, 1.0);
        assert_eq!(
            res_v,
            KeyMappingResult::ForwardTransformed {
                keysym: KEY_v,
                modifiers: ModifierState::CONTROL | ModifierState::SHIFT,
                text_payload: None,
            }
        );
    }

    #[test]
    fn test_gui_cmd_c_translated_to_ctrl_c() {
        let mut km = SmartKeymapper::new();
        km.update_modifiers(ModifierState::COMMAND);

        let res = km.translate_key(KEY_c, true, TargetAppKind::GuiApplication, 1.0);
        assert_eq!(
            res,
            KeyMappingResult::ForwardTransformed {
                keysym: KEY_c,
                modifiers: ModifierState::CONTROL,
                text_payload: None,
            }
        );
    }

    #[test]
    fn test_cocoa_emacs_line_navigation() {
        let mut km = SmartKeymapper::new();
        km.update_modifiers(ModifierState::CONTROL);

        // Ctrl+A -> Home
        let res_a = km.translate_key(KEY_a, true, TargetAppKind::GuiApplication, 1.0);
        assert_eq!(
            res_a,
            KeyMappingResult::ForwardTransformed {
                keysym: KEY_Home,
                modifiers: ModifierState::NONE,
                text_payload: None,
            }
        );

        // Ctrl+E -> End
        let res_e = km.translate_key(KEY_e, true, TargetAppKind::GuiApplication, 1.0);
        assert_eq!(
            res_e,
            KeyMappingResult::ForwardTransformed {
                keysym: KEY_End,
                modifiers: ModifierState::NONE,
                text_payload: None,
            }
        );
    }

    #[test]
    fn test_system_shortcut_interception() {
        let mut km = SmartKeymapper::new();

        // Cmd+Space -> Spotlight
        km.update_modifiers(ModifierState::COMMAND);
        let res_spotlight = km.translate_key(KEY_Space, true, TargetAppKind::GuiApplication, 1.0);
        assert_eq!(res_spotlight, KeyMappingResult::Intercepted(SystemAction::SpotlightSearch));

        // Cmd+Shift+3 -> ScreenshotFull
        km.update_modifiers(ModifierState::COMMAND | ModifierState::SHIFT);
        let res_ss = km.translate_key(KEY_3, true, TargetAppKind::GuiApplication, 1.0);
        assert_eq!(res_ss, KeyMappingResult::Intercepted(SystemAction::ScreenshotFull));
    }
}
