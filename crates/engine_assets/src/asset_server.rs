// EVOLVE-BLOCK-START
use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use serde::de::DeserializeOwned;

use crate::handle::Handle;

#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] ron::error::SpannedError),
}

struct AssetEntry<T> {
    value: T,
    ref_count: usize,
}

#[derive(Resource, Default)]
pub struct AssetServer<T: Send + Sync + 'static> {
    assets: HashMap<u32, AssetEntry<T>>,
    next_id: u32,
}

impl<T: Send + Sync + 'static> AssetServer<T> {
    pub fn add(&mut self, value: T) -> Handle<T> {
        let id = self.next_id;
        self.next_id += 1;
        self.assets.insert(
            id,
            AssetEntry {
                value,
                ref_count: 1,
            },
        );
        Handle::new(id)
    }

    pub fn get(&self, handle: Handle<T>) -> Option<&T> {
        self.assets.get(&handle.id).map(|entry| &entry.value)
    }

    pub fn get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.assets
            .get_mut(&handle.id)
            .map(|entry| &mut entry.value)
    }

    pub fn ref_count(&self, handle: Handle<T>) -> Option<usize> {
        self.assets.get(&handle.id).map(|entry| entry.ref_count)
    }

    /// Increments the reference count for the asset behind `handle`.
    pub fn clone_handle(&mut self, handle: Handle<T>) {
        if let Some(entry) = self.assets.get_mut(&handle.id) {
            entry.ref_count += 1;
        }
    }

    pub fn load(&mut self, path: &str) -> Result<Handle<T>, AssetError>
    where
        T: DeserializeOwned,
    {
        let contents = std::fs::read_to_string(path)?;
        let value: T = ron::from_str(&contents)?;
        Ok(self.add(value))
    }

    /// Decrements the reference count and evicts the asset when it reaches zero.
    ///
    /// Returns `false` if `handle` is not registered.
    pub fn remove(&mut self, handle: Handle<T>) -> bool {
        let Some(entry) = self.assets.get_mut(&handle.id) else {
            return false;
        };
        entry.ref_count -= 1;
        if entry.ref_count == 0 {
            self.assets.remove(&handle.id);
        }
        true
    }
}
// EVOLVE-BLOCK-END
