//! Shape repository — caches hydrated art shapes keyed by name.

use std::collections::BTreeMap;

use bevy_ecs::prelude::Resource;
use engine_render::shape::Shape;

use crate::card::identity::signature::{Aspect, CardSignature, Element};

use super::armor1;
use super::barbarian_icons_01_t;

/// A resolved art entry binding a shape list to its card identity metadata.
pub struct ArtEntry {
    shapes: Vec<Shape>,
    element: Element,
    aspect: Aspect,
    signature: CardSignature,
}

impl ArtEntry {
    pub fn new(
        shapes: Vec<Shape>,
        element: Element,
        aspect: Aspect,
        signature: CardSignature,
    ) -> Self {
        Self {
            shapes,
            element,
            aspect,
            signature,
        }
    }

    pub fn element(&self) -> Element {
        self.element
    }

    pub fn aspect(&self) -> Aspect {
        self.aspect
    }

    pub fn signature(&self) -> CardSignature {
        self.signature
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

/// Cached shape repository. Call `hydrate_all` once during startup
/// (e.g. splash screen) to populate, then `get` to retrieve cloned shapes.
#[derive(Resource)]
pub struct ShapeRepository {
    cache: BTreeMap<&'static str, ArtEntry>,
}

impl Default for ShapeRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ShapeRepository {
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
        }
    }

    /// Hydrate all registered art shapes and store them in the cache.
    pub fn hydrate_all(&mut self) {
        self.cache.insert(
            "armor1",
            ArtEntry::new(
                armor1::armor1(),
                Element::Solidum,
                Aspect::Solid,
                CardSignature::default(),
            ),
        );
        self.cache.insert(
            "barbarian_icons_01_t",
            ArtEntry::new(
                barbarian_icons_01_t::barbarian_icons_01_t(),
                Element::Solidum,
                Aspect::Solid,
                CardSignature::default(),
            ),
        );
    }

    /// Returns a clone of the cached shapes for `name`, or `None` if not hydrated.
    pub fn get(&self, name: &str) -> Option<Vec<Shape>> {
        self.cache.get(name).map(|entry| entry.shapes.clone())
    }

    /// Returns the full art entry for `name`, or `None` if not hydrated.
    pub fn get_entry(&self, name: &str) -> Option<&ArtEntry> {
        self.cache.get(name)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn insert(&mut self, name: &'static str, entry: ArtEntry) {
        self.cache.insert(name, entry);
    }

    pub fn by_element(&self, element: Element) -> Vec<(&str, &ArtEntry)> {
        self.cache
            .iter()
            .filter(|(_, entry)| entry.element == element)
            .map(|(&name, entry)| (name, entry))
            .collect()
    }

    pub fn by_aspect(&self, aspect: Aspect) -> Vec<(&str, &ArtEntry)> {
        self.cache
            .iter()
            .filter(|(_, entry)| entry.aspect == aspect)
            .map(|(&name, entry)| (name, entry))
            .collect()
    }

    /// Returns the `n` entries closest to the given signature, sorted by distance ascending.
    pub fn closest_to(&self, query: &CardSignature, n: usize) -> Vec<(&str, &ArtEntry)> {
        let mut entries: Vec<(&str, &ArtEntry, f32)> = self
            .cache
            .iter()
            .map(|(&name, entry)| (name, entry, entry.signature.distance_to(query)))
            .collect();
        entries.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        entries
            .into_iter()
            .take(n)
            .map(|(name, entry, _)| (name, entry))
            .collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&&'static str, &ArtEntry)> {
        self.cache.iter()
    }
}

/// Returns the first art entry whose element matches the dominant axis of `signature`.
///
/// The dominant element is the one with the highest absolute intensity value.
/// Falls back to the closest entry by signature distance if no match for that element exists.
/// Returns `None` if the repository is empty.
pub fn select_art_for_signature<'a>(
    signature: &CardSignature,
    repo: &'a ShapeRepository,
) -> Option<&'a ArtEntry> {
    let dominant = Element::ALL.iter().copied().max_by(|&a, &b| {
        signature
            .intensity(a)
            .partial_cmp(&signature.intensity(b))
            .unwrap_or(std::cmp::Ordering::Equal)
    })?;
    repo.by_element(dominant)
        .into_iter()
        .next()
        .map(|(_, e)| e)
        .or_else(|| {
            repo.closest_to(signature, 1)
                .into_iter()
                .next()
                .map(|(_, e)| e)
        })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{ArtEntry, ShapeRepository};
    use crate::card::identity::signature::{Aspect, CardSignature, Element};

    #[test]
    fn when_get_entry_called_with_known_name_then_returns_art_entry_with_metadata() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.hydrate_all();

        // Act
        let entry = repo.get_entry("armor1");

        // Assert
        let entry = entry.expect("armor1 should be in repository");
        assert_eq!(entry.element(), Element::Solidum);
        assert_eq!(entry.aspect(), Aspect::Solid);
        assert_eq!(entry.signature(), CardSignature::default());
        assert!(!entry.shapes().is_empty());
    }

    #[test]
    fn when_get_entry_called_with_unknown_name_then_returns_none() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.hydrate_all();

        // Act
        let result = repo.get_entry("nonexistent_art");

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_get_called_with_known_name_then_returns_shapes() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.hydrate_all();

        // Act
        let result = repo.get("armor1");

        // Assert
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn when_get_called_with_unknown_name_then_returns_none() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.hydrate_all();

        // Act
        let result = repo.get("nonexistent_art");

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_hydrate_all_called_then_repository_contains_expected_entry_count() {
        // Arrange
        let mut repo = ShapeRepository::new();

        // Act
        repo.hydrate_all();

        // Assert
        assert_eq!(repo.len(), 2);
    }

    fn make_entry(element: Element, aspect: Aspect) -> ArtEntry {
        ArtEntry::new(vec![], element, aspect, CardSignature::default())
    }

    fn make_entry_with_sig(element: Element, aspect: Aspect, axes: [f32; 8]) -> ArtEntry {
        ArtEntry::new(vec![], element, aspect, CardSignature::new(axes))
    }

    #[test]
    fn when_by_element_called_then_returns_only_matching_entries() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Solidum, Aspect::Solid));
        repo.insert("b", make_entry(Element::Solidum, Aspect::Fragile));
        repo.insert("c", make_entry(Element::Febris, Aspect::Heat));

        // Act
        let results = repo.by_element(Element::Solidum);

        // Assert
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|(_, e)| e.element() == Element::Solidum));
    }

    #[test]
    fn when_by_element_called_with_no_matches_then_returns_empty_vec() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Solidum, Aspect::Solid));

        // Act
        let results = repo.by_element(Element::Febris);

        // Assert
        assert!(results.is_empty());
    }

    #[test]
    fn when_by_aspect_called_then_returns_only_matching_entries() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Solidum, Aspect::Solid));
        repo.insert("b", make_entry(Element::Solidum, Aspect::Fragile));
        repo.insert("c", make_entry(Element::Febris, Aspect::Heat));

        // Act
        let results = repo.by_aspect(Aspect::Solid);

        // Assert
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].1.aspect(), Aspect::Solid);
    }

    #[test]
    fn when_by_aspect_called_with_no_matches_then_returns_empty_vec() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Solidum, Aspect::Solid));

        // Act
        let results = repo.by_aspect(Aspect::Heat);

        // Assert
        assert!(results.is_empty());
    }

    #[test]
    fn when_closest_to_called_with_n2_then_returns_two_nearest_in_ascending_order() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert(
            "a",
            make_entry_with_sig(Element::Solidum, Aspect::Solid, [1.0; 8]),
        );
        repo.insert(
            "b",
            make_entry_with_sig(Element::Solidum, Aspect::Solid, [0.5; 8]),
        );
        repo.insert(
            "c",
            make_entry_with_sig(Element::Solidum, Aspect::Solid, [0.0; 8]),
        );
        let query = CardSignature::new([-0.1; 8]);

        // Act
        let results = repo.closest_to(&query, 2);

        // Assert
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "c"); // [0.0;8] is closest to [-0.1;8]
        assert_eq!(results[1].0, "b"); // [0.5;8] is next closest
    }

    #[test]
    fn when_closest_to_called_with_large_n_then_returns_all_entries() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Solidum, Aspect::Solid));
        repo.insert("b", make_entry(Element::Febris, Aspect::Heat));

        // Act
        let results = repo.closest_to(&CardSignature::default(), 10);

        // Assert
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn when_closest_to_called_on_empty_repository_then_returns_empty_vec() {
        // Arrange
        let repo = ShapeRepository::new();

        // Act
        let results = repo.closest_to(&CardSignature::default(), 5);

        // Assert
        assert!(results.is_empty());
    }

    #[test]
    fn when_by_element_results_inspected_then_all_entries_share_same_element() {
        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("a", make_entry(Element::Lumines, Aspect::Light));
        repo.insert("b", make_entry(Element::Lumines, Aspect::Dark));
        repo.insert("c", make_entry(Element::Varias, Aspect::Change));
        repo.insert("d", make_entry(Element::Varias, Aspect::Stasis));

        // Act
        let results = repo.by_element(Element::Lumines);

        // Assert
        assert_eq!(results.len(), 2);
        for (_, entry) in &results {
            assert_eq!(entry.element(), Element::Lumines);
        }
    }

    #[test]
    fn when_selecting_art_with_negative_dominant_axis_then_returns_entry_for_that_element() {
        use super::select_art_for_signature;

        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("solidum", make_entry(Element::Solidum, Aspect::Solid));
        repo.insert("febris", make_entry(Element::Febris, Aspect::Heat));
        repo.insert("ordinem", make_entry(Element::Ordinem, Aspect::Order));
        repo.insert("lumines", make_entry(Element::Lumines, Aspect::Light));
        repo.insert("varias", make_entry(Element::Varias, Aspect::Change));
        repo.insert("inertiae", make_entry(Element::Inertiae, Aspect::Force));
        repo.insert("subsidium", make_entry(Element::Subsidium, Aspect::Growth));
        repo.insert("spatium", make_entry(Element::Spatium, Aspect::Expansion));
        // Ordinem axis (index 2) is dominant at -0.95; magnitude beats all others
        let signature = CardSignature::new([0.1, 0.2, -0.95, 0.3, 0.0, 0.1, 0.2, 0.1]);

        // Act
        let result = select_art_for_signature(&signature, &repo);

        // Assert
        let entry = result.expect("expected Some for dominant negative Ordinem");
        assert_eq!(entry.element(), Element::Ordinem);
    }

    #[test]
    fn when_selecting_art_from_empty_repository_then_returns_none() {
        use super::select_art_for_signature;

        // Arrange
        let repo = ShapeRepository::new();
        let signature = CardSignature::new([0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Act
        let result = select_art_for_signature(&signature, &repo);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_no_element_match_then_falls_back_to_closest_by_signature() {
        use super::select_art_for_signature;

        // Arrange
        let mut repo = ShapeRepository::new();
        repo.insert("febris", make_entry(Element::Febris, Aspect::Heat));
        // Signature whose dominant axis is Solidum — no Solidum art exists
        let signature = CardSignature::new([0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Act
        let result = select_art_for_signature(&signature, &repo);

        // Assert — the Febris entry must be returned as the closest fallback
        let entry = result.expect("expected Some when repo is non-empty");
        assert_eq!(entry.element(), Element::Febris);
    }
}
