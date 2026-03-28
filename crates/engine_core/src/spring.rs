#[allow(clippy::too_many_arguments)]
pub fn spring_step(
    current: f32,
    target: f32,
    velocity: f32,
    dt: f32,
    stiffness: f32,
    damping: f32,
) -> (f32, f32) {
    let displacement = target - current;
    let acceleration = displacement * stiffness - velocity * damping;
    let new_velocity = velocity + acceleration * dt;
    let new_position = current + new_velocity * dt;
    (new_position, new_velocity)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// @doc: Spring math correctness — verifies that the acceleration, velocity,
    /// and position formulas produce expected values for a known initial state.
    /// If the formula were wrong (e.g., wrong sign on damping), cards would
    /// oscillate wildly or freeze instead of smoothly settling.
    #[test]
    fn when_spring_step_from_zero_toward_positive_target_then_returns_computed_position_and_velocity()
     {
        // Arrange
        // acceleration = (100-0)*200 - 0*20 = 20_000
        // new_velocity = 0 + 20_000*0.016  = 320.0
        // new_position = 0 + 320.0*0.016   = 5.12

        // Act
        let (pos, vel) = spring_step(0.0, 100.0, 0.0, 0.016, 200.0, 20.0);

        // Assert
        assert!((pos - 5.12).abs() < 1e-4, "expected pos≈5.12, got {pos}");
        assert!((vel - 320.0).abs() < 1e-3, "expected vel≈320.0, got {vel}");
    }

    /// @doc: Equilibrium stability — a spring at rest at its target must produce
    /// zero force. If floating-point drift caused even tiny movement here, settled
    /// cards would jitter indefinitely instead of holding position.
    #[test]
    fn when_spring_step_at_target_with_zero_velocity_then_position_and_velocity_unchanged() {
        // Act
        let (pos, vel) = spring_step(50.0, 50.0, 0.0, 0.016, 200.0, 20.0);

        // Assert
        assert!((pos - 50.0).abs() < 1e-6, "expected pos≈50.0, got {pos}");
        assert!(vel.abs() < 1e-6, "expected vel≈0.0, got {vel}");
    }

    /// @doc: Overshoot produces restoring force — damped spring pulls back toward target, preventing oscillation blow-up
    #[test]
    fn when_spring_step_past_target_with_forward_velocity_then_velocity_decreases() {
        // Arrange
        // displacement = 100-150 = -50, acceleration = -50*200 - 50*20 = -11_000
        // new_velocity = 50 + (-11_000)*0.016 = 50 - 176 = -126

        // Act
        let (_, vel) = spring_step(150.0, 100.0, 50.0, 0.016, 200.0, 20.0);

        // Assert
        assert!(
            vel < 50.0,
            "expected velocity to decrease from 50.0, got {vel}"
        );
    }

    /// @doc: Damped convergence guarantee — after enough iterations the spring
    /// must settle near its target. An underdamped or divergent spring would cause
    /// cards to bounce forever or fly off-screen instead of snapping into their
    /// hand/stash layout positions.
    #[test]
    fn when_spring_step_iterated_300_times_then_converges_toward_target() {
        // Arrange
        let mut pos = 0.0_f32;
        let mut vel = 0.0_f32;
        let target = 100.0;

        // Act
        for _ in 0..300 {
            let (p, v) = spring_step(pos, target, vel, 0.016, 200.0, 20.0);
            pos = p;
            vel = v;
        }

        // Assert
        assert!(
            (pos - target).abs() < 1.0,
            "expected pos within 1.0 of target, got {pos}"
        );
    }

    /// @doc: Semi-implicit Euler integration — position uses updated velocity, giving better energy conservation than explicit Euler
    #[test]
    fn when_spring_step_position_computed_then_uses_new_velocity_not_old() {
        // Arrange
        // accel = 100*200 - 0*20 = 20_000
        // new_vel = 0 + 20_000*0.016 = 320.0
        // new_pos = 0 + 320.0*0.016 = 5.12 (semi-implicit Euler)
        // If explicit Euler were used: new_pos = 0 + 0*0.016 = 0.0

        // Act
        let (pos, _) = spring_step(0.0, 100.0, 0.0, 0.016, 200.0, 20.0);

        // Assert
        assert!(
            pos > 1.0,
            "semi-implicit Euler should give pos≈5.12, explicit Euler would give 0.0, got {pos}"
        );
    }
}
