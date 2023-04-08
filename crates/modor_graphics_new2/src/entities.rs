use crate::components::camera::Camera2DRegistry;
use crate::components::font::{FontKey, FontRegistry, DEFAULT_FONT_FILE};
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::RenderTargetRegistry;
use crate::components::renderer::Renderer;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureKey, TextureRegistry};
use crate::{Font, FontSource, Size, Texture, TextureSource};
use modor::{BuiltEntity, EntityBuilder};

pub fn renderer() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Renderer::new())
        .with_child(RenderTargetRegistry::default())
        .with_child(Camera2DRegistry::default())
        .with_child(ShaderRegistry::default())
        .with_child(MeshRegistry::default())
        .with_child(MaterialRegistry::default())
        .with_child(TextureRegistry::default())
        .with_child(FontRegistry::default())
        .with_child(OpaqueInstanceRegistry::default())
        .with_child(TransparentInstanceRegistry::default())
        .with_child(Shader::default())
        .with_child(Shader::ellipse())
        .with_child(Mesh::rectangle())
        .with_child(Texture::new(
            TextureKey::White,
            TextureSource::RgbaBuffer(vec![255; 4], Size::new(1, 1)),
        ))
        .with_child(Texture::new(
            TextureKey::Invisible,
            TextureSource::RgbaBuffer(vec![0; 4], Size::new(1, 1)),
        ))
        .with_child(Font::new(
            FontKey::Default,
            FontSource::File(DEFAULT_FONT_FILE),
        ))
}
