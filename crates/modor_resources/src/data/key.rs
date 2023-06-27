use crate::Resource;
use derivative::Derivative;
use std::any::Any;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
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
/// #     fn key(&self) -> &ResourceKey {
/// #         unimplemented!()
/// #     }
/// #
/// #     fn state(&self) -> ResourceState<'_> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// const PLAYER_MATERIAL: IndexResKey<Material> = IndexResKey::new("player");
///
/// fn player_entity(player_id: usize) -> impl BuiltEntity {
///     let material_key = PLAYER_MATERIAL.get(player_id);
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

// TODO: remove below items

/// A key used for identifying a resource.
///
/// This is a wrapped value of any type implementing common traits.
///
/// # Examples
///
/// See [`ResourceRegistry`](crate::ResourceRegistry).
pub struct ResourceKey(Box<dyn DynResourceKey>);

impl ResourceKey {
    /// Creates a new key from a `value`.
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
    {
        Self(Box::new(value))
    }
}

impl Clone for ResourceKey {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

impl Hash for ResourceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.dyn_hash(state);
    }
}

impl PartialEq for ResourceKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_partial_eq(&*other.0)
    }
}

impl Eq for ResourceKey {}

impl Debug for ResourceKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_fmt(f)
    }
}

trait DynResourceKey: Sync + Send + UnwindSafe + RefUnwindSafe {
    fn as_any(&self) -> &dyn Any;

    fn dyn_clone(&self) -> Box<dyn DynResourceKey>;

    fn dyn_hash(&self, hasher: &mut dyn Hasher);

    fn dyn_partial_eq(&self, other: &dyn DynResourceKey) -> bool;

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T> DynResourceKey for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_clone(&self) -> Box<dyn DynResourceKey> {
        Box::new(self.clone())
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        T::hash(self, &mut hasher);
    }

    fn dyn_partial_eq(&self, other: &dyn DynResourceKey) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |o| self == o)
    }

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}

/// A trait implemented for all values convertible to a [`ResourceKey`](ResourceKey).
pub trait IntoResourceKey {
    /// Converts the value into a [`ResourceKey`](ResourceKey).
    fn into_key(self) -> ResourceKey;
}

impl<T> IntoResourceKey for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    fn into_key(self) -> ResourceKey {
        ResourceKey::new(self)
    }
}
