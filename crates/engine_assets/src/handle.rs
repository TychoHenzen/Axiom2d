use std::marker::PhantomData;

/// Type-safe asset handle. The phantom type parameter prevents mixing handles
/// across asset types.
///
/// ```compile_fail
/// use engine_assets::handle::Handle;
/// fn takes_string(_h: Handle<String>) {}
/// let h: Handle<u32> = Handle { id: 0, _marker: std::marker::PhantomData };
/// takes_string(h); // ERROR: expected Handle<String>, found Handle<u32>
/// ```
pub struct Handle<T> {
    pub id: u32,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub(crate) fn new(id: u32) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle").field("id", &self.id).finish()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};

    #[test]
    fn when_same_id_then_hashmap_deduplicates() {
        // Arrange
        let a = Handle::<u32>::new(42);
        let b = Handle::<u32>::new(42);
        let mut map: HashMap<Handle<u32>, &str> = HashMap::new();

        // Act
        map.insert(a, "first");
        map.insert(b, "second");

        // Assert
        assert_eq!(map.len(), 1);
        assert_eq!(map[&a], "second");
        assert!(format!("{a:?}").contains("42"));
    }

    #[test]
    fn when_different_ids_then_btreeset_orders_by_id() {
        // Arrange
        let lower = Handle::<u32>::new(1);
        let higher = Handle::<u32>::new(2);
        let mut set = BTreeSet::new();

        // Act
        set.insert(higher);
        set.insert(lower);
        let ordered: Vec<u32> = set.iter().map(|h| h.id).collect();

        // Assert
        assert_ne!(lower, higher);
        assert_eq!(ordered, vec![1, 2]);
    }
}
