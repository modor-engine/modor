use crate::internal::entity_actions::data::AddComponentFn;
use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use std::any::TypeId;
use std::num::NonZeroUsize;

pub(in super::super) struct ActionResult {
    pub(in super::super) deleted_entity_idxs: Vec<usize>,
    pub(in super::super) entity_builders: Vec<CreateEntityFn>,
    pub(in super::super) deleted_component_types: Vec<(usize, TypeId)>,
    pub(in super::super) component_adders: Vec<AddComponentFn>,
    pub(in super::super) deleted_group_idxs: Vec<NonZeroUsize>,
    pub(in super::super) replaced_group_builders: Vec<(NonZeroUsize, BuildGroupFn)>,
}
