//! # OCR Text Search Indexing Subsystem
//!
//! Indexes visual text recognized in scanned documents, images, and screenshots
//! with bounding box locations, confidence scores, and snippet extraction.

use std::collections::HashMap;

/// A single word token extracted by Optical Character Recognition (OCR).
#[derive(Debug, Clone, PartialEq)]
pub struct OcrToken {
    /// Exact text content.
    pub text: String,
    /// Lowercased normalized text for matching.
    pub normalized: String,
    /// Normalized coordinates $[x, y, w, h]$ within page $[0.0, 1.0]$.
    pub bounding_box: [f32; 4],
    /// Recognition confidence score $[0.0, 1.0]$.
    pub confidence: f32,
}

impl OcrToken {
    /// Creates a new OCR token.
    pub fn new(text: impl Into<String>, bounding_box: [f32; 4], confidence: f32) -> Self {
        let text = text.into();
        let normalized = text.to_lowercase();
        Self {
            text,
            normalized,
            bounding_box,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// A scanned document, PDF page, or image containing OCR-recognized text.
#[derive(Debug, Clone, PartialEq)]
pub struct OcrDocument {
    /// Unique document identifier.
    pub id: String,
    /// Absolute filesystem path to file.
    pub file_path: String,
    /// Display title or file name.
    pub title: String,
    /// Page number (1-indexed).
    pub page: u32,
    /// Ordered list of tokens in reading order.
    pub tokens: Vec<OcrToken>,
    /// Full aggregated text string.
    pub full_text: String,
}

impl OcrDocument {
    /// Creates a new OCR document record from extracted tokens.
    pub fn new(id: impl Into<String>, file_path: impl Into<String>, title: impl Into<String>, page: u32, tokens: Vec<OcrToken>) -> Self {
        let full_text = tokens
            .iter()
            .map(|t| t.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        Self {
            id: id.into(),
            file_path: file_path.into(),
            title: title.into(),
            page,
            tokens,
            full_text,
        }
    }
}

/// Search hit result for an OCR document match.
#[derive(Debug, Clone, PartialEq)]
pub struct OcrSearchResult {
    /// Document identifier.
    pub document_id: String,
    /// File path to original document.
    pub file_path: String,
    /// Document title.
    pub title: String,
    /// Page number where match occurred.
    pub page: u32,
    /// Contextual snippet showing matched keywords with surrounding words.
    pub snippet: String,
    /// Bounding boxes of matching tokens on the page.
    pub matching_boxes: Vec<[f32; 4]>,
    /// Relevance score.
    pub score: f32,
}

/// Inverted search index for visual text across images and documents.
pub struct OcrIndex {
    documents: HashMap<String, OcrDocument>,
    token_postings: HashMap<String, Vec<(String, usize)>>,
}

impl Default for OcrIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrIndex {
    /// Creates an empty OCR index.
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            token_postings: HashMap::new(),
        }
    }

    /// Indexes an OCR document into the inverted postings table.
    pub fn index_document(&mut self, doc: OcrDocument) {
        let doc_id = doc.id.clone();
        for (idx, token) in doc.tokens.iter().enumerate() {
            let key = token.normalized.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if !key.is_empty() {
                self.token_postings
                    .entry(key)
                    .or_default()
                    .push((doc_id.clone(), idx));
            }
        }
        self.documents.insert(doc_id, doc);
    }

    /// Searches for documents containing the query words.
    pub fn search(&self, query: &str) -> Vec<OcrSearchResult> {
        let query_tokens: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase().trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if query_tokens.is_empty() {
            return Vec::new();
        }

        // Aggregate hits per document
        let mut doc_hits: HashMap<String, Vec<usize>> = HashMap::new();

        for q in &query_tokens {
            for (post_key, postings) in &self.token_postings {
                if post_key.contains(q) {
                    for (doc_id, token_idx) in postings {
                        doc_hits.entry(doc_id.clone()).or_default().push(*token_idx);
                    }
                }
            }
        }

        let mut results = Vec::new();

        for (doc_id, mut indices) in doc_hits {
            if let Some(doc) = self.documents.get(&doc_id) {
                indices.sort_unstable();
                indices.dedup();

                let mut matching_boxes = Vec::new();
                let mut snippet_tokens = Vec::new();

                for &idx in &indices {
                    if let Some(tok) = doc.tokens.get(idx) {
                        matching_boxes.push(tok.bounding_box);
                    }
                }

                // Generate snippet around the first matched token
                if let Some(&first_idx) = indices.first() {
                    let start = first_idx.saturating_sub(4);
                    let end = (first_idx + 8).min(doc.tokens.len());
                    for t in &doc.tokens[start..end] {
                        snippet_tokens.push(t.text.clone());
                    }
                }

                let snippet = if snippet_tokens.is_empty() {
                    doc.full_text.chars().take(80).collect()
                } else {
                    format!("... {} ...", snippet_tokens.join(" "))
                };

                let score = (indices.len() as f32 * 10.0) + (matching_boxes.len() as f32 * 2.0);

                results.push(OcrSearchResult {
                    document_id: doc_id,
                    file_path: doc.file_path.clone(),
                    title: doc.title.clone(),
                    page: doc.page,
                    snippet,
                    matching_boxes,
                    score,
                });
            }
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_indexing_and_search() {
        let mut index = OcrIndex::new();

        let receipt_tokens = vec![
            OcrToken::new("Apple", [0.1, 0.1, 0.2, 0.05], 0.98),
            OcrToken::new("Store", [0.32, 0.1, 0.2, 0.05], 0.99),
            OcrToken::new("Receipt", [0.55, 0.1, 0.2, 0.05], 0.95),
            OcrToken::new("Total:", [0.1, 0.8, 0.2, 0.05], 0.99),
            OcrToken::new("$1,299.00", [0.35, 0.8, 0.3, 0.05], 0.97),
        ];

        let doc = OcrDocument::new(
            "doc-1",
            "/Users/demo/Documents/receipt_macbook.pdf",
            "Apple Store Receipt",
            1,
            receipt_tokens,
        );

        index.index_document(doc);

        let search_res = index.search("Receipt");
        assert_eq!(search_res.len(), 1);
        assert_eq!(search_res[0].title, "Apple Store Receipt");
        assert!(search_res[0].snippet.contains("Receipt"));
        assert_eq!(search_res[0].matching_boxes.len(), 1);
    }
}
