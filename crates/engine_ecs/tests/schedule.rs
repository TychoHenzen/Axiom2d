#![allow(clippy::unwrap_used)]

use engine_ecs::schedule::{PHASE_COUNT, Phase};

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

/// @doc: PHASE_COUNT is the compile-time constant used to size arrays and
/// pre-allocate schedule storage throughout the engine. If a variant is added
/// without updating PHASE_COUNT the constant drifts from reality, causing
/// under-allocated buffers and missed schedule slots at runtime.
#[test]
fn when_phase_count_then_equals_eighteen() {
    // Act
    let count = PHASE_COUNT;

    // Assert
    assert_eq!(
        count, 18,
        "PHASE_COUNT must equal the number of Phase variants"
    );
}

/// @doc: The Phase sequence is a frozen public contract: downstream crates,
/// save files, and documentation all reference phases by name and position.
/// Silently reordering or renaming a variant would shift every dependent
/// system to the wrong execution slot without a compile error. This test
/// pins the exact ordered list so any change is an explicit, visible break.
#[test]
fn when_all_phases_mapped_to_names_then_frozen_sequence() {
    // Arrange
    let expected = [
        "Startup",
        "OnEnable",
        "FixedUpdate",
        "AsyncFixedUpdate",
        "OnCollision",
        "Input",
        "Update",
        "Async",
        "Animate",
        "LateUpdate",
        "OnBecameVisible",
        "Render",
        "PostRender",
        "AsyncEndOfFrame",
        "OnPause",
        "OnDisable",
        "OnDestroy",
        "WaitForVBlank",
    ];

    // Act
    let actual: Vec<&str> = Phase::ALL.iter().map(|p| p.name()).collect();

    // Assert
    assert_eq!(
        actual, expected,
        "Phase list is frozen at 18. Do not add phases — add systems to existing phases instead."
    );
}
