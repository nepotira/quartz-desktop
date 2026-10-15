//! # Desktop Stacks Subsystem
//!
//! Groups desktop items dynamically into tidy stacks (by Kind, Date, or Tags)
//! with spring-animated fan expansion, hover inspection, and category resolution.

use qz_animation::{SpringAnimation, SpringParams};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Grouping criteria for organizing Desktop Stacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackGrouping {
    /// Group by file kind (Images, Documents, Screenshots, Movies, etc.).
    Kind,
    /// Group by last opened date.
    DateLastOpened,
    /// Group by date added to desktop.
    DateAdded,
    /// Group by modification date.
    DateModified,
    /// Group by creation date.
    DateCreated,
    /// Group by Finder tags/colors.
    Tags,
}

/// Category classification when grouping by file kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackKindCategory {
    /// Screenshots captured on the desktop.
    Screenshots,
    /// Digital images and photos (PNG, JPEG, HEIC, TIFF, WebP, SVG).
    Images,
    /// Text and office documents (PDF, TXT, DOCX, PAGES, RTF, MD).
    Documents,
    /// Video and movie files (MP4, MOV, MKV, AVI).
    Movies,
    /// Audio files and podcasts (MP3, AAC, M4A, WAV, FLAC).
    Music,
    /// Archives and compressed files (ZIP, TAR, GZ, DMG, ISO).
    Archives,
    /// Code and developer source files (RS, C, PY, JS, TS, TOML, JSON).
    DeveloperCode,
    /// Generic or uncategorized desktop files.
    Other,
}

impl StackKindCategory {
    /// Human-readable title of the stack category.
    pub fn display_title(&self) -> &'static str {
        match self {
            Self::Screenshots => "Screenshots",
            Self::Images => "Images",
            Self::Documents => "Documents",
            Self::Movies => "Movies",
            Self::Music => "Music",
            Self::Archives => "Archives",
            Self::DeveloperCode => "Developer",
            Self::Other => "Other",
        }
    }

    /// Determines kind category from a file path extension and name.
    pub fn from_path(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if file_name.starts_with("screen shot") || file_name.starts_with("screenshot") {
            return Self::Screenshots;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();

        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "heic" | "webp" | "tiff" | "gif" | "svg" => Self::Images,
            "pdf" | "txt" | "doc" | "docx" | "pages" | "rtf" | "md" => Self::Documents,
            "mp4" | "mov" | "mkv" | "avi" | "webm" => Self::Movies,
            "mp3" | "m4a" | "aac" | "flac" | "wav" => Self::Music,
            "zip" | "tar" | "gz" | "dmg" | "iso" | "7z" => Self::Archives,
            "rs" | "c" | "cpp" | "h" | "py" | "js" | "ts" | "json" | "toml" | "xml" => {
                Self::DeveloperCode
            }
            _ => Self::Other,
        }
    }
}

/// A desktop file item participating in stacks.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopItem {
    /// Absolute path to file on disk.
    pub path: PathBuf,
    /// Display name.
    pub display_name: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Unix timestamp of creation.
    pub created_secs: u64,
    /// Unix timestamp of last modification.
    pub modified_secs: u64,
    /// Finder tags associated with file.
    pub tags: Vec<String>,
}

/// A stack group rendering on the desktop surface.
#[derive(Debug, Clone)]
pub struct DesktopStack {
    /// Category or grouping key title.
    pub key: String,
    /// All items aggregated into this stack.
    pub items: Vec<DesktopItem>,
    /// Whether this stack is currently expanded on the desktop.
    pub is_expanded: bool,
    /// Expansion progress spring animation $[0.0, 1.0]$.
    pub expansion_spring: SpringAnimation,
    /// Normalized expansion factor $[0.0, 1.0]$.
    pub expansion_factor: f32,
    /// Desktop icon grid coordinate $(x, y)$ in pixels.
    pub grid_pos: [f32; 2],
}

impl DesktopStack {
    /// Creates a new desktop stack group.
    pub fn new(key: impl Into<String>, items: Vec<DesktopItem>, grid_pos: [f32; 2]) -> Self {
        let spring_params = SpringParams::dock_bounce();
        let expansion_spring = SpringAnimation::new(spring_params, 0.0, 0.0, 0.0);

        Self {
            key: key.into(),
            items,
            is_expanded: false,
            expansion_spring,
            expansion_factor: 0.0,
            grid_pos,
        }
    }

    /// Toggles the stack expansion state.
    pub fn toggle_expansion(&mut self) -> bool {
        self.is_expanded = !self.is_expanded;
        let target = if self.is_expanded { 1.0 } else { 0.0 };
        self.expansion_spring.retarget(target);
        self.is_expanded
    }

    /// Advances the spring animation by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        let state = self.expansion_spring.step(dt as f64);
        self.expansion_factor = state.position.clamp(0.0, 1.0) as f32;
    }

    /// Computes the fan or grid position for an item inside this stack during expansion.
    pub fn item_offset_in_stack(&self, index: usize) -> [f32; 2] {
        if self.items.is_empty() {
            return [0.0, 0.0];
        }

        // When collapsed (expansion_factor == 0), items sit tightly stacked with slight 2px vertical stagger
        let collapsed_stagger = (index.min(3) as f32) * -3.0;
        let collapsed_y = collapsed_stagger;

        // When expanded, items unfold downward or in a column
        let expanded_gap = 72.0;
        let expanded_y = (index as f32 + 1.0) * expanded_gap;

        let y = collapsed_y + (expanded_y - collapsed_y) * self.expansion_factor;
        [0.0, y]
    }
}

/// Desktop Stacks Organizer grouping and managing desktop surface files.
pub struct DesktopStacksManager {
    grouping: StackGrouping,
    stacks: BTreeMap<String, DesktopStack>,
}

impl Default for DesktopStacksManager {
    fn default() -> Self {
        Self::new(StackGrouping::Kind)
    }
}

impl DesktopStacksManager {
    /// Creates a new Desktop Stacks manager with specified grouping.
    pub fn new(grouping: StackGrouping) -> Self {
        Self {
            grouping,
            stacks: BTreeMap::new(),
        }
    }

    /// Active grouping mode.
    #[inline]
    pub fn grouping(&self) -> StackGrouping {
        self.grouping
    }

    /// Sets the grouping mode and reorganizes items.
    pub fn set_grouping(&mut self, grouping: StackGrouping, desktop_items: &[DesktopItem]) {
        self.grouping = grouping;
        self.rebuild_stacks(desktop_items);
    }

    /// Rebuilds all desktop stacks from raw desktop items.
    pub fn rebuild_stacks(&mut self, desktop_items: &[DesktopItem]) {
        self.stacks.clear();

        let mut groups: BTreeMap<String, Vec<DesktopItem>> = BTreeMap::new();

        for item in desktop_items {
            let key = match self.grouping {
                StackGrouping::Kind => {
                    let kind = StackKindCategory::from_path(&item.path);
                    kind.display_title().to_string()
                }
                StackGrouping::DateModified => {
                    Self::date_bucket_label(item.modified_secs)
                }
                StackGrouping::DateCreated => {
                    Self::date_bucket_label(item.created_secs)
                }
                StackGrouping::Tags => {
                    if let Some(tag) = item.tags.first() {
                        tag.clone()
                    } else {
                        "No Tags".to_string()
                    }
                }
                _ => "Other".to_string(),
            };

            groups.entry(key).or_default().push(item.clone());
        }

        let mut col_x = 48.0;
        for (key, items) in groups {
            let stack = DesktopStack::new(key.clone(), items, [col_x, 64.0]);
            self.stacks.insert(key, stack);
            col_x += 96.0;
        }
    }

    fn date_bucket_label(timestamp_secs: u64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let age_secs = now.saturating_sub(timestamp_secs);
        let day_secs = 86400;

        if age_secs < day_secs {
            "Today".to_string()
        } else if age_secs < day_secs * 2 {
            "Yesterday".to_string()
        } else if age_secs < day_secs * 7 {
            "Previous 7 Days".to_string()
        } else if age_secs < day_secs * 30 {
            "Previous 30 Days".to_string()
        } else {
            "Older".to_string()
        }
    }

    /// Updates spring animation states for all stacks.
    pub fn update(&mut self, dt: f32) {
        for stack in self.stacks.values_mut() {
            stack.update(dt);
        }
    }

    /// Read-only access to all active stacks.
    pub fn stacks(&self) -> &BTreeMap<String, DesktopStack> {
        &self.stacks
    }

    /// Mutable access to a stack by key.
    pub fn stack_mut(&mut self, key: &str) -> Option<&mut DesktopStack> {
        self.stacks.get_mut(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_kind_categorization() {
        assert_eq!(
            StackKindCategory::from_path(Path::new("Screen Shot 2026-10-15.png")),
            StackKindCategory::Screenshots
        );
        assert_eq!(
            StackKindCategory::from_path(Path::new("vacation.jpeg")),
            StackKindCategory::Images
        );
        assert_eq!(
            StackKindCategory::from_path(Path::new("quarterly_report.pdf")),
            StackKindCategory::Documents
        );
        assert_eq!(
            StackKindCategory::from_path(Path::new("movie.mov")),
            StackKindCategory::Movies
        );
        assert_eq!(
            StackKindCategory::from_path(Path::new("main.rs")),
            StackKindCategory::DeveloperCode
        );
    }

    #[test]
    fn test_desktop_stack_expansion_spring() {
        let items = vec![
            DesktopItem {
                path: PathBuf::from("/Users/demo/Desktop/img1.png"),
                display_name: "img1.png".into(),
                size_bytes: 1024,
                created_secs: 100,
                modified_secs: 100,
                tags: vec![],
            },
            DesktopItem {
                path: PathBuf::from("/Users/demo/Desktop/img2.png"),
                display_name: "img2.png".into(),
                size_bytes: 2048,
                created_secs: 100,
                modified_secs: 100,
                tags: vec![],
            },
        ];

        let mut stack = DesktopStack::new("Images", items, [100.0, 100.0]);
        assert_eq!(stack.expansion_factor, 0.0);

        stack.toggle_expansion();
        assert!(stack.is_expanded);

        for _ in 0..60 {
            stack.update(1.0 / 60.0);
        }

        assert!(stack.expansion_factor > 0.5);
    }
}
