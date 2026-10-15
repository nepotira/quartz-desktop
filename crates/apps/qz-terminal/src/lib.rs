//! # QuartzLinux Terminal Application (`qz-terminal`)
//!
//! Production-grade GPU-accelerated Liquid Glass terminal emulator for QuartzLinux (`qz-desktop`),
//! built to be visually, mathematically, physically, and functionally indistinguishable
//! from **macOS Tahoe 26 Terminal (`Terminal.app`)**.
//!
//! ### Core Subsystems:
//! - [`grid`]: High-performance character cell grid, 24-bit RGB truecolor & ANSI/VT100 escape sequence parser,
//!   deep 10,000-line scrollback buffer, and configurable cursor styles (Block, Underline, Beam).
//! - [`dnd`]: Shell metacharacter path escaping engine (supporting backslash and single-quoted modes)
//!   and percent-decoded URI list handling for files and folders dragged from Finder or the Desktop.
//! - [`pty`]: Pseudo-terminal (PTY) session management, shell detection (`$SHELL`, `/bin/zsh`, `/bin/bash`),
//!   and dynamic window dimension resizing (`TIOCSWINSZ`).
//! - [`theme`]: Liquid Glass visual profiles (Tahoe Glass, Pro, Silver, Homebrew, Ocean) with background
//!   Kawase blur ($\sigma = 24.0\text{px}$), refraction ($\eta = 1.52$), and typography zoom controls.
//! - [`window`]: Liquid Glass multi-tab and split-pane window controller with native tab bar integration
//!   and in-terminal search (`Cmd+F`).

#![deny(missing_docs)]

pub mod dnd;
pub mod grid;
pub mod pty;
pub mod theme;
pub mod window;

// Re-exports for convenient top-level access
pub use dnd::{
    decode_percent_encoded, escape_backslash, escape_single_quoted, process_drag_drop_paths,
    uri_to_path, EscapingStrategy,
};
pub use grid::{standard_ansi_color, Cell, CellFlags, Cursor, CursorStyle, TerminalGrid};
pub use pty::{PtySession, SessionState};
pub use theme::TerminalProfile;
pub use window::{SplitOrientation, TerminalTab, TerminalWindowController};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_terminal_end_to_end_workflow() {
        // 1. Initialize Terminal Window Controller in user home directory
        let mut win = TerminalWindowController::new("/Users/nepo", Vec2::new(900.0, 600.0));
        assert_eq!(win.tab_count(), 1);
        assert_eq!(win.profile.name, "Tahoe Glass");

        // 2. Drag a folder from Finder with spaces and parentheses
        let dropped_uri = "file:///Volumes/MacTux/Projects/Alpha%20%28v2%29";
        let escaped = win.handle_drop_paths(dropped_uri);
        assert_eq!(escaped, "/Volumes/MacTux/Projects/Alpha\\ \\(v2\\)");

        // 3. Receive output from shell and poll into grid
        let tab = win.active_tab_mut().expect("active tab exists");
        tab.pty.receive_output_bytes(b"\x1b[32mSuccess\x1b[0m\n");
        tab.poll_and_render();

        let first_cell = tab.grid.cell(0, 0).expect("cell exists");
        assert_eq!(first_cell.character, 'S');
        assert_eq!(first_cell.fg, [13, 188, 121, 255]); // Green

        // 4. Zoom font size
        win.profile.zoom_in();
        assert_eq!(win.profile.font_size_pt, 14.0);

        // 5. Open new tab and verify isolation
        let tab2_idx = win.new_tab("/Users/nepo/Documents");
        assert_eq!(tab2_idx, 1);
        assert_eq!(win.tab_count(), 2);
        assert_eq!(win.active_tab().unwrap().working_dir, "/Users/nepo/Documents");
    }
}
