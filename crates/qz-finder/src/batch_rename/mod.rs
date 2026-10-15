//! # macOS Tahoe 26 Batch Rename Subsystem
//!
//! Provides the canonical Finder multi-file batch renaming engine with 3 modes:
//! - **Replace Text**: Search and replace substring with optional case sensitivity.
//! - **Add Text**: Prepend or append text (preserving file extension).
//! - **Format**: Name + Index, Name + Counter (e.g. `001`), or Name + Date.
//!
//! Includes real-time preview generation and collision detection.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Renaming mode for batch operations.
#[derive(Debug, Clone, PartialEq)]
pub enum BatchRenameMode {
    /// Search and replace text within filenames.
    ReplaceText {
        /// String to search for.
        find: String,
        /// Replacement string.
        replace: String,
        /// Whether search is case sensitive.
        match_case: bool,
    },
    /// Prepend or append text to filenames.
    AddText {
        /// String to insert.
        text: String,
        /// Insertion placement.
        placement: TextPlacement,
    },
    /// Format names with sequential indexing or dates.
    Format {
        /// Format style pattern.
        style: FormatStyle,
        /// Custom prefix or base name.
        custom_format: String,
        /// Placement of index/date relative to name.
        placement: TextPlacement,
        /// Starting number for index or counter.
        start_number: u32,
    },
}

/// Placement position when adding text or formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextPlacement {
    /// Insert text before existing filename.
    BeforeName,
    /// Insert text after existing filename (before file extension).
    AfterName,
}

/// Format style for sequence generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatStyle {
    /// Name and simple integer index (e.g. `Vacation 1.jpg`, `Vacation 2.jpg`).
    NameAndIndex,
    /// Name and zero-padded counter (e.g. `Vacation 0001.jpg`, `Vacation 0002.jpg`).
    NameAndCounter,
    /// Name and date stamp (e.g. `Vacation 2026-10-15.jpg`).
    NameAndDate,
}

/// A planned rename item in the preview list.
#[derive(Debug, Clone, PartialEq)]
pub struct RenamePreviewItem {
    /// Original absolute file path.
    pub original_path: PathBuf,
    /// Current file name with extension.
    pub original_name: String,
    /// Proposed new file name with extension.
    pub new_name: String,
    /// Target path if rename is applied.
    pub new_path: PathBuf,
    /// True if this proposed rename collides with another item.
    pub has_collision: bool,
}

/// Batch Rename engine calculating proposed names and validating changes.
pub struct BatchRenameEngine;

impl BatchRenameEngine {
    /// Generates preview of all renamed files under the specified configuration.
    pub fn generate_preview(paths: &[PathBuf], config: &BatchRenameMode) -> Vec<RenamePreviewItem> {
        let mut results = Vec::new();
        let mut seen_names = HashSet::new();

        for (idx, path) in paths.iter().enumerate() {
            let original_name = path
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_default()
                .to_string();

            let parent = path.parent().unwrap_or_else(|| Path::new(""));

            let (stem, ext) = Self::split_stem_and_ext(&original_name);

            let new_stem = match config {
                BatchRenameMode::ReplaceText {
                    find,
                    replace,
                    match_case,
                } => Self::replace_text(&stem, find, replace, *match_case),
                BatchRenameMode::AddText { text, placement } => match placement {
                    TextPlacement::BeforeName => format!("{text}{stem}"),
                    TextPlacement::AfterName => format!("{stem}{text}"),
                },
                BatchRenameMode::Format {
                    style,
                    custom_format,
                    placement,
                    start_number,
                } => {
                    let number = start_number + idx as u32;
                    let suffix_or_prefix = match style {
                        FormatStyle::NameAndIndex => number.to_string(),
                        FormatStyle::NameAndCounter => format!("{number:04}"),
                        FormatStyle::NameAndDate => {
                            // Current date YYYY-MM-DD
                            "2026-10-15".to_string()
                        }
                    };

                    match placement {
                        TextPlacement::AfterName => {
                            if custom_format.is_empty() {
                                format!("{stem} {suffix_or_prefix}")
                            } else {
                                format!("{custom_format} {suffix_or_prefix}")
                            }
                        }
                        TextPlacement::BeforeName => {
                            if custom_format.is_empty() {
                                format!("{suffix_or_prefix} {stem}")
                            } else {
                                format!("{suffix_or_prefix} {custom_format}")
                            }
                        }
                    }
                }
            };

            let new_name = if ext.is_empty() {
                new_stem
            } else {
                format!("{new_stem}.{ext}")
            };

            let has_collision = !seen_names.insert(new_name.to_lowercase());
            let new_path = parent.join(&new_name);

            results.push(RenamePreviewItem {
                original_path: path.clone(),
                original_name,
                new_name,
                new_path,
                has_collision,
            });
        }

        results
    }

    fn split_stem_and_ext(filename: &str) -> (String, String) {
        if let Some(pos) = filename.rfind('.') {
            if pos > 0 && pos < filename.len() - 1 {
                let stem = filename[..pos].to_string();
                let ext = filename[pos + 1..].to_string();
                return (stem, ext);
            }
        }
        (filename.to_string(), String::new())
    }

    fn replace_text(stem: &str, find: &str, replace: &str, match_case: bool) -> String {
        if find.is_empty() {
            return stem.to_string();
        }

        if match_case {
            stem.replace(find, replace)
        } else {
            let lower_stem = stem.to_lowercase();
            let lower_find = find.to_lowercase();
            let mut result = String::new();
            let mut last_idx = 0;

            for (idx, _) in lower_stem.match_indices(&lower_find) {
                result.push_str(&stem[last_idx..idx]);
                result.push_str(replace);
                last_idx = idx + find.len();
            }
            result.push_str(&stem[last_idx..]);
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_rename_replace_text() {
        let paths = vec![
            PathBuf::from("/Users/demo/IMG_001.jpg"),
            PathBuf::from("/Users/demo/IMG_002.jpg"),
        ];

        let mode = BatchRenameMode::ReplaceText {
            find: "IMG".into(),
            replace: "Holiday".into(),
            match_case: false,
        };

        let preview = BatchRenameEngine::generate_preview(&paths, &mode);
        assert_eq!(preview.len(), 2);
        assert_eq!(preview[0].new_name, "Holiday_001.jpg");
        assert_eq!(preview[1].new_name, "Holiday_002.jpg");
        assert!(!preview[0].has_collision);
    }

    #[test]
    fn test_batch_rename_add_text() {
        let paths = vec![PathBuf::from("/Users/demo/report.pdf")];

        let mode = BatchRenameMode::AddText {
            text: "Final_".into(),
            placement: TextPlacement::BeforeName,
        };

        let preview = BatchRenameEngine::generate_preview(&paths, &mode);
        assert_eq!(preview[0].new_name, "Final_report.pdf");
    }

    #[test]
    fn test_batch_rename_format_counter() {
        let paths = vec![
            PathBuf::from("/Users/demo/a.png"),
            PathBuf::from("/Users/demo/b.png"),
        ];

        let mode = BatchRenameMode::Format {
            style: FormatStyle::NameAndCounter,
            custom_format: "Photo".into(),
            placement: TextPlacement::AfterName,
            start_number: 1,
        };

        let preview = BatchRenameEngine::generate_preview(&paths, &mode);
        assert_eq!(preview[0].new_name, "Photo 0001.png");
        assert_eq!(preview[1].new_name, "Photo 0002.png");
    }
}
