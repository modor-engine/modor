use crate::components::camera::Camera2DRegistry;
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::RenderTargetRegistry;
use crate::components::renderer::Renderer;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureKey, TextureRegistry};
use crate::{Size, Texture, TextureSource};
use modor::{BuiltEntity, EntityBuilder};

/// Creates graphics module.
///
/// If this entity is not created, no rendering will be performed.
///
/// # Platform-specific
///
/// - Android and web: next update will panic if the [`runner`](crate::runner()) is not used.
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(GraphicsModule)
        .with(Renderer::new())
        .with(OpaqueInstanceRegistry::default())
        .with(TransparentInstanceRegistry::default())
        .with(RenderTargetRegistry::default())
        .with(Camera2DRegistry::default())
        .with(ShaderRegistry::default())
        .with(MeshRegistry::default())
        .with(MaterialRegistry::default())
        .with(TextureRegistry::default())
        .with_child(Shader::default())
        .with_child(Shader::ellipse())
        .with_child(Mesh::rectangle())
        .with_child(Texture::new(
            TextureKey::White,
            TextureSource::RgbaBuffer(vec![255; 4], Size::ONE),
        ))
        .with_child(Texture::new(
            TextureKey::Invisible,
            TextureSource::RgbaBuffer(vec![0; 4], Size::ONE),
        ))
}

#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct GraphicsModule;
