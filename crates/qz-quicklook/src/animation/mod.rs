//! # Quick Look Animation Module
//!
//! Continuous 120fps spring-driven rect expansion and collapse physics.

pub mod rect_expansion;

pub use rect_expansion::{ExpansionPhase, RectBounds, RectExpansionAnimator};
