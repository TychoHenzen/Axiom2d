use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn total_horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub fn total_vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_zero_margin_then_totals_zero() {
        // Arrange
        let margin = Margin::default();

        // Act / Assert
        assert_eq!(margin.total_horizontal(), 0.0);
        assert_eq!(margin.total_vertical(), 0.0);
    }

    #[test]
    fn when_asymmetric_margin_then_correct_pairs() {
        // Arrange
        let margin = Margin {
            top: 5.0,
            right: 10.0,
            bottom: 15.0,
            left: 20.0,
        };

        // Act / Assert
        assert_eq!(margin.total_horizontal(), 30.0);
        assert_eq!(margin.total_vertical(), 20.0);
    }
}
