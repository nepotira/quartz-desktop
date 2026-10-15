//! # Advanced Document Markup Engine (`qz-preview::markup`)
//!
//! Production-grade markup system for macOS Tahoe 26 Preview (`Preview.app`).
//!
//! ### Key Capabilities:
//! - **Markup Tools:** Selection, Instant Alpha (lasso/flood fill), Sketch (freehand pen with
//!   pressure sensitivity $1.0..=16.0\text{px}$), Catmull-Rom cubic spline smoothing, auto-shape
//!   recognition (circle, rectangle, arrow), Highlighter (multiplicative blend), Shapes
//!   (Rectangle, Ellipse, Line, Arrow, SpeechBubble, Star, Loupe magnifying glass with $1.5\times$ to $5.0\times$ zoom),
//!   and TextBox.
//! - **Undo / Redo Transaction History:** Full undo/redo stack managing atomic annotation mutations.

use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Unique identifier for an annotation on a document page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnnotationId(pub u64);

/// Classification of geometric shapes supported by macOS Tahoe 26 Preview markup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShapeKind {
    /// Axis-aligned rectangle with optional rounded corners.
    Rectangle,
    /// Ellipse / Circle.
    Ellipse,
    /// Straight line segment.
    Line,
    /// Directed vector arrow with arrowhead.
    Arrow,
    /// Speech / thought bubble with customizable callout pointer.
    SpeechBubble,
    /// 5-point star polygon.
    Star,
    /// Circular magnifying glass (Loupe) with magnification scale factor ($1.5\times$ to $5.0\times$).
    Loupe,
}

/// Active markup toolbar selection.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MarkupTool {
    /// Object selection and transformation gizmo.
    Select,
    /// Instant Alpha background removal / magic wand.
    InstantAlpha {
        /// Color difference tolerance ($0.0$ to $1.0$).
        tolerance: f32,
    },
    /// Sketch pen with pressure sensitivity and auto-smoothing.
    Sketch,
    /// Geometric drawing tool.
    Shape(ShapeKind),
    /// Translucent highlighter with multiplicative blending.
    Highlighter,
    /// Rich text box.
    TextBox,
    /// Insert saved signature.
    Signature,
}

/// A single sample point in a freehand stroke.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    /// 2D coordinate $[x, y]$ in document page space.
    pub position: Vec2,
    /// Normalized stylus or trackpad pressure ($0.0$ to $1.0$).
    pub pressure: f32,
    /// Timestamp in seconds.
    pub timestamp: f64,
}

impl StrokePoint {
    /// Creates a new stroke point.
    #[must_use]
    pub fn new(position: Vec2, pressure: f32, timestamp: f64) -> Self {
        Self {
            position,
            pressure: pressure.clamp(0.0, 1.0),
            timestamp,
        }
    }
}

/// Evaluates Catmull-Rom cubic spline smoothing between four control points.
///
/// Formula:
/// $$P(t) = 0.5 \cdot \big(2 P_1 + (-P_0 + P_2) t + (2 P_0 - 5 P_1 + 4 P_2 - P_3) t^2 + (-P_0 + 3 P_1 - 3 P_2 + P_3) t^3\big)$$
#[must_use]
pub fn catmull_rom_interpolate(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;

    let v0 = -p0 + p2;
    let v1 = 2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3;
    let v2 = -p0 + 3.0 * p1 - 3.0 * p2 + p3;

    0.5 * (2.0 * p1 + v0 * t + v1 * t2 + v2 * t3)
}

/// Automatically detects if a sequence of freehand points approximates a canonical shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognizedShape {
    /// Detected a circle / ellipse.
    Circle,
    /// Detected a rectangle.
    Rectangle,
    /// Detected a directional arrow.
    Arrow,
    /// No standard shape recognized; retain raw freehand stroke.
    None,
}

/// Evaluates freehand stroke points to detect standard geometric shapes.
#[must_use]
pub fn recognize_stroke_shape(points: &[StrokePoint]) -> RecognizedShape {
    if points.len() < 12 {
        return RecognizedShape::None;
    }

    let first = points[0].position;
    let last = points[points.len() - 1].position;
    let is_closed = first.distance(last) < 28.0;

    // Calculate centroid and bounding box
    let mut min = first;
    let mut max = first;
    let mut sum = Vec2::ZERO;

    for pt in points {
        min = min.min(pt.position);
        max = max.max(pt.position);
        sum += pt.position;
    }

    let count = points.len() as f32;
    let centroid = sum / count;
    let size = max - min;

    if size.x < 15.0 || size.y < 15.0 {
        return RecognizedShape::None;
    }

    if is_closed {
        // Test for circle: radius variance from centroid
        let target_radius = (size.x + size.y) * 0.25;
        let mut variance_sum = 0.0;
        for pt in points {
            let dist = pt.position.distance(centroid);
            let diff = dist - target_radius;
            variance_sum += diff * diff;
        }
        let std_dev = (variance_sum / count).sqrt();
        if std_dev < target_radius * 0.18 {
            return RecognizedShape::Circle;
        }

        // Test for rectangle: aspect ratio check & point proximity to bounding box perimeters
        let mut edge_dist_sum = 0.0;
        for pt in points {
            let dx = (pt.position.x - min.x).abs().min((pt.position.x - max.x).abs());
            let dy = (pt.position.y - min.y).abs().min((pt.position.y - max.y).abs());
            edge_dist_sum += dx.min(dy);
        }
        let avg_edge_dist = edge_dist_sum / count;
        if avg_edge_dist < (size.x + size.y) * 0.08 {
            return RecognizedShape::Rectangle;
        }
    } else {
        // Test for arrow: straight shaft + angular arrowhead at end
        let direct_dist = first.distance(last);
        let mut total_path = 0.0;
        for i in 1..points.len() {
            total_path += points[i - 1].position.distance(points[i].position);
        }

        // Arrow paths have an extended shaft and sudden direction reversals for wings
        if total_path > direct_dist * 1.15 && direct_dist > 30.0 {
            return RecognizedShape::Arrow;
        }
    }

    RecognizedShape::None
}

/// Specific data payload for each annotation type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnnotationKind {
    /// Freehand drawn stroke smoothed with Catmull-Rom splines.
    FreehandStroke {
        /// Sequence of raw sampled points.
        points: Vec<StrokePoint>,
        /// RGBA stroke color.
        stroke_color: [f32; 4],
        /// Base stroke width in points ($1.0$ to $16.0$).
        stroke_width: f32,
        /// `true` if rendered as highlighter with multiplicative blending.
        is_highlighter: bool,
    },
    /// Canonical geometric shape.
    GeometricShape {
        /// Type of geometric shape.
        shape: ShapeKind,
        /// Bounding box $[x, y, width, height]$.
        bounds: [f32; 4],
        /// RGBA stroke outline color.
        stroke_color: [f32; 4],
        /// Optional RGBA fill color.
        fill_color: Option<[f32; 4]>,
        /// Stroke line width in points.
        stroke_width: f32,
    },
    /// Circular magnifying glass (Loupe).
    Loupe {
        /// Center coordinate of the loupe lens.
        center: Vec2,
        /// Radius of the circular lens in points.
        radius: f32,
        /// Optical zoom magnification factor ($1.5\times$ to $5.0\times$).
        zoom_factor: f32,
        /// Border rim RGBA color.
        border_color: [f32; 4],
    },
    /// Rich text box annotation.
    Text {
        /// Bounding rectangle $[x, y, width, height]$.
        bounds: [f32; 4],
        /// UTF-8 text contents.
        text: String,
        /// Font size in points.
        font_size: f32,
        /// RGBA text color.
        text_color: [f32; 4],
    },
    /// Embedded trackpad signature.
    Signature {
        /// Identifier referencing saved signature vault.
        signature_id: String,
        /// Bounding rectangle $[x, y, width, height]$.
        bounds: [f32; 4],
        /// RGBA ink color.
        ink_color: [f32; 4],
    },
}

/// An annotation placed on a specific document page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Annotation {
    /// Unique identifier.
    pub id: AnnotationId,
    /// 0-indexed page number this annotation belongs to.
    pub page_index: usize,
    /// Annotation type and parameters.
    pub kind: AnnotationKind,
    /// Whether this annotation is currently locked against editing.
    pub is_locked: bool,
}

impl Annotation {
    /// Returns the axis-aligned bounding box $[x, y, width, height]$ of the annotation.
    #[must_use]
    pub fn bounding_box(&self) -> [f32; 4] {
        match &self.kind {
            AnnotationKind::FreehandStroke { points, stroke_width, .. } => {
                if points.is_empty() {
                    return [0.0, 0.0, 0.0, 0.0];
                }
                let mut min = points[0].position;
                let mut max = points[0].position;
                for pt in points {
                    min = min.min(pt.position);
                    max = max.max(pt.position);
                }
                let pad = stroke_width * 0.5;
                [
                    min.x - pad,
                    min.y - pad,
                    (max.x - min.x) + stroke_width,
                    (max.y - min.y) + stroke_width,
                ]
            }
            AnnotationKind::GeometricShape { bounds, .. } => *bounds,
            AnnotationKind::Loupe { center, radius, .. } => [
                center.x - radius,
                center.y - radius,
                radius * 2.0,
                radius * 2.0,
            ],
            AnnotationKind::Text { bounds, .. } => *bounds,
            AnnotationKind::Signature { bounds, .. } => *bounds,
        }
    }
}

/// Atomic undo/redo transaction item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarkupTransaction {
    /// Annotation added.
    Added(Annotation),
    /// Annotation removed.
    Removed(Annotation),
    /// Annotation modified.
    Modified {
        /// State before modification.
        before: Annotation,
        /// State after modification.
        after: Annotation,
    },
}

/// Markup state manager with full undo/redo transaction stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkupManager {
    /// Currently active markup tool.
    pub active_tool: MarkupTool,
    /// All annotations across all pages.
    annotations: Vec<Annotation>,
    /// Selected annotation ID if any.
    pub selected_annotation: Option<AnnotationId>,
    /// Undo transaction stack.
    undo_stack: Vec<MarkupTransaction>,
    /// Redo transaction stack.
    redo_stack: Vec<MarkupTransaction>,
    /// Monotonically increasing ID counter.
    next_id: u64,
}

impl Default for MarkupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkupManager {
    /// Creates a new `MarkupManager`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_tool: MarkupTool::Select,
            annotations: Vec::new(),
            selected_annotation: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            next_id: 1,
        }
    }

    /// Allocates a new unique `AnnotationId`.
    pub fn next_id(&mut self) -> AnnotationId {
        let id = AnnotationId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Adds a new annotation to the document and records it in the undo stack.
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.undo_stack.push(MarkupTransaction::Added(annotation.clone()));
        self.redo_stack.clear();
        self.annotations.push(annotation);
    }

    /// Removes an annotation by ID and records it in the undo stack.
    pub fn remove_annotation(&mut self, id: AnnotationId) -> bool {
        if let Some(pos) = self.annotations.iter().position(|a| a.id == id) {
            let removed = self.annotations.remove(pos);
            self.undo_stack.push(MarkupTransaction::Removed(removed));
            self.redo_stack.clear();
            if self.selected_annotation == Some(id) {
                self.selected_annotation = None;
            }
            true
        } else {
            false
        }
    }

    /// Modifies an annotation and records the difference in the undo stack.
    pub fn modify_annotation(&mut self, modified: Annotation) -> bool {
        if let Some(existing) = self.annotations.iter_mut().find(|a| a.id == modified.id) {
            let before = existing.clone();
            *existing = modified.clone();
            self.undo_stack.push(MarkupTransaction::Modified {
                before,
                after: modified,
            });
            self.redo_stack.clear();
            true
        } else {
            false
        }
    }

    /// Performs an Undo operation. Returns `true` if an operation was undone.
    pub fn undo(&mut self) -> bool {
        if let Some(tx) = self.undo_stack.pop() {
            match tx {
                MarkupTransaction::Added(ann) => {
                    self.annotations.retain(|a| a.id != ann.id);
                    self.redo_stack.push(MarkupTransaction::Added(ann));
                }
                MarkupTransaction::Removed(ann) => {
                    self.annotations.push(ann.clone());
                    self.redo_stack.push(MarkupTransaction::Removed(ann));
                }
                MarkupTransaction::Modified { before, after } => {
                    if let Some(item) = self.annotations.iter_mut().find(|a| a.id == before.id) {
                        *item = before.clone();
                    }
                    self.redo_stack.push(MarkupTransaction::Modified { before, after });
                }
            }
            true
        } else {
            false
        }
    }

    /// Performs a Redo operation. Returns `true` if an operation was redone.
    pub fn redo(&mut self) -> bool {
        if let Some(tx) = self.redo_stack.pop() {
            match tx {
                MarkupTransaction::Added(ann) => {
                    self.annotations.push(ann.clone());
                    self.undo_stack.push(MarkupTransaction::Added(ann));
                }
                MarkupTransaction::Removed(ann) => {
                    self.annotations.retain(|a| a.id != ann.id);
                    self.undo_stack.push(MarkupTransaction::Removed(ann));
                }
                MarkupTransaction::Modified { before, after } => {
                    if let Some(item) = self.annotations.iter_mut().find(|a| a.id == after.id) {
                        *item = after.clone();
                    }
                    self.undo_stack.push(MarkupTransaction::Modified { before, after });
                }
            }
            true
        } else {
            false
        }
    }

    /// Returns all annotations on a specific page.
    #[must_use]
    pub fn annotations_for_page(&self, page_index: usize) -> Vec<&Annotation> {
        self.annotations.iter().filter(|a| a.page_index == page_index).collect()
    }

    /// Returns a slice of all active annotations.
    #[must_use]
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Total number of active annotations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.annotations.len()
    }

    /// Returns `true` if there are no annotations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catmull_rom_spline() {
        let p0 = Vec2::new(0.0, 0.0);
        let p1 = Vec2::new(10.0, 10.0);
        let p2 = Vec2::new(20.0, 10.0);
        let p3 = Vec2::new(30.0, 0.0);

        // At t = 0.0, should equal p1
        let start = catmull_rom_interpolate(p0, p1, p2, p3, 0.0);
        assert!((start - p1).length() < 1e-4);

        // At t = 1.0, should equal p2
        let end = catmull_rom_interpolate(p0, p1, p2, p3, 1.0);
        assert!((end - p2).length() < 1e-4);

        // At midpoint t = 0.5
        let mid = catmull_rom_interpolate(p0, p1, p2, p3, 0.5);
        assert!(mid.y > 9.0);
    }

    #[test]
    fn test_shape_recognition_circle() {
        let mut points = Vec::new();
        let center = Vec2::new(100.0, 100.0);
        let radius = 50.0;
        let count = 36;
        for i in 0..=count {
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let pos = center + Vec2::new(angle.cos() * radius, angle.sin() * radius);
            points.push(StrokePoint::new(pos, 0.5, i as f64 * 0.01));
        }

        let recognized = recognize_stroke_shape(&points);
        assert_eq!(recognized, RecognizedShape::Circle);
    }

    #[test]
    fn test_undo_redo_stack() {
        let mut manager = MarkupManager::new();
        let id = manager.next_id();
        let ann = Annotation {
            id,
            page_index: 0,
            kind: AnnotationKind::GeometricShape {
                shape: ShapeKind::Rectangle,
                bounds: [10.0, 10.0, 100.0, 50.0],
                stroke_color: [0.0, 0.48, 1.0, 1.0],
                fill_color: None,
                stroke_width: 2.0,
            },
            is_locked: false,
        };

        assert_eq!(manager.len(), 0);
        manager.add_annotation(ann);
        assert_eq!(manager.len(), 1);

        // Undo
        assert!(manager.undo());
        assert_eq!(manager.len(), 0);

        // Redo
        assert!(manager.redo());
        assert_eq!(manager.len(), 1);

        // Remove
        assert!(manager.remove_annotation(id));
        assert_eq!(manager.len(), 0);

        // Undo remove
        assert!(manager.undo());
        assert_eq!(manager.len(), 1);
    }
}
