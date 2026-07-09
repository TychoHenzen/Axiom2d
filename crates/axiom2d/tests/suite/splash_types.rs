#![allow(clippy::unwrap_used)]

use axiom2d::prelude::*;

/// @doc: Verifies `SPLASH_DURATION` is a positive finite constant.
#[test]
#[allow(clippy::assertions_on_constants)]
fn when_splash_duration_constant_then_is_positive_and_finite() {
    // Assert
    assert!(SPLASH_DURATION > 0.0, "SPLASH_DURATION should be positive");
    assert!(
        SPLASH_DURATION.is_finite(),
        "SPLASH_DURATION should be finite"
    );
}

/// @doc: Verifies `SplashScreen::new` creates resource with expected initial elapsed, duration, and done state.
#[test]
fn when_splash_screen_created_then_initial_state_is_correct() {
    // Arrange
    let duration = 3.0;

    // Act
    let splash = SplashScreen::new(duration);

    // Assert
    assert_eq!(
        splash.duration, 3.0,
        "duration should match constructor argument"
    );
    assert_eq!(splash.elapsed, 0.0, "elapsed should start at zero");
    assert!(!splash.done, "done should start as false");
}

/// @doc: Verifies `SplashScreen` accepts zero duration as edge case.
#[test]
fn when_splash_screen_created_with_zero_duration_then_elapsed_starts_zero() {
    // Arrange
    let splash = SplashScreen::new(0.0);

    // Assert
    assert_eq!(splash.elapsed, 0.0, "elapsed should start at zero");
    assert_eq!(splash.duration, 0.0, "duration should be zero");
    assert!(!splash.done, "done should start as false");
}

/// @doc: Verifies `PreloadHooks::new` creates resource with executed=false.
#[test]
fn when_preload_hooks_created_then_executed_is_false() {
    // Arrange
    let hooks = PreloadHooks::new();

    // Assert
    assert!(
        !hooks.executed,
        "new PreloadHooks should have executed=false"
    );
}

/// @doc: Verifies `PostSplashSetup::new` creates resource with executed=false.
#[test]
fn when_post_splash_setup_created_then_executed_is_false() {
    // Arrange
    let setup = PostSplashSetup::new();

    // Assert
    assert!(
        !setup.executed,
        "new PostSplashSetup should have executed=false"
    );
}
