use std::any::Any;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::panic::{RefUnwindSafe, UnwindSafe};

// This is a clonable dynamic `HashMap` key.
#[derive(Eq)]
pub struct DynKey(Box<dyn DynKeyType>);

impl DynKey {
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Sync + Send + UnwindSafe + RefUnwindSafe + PartialEq + Eq + Hash + Debug + Clone,
    {
        Self(Box::new(value))
    }
}

impl Hash for DynKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for DynKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Clone for DynKey {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

impl Debug for DynKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

trait DynKeyType: DynType + Debug + Sync + Send + UnwindSafe + RefUnwindSafe {
    fn as_dyn(&self) -> &dyn DynType;

    fn dyn_clone(&self) -> Box<dyn DynKeyType>;
}

impl<T> DynKeyType for T
where
    T: Any + Sync + Send + UnwindSafe + RefUnwindSafe + PartialEq + Eq + Hash + Debug + Clone,
{
    fn as_dyn(&self) -> &dyn DynType {
        self
    }

    fn dyn_clone(&self) -> Box<dyn DynKeyType> {
        Box::new(self.clone())
    }
}

impl PartialEq for dyn DynKeyType {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other.as_dyn())
    }
}

impl Eq for dyn DynKeyType {}

impl Hash for dyn DynKeyType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dyn_hash(state);
    }
}

trait DynType {
    fn as_any(&self) -> &dyn Any;

    fn dyn_eq(&self, other: &dyn DynType) -> bool;

    fn dyn_hash(&self, hasher: &mut dyn Hasher);
}

impl<T> DynType for T
where
    T: Any + PartialEq + Eq + Hash,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &dyn DynType) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |other| self == other)
    }

    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        T::hash(self, &mut hasher);
    }
}
