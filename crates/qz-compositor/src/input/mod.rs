//! # Quartz Input Management Subsystem
//!
//! Submodules:
//! - `keymapper`: Smart Command/Control translation, terminal SIGINT isolation, and Cocoa/Emacs keybindings.
//! - `click_through`: Cmd-click-through to background surfaces, unfocused scroll dispatch, and traffic lights hover logic.

pub mod click_through;
pub mod keymapper;

pub use click_through::{ClickThroughAction, ClickThroughDispatcher, ClickThroughResult, WindowSurfaceInfo};
pub use keymapper::{KeyMappingResult, ModifierState, SmartKeymapper, TargetAppKind};
