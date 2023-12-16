use crate::components::material::{MaterialRegistry, MaterialUpdate};
use crate::components::mesh::{Mesh, RECTANGLE_MESH};
use crate::components::shader::Shader;
use crate::{Camera2D, InstanceGroup2D, Material};
use modor::Custom;
use modor_resources::{ResKey, ResourceAccessor};

/// The rendering of an [`InstanceGroup2D`].
///
/// # Requirements
///
/// Instances are rendered only if:
/// - graphics [`module`](crate::module()) is initialized
/// - instance entities linked to the [`InstanceGroup2D`] have
///      [`Transform2D`](modor_physics::Transform2D) component
///
/// # Related components
///
/// - [`InstanceGroup2D`](InstanceGroup2D)
/// - [`Camera2D`](Camera2D)
/// - [`Material`](Material)
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
///
/// # Examples
///
/// # Examples
///
/// See [`instance_group_2d`](crate::instance_group_2d()) and
/// [`instance_2d`](crate::instance_2d()) as most of the time these methods will be used
/// to create an instance rendering.
#[non_exhaustive]
#[derive(Component, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceRendering2D {
    // Field order is important to optimize rendering by avoiding resource rebinding when necessary.
    shader_key: Option<ResKey<Shader>>,
    /// Key of the [`InstanceGroup2D`] to render.
    pub group_key: ResKey<InstanceGroup2D>,
    pub(crate) mesh_key: ResKey<Mesh>,
    /// Key of the [`Camera2D`] used to render the instances.
    pub camera_key: ResKey<Camera2D>,
    /// Key of the [`Material`] used to render the instances.
    pub material_key: ResKey<Material>,
    pub(crate) is_transparent: bool,
}

#[systems]
impl InstanceRendering2D {
    /// Creates a new instance rendering.
    pub fn new(
        group_key: ResKey<InstanceGroup2D>,
        camera_key: ResKey<Camera2D>,
        material_key: ResKey<Material>,
    ) -> Self {
        Self {
            shader_key: None,
            group_key,
            mesh_key: RECTANGLE_MESH,
            camera_key,
            material_key,
            is_transparent: false,
        }
    }

    #[run_after(
        component(MaterialRegistry),
        component(Material),
        action(MaterialUpdate)
    )]
    fn update(&mut self, materials: Custom<ResourceAccessor<'_, Material>>) {
        let material = materials.get(self.material_key);
        self.shader_key = material.map(|material| material.shader_key);
        self.is_transparent = material.map_or(false, |material| material.is_transparent);
    }
}
