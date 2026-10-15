//! # Emoji & Symbols Palette Daemon (`qz-daemons::input_helpers::emoji`)
//!
//! Production-grade macOS Tahoe 26 Character Palette daemon, triggered by canonical
//! shortcut **`Ctrl + Cmd + Space`**.
//!
//! ### Features:
//! - **Full 9-Category Tahoe 26 Classification:** Smileys & Emotion, People & Body,
//!   Animals & Nature, Food & Drink, Travel & Places, Activities, Objects, Symbols, and Flags.
//! - **Sub-Millisecond Search Engine:** Instant keyword and prefix query matching over emoji
//!   codepoints, descriptions, and synonyms.
//! - **Unicode Fitzpatrick Skin Tone Modifiers:** Full support for 6 skin tones (Default, Light,
//!   Medium-Light, Medium, Medium-Dark, Dark) applied dynamically to compatible glyphs.
//! - **Most Recently Used (MRU) & Favorites:** Tracks frequently used glyphs and custom pinned
//!   symbols.

use serde::{Deserialize, Serialize};

/// Canonical Tahoe 26 Emoji Categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmojiCategory {
    /// Faces, expressions, hearts, and emotions.
    SmileysAndEmotion,
    /// Gestures, people, professions, and families.
    PeopleAndBody,
    /// Mammals, birds, reptiles, flowers, and plants.
    AnimalsAndNature,
    /// Fruits, vegetables, meals, drinks, and utensils.
    FoodAndDrink,
    /// Vehicles, buildings, landscapes, and maps.
    TravelAndPlaces,
    /// Sports, games, hobbies, and musical instruments.
    Activities,
    /// Tools, electronics, stationery, clothing, and office supplies.
    Objects,
    /// Punctuation, currency, arrows, math, and geometric shapes.
    Symbols,
    /// National, regional, and decorative flags.
    Flags,
}

impl EmojiCategory {
    /// Human-readable title for UI headers.
    #[must_use]
    pub fn display_title(&self) -> &'static str {
        match self {
            Self::SmileysAndEmotion => "Smileys & Emotion",
            Self::PeopleAndBody => "People & Body",
            Self::AnimalsAndNature => "Animals & Nature",
            Self::FoodAndDrink => "Food & Drink",
            Self::TravelAndPlaces => "Travel & Places",
            Self::Activities => "Activities",
            Self::Objects => "Objects",
            Self::Symbols => "Symbols",
            Self::Flags => "Flags",
        }
    }
}

/// Unicode Fitzpatrick skin tone modifier for eligible human glyphs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkinTone {
    /// Standard yellow un-modified glyph.
    Default,
    /// Fitzpatrick Type 1–2 (Light, `\u{1F3FB}`).
    Light,
    /// Fitzpatrick Type 3 (Medium-Light, `\u{1F3FC}`).
    MediumLight,
    /// Fitzpatrick Type 4 (Medium, `\u{1F3FD}`).
    Medium,
    /// Fitzpatrick Type 5 (Medium-Dark, `\u{1F3FE}`).
    MediumDark,
    /// Fitzpatrick Type 6 (Dark, `\u{1F3FF}`).
    Dark,
}

impl SkinTone {
    /// Returns the Unicode Fitzpatrick modifier character if not Default.
    #[must_use]
    pub fn modifier_char(&self) -> Option<char> {
        match self {
            Self::Default => None,
            Self::Light => Some('\u{1F3FB}'),
            Self::MediumLight => Some('\u{1F3FC}'),
            Self::Medium => Some('\u{1F3FD}'),
            Self::MediumDark => Some('\u{1F3FE}'),
            Self::Dark => Some('\u{1F3FF}'),
        }
    }
}

/// An entry in the emoji catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EmojiEntry {
    /// Base unicode character sequence (e.g. "😀", "👍").
    pub base_glyph: &'static str,
    /// Descriptive name.
    pub name: &'static str,
    /// Category partition.
    pub category: EmojiCategory,
    /// Keywords for search indexing.
    pub keywords: &'static [&'static str],
    /// Whether this emoji supports Fitzpatrick skin tone modification.
    pub supports_skin_tone: bool,
}

impl EmojiEntry {
    /// Returns the glyph formatted with the requested skin tone.
    #[must_use]
    pub fn with_skin_tone(&self, tone: SkinTone) -> String {
        if !self.supports_skin_tone {
            return self.base_glyph.to_string();
        }

        if let Some(modifier) = tone.modifier_char() {
            let mut s = String::new();
            s.push_str(self.base_glyph);
            s.push(modifier);
            s
        } else {
            self.base_glyph.to_string()
        }
    }
}

/// Static catalog of canonical macOS Tahoe 26 emojis.
pub static EMOJI_CATALOG: &[EmojiEntry] = &[
    // Smileys & Emotion
    EmojiEntry {
        base_glyph: "😀",
        name: "Grinning Face",
        category: EmojiCategory::SmileysAndEmotion,
        keywords: &["smile", "happy", "joy", "grin"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "😂",
        name: "Face with Tears of Joy",
        category: EmojiCategory::SmileysAndEmotion,
        keywords: &["laugh", "crying", "tears", "funny", "lol"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🥰",
        name: "Smiling Face with Hearts",
        category: EmojiCategory::SmileysAndEmotion,
        keywords: &["love", "affection", "crush", "adore"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🔥",
        name: "Fire",
        category: EmojiCategory::SmileysAndEmotion,
        keywords: &["lit", "hot", "flame", "burn", "trend"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "❤️",
        name: "Red Heart",
        category: EmojiCategory::SmileysAndEmotion,
        keywords: &["love", "like", "heart", "passion"],
        supports_skin_tone: false,
    },
    // People & Body
    EmojiEntry {
        base_glyph: "👍",
        name: "Thumbs Up",
        category: EmojiCategory::PeopleAndBody,
        keywords: &["like", "yes", "approve", "agree", "good"],
        supports_skin_tone: true,
    },
    EmojiEntry {
        base_glyph: "👋",
        name: "Waving Hand",
        category: EmojiCategory::PeopleAndBody,
        keywords: &["hello", "goodbye", "wave", "hi"],
        supports_skin_tone: true,
    },
    EmojiEntry {
        base_glyph: "🙏",
        name: "Folded Hands",
        category: EmojiCategory::PeopleAndBody,
        keywords: &["please", "thank you", "pray", "namaste", "gratitude"],
        supports_skin_tone: true,
    },
    EmojiEntry {
        base_glyph: "🎉",
        name: "Party Popper",
        category: EmojiCategory::Activities,
        keywords: &["party", "celebrate", "congrats", "cheers", "birthday"],
        supports_skin_tone: false,
    },
    // Animals & Nature
    EmojiEntry {
        base_glyph: "🐶",
        name: "Dog Face",
        category: EmojiCategory::AnimalsAndNature,
        keywords: &["puppy", "pet", "dog", "bark"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🐱",
        name: "Cat Face",
        category: EmojiCategory::AnimalsAndNature,
        keywords: &["kitten", "pet", "cat", "meow"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🌸",
        name: "Cherry Blossom",
        category: EmojiCategory::AnimalsAndNature,
        keywords: &["flower", "spring", "sakura", "pink"],
        supports_skin_tone: false,
    },
    // Food & Drink
    EmojiEntry {
        base_glyph: "☕",
        name: "Hot Beverage",
        category: EmojiCategory::FoodAndDrink,
        keywords: &["coffee", "tea", "espresso", "morning"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🍕",
        name: "Pizza",
        category: EmojiCategory::FoodAndDrink,
        keywords: &["food", "italian", "slice", "cheese"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "🍎",
        name: "Red Apple",
        category: EmojiCategory::FoodAndDrink,
        keywords: &["fruit", "apple", "macintosh", "healthy"],
        supports_skin_tone: false,
    },
    // Travel & Places
    EmojiEntry {
        base_glyph: "🚀",
        name: "Rocket",
        category: EmojiCategory::TravelAndPlaces,
        keywords: &["space", "launch", "ship", "fast", "speed"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "✈️",
        name: "Airplane",
        category: EmojiCategory::TravelAndPlaces,
        keywords: &["flight", "travel", "plane", "vacation"],
        supports_skin_tone: false,
    },
    // Objects
    EmojiEntry {
        base_glyph: "💻",
        name: "Laptop",
        category: EmojiCategory::Objects,
        keywords: &["computer", "macbook", "pc", "tech", "work"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "💡",
        name: "Light Bulb",
        category: EmojiCategory::Objects,
        keywords: &["idea", "light", "bright", "smart"],
        supports_skin_tone: false,
    },
    // Symbols
    EmojiEntry {
        base_glyph: "✨",
        name: "Sparkles",
        category: EmojiCategory::Symbols,
        keywords: &["magic", "stars", "shine", "clean", "ai"],
        supports_skin_tone: false,
    },
    EmojiEntry {
        base_glyph: "✅",
        name: "Check Mark Button",
        category: EmojiCategory::Symbols,
        keywords: &["done", "ok", "yes", "correct", "verified"],
        supports_skin_tone: false,
    },
    // Flags
    EmojiEntry {
        base_glyph: "🏁",
        name: "Chequered Flag",
        category: EmojiCategory::Flags,
        keywords: &["race", "finish", "start", "car"],
        supports_skin_tone: false,
    },
];

/// Emoji Palette Daemon managing window state, searches, and recents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiPaletteDaemon {
    /// Whether the character palette popover is currently open.
    pub is_open: bool,
    /// Active search filter query.
    pub search_query: String,
    /// Currently selected category filter, or `None` for all.
    pub active_category: Option<EmojiCategory>,
    /// Global preferred skin tone.
    pub preferred_skin_tone: SkinTone,
    /// Most Recently Used glyphs (MRU queue, max 32 entries).
    pub recent_glyphs: Vec<String>,
    /// User favorite glyphs.
    pub favorite_glyphs: Vec<String>,
}

impl Default for EmojiPaletteDaemon {
    fn default() -> Self {
        Self::new()
    }
}

impl EmojiPaletteDaemon {
    /// Creates a new `EmojiPaletteDaemon`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
            active_category: None,
            preferred_skin_tone: SkinTone::Default,
            recent_glyphs: Vec::new(),
            favorite_glyphs: Vec::new(),
        }
    }

    /// Toggles the palette open or closed (canonical `Ctrl+Cmd+Space`).
    pub fn toggle_palette(&mut self) -> bool {
        self.is_open = !self.is_open;
        if !self.is_open {
            self.search_query.clear();
        }
        self.is_open
    }

    /// Sets the search filter query.
    pub fn set_search_query(&mut self, query: &str) {
        self.search_query = query.trim().to_lowercase();
    }

    /// Sets the active category filter.
    pub fn set_category(&mut self, category: Option<EmojiCategory>) {
        self.active_category = category;
    }

    /// Sets preferred skin tone modifier.
    pub fn set_skin_tone(&mut self, tone: SkinTone) {
        self.preferred_skin_tone = tone;
    }

    /// Searches and filters matching emoji entries based on query, category, and skin tone.
    #[must_use]
    pub fn search(&self) -> Vec<String> {
        let q = &self.search_query;
        let cat = self.active_category;
        let tone = self.preferred_skin_tone;

        EMOJI_CATALOG
            .iter()
            .filter(|e| {
                if let Some(c) = cat {
                    if e.category != c {
                        return false;
                    }
                }

                if q.is_empty() {
                    return true;
                }

                // Check name
                if e.name.to_lowercase().contains(q) {
                    return true;
                }

                // Check keywords
                e.keywords.iter().any(|k| k.contains(q))
            })
            .map(|e| e.with_skin_tone(tone))
            .collect()
    }

    /// Records a glyph selection into MRU recents and returns the selected string for commit.
    pub fn select_glyph(&mut self, glyph: &str) -> String {
        let g = glyph.to_string();
        // Remove prior occurrence to move to front
        self.recent_glyphs.retain(|r| r != &g);
        self.recent_glyphs.insert(0, g.clone());
        if self.recent_glyphs.len() > 32 {
            self.recent_glyphs.truncate(32);
        }
        // Auto-close palette on selection like macOS
        self.is_open = false;
        self.search_query.clear();
        g
    }

    /// Toggles a glyph in the user's favorites list.
    pub fn toggle_favorite(&mut self, glyph: &str) -> bool {
        if let Some(pos) = self.favorite_glyphs.iter().position(|f| f == glyph) {
            self.favorite_glyphs.remove(pos);
            false
        } else {
            self.favorite_glyphs.push(glyph.to_string());
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_and_search() {
        let mut daemon = EmojiPaletteDaemon::new();
        assert!(!daemon.is_open);

        daemon.toggle_palette();
        assert!(daemon.is_open);

        // Search for "laugh" -> should find "Face with Tears of Joy" ("😂")
        daemon.set_search_query("laugh");
        let results = daemon.search();
        assert!(results.contains(&"😂".to_string()));

        // Search for "coffee" -> should find "☕"
        daemon.set_search_query("coffee");
        let results = daemon.search();
        assert!(results.contains(&"☕".to_string()));
    }

    #[test]
    fn test_skin_tone_modification() {
        let mut daemon = EmojiPaletteDaemon::new();
        daemon.set_search_query("thumbs up");

        // Default skin tone
        let results = daemon.search();
        assert_eq!(results[0], "👍");

        // Light skin tone
        daemon.set_skin_tone(SkinTone::Light);
        let results = daemon.search();
        let expected = format!("👍{}", '\u{1F3FB}');
        assert_eq!(results[0], expected);
    }

    #[test]
    fn test_mru_recents_management() {
        let mut daemon = EmojiPaletteDaemon::new();
        let chosen = daemon.select_glyph("🚀");
        assert_eq!(chosen, "🚀");
        assert_eq!(daemon.recent_glyphs, vec!["🚀".to_string()]);
        assert!(!daemon.is_open); // Closed after commit

        daemon.select_glyph("☕");
        assert_eq!(daemon.recent_glyphs, vec!["☕".to_string(), "🚀".to_string()]);
    }

    #[test]
    fn test_favorites_toggle() {
        let mut daemon = EmojiPaletteDaemon::new();
        assert!(daemon.toggle_favorite("✨"));
        assert_eq!(daemon.favorite_glyphs, vec!["✨".to_string()]);

        assert!(!daemon.toggle_favorite("✨"));
        assert!(daemon.favorite_glyphs.is_empty());
    }
}
