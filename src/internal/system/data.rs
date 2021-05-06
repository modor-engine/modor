use crate::{SystemWrapper, TypeAccess};
use std::any::TypeId;

pub(crate) struct SystemInfo {
    pub(super) wrapper: SystemWrapper,
    pub(super) component_types: Vec<TypeAccess>,
    pub(super) entity_type: Option<TypeId>,
    pub(super) group_actions: bool,
}

impl SystemInfo {
    pub(crate) fn new(
        wrapper: SystemWrapper,
        component_types: Vec<TypeAccess>,
        entity_type: Option<TypeId>,
        group_actions: bool,
    ) -> Self {
        Self {
            wrapper,
            component_types,
            entity_type,
            group_actions,
        }
    }
}
