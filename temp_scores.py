import json

result = [
    {
        "file": "booster_device.rs",
        "status": "scored",
        "scores": [7, 10, 9, 7, 6, 10, 7, 10],
        "overall": 8.1,
        "findings": []
    },
    {
        "file": "booster_double_click.rs",
        "status": "scored",
        "scores": [7, 10, 10, 8, 5, 10, 6, 10],
        "overall": 8.0,
        "findings": [{"severity": "low", "category": "coverage_depth", "detail": "Missing edge cases: boundary at exactly DOUBLE_CLICK_WINDOW, rapid multi-click sequences, Entity::from_raw(0) as valid entity", "suggestion": "Add test for timing exactly at DOUBLE_CLICK_WINDOW, triple-click behavior, Entity::PLACEHOLDER guard"}]
    },
    {
        "file": "booster_opening.rs",
        "status": "scored",
        "scores": [7, 10, 10, 8, 5, 10, 7, 10],
        "overall": 8.1,
        "findings": [{"severity": "low", "category": "coverage_depth", "detail": "Missing edge cases: negative advance time, zero-card opening, single-card opening, max-card opening", "suggestion": "Add tests for edge case inputs to advance() and fan_position() with 0, 1, and large card counts"}]
    },
    {
        "file": "booster_pack.rs",
        "status": "scored",
        "scores": [6, 10, 9, 6, 5, 10, 6, 10],
        "overall": 7.5,
        "findings": [{"severity": "low", "category": "coverage_depth", "detail": "Missing empty signature list, edge-of-bounds vertex tests, pack with 0 cards, pack with max cards", "suggestion": "Add spawn tests with empty sigs, verify Pack component invariants after spawn, add boundary vertex tests"}]
    },
    {
        "file": "booster_sampling.rs",
        "status": "scored",
        "scores": [8, 10, 10, 7, 6, 9, 8, 10],
        "overall": 8.4,
        "findings": []
    },
    {
        "file": "card_art_selection.rs",
        "status": "scored",
        "scores": [8, 9, 10, 9, 8, 9, 8, 10],
        "overall": 8.7,
        "findings": []
    },
    {
        "file": "card_combiner_device.rs",
        "status": "scored",
        "scores": [8, 10, 9, 8, 9, 9, 7, 10],
        "overall": 8.8,
        "findings": []
    },
    {
        "file": "card_identity_card_name.rs",
        "status": "scored",
        "scores": [8, 10, 10, 8, 8, 10, 7, 10],
        "overall": 8.8,
        "findings": []
    },
    {
        "file": "card_identity_nouns.rs",
        "status": "scored",
        "scores": [6, 10, 10, 8, 6, 10, 7, 10],
        "overall": 8.1,
        "findings": []
    },
    {
        "file": "card_identity_signature.rs",
        "status": "scored",
        "scores": [9, 10, 10, 9, 9, 9, 9, 10],
        "overall": 9.4,
        "findings": []
    },
    {
        "file": "card_identity_signature_types.rs",
        "status": "scored",
        "scores": [4, 10, 10, 8, 5, 10, 6, 10],
        "overall": 7.5,
        "findings": [
            {"severity": "medium", "category": "assertion_quality", "detail": "Many tests verify Rust derive macros (Clone, Copy, Default, Eq, Ord, PartialEq) which are banned per project guidelines", "suggestion": "Remove derive-verification tests and replace with behavioral tests of the type purpose"},
            {"severity": "low", "category": "coverage_depth", "detail": "Tests only struct-lifetime properties rather than behavioral properties", "suggestion": "Add tests for signature operations, element indexing edge cases, or serialization roundtrips"}
        ]
    },
    {
        "file": "card_identity_templates.rs",
        "status": "scored",
        "scores": [7, 10, 10, 8, 7, 10, 8, 10],
        "overall": 8.5,
        "findings": []
    },
    {
        "file": "card_identity_visual_params.rs",
        "status": "scored",
        "scores": [9, 10, 10, 9, 8, 10, 9, 10],
        "overall": 9.3,
        "findings": []
    },
    {
        "file": "card_interaction_apply.rs",
        "status": "scored",
        "scores": [8, 10, 8, 8, 6, 9, 7, 10],
        "overall": 8.2,
        "findings": []
    },
    {
        "file": "card_interaction_click_resolve.rs",
        "status": "scored",
        "scores": [8, 10, 8, 8, 7, 9, 7, 10],
        "overall": 8.4,
        "findings": []
    },
    {
        "file": "card_interaction_drag.rs",
        "status": "scored",
        "scores": [9, 9, 7, 9, 9, 9, 8, 10],
        "overall": 8.8,
        "findings": []
    },
    {
        "file": "card_interaction_drag_state.rs",
        "status": "scored",
        "scores": [3, 10, 10, 7, 2, 10, 4, 10],
        "overall": 6.5,
        "findings": [
            {"severity": "medium", "category": "assertion_quality", "detail": "All tests verify struct field access and Option construction -- banned per project guidelines", "suggestion": "Replace with tests of drag state transitions or behavioral tests that exercise DragState through the systems that consume it"},
            {"severity": "high", "category": "coverage_depth", "detail": "No behavioral testing at all: only verifies that struct fields store what was assigned to them", "suggestion": "Add tests for DragState lifecycle: start drag, update during drag, clear on release, guard against double-drag, origin_zone validation"}
        ]
    },
    {
        "file": "card_rendering_baked_mesh.rs",
        "status": "scored",
        "scores": [3, 10, 10, 7, 2, 10, 4, 10],
        "overall": 6.5,
        "findings": [
            {"severity": "medium", "category": "assertion_quality", "detail": "Most tests verify Rust/ECS derive guarantees (Default, Clone, Component derive, struct construction) which are banned per project guidelines", "suggestion": "Remove all derive-verification tests and replace with behavioral tests that render a baked mesh, verify vertex/uv transforms, or test overlay layering"},
            {"severity": "high", "category": "coverage_depth", "detail": "No behavioral tests: zero coverage of mesh baking logic, overlay positioning, vertex transformations, or interaction with rendering systems", "suggestion": "Add tests for bake_front_face output invariants, CardOverlays layering order, mesh transformation accuracy"}
        ]
    },
    {
        "file": "card_interaction_system.rs",
        "status": "not_found",
        "findings": []
    },
    {
        "file": "stash_render_system.rs",
        "status": "not_found",
        "findings": [],
        "note": "Related file stash_render.rs exists (21 tests, score 8.7) in extras"
    },
    {
        "file": "terrain_collision.rs",
        "status": "not_found",
        "findings": []
    },
    {
        "file": "terrain_rendering.rs",
        "status": "not_found",
        "findings": []
    },
    {
        "file": "terrain_system.rs",
        "status": "not_found",
        "findings": [],
        "note": "Related file terrain.rs exists (3 tests, score 7.5) in extras"
    },
    {
        "file": "stash_render.rs",
        "status": "scored_extra",
        "scores": [8, 9, 8, 9, 9, 9, 8, 10],
        "overall": 8.7,
        "findings": []
    },
    {
        "file": "terrain.rs",
        "status": "scored_extra",
        "scores": [6, 10, 10, 6, 4, 10, 6, 10],
        "overall": 7.5,
        "findings": [{"severity": "medium", "category": "coverage_depth", "detail": "Only tests the empty-tileset build-script codegen path -- zero coverage of non-empty tilesets", "suggestion": "Once TSX assets are added, add tests for tile properties, adjacency rules, and tile rendering invariants"}]
    }
]

print(json.dumps(result, indent=2))
