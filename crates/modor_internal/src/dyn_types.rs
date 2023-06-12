use std::any::Any;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::panic::{RefUnwindSafe, UnwindSafe};

// TODO: remove this module
// coverage: off

pub struct DynType(Box<dyn DynTrait>);

impl DynType {
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
    {
        Self(Box::new(value))
    }
}

impl Clone for DynType {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

impl Hash for DynType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.dyn_hash(state);
    }
}

impl PartialEq for DynType {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_partial_eq(&*other.0)
    }
}

impl Eq for DynType {}

impl Debug for DynType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_fmt(f)
    }
}

trait DynTrait: Sync + Send + UnwindSafe + RefUnwindSafe {
    fn as_any(&self) -> &dyn Any;

    fn dyn_clone(&self) -> Box<dyn DynTrait>;

    fn dyn_hash(&self, hasher: &mut dyn Hasher);

    fn dyn_partial_eq(&self, other: &dyn DynTrait) -> bool;

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T> DynTrait for T
where
    T: Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_clone(&self) -> Box<dyn DynTrait> {
        Box::new(self.clone())
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        T::hash(self, &mut hasher);
    }

    fn dyn_partial_eq(&self, other: &dyn DynTrait) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |o| self == o)
    }

    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}
