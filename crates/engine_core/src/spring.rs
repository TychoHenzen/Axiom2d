// EVOLVE-BLOCK-START
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
// EVOLVE-BLOCK-END
