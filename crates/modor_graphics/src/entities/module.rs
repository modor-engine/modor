use crate::components::camera::Camera2DRegistry;
use crate::components::instance_group::InstanceGroup2DRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::RenderTargetRegistry;
use crate::components::renderer::Renderer;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::{Size, Texture};
use modor::{BuiltEntity, EntityBuilder};
use modor_input::InputModule;
use modor_physics::PhysicsModule;
use modor_resources::ResKey;

pub(crate) const DEFAULT_SHADER: ResKey<Shader> = ResKey::new("default(modor_graphics)");
pub(crate) const ELLIPSE_SHADER: ResKey<Shader> = ResKey::new("ellipse(modor_graphics)");

/// Creates the graphics module.
///
/// If this entity is not created, no rendering will be performed.
///
/// The created entity can be identified using the [`GraphicsModule`] component.
///
/// # Dependencies
///
/// This module initializes automatically the [input module](modor_input::module())
/// and [physics module](modor_physics::module()).
///
/// # Platform-specific
///
/// - Android and web: next update will panic if the graphics [`runner`](crate::runner()) is not
/// used.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(modor_graphics::module());
/// ```
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(GraphicsModule)
        .component(Renderer::new())
        .component(RenderTargetRegistry::default())
        .component(Camera2DRegistry::default())
        .component(ShaderRegistry::default())
        .component(MeshRegistry::default())
        .component(MaterialRegistry::default())
        .component(TextureRegistry::default())
        .component(InstanceGroup2DRegistry::default())
        .child_component(Shader::from_string(
            DEFAULT_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")),
        ))
        .child_component(Shader::from_string(
            ELLIPSE_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
        ))
        .child_component(Mesh::rectangle())
        .child_component(Texture::from_size(WHITE_TEXTURE, Size::ONE))
        .child_component(Texture::from_buffer(
            INVISIBLE_TEXTURE,
            Size::ONE,
            vec![0; 4],
        ))
        .dependency::<PhysicsModule, _, _>(modor_physics::module)
        .dependency::<InputModule, _, _>(modor_input::module)
}

/// The component that identifies the graphics module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct GraphicsModule;
