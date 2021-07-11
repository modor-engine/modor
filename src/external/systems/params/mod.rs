use crate::external::systems::definition::internal::ArchetypeInfo;
use crate::SystemData;
use std::any::TypeId;

// TODO: make sure traits are not implementable outside
// TODO: move items in private mods when possible
// TODO: add doc

pub trait SystemParam<'a, 'b>: 'b {
    type Guard: 'a;
    type Const: 'b;
    type Iter: Iterator<Item = Self::Const> + DoubleEndedIterator + ExactSizeIterator + 'b;
    type IterMut: Iterator<Item = Self> + DoubleEndedIterator + ExactSizeIterator + 'b;

    fn mandatory_component_types() -> Vec<TypeId>;

    fn lock(data: &'a SystemData<'_>) -> Self::Guard;

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize;

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter;

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut;
}

pub struct SystemParamData<'a, G> {
    guard: &'a G,
    sorted_archetypes: &'a [ArchetypeInfo],
    item_count: usize,
}

macro_rules! map_system_param_data {
    ($data:ident, $guard_field:tt) => {
        SystemParamData {
            guard: &$data.guard.$guard_field,
            sorted_archetypes: $data.sorted_archetypes,
            item_count: $data.item_count,
        }
    };
}

pub struct SystemParamDataMut<'a, G> {
    guard: &'a mut G,
    sorted_archetypes: &'a [ArchetypeInfo],
    item_count: usize,
}

macro_rules! map_system_param_data_mut {
    ($data:ident, $guard_field:tt) => {
        SystemParamDataMut {
            guard: &mut $data.guard.$guard_field,
            sorted_archetypes: $data.sorted_archetypes,
            item_count: $data.item_count,
        }
    };
}

pub(crate) mod tuples;
pub(crate) mod components;
