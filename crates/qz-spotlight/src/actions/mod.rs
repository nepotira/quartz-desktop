//! # System and Menu Actions Subsystem Module
//!
//! Direct execution of system tasks and active application menu bar items.

pub mod menu_runner;
pub mod system_actions;

pub use menu_runner::{FlattenedMenuItem, MenuActionHit, MenuRunner};
pub use system_actions::{SystemActionHit, SystemActionId, SystemActionRegistry, SystemCommand};
