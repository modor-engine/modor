use fxhash::FxHashMap;
use modor::{
    BuiltEntity, Component, ComponentSystems, Custom, EntityBuilder, EntityMut, Filter, Or, Query,
    QuerySystemParam, QuerySystemParamWithLifetime, SingleRef, SystemParamWithLifetime,
    VariableSend, VariableSync, With,
};
use modor_graphics::{Color, RenderTarget, Size, Texture, TextureSource, Window, MAIN_RENDERING};
use modor_resources::{ResKey, Resource, ResourceRegistry};
use std::any::Any;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::marker::PhantomData;

/*
- targets: all targets with MAIN_RENDERING category without NoPicking component
- textures: all textures with target with MAIN_RENDERING category
- cameras: all cameras without PickingCamera component
- materials: all materials having a MaterialConverter
- renderings: all renderings with registered target, camera and material and without NoPicking component
 */

const PICKING_RENDERING: &str = "PICKING";
const WINDOW_INTERNAL_TEXTURE_KEY: ResKey<Texture> = ResKey::new("window_texture(modor_picking)");

type TargetEntityFilter = Filter<Or<(With<Texture>, With<Window>)>>;

#[derive(Debug, Default, SingletonComponent)]
pub(crate) struct ResourceSync<R: 'static + SynchronizedResource> {
    new_resources: FxHashMap<ResKey<R>, TrackedGenerationData<R>>,
    phantom: PhantomData<fn(R)>,
}

#[systems]
impl<R> ResourceSync<R>
where
    R: SynchronizedResource + ComponentSystems,
{
    #[run_after(component(R::Dependency))]
    fn update(
        &mut self,
        mut entity: EntityMut<'_>,
        mut resources: Query<'_, R::Query>,
        registry: SingleRef<'_, '_, ResourceRegistry<R>>,
        primary_registry: SingleRef<'_, '_, ResourceRegistry<R::PrimaryResource>>,
    ) {
        let registry = registry.get();
        let primary_registry = primary_registry.get();
        self.reset();
        for res in resources.iter() {
            if R::is_kept(&res) {
                let new_data = R::generation_data(&res);
                match self.new_resources.entry(R::extract_key(&res)) {
                    Entry::Occupied(mut entry) => entry.get_mut().update(new_data),
                    Entry::Vacant(entry) => {
                        let data = entry.insert(TrackedGenerationData::new(new_data));
                        let _ = Self::create_resource(&res, data, primary_registry, &mut entity);
                    }
                }
            }
        }
        for data in self.new_resources.values() {
            Self::update_resource(data, registry, &mut resources, &mut entity);
        }
        self.new_resources.retain(|_, data| data.is_found);
    }

    pub(crate) fn generated_key(&self, source_key: ResKey<R>) -> Option<ResKey<R>> {
        self.new_resources.get(&source_key).map(|data| data.key)
    }

    fn reset(&mut self) {
        for resource in self.new_resources.values_mut() {
            resource.is_found = false;
        }
    }

    fn create_resource(
        res: &ConstParam<'_, R::Query>,
        data: &mut TrackedGenerationData<R>,
        primary_registry: &ResourceRegistry<R::PrimaryResource>,
        entity: &mut EntityMut<'_>,
    ) -> Option<()> {
        if let Some(generated_entity) = R::create_entity(data.key, &data.data) {
            entity.create_child(generated_entity);
        } else {
            let key = R::extract_primary_key(&res)?;
            let component = R::create_component(data.key, &data.data)?;
            let id = primary_registry.entity_id(key)?;
            entity.world().add_component(id, component);
        }
        Some(())
    }

    fn update_resource(
        data: &TrackedGenerationData<R>,
        registry: &ResourceRegistry<R>,
        resources: &mut Query<'_, R::Query>,
        entity: &mut EntityMut<'_>,
    ) {
        if let Some(id) = registry.entity_id(data.key) {
            if data.is_found {
                if let Some(resource) = resources.get_mut(id) {
                    R::update(resource, &data.data);
                }
            } else {
                entity.world().delete_entity(id);
            }
        }
    }
}

#[derive(Debug)]
struct TrackedGenerationData<R: SynchronizedResource> {
    data: R::GenerationData,
    key: ResKey<R>,
    is_found: bool,
}

impl<R> TrackedGenerationData<R>
where
    R: SynchronizedResource,
{
    fn new(data: R::GenerationData) -> Self {
        Self {
            data,
            key: ResKey::unique("picking(modor_picking)"),
            is_found: true,
        }
    }

    fn update(&mut self, data: R::GenerationData) {
        self.data = data;
        self.is_found = true;
    }
}

type ConstParam<'a, Q> =
    <<Q as QuerySystemParamWithLifetime<'a>>::ConstParam as SystemParamWithLifetime<'a>>::Param;

pub(crate) trait SynchronizedResource: Resource {
    type Query: QuerySystemParam;
    type GenerationData: 'static + Sync + Send + Debug;
    type PrimaryResource: Resource + Component;
    type Dependency: ComponentSystems;

    fn is_kept<'a>(resource: &ConstParam<'_, Self::Query>) -> bool;

    fn generation_data<'a>(resource: &ConstParam<'_, Self::Query>) -> Self::GenerationData;

    fn extract_key<'a>(resource: &ConstParam<'_, Self::Query>) -> ResKey<Self>;

    fn create_entity(
        generated_key: ResKey<Self>,
        data: &Self::GenerationData,
    ) -> Option<impl BuiltEntity + Any + VariableSync + VariableSend>;

    fn extract_primary_key<'a>(
        resource: &ConstParam<'_, Self::Query>,
    ) -> Option<ResKey<Self::PrimaryResource>>;

    fn create_component(generated_key: ResKey<Self>, data: &Self::GenerationData) -> Option<Self>;

    fn update(
        resource: <Self::Query as SystemParamWithLifetime<'_>>::Param,
        data: &Self::GenerationData,
    );
}

impl SynchronizedResource for RenderTarget {
    type Query = (&'static mut Self, TargetEntityFilter);
    type GenerationData = ();
    type PrimaryResource = Self;
    type Dependency = ResourceRegistry<Self>;

    fn is_kept<'a>(resource: &ConstParam<'_, Self::Query>) -> bool {
        resource.0.category == MAIN_RENDERING
    }

    fn generation_data<'a>(_resource: &ConstParam<'_, Self::Query>) -> Self::GenerationData {
        ()
    }

    fn extract_key<'a>(resource: &ConstParam<'_, Self::Query>) -> ResKey<Self> {
        resource.0.key()
    }

    fn create_entity(
        generated_key: ResKey<Self>,
        _data: &Self::GenerationData,
    ) -> Option<impl BuiltEntity + Any + VariableSync + VariableSend> {
        Some(
            EntityBuilder::new()
                .component(RenderTarget::new(generated_key))
                .with(|t| t.is_anti_aliasing_enabled = false)
                .with(|t| t.background_color = Color::WHITE)
                .with(|t| t.category = PICKING_RENDERING),
        )
    }

    fn extract_primary_key<'a>(
        _resource: &ConstParam<'_, Self::Query>,
    ) -> Option<ResKey<Self::PrimaryResource>> {
        None
    }

    fn create_component(
        _generated_key: ResKey<Self>,
        _data: &Self::GenerationData,
    ) -> Option<Self> {
        None
    }

    fn update(
        _resource: <Self::Query as SystemParamWithLifetime<'_>>::Param,
        _data: &Self::GenerationData,
    ) {
        // nothing to update
    }
}

impl SynchronizedResource for Texture {
    type Query = Custom<TargetEntity<'static>>;
    type GenerationData = Size;
    type PrimaryResource = RenderTarget;
    type Dependency = ResourceRegistry<Texture>;

    fn is_kept<'a>(resource: &ConstParam<'_, Self::Query>) -> bool {
        resource.target.category == MAIN_RENDERING
    }

    fn generation_data<'a>(resource: &ConstParam<'_, Self::Query>) -> Self::GenerationData {
        resource
            .window
            .map(|w| w.size())
            .or_else(|| resource.texture.and_then(|t| t.size()))
            .unwrap_or(Size::ZERO)
    }

    fn extract_key<'a>(resource: &ConstParam<'_, Self::Query>) -> ResKey<Self> {
        resource
            .texture
            .map(|texture| texture.key())
            .unwrap_or(WINDOW_INTERNAL_TEXTURE_KEY)
    }

    fn create_entity(
        _generated_key: ResKey<Self>,
        _data: &Self::GenerationData,
    ) -> Option<EntityBuilder> {
        None
    }

    fn extract_primary_key<'a>(
        resource: &ConstParam<'_, Self::Query>,
    ) -> Option<ResKey<Self::PrimaryResource>> {
        Some(resource.target.key())
    }

    fn create_component(generated_key: ResKey<Self>, data: &Self::GenerationData) -> Option<Self> {
        Some(Texture::from_size(generated_key, *data))
    }

    fn update(
        mut resource: <Self::Query as SystemParamWithLifetime<'_>>::Param,
        data: &Self::GenerationData,
    ) {
        let texture = resource
            .texture
            .as_mut()
            .expect("internal error: missing target texture");
        if texture.size() != Some(*data) {
            texture.set_source(TextureSource::Size(*data));
        }
    }
}

#[allow(unused)]
#[derive(modor::QuerySystemParam)]
pub(crate) struct TargetEntity<'a> {
    target: &'a RenderTarget,
    texture: Option<&'a mut Texture>,
    window: Option<&'a Window>,
    _filter: TargetEntityFilter,
}
