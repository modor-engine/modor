use modor_internal::dyn_types::DynType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShaderKey(DynType);

impl ShaderKey {
    pub(crate) fn new(ref_: ShaderRef) -> Self {
        Self(DynType::new(ref_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShaderRef {
    Rectangle,
    Ellipse,
}
