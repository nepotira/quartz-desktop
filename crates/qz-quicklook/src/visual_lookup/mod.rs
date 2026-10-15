//! # Visual Look Up & Subject Lift Subsystem
//!
//! Provides the Tahoe 26 Visual Look Up and Subject Lift features:
//! - **Subject Lift**: Foreground object segmentation with glowing golden rim outline,
//!   and instant "Copy Subject" transparent PNG extraction.
//! - **Visual Look Up**: Machine intelligence entity classification (Pets, Plants,
//!   Landmarks, Food, Art, Recognized Text) with interactive information cards.

/// Semantic entity classification recognized in the image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    /// Domestic pet or animal (dog breed, cat breed, bird).
    Pet,
    /// Botanical plant or flower species.
    Plant,
    /// Geographic monument or architectural landmark.
    Landmark,
    /// Famous painting, sculpture, or artwork.
    Artwork,
    /// Culinary dish or recipe.
    Food,
    /// Structured text (phone number, tracking code, URL, date).
    RecognizedText,
}

impl EntityKind {
    /// Display title for entity kind.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Pet => "Pet",
            Self::Plant => "Plant",
            Self::Landmark => "Landmark",
            Self::Artwork => "Artwork",
            Self::Food => "Food",
            Self::RecognizedText => "Text",
        }
    }
}

/// An informative card revealed when tapping the Visual Look Up badge.
#[derive(Debug, Clone, PartialEq)]
pub struct VisualLookupCard {
    /// Recognized entity category.
    pub kind: EntityKind,
    /// Primary title (e.g. "Golden Retriever" or "Eiffel Tower").
    pub title: String,
    /// Scientific name or secondary location.
    pub subtitle: String,
    /// Descriptive snippet.
    pub description: String,
    /// Recognition confidence $[0.0, 1.0]$.
    pub confidence: f32,
}

/// A segmented foreground subject ready for Subject Lift.
#[derive(Debug, Clone, PartialEq)]
pub struct LiftedSubject {
    /// Normalized bounding box $[x, y, w, h]$ in $[0.0, 1.0]$.
    pub normalized_bounds: [f32; 4],
    /// Perimeter contour polygon points in normalized coordinates.
    pub contour_points: Vec<[f32; 2]>,
    /// Associated Visual Look Up metadata if classified.
    pub lookup_card: Option<VisualLookupCard>,
}

/// Subject Lift controller managing the lift state, rim shimmer, and clipboard extraction.
pub struct SubjectLiftController {
    is_subject_lifted: bool,
    active_subject: Option<LiftedSubject>,
    shimmer_phase: f32,
    is_card_open: bool,
}

impl Default for SubjectLiftController {
    fn default() -> Self {
        Self::new()
    }
}

impl SubjectLiftController {
    /// Creates a new Subject Lift controller.
    pub fn new() -> Self {
        Self {
            is_subject_lifted: false,
            active_subject: None,
            shimmer_phase: 0.0,
            is_card_open: false,
        }
    }

    /// Sets the recognized subject in the currently displayed image.
    pub fn set_recognized_subject(&mut self, subject: Option<LiftedSubject>) {
        self.active_subject = subject;
        self.is_subject_lifted = false;
        self.shimmer_phase = 0.0;
        self.is_card_open = false;
    }

    /// Toggles the lifted state of the foreground subject (e.g. on long press or click).
    pub fn toggle_lift(&mut self) -> bool {
        if self.active_subject.is_some() {
            self.is_subject_lifted = !self.is_subject_lifted;
            self.shimmer_phase = 0.0;
            self.is_subject_lifted
        } else {
            false
        }
    }

    /// Toggles the Visual Look Up information card popover.
    pub fn toggle_lookup_card(&mut self) -> bool {
        if self.has_lookup_info() {
            self.is_card_open = !self.is_card_open;
            self.is_card_open
        } else {
            false
        }
    }

    /// Advances the golden rim shimmer animation clock by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if self.is_subject_lifted {
            self.shimmer_phase = (self.shimmer_phase + dt * 2.5) % std::f32::consts::TAU;
        }
    }

    /// Simulates generating an isolated transparent PNG of the lifted subject for clipboard copy.
    pub fn copy_subject_to_clipboard(&self) -> Option<Vec<u8>> {
        if self.is_subject_lifted && self.active_subject.is_some() {
            // Returns synthesized PNG header payload for the lifted subject
            let mock_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            Some(mock_png)
        } else {
            None
        }
    }

    /// Whether a subject is currently lifted from background.
    #[inline]
    pub fn is_subject_lifted(&self) -> bool {
        self.is_subject_lifted
    }

    /// Active recognized subject if any.
    pub fn active_subject(&self) -> Option<&LiftedSubject> {
        self.active_subject.as_ref()
    }

    /// Current shimmer phase in radians.
    #[inline]
    pub fn shimmer_phase(&self) -> f32 {
        self.shimmer_phase
    }

    /// Whether entity info card popover is currently open.
    #[inline]
    pub fn is_card_open(&self) -> bool {
        self.is_card_open
    }

    /// Whether the active subject has Visual Look Up info available.
    #[inline]
    pub fn has_lookup_info(&self) -> bool {
        self.active_subject
            .as_ref()
            .and_then(|s| s.lookup_card.as_ref())
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subject_lift_and_visual_lookup() {
        let mut controller = SubjectLiftController::new();
        assert!(!controller.is_subject_lifted());

        let subject = LiftedSubject {
            normalized_bounds: [0.2, 0.2, 0.6, 0.6],
            contour_points: vec![[0.2, 0.2], [0.8, 0.2], [0.8, 0.8], [0.2, 0.8]],
            lookup_card: Some(VisualLookupCard {
                kind: EntityKind::Pet,
                title: "Golden Retriever".into(),
                subtitle: "Canis lupus familiaris".into(),
                description: "Gentle and affectionate sporting dog breed.".into(),
                confidence: 0.98,
            }),
        };

        controller.set_recognized_subject(Some(subject));
        assert!(controller.has_lookup_info());

        // Lift subject
        assert!(controller.toggle_lift());
        assert!(controller.is_subject_lifted());

        // Update shimmer
        controller.update(0.1);
        assert!(controller.shimmer_phase() > 0.0);

        // Copy subject produces PNG payload
        let png = controller.copy_subject_to_clipboard();
        assert!(png.is_some());
        assert_eq!(&png.unwrap()[0..4], &[0x89, 0x50, 0x4E, 0x47]);

        // Toggle lookup card
        assert!(controller.toggle_lookup_card());
        assert!(controller.is_card_open());
    }
}
