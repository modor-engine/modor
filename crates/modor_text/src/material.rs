use crate::material::internal::TextMaterial2DData;
use crate::resources::TextResources;
use modor::{App, Glob, GlobRef};
use modor_graphics::{Color, Material, Model2DGlob, ShaderGlobRef, TextureGlob};

/// A material for 2D text rendering.
///
/// # Examples
///
/// See [`Text2D`](crate::Text2D).
#[derive(Debug)]
pub struct TextMaterial2D {
    // The color of the rendered text.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    texture: GlobRef<TextureGlob>,
    shader: ShaderGlobRef<Self>,
}

impl Material for TextMaterial2D {
    type Data = TextMaterial2DData;
    type InstanceData = ();

    fn shader(&self) -> ShaderGlobRef<Self> {
        self.shader.clone()
    }

    fn textures(&self) -> Vec<GlobRef<TextureGlob>> {
        vec![self.texture.clone()]
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        TextMaterial2DData {
            color: self.color.into(),
        }
    }

    fn instance_data(_app: &mut App, _model: &Glob<Model2DGlob>) -> Self::InstanceData {}
}

impl TextMaterial2D {
    pub(crate) fn new(app: &mut App, texture: GlobRef<TextureGlob>) -> Self {
        let resources = app.get_mut::<TextResources>();
        Self {
            color: Color::WHITE,
            texture,
            shader: resources.text_shader.glob(),
        }
    }
}

pub(super) mod internal {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct TextMaterial2DData {
        pub(crate) color: [f32; 4],
    }
}
