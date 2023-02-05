use modor_internal::dyn_types::DynType;
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::panic::{RefUnwindSafe, UnwindSafe};

pub trait CameraRef:
    Any + Clone + Hash + PartialEq + Eq + Debug + Sync + Send + UnwindSafe + RefUnwindSafe
{
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CameraKey(DynType);

impl CameraKey {
    pub(crate) fn new(ref_: impl CameraRef) -> Self {
        Self(DynType::new(ref_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DefaultCameraRef;

impl CameraRef for DefaultCameraRef {}
