use modor_internal::dyn_types::DynType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ModelKey(DynType);

impl ModelKey {
    pub(crate) fn new(ref_: ModelRef) -> Self {
        Self(DynType::new(ref_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ModelRef {
    Rectangle,
}
