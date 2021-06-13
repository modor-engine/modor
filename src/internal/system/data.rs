use crate::external::systems::building::internal::TypeAccess;
use crate::SystemWrapper;
use std::any::TypeId;

pub(crate) struct SystemDetails {
    pub(super) wrapper: SystemWrapper,
    pub(super) component_types: Vec<TypeAccess>,
    pub(super) entity_type: Option<TypeId>,
    pub(super) actions: bool,
}

impl SystemDetails {
    pub(crate) fn new(
        wrapper: SystemWrapper,
        component_types: Vec<TypeAccess>,
        entity_type: Option<TypeId>,
        actions: bool,
    ) -> Self {
        Self {
            wrapper,
            component_types,
            entity_type,
            actions,
        }
    }
}
