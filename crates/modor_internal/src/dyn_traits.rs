use core::fmt;
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

#[macro_export]
macro_rules! dyn_clone_trait {
    ($vis:vis $trait_name:ident, $returned_trait_name:ident) => {
        $vis trait $trait_name: std::any::Any {
            fn dyn_clone(&self) -> Box<dyn $returned_trait_name>;
        }

        impl<T> $trait_name for T
        where
            T: std::any::Any + std::clone::Clone + $returned_trait_name,
        {
            fn dyn_clone(&self) -> Box<dyn $returned_trait_name> {
                Box::new(self.clone())
            }
        }
    };
}

pub trait DynHash {
    fn dyn_hash(&self, hasher: &mut dyn Hasher);
}

impl<T> DynHash for T
where
    T: Hash,
{
    fn dyn_hash(&self, mut hasher: &mut dyn Hasher) {
        T::hash(self, &mut hasher);
    }
}

pub trait DynPartialEq: Any {
    fn dyn_partial_eq(&self, other: &dyn DynPartialEq) -> bool;

    fn as_any(&self) -> &dyn Any;

    fn as_dyn_partial_eq(&self) -> &dyn DynPartialEq;
}

impl<T> DynPartialEq for T
where
    T: Any + PartialEq,
{
    fn dyn_partial_eq(&self, other: &dyn DynPartialEq) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |o| self == o)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_dyn_partial_eq(&self) -> &dyn DynPartialEq {
        self
    }
}

pub trait DynDebug {
    #[allow(clippy::missing_errors_doc)]
    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

impl<T> DynDebug for T
where
    T: Any + Debug,
{
    fn dyn_fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt(f)
    }
}
