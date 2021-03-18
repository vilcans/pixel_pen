use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

/// Wraps an object and sets a "dirty" flag whenever any code accesses it mutably.
pub struct MutationMonitor<T> {
    target: T,
    /// Is set to true whenever the target is dereferenced via [`DerefMut`].
    pub dirty: bool,
}

impl<T> MutationMonitor<T> {
    /// Wrap the target object and set the dirty flag.
    pub fn new_dirty(target: T) -> Self {
        Self {
            target,
            dirty: true,
        }
    }
}

impl<T> Deref for MutationMonitor<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.target
    }
}

impl<T> DerefMut for MutationMonitor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.target
    }
}

impl<T> Serialize for MutationMonitor<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.target.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for MutationMonitor<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            target: T::deserialize(deserializer)?,
            dirty: true,
        })
    }
}
