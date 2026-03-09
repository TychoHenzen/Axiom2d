use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Pixels(pub f32);

impl Add for Pixels {
    type Output = Pixels;
    fn add(self, rhs: Pixels) -> Pixels {
        Pixels(self.0 + rhs.0)
    }
}

impl Sub for Pixels {
    type Output = Pixels;
    fn sub(self, rhs: Pixels) -> Pixels {
        Pixels(self.0 - rhs.0)
    }
}

impl Mul<f32> for Pixels {
    type Output = Pixels;
    fn mul(self, rhs: f32) -> Pixels {
        Pixels(self.0 * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Seconds(pub f32);

impl Add for Seconds {
    type Output = Seconds;
    fn add(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 + rhs.0)
    }
}

impl Sub for Seconds {
    type Output = Seconds;
    fn sub(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 - rhs.0)
    }
}

impl Mul<f32> for Seconds {
    type Output = Seconds;
    fn mul(self, rhs: f32) -> Seconds {
        Seconds(self.0 * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(pub u64);

#[cfg(test)]
mod tests {
    use super::{EntityId, Pixels, Seconds, TextureId};

    #[test]
    fn when_constructing_pixels_then_supports_copy_eq_debug_and_arithmetic() {
        // Arrange
        let a = Pixels(1.0);

        // Act
        let b = a;

        // Assert
        assert_eq!(Pixels(3.5).0, 3.5);
        assert_eq!(a, b);
        assert_eq!(Pixels(2.0), Pixels(2.0));
        assert_ne!(Pixels(2.0), Pixels(3.0));
        let s = format!("{:?}", Pixels(5.0));
        assert!(s.contains('5'));
        assert_eq!(Pixels(1.5) + Pixels(2.5), Pixels(4.0));
        assert_eq!(Pixels(5.0) - Pixels(2.0), Pixels(3.0));
        assert_eq!(Pixels(4.0) * 0.5, Pixels(2.0));
    }

    #[test]
    fn when_constructing_seconds_then_supports_copy_eq_debug_and_arithmetic() {
        // Arrange
        let a = Seconds(0.5);

        // Act
        let b = a;

        // Assert
        assert_eq!(Seconds(1.0).0, 1.0);
        assert_eq!(a, b);
        assert_eq!(Seconds(1.0), Seconds(1.0));
        assert_ne!(Seconds(1.0), Seconds(2.0));
        let s = format!("{:?}", Seconds(0.016));
        assert!(s.contains("0.016"));
        assert_eq!(Seconds(0.5) + Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(1.0) - Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(0.016) * 2.0, Seconds(0.032));
    }

    #[test]
    fn when_constructing_id_types_then_supports_copy_eq_hash_and_debug() {
        // Arrange
        let tid = TextureId(1);
        let eid = EntityId(2);

        // Act
        let tid2 = tid;
        let eid2 = eid;

        // Assert
        assert_eq!(TextureId(42).0, 42);
        assert_eq!(EntityId(999).0, 999);
        assert_eq!(tid, tid2);
        assert_eq!(eid, eid2);
        assert_eq!(TextureId(5), TextureId(5));
        assert_ne!(TextureId(5), TextureId(6));
        assert_eq!(EntityId(10), EntityId(10));
        assert_ne!(EntityId(10), EntityId(11));
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(TextureId(1), "player");
        assert_eq!(map[&TextureId(1)], "player");
        let mut map2 = HashMap::new();
        map2.insert(EntityId(1), "hero");
        assert_eq!(map2[&EntityId(1)], "hero");
        let s = format!("{:?}", TextureId(7));
        assert!(s.contains('7'));
        let s = format!("{:?}", EntityId(8));
        assert!(s.contains('8'));
    }
}
