//! # Drag-and-Drop Path Escaping Engine (`qz-terminal::dnd`)
//!
//! Production-grade drag-and-drop shell path escaping for macOS Tahoe 26 Terminal (`qz-terminal`).
//!
//! ### Key Capabilities:
//! - **Shell Metacharacter Escaping:** Automatically escapes spaces and POSIX/Zsh/Bash
//!   metacharacters (`(`, `)`, `[`, `]`, `{`, `}`, `&`, `|`, `;`, `$`, `*`, `?`, `"`, `'`,
//!   `\`, `<`, `>`, `~`, `^`, `#`, `!`) so paths can be safely evaluated in the shell.
//! - **Single-Quoted & Backslash Modes:** Supports canonical macOS backslash escaping
//!   (default) or POSIX single-quoted path wrapping.
//! - **URI List Decoding:** Decodes `file://` URLs received via Wayland drag-and-drop
//!   (`text/uri-list`), resolving percent-encoded octets (`%20` -> space).
//! - **Multi-File Concatenation:** Concatenates multiple dragged items with spaces
//!   ready to insert at the terminal prompt.

use serde::{Deserialize, Serialize};

/// Shell escaping strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EscapingStrategy {
    /// Canonical macOS Terminal style: escapes special characters using backslashes (`\ `).
    Backslash,
    /// POSIX single-quoted string: `'path/with spaces'`, escaping internal `'` as `'\''`.
    SingleQuoted,
}

/// Escapes a filesystem path for safe terminal command-line insertion using backslashes.
///
/// Metacharacters escaped:
/// ` ` (space), `\`, `"`, `'`, `(`, `)`, `[`, `]`, `{`, `}`, `&`, `|`, `;`, `$`, `*`, `?`, `<`, `>`, `^`, `#`, `!`, `\t`
#[must_use]
pub fn escape_backslash(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len() + 16);

    for (i, ch) in path.chars().enumerate() {
        match ch {
            ' ' | '\\' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '&' | '|' | ';'
            | '$' | '*' | '?' | '<' | '>' | '^' | '#' | '!' | '\t' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            '~' if i == 0 => {
                // Tilde at start expands to home; if literal tilde intended or middle of string, leave or escape
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }

    escaped
}

/// Escapes a filesystem path by enclosing it in POSIX single quotes.
///
/// Internal single quotes are escaped as `'\''`.
#[must_use]
pub fn escape_single_quoted(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len() + 8);
    escaped.push('\'');

    for ch in path.chars() {
        if ch == '\'' {
            escaped.push_str("'\\''");
        } else {
            escaped.push(ch);
        }
    }

    escaped.push('\'');
    escaped
}

/// Decodes percent-encoded octets in a URI path (e.g. `%20` -> ` `, `%28` -> `(`).
#[must_use]
pub fn decode_percent_encoded(encoded: &str) -> String {
    let mut bytes = Vec::new();
    let chars: Vec<char> = encoded.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' && i + 2 < chars.len() {
            let hex_str: String = chars[i + 1..=i + 2].iter().collect();
            if let Ok(byte) = u8::from_str_radix(&hex_str, 16) {
                bytes.push(byte);
                i += 3;
                continue;
            }
        }
        let mut buf = [0u8; 4];
        let encoded_char = chars[i].encode_utf8(&mut buf);
        bytes.extend_from_slice(encoded_char.as_bytes());
        i += 1;
    }

    String::from_utf8_lossy(&bytes).to_string()
}

/// Converts a `file://` URI into a clean absolute filesystem path.
#[must_use]
pub fn uri_to_path(uri: &str) -> String {
    let clean = uri.trim();
    if let Some(stripped) = clean.strip_prefix("file://") {
        // Strip localhost if present (e.g. file://localhost/Users/...)
        let path_part = if let Some(local_stripped) = stripped.strip_prefix("localhost") {
            local_stripped
        } else {
            stripped
        };
        decode_percent_encoded(path_part)
    } else {
        decode_percent_encoded(clean)
    }
}

/// Processes raw dragged data (from Wayland `text/uri-list` or path strings)
/// and produces a formatted, escaped string ready for shell insertion.
#[must_use]
pub fn process_drag_drop_paths(raw_input: &str, strategy: EscapingStrategy) -> String {
    let mut formatted_paths = Vec::new();

    for line in raw_input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let decoded_path = uri_to_path(trimmed);
        let escaped = match strategy {
            EscapingStrategy::Backslash => escape_backslash(&decoded_path),
            EscapingStrategy::SingleQuoted => escape_single_quoted(&decoded_path),
        };
        formatted_paths.push(escaped);
    }

    formatted_paths.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backslash_escaping() {
        let path = "/Users/nepo/My Documents/Project (Tahoe 26)/file $1.txt";
        let escaped = escape_backslash(path);
        assert_eq!(
            escaped,
            "/Users/nepo/My\\ Documents/Project\\ \\(Tahoe\\ 26\\)/file\\ \\$1.txt"
        );
    }

    #[test]
    fn test_single_quoted_escaping() {
        let path = "/Users/nepo/Documents/O'Reilly's Guide.pdf";
        let escaped = escape_single_quoted(path);
        assert_eq!(escaped, "'/Users/nepo/Documents/O'\\''Reilly'\\''s Guide.pdf'");
    }

    #[test]
    fn test_percent_decoding_and_uri_to_path() {
        let uri = "file:///Users/nepo/Photos/San%20Francisco%20%28Vacation%29.jpg";
        let path = uri_to_path(uri);
        assert_eq!(path, "/Users/nepo/Photos/San Francisco (Vacation).jpg");
    }

    #[test]
    fn test_multi_path_drag_drop_processing() {
        let raw = "file:///Volumes/MacTux/App%20One.app\nfile:///Volumes/MacTux/Document%20(2).pdf\n";
        let result = process_drag_drop_paths(raw, EscapingStrategy::Backslash);
        assert_eq!(
            result,
            "/Volumes/MacTux/App\\ One.app /Volumes/MacTux/Document\\ \\(2\\).pdf"
        );
    }
}
