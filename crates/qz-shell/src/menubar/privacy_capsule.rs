//! # Hardware Privacy Capsule Subsystem
//!
//! Real-time hardware sensor access indicators matching macOS Tahoe 26:
//! - **Orange Dot (`#FF9500`)**: Active microphone stream detected (PipeWire).
//! - **Green Dot (`#34C759`)**: Active webcam/camera stream detected (V4L2).
//! - **Purple Dot (`#AF52DE`)**: Active screen recording or window capture session.
//! - Expands an animated Liquid Glass capsule upon sensor activation revealing
//!   the application name, and exposes sensor state at the top of Control Center.

use glam::Vec4;
use qz_animation::{SpringAnimation, SpringParams};

/// Active hardware sensor type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrivacySensor {
    /// Microphone audio capture.
    Microphone,
    /// Video camera capture.
    Camera,
    /// Screen or window frame recording.
    ScreenRecording,
}

impl PrivacySensor {
    /// Dot indicator color in RGBA matching Apple design guidelines.
    pub fn indicator_color(&self) -> Vec4 {
        match self {
            Self::Microphone => Vec4::new(1.000, 0.584, 0.000, 1.0), // Orange #FF9500
            Self::Camera => Vec4::new(0.204, 0.780, 0.349, 1.0),     // Green #34C759
            Self::ScreenRecording => Vec4::new(0.686, 0.322, 0.871, 1.0), // Purple #AF52DE
        }
    }
}

/// An active hardware sensor access record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorAccess {
    /// Sensor being accessed.
    pub sensor: PrivacySensor,
    /// Application ID accessing hardware (e.g. "org.mozilla.firefox").
    pub client_app_id: String,
    /// Human-readable application name (e.g. "Firefox").
    pub client_app_name: String,
}

/// Manages privacy dots and the expanding Liquid Glass capsule.
pub struct PrivacyCapsuleManager {
    active_accesses: Vec<SensorAccess>,
    expansion_spring: SpringAnimation,
    is_expanded: bool,
}

impl Default for PrivacyCapsuleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PrivacyCapsuleManager {
    /// Creates a new privacy sensor coordinator.
    pub fn new() -> Self {
        Self {
            active_accesses: Vec::new(),
            expansion_spring: SpringAnimation::new(SpringParams::window_open(), 0.0, 0.0, 0.0),
            is_expanded: false,
        }
    }

    /// Registers a newly initiated hardware sensor stream.
    pub fn register_sensor_access(&mut self, access: SensorAccess) {
        if !self.active_accesses.contains(&access) {
            self.active_accesses.push(access);
            // Trigger animated expansion of the capsule
            self.is_expanded = true;
            self.expansion_spring.retarget(1.0);
        }
    }

    /// Unregisters a terminated sensor stream.
    pub fn revoke_sensor_access(&mut self, sensor: PrivacySensor, app_id: &str) {
        self.active_accesses.retain(|a| !(a.sensor == sensor && a.client_app_id == app_id));
        if self.active_accesses.is_empty() {
            // Collapse capsule back into minimal dot
            self.is_expanded = false;
            self.expansion_spring.retarget(0.0);
        }
    }

    /// Whether any hardware sensor is currently active.
    #[inline]
    pub fn has_active_sensors(&self) -> bool {
        !self.active_accesses.is_empty()
    }

    /// Returns list of active hardware accesses.
    pub fn active_accesses(&self) -> &[SensorAccess] {
        &self.active_accesses
    }

    /// Current animated capsule expansion progress $[0.0, 1.0]$.
    pub fn expansion_progress(&self) -> f32 {
        self.expansion_spring.sample_at(self.expansion_spring.elapsed_time()).position.clamp(0.0, 1.0) as f32
    }

    /// Advances the spring animation by `dt` seconds.
    pub fn step(&mut self, dt: f64) {
        self.expansion_spring.step(dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_dot_colors() {
        let mic = PrivacySensor::Microphone;
        assert_eq!(mic.indicator_color().x, 1.000);

        let cam = PrivacySensor::Camera;
        assert_eq!(cam.indicator_color().y, 0.780);

        let rec = PrivacySensor::ScreenRecording;
        assert_eq!(rec.indicator_color().z, 0.871);
    }

    #[test]
    fn test_privacy_capsule_expansion() {
        let mut manager = PrivacyCapsuleManager::new();
        assert!(!manager.has_active_sensors());

        manager.register_sensor_access(SensorAccess {
            sensor: PrivacySensor::Camera,
            client_app_id: "org.quartz.preview".to_string(),
            client_app_name: "Preview".to_string(),
        });

        assert!(manager.has_active_sensors());
        assert!(manager.is_expanded);

        manager.revoke_sensor_access(PrivacySensor::Camera, "org.quartz.preview");
        assert!(!manager.has_active_sensors());
        assert!(!manager.is_expanded);
    }
}
