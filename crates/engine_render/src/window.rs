#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowConfig {
    pub title: &'static str,
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Axiom2d",
            width: 1280,
            height: 720,
            vsync: true,
            resizable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_default_window_config_created_then_fields_match_expected_defaults() {
        // Act
        let cfg = WindowConfig::default();

        // Assert
        assert_eq!(cfg.title, "Axiom2d");
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
        assert_eq!(cfg.vsync, true);
        assert_eq!(cfg.resizable, true);
    }

    #[test]
    fn when_window_config_constructed_with_custom_values_then_fields_match() {
        // Act
        let cfg = WindowConfig {
            title: "My Game",
            width: 800,
            height: 600,
            vsync: false,
            resizable: false,
        };

        // Assert
        assert_eq!(cfg.title, "My Game");
        assert_eq!(cfg.width, 800);
        assert_eq!(cfg.height, 600);
        assert_eq!(cfg.vsync, false);
        assert_eq!(cfg.resizable, false);
    }

    #[test]
    fn when_using_window_config_then_supports_copy_clone_eq_and_debug() {
        // Arrange
        let a = WindowConfig {
            title: "Test",
            width: 1920,
            height: 1080,
            vsync: false,
            resizable: true,
        };

        // Act
        let b = a;

        // Assert
        assert_eq!(a, b);
        assert_eq!(a.clone(), a);
        assert_ne!(a, WindowConfig { width: 640, ..a });
        assert!(format!("{a:?}").contains("WindowConfig"));
    }
}
