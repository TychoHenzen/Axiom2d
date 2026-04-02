#![allow(clippy::unwrap_used)]

use engine_ecs::schedule::Phase;

/// @doc: Phase indices must match enum declaration order because the app
/// loop iterates `Phase::ALL` by index to run schedules in the canonical
/// Input->PreUpdate->Update->PostUpdate->Render sequence. If a variant were
/// reordered without updating ALL, systems would execute in the wrong phase.
#[test]
fn when_index_then_matches_declaration_order() {
    for (expected, phase) in Phase::ALL.iter().enumerate() {
        assert_eq!(
            phase.index(),
            expected,
            "{phase:?} should have index {expected}"
        );
    }
}
