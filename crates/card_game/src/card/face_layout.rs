use engine_core::prelude::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum OffsetY {
    /// Flush to the top edge of the card.
    TopFlush,
    /// Flush to the bottom edge of the card.
    BottomFlush,
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

// half_h values as fractions of card height:
// 7.5 / 90.0 = 1/12, 22.5 / 90.0 = 1/4, 15.0 / 90.0 = 1/6
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
        offset_y: OffsetY::TopFlush,
        half_w_frac: 0.45,
        half_h_frac: 1.0 / 12.0,
        color: Color {
            r: 0.863,
            g: 0.863,
            b: 0.863,
            a: 1.0,
        },
        use_art_shader: false,
    },
    // Art area
    FaceRegion {
        offset_y: OffsetY::Fractional(-0.1),
        half_w_frac: 0.45,
        half_h_frac: 0.25,
        color: Color {
            r: 0.706,
            g: 0.784,
            b: 0.902,
            a: 1.0,
        },
        use_art_shader: true,
    },
    // Description strip
    FaceRegion {
        offset_y: OffsetY::BottomFlush,
        half_w_frac: 0.45,
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
            OffsetY::TopFlush => -(card_h * 0.5 - half_h),
            OffsetY::BottomFlush => card_h * 0.5 - half_h,
            OffsetY::Fractional(frac) => card_h * frac,
        };
        (half_w, half_h, offset_y)
    }
}
