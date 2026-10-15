//! # Desktop Subsystem Module
//!
//! Desktop icon surface, dynamic stacks, and categorization.

pub mod stacks;

pub use stacks::{
    DesktopItem, DesktopStack, DesktopStacksManager, StackGrouping, StackKindCategory,
};
