//! # qz-compositor
//!
//! Wayland Core Compositor engine for QuartzLinux (macOS Tahoe 26 architecture):
//! - **ProMotion Dynamic VRR (10Hz–120Hz)**: Adaptive frame pacing scheduler driven by springs and gestures.
//! - **Retina Fractional Scaling**: EDID physical DPI computation and `wp-fractional-scale-v1` quantizer ($1/120$).
//! - **Multi-Touch Trackpad Gestures**: 1:1 interaction engine with continuous spring physics (`CASpringAnimation`).
//! - **Spaces, Stage Manager & PiP**: Virtual desktop carousel, 3D tilted Stage Manager lateral groups, and magnetic PiP.
//! - **Input & Click-Through**: `Cmd`-Click-Through, unfocused scroll routing, and POSIX terminal signal isolation.

#![deny(missing_docs)]

pub mod gestures;
pub mod input;
pub mod retina;
pub mod spaces;
pub mod tiling;
pub mod vrr;

pub use gestures::{GestureEngine, GesturePhase, TrackpadGesture};
pub use input::{
    ClickThroughAction, ClickThroughDispatcher, ClickThroughResult, KeyMappingResult,
    ModifierState, SmartKeymapper, TargetAppKind, WindowSurfaceInfo,
};
pub use retina::{ClientScaleMode, MonitorPhysicalSpecs, RetinaCoordinator};
pub use spaces::{
    HotCornerAction, PipCorner, PipController, ScreenCorner, Space, SpaceKind,
    SpacesCoordinator, StageManagerGroup,
};
pub use tiling::{TilingGeometry, TilingLayoutManager, TilingOption, TAHOE_TILING_MARGIN};
pub use vrr::{FramePaceMode, ProMotionScheduler};
