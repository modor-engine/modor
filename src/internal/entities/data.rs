#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(in super::super) struct EntityLocation {
    pub(in super::super) archetype_idx: usize,
    pub(in super::super) entity_pos: usize,
}

impl EntityLocation {
    pub(in super::super) fn new(archetype_idx: usize, entity_pos: usize) -> Self {
        Self {
            archetype_idx,
            entity_pos,
        }
    }
}
