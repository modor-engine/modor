use crate::keys::models::ModelKey;
use crate::resources::models::Model;
use fxhash::FxHashMap;
use modor::{Built, Changed, Entity, EntityBuilder, Filter, Query};

pub(crate) struct ModelRegistry {
    entity_ids: FxHashMap<ModelKey, usize>,
}

#[singleton]
impl ModelRegistry {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            entity_ids: FxHashMap::default(),
        })
    }

    #[run]
    fn register(&mut self, models: Query<'_, (&Model, Entity<'_>, Filter<Changed<Model>>)>) {
        for (model, entity, _) in models.iter() {
            self.entity_ids.insert(model.key().clone(), entity.id());
        }
    }

    pub(crate) fn find<'a>(
        &self,
        key: &ModelKey,
        query: &'a Query<'_, &Model>,
    ) -> Option<&'a Model> {
        self.entity_ids.get(key).and_then(|&i| query.get(i))
    }
}
