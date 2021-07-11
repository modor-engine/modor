use crate::external::systems::definition::internal::ArchetypeInfo;
use crate::external::systems::params::{SystemParam, SystemParamData, SystemParamDataMut};
use crate::SystemData;
use std::any::{Any, TypeId};
use std::iter::FlatMap;
use std::slice::Iter;
use std::sync::{RwLock, RwLockReadGuard};

impl<'a, 'b, C> SystemParam<'a, 'b> for &'b C
where
    C: Any,
{
    type Guard = RwLockReadGuard<'a, Vec<Vec<C>>>;
    type Const = Self;
    type Iter = ComponentIter<'b, C>;
    type IterMut = Self::Iter;

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        /*let type_idx = data.type_idxs[&TypeId::of::<C>()];
        data.components[type_idx]
            .downcast_ref::<RwLock<Vec<Vec<C>>>>()
            .expect("Wrong type index used")
            .try_read()
            .expect("Resource already locked")*/
        todo!("make sure archetype order is the same for all types")
    }

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        archetypes.iter().map(|a| guard[a.idx].len()).sum()
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        ComponentIter {
            components: ComponentArchetypeIter {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter(),
            }
            .flat_map(|c| c.iter()),
            len: data.item_count,
        }
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        ComponentIter {
            components: ComponentArchetypeIter {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter(),
            }
            .flat_map(|c| c.iter()),
            len: data.item_count,
        }
    }
}

struct ComponentArchetypeIter<'a, C> {
    sorted_archetypes: Iter<'a, ArchetypeInfo>,
    last_archetype_idx: Option<usize>,
    component_archetypes: Iter<'a, Vec<C>>,
}

impl<'a, C> Iterator for ComponentArchetypeIter<'a, C> {
    type Item = &'a Vec<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next()?.idx;
        let nth = archetype_idx - self.last_archetype_idx.map_or(0, |i| i + 1);
        self.last_archetype_idx = Some(archetype_idx);
        self.component_archetypes.nth(nth)
    }
}

impl<'a, C> DoubleEndedIterator for ComponentArchetypeIter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next_back()?.idx;
        let nth_back = self.sorted_archetypes.len() - archetype_idx;
        self.component_archetypes.nth_back(nth_back)
    }
}

struct ComponentIter<'a, C> {
    #[allow(clippy::type_complexity)]
    components: FlatMap<ComponentArchetypeIter<'a, C>, Iter<'a, C>, fn(&Vec<C>) -> Iter<'_, C>>,
    len: usize,
}

impl<'a, C> Iterator for ComponentIter<'a, C> {
    type Item = &'a C;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.components.next()
    }
}

impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.components.next_back()
    }
}

impl<'a, C> ExactSizeIterator for ComponentIter<'a, C> {
    fn len(&self) -> usize {
        self.len
    }
}
