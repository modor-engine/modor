use crate::data::{ManagedResources, ResState};
use crate::NoPicking;
use fxhash::FxHashMap;
use modor::{
    BuiltEntity, Custom, EntityBuilder, EntityMut, Filter, Not, Or, Query, SingleRef, With,
};
use modor_graphics::{
    Color, RenderTarget, Size, Texture, TextureBuffer, TextureBufferPart, TextureSource, Window,
    MAIN_RENDERING,
};
use modor_resources::{ResKey, Resource, ResourceRegistry};

const PICKING_RENDERING: &str = "PICKING";

type TargetRegistry = ResourceRegistry<RenderTarget>;

// must not be accessed by end user
#[derive(Debug, Default, SingletonComponent)]
pub struct ManagedTargets {
    pub(crate) resources: ManagedResources<ResKey<RenderTarget>, RenderTarget>,
    pub(crate) textures: FxHashMap<ResKey<Texture>, ManagedTexture>,
}

#[systems]
impl ManagedTargets {
    #[run_after(component(TargetRegistry), component(Window), component(Texture))]
    fn update(&mut self, mut targets: Custom<TargetAccess<'_>>) {
        let mut targets = targets.as_mut();
        self.resources.reset();
        self.register_resources(&mut targets);
        self.resources
            .delete_not_registered(targets.registry, targets.entity.world());
        self.textures
            .retain(|_, texture| self.resources.contains(texture.src_target_key));
        for (&key, managed_key) in self.resources.iter() {
            if let Some((texture_key, managed_texture)) =
                Self::update_resource(key, managed_key, &mut targets)
            {
                self.textures.insert(texture_key, managed_texture);
            }
        }
    }

    fn register_resources(&mut self, targets: &mut TargetAccessMut<'_, '_>) {
        for target in targets.query.iter() {
            if target.target.category != MAIN_RENDERING {
                continue;
            }
            if let ResState::New(key) = self.resources.register(target.target.key()) {
                let new_texture_key = ResKey::unique("managed-texture(modor_picking)");
                if let Some(texture_key) = target.texture.map(Resource::key) {
                    let managed_texture = ManagedTexture::new(new_texture_key, target.target.key());
                    self.textures.insert(texture_key, managed_texture);
                }
                targets
                    .entity
                    .create_child(Self::create_resource(key, new_texture_key, &target));
            }
        }
    }

    fn create_resource(
        target_key: ResKey<RenderTarget>,
        texture_key: ResKey<Texture>,
        target: &ConstTargetEntity<'_>,
    ) -> impl BuiltEntity {
        let data = target.data();
        EntityBuilder::new()
            .component(RenderTarget::new(target_key))
            .with(|t| t.is_anti_aliasing_enabled = false)
            .with(|t| t.background_color = Color::WHITE)
            .with(|t| t.category = PICKING_RENDERING)
            .component(Texture::from_size(texture_key, data.size))
            .with(|t| t.is_smooth = false)
            .with(|t| t.is_repeated = data.is_texture_repeated)
            .component(TextureBuffer::default())
            .with(|b| b.part = TextureBufferPart::Pixels(vec![]))
    }

    fn update_resource(
        key: ResKey<RenderTarget>,
        managed_key: ResKey<RenderTarget>,
        targets: &mut TargetAccessMut<'_, '_>,
    ) -> Option<(ResKey<Texture>, ManagedTexture)> {
        let target = targets.target(key)?;
        let data = target.data();
        let target_key = target.target.key();
        let texture_key = target.texture.map(Resource::key);
        let mut managed_target = targets.target_mut(managed_key)?;
        let managed_texture = managed_target.texture.as_mut()?;
        let managed_texture_key = managed_texture.key();
        managed_texture.is_repeated = data.is_texture_repeated;
        if managed_texture.size() != Some(data.size) {
            managed_texture.set_source(TextureSource::Size(data.size));
        }
        Some((
            texture_key?,
            ManagedTexture::new(managed_texture_key, target_key),
        ))
    }
}

#[derive(Debug)]
pub(crate) struct ManagedTexture {
    pub(crate) key: ResKey<Texture>,
    src_target_key: ResKey<RenderTarget>,
}

impl ManagedTexture {
    fn new(key: ResKey<Texture>, src_target_key: ResKey<RenderTarget>) -> Self {
        Self {
            key,
            src_target_key,
        }
    }
}

struct TargetData {
    size: Size,
    is_texture_repeated: bool,
}

type TargetFilter = (Or<(With<Texture>, With<Window>)>, Not<With<NoPicking>>);

#[allow(unused)]
#[derive(QuerySystemParam)]
struct TargetEntity<'a> {
    target: &'a RenderTarget,
    texture: Option<&'a mut Texture>,
    window: Option<&'a Window>,
    _filter: Filter<TargetFilter>,
}

impl ConstTargetEntity<'_> {
    fn data(&self) -> TargetData {
        TargetData {
            size: self
                .window
                .map(Window::size)
                .or_else(|| self.texture.and_then(Texture::size))
                .unwrap_or(Size::ONE),
            is_texture_repeated: self.texture.map_or(false, |texture| texture.is_repeated),
        }
    }
}

#[derive(SystemParam)]
struct TargetAccess<'a> {
    entity: EntityMut<'a>,
    registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    query: Query<'a, Custom<TargetEntity<'static>>>,
}

impl<'a> TargetAccess<'a> {
    fn as_mut<'b>(&'b mut self) -> TargetAccessMut<'a, 'b> {
        TargetAccessMut {
            entity: &mut self.entity,
            registry: self.registry.get(),
            query: &mut self.query,
        }
    }
}

struct TargetAccessMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    registry: &'b ResourceRegistry<RenderTarget>,
    query: &'b mut Query<'a, Custom<TargetEntity<'static>>>,
}

impl TargetAccessMut<'_, '_> {
    fn target(&self, key: ResKey<RenderTarget>) -> Option<Custom<ConstTargetEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get(id)
    }

    fn target_mut(&mut self, key: ResKey<RenderTarget>) -> Option<Custom<TargetEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get_mut(id)
    }
}
