use crate::components::camera::Camera2DRegistry;
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::RenderTargetRegistry;
use crate::components::renderer::Renderer;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::TextureRegistry;
use crate::Texture;
use modor::{BuiltEntity, EntityBuilder};

pub fn renderer() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Renderer::new())
        .with_child(RenderTargetRegistry::new())
        .with_child(Camera2DRegistry::new())
        .with_child(ShaderRegistry::new())
        .with_child(MeshRegistry::new())
        .with_child(MaterialRegistry::new())
        .with_child(TextureRegistry::new())
        .with_child(OpaqueInstanceRegistry::default())
        .with_child(TransparentInstanceRegistry::default())
        .with_child(Shader::default())
        .with_child(Shader::ellipse())
        .with_child(Mesh::rectangle())
        .with_child(Texture::blank())
}
