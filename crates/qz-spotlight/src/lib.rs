//! # QuartzLinux Spotlight Subsystem (`qz-spotlight`)
//!
//! macOS Tahoe 26 4-Mode System Launcher and Global Search Engine:
//!
//! 1. **Mode 1 — Universal Search (`Cmd+1` / `Cmd+Space`)**:
//!    - Instant fuzzy search across applications, documents, system preferences, bookmarks, and contacts.
//!    - **Scientific Pratt Parser Calculator**: High-precision evaluation with binding power precedence,
//!      trigonometric/logarithmic/exponential functions, and mathematical constants ($\pi, e, \tau, \phi$).
//!    - **Unit & Currency Converter**: Natural language conversions across Length, Mass, Temperature,
//!      Digital Memory (decimal and binary units), Time, and Currencies.
//!    - **OCR Text Search**: Recognition index matching visual text within scanned images and documents,
//!      highlighting bounding boxes and extracting contextual snippets.
//!
//! 2. **Mode 2 — System & Menu Actions (`Cmd+2`)**:
//!    - Quick execution of system management tasks: Sleep, Restart, Shut Down, Lock Screen, Log Out,
//!      Empty Trash, Appearance Mode (Dark/Light), Hardware radios (Wi-Fi, Bluetooth), and Screenshots.
//!    - **Active Application Menu Runner**: Traverses and flattens the active app's menu tree
//!      (`File > Export As > PDF...`), enabling direct search and execution from the keyboard.
//!
//! 3. **Mode 3 — Native Clipboard History (`Cmd+3`)**:
//!    - Ring buffer storage (default capacity 200 items) capturing text, rich markup, images, and files.
//!    - **Automatic Color Swatch Detection**: Instantly recognizes CSS/Hex color values (`#RGB`, `#RRGGBB`,
//!      `#RRGGBBAA`, `rgb()`) and generates live color preview swatches.
//!    - **Pinning Protection**: Pinned favorites are permanently preserved from ring-buffer eviction.
//!    - Full-text search and filtering across clipboard history.
//!
//! 4. **Mode 4 — Categorized App Library (`Cmd+4`)**:
//!    - Semantic categorization grid: Productivity & Finance, Utilities, Developer Tools, Creativity,
//!      Media, Social & Communication, and System & Settings.
//!    - **Jiggle Mode Oscillation Physics**: $8.0\text{Hz}$ oscillatory rotation ($\pm 1.5^\circ$) with
//!      inter-item phase desynchronization and uninstall badges.
//!    - Application launch tracking and usage frequency rankings.
//!
//! All modes are animated with continuous $C^2$ spring physics (`qz-animation`), smoothly
//! morphing window dimensions between the compact 56px search pill and 520px expanded results.

#![deny(missing_docs)]

pub mod actions;
pub mod app_library;
pub mod clipboard;
pub mod controller;
pub mod search;

// Re-exports
pub use actions::{
    FlattenedMenuItem, MenuActionHit, MenuRunner, SystemActionHit, SystemActionId,
    SystemActionRegistry, SystemCommand,
};
pub use app_library::{AppCategory, AppEntry, AppLibrary, JiggleState};
pub use clipboard::{ClipboardContent, ClipboardEntry, ClipboardHistory};
pub use controller::{ControllerEvent, SpotlightController, SpotlightMode};
pub use search::{
    AstNode, CalcError, ConversionResult, ConvertError, IndexedItem, OcrDocument, OcrIndex,
    OcrSearchResult, OcrToken, PrattParser, SearchAction, SearchCategory, SearchEngine,
    SearchResultItem, UnitConverter, UnitDimension,
};
