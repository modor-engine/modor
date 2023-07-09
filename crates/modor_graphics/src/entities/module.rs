use crate::components::camera::Camera2DRegistry;
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::RenderTargetRegistry;
use crate::components::renderer::Renderer;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::{Size, Texture};
use modor::{BuiltEntity, EntityBuilder};

/// Creates the graphics module.
///
/// If this entity is not created, no rendering will be performed.
///
/// The created entity can be identified using the [`GraphicsModule`] component.
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
        .component(OpaqueInstanceRegistry::default())
        .component(TransparentInstanceRegistry::default())
        .component(RenderTargetRegistry::default())
        .component(Camera2DRegistry::default())
        .component(ShaderRegistry::default())
        .component(MeshRegistry::default())
        .component(MaterialRegistry::default())
        .component(TextureRegistry::default())
        .child_component(Shader::default())
        .child_component(Shader::ellipse())
        .child_component(Mesh::rectangle())
        .child_component(Texture::from_size(WHITE_TEXTURE, Size::ONE))
        .child_component(Texture::from_buffer(
            INVISIBLE_TEXTURE,
            Size::ONE,
            vec![0; 4],
        ))
}

/// The component that identifies the graphics module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct GraphicsModule;
