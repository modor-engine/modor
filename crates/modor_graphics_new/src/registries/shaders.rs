use crate::keys::shaders::ShaderKey;
use crate::resources::shaders::Shader;
use fxhash::FxHashMap;
use modor::{Built, Changed, Entity, EntityBuilder, Filter, Query};

pub(crate) struct ShaderRegistry {
    entity_ids: FxHashMap<ShaderKey, usize>,
}

#[singleton]
impl ShaderRegistry {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            entity_ids: FxHashMap::default(),
        })
    }

    #[run]
    fn register(&mut self, shaders: Query<'_, (&Shader, Entity<'_>, Filter<Changed<Shader>>)>) {
        for (shader, entity, _) in shaders.iter() {
            self.entity_ids.insert(shader.key().clone(), entity.id());
        }
    }

    pub(crate) fn find<'a>(
        &self,
        key: &ShaderKey,
        query: &'a Query<'_, &Shader>,
    ) -> Option<&'a Shader> {
        self.entity_ids.get(key).and_then(|&i| query.get(i))
    }
}
