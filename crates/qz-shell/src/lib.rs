//! # qz-shell
//!
//! macOS Tahoe 26 Desktop Shell Environment for QuartzLinux:
//! - **Top Menu Bar**: Hardware display notch handling, canonical Apple menu with Option instant actions,
//!   0ms lateral hover sliding, 120ms double-flash activation, Option-mutable accelerators,
//!   Help Search Rector with blue guide arrow, right-side status tray with Option diagnostics,
//!   and real-time Hardware Privacy Capsule (Microphone, Camera, Screen Recording).
//! - **Control Center**: Modular Liquid Glass blocks, Focus Modes, tactual Sliders (Brightness/Volume),
//!   and Now Playing MPRIS2 widget with Median Cut dominant color extraction.
//! - **Notification Center**: 3D Notification Stacks grouped by app, spring rollout unroll, and inline replies.
//! - **Interactive Widgets**: Desktop & panel widgets with 250ms Adaptive Chromatic Transition to monochrome.
//! - **Tripartite Dock**: Pinned/running apps, recent apps, Stacks across 4 modes (Fan, Grid, List, Auto),
//!   analytical parabolic cosine magnification, launch bounce, 500ms Spring-Loaded Exposé,
//!   separator resizing with Option snap, and dynamic Trash/Eject morphing.

#![deny(missing_docs)]

pub mod control_center;
pub mod dock;
pub mod menubar;
pub mod notifications;
pub mod widgets;

pub use control_center::{
    ControlCenterController, FocusDuration, FocusMode, NowPlayingState,
};
pub use dock::{
    DockAppItem, DockCoordinator, DockStack, StackItem, StackViewMode, TrashMode,
};
pub use menubar::{
    AppleMenuAction, AppleMenuBuilder, AppleMenuItem, BluetoothDiagnostics,
    DisplayNotch, GlobalMenuManager, HelpSearchResult, MenuNode, MenuNotchPartitioner,
    PositionedMenuItem, PrivacyCapsuleManager, PrivacySensor, SensorAccess,
    StatusIconKind, StatusItem, StatusItemsManager, WifiDiagnostics, WindowMenuEntry,
};
pub use notifications::{
    NotificationCenterManager, NotificationItem, NotificationStack,
};
pub use widgets::{
    DesktopWidget, WidgetColorMode, WidgetKind, WidgetManager, WidgetSize,
};
