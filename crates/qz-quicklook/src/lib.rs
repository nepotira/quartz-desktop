//! # QuartzLinux Quick Look Daemon Subsystem (`qz-quicklook`)
//!
//! Instantaneous spacebar preview daemon indistinguishable from macOS Tahoe 26:
//!
//! 1. **Spacebar Preview Daemon (`controller`)**:
//!    - Intercepts `Space` key events from Finder and Spotlight to summon or dismiss previews.
//!    - Smoothly retargets active preview window in-place when user navigates files with arrow keys.
//!
//! 2. **120fps Rect Expansion Animation (`animation`)**:
//!    - Continuous $C^2$ geometric morphing between source thumbnail rect and destination preview rect
//!      driven by analytical spring physics (`qz_animation::SpringAnimation`).
//!
//! 3. **Multi-Format Preview Generators (`preview`)**:
//!    - High-fidelity previews for Images (Display P3, alpha), PDFs (multi-page TOC),
//!      Source Code (syntax highlighting tokens), Media (audio/video codecs), and Archives.
//!
//! 4. **Interactive Markup Subsystem (`markup`)**:
//!    - Inline annotation tools: Pen, Highlighter, Arrow, Geometric Shapes (Rect, Ellipse),
//!      Text Notes, and Trackpad Signatures with localized undo/redo history.
//!
//! 5. **Media Trimming Subsystem (`trim`)**:
//!    - Non-destructive video and audio trimming with tactile In/Out handles and playhead scrubbing.
//!
//! 6. **Visual Look Up & Subject Lift (`visual_lookup`)**:
//!    - Machine intelligence entity classification (Pets, Plants, Landmarks, Food, Art)
//!      and instant "Copy Subject" foreground isolation with golden rim shimmer.

#![deny(missing_docs)]

pub mod animation;
pub mod controller;
pub mod markup;
pub mod preview;
pub mod trim;
pub mod visual_lookup;

// Re-exports
pub use animation::{ExpansionPhase, RectBounds, RectExpansionAnimator};
pub use controller::{QuickLookController, QuickLookEvent};
pub use markup::{MarkupElement, MarkupSession, MarkupToolType};
pub use preview::{PreviewGenerator, PreviewPayload, QuickLookPreview};
pub use trim::{MediaTrimController, TrimBounds, TrimError};
pub use visual_lookup::{
    EntityKind, LiftedSubject, SubjectLiftController, VisualLookupCard,
};
