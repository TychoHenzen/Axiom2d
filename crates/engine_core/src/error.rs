use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(test)]
mod tests {
    use super::EngineError;

    #[test]
    fn when_not_found_displayed_then_contains_resource_name() {
        // Arrange
        let err = EngineError::NotFound("player".into());

        // Act
        let display = format!("{err}");

        // Assert
        assert!(display.contains("player"));
    }

    #[test]
    fn when_invalid_input_displayed_then_contains_reason() {
        // Arrange
        let err = EngineError::InvalidInput("negative scale".into());

        // Act
        let display = format!("{err}");

        // Assert
        assert!(display.contains("negative scale"));
    }

    #[test]
    fn when_engine_error_boxed_then_implements_std_error() {
        // Act
        let _: Box<dyn std::error::Error> = Box::new(EngineError::NotFound("x".into()));
    }

    #[test]
    fn when_engine_error_debug_formatted_then_identifies_variant() {
        // Arrange
        let err = EngineError::NotFound("x".into());

        // Act
        let debug = format!("{err:?}");

        // Assert
        assert!(debug.contains("NotFound"));
    }
}
