//! # qz-icons
//!
//! 100% Procedural Mathematical SDF, Cubic Bézier & Multi-Stop Gradient Icon Engine
//! for QuartzLinux (macOS Tahoe 26 Architecture).
//!
//! ## Zero-Tolerance Anti-Fraud Law #1
//! - **PROHIBITED** to use font glyphs, private Unicode (`\u{F8FF}`), or emojis for icons.
//! - Every icon in the system (Apple Logo, S-Curve Blue Folder with Embossed Glyphs,
//!   all 7 3D Dock Icons, White Battery `22x11px`, Wi-Fi, Bluetooth, Search Loupe,
//!   Control Center, AirDrop, iCloud, and Sidebar items) is evaluated via exact analytical
//!   Signed Distance Fields (SDFs) on both GPU (GLSL 450) and CPU (`glam::Vec2`/`Vec4`).

#![deny(missing_docs)]
#![deny(unsafe_code)]

use glam::{Mat2, Vec2, Vec3, Vec4};

/// Embedded GLSL 450 procedural icon shader library (`icons_tahoe.glsl`).
pub const ICONS_TAHOE_GLSL: &str = include_str!("../shaders/icons_tahoe.glsl");

/// Mathematical SDF primitives and GLSL-equivalent helper functions.
pub mod primitives {
    use glam::Vec2;

    /// Smooth minimum (`smin`) polynomial union between two SDF distances $a$ and $b$ with blend radius $k$.
    ///
    /// $$h = \text{clamp}\left(0.5 + 0.5 \frac{b - a}{k}, 0, 1\right), \quad \text{smin}(a, b, k) = \text{mix}(b, a, h) - k \cdot h (1 - h)$$
    #[inline]
    pub fn smin(a: f32, b: f32, k: f32) -> f32 {
        let h = (0.5 + 0.5 * (b - a) / k.max(1e-6)).clamp(0.0, 1.0);
        b * (1.0 - h) + a * h - k * h * (1.0 - h)
    }

    /// Hermite smooth interpolation (`smoothstep`) between `edge0` and `edge1`.
    #[inline]
    pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
        let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Signed Distance Field of a capsule/line segment from `a` to `b` with radius `r`.
    #[inline]
    pub fn sd_segment(p: Vec2, a: Vec2, b: Vec2, r: f32) -> f32 {
        let pa = p - a;
        let ba = b - a;
        let denom = ba.dot(ba).max(1e-8);
        let h = (pa.dot(ba) / denom).clamp(0.0, 1.0);
        (pa - ba * h).length() - r
    }

    /// Signed Distance Field of a $G^2$ continuous Superellipse / Squircle with norm $p = 2.35$.
    ///
    /// $$\text{lp} = \left(m_x^{2.35} + m_y^{2.35}\right)^{1 / 2.35}$$
    #[inline]
    pub fn sd_squircle_g2(p: Vec2, half_size: Vec2, r: f32) -> f32 {
        let q = p.abs() - (half_size - Vec2::splat(r));
        let m = q.max(Vec2::ZERO);
        let lp = (m.x.powf(2.35) + m.y.powf(2.35)).powf(1.0 / 2.35);
        lp + q.x.max(q.y).min(0.0) - r
    }

    /// Signed Distance Field of a rounded box with half-extents `b` and corner radius `r`.
    #[inline]
    pub fn sd_round_box(p: Vec2, b: Vec2, r: f32) -> f32 {
        let q = p.abs() - b + Vec2::splat(r);
        q.x.max(q.y).min(0.0) + q.max(Vec2::ZERO).length() - r
    }

    /// Signed Distance Field of a circle centered at origin with radius `r`.
    #[inline]
    pub fn sd_circle(p: Vec2, r: f32) -> f32 {
        p.length() - r
    }
}

use primitives::{sd_circle, sd_round_box, sd_segment, sd_squircle_g2, smoothstep, smin};

/// Evaluates the official macOS Tahoe 26 Apple Logo SDF at normalized coordinate `uv` $\in [0, 1]^2$.
///
/// Implements Pillar 5.1:
/// - Smooth boolean union (`smin`) of upper and lower cardioid lobes.
/// - Top and bottom central dips (`top_dip`, `bot_dip`).
/// - Right lateral circular bite subtraction (`bite` at `(0.58, 0.05)` with radius `0.26`).
/// - Upper leaf tilted at $42^\circ$ (`0.733 rad`) formed by the intersection of two circles.
pub fn draw_apple_logo_sdf(uv: Vec2) -> f32 {
    let mut p = (uv - Vec2::new(0.5, 0.54)) * 2.25;
    p.y = -p.y; // +Y upwards
    let q = Vec2::new(p.x.abs(), p.y);

    let upper_lobe = ((q - Vec2::new(0.23, 0.04)) * Vec2::new(1.0, 0.92)).length() - 0.39;
    let lower_lobe = ((q - Vec2::new(0.18, -0.22)) * Vec2::new(1.05, 0.85)).length() - 0.31;
    let mut body = smin(upper_lobe, lower_lobe, 0.14);

    let top_dip = (p - Vec2::new(0.0, 0.38)).length() - 0.15;
    let bot_dip = (p - Vec2::new(0.0, -0.54)).length() - 0.14;
    body = body.max(-top_dip);
    body = body.max(-bot_dip);

    let bite = (p - Vec2::new(0.58, 0.05)).length() - 0.26;
    body = body.max(-bite);

    let lp_raw = p - Vec2::new(0.08, 0.54);
    let c = 0.733_f32.cos();
    let s = 0.733_f32.sin();
    let rot = Mat2::from_cols(Vec2::new(c, s), Vec2::new(-s, c));
    let lp = rot * lp_raw;
    let leaf = ((lp - Vec2::new(0.0, -0.18)).length() - 0.29)
        .max((lp - Vec2::new(0.0, 0.18)).length() - 0.29);

    body.min(leaf)
}

/// Evaluates the RGBA color of the official Apple Logo at `uv` $\in [0, 1]^2$.
pub fn draw_apple_logo_rgba(uv: Vec2, is_dark_mode: bool) -> Vec4 {
    let d = draw_apple_logo_sdf(uv);
    let alpha = 1.0 - smoothstep(-0.018, 0.018, d);
    let rgb = if is_dark_mode {
        Vec3::splat(1.0)
    } else {
        Vec3::splat(0.08)
    };
    Vec4::from((rgb, alpha))
}

/// Embossed central insignia identifier for macOS Tahoe 26 Blue Folders (Pillar 5.2).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderGlyphId {
    /// Plain standard folder without central embossed glyph (`0`).
    Normal = 0,
    /// Applications folder with stylized 'A' glyph (`1`).
    Applications = 1,
    /// Documents folder with lined page glyph (`2`).
    Documents = 2,
    /// Downloads folder with arrow-in-circle glyph (`3`).
    Downloads = 3,
    /// Desktop folder with desktop window and dock bar glyph (`4`).
    Desktop = 4,
    /// Developer folder with `< / >` code glyph (`5`).
    Developer = 5,
}

/// Evaluates the official macOS Tahoe 26 Blue Folder RGBA at `uv` $\in [0, 1]^2$ (Pillar 5.2).
///
/// Features:
/// - Soft drop shadow below the folder.
/// - Rear tab with continuous S-curve transition (`smoothstep(0.32, 0.46, uv.x)`) and `#2CA2F8 -> #1472D4` gradient.
/// - Internal junction ambient occlusion at `y = 0.27`.
/// - Front cover (`y = 0.27..0.88`, `R = 0.085`) with 3-stop gradient (`#78DAFF -> #36A8FF -> #0B7CFF`).
/// - `1.0px` white/cyan specular crest at `y = 0.275` and dual subtle bottom grooves at `y = 0.82` and `y = 0.85`.
/// - Embossed central glyph (`FolderGlyphId`).
pub fn draw_macos_folder_tahoe(uv: Vec2, folder_glyph: FolderGlyphId) -> Vec4 {
    let p = uv;
    let px = 1.0 / 84.0;

    // Soft folder shadow
    let sq = (p - Vec2::new(0.50, 0.60)).abs() - Vec2::new(0.42, 0.27) + Vec2::splat(0.08);
    let d_sh = sq.x.max(sq.y).min(0.0) + sq.max(Vec2::ZERO).length() - 0.08;
    let mut col = Vec4::new(0.0, 0.0, 0.0, (1.0 - smoothstep(-0.01, 0.10, d_sh)) * 0.28);

    // 1. Rear tab with continuous S-curve transition
    let s_curve = smoothstep(0.32, 0.46, p.x);
    let top_y = 0.14 + (0.22 - 0.14) * s_curve;
    let b_min = Vec2::new(0.06, top_y);
    let b_max = Vec2::new(0.94, 0.86);
    let qb = (p - (b_min + b_max) * 0.5).abs() - (b_max - b_min) * 0.5 + Vec2::splat(0.075);
    let d_back = qb.x.max(qb.y).min(0.0) + qb.max(Vec2::ZERO).length() - 0.075;

    if d_back < px {
        let a_b = 1.0 - smoothstep(-px, px, d_back);
        let t_back = ((p.y - 0.14) / 0.24).clamp(0.0, 1.0);
        let mut b_rgb = Vec3::new(0.173, 0.635, 0.973).lerp(Vec3::new(0.078, 0.447, 0.831), t_back);
        let rim = (-((p.y - top_y) / 0.010).powi(2)).exp();
        b_rgb += Vec3::splat(0.30 * rim);
        col = col.lerp(Vec4::from((b_rgb, 1.0)), a_b);
    }

    // Internal junction ambient occlusion between rear tab and front cover
    if p.y <= 0.27 && d_back <= 0.0 {
        let junction_ao = (-((p.y - 0.27) / 0.024).powi(2)).exp();
        let factor = 1.0 - junction_ao * 0.24;
        col.x *= factor;
        col.y *= factor;
        col.z *= factor;
    }

    // 2. Front cover (y = 0.27..0.88, R = 0.085)
    let f_min = Vec2::new(0.05, 0.27);
    let f_max = Vec2::new(0.95, 0.88);
    let qf = (p - (f_min + f_max) * 0.5).abs() - (f_max - f_min) * 0.5 + Vec2::splat(0.085);
    let d_front = qf.x.max(qf.y).min(0.0) + qf.max(Vec2::ZERO).length() - 0.085;

    if d_front < px {
        let a_f = 1.0 - smoothstep(-px, px, d_front);
        let ty = ((p.y - 0.27) / 0.61).clamp(0.0, 1.0);
        let c1 = Vec3::new(0.471, 0.855, 1.000); // #78DAFF
        let c2 = Vec3::new(0.212, 0.659, 1.000); // #36A8FF
        let c3 = Vec3::new(0.043, 0.486, 1.000); // #0B7CFF
        let mut f_rgb = if ty < 0.5 {
            c1.lerp(c2, ty * 2.0)
        } else {
            c2.lerp(c3, (ty - 0.5) * 2.0)
        };

        // White/cyan 1.0px specular crest at y = 0.275
        let crest = (-((p.y - 0.275) / 0.011).powi(2)).exp();
        f_rgb = f_rgb.lerp(Vec3::new(0.92, 0.98, 1.00), crest * 0.72);

        // Two subtle horizontal grooves near base (y = 0.82 and y = 0.85)
        let g1 = (-((p.y - 0.82) / 0.006).powi(2)).exp() * 0.11;
        let g2 = (-((p.y - 0.85) / 0.006).powi(2)).exp() * 0.09;
        f_rgb = (f_rgb - Vec3::splat(g1 + g2)).clamp(Vec3::ZERO, Vec3::ONE);

        // Central embossed glyph
        let glyph_id = folder_glyph as i32;
        if glyph_id > 0 {
            let gp = (p - Vec2::new(0.50, 0.575)) * 4.2;
            let d_glyph = match folder_glyph {
                FolderGlyphId::Normal => 1.0,
                FolderGlyphId::Applications => {
                    let s1 = sd_segment(gp, Vec2::new(0.0, -0.45), Vec2::new(-0.38, 0.42), 0.07);
                    let s2 = sd_segment(gp, Vec2::new(0.0, -0.45), Vec2::new(0.38, 0.42), 0.07);
                    let s3 = sd_segment(gp, Vec2::new(-0.28, 0.18), Vec2::new(0.28, 0.18), 0.07);
                    s1.min(s2.min(s3))
                }
                FolderGlyphId::Documents => {
                    let doc_box = sd_round_box(gp, Vec2::new(0.32, 0.42), 0.06).abs() - 0.04;
                    let l1 = sd_segment(gp, Vec2::new(-0.18, -0.14), Vec2::new(0.18, -0.14), 0.04);
                    let l2 = sd_segment(gp, Vec2::new(-0.18, 0.04), Vec2::new(0.18, 0.04), 0.04);
                    let l3 = sd_segment(gp, Vec2::new(-0.18, 0.22), Vec2::new(0.10, 0.22), 0.04);
                    doc_box.min(l1.min(l2.min(l3)))
                }
                FolderGlyphId::Downloads => {
                    let ring = (gp.length() - 0.48).abs() - 0.06;
                    let a1 = sd_segment(gp, Vec2::new(0.0, -0.25), Vec2::new(0.0, 0.22), 0.07);
                    let a2 = sd_segment(gp, Vec2::new(-0.18, 0.05), Vec2::new(0.0, 0.24), 0.07);
                    let a3 = sd_segment(gp, Vec2::new(0.18, 0.05), Vec2::new(0.0, 0.24), 0.07);
                    ring.min(a1.min(a2.min(a3)))
                }
                FolderGlyphId::Desktop => {
                    let desk_box = sd_round_box(gp, Vec2::new(0.44, 0.32), 0.07).abs() - 0.045;
                    let dock_bar = sd_round_box(gp - Vec2::new(0.0, 0.18), Vec2::new(0.24, 0.04), 0.02);
                    desk_box.min(dock_bar)
                }
                FolderGlyphId::Developer => {
                    let s1 = sd_segment(gp, Vec2::new(-0.22, -0.25), Vec2::new(-0.45, 0.0), 0.065);
                    let s2 = sd_segment(gp, Vec2::new(-0.45, 0.00), Vec2::new(-0.22, 0.25), 0.065);
                    let s3 = sd_segment(gp, Vec2::new(0.22, -0.25), Vec2::new(0.45, 0.0), 0.065);
                    let s4 = sd_segment(gp, Vec2::new(0.45, 0.00), Vec2::new(0.22, 0.25), 0.065);
                    let s5 = sd_segment(gp, Vec2::new(0.08, -0.32), Vec2::new(-0.08, 0.32), 0.065);
                    s1.min(s2.min(s3.min(s4.min(s5))))
                }
            };
            let gm = 1.0 - smoothstep(-0.04, 0.04, d_glyph);
            let embossed = f_rgb * Vec3::new(0.55, 0.72, 0.92);
            f_rgb = f_rgb.lerp(embossed, gm * 0.65);
        }

        col = col.lerp(Vec4::from((f_rgb, 1.0)), a_f);
    }

    col
}

/// Canonical 3D Dock Icon Identifiers (Pillar 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockIconId {
    /// Finder bicolor 3D smiling face (`draw_dock_finder`).
    Finder,
    /// Launchpad 3x3 vibrant squircle grid (`draw_dock_launchpad`).
    Launchpad,
    /// Safari 3D compass with 36 polar ticks (`draw_dock_safari`).
    Safari,
    /// Terminal metallic obsidian squircle with `>_` prompt (`draw_dock_terminal`).
    Terminal,
    /// System Settings 12-tooth 3D bevel gear (`draw_dock_settings`).
    Settings,
    /// Activity Monitor dark screen with glowing ECG waveform (`draw_dock_activity_monitor`).
    ActivityMonitor,
    /// Trash 3D anisotropic aluminum cylinder (`draw_dock_trash`).
    Trash,
}

/// Evaluates the 3D Finder Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_finder(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let nose_bend = smoothstep(-0.08, 0.08, p.y);
    let split_x = (-0.05 - 0.06 * p.y) * (1.0 - nose_bend) + (0.12 - 0.08 * (p.y - 0.08)) * nose_bend;
    let side = smoothstep(split_x - 0.02, split_x + 0.02, p.x);

    let left_col = Vec3::new(0.22, 0.72, 1.00).lerp(Vec3::new(0.06, 0.44, 0.90), uv.y);
    let right_col = Vec3::new(0.97, 0.99, 1.00).lerp(Vec3::new(0.82, 0.90, 0.97), uv.y);
    let mut rgb = left_col.lerp(right_col, side);

    let d_nose_top = sd_segment(p, Vec2::new(-0.02, -0.84), Vec2::new(-0.08, 0.04), 0.032);
    let d_nose_mid = sd_segment(p, Vec2::new(-0.08, 0.04), Vec2::new(0.12, 0.01), 0.032);
    let d_nose_bot = sd_segment(p, Vec2::new(0.09, 0.34), Vec2::new(0.05, 0.84), 0.030);
    let d_nose = d_nose_top.min(d_nose_mid).min(d_nose_bot);

    let d_eye_l = sd_segment(p, Vec2::new(-0.36, -0.28), Vec2::new(-0.36, -0.11), 0.052);
    let d_eye_r = sd_segment(p, Vec2::new(0.34, -0.28), Vec2::new(0.34, -0.11), 0.052);

    let smile_arc_y = 0.24 + 0.18 * (1.0 - p.x * p.x * 1.9);
    let mut d_smile = (p.y - smile_arc_y).abs() - 0.034 * (1.0 - 0.4 * p.x * p.x);
    d_smile = d_smile.max(p.x.abs() - 0.48);

    let d_features = d_eye_l.min(d_eye_r).min(d_nose.min(d_smile));
    rgb = rgb.lerp(Vec3::new(0.08, 0.15, 0.26), 1.0 - smoothstep(-0.015, 0.015, d_features));

    let inner_bevel = smoothstep(-0.12, 0.0, d_sq);
    rgb = rgb.lerp(rgb * 0.78, inner_bevel * uv.y);
    rgb += Vec3::splat(0.35 * (1.0 - smoothstep(0.0, 0.04, (d_sq + 0.02).abs())) * (1.0 - uv.y));
    Vec4::from((rgb.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D Launchpad Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_launchpad(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let mut bg = Vec3::new(0.24, 0.26, 0.32).lerp(Vec3::new(0.12, 0.14, 0.18), uv.y);
    for ix in -1..=1 {
        for iy in -1..=1 {
            let center = Vec2::new(ix as f32 * 0.34, iy as f32 * 0.34);
            let d_item = sd_round_box(p - center, Vec2::splat(0.11), 0.04);
            let item_mask = 1.0 - smoothstep(-0.015, 0.015, d_item);
            let mut col = Vec3::new(0.95, 0.35, 0.25)
                .lerp(Vec3::new(0.20, 0.70, 0.95), (ix + 1) as f32 / 2.0);
            col = col.lerp(Vec3::new(0.35, 0.85, 0.40), (iy + 1) as f32 / 2.0);
            bg = bg.lerp(col, item_mask);
        }
    }

    let rim = 1.0 - smoothstep(0.0, 0.04, (d_sq + 0.02).abs());
    bg += Vec3::splat(0.3 * rim * (1.0 - uv.y));
    Vec4::from((bg.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D Safari Compass Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_safari(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let bg = Vec3::new(0.96, 0.97, 0.99);
    let r = p.length();
    let d_dial = r - 0.72;
    let mut dial_col = Vec3::new(0.16, 0.74, 0.99).lerp(Vec3::new(0.04, 0.42, 0.92), uv.y);

    let angle = p.y.atan2(p.x);
    let ticks = (angle * 18.0).sin().abs();
    let in_ring = if r >= 0.58 && r <= 0.67 { 1.0 } else { 0.0 };
    let tick_ring = in_ring * (1.0 - smoothstep(0.05, 0.18, ticks));
    dial_col = dial_col.lerp(Vec3::new(0.95, 0.98, 1.0), tick_ring * 0.85);

    let c = std::f32::consts::FRAC_1_SQRT_2;
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let rot = Mat2::from_cols(Vec2::new(c, s), Vec2::new(-s, c));
    let np = rot * p;
    let needle_shape = np.y.abs() + np.x.abs() * 0.16 - 0.095;
    let needle_shadow = (np.y - 0.03).abs() + np.x.abs() * 0.16 - 0.095;

    dial_col *= 1.0 - (1.0 - smoothstep(-0.02, 0.04, needle_shadow)) * 0.35;

    if needle_shape < 0.02 {
        let nm = 1.0 - smoothstep(-0.015, 0.015, needle_shape);
        let mut n_col = if np.x > 0.0 {
            Vec3::new(1.00, 0.23, 0.19)
        } else {
            Vec3::new(0.98, 0.98, 1.00)
        };
        if np.y > 0.0 {
            n_col *= 0.84;
        }
        dial_col = dial_col.lerp(n_col, nm);
    }

    let rgb = bg.lerp(dial_col, 1.0 - smoothstep(-0.015, 0.015, d_dial));
    Vec4::from((rgb.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D Terminal Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_terminal(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let mut bg = Vec3::new(0.14, 0.15, 0.18).lerp(Vec3::new(0.06, 0.07, 0.09), uv.y);
    let p1 = sd_segment(p, Vec2::new(-0.48, -0.18), Vec2::new(-0.24, 0.0), 0.038);
    let p2 = sd_segment(p, Vec2::new(-0.24, 0.00), Vec2::new(-0.48, 0.18), 0.038);
    let d_chevron = p1.min(p2);
    let d_cursor = sd_segment(p, Vec2::new(-0.12, 0.18), Vec2::new(0.18, 0.18), 0.038);
    let d_prompt = d_chevron.min(d_cursor);

    let prompt_mask = 1.0 - smoothstep(-0.015, 0.015, d_prompt);
    bg = bg.lerp(Vec3::new(0.25, 0.92, 0.45), prompt_mask);

    let rim = 1.0 - smoothstep(0.0, 0.04, (d_sq + 0.02).abs());
    bg += Vec3::splat(0.3 * rim * (1.0 - uv.y));
    Vec4::from((bg.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D System Settings Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_settings(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let mut bg = Vec3::new(0.55, 0.57, 0.62).lerp(Vec3::new(0.32, 0.34, 0.38), uv.y);
    let r = p.length();
    let a = p.y.atan2(p.x);

    let outer_gear = r - (0.54 + 0.065 * smoothstep(-0.4, 0.4, (a * 12.0).cos()));
    let inner_ring = (r - 0.34).abs() - 0.065;
    let mut spokes = p
        .x
        .abs()
        .min((p.x * 0.5 + p.y * 0.866).abs().min((p.x * 0.5 - p.y * 0.866).abs()))
        - 0.055;
    spokes = spokes.max(r - 0.34);
    let center_hub = (r - 0.12).abs() - 0.045;

    let mut d_gear = outer_gear.max(-(r - 0.42));
    d_gear = d_gear.min(inner_ring.min(spokes.min(center_hub)));

    bg *= 1.0 - (1.0 - smoothstep(-0.01, 0.06, d_gear)) * 0.40;
    let gear_col = Vec3::new(0.74, 0.76, 0.80)
        .lerp(Vec3::new(0.95, 0.96, 0.98), 0.5 + 0.5 * (a * 2.0 - 1.2).cos());
    let rgb = bg.lerp(gear_col, 1.0 - smoothstep(-0.015, 0.015, d_gear));
    Vec4::from((rgb.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D Activity Monitor Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_activity_monitor(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let d_sq = sd_squircle_g2(p, Vec2::splat(0.84), 0.36);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_sq);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let mut bg = Vec3::new(0.11, 0.13, 0.17).lerp(Vec3::new(0.05, 0.06, 0.09), uv.y);

    // ECG pulse waveform segments
    let s1 = sd_segment(p, Vec2::new(-0.68, 0.10), Vec2::new(-0.28, 0.10), 0.036);
    let s2 = sd_segment(p, Vec2::new(-0.28, 0.10), Vec2::new(-0.10, -0.46), 0.036);
    let s3 = sd_segment(p, Vec2::new(-0.10, -0.46), Vec2::new(0.08, 0.48), 0.036);
    let s4 = sd_segment(p, Vec2::new(0.08, 0.48), Vec2::new(0.24, -0.18), 0.036);
    let s5 = sd_segment(p, Vec2::new(0.24, -0.18), Vec2::new(0.38, 0.10), 0.036);
    let s6 = sd_segment(p, Vec2::new(0.38, 0.10), Vec2::new(0.68, 0.10), 0.036);
    let d_ecg = s1.min(s2.min(s3.min(s4.min(s5.min(s6)))));

    let glow = (-((d_ecg.max(0.0)) / 0.12).powi(2)).exp() * 0.35;
    bg += Vec3::new(0.10, 0.85, 0.45) * glow;
    let line_mask = 1.0 - smoothstep(-0.015, 0.015, d_ecg);
    bg = bg.lerp(Vec3::new(0.22, 0.96, 0.52), line_mask);

    let rim = 1.0 - smoothstep(0.0, 0.04, (d_sq + 0.02).abs());
    bg += Vec3::splat(0.28 * rim * (1.0 - uv.y));
    Vec4::from((bg.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates the 3D Anodized Aluminum Trash Dock Icon at `uv` $\in [0, 1]^2$.
pub fn draw_dock_trash(uv: Vec2) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let top_w = 0.56;
    let bot_w = 0.42;
    let y_norm = ((p.y - (-0.52)) / 1.18).clamp(0.0, 1.0);
    let half_w = top_w + (bot_w - top_w) * y_norm;

    let d_side = p.x.abs() - half_w;
    let bottom_curve = 0.08 * (1.0 - (p.x / bot_w).clamp(-1.0, 1.0).powi(2));
    let d_vert = (-p.y - 0.52).max(p.y - (0.64 + bottom_curve));
    let d_body = d_side.max(d_vert);

    let rp = (p - Vec2::new(0.0, -0.52)) / Vec2::new(top_w + 0.03, 0.11);
    let d_rim = rp.length() - 1.0;

    let d_all = d_body.min(d_rim);
    let mask = 1.0 - smoothstep(-0.02, 0.02, d_all);
    if mask <= 0.001 {
        return Vec4::ZERO;
    }

    let nx = (p.x / half_w.max(0.01)).clamp(-1.0, 1.0);
    let specular = (-((nx + 0.28) * 2.6).powi(2)).exp() * 0.35;
    let mut metal = Vec3::new(0.58, 0.60, 0.65)
        .lerp(Vec3::new(0.86, 0.88, 0.92), 0.5 - 0.4 * (nx - 0.1).abs());
    metal += Vec3::splat(specular);
    metal *= 1.0 + (0.82 - 1.0) * y_norm;

    let grooves = (nx * 22.0).sin();
    let in_groove = if y_norm >= 0.15 && y_norm <= 0.88 && nx.abs() <= 0.82 {
        1.0
    } else {
        0.0
    };
    metal *= 1.0 - 0.12 * smoothstep(0.4, 0.9, grooves) * in_groove;

    let ip = (p - Vec2::new(0.0, -0.52)) / Vec2::new(top_w - 0.04, 0.075);
    let d_hole = ip.length() - 1.0;
    if d_hole < 0.02 {
        let inner_col = Vec3::new(0.18, 0.20, 0.23)
            .lerp(Vec3::new(0.38, 0.40, 0.45), ((p.y + 0.60) * 6.0).clamp(0.0, 1.0));
        metal = metal.lerp(inner_col, 1.0 - smoothstep(-0.02, 0.02, d_hole));
    }

    Vec4::from((metal.clamp(Vec3::ZERO, Vec3::ONE), mask))
}

/// Evaluates any of the 7 canonical 3D Dock Icons at `uv` $\in [0, 1]^2$.
pub fn draw_dock_icon(icon: DockIconId, uv: Vec2) -> Vec4 {
    match icon {
        DockIconId::Finder => draw_dock_finder(uv),
        DockIconId::Launchpad => draw_dock_launchpad(uv),
        DockIconId::Safari => draw_dock_safari(uv),
        DockIconId::Terminal => draw_dock_terminal(uv),
        DockIconId::Settings => draw_dock_settings(uv),
        DockIconId::ActivityMonitor => draw_dock_activity_monitor(uv),
        DockIconId::Trash => draw_dock_trash(uv),
    }
}

/// Evaluates the pure-white macOS Tahoe 26 Battery status icon (`22x11px` aspect ratio, zero neon green).
pub fn draw_battery_tahoe_rgba(uv: Vec2, charge_level: f32) -> Vec4 {
    let p = uv * 2.0 - Vec2::ONE;
    let body = sd_round_box(p - Vec2::new(-0.08, 0.0), Vec2::new(0.68, 0.36), 0.12).abs() - 0.045;
    let nub = sd_round_box(p - Vec2::new(0.70, 0.0), Vec2::new(0.06, 0.16), 0.04);
    let w = 0.56 * charge_level.clamp(0.05, 1.0);
    let inner = sd_round_box(p - Vec2::new(-0.08 - (0.56 - w), 0.0), Vec2::new(w, 0.24), 0.06);
    let fill_mask = 1.0 - smoothstep(-0.02, 0.02, inner);
    let border_d = body.min(nub);
    let border_a = 1.0 - smoothstep(-0.02, 0.02, border_d);
    Vec4::new(1.0, 1.0, 1.0, (border_a * 0.45).max(fill_mask))
}

/// Evaluates the macOS Wi-Fi status icon SDF at `p` $\in [-1, 1]^2$.
pub fn sd_wifi_icon(mut p: Vec2) -> f32 {
    p.y -= 0.35;
    let mut d = sd_circle(p, 0.12);
    for i in 1..=3 {
        let r = i as f32 * 0.28;
        let ring = (p.length() - r).abs() - 0.055;
        let cone = (p.x.abs() * 1.15 + p.y).max(-p.y - 0.05);
        d = d.min(ring.max(cone));
    }
    d
}

/// Evaluates the macOS Search Loupe SDF at `p` $\in [-1, 1]^2$.
pub fn sd_search_loupe(p: Vec2) -> f32 {
    let ring = (sd_circle(p - Vec2::new(-0.12, -0.12), 0.42)).abs() - 0.075;
    let handle = sd_segment(p, Vec2::new(0.18, 0.18), Vec2::new(0.65, 0.65), 0.085);
    ring.min(handle)
}

/// Evaluates the macOS Control Center dual-pill toggle SDF at `p` $\in [-1, 1]^2$.
pub fn sd_control_center_icon(p: Vec2) -> f32 {
    let top_pill = sd_round_box(p - Vec2::new(0.0, -0.28), Vec2::new(0.58, 0.18), 0.18).abs() - 0.03;
    let top_knob = sd_circle(p - Vec2::new(-0.24, -0.28), 0.14);
    let bot_pill = sd_round_box(p - Vec2::new(0.0, 0.28), Vec2::new(0.58, 0.18), 0.18).abs() - 0.03;
    let bot_knob = sd_circle(p - Vec2::new(0.24, 0.28), 0.14);
    top_pill.min(top_knob).min(bot_pill.min(bot_knob))
}

/// Evaluates the macOS iCloud smooth boolean union SDF at `p` $\in [-1, 1]^2$.
pub fn sd_icloud_icon(p: Vec2) -> f32 {
    let c1 = sd_circle(p - Vec2::new(-0.32, 0.12), 0.26);
    let c2 = sd_circle(p - Vec2::new(-0.06, -0.12), 0.34);
    let c3 = sd_circle(p - Vec2::new(0.28, -0.02), 0.28);
    let base = sd_round_box(p - Vec2::new(-0.02, 0.18), Vec2::new(0.46, 0.18), 0.12);
    smin(smin(c1, c2, 0.08), smin(c3, base, 0.08), 0.08)
}

/// Rasterizes any procedural icon shader `eval_fn(uv: Vec2) -> Vec4` into an RGBA8 byte buffer.
pub fn rasterize_icon_rgba8<F>(width: u32, height: u32, eval_fn: F) -> Vec<u8>
where
    F: Fn(Vec2) -> Vec4,
{
    let w = width.max(1);
    let h = height.max(1);
    let mut buf = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        let v = (y as f32 + 0.5) / h as f32;
        for x in 0..w {
            let u = (x as f32 + 0.5) / w as f32;
            let c = eval_fn(Vec2::new(u, v)).clamp(Vec4::ZERO, Vec4::ONE);
            buf.push((c.x * 255.0).round() as u8);
            buf.push((c.y * 255.0).round() as u8);
            buf.push((c.z * 255.0).round() as u8);
            buf.push((c.w * 255.0).round() as u8);
        }
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apple_logo_sdf_geometry() {
        // Center of Apple body should be inside (negative SDF)
        let d_center = draw_apple_logo_sdf(Vec2::new(0.45, 0.58));
        assert!(d_center < 0.0, "Expected inside Apple body, got {}", d_center);

        // Right bite region should be outside (positive SDF)
        let d_bite = draw_apple_logo_sdf(Vec2::new(0.76, 0.52));
        assert!(d_bite > 0.0, "Expected outside in right bite region, got {}", d_bite);
    }

    #[test]
    fn test_macos_folder_tahoe_all_glyphs() {
        for glyph in [
            FolderGlyphId::Normal,
            FolderGlyphId::Applications,
            FolderGlyphId::Documents,
            FolderGlyphId::Downloads,
            FolderGlyphId::Desktop,
            FolderGlyphId::Developer,
        ] {
            let center_rgba = draw_macos_folder_tahoe(Vec2::new(0.5, 0.55), glyph);
            assert!(center_rgba.w > 0.95);
            assert!(center_rgba.z > center_rgba.x, "Folder must be celestial blue");
        }
    }

    #[test]
    fn test_all_seven_dock_icons_render() {
        let icons = [
            DockIconId::Finder,
            DockIconId::Launchpad,
            DockIconId::Safari,
            DockIconId::Terminal,
            DockIconId::Settings,
            DockIconId::ActivityMonitor,
            DockIconId::Trash,
        ];
        for icon in icons {
            let c = draw_dock_icon(icon, Vec2::new(0.5, 0.5));
            assert!(c.w > 0.9, "Dock icon {:?} center must be opaque", icon);
        }
    }

    #[test]
    fn test_rasterize_icon_rgba8_dimensions() {
        let buf = rasterize_icon_rgba8(32, 32, |uv| draw_apple_logo_rgba(uv, true));
        assert_eq!(buf.len(), 32 * 32 * 4);
    }
}
