//! # Search Subsystem Module
//!
//! Universal Search, Scientific Pratt Parser Calculator, Unit/Currency Converter, and OCR indexing.

pub mod calc;
pub mod converter;
pub mod engine;
pub mod ocr;

pub use calc::{AstNode, CalcError, PrattParser};
pub use converter::{ConversionResult, ConvertError, UnitConverter, UnitDimension};
pub use engine::{IndexedItem, SearchAction, SearchCategory, SearchEngine, SearchResultItem};
pub use ocr::{OcrDocument, OcrIndex, OcrSearchResult, OcrToken};
