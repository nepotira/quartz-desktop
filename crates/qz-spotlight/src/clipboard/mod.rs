//! # Native Clipboard History Subsystem
//!
//! Ring-buffered clipboard history with automatic color swatch detection,
//! item pinning protection, full-text fuzzy filtering, and rich format support.

use std::time::{SystemTime, UNIX_EPOCH};

/// Content payload types captured in the clipboard history.
#[derive(Debug, Clone, PartialEq)]
pub enum ClipboardContent {
    /// Plain UTF-8 text.
    PlainText(String),
    /// Rich text with both plain fallback and markup (RTF or HTML).
    RichText {
        /// Plain text fallback.
        plain: String,
        /// Rich markup payload.
        markup: String,
    },
    /// Detected color value with live RGB swatch preview.
    ColorSwatch {
        /// Canonical hex string (e.g. "#FF8000").
        hex: String,
        /// Normalized red component $[0.0, 1.0]$.
        r: f32,
        /// Normalized green component $[0.0, 1.0]$.
        g: f32,
        /// Normalized blue component $[0.0, 1.0]$.
        b: f32,
        /// Normalized alpha component $[0.0, 1.0]$.
        a: f32,
    },
    /// Image preview captured from screenshots or copy buffer.
    Image {
        /// Pixel width.
        width: u32,
        /// Pixel height.
        height: u32,
        /// RGBA thumbnail byte buffer.
        rgba_preview: Vec<u8>,
    },
    /// List of file paths copied in Finder.
    Files(Vec<String>),
}

/// An entry stored in the clipboard history.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipboardEntry {
    /// Unique item identifier.
    pub id: u64,
    /// Unix timestamp when item was copied.
    pub timestamp_secs: u64,
    /// Whether user pinned this item (exempt from ring buffer eviction).
    pub is_pinned: bool,
    /// Captured content payload.
    pub content: ClipboardContent,
    /// Primary search summary text.
    pub search_text: String,
}

impl ClipboardEntry {
    /// Creates a new entry, automatically detecting whether plain text represents a color swatch.
    pub fn new(id: u64, content: ClipboardContent) -> Self {
        let timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // If content is plain text, check for color patterns
        let resolved_content = match content {
            ClipboardContent::PlainText(ref text) => {
                if let Some(swatch) = Self::detect_color_swatch(text) {
                    swatch
                } else {
                    content
                }
            }
            other => other,
        };

        let search_text = match &resolved_content {
            ClipboardContent::PlainText(t) => t.clone(),
            ClipboardContent::RichText { plain, .. } => plain.clone(),
            ClipboardContent::ColorSwatch { hex, .. } => hex.clone(),
            ClipboardContent::Image { width, height, .. } => format!("Image {width}x{height}"),
            ClipboardContent::Files(paths) => paths.join(", "),
        };

        Self {
            id,
            timestamp_secs,
            is_pinned: false,
            content: resolved_content,
            search_text,
        }
    }

    /// Attempts to parse common CSS and hex color representations.
    pub fn detect_color_swatch(text: &str) -> Option<ClipboardContent> {
        let s = text.trim();

        // Check Hex #RGB, #RGBA, #RRGGBB, #RRGGBBAA
        if let Some(stripped) = s.strip_prefix('#') {
            match stripped.len() {
                3 => {
                    // #RGB
                    let r = u8::from_str_radix(&stripped[0..1].repeat(2), 16).ok()?;
                    let g = u8::from_str_radix(&stripped[1..2].repeat(2), 16).ok()?;
                    let b = u8::from_str_radix(&stripped[2..3].repeat(2), 16).ok()?;
                    return Some(ClipboardContent::ColorSwatch {
                        hex: format!("#{r:02X}{g:02X}{b:02X}"),
                        r: r as f32 / 255.0,
                        g: g as f32 / 255.0,
                        b: b as f32 / 255.0,
                        a: 1.0,
                    });
                }
                6 => {
                    // #RRGGBB
                    let r = u8::from_str_radix(&stripped[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&stripped[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&stripped[4..6], 16).ok()?;
                    return Some(ClipboardContent::ColorSwatch {
                        hex: format!("#{r:02X}{g:02X}{b:02X}"),
                        r: r as f32 / 255.0,
                        g: g as f32 / 255.0,
                        b: b as f32 / 255.0,
                        a: 1.0,
                    });
                }
                8 => {
                    // #RRGGBBAA
                    let r = u8::from_str_radix(&stripped[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&stripped[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&stripped[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&stripped[6..8], 16).ok()?;
                    return Some(ClipboardContent::ColorSwatch {
                        hex: format!("#{r:02X}{g:02X}{b:02X}{a:02X}"),
                        r: r as f32 / 255.0,
                        g: g as f32 / 255.0,
                        b: b as f32 / 255.0,
                        a: a as f32 / 255.0,
                    });
                }
                _ => {}
            }
        }

        // Check rgb(r, g, b)
        if s.starts_with("rgb(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() == 3 {
                let r: u8 = parts[0].parse().ok()?;
                let g: u8 = parts[1].parse().ok()?;
                let b: u8 = parts[2].parse().ok()?;
                return Some(ClipboardContent::ColorSwatch {
                    hex: format!("#{r:02X}{g:02X}{b:02X}"),
                    r: r as f32 / 255.0,
                    g: g as f32 / 255.0,
                    b: b as f32 / 255.0,
                    a: 1.0,
                });
            }
        }

        None
    }
}

/// Ring-buffered clipboard history storage.
pub struct ClipboardHistory {
    entries: Vec<ClipboardEntry>,
    capacity: usize,
    next_id: u64,
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new(200)
    }
}

impl ClipboardHistory {
    /// Creates a new clipboard history with the specified maximum capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
            next_id: 1,
        }
    }

    /// Maximum entry capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Number of active items currently stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether history is currently empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Records a new clipboard copy into history, evicting the oldest unpinned item if full.
    pub fn push(&mut self, content: ClipboardContent) -> u64 {
        // Dedup: check if identical to the most recent entry
        if let Some(last) = self.entries.first() {
            if last.content == content {
                return last.id;
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let entry = ClipboardEntry::new(id, content);

        if self.entries.len() >= self.capacity {
            // Evict oldest unpinned entry (search from back)
            if let Some(evict_idx) = self.entries.iter().rposition(|e| !e.is_pinned) {
                self.entries.remove(evict_idx);
            }
        }

        self.entries.insert(0, entry);
        id
    }

    /// Toggles pinned state for the entry with the given ID.
    pub fn toggle_pin(&mut self, id: u64) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.is_pinned = !entry.is_pinned;
            entry.is_pinned
        } else {
            false
        }
    }

    /// Removes an entry by ID.
    pub fn remove(&mut self, id: u64) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| e.id == id) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clears all unpinned entries, preserving pinned favorites.
    pub fn clear_unpinned(&mut self) {
        self.entries.retain(|e| e.is_pinned);
    }

    /// Searches clipboard items matching a query string.
    pub fn search(&self, query: &str) -> Vec<&ClipboardEntry> {
        let trimmed = query.trim().to_lowercase();
        if trimmed.is_empty() {
            return self.entries.iter().collect();
        }

        self.entries
            .iter()
            .filter(|e| e.search_text.to_lowercase().contains(&trimmed))
            .collect()
    }

    /// Returns a slice of all stored entries.
    pub fn entries(&self) -> &[ClipboardEntry] {
        &self.entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_color_swatch_detection() {
        let mut history = ClipboardHistory::new(10);
        let id = history.push(ClipboardContent::PlainText("#FF5500".into()));

        let entry = history.entries().iter().find(|e| e.id == id).unwrap();
        match &entry.content {
            ClipboardContent::ColorSwatch { hex, r, g, b, a } => {
                assert_eq!(hex, "#FF5500");
                assert_eq!(*r, 1.0);
                assert!((g - 0.333).abs() < 0.01);
                assert_eq!(*b, 0.0);
                assert_eq!(*a, 1.0);
            }
            _ => panic!("Expected ColorSwatch variant"),
        }
    }

    #[test]
    fn test_clipboard_pin_protection_from_eviction() {
        let mut history = ClipboardHistory::new(3);

        let id1 = history.push(ClipboardContent::PlainText("Item 1".into()));
        let _id2 = history.push(ClipboardContent::PlainText("Item 2".into()));
        let _id3 = history.push(ClipboardContent::PlainText("Item 3".into()));

        // Pin Item 1
        assert!(history.toggle_pin(id1));

        // Push Item 4 - should evict oldest UNPINNED item (Item 2)
        let _id4 = history.push(ClipboardContent::PlainText("Item 4".into()));

        let ids: Vec<u64> = history.entries().iter().map(|e| e.id).collect();
        // Item 1 must remain preserved!
        assert!(ids.contains(&id1));
        assert_eq!(history.len(), 3);
    }
}
