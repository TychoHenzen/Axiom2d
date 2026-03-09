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
    use super::*;

    #[test]
    fn when_pixels_arithmetic_then_add_sub_mul_produce_correct_results() {
        assert_eq!(Pixels(1.5) + Pixels(2.5), Pixels(4.0));
        assert_eq!(Pixels(5.0) - Pixels(2.0), Pixels(3.0));
        assert_eq!(Pixels(4.0) * 0.5, Pixels(2.0));
    }

    #[test]
    fn when_seconds_arithmetic_then_add_sub_mul_produce_correct_results() {
        assert_eq!(Seconds(0.5) + Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(1.0) - Seconds(0.25), Seconds(0.75));
        assert_eq!(Seconds(0.016) * 2.0, Seconds(0.032));
    }
}
