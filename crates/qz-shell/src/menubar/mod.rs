//! # Top Menu Bar Subsystem
//!
//! Submodules:
//! - `notch`: Display cutout layout partitioning and pointer pass-through.
//! - `apple_menu`: Canonical system menu with Option-key instant actions.
//! - `global_menu`: Bold application menu, 0ms hover sliding, item double-flash, and Help Rector.
//! - `status_items`: Right-side status icons, Poof removal, and Option diagnostics.
//! - `privacy_capsule`: Hardware microphone/camera/screen recording indicators.

pub mod apple_menu;
pub mod global_menu;
pub mod notch;
pub mod privacy_capsule;
pub mod status_items;

pub use apple_menu::{AppleMenuAction, AppleMenuBuilder, AppleMenuItem};
pub use global_menu::{GlobalMenuManager, HelpSearchResult, MenuNode, WindowMenuEntry};
pub use notch::{DisplayNotch, MenuNotchPartitioner, PositionedMenuItem};
pub use privacy_capsule::{PrivacyCapsuleManager, PrivacySensor, SensorAccess};
pub use status_items::{BluetoothDiagnostics, StatusIconKind, StatusItem, StatusItemsManager, WifiDiagnostics};
