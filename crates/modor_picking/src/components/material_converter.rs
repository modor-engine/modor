use crate::components::picking::{PickingMaterialData, PickingUpdate};
use crate::Picking;
use modor::{Custom, EntityMut, Filter, Query, SingleMut, SingleRef, With};
use modor_graphics::{material, Material, MaterialSource, MaterialSync, Texture};
use modor_resources::{ResKey, ResourceRegistry};
use std::marker::PhantomData;

// TODO: move update logic in system_params module
// TODO: handle texture in materials ("nested" picking)

#[derive(Debug, Component)]
pub struct PickingMaterialConverter<S: 'static, D: 'static> {
    phantom: PhantomData<fn(S, D)>,
}

impl<S, D> Default for PickingMaterialConverter<S, D>
where
    S: 'static,
    D: 'static,
{
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

#[systems]
impl<S, D> PickingMaterialConverter<S, D>
where
    S: MaterialSource,
    D: PickingMaterialSource<Source = S>,
{
    #[run_as(action(PickingMaterialUpdate))]
    fn update_materials(
        mut picking: SingleMut<'_, '_, Picking>,
        mut resources: Custom<MaterialUpdateResource<'_, S, D>>,
    ) {
        let mut resources = resources.as_mut();
        let picking = picking.get_mut();
        let updated_materials =
            picking
                .materials
                .iter_mut()
                .filter_map(|(&material_key, picking_material)| {
                    Self::update_material(material_key, picking_material, &mut resources, |key| {
                        picking
                            .target_textures
                            .get(&key)
                            .map(|picking_texture| picking_texture.texture_key)
                    })
                });
        picking.updated_materials.extend(updated_materials);
    }

    fn update_material(
        material_key: ResKey<Material>,
        picking_material: &mut PickingMaterialData,
        resources: &mut MaterialUpdateResourceMut<'_, '_, S, D>,
        render_texture_converter: impl Fn(ResKey<Texture>) -> Option<ResKey<Texture>>,
    ) -> Option<ResKey<Material>> {
        let new_material = resources.convert_material(material_key, render_texture_converter)?;
        if let Some(mut dst_material) = resources.dst_material(picking_material.material_key) {
            *dst_material.material = new_material;
        } else {
            resources
                .entity
                .create_child(material(picking_material.material_key, new_material));
        }
        Some(picking_material.material_key)
    }
}

pub trait PickingMaterialSource: MaterialSource {
    type Source: MaterialSource;

    fn convert(
        material: &Self::Source,
        render_texture_converter: impl Fn(ResKey<Texture>) -> Option<ResKey<Texture>>,
    ) -> Self;
}

#[derive(Action)]
pub(crate) struct PickingMaterialUpdate(PickingUpdate);

#[derive(QuerySystemParam)]
struct MaterialEntity<'a, T>
where
    T: MaterialSource,
{
    material: &'a mut T,
    _filter: Filter<(With<Material>, With<MaterialSync<T>>)>,
}

#[derive(SystemParam)]
struct MaterialUpdateResource<'a, S, D>
where
    S: MaterialSource,
    D: MaterialSource,
{
    entity: EntityMut<'a>,
    material_registry: SingleRef<'a, 'static, ResourceRegistry<Material>>,
    src_materials: Query<'a, Custom<MaterialEntity<'static, S>>>,
    dst_materials: Query<'a, Custom<MaterialEntity<'static, D>>>,
}

impl<'a, S, D> MaterialUpdateResource<'a, S, D>
where
    S: MaterialSource,
    D: MaterialSource,
{
    fn as_mut<'b>(&'b mut self) -> MaterialUpdateResourceMut<'a, 'b, S, D> {
        MaterialUpdateResourceMut {
            entity: &mut self.entity,
            material_registry: self.material_registry.get(),
            src_materials: &self.src_materials,
            dst_materials: &mut self.dst_materials,
        }
    }
}

struct MaterialUpdateResourceMut<'a, 'b, S, D>
where
    S: MaterialSource,
    D: MaterialSource,
{
    entity: &'b mut EntityMut<'a>,
    material_registry: &'b ResourceRegistry<Material>,
    src_materials: &'b Query<'a, Custom<MaterialEntity<'static, S>>>,
    dst_materials: &'b mut Query<'a, Custom<MaterialEntity<'static, D>>>,
}

impl<S, D> MaterialUpdateResourceMut<'_, '_, S, D>
where
    S: MaterialSource,
    D: PickingMaterialSource<Source = S>,
{
    fn convert_material(
        &mut self,
        material_key: ResKey<Material>,
        render_texture_converter: impl Fn(ResKey<Texture>) -> Option<ResKey<Texture>>,
    ) -> Option<D> {
        let material_id = self.material_registry.entity_id(material_key)?;
        let material = self.src_materials.get(material_id)?;
        Some(D::convert(material.material, render_texture_converter))
    }

    fn dst_material(
        &mut self,
        material_key: ResKey<Material>,
    ) -> Option<Custom<MaterialEntity<'_, D>>> {
        let material_id = self.material_registry.entity_id(material_key)?;
        self.dst_materials.get_mut(material_id)
    }
}
