#![allow(clippy::unwrap_used)]

use card_game::card::art::repository::{ArtEntry, ShapeRepository};
use card_game::card::art_selection::{
    art_bounding_box, fit_art_mesh_to_region, select_art_for_signature,
};
use card_game::card::identity::signature::{Aspect, CardSignature, Element};
use engine_render::shape::TessellatedColorMesh;

fn make_entry(element: Element, aspect: Aspect) -> ArtEntry {
    ArtEntry::new(vec![], element, aspect, CardSignature::default())
}

fn make_entry_at(element: Element, aspect: Aspect, sig: [f32; 8]) -> ArtEntry {
    ArtEntry::new(vec![], element, aspect, CardSignature::new(sig))
}

// --- select_art_for_signature ---

/// @doc: When one art entry's signature nearly matches the query and all others
/// are far away, the close entry is overwhelmingly likely to be selected.
/// This verifies the Gaussian distance weighting picks nearby art.
#[test]
fn when_one_entry_is_close_and_others_far_then_close_entry_selected() {
    // Arrange — Ordinem entry close to query, all others far away
    let mut repo = ShapeRepository::new();
    repo.insert(
        "solidum",
        make_entry_at(
            Element::Solidum,
            Aspect::Solid,
            [0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9],
        ),
    );
    repo.insert(
        "febris",
        make_entry_at(
            Element::Febris,
            Aspect::Heat,
            [-0.9, -0.9, -0.9, -0.9, -0.9, -0.9, -0.9, -0.9],
        ),
    );
    repo.insert(
        "ordinem",
        make_entry_at(
            Element::Ordinem,
            Aspect::Order,
            [0.1, 0.2, -0.9, 0.3, 0.0, 0.1, 0.2, 0.1],
        ),
    );
    repo.insert(
        "lumines",
        make_entry_at(
            Element::Lumines,
            Aspect::Light,
            [0.9, -0.9, 0.9, -0.9, 0.9, -0.9, 0.9, -0.9],
        ),
    );
    let signature = CardSignature::new([0.1, 0.2, -0.95, 0.3, 0.0, 0.1, 0.2, 0.1]);

    // Act
    let result = select_art_for_signature(&signature, &repo);

    // Assert
    let entry = result.expect("expected Some");
    assert_eq!(entry.element(), Element::Ordinem);
}

#[test]
fn when_selecting_art_from_empty_repository_then_returns_none() {
    // Arrange
    let repo = ShapeRepository::new();
    let signature = CardSignature::new([0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = select_art_for_signature(&signature, &repo);

    // Assert
    assert!(result.is_none());
}

/// @doc: When the repository has a single entry, it is always selected
/// regardless of signature distance — there is no competing art.
#[test]
fn when_single_entry_in_repo_then_always_selected() {
    // Arrange
    let mut repo = ShapeRepository::new();
    repo.insert("febris", make_entry(Element::Febris, Aspect::Heat));
    let signature = CardSignature::new([0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = select_art_for_signature(&signature, &repo);

    // Assert
    let entry = result.expect("expected Some when repo is non-empty");
    assert_eq!(entry.element(), Element::Febris);
}

/// @doc: Art selection is probabilistic — even when multiple entries are
/// equidistant from the query, different signatures produce different picks
/// due to the signature-seeded RNG. This prevents booster packs from having
/// all cards with identical art.
#[test]
fn when_many_nearby_signatures_then_not_all_select_same_entry() {
    // Arrange — 5 entries at moderate distances from the query region
    let mut repo = ShapeRepository::new();
    repo.insert(
        "a",
        make_entry_at(Element::Solidum, Aspect::Solid, [0.0; 8]),
    );
    repo.insert(
        "b",
        make_entry_at(
            Element::Febris,
            Aspect::Heat,
            [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    );
    repo.insert(
        "c",
        make_entry_at(
            Element::Ordinem,
            Aspect::Order,
            [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    );
    repo.insert(
        "d",
        make_entry_at(
            Element::Lumines,
            Aspect::Light,
            [0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    );
    repo.insert(
        "e",
        make_entry_at(
            Element::Varias,
            Aspect::Change,
            [0.0, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0],
        ),
    );

    // Act — generate 50 signatures near the origin, each seeds a different RNG
    let mut elements_seen = std::collections::HashSet::new();
    for i in 0..50 {
        let delta = (i as f32) * 0.002 - 0.05;
        let sig = CardSignature::new([delta, 0.001 * (i as f32), 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        if let Some(entry) = select_art_for_signature(&sig, &repo) {
            elements_seen.insert(entry.element());
        }
    }

    // Assert — at least 2 different art entries were selected
    assert!(
        elements_seen.len() >= 2,
        "expected variety in art selection, but only saw {elements_seen:?}"
    );
}

/// @doc: Art selection favors nearby entries — when one entry sits right on
/// the query signature and another is far away, the near entry is picked
/// more often across many nearby signatures. This validates the Gaussian
/// distance weighting produces a proximity bias.
#[test]
fn when_entry_nearby_then_selected_more_often_than_distant_entry() {
    // Arrange — "near" at origin, "far" at distance ~2.83
    let mut repo = ShapeRepository::new();
    repo.insert(
        "near",
        make_entry_at(Element::Solidum, Aspect::Solid, [0.0; 8]),
    );
    repo.insert(
        "far",
        make_entry_at(
            Element::Febris,
            Aspect::Heat,
            [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
        ),
    );

    // Act — sample 200 signatures near the origin
    let mut near_count = 0u32;
    let mut far_count = 0u32;
    for i in 0..200 {
        let delta = (i as f32) * 0.0005 - 0.05;
        let sig = CardSignature::new([delta, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        if let Some(entry) = select_art_for_signature(&sig, &repo) {
            if entry.element() == Element::Solidum {
                near_count += 1;
            } else {
                far_count += 1;
            }
        }
    }

    // Assert — near should be selected significantly more often
    assert!(
        near_count > far_count,
        "expected near ({near_count}) > far ({far_count})"
    );
}

/// @doc: Art selection is deterministic per signature — the same card
/// signature always selects the same art entry, providing hash-like stability.
#[test]
fn when_same_signature_called_twice_then_same_result() {
    // Arrange
    let mut repo = ShapeRepository::new();
    repo.insert(
        "a",
        make_entry_at(Element::Solidum, Aspect::Solid, [0.0; 8]),
    );
    repo.insert(
        "b",
        make_entry_at(
            Element::Febris,
            Aspect::Heat,
            [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    );
    repo.insert(
        "c",
        make_entry_at(
            Element::Ordinem,
            Aspect::Order,
            [0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ),
    );
    let sig = CardSignature::new([0.1, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result1 = select_art_for_signature(&sig, &repo);
    let result2 = select_art_for_signature(&sig, &repo);

    // Assert
    assert_eq!(
        result1.map(ArtEntry::element),
        result2.map(ArtEntry::element)
    );
}

// --- art_bounding_box ---

#[test]
fn when_computing_bbox_from_empty_mesh_then_returns_none() {
    // Arrange
    let mesh = TessellatedColorMesh::new();

    // Act
    let result = art_bounding_box(&mesh);

    // Assert
    assert!(result.is_none());
}

#[test]
fn when_computing_bbox_with_multiple_batches_then_spans_all() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    mesh.push_vertices(
        &[[-50.0, 20.0], [-10.0, 20.0], [-10.0, 50.0]],
        &[0, 1, 2],
        [1.0, 0.0, 0.0, 1.0],
    );
    mesh.push_vertices(
        &[[0.0, 30.0], [100.0, 30.0], [100.0, 80.0]],
        &[0, 1, 2],
        [0.0, 1.0, 0.0, 1.0],
    );

    // Act
    let result = art_bounding_box(&mesh);

    // Assert
    let (min, max) = result.expect("non-empty mesh");
    assert_eq!(min, [-50.0, 20.0]);
    assert_eq!(max, [100.0, 80.0]);
}

// --- fit_art_mesh_to_region ---

/// @doc: Art fitting must constrain all vertices to the card's art region.
/// Vertices outside the region would bleed into the card border or title
/// area, breaking the visual layout. The fit uses uniform scale + center
/// offset to preserve the original art proportions.
#[test]
fn when_fitting_art_into_region_then_all_vertices_within_bounds() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    mesh.push_vertices(
        &[
            [-100.0, -100.0],
            [100.0, -100.0],
            [100.0, 100.0],
            [-100.0, 100.0],
        ],
        &[0, 1, 2, 0, 2, 3],
        [1.0, 0.0, 0.0, 1.0],
    );
    let half_w = 24.0_f32;
    let half_h = 16.65_f32;
    let center_y = -7.2_f32;

    // Act
    let result = fit_art_mesh_to_region(&mesh, half_w, half_h, center_y);

    // Assert
    for v in &result.vertices {
        let [x, y] = v.position;
        assert!(
            x >= -half_w && x <= half_w,
            "x={x} outside [-{half_w}, {half_w}]"
        );
        assert!(
            y >= center_y - half_h && y <= center_y + half_h,
            "y={y} outside [{}, {}]",
            center_y - half_h,
            center_y + half_h
        );
    }
}

/// @doc: Art fitting preserves aspect ratio via uniform scaling — a 2:1
/// landscape art piece must stay 2:1 on the card. Non-uniform scaling would
/// squash or stretch the vector art, making it look distorted.
#[test]
fn when_fitting_art_with_nonsquare_input_then_aspect_ratio_preserved() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    mesh.push_vertices(
        &[
            [-200.0, -100.0],
            [200.0, -100.0],
            [200.0, 100.0],
            [-200.0, 100.0],
        ],
        &[0, 1, 2, 0, 2, 3],
        [1.0, 1.0, 1.0, 1.0],
    );
    let half = 20.0_f32;

    // Act
    let result = fit_art_mesh_to_region(&mesh, half, half, 0.0);

    // Assert
    let (min, max) = art_bounding_box(&result).unwrap();
    let out_w = max[0] - min[0];
    let out_h = max[1] - min[1];
    let ratio = out_w / out_h;
    assert!(
        (ratio - 2.0).abs() < 0.01,
        "expected 2:1 aspect ratio, got {ratio:.4}"
    );
}

#[test]
fn when_fitting_art_then_indices_preserved_unchanged() {
    // Arrange
    let mut mesh = TessellatedColorMesh::new();
    mesh.push_vertices(
        &[[-50.0, -50.0], [50.0, -50.0], [50.0, 50.0]],
        &[0, 1, 2],
        [1.0, 0.0, 0.0, 1.0],
    );
    mesh.push_vertices(
        &[[10.0, 10.0], [60.0, 10.0], [60.0, 60.0]],
        &[0, 1, 2],
        [0.0, 1.0, 0.0, 1.0],
    );
    let original_indices = mesh.indices.clone();

    // Act
    let result = fit_art_mesh_to_region(&mesh, 20.0, 20.0, 0.0);

    // Assert
    assert_eq!(result.indices, original_indices);
}
