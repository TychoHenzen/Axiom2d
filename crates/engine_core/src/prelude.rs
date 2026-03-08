pub use crate::color::Color;
pub use crate::error::EngineError;
pub use crate::transform::Transform2D;
pub use crate::types::{EntityId, Pixels, Seconds, TextureId};
pub use glam::{Affine2, Mat3, Vec2};

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn when_prelude_imported_then_exports_newtypes() {
        // Act
        let _ = Pixels(1.0);
        let _ = Seconds(0.5);
        let _ = TextureId(1);
        let _ = EntityId(1);
    }

    #[test]
    fn when_prelude_imported_then_exports_color() {
        // Act
        let _ = Color::new(1.0, 0.0, 0.0, 1.0);
        let _ = Color::WHITE;
        let _ = Color::RED;
    }

    #[test]
    fn when_prelude_imported_then_exports_error() {
        // Act
        let _ = EngineError::NotFound("x".into());
    }

    #[test]
    fn when_prelude_imported_then_exports_transform() {
        // Act
        let _ = Transform2D::default();
    }

    #[test]
    fn when_prelude_imported_then_exports_glam_types() {
        // Act
        let _ = Vec2::new(1.0, 2.0);
        let _ = Mat3::IDENTITY;
        let _ = Affine2::IDENTITY;
    }
}
