use crate::new::target_sync::generic::{EntityState, GenerationData, GenerationDataRegister};
use modor::{BuiltEntity, Custom, EntityBuilder, EntityMut, Filter, Or, Query, SingleRef, With};
use modor_graphics::{
    Color, RenderTarget, Size, Texture, TextureBuffer, TextureBufferPart, TextureSource, Window,
    MAIN_RENDERING,
};
use modor_resources::{ResKey, Resource, ResourceRegistry};

const PICKING_RENDERING: &str = "PICKING";

#[derive(Debug, SingletonComponent)]
pub(crate) struct TargetSync {
    generation_data: GenerationDataRegister<RenderTarget, TargetData>,
}

#[systems]
impl TargetSync {
    #[run]
    fn update(&mut self, mut entity: EntityMut<'_>, mut resources: Custom<TargetResources<'_>>) {
        let mut resources = resources.as_mut();
        self.generation_data.reset();
        for target in resources.query.iter() {
            if target.target.category == MAIN_RENDERING {
                let key = target.target.key();
                let data = TargetData::new(&target);
                if let EntityState::New(data) = self.generation_data.register(key, data) {
                    entity.create_child(Self::create_target(data))
                }
            }
        }
        self.generation_data
            .remove_outdated(resources.registry, entity.world());
        for (key, data) in self.generation_data.iter_mut() {
            Self::update_target(key, data, &mut resources);
        }
    }

    fn create_target(data: &GenerationData<RenderTarget, TargetData>) -> impl BuiltEntity {
        let texture_key = data.data.generated_texture_key;
        EntityBuilder::new()
            .component(RenderTarget::new(data.generated_key))
            .with(|t| t.is_anti_aliasing_enabled = false)
            .with(|t| t.background_color = Color::WHITE)
            .with(|t| t.category = PICKING_RENDERING)
            .component(Texture::from_size(texture_key, data.data.size))
            .with(|t| t.is_smooth = false)
            .with(|t| t.is_repeated = data.data.is_texture_repeated)
            .component(TextureBuffer::default())
            .with(|b| b.part = TextureBufferPart::Pixels(vec![]))
    }

    fn update_target(
        key: ResKey<RenderTarget>,
        data: &mut GenerationData<RenderTarget, TargetData>,
        resources: &mut TargetResourcesMut<'_, '_>,
    ) {
        if let Some(target) = resources.target(key) {
            let size = target.size();
            let is_texture_repeated = target.is_texture_repeated();
            if let Some(mut generated_target) = resources.target_mut(data.generated_key) {
                if let Some(texture) = &mut generated_target.texture {
                    data.data.is_texture_repeated = is_texture_repeated;
                    texture.is_repeated = is_texture_repeated;
                    if data.data.size != size {
                        data.data.size = size;
                        texture.set_source(TextureSource::Size(size));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct TargetData {
    size: Size,
    is_texture_repeated: bool,
    texture_key: Option<ResKey<Texture>>,
    generated_texture_key: ResKey<Texture>,
}

impl TargetData {
    fn new(target: &ConstTargetEntity<'_>) -> Self {
        Self {
            size: target.size(),
            is_texture_repeated: target.is_texture_repeated(),
            texture_key: target.texture.map(|texture| texture.key()),
            generated_texture_key: ResKey::unique("generated-texture(modor_picking)"),
        }
    }
}

#[derive(QuerySystemParam)]
struct TargetEntity<'a> {
    target: &'a RenderTarget,
    texture: Option<&'a mut Texture>,
    window: Option<&'a Window>,
    _filter: Filter<Or<(With<Texture>, With<Window>)>>,
}

impl ConstTargetEntity<'_> {
    fn size(&self) -> Size {
        self.window
            .map(|w| w.size())
            .or_else(|| self.texture.and_then(|t| t.size()))
            .unwrap_or(Size::ONE)
    }

    fn is_texture_repeated(&self) -> bool {
        self.texture.map_or(false, |texture| texture.is_repeated)
    }
}

#[derive(SystemParam)]
struct TargetResources<'a> {
    registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    query: Query<'a, Custom<TargetEntity<'static>>>,
}

impl<'a> TargetResources<'a> {
    fn as_mut<'b>(&'b mut self) -> TargetResourcesMut<'a, 'b> {
        TargetResourcesMut {
            registry: self.registry.get(),
            query: &mut self.query,
        }
    }
}

struct TargetResourcesMut<'a, 'b> {
    registry: &'b ResourceRegistry<RenderTarget>,
    query: &'b mut Query<'a, Custom<TargetEntity<'static>>>,
}

impl TargetResourcesMut<'_, '_> {
    fn target(&self, key: ResKey<RenderTarget>) -> Option<Custom<ConstTargetEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get(id)
    }

    fn target_mut(&mut self, key: ResKey<RenderTarget>) -> Option<Custom<TargetEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get_mut(id)
    }
}

mod generic {
    use fxhash::FxHashMap;
    use modor::{Component, World};
    use modor_resources::{ResKey, Resource, ResourceRegistry};
    use std::collections::hash_map::Entry;

    pub(crate) enum EntityState<'a, R, D> {
        New(&'a GenerationData<R, D>),
        Existing,
    }

    #[derive(Debug)]
    pub(crate) struct GenerationDataRegister<R, D> {
        data: FxHashMap<ResKey<R>, GenerationData<R, D>>,
    }

    impl<R, D> GenerationDataRegister<R, D>
    where
        R: Resource + Component,
    {
        pub(crate) fn iter_mut(
            &mut self,
        ) -> impl Iterator<Item = (ResKey<R>, &mut GenerationData<R, D>)> {
            self.data.iter_mut().map(|(&k, v)| (k, v))
        }

        pub(crate) fn reset(&mut self) {
            for data in self.data.values_mut() {
                data.is_found = false;
            }
        }

        pub(crate) fn register(&mut self, key: ResKey<R>, new_data: D) -> EntityState<'_, R, D> {
            match self.data.entry(key) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().update(new_data);
                    EntityState::Existing
                }
                Entry::Vacant(entry) => {
                    EntityState::New(entry.insert(GenerationData::new(new_data)))
                }
            }
        }

        pub(crate) fn remove_outdated(
            &mut self,
            registry: &ResourceRegistry<R>,
            world: &mut World<'_>,
        ) {
            for data in self.data.values_mut() {
                if let Some(id) = registry.entity_id(data.generated_key) {
                    world.delete_entity(id);
                }
            }
            self.data.retain(|_, data| data.is_found);
        }
    }

    #[derive(Debug)]
    pub(crate) struct GenerationData<R, D> {
        pub(crate) data: D,
        pub(crate) generated_key: ResKey<R>,
        is_found: bool,
    }

    impl<R, D> GenerationData<R, D>
    where
        R: Resource,
    {
        fn new(data: D) -> Self {
            Self {
                data,
                generated_key: ResKey::unique("generated-resource(modor_picking)"),
                is_found: true,
            }
        }

        fn update(&mut self, data: D) {
            self.data = data;
            self.is_found = true;
        }
    }
}
