use crate::components::managed_cameras::ManagedCameras;
use crate::components::managed_materials::ManagedMaterials;
use crate::components::managed_renderings::{ManagedRenderings, PickingRendering};
use crate::components::managed_targets::ManagedTargets;
use crate::components::materials::{
    Default2DPickingMaterial, Default2DPickingMaterialInstanceData,
};
use crate::PickingMaterialConverter;
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics::{Default2DMaterial, GraphicsModule, Shader, Size, Texture};
use modor_resources::{ResKey, ResourceRegistry};

pub(crate) const DEFAULT_SHADER: ResKey<Shader> = ResKey::new("default(modor_picking)");
pub(crate) const ELLIPSE_SHADER: ResKey<Shader> = ResKey::new("ellipse(modor_picking)");
pub(crate) const BLANK_TEXTURE: ResKey<Texture> = ResKey::new("blank(modor_picking)");

/// Creates the color picking module.
///
/// The created entity can be identified using the [`PickingModule`] component.
///
/// # Dependencies
///
/// This module initializes automatically the [graphics module](modor_graphics::module()).
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(modor_picking::module());
/// ```
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(PickingModule)
        .component(ManagedTargets::default())
        .component(ManagedCameras::default())
        .component(ManagedMaterials::default())
        .component(ManagedRenderings::default())
        .component(ResourceRegistry::<PickingRendering>::default())
        .child_component(Texture::from_size(BLANK_TEXTURE, Size::ONE))
        .child_component(Shader::from_string::<Default2DPickingMaterialInstanceData>(
            DEFAULT_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")),
            true,
        ))
        .child_component(Shader::from_string::<Default2DPickingMaterialInstanceData>(
            ELLIPSE_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
            true,
        ))
        .child_component(PickingMaterialConverter::<
            Default2DMaterial,
            Default2DPickingMaterial,
        >::default())
        .dependency::<GraphicsModule, _, _>(modor_graphics::module)
}

/// The component that identifies the color picking module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct PickingModule;
