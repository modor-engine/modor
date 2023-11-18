#![allow(clippy::non_canonical_clone_impl)]

use crate::Resource;
use derivative::Derivative;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// A key used for identifying a resource.
///
/// `R` type ensures at compile time that a key is used for the correct resource type.
/// Each `R` type also has its dedicated namespace, which means that two keys with same ID but
/// different `R` types are considered as different.
///
/// # Examples
///
/// See [`ResourceRegistry`](crate::ResourceRegistry).
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = "")
)]
pub struct ResKey<R> {
    id: ResKeyId,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    label: &'static str,
    #[derivative(PartialEq = "ignore", Hash = "ignore")]
    phantom: PhantomData<fn(R)>,
}

impl<R> ResKey<R> {
    /// Creates a new key identified by an `id`.
    pub const fn new(id: &'static str) -> Self
    where
        R: Resource,
    {
        Self {
            id: ResKeyId::Label(id),
            label: id,
            phantom: PhantomData,
        }
    }

    /// Creates a new key with a unique internal ID.
    ///
    /// It is ensured that the generated internal ID is not assigned to any other created key
    /// (a static [`AtomicU64`] is internally incremented to generate a new ID).
    ///
    /// `label` is only used for debugging purpose. It is not used to identify the key.
    ///
    /// # Panics
    ///
    /// This will panic if at least 2^64 unique keys have already been created.
    /// However, in the real world, this is not expected to happen as it would take 573 years with
    /// one billion keys created each second.
    pub fn unique(label: &'static str) -> Self
    where
        R: Resource,
    {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "too many `ResKey` instances created");
        Self {
            id: ResKeyId::Index(id),
            label,
            phantom: PhantomData,
        }
    }

    /// Returns the ID that identifies the key.
    pub const fn id(self) -> ResKeyId {
        self.id
    }

    /// Returns the key label used to identify the key for debugging purpose.
    pub fn label(self) -> String {
        match self.id {
            ResKeyId::Label(label) => label.into(),
            ResKeyId::LabeledIndex(label, index) => format!("{label}.{index}"),
            ResKeyId::Index(index) => format!("{}#{index}", self.label),
        }
    }
}

/// A key generator used for creating keys identified by a predictable index.
///
/// This type can be used to create [`ResKey`]s in a more dynamic manner.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_resources::*;
/// #
/// # struct Material;
/// #
/// # impl Resource for Material {
/// #     fn key(&self) -> ResKey<Self> {
/// #         unimplemented!()
/// #     }
/// #
/// #     fn state(&self) -> ResourceState<'_> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// const PLAYER_MATERIALS: IndexResKey<Material> = IndexResKey::new("player");
///
/// fn player_entity(player_id: usize) -> impl BuiltEntity {
///     let material_key = PLAYER_MATERIALS.get(player_id);
///     EntityBuilder::new()
///     // ...
/// }
/// ```
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Debug(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = "")
)]
pub struct IndexResKey<R> {
    root: ResKey<R>,
}

impl<R> IndexResKey<R> {
    /// Creates a new key generator that creates keys identified by an `id`.
    pub const fn new(id: &'static str) -> Self
    where
        R: Resource,
    {
        Self {
            root: ResKey::new(id),
        }
    }

    /// Creates a new key identified by the ID of the generator and `index`.
    pub const fn get(self, index: usize) -> ResKey<R> {
        ResKey {
            id: ResKeyId::LabeledIndex(self.root.label, index),
            label: self.root.label,
            phantom: PhantomData,
        }
    }
}

/// The ID of a [`ResKey`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResKeyId {
    /// The key is identified by a label.
    Label(&'static str),
    /// The key is identified by a label and an index.
    LabeledIndex(&'static str, usize),
    /// The key is identified by an index.
    Index(u64),
}
