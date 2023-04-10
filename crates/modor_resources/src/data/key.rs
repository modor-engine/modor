use std::any::Any;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::{RefUnwindSafe, UnwindSafe};

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
    fn into_key(self) -> ResourceKey;
}

impl<T> IntoResourceKey for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    /// Converts a value into a [`ResourceKey`](ResourceKey).
    fn into_key(self) -> ResourceKey {
        ResourceKey::new(self)
    }
}
