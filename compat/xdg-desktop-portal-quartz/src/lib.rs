//! # QuartzLinux Native XDG Desktop Portal (`xdg-desktop-portal-quartz`)
//!
//! Production-grade XDG Desktop Portal implementation for QuartzLinux (`qz-desktop`),
//! built to be visually, mathematically, physically, and functionally indistinguishable
//! from macOS Tahoe 26 system dialogs and services.
//!
//! ### Core Subsystems:
//! - [`file_chooser`]: Native modal sheet FileChooser dropping down from parent window titlebars
//!   with drag-to-retarget breadcrumbs, integrated Spacebar Quick Look preview, and MIME pattern filtering.
//! - [`screenshot`]: Fullscreen, area selection, and isolated window capture with alpha and dual-layer erf shadows,
//!   plus Digital Color Meter loupe pipette.
//! - [`settings`]: FreeDesktop Appearance portal synchronization (`color-scheme`, `accent-color`, `contrast`).
//! - [`screencast`]: PipeWire media stream handshake at 60Hz or 120Hz ProMotion VRR.
//! - [`service`]: Master portal coordinator routing D-Bus interfaces and generating `quartz-portals.conf`.

#![deny(missing_docs)]

pub mod file_chooser;
pub mod screencast;
pub mod screenshot;
pub mod settings;
pub mod service;

// Re-exports for convenient top-level access
pub use file_chooser::{
    FileChooserAction, FileChooserError, FileChooserOptions, FileChooserSheet, FileItem,
    FilterPattern,
};
pub use screencast::{CastSource, CastStream, CursorMode, ScreenCastPortal};
pub use screenshot::{PickColorResult, ScreenshotMode, ScreenshotPortal, ScreenshotRequest, ScreenshotResult};
pub use service::QuartzPortalService;
pub use settings::{AppearanceSettings, ColorSchemePreference, SettingsPortal};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portal_end_to_end_integration() {
        let mut service = QuartzPortalService::new();

        // 1. FileChooser in modal sheet with drag-to-retarget
        let options = FileChooserOptions {
            title: "Open Document".to_string(),
            action: FileChooserAction::OpenFile,
            parent_window_id: Some("wayland-surface-42".to_string()),
            initial_folder: "/home/nepo/Documents".to_string(),
            default_name: None,
            filters: vec![FilterPattern {
                name: "PDFs".to_string(),
                patterns: vec!["*.pdf".to_string()],
            }],
            active_filter_index: 0,
        };
        service.open_file_chooser(options);
        assert!(service.active_file_chooser.is_some());

        let sheet = service.active_file_chooser.as_mut().unwrap();
        sheet.set_items(vec![FileItem {
            name: "Proposal.pdf".to_string(),
            path: "/home/nepo/Documents/Proposal.pdf".to_string(),
            is_directory: false,
            size_bytes: 4096,
            mime_type: "application/pdf".to_string(),
        }]);

        // Drag to retarget
        sheet.handle_drag_retarget("/home/nepo/Desktop");
        assert_eq!(sheet.active_folder, "/home/nepo/Desktop");

        // 2. Settings portal
        let color_scheme = service.settings.read_setting("org.freedesktop.appearance", "color-scheme");
        assert_eq!(color_scheme, Some("1".to_string()));

        // 3. ScreenCast stream at 120Hz
        let stream = service.screencast.create_stream(
            CastSource::Window("app-win-1".to_string()),
            1920,
            1080,
            120,
            CursorMode::Embedded,
        );
        assert_eq!(stream.framerate_hz, 120);

        // 4. Portal config file
        let conf = QuartzPortalService::generate_portal_config();
        assert!(conf.contains("org.freedesktop.impl.portal.FileChooser=quartz"));
    }
}
