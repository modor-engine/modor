use crate::components::material_converter::PickingMaterialUpdate;
use crate::system_params::camera_update::CameraUpdateResource;
use crate::system_params::registration::{RegistrationResources, RegistrationResourcesRef};
use crate::system_params::rendering_update::{PickingRendering, RenderingUpdateResource};
use crate::system_params::target_update::TargetUpdateResource;
use crate::NoPicking;
use fxhash::{FxHashMap, FxHashSet};
use modor::{ComponentSystems, Custom, Filter, Not, Query, With};
use modor_graphics::{
    Camera2D, InstanceGroup2D, InstanceRendering2D, Material, Pixel, RenderTarget, Texture,
    TextureBuffer, MAIN_RENDERING,
};
use modor_resources::{ResKey, Resource, ResourceRegistry};

// TODO: delete outdated resources

#[derive(Debug, Default, SingletonComponent)]
pub struct Picking {
    pub(crate) materials: FxHashMap<ResKey<Material>, PickingMaterialData>,
    pub(crate) updated_materials: FxHashSet<ResKey<Material>>,
    pub(crate) target_textures: FxHashMap<ResKey<Texture>, PickingTargetTextureData>,
    targets: FxHashMap<ResKey<RenderTarget>, PickingTargetData>,
    cameras: FxHashMap<ResKey<Camera2D>, PickingCameraData>,
    renderings: Vec<PickingRenderingData>,
}

#[systems]
impl Picking {
    #[run_as(action(PickingRegistration))]
    fn register(
        &mut self,
        renderings: Query<'_, (&InstanceRendering2D, Filter<Not<With<NoPicking>>>)>,
        resources: Custom<RegistrationResources<'_>>,
    ) {
        let resources = resources.as_ref();
        self.renderings = renderings
            .iter()
            .filter_map(|(rendering, _)| self.register_rendering(rendering, resources))
            .collect();
        self.updated_materials.clear();
    }

    #[run_as(action(PickingUpdate))]
    fn update_targets(&self, mut resources: Custom<TargetUpdateResource<'_>>) {
        let mut resources = resources.as_mut();
        for (&key, data) in &self.targets {
            resources.update_target(key, data);
        }
    }

    #[run_as(action(PickingUpdate))]
    fn update_cameras(&self, mut resources: Custom<CameraUpdateResource<'_>>) {
        let mut resources = resources.as_mut();
        for (&key, data) in &self.cameras {
            resources.update_camera(key, data);
        }
    }

    #[run_after(action(PickingMaterialUpdate))]
    fn update_renderings(&self, mut resources: Custom<RenderingUpdateResource<'_>>) {
        let mut resources = resources.as_mut();
        for (index, data) in self.renderings.iter().enumerate() {
            if self.updated_materials.contains(&data.material_key) {
                resources.update_rendering(data, index);
            }
        }
    }

    pub fn picked_entity_id(
        &self,
        pixel: Pixel,
        target_key: ResKey<RenderTarget>,
        target_registry: &ResourceRegistry<RenderTarget>,
        target_buffers: &mut Query<'_, &mut TextureBuffer>,
    ) -> Option<usize> {
        let buffer = self.generated_buffer(target_key, target_registry, target_buffers)?;
        let color = buffer.pixel(pixel)?;
        let color_array: [u8; 4] = [
            (Self::srgb_to_rgb(color.r) * 255.).round() as u8,
            (Self::srgb_to_rgb(color.g) * 255.).round() as u8,
            (Self::srgb_to_rgb(color.b) * 255.).round() as u8,
            (color.a * 255.) as u8,
        ];
        let entity_id: &[u32] = bytemuck::cast_slice(&color_array);
        Some(entity_id[0] as usize)
    }

    pub fn generated_buffer<'a>(
        &self,
        target_key: ResKey<RenderTarget>,
        target_registry: &ResourceRegistry<RenderTarget>,
        target_buffers: &'a mut Query<'_, &mut TextureBuffer>,
    ) -> Option<&'a mut TextureBuffer> {
        let picking_target_key = self.targets.get(&target_key).map(|t| t.target_key)?;
        let id = target_registry.entity_id(picking_target_key)?;
        target_buffers.get_mut(id)
    }

    fn register_rendering(
        &mut self,
        rendering: &InstanceRendering2D,
        resources: RegistrationResourcesRef<'_, '_>,
    ) -> Option<PickingRenderingData> {
        let camera = resources.cameras.get(rendering.camera_key)?;
        let target_keys: Vec<_> = camera
            .target_keys
            .iter()
            .filter_map(|&target_key| self.register_target(target_key, resources))
            .collect();
        let camera_key = self.register_camera(rendering, target_keys)?;
        let material_key = self.register_material(rendering);
        Some(PickingRenderingData {
            group_key: rendering.group_key,
            camera_key,
            material_key,
        })
    }

    fn register_target(
        &mut self,
        key: ResKey<RenderTarget>,
        resources: RegistrationResourcesRef<'_, '_>,
    ) -> Option<ResKey<RenderTarget>> {
        let target_id = resources.target_registry.entity_id(key)?;
        let (target, texture, _) = resources.targets.get(target_id)?;
        (target.category == MAIN_RENDERING).then_some(())?;
        let picking_target = self
            .targets
            .entry(target.key())
            .or_insert_with(PickingTargetData::new);
        if let Some(texture_key) = texture.map(|texture| texture.key()) {
            self.target_textures.insert(
                texture_key,
                PickingTargetTextureData::new(picking_target.texture_key),
            );
        }
        Some(picking_target.target_key)
    }

    fn register_camera(
        &mut self,
        rendering: &InstanceRendering2D,
        target_keys: Vec<ResKey<RenderTarget>>,
    ) -> Option<ResKey<Camera2D>> {
        (!target_keys.is_empty()).then(|| {
            self.cameras
                .entry(rendering.camera_key)
                .or_insert_with(|| PickingCameraData::new(target_keys))
                .camera_key
        })
    }

    fn register_material(&mut self, rendering: &InstanceRendering2D) -> ResKey<Material> {
        self.materials
            .entry(rendering.material_key)
            .or_insert_with(|| PickingMaterialData::new())
            .material_key
    }

    fn srgb_to_rgb(component: f32) -> f32 {
        if component <= 0.04045 {
            component / 12.92
        } else {
            ((component + 0.055) / 1.055).powf(2.4)
        }
    }
}

#[derive(Action)]
struct PickingRegistration(
    <ResourceRegistry<Camera2D> as ComponentSystems>::Action,
    <ResourceRegistry<RenderTarget> as ComponentSystems>::Action,
    <ResourceRegistry<Material> as ComponentSystems>::Action,
    <ResourceRegistry<PickingRendering> as ComponentSystems>::Action,
);

#[derive(Action)]
pub(crate) struct PickingUpdate(PickingRegistration);

#[derive(Debug)]
pub(crate) struct PickingTargetData {
    pub(crate) target_key: ResKey<RenderTarget>,
    pub(crate) texture_key: ResKey<Texture>,
}

impl PickingTargetData {
    fn new() -> Self {
        Self {
            target_key: ResKey::unique("picking(modor_picking)"),
            texture_key: ResKey::unique("picking(modor_picking)"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PickingTargetTextureData {
    pub(crate) texture_key: ResKey<Texture>,
}

impl PickingTargetTextureData {
    fn new(texture_key: ResKey<Texture>) -> Self {
        Self { texture_key }
    }
}

#[derive(Debug)]
pub(crate) struct PickingCameraData {
    pub(crate) camera_key: ResKey<Camera2D>,
    pub(crate) target_keys: Vec<ResKey<RenderTarget>>,
}

impl PickingCameraData {
    fn new(target_keys: Vec<ResKey<RenderTarget>>) -> Self {
        Self {
            camera_key: ResKey::unique("picking(modor_picking)"),
            target_keys,
        }
    }
}

#[derive(Debug)]
pub(crate) struct PickingMaterialData {
    pub(crate) material_key: ResKey<Material>,
}

impl PickingMaterialData {
    fn new() -> Self {
        Self {
            material_key: ResKey::unique("picking(modor_picking)"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct PickingRenderingData {
    pub(crate) group_key: ResKey<InstanceGroup2D>,
    pub(crate) camera_key: ResKey<Camera2D>,
    pub(crate) material_key: ResKey<Material>,
}
