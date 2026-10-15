//! # QuartzLinux Preview Application (`qz-preview`)
//!
//! Production-grade Preview desktop application for QuartzLinux (`qz-desktop`),
//! built to be visually, mathematically, physically, and functionally indistinguishable
//! from **macOS Tahoe 26 Preview (`Preview.app`)**.
//!
//! ### Core Subsystems:
//! - [`document`]: Multi-page PDF and raster image document model, continuous zoom ($0.1\times$ to $10.0\times$),
//!   actual size (`Cmd+0`), zoom to fit (`Cmd+9`), and page rotation/reordering.
//! - [`markup`]: Advanced markup toolset including pressure-sensitive sketch pen ($1.0..=16.0\text{px}$),
//!   Catmull-Rom cubic spline smoothing, auto-shape recognition (circles, rectangles, arrows),
//!   optical magnifying glass (Loupe $1.5\times$ to $5.0\times$), highlighters, text boxes, and full undo/redo transaction stack.
//! - [`signature`]: Trackpad digital signature capture with normalized $(u, v, p)$ touch coordinates,
//!   cubic Hermite interpolation, variable ink width calculation, and persistent signature vault.
//! - [`export`]: Multi-format conversion (PDF, PNG, JPEG, WebP, TIFF) with embedded ICC color profiles
//!   (Display P3, sRGB, Adobe RGB, ProPhoto RGB) and annotation flattening.
//! - [`window`]: Liquid Glass window controller, unified titlebar with dirty state bullet dot (`•`),
//!   Document Proxy Icon supporting drag-to-retarget into Finder/Mail, and collapsible sidebar/markup panels.

#![deny(missing_docs)]

pub mod document;
pub mod export;
pub mod markup;
pub mod signature;
pub mod window;

// Re-exports for convenient top-level access
pub use document::{DocumentFormat, DocumentModel, DocumentPage, ViewMode};
pub use export::{ColorProfile, DocumentExporter, ExportError, ExportOptions, ExportReport, PageRange};
pub use markup::{
    catmull_rom_interpolate, recognize_stroke_shape, Annotation, AnnotationId, AnnotationKind,
    MarkupManager, MarkupTool, MarkupTransaction, RecognizedShape, ShapeKind, StrokePoint,
};
pub use signature::{
    compute_ink_width, hermite_interpolate, SavedSignature, SignatureSample, SignatureStroke,
    SignatureVault,
};
pub use window::{PreviewWindowController, ProxyIconDragState, SidebarTab};

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_end_to_end_preview_workflow() {
        // 1. Create a 2-page PDF document
        let page1 = DocumentPage::new(0, Vec2::new(595.0, 842.0)); // A4
        let page2 = DocumentPage::new(1, Vec2::new(595.0, 842.0));
        let doc = DocumentModel::new(
            "/home/nepo/Documents/Contract.pdf",
            "Contract.pdf",
            DocumentFormat::Pdf,
            vec![page1, page2],
            Vec2::new(1280.0, 800.0),
        );

        let mut win = PreviewWindowController::new(doc, Vec2::new(1280.0, 800.0));
        assert_eq!(win.window_title(), "Contract.pdf");

        // 2. Open markup bar and add a shape annotation
        win.toggle_markup_bar();
        assert!(win.is_markup_bar_visible);

        let ann_id = win.markup.next_id();
        let rect_ann = Annotation {
            id: ann_id,
            page_index: 0,
            kind: AnnotationKind::GeometricShape {
                shape: ShapeKind::Rectangle,
                bounds: [50.0, 50.0, 200.0, 100.0],
                stroke_color: [1.0, 0.0, 0.0, 1.0],
                fill_color: Some([1.0, 1.0, 0.0, 0.2]),
                stroke_width: 2.0,
            },
            is_locked: false,
        };
        win.markup.add_annotation(rect_ann);
        assert_eq!(win.markup.len(), 1);

        // 3. Create trackpad signature and insert onto page 1
        let mut stroke = SignatureStroke::new();
        stroke.add_sample(SignatureSample::new(Vec2::new(0.2, 0.5), 0.7, 0.0));
        stroke.add_sample(SignatureSample::new(Vec2::new(0.8, 0.5), 0.9, 0.1));
        let saved_sig = SavedSignature::new("sig-01", "CEO Signature", 1700000000, vec![stroke]);
        let placement = saved_sig.placement_bounds(Vec2::new(300.0, 700.0), 150.0);
        win.signature_vault.add_signature(saved_sig);

        let sig_ann_id = win.markup.next_id();
        let sig_ann = Annotation {
            id: sig_ann_id,
            page_index: 0,
            kind: AnnotationKind::Signature {
                signature_id: "sig-01".to_string(),
                bounds: placement,
                ink_color: [0.0, 0.1, 0.4, 1.0],
            },
            is_locked: false,
        };
        win.markup.add_annotation(sig_ann);
        assert_eq!(win.markup.len(), 2);

        // 4. Export flattened document with Display P3 color profile
        let export_opts = ExportOptions {
            target_format: DocumentFormat::Png,
            color_profile: ColorProfile::DisplayP3,
            dpi: 144, // 2x Retina
            quality: 95,
            flatten_annotations: true,
            page_range: PageRange::Single(0),
        };

        let report = win.export_document(&export_opts).expect("export should succeed");
        assert_eq!(report.pages_exported, 1);
        assert_eq!(report.color_profile, ColorProfile::DisplayP3);
        assert_eq!(report.annotations_flattened, 2);
    }
}
