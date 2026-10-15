//! # Universal Search Engine Subsystem
//!
//! Orchestrates fast multi-category queries across Applications, Files/Documents,
//! System Settings, Scientific Calculations, Unit Conversions, and OCR-recognized text.

use crate::search::calc::PrattParser;
use crate::search::converter::UnitConverter;
use crate::search::ocr::OcrIndex;

/// Category classification for a search hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SearchCategory {
    /// Scientific calculation result card.
    Calculator,
    /// Unit or currency conversion result card.
    Converter,
    /// Installed application.
    Applications,
    /// System preferences pane.
    SystemSettings,
    /// File or document on disk.
    Documents,
    /// Scanned document or image matched via OCR.
    OcrDocument,
    /// Web bookmark or browser history.
    Bookmarks,
    /// Address book contact.
    Contacts,
}

/// Action to perform when user activates a search result.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchAction {
    /// Launch application with executable path and arguments.
    LaunchApp {
        /// Path to desktop executable or .app bundle.
        exec_path: String,
    },
    /// Open file in default application or reveal in Finder.
    OpenFile {
        /// Absolute path to file on disk.
        file_path: String,
    },
    /// Open a specific System Settings panel.
    OpenSettingsPane {
        /// Identifier for the settings pane.
        pane_id: String,
    },
    /// Copy evaluated result text to system clipboard.
    CopyResult {
        /// String to be copied.
        text_to_copy: String,
    },
    /// Open URL in default web browser.
    OpenUrl {
        /// Web URL string.
        url: String,
    },
}

/// A unified search result entry displayed in Spotlight list.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultItem {
    /// Unique item ID.
    pub id: String,
    /// Primary title string.
    pub title: String,
    /// Secondary subtitle or path.
    pub subtitle: String,
    /// Category grouping.
    pub category: SearchCategory,
    /// Icon resource identifier.
    pub icon_name: String,
    /// Computed ranking score (higher is better).
    pub score: f32,
    /// Action executed on Enter.
    pub action: SearchAction,
    /// Optional rich Quick Look preview payload (file path or markdown text).
    pub preview_payload: Option<String>,
}

/// An entry indexed into the Universal Search database.
#[derive(Debug, Clone)]
pub struct IndexedItem {
    /// Unique item ID.
    pub id: String,
    /// Title to match against.
    pub title: String,
    /// Subtitle or path description.
    pub subtitle: String,
    /// Keywords and aliases for search.
    pub keywords: Vec<String>,
    /// Category classification.
    pub category: SearchCategory,
    /// Icon identifier.
    pub icon_name: String,
    /// Action executed when selected.
    pub action: SearchAction,
    /// Optional Quick Look file path or preview text.
    pub preview_payload: Option<String>,
    /// Launch or access count for ranking boost.
    pub usage_frequency: u32,
}

/// Core Universal Search Engine.
pub struct SearchEngine {
    indexed_items: Vec<IndexedItem>,
    converter: UnitConverter,
    ocr_index: OcrIndex,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine {
    /// Creates a new Universal Search Engine with default converters and empty index.
    pub fn new() -> Self {
        Self {
            indexed_items: Vec::new(),
            converter: UnitConverter::new(),
            ocr_index: OcrIndex::new(),
        }
    }

    /// Access reference to OCR index.
    pub fn ocr_index(&self) -> &OcrIndex {
        &self.ocr_index
    }

    /// Access mutable reference to OCR index.
    pub fn ocr_index_mut(&mut self) -> &mut OcrIndex {
        &mut self.ocr_index
    }

    /// Adds an item to the searchable index.
    pub fn add_item(&mut self, item: IndexedItem) {
        self.indexed_items.push(item);
    }

    /// Executes query and returns sorted search hits.
    pub fn query(&self, input: &str) -> Vec<SearchResultItem> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // 1. Try Unit & Currency Converter
        if let Ok(conv) = self.converter.convert_query(trimmed) {
            results.push(SearchResultItem {
                id: "converter_result".to_string(),
                title: format!("{:.4} {}", conv.to_amount, conv.to_unit),
                subtitle: conv.formatted.clone(),
                category: SearchCategory::Converter,
                icon_name: "arrow.triangle.2.circlepath".to_string(),
                score: 2000.0,
                action: SearchAction::CopyResult {
                    text_to_copy: conv.to_amount.to_string(),
                },
                preview_payload: Some(conv.formatted),
            });
        }

        // 2. Try Scientific Calculator if query contains mathematical characters or functions
        if Self::looks_like_math(trimmed) {
            if let Ok(val) = PrattParser::evaluate_expression(trimmed) {
                let formatted_val = format!("{val}");
                results.push(SearchResultItem {
                    id: "calc_result".to_string(),
                    title: formatted_val.clone(),
                    subtitle: format!("{trimmed} = {formatted_val}"),
                    category: SearchCategory::Calculator,
                    icon_name: "function".to_string(),
                    score: 1800.0,
                    action: SearchAction::CopyResult {
                        text_to_copy: formatted_val.clone(),
                    },
                    preview_payload: Some(format!("Calculation:\n{trimmed} = {formatted_val}")),
                });
            }
        }

        // 3. Search indexed items with fuzzy ranking
        let query_lower = trimmed.to_lowercase();
        for item in &self.indexed_items {
            if let Some(score) = Self::compute_match_score(&query_lower, item) {
                results.push(SearchResultItem {
                    id: item.id.clone(),
                    title: item.title.clone(),
                    subtitle: item.subtitle.clone(),
                    category: item.category,
                    icon_name: item.icon_name.clone(),
                    score,
                    action: item.action.clone(),
                    preview_payload: item.preview_payload.clone(),
                });
            }
        }

        // 4. Search OCR Index
        let ocr_hits = self.ocr_index.search(trimmed);
        for hit in ocr_hits {
            results.push(SearchResultItem {
                id: format!("ocr_{}", hit.document_id),
                title: hit.title.clone(),
                subtitle: hit.snippet.clone(),
                category: SearchCategory::OcrDocument,
                icon_name: "text.viewfinder".to_string(),
                score: 300.0 + hit.score,
                action: SearchAction::OpenFile {
                    file_path: hit.file_path.clone(),
                },
                preview_payload: Some(hit.file_path),
            });
        }

        // Sort results descending by score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn looks_like_math(query: &str) -> bool {
        let math_chars = ['+', '-', '*', '/', '^', '%', '(', ')', '='];
        let has_math_char = query.chars().any(|c| math_chars.contains(&c));
        let math_keywords = ["sin", "cos", "tan", "sqrt", "log", "ln", "abs", "pi", "tau"];
        let has_keyword = math_keywords.iter().any(|k| query.to_lowercase().contains(k));
        has_math_char || has_keyword
    }

    fn compute_match_score(query: &str, item: &IndexedItem) -> Option<f32> {
        let title_lower = item.title.to_lowercase();

        let mut base_score: f32 = 0.0;

        // Exact match
        if title_lower == query {
            base_score = 1000.0;
        } else if title_lower.starts_with(query) {
            // Prefix match
            base_score = 800.0 + (query.len() as f32 / title_lower.len() as f32) * 50.0;
        } else if title_lower.split_whitespace().any(|word| word.starts_with(query)) {
            // Word boundary match
            base_score = 600.0;
        } else if let Some(pos) = title_lower.find(query) {
            // Substring match
            base_score = 400.0 - (pos as f32 * 2.0);
        } else {
            // Check keywords
            for kw in &item.keywords {
                let kw_lower = kw.to_lowercase();
                if kw_lower == query {
                    base_score = 500.0;
                    break;
                } else if kw_lower.starts_with(query) {
                    base_score = 450.0;
                    break;
                }
            }
        }

        if base_score > 0.0 {
            // Boost based on category and usage frequency
            let category_boost = match item.category {
                SearchCategory::Applications => 100.0,
                SearchCategory::SystemSettings => 50.0,
                SearchCategory::Documents => 20.0,
                SearchCategory::Bookmarks => 10.0,
                SearchCategory::Contacts => 15.0,
                _ => 0.0,
            };

            let freq_boost = (item.usage_frequency as f32 * 5.0).min(100.0);
            Some(base_score + category_boost + freq_boost)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_calc_and_converter_priority() {
        let engine = SearchEngine::new();

        // Calculation
        let calc_hits = engine.query("2 + 2 * 10");
        assert!(!calc_hits.is_empty());
        assert_eq!(calc_hits[0].category, SearchCategory::Calculator);
        assert_eq!(calc_hits[0].title, "22");

        // Conversion
        let conv_hits = engine.query("10 km to mi");
        assert!(!conv_hits.is_empty());
        assert_eq!(conv_hits[0].category, SearchCategory::Converter);
    }

    #[test]
    fn test_search_engine_app_ranking() {
        let mut engine = SearchEngine::new();
        engine.add_item(IndexedItem {
            id: "app-safari".into(),
            title: "Safari".into(),
            subtitle: "/Applications/Safari.app".into(),
            keywords: vec!["browser".into(), "web".into(), "internet".into()],
            category: SearchCategory::Applications,
            icon_name: "safari".into(),
            action: SearchAction::LaunchApp {
                exec_path: "/Applications/Safari.app".into(),
            },
            preview_payload: None,
            usage_frequency: 45,
        });

        engine.add_item(IndexedItem {
            id: "doc-safari-notes".into(),
            title: "Safari Notes.txt".into(),
            subtitle: "~/Documents/Safari Notes.txt".into(),
            keywords: vec![],
            category: SearchCategory::Documents,
            icon_name: "doc.text".into(),
            action: SearchAction::OpenFile {
                file_path: "~/Documents/Safari Notes.txt".into(),
            },
            preview_payload: None,
            usage_frequency: 1,
        });

        let hits = engine.query("safari");
        assert_eq!(hits.len(), 2);
        // Application should rank higher than document
        assert_eq!(hits[0].id, "app-safari");
        assert_eq!(hits[0].category, SearchCategory::Applications);
    }
}
