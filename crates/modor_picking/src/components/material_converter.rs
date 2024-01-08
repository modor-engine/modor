use crate::components::managed_materials::{
    ManagedMaterials, MaterialNotRegisteredDeletion, MaterialReset,
};
use crate::components::managed_targets::ManagedTargets;
use crate::data::ResState;
use derivative::Derivative;
use modor::{
    BuiltEntity, Component, ComponentSystems, Custom, EntityMut, Filter, Query, SingleMut,
    SingleRef, With,
};
use modor_graphics::{Material, MaterialSource, MaterialSync, Texture};
use modor_resources::{ResKey, Resource, ResourceRegistry};
use std::marker::PhantomData;

/// Register a material for picking.
///
/// `S` is the material type taken as source, and `D` the material type used to generate picking material.
///
/// Any rendering using a material not registered with this component will not be tracked by the picking module.
///
/// [`Default2DMaterial`](modor_graphics::Default2DMaterial) is supported by default by the picking module.
///
/// # Requirements
///
/// The material converter is effictive only if:
/// - picking [`module`](crate::module()) is initialized
#[derive(Debug, Derivative, Component)]
#[derivative(Default(bound = ""))]
pub struct PickingMaterialConverter<S: 'static, D: 'static> {
    phantom: PhantomData<fn(S, D)>,
}

#[systems]
impl<S, D> PickingMaterialConverter<S, D>
where
    S: MaterialSource,
    D: PickingMaterialSource<Source = S>,
{
    #[run_as(action(MaterialRegistration))]
    fn register_resources(
        mut entity: EntityMut<'_>,
        mut managed_materials: SingleMut<'_, '_, ManagedMaterials>,
        managed_targets: SingleRef<'_, '_, ManagedTargets>,
        mut materials: Custom<MaterialAccess<'_, S, D>>,
    ) {
        let managed_materials = managed_materials.get_mut();
        let managed_targets = managed_targets.get();
        let materials = materials.as_mut();
        for material in materials.src_query.iter() {
            if material.picking_material.is_some() {
                continue;
            }
            let src_key = material.material.key();
            if let ResState::New(key) = managed_materials.resources.register(src_key) {
                entity.create_child(Self::create_resource(key, &material, managed_targets));
            }
        }
    }

    #[run_after(action(MaterialNotRegisteredDeletion))]
    fn update_resources(
        mut managed_materials: SingleMut<'_, '_, ManagedMaterials>,
        managed_targets: SingleRef<'_, '_, ManagedTargets>,
        mut materials: Custom<MaterialAccess<'_, S, D>>,
    ) {
        let managed_targets = managed_targets.get();
        let mut materials = materials.as_mut();
        for (&key, managed_key) in managed_materials.get_mut().resources.iter() {
            Self::update_resource(key, managed_key, &mut materials, managed_targets);
        }
    }

    fn create_resource(
        key: ResKey<Material>,
        material: &ConstMaterialEntity<'_, S>,
        managed_targets: &ManagedTargets,
    ) -> impl BuiltEntity {
        modor_graphics::material(key, material.convert::<D>(managed_targets))
    }

    fn update_resource(
        key: ResKey<Material>,
        managed_key: ResKey<Material>,
        materials: &mut MaterialAccessMut<'_, '_, S, D>,
        managed_targets: &ManagedTargets,
    ) -> Option<()> {
        let material = materials.src_material(key)?;
        let new_material = material.convert(managed_targets);
        let mut managed_material = materials.dst_material_mut(managed_key)?;
        *managed_material.source = new_material;
        Some(())
    }
}

#[derive(Action)]
pub(crate) struct MaterialRegistration(
    MaterialReset,
    <ManagedTargets as ComponentSystems>::Action,
    <ResourceRegistry<Material> as ComponentSystems>::Action,
);

#[derive(modor::Component, NoSystem)]
struct PickingMaterial;

/// A trait for defining a picking material source used to render picking buffer.
///
/// The color of the instances rendered by this material must correspond to their entity ID. To convert an entity ID
/// to a color, you can follow these steps:
/// - Convert entity ID to `u32`.
/// - Transmute the ID into 4 `u8` values using [`bytemuck::cast_slice`].
/// - The obtained `u8` values correspond to the RGBA color to use for rendering.
pub trait PickingMaterialSource: MaterialSource {
    /// The material type used for standard rendering.
    type Source: MaterialSource;

    /// Converts the material used for standard rendering into material used for picking.
    ///
    /// In case the source material uses a texture corresponding to a render target, the key of the texture can be
    /// converted to the equivalent picking texture using `render_texture_converter`.
    fn convert(
        material: &Self::Source,
        render_texture_converter: impl Fn(ResKey<Texture>) -> Option<ResKey<Texture>>,
    ) -> Self;
}

#[allow(unused)]
#[derive(QuerySystemParam)]
struct MaterialEntity<'a, T: Component> {
    source: &'a mut T,
    material: &'a Material,
    picking_material: Option<&'a PickingMaterial>,
    _filter: Filter<With<MaterialSync<T>>>,
}

impl<T> ConstMaterialEntity<'_, T>
where
    T: MaterialSource,
{
    fn convert<D>(&self, managed_targets: &ManagedTargets) -> D
    where
        D: PickingMaterialSource<Source = T>,
    {
        D::convert(self.source, |texture_key| {
            managed_targets
                .textures
                .get(&texture_key)
                .map(|texture| texture.key)
        })
    }
}

#[derive(SystemParam)]
struct MaterialAccess<'a, S: Component, D: Component> {
    registry: SingleRef<'a, 'static, ResourceRegistry<Material>>,
    src_query: Query<'a, Custom<MaterialEntity<'static, S>>>,
    dst_query: Query<'a, Custom<MaterialEntity<'static, D>>>,
}

impl<'a, S, D> MaterialAccess<'a, S, D>
where
    S: MaterialSource,
    D: MaterialSource,
{
    fn as_mut<'b>(&'b mut self) -> MaterialAccessMut<'a, 'b, S, D> {
        MaterialAccessMut {
            registry: self.registry.get(),
            src_query: &self.src_query,
            dst_query: &mut self.dst_query,
        }
    }
}

struct MaterialAccessMut<'a, 'b, S: Component, D: Component> {
    registry: &'b ResourceRegistry<Material>,
    src_query: &'b Query<'a, Custom<MaterialEntity<'static, S>>>,
    dst_query: &'b mut Query<'a, Custom<MaterialEntity<'static, D>>>,
}

impl<S, D> MaterialAccessMut<'_, '_, S, D>
where
    S: MaterialSource,
    D: PickingMaterialSource<Source = S>,
{
    fn src_material(
        &mut self,
        key: ResKey<Material>,
    ) -> Option<Custom<ConstMaterialEntity<'_, S>>> {
        let id = self.registry.entity_id(key)?;
        self.src_query.get(id)
    }

    fn dst_material_mut(&mut self, key: ResKey<Material>) -> Option<Custom<MaterialEntity<'_, D>>> {
        let id = self.registry.entity_id(key)?;
        self.dst_query.get_mut(id)
    }
}
