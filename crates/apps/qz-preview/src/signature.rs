//! # Trackpad Signature Capture & Vault Engine (`qz-preview::signature`)
//!
//! Production-grade trackpad digital signature system for macOS Tahoe 26 Preview (`Preview.app`).
//!
//! ### Key Capabilities:
//! - **Trackpad Drawing Surface:** Captures raw touch inputs in normalized coordinates
//!   $(u, v) \in [0.0, 1.0] \times [0.0, 1.0]$ with normalized pressure $p \in [0.0, 1.0]$.
//! - **Cubic Hermite Spline Interpolation:** Smooths captured touch points with $C^1$ continuity
//!   and realistic variable line thickness derived from velocity and pressure.
//! - **Signature Vault:** Manages saved signatures ("Formal Signature", "Initials", etc.)
//!   and handles placement onto PDF documents with arbitrary bounding box scaling.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// A single sample point recorded on the trackpad surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignatureSample {
    /// Normalized trackpad coordinate $[u, v] \in [0.0, 1.0] \times [0.0, 1.0]$.
    pub uv: Vec2,
    /// Normalized pressure $[0.0, 1.0]$.
    pub pressure: f32,
    /// High-resolution timestamp in seconds.
    pub timestamp: f64,
}

impl SignatureSample {
    /// Creates a new `SignatureSample`.
    #[must_use]
    pub fn new(uv: Vec2, pressure: f32, timestamp: f64) -> Self {
        Self {
            uv: Vec2::new(uv.x.clamp(0.0, 1.0), uv.y.clamp(0.0, 1.0)),
            pressure: pressure.clamp(0.0, 1.0),
            timestamp,
        }
    }
}

/// Computes cubic Hermite interpolation between two points given their velocity tangents.
///
/// Formula:
/// $$P(t) = (2t^3 - 3t^2 + 1) P_0 + (t^3 - 2t^2 + t) M_0 + (-2t^3 + 3t^2) P_1 + (t^3 - t^2) M_1$$
#[must_use]
pub fn hermite_interpolate(p0: Vec2, m0: Vec2, p1: Vec2, m1: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    h00 * p0 + h10 * m0 + h01 * p1 + h11 * m1
}

/// Calculates ink stroke width in points given instantaneous pressure and speed.
///
/// Thinner stroke at high velocity (calligraphic flick), thicker stroke at high pressure.
#[must_use]
pub fn compute_ink_width(pressure: f32, speed: f32, base_width: f32) -> f32 {
    let p = pressure.clamp(0.1, 1.0);
    let speed_damping = (1.0 - (speed * 0.005).min(0.6)).max(0.4);
    let width = base_width * (0.6 + 0.8 * p) * speed_damping;
    width.clamp(1.0, base_width * 2.2)
}

/// A continuous stroke consisting of sequential touch points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignatureStroke {
    /// Samples making up this stroke.
    pub samples: Vec<SignatureSample>,
}

impl SignatureStroke {
    /// Creates a new empty stroke.
    #[must_use]
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Appends a touch sample to the stroke.
    pub fn add_sample(&mut self, sample: SignatureSample) {
        self.samples.push(sample);
    }

    /// Evaluates smoothed points along the stroke using cubic Hermite interpolation.
    #[must_use]
    pub fn evaluate_smoothed_points(&self, samples_per_segment: usize) -> Vec<Vec2> {
        if self.samples.len() < 2 {
            return self.samples.iter().map(|s| s.uv).collect();
        }

        let mut result = Vec::new();
        let count = self.samples.len();

        for i in 0..count - 1 {
            let p0 = self.samples[i].uv;
            let p1 = self.samples[i + 1].uv;

            // Finite difference tangents
            let m0 = if i == 0 {
                p1 - p0
            } else {
                0.5 * (p1 - self.samples[i - 1].uv)
            };

            let m1 = if i + 1 == count - 1 {
                p1 - p0
            } else {
                0.5 * (self.samples[i + 2].uv - p0)
            };

            for step in 0..samples_per_segment {
                let t = step as f32 / samples_per_segment as f32;
                result.push(hermite_interpolate(p0, m0, p1, m1, t));
            }
        }

        if let Some(last) = self.samples.last() {
            result.push(last.uv);
        }

        result
    }
}

impl Default for SignatureStroke {
    fn default() -> Self {
        Self::new()
    }
}

/// A stored digital signature profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedSignature {
    /// Unique identifier (e.g. UUID or timestamp string).
    pub id: String,
    /// User-friendly label (e.g. "Formal Signature", "Initials").
    pub name: String,
    /// Creation timestamp (Unix epoch seconds).
    pub created_at_epoch: u64,
    /// Component strokes captured from the trackpad.
    pub strokes: Vec<SignatureStroke>,
    /// Normalized aspect ratio (width / height) computed from tight bounding box.
    pub aspect_ratio: f32,
}

impl SavedSignature {
    /// Creates a new `SavedSignature` and computes its intrinsic aspect ratio.
    #[must_use]
    pub fn new(id: &str, name: &str, created_at_epoch: u64, strokes: Vec<SignatureStroke>) -> Self {
        let mut min = Vec2::new(1.0, 1.0);
        let mut max = Vec2::new(0.0, 0.0);
        let mut has_points = false;

        for stroke in &strokes {
            for sample in &stroke.samples {
                min = min.min(sample.uv);
                max = max.max(sample.uv);
                has_points = true;
            }
        }

        let aspect_ratio = if has_points && (max.y - min.y) > 0.001 {
            (max.x - min.x) / (max.y - min.y)
        } else {
            2.5 // Default signature aspect ratio
        };

        Self {
            id: id.to_string(),
            name: name.to_string(),
            created_at_epoch,
            strokes,
            aspect_ratio: aspect_ratio.clamp(0.5, 10.0),
        }
    }

    /// Computes target placement bounding box $[x, y, w, h]$ on a page given
    /// desired width, preserving signature aspect ratio.
    #[must_use]
    pub fn placement_bounds(&self, origin: Vec2, target_width: f32) -> [f32; 4] {
        let target_height = target_width / self.aspect_ratio;
        [origin.x, origin.y, target_width, target_height]
    }
}

/// Manages stored signatures on disk.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureVault {
    signatures: Vec<SavedSignature>,
}

impl SignatureVault {
    /// Creates a new empty `SignatureVault`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            signatures: Vec::new(),
        }
    }

    /// Adds a signature to the vault.
    pub fn add_signature(&mut self, signature: SavedSignature) {
        self.signatures.retain(|s| s.id != signature.id);
        self.signatures.push(signature);
    }

    /// Removes a signature by ID.
    pub fn remove_signature(&mut self, id: &str) -> bool {
        let init_len = self.signatures.len();
        self.signatures.retain(|s| s.id != id);
        self.signatures.len() < init_len
    }

    /// Finds a signature by ID.
    #[must_use]
    pub fn get_signature(&self, id: &str) -> Option<&SavedSignature> {
        self.signatures.iter().find(|s| s.id == id)
    }

    /// Returns all saved signatures.
    #[must_use]
    pub fn signatures(&self) -> &[SavedSignature] {
        &self.signatures
    }

    /// Number of saved signatures in vault.
    #[must_use]
    pub fn len(&self) -> usize {
        self.signatures.len()
    }

    /// Returns `true` if vault is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hermite_interpolation() {
        let p0 = Vec2::new(0.0, 0.0);
        let m0 = Vec2::new(1.0, 0.0);
        let p1 = Vec2::new(10.0, 10.0);
        let m1 = Vec2::new(0.0, 1.0);

        // At t = 0.0
        let start = hermite_interpolate(p0, m0, p1, m1, 0.0);
        assert!((start - p0).length() < 1e-4);

        // At t = 1.0
        let end = hermite_interpolate(p0, m0, p1, m1, 1.0);
        assert!((end - p1).length() < 1e-4);

        // At t = 0.5
        let mid = hermite_interpolate(p0, m0, p1, m1, 0.5);
        assert!(mid.x > 0.0 && mid.y > 0.0);
    }

    #[test]
    fn test_signature_vault_and_placement() {
        let mut vault = SignatureVault::new();
        assert!(vault.is_empty());

        let mut stroke = SignatureStroke::new();
        stroke.add_sample(SignatureSample::new(Vec2::new(0.1, 0.2), 0.5, 0.0));
        stroke.add_sample(SignatureSample::new(Vec2::new(0.5, 0.4), 0.8, 0.1));
        stroke.add_sample(SignatureSample::new(Vec2::new(0.9, 0.2), 0.3, 0.2));

        let sig = SavedSignature::new("sig-1", "Formal Signature", 1700000000, vec![stroke]);
        vault.add_signature(sig);

        assert_eq!(vault.len(), 1);
        let retrieved = vault.get_signature("sig-1").expect("signature exists");
        assert_eq!(retrieved.name, "Formal Signature");

        let bounds = retrieved.placement_bounds(Vec2::new(50.0, 100.0), 200.0);
        assert_eq!(bounds[0], 50.0);
        assert_eq!(bounds[1], 100.0);
        assert_eq!(bounds[2], 200.0);
        assert!(bounds[3] > 10.0); // Computed height according to aspect ratio
    }
}
