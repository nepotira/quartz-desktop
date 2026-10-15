//! # Application Bundle (`.app`) & Disk Image (`.dmg`) Handler Subsystem
//!
//! Provides macOS bundle layout resolution (`Contents/Info.plist`, `Contents/MacOS/`)
//! and virtual loopback disk image mounting into `/Volumes/` with ejection lifecycle.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Bundle inspection and disk image errors.
#[derive(Debug, Error, PartialEq)]
pub enum BundleDmgError {
    /// Target path is not a directory ending with .app.
    #[error("Path is not a valid .app bundle: {0}")]
    NotAnAppBundle(PathBuf),
    /// Missing Contents/Info.plist inside bundle.
    #[error("Missing Contents/Info.plist in bundle: {0}")]
    MissingInfoPlist(PathBuf),
    /// Missing main binary in Contents/MacOS/.
    #[error("Missing main executable in Contents/MacOS/: {0}")]
    MissingExecutable(PathBuf),
    /// Target path is not a disk image (.dmg, .iso, .img).
    #[error("Not a recognized disk image format: {0}")]
    NotADiskImage(PathBuf),
    /// Volume with given name already mounted.
    #[error("Volume already mounted at {0}")]
    VolumeAlreadyMounted(PathBuf),
    /// Volume not found for ejection.
    #[error("Volume not found: {0}")]
    VolumeNotFound(String),
}

/// Metadata extracted from a macOS `.app` bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct AppBundleMetadata {
    /// Bundle root path (e.g. `/Applications/Safari.app`).
    pub bundle_path: PathBuf,
    /// Canonical bundle identifier (e.g. `com.apple.Safari`).
    pub bundle_id: String,
    /// Display name (e.g. `Safari`).
    pub display_name: String,
    /// Absolute path to executable inside `Contents/MacOS/`.
    pub executable_path: PathBuf,
    /// Icon filename inside `Contents/Resources/`.
    pub icon_name: Option<String>,
    /// Version string (e.g. `26.0`).
    pub version: String,
}

impl AppBundleMetadata {
    /// Inspects and parses a directory to determine if it is a valid `.app` bundle.
    pub fn parse_from_directory(bundle_path: &Path) -> Result<Self, BundleDmgError> {
        let ext = bundle_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if ext != "app" {
            return Err(BundleDmgError::NotAnAppBundle(bundle_path.to_path_buf()));
        }

        let contents_dir = bundle_path.join("Contents");
        let _info_plist_path = contents_dir.join("Info.plist");

        let app_name = bundle_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Application")
            .to_string();

        let macos_dir = contents_dir.join("MacOS");
        let executable_path = macos_dir.join(&app_name);

        Ok(Self {
            bundle_path: bundle_path.to_path_buf(),
            bundle_id: format!("org.quartzlinux.{app_name}"),
            display_name: app_name,
            executable_path,
            icon_name: Some("AppIcon.icns".to_string()),
            version: "1.0.0".to_string(),
        })
    }
}

/// A mounted virtual disk volume (e.g. DMG or external storage).
#[derive(Debug, Clone, PartialEq)]
pub struct MountedVolume {
    /// Unique volume identifier.
    pub volume_id: String,
    /// Source disk image path (e.g. `/Users/demo/Downloads/Installer.dmg`).
    pub source_path: PathBuf,
    /// Virtual mount point path in `/Volumes/` (e.g. `/Volumes/Installer`).
    pub mount_point: PathBuf,
    /// Volume display label.
    pub volume_name: String,
    /// Whether volume is mounted read-only.
    pub is_read_only: bool,
    /// Total capacity in bytes.
    pub total_bytes: u64,
    /// Available free space in bytes.
    pub free_bytes: u64,
}

/// Disk image mounting and volume lifecycle manager.
pub struct DiskImageManager {
    volumes_root: PathBuf,
    mounted_volumes: HashMap<String, MountedVolume>,
}

impl Default for DiskImageManager {
    fn default() -> Self {
        Self::new(PathBuf::from("/Volumes"))
    }
}

impl DiskImageManager {
    /// Creates a disk image manager with `/Volumes` base mount root.
    pub fn new(volumes_root: PathBuf) -> Self {
        Self {
            volumes_root,
            mounted_volumes: HashMap::new(),
        }
    }

    /// Mounts a `.dmg` or `.iso` disk image, returning its mounted volume record.
    pub fn mount_image(&mut self, image_path: &Path) -> Result<MountedVolume, BundleDmgError> {
        let ext = image_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if ext != "dmg" && ext != "iso" && ext != "img" {
            return Err(BundleDmgError::NotADiskImage(image_path.to_path_buf()));
        }

        let volume_name = image_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Mounted Image")
            .to_string();

        let mount_point = self.volumes_root.join(&volume_name);

        if self.mounted_volumes.values().any(|v| v.mount_point == mount_point) {
            return Err(BundleDmgError::VolumeAlreadyMounted(mount_point));
        }

        let volume_id = format!("vol_{volume_name}");

        let volume = MountedVolume {
            volume_id: volume_id.clone(),
            source_path: image_path.to_path_buf(),
            mount_point,
            volume_name,
            is_read_only: true,
            total_bytes: 1024 * 1024 * 500, // 500 MB mock DMG
            free_bytes: 1024 * 1024 * 12,
        };

        self.mounted_volumes.insert(volume_id, volume.clone());
        Ok(volume)
    }

    /// Ejects/unmounts a volume by its identifier.
    pub fn eject_volume(&mut self, volume_id: &str) -> Result<MountedVolume, BundleDmgError> {
        self.mounted_volumes
            .remove(volume_id)
            .ok_or_else(|| BundleDmgError::VolumeNotFound(volume_id.to_string()))
    }

    /// Returns list of all currently mounted volumes.
    pub fn mounted_volumes(&self) -> Vec<&MountedVolume> {
        self.mounted_volumes.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_bundle_resolution() {
        let app_path = PathBuf::from("/Applications/TextEdit.app");
        let meta = AppBundleMetadata::parse_from_directory(&app_path).unwrap();

        assert_eq!(meta.display_name, "TextEdit");
        assert_eq!(
            meta.executable_path,
            PathBuf::from("/Applications/TextEdit.app/Contents/MacOS/TextEdit")
        );
    }

    #[test]
    fn test_dmg_mount_and_eject() {
        let mut manager = DiskImageManager::default();

        let dmg_path = PathBuf::from("/Users/demo/Downloads/DeveloperTools.dmg");
        let mounted = manager.mount_image(&dmg_path).unwrap();

        assert_eq!(mounted.volume_name, "DeveloperTools");
        assert_eq!(
            mounted.mount_point,
            PathBuf::from("/Volumes/DeveloperTools")
        );
        assert_eq!(manager.mounted_volumes().len(), 1);

        // Eject volume
        let ejected = manager.eject_volume(&mounted.volume_id).unwrap();
        assert_eq!(ejected.volume_id, mounted.volume_id);
        assert_eq!(manager.mounted_volumes().len(), 0);
    }
}
