// EVOLVE-BLOCK-START
use engine_core::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OffsetY {
    /// Fractional offset relative to card center (e.g. -0.1 = 10% above center).
    Fractional(f32),
}

pub(crate) struct FaceRegion {
    pub offset_y: OffsetY,
    pub half_w_frac: f32,
    pub half_h_frac: f32,
    pub color: Color,
    pub use_art_shader: bool,
}

// Inner region width: 0.40 → half_w = 24 on a 60-wide card (6-unit border margin per side).
// Vertical: name/desc use Fractional offsets for ~4-unit top/bottom border margins.
pub(crate) const FRONT_FACE_REGIONS: [FaceRegion; 4] = [
    // Border (full card)
    FaceRegion {
        offset_y: OffsetY::Fractional(0.0),
        half_w_frac: 0.5,
        half_h_frac: 0.5,
        color: Color::WHITE,
        use_art_shader: false,
    },
    // Name strip
    FaceRegion {
        offset_y: OffsetY::Fractional(-0.36),
        half_w_frac: 0.40,
        half_h_frac: 1.0 / 12.0,
        color: Color {
            r: 0.863,
            g: 0.863,
            b: 0.863,
            a: 1.0,
        },
        use_art_shader: false,
    },
    // Art area (fits between name strip bottom and desc strip top)
    FaceRegion {
        offset_y: OffsetY::Fractional(-0.08),
        half_w_frac: 0.40,
        half_h_frac: 0.185,
        color: Color {
            r: 0.706,
            g: 0.784,
            b: 0.902,
            a: 1.0,
        },
        use_art_shader: true,
    },
    // Description strip (inset from bottom edge by ~5 units on a 90-tall card)
    FaceRegion {
        offset_y: OffsetY::Fractional(0.28),
        half_w_frac: 0.40,
        half_h_frac: 1.0 / 6.0,
        color: Color {
            r: 0.941,
            g: 0.941,
            b: 0.784,
            a: 1.0,
        },
        use_art_shader: false,
    },
];

impl FaceRegion {
    pub fn resolve(&self, card_w: f32, card_h: f32) -> (f32, f32, f32) {
        let half_w = card_w * self.half_w_frac;
        let half_h = card_h * self.half_h_frac;
        let offset_y = match self.offset_y {
            OffsetY::Fractional(frac) => card_h * frac,
        };
        (half_w, half_h, offset_y)
    }
}
// EVOLVE-BLOCK-END
