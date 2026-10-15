//! # Interactive Markup Annotation Subsystem
//!
//! Provides the Tahoe 26 Quick Look inline annotation tools: Pen, Highlighter,
//! Geometric Shapes (Arrow, Rect, Ellipse), Text Notes, and Trackpad Signatures with Undo/Redo.

/// Annotation tool modes available in Markup toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupToolType {
    /// Freehand drawing pen with pressure variation.
    Pen,
    /// Semi-transparent highlighter with multiply blend.
    Highlighter,
    /// Directed vector arrow.
    Arrow,
    /// Bounding rectangle shape.
    Rectangle,
    /// Ellipse / Circle shape.
    Ellipse,
    /// Editable text box note.
    TextNote,
    /// Trackpad or saved signature.
    Signature,
}

/// An annotation element placed on the document canvas.
#[derive(Debug, Clone, PartialEq)]
pub enum MarkupElement {
    /// Freehand pen stroke points.
    Stroke {
        /// Sequential $[x, y]$ coordinates along the stroke.
        points: Vec<[f32; 2]>,
        /// RGBA color.
        color: [f32; 4],
        /// Stroke width in points.
        width: f32,
        /// True if highlighter mode (semi-transparent multiply blending).
        is_highlighter: bool,
    },
    /// Directed vector arrow from start to end.
    Arrow {
        /// Origin point $[x, y]$.
        start: [f32; 2],
        /// Destination tip point $[x, y]$.
        end: [f32; 2],
        /// Arrow color.
        color: [f32; 4],
        /// Line thickness.
        stroke_width: f32,
    },
    /// Geometric rectangle shape $[x, y, w, h]$.
    Rectangle {
        /// Coordinates $[x, y, w, h]$.
        bounds: [f32; 4],
        /// Border stroke color.
        stroke_color: [f32; 4],
        /// Optional interior fill color.
        fill_color: Option<[f32; 4]>,
        /// Border thickness.
        stroke_width: f32,
    },
    /// Geometric ellipse shape $[x, y, w, h]$.
    Ellipse {
        /// Coordinates $[x, y, w, h]$.
        bounds: [f32; 4],
        /// Border stroke color.
        stroke_color: [f32; 4],
        /// Optional interior fill color.
        fill_color: Option<[f32; 4]>,
        /// Border thickness.
        stroke_width: f32,
    },
    /// Text label note.
    TextNote {
        /// UTF-8 content string.
        text: String,
        /// Top-left anchor $[x, y]$.
        pos: [f32; 2],
        /// Font size in points.
        font_size: f32,
        /// Text color.
        color: [f32; 4],
    },
}

/// Markup editing session state with localized undo/redo history.
pub struct MarkupSession {
    is_active: bool,
    active_tool: MarkupToolType,
    elements: Vec<MarkupElement>,
    undo_stack: Vec<Vec<MarkupElement>>,
    redo_stack: Vec<Vec<MarkupElement>>,
    active_color: [f32; 4],
    active_stroke_width: f32,
    rotation_degrees: u32,
}

impl Default for MarkupSession {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkupSession {
    /// Creates a new markup session with standard defaults.
    pub fn new() -> Self {
        Self {
            is_active: false,
            active_tool: MarkupToolType::Pen,
            elements: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            active_color: [0.0, 0.48, 1.0, 1.0], // System Blue
            active_stroke_width: 3.0,
            rotation_degrees: 0,
        }
    }

    /// Whether Markup toolbar is currently active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Toggles markup mode.
    pub fn toggle_active(&mut self) -> bool {
        self.is_active = !self.is_active;
        self.is_active
    }

    /// Sets the active drawing tool.
    pub fn set_tool(&mut self, tool: MarkupToolType) {
        self.active_tool = tool;
    }

    /// Active drawing tool.
    #[inline]
    pub fn active_tool(&self) -> MarkupToolType {
        self.active_tool
    }

    /// Rotates document clockwise by 90 degrees (`Cmd+R`).
    pub fn rotate_clockwise(&mut self) -> u32 {
        self.rotation_degrees = (self.rotation_degrees + 90) % 360;
        self.rotation_degrees
    }

    /// Current document rotation in degrees (0, 90, 180, 270).
    #[inline]
    pub fn rotation_degrees(&self) -> u32 {
        self.rotation_degrees
    }

    /// Commits a new markup element, saving the previous state to undo stack.
    pub fn add_element(&mut self, element: MarkupElement) {
        self.undo_stack.push(self.elements.clone());
        self.redo_stack.clear();
        self.elements.push(element);
    }

    /// Undoes the last markup modification.
    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.elements.clone());
            self.elements = prev;
            true
        } else {
            false
        }
    }

    /// Redoes the last undone markup modification.
    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.elements.clone());
            self.elements = next;
            true
        } else {
            false
        }
    }

    /// All active markup elements on the canvas.
    pub fn elements(&self) -> &[MarkupElement] {
        &self.elements
    }

    /// Clears all markup annotations.
    pub fn clear(&mut self) {
        if !self.elements.is_empty() {
            self.undo_stack.push(self.elements.clone());
            self.elements.clear();
            self.redo_stack.clear();
        }
    }

    /// Performs Catmull-Rom smoothing on raw sampled points.
    pub fn smooth_points(raw_points: &[[f32; 2]]) -> Vec<[f32; 2]> {
        if raw_points.len() < 3 {
            return raw_points.to_vec();
        }

        let mut smoothed = Vec::with_capacity(raw_points.len() * 2);
        smoothed.push(raw_points[0]);

        for i in 0..raw_points.len() - 1 {
            let p0 = if i == 0 { raw_points[0] } else { raw_points[i - 1] };
            let p1 = raw_points[i];
            let p2 = raw_points[i + 1];
            let p3 = if i + 2 < raw_points.len() {
                raw_points[i + 2]
            } else {
                p2
            };

            // Interpolate midpoint
            let mid_x = 0.5 * (2.0 * p1[0] + (-p0[0] + p2[0]) * 0.5 + (2.0 * p0[0] - 5.0 * p1[0] + 4.0 * p2[0] - p3[0]) * 0.25);
            let mid_y = 0.5 * (2.0 * p1[1] + (-p0[1] + p2[1]) * 0.5 + (2.0 * p0[1] - 5.0 * p1[1] + 4.0 * p2[1] - p3[1]) * 0.25);

            smoothed.push([mid_x, mid_y]);
            smoothed.push(p2);
        }

        smoothed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markup_undo_redo() {
        let mut session = MarkupSession::new();
        assert_eq!(session.elements().len(), 0);

        session.add_element(MarkupElement::Arrow {
            start: [10.0, 10.0],
            end: [100.0, 100.0],
            color: [1.0, 0.0, 0.0, 1.0],
            stroke_width: 2.0,
        });

        assert_eq!(session.elements().len(), 1);

        assert!(session.undo());
        assert_eq!(session.elements().len(), 0);

        assert!(session.redo());
        assert_eq!(session.elements().len(), 1);
    }

    #[test]
    fn test_markup_rotation_degrees() {
        let mut session = MarkupSession::new();
        assert_eq!(session.rotation_degrees(), 0);

        assert_eq!(session.rotate_clockwise(), 90);
        assert_eq!(session.rotate_clockwise(), 180);
        assert_eq!(session.rotate_clockwise(), 270);
        assert_eq!(session.rotate_clockwise(), 0);
    }
}
