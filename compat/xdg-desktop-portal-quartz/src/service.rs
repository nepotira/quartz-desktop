//! # Quartz Portal Master Service Coordinator (`xdg-desktop-portal-quartz::service`)
//!
//! Production-grade XDG Desktop Portal master coordinator for QuartzLinux (`org.freedesktop.impl.portal.desktop.quartz`).
//!
//! ### Key Capabilities:
//! - **Unified Service Dispatcher:** Routes incoming D-Bus portal invocations for
//!   `FileChooser`, `Screenshot`, `Settings`, and `ScreenCast`.
//! - **Portal Configuration Generator:** Produces `quartz-portals.conf` configuring
//!   XDG Desktop Portal frontends to route interfaces to Quartz.

use crate::file_chooser::{FileChooserOptions, FileChooserSheet};
use crate::screencast::ScreenCastPortal;
use crate::screenshot::{PickColorResult, ScreenshotPortal, ScreenshotRequest, ScreenshotResult};
use crate::settings::SettingsPortal;
use serde::{Deserialize, Serialize};

/// Master service coordinating all portal implementations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuartzPortalService {
    /// Active FileChooser modal sheet if currently presented.
    pub active_file_chooser: Option<FileChooserSheet>,
    /// Settings portal manager.
    pub settings: SettingsPortal,
    /// ScreenCast portal manager.
    pub screencast: ScreenCastPortal,
}

impl QuartzPortalService {
    /// Creates a new `QuartzPortalService`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_file_chooser: None,
            settings: SettingsPortal::new(),
            screencast: ScreenCastPortal::new(),
        }
    }

    /// Presents a modal Sheet FileChooser for a client.
    pub fn open_file_chooser(&mut self, options: FileChooserOptions) {
        self.active_file_chooser = Some(FileChooserSheet::new(options));
    }

    /// Dispatches a Screenshot request.
    #[must_use]
    pub fn take_screenshot(&self, request: &ScreenshotRequest) -> ScreenshotResult {
        ScreenshotPortal::handle_screenshot(request)
    }

    /// Dispatches a Digital Color Meter PickColor request.
    #[must_use]
    pub fn pick_color(&self) -> PickColorResult {
        ScreenshotPortal::handle_pick_color()
    }

    /// Generates the standard FreeDesktop `quartz-portals.conf` content.
    #[must_use]
    pub fn generate_portal_config() -> &'static str {
        "[preferred]\ndefault=quartz\norg.freedesktop.impl.portal.FileChooser=quartz\norg.freedesktop.impl.portal.Screenshot=quartz\norg.freedesktop.impl.portal.Settings=quartz\norg.freedesktop.impl.portal.ScreenCast=quartz\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_service_lifecycle_and_config() {
        let mut service = QuartzPortalService::new();
        assert!(service.active_file_chooser.is_none());

        service.open_file_chooser(FileChooserOptions::default());
        assert!(service.active_file_chooser.is_some());

        let shot = service.take_screenshot(&ScreenshotRequest::default());
        assert!(!shot.uri.is_empty());

        let conf = QuartzPortalService::generate_portal_config();
        assert!(conf.contains("org.freedesktop.impl.portal.FileChooser=quartz"));
        assert!(conf.contains("org.freedesktop.impl.portal.Screenshot=quartz"));
    }
}
